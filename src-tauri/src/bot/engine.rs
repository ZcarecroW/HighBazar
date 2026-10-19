//! The running bot session: reacts to chat and Twitch events, schedules
//! companion messages, runs moderation and sends messages.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use parking_lot::{Mutex, RwLock};
use rand::RngExt;
use rand::seq::SliceRandom;
use serde_json::json;
use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc, watch};

use super::context::{ChatContext, Line, SeenChatters};
use super::moderation::{self, Finding, ModLogEntry, RuleState};
use super::research::{self, Knowledge};
use super::translate::{self, Foreign, Translation};
use super::{
    BotStatus, Control, EVT_FEED, EVT_MODLOG, EVT_PENDING, EVT_STATUS, EVT_TRANSLATION, FeedItem, ManualAction, PendingMessage,
    SimMessage, output, prompt,
};
use crate::ai::{self, AiRequest};
use crate::config::{AiProfile, ChannelProfile, CompanionProfile, ModAction, ModerationProfile, new_id};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::storage;
use crate::twitch::emotes::{self, EmoteCatalog};
use crate::twitch::eventsub::{self, ChatMessage, ChatNotification, TwitchEvent, role_requirement};

/// Well-known bot accounts the companion never reacts to.
const KNOWN_BOTS: &[&str] = &[
    "nightbot", "streamelements", "streamlabs", "moobot", "fossabot", "wizebot", "sery_bot", "soundalerts",
    "kofistreambot", "botrixoficial", "blerp", "commanderroot", "pokemoncommunitygame", "streamstickers",
    "own3d", "frostytoolsdotcom", "lurxx", "tangiabot", "creatisbot", "deepbot", "coebot", "phantombot",
];

const MAX_INFLIGHT: usize = 3;

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

struct Profiles {
    ch: ChannelProfile,
    comp: CompanionProfile,
    modp: ModerationProfile,
    ai: Option<AiProfile>,
    mod_ai: Option<AiProfile>,
}

#[derive(Default, Clone)]
struct StreamState {
    live: bool,
    title: String,
    game: String,
    language: String,
    viewers: u64,
    started_at: Option<chrono::DateTime<chrono::Utc>>,
}

enum Task {
    Interval(String),
    Mention(ChatMessage, bool),
    Welcome(ChatMessage),
    Event(String, String),
    Command(String, ChatMessage, String),
}

impl Task {
    fn label(&self) -> &'static str {
        match self {
            Task::Interval(_) => "timed",
            Task::Mention(..) => "reply",
            Task::Welcome(_) => "welcome",
            Task::Event(..) => "event",
            Task::Command(..) => "command",
        }
    }
}

struct Outgoing {
    text: String,
    reason: String,
}

/// A message for AI moderation; foreign ones are translated in the same call.
struct ModJob {
    m: ChatMessage,
    lang: Foreign,
}

pub struct Session {
    app: AppHandle,
    st: Arc<AppState>,
    channel_id: String,
    sandbox: bool,
    broadcaster_id: String,
    broadcaster_login: String,
    broadcaster_name: String,
    sender_id: String,
    sender_login: String,
    via_bot: bool,

    ctx: Mutex<ChatContext>,
    stream: Mutex<StreamState>,
    emotes: RwLock<EmoteCatalog>,
    knowledge: Mutex<Option<Knowledge>>,
    researching: AtomicBool,
    researched_game: Mutex<String>,
    seen: Mutex<SeenChatters>,
    seen_dirty: AtomicBool,
    stream_seen: Mutex<HashSet<String>>,
    cooldowns: Mutex<HashMap<String, i64>>,
    welcomes: Mutex<VecDeque<i64>>,
    strikes: Mutex<HashMap<String, Vec<i64>>>,
    rules: Mutex<RuleState>,
    pending: Mutex<Vec<PendingMessage>>,
    modlog: Mutex<Vec<ModLogEntry>>,
    follows: Mutex<(Vec<String>, i64)>,
    next_interval: Mutex<i64>,
    paused_reason: Mutex<String>,
    connected: AtomicBool,
    last_error: Mutex<String>,
    /// Limits parallel AI calls; further tasks wait for a free slot.
    inflight: tokio::sync::Semaphore,
    budget_warned: AtomicBool,
    no_ai_warned: AtomicBool,
    mod_rate_warned: Mutex<i64>,
    out_tx: mpsc::UnboundedSender<Outgoing>,
    out_rx: Mutex<Option<mpsc::UnboundedReceiver<Outgoing>>>,
    mod_tx: mpsc::UnboundedSender<ModJob>,
    mod_rx: Mutex<Option<mpsc::UnboundedReceiver<ModJob>>>,
    tr_tx: mpsc::UnboundedSender<ChatMessage>,
    tr_rx: Mutex<Option<mpsc::UnboundedReceiver<ChatMessage>>>,
}

impl Session {
    pub async fn create(app: AppHandle, st: Arc<AppState>, channel_id: &str, sandbox: bool) -> AppResult<Arc<Self>> {
        let (ch, broadcaster, bot) = {
            let cfg = st.config.read();
            let ch = cfg
                .channel(channel_id)
                .cloned()
                .ok_or_else(|| AppError::msg("Channel profile not found."))?;
            let broadcaster = ch.streamer_account_id.as_deref().and_then(|id| cfg.account(id)).cloned();
            let bot = ch.bot_account_id.as_deref().and_then(|id| cfg.account(id)).cloned();
            (ch, broadcaster, bot)
        };
        if !sandbox {
            if st.config.read().twitch_client_id().is_empty() {
                return Err(AppError::msg("No Twitch Client ID configured (Settings → Advanced)."));
            }
            let b = broadcaster
                .as_ref()
                .ok_or_else(|| AppError::msg("Connect the streamer's Twitch account in Channels first."))?;
            if !st.tokens.has(&b.id) {
                return Err(AppError::msg("The streamer account needs to be reconnected (Channels)."));
            }
            if ch.ai_profile_id.is_empty() && (ch.companion_enabled || ch.moderation_enabled) {
                // Not fatal: local moderation rules still work without AI.
                log::warn!("no AI profile selected");
            }
        }
        let (b_id, b_login, b_name) = broadcaster
            .as_ref()
            .map(|b| (b.id.clone(), b.login.clone(), b.display_name.clone()))
            .unwrap_or_else(|| ("sandbox".into(), "sandbox".into(), ch.name.clone()));
        let (sender_id, sender_login, via_bot) = match &bot {
            Some(b) => (b.id.clone(), b.login.clone(), true),
            None => (b_id.clone(), b_login.clone(), false),
        };
        let seen_path = st.data_dir.join("channels").join(&b_id).join("seen.json");
        let mut seen: SeenChatters = storage::read_json(&seen_path).unwrap_or_default();
        if seen.since == 0 {
            seen.since = chrono::Utc::now().timestamp();
        }
        let (out_tx, out_rx) = mpsc::unbounded_channel();
        let (mod_tx, mod_rx) = mpsc::unbounded_channel();
        let (tr_tx, tr_rx) = mpsc::unbounded_channel();
        Ok(Arc::new(Self {
            app,
            st,
            channel_id: channel_id.to_string(),
            sandbox,
            broadcaster_id: b_id,
            broadcaster_login: b_login,
            broadcaster_name: b_name,
            sender_id,
            sender_login,
            via_bot,
            ctx: Mutex::new(ChatContext::default()),
            stream: Mutex::new(StreamState::default()),
            emotes: RwLock::new(EmoteCatalog::default()),
            knowledge: Mutex::new(None),
            researching: AtomicBool::new(false),
            researched_game: Mutex::new(String::new()),
            seen: Mutex::new(seen),
            seen_dirty: AtomicBool::new(true),
            stream_seen: Mutex::new(HashSet::new()),
            cooldowns: Mutex::new(HashMap::new()),
            welcomes: Mutex::new(VecDeque::new()),
            strikes: Mutex::new(HashMap::new()),
            rules: Mutex::new(RuleState::default()),
            pending: Mutex::new(Vec::new()),
            modlog: Mutex::new(Vec::new()),
            follows: Mutex::new((Vec::new(), 0)),
            next_interval: Mutex::new(0),
            paused_reason: Mutex::new(String::new()),
            connected: AtomicBool::new(false),
            last_error: Mutex::new(String::new()),
            inflight: tokio::sync::Semaphore::new(MAX_INFLIGHT),
            budget_warned: AtomicBool::new(false),
            no_ai_warned: AtomicBool::new(false),
            mod_rate_warned: Mutex::new(0),
            out_tx,
            out_rx: Mutex::new(Some(out_rx)),
            mod_tx,
            mod_rx: Mutex::new(Some(mod_rx)),
            tr_tx,
            tr_rx: Mutex::new(Some(tr_rx)),
        }))
    }

    fn has_twitch(&self) -> bool {
        self.broadcaster_id != "sandbox"
    }

    fn profiles(&self) -> Profiles {
        let cfg = self.st.config.read();
        let ch = cfg.channel(&self.channel_id).cloned().unwrap_or_default();
        let comp = cfg.companion(&ch.companion_profile_id);
        let modp = cfg.moderation(&ch.moderation_profile_id);
        let ai = cfg.ai_profile(&ch.ai_profile_id);
        let mod_ai = if ch.moderation_ai_profile_id.is_empty() {
            ai.clone()
        } else {
            cfg.ai_profile(&ch.moderation_ai_profile_id).or_else(|| ai.clone())
        };
        Profiles { ch, comp, modp, ai, mod_ai }
    }

    // ------------------------------------------------------------------
    // Feed / status
    // ------------------------------------------------------------------

    fn feed(&self, item: FeedItem) {
        let _ = self.app.emit(EVT_FEED, item);
    }

    fn error(&self, text: impl Into<String>) {
        let t = text.into();
        *self.last_error.lock() = t.clone();
        self.feed(FeedItem::new("error", t));
    }

    /// Feed entry the UI translates via `feed.<key>`; `text` is the English
    /// fallback.
    fn keyed(&self, kind: &str, key: &str, params: serde_json::Value, text: String) {
        if kind == "error" {
            *self.last_error.lock() = text.clone();
        }
        let mut item = FeedItem::new(kind, text);
        item.meta = json!({ "key": key, "params": params });
        self.feed(item);
    }

    fn emit_pending(&self) {
        let list = self.pending.lock().clone();
        let _ = self.app.emit(EVT_PENDING, list);
    }

    fn status(&self) -> BotStatus {
        let p = self.profiles();
        let usage = self.st.usage.lock().today_for(&self.channel_id);
        // A copy, so no lock is held while the others are taken.
        let stream = self.stream.lock().clone();
        let next = *self.next_interval.lock();
        let rate = self.ctx.lock().rate_per_min(now_ms());
        BotStatus {
            running: true,
            sandbox: self.sandbox,
            connected: self.sandbox || self.connected.load(Ordering::Relaxed),
            channel_id: self.channel_id.clone(),
            channel_login: self.broadcaster_login.clone(),
            channel_name: self.broadcaster_name.clone(),
            sender_login: self.sender_login.clone(),
            live: stream.live,
            game: stream.game.clone(),
            title: stream.title.clone(),
            language: stream.language.clone(),
            viewers: stream.viewers,
            started_at: stream.started_at.map(|d| d.to_rfc3339()),
            next_message_at: (next > 0 && p.ch.companion_enabled && p.comp.interval_enabled).then_some(next),
            paused_reason: self.paused_reason.lock().clone(),
            chat_rate: rate,
            emote_count: self.emotes.read().total(),
            knowledge_items: self.knowledge.lock().as_ref().map(|k| k.facts.len() + k.tips.len()).unwrap_or(0),
            researching: self.researching.load(Ordering::Relaxed),
            cost_today: usage.cost_usd,
            budget: p.ch.daily_budget_usd,
            messages_today: usage.messages_sent,
            last_error: self.last_error.lock().clone(),
        }
    }

    fn emit_status(&self) {
        let _ = self.app.emit(EVT_STATUS, self.status());
    }

    // ------------------------------------------------------------------
    // AI helpers
    // ------------------------------------------------------------------

    fn budget_ok(&self, p: &Profiles) -> bool {
        if p.ch.daily_budget_usd <= 0.0 {
            return true;
        }
        let spent = self.st.usage.lock().today_for(&self.channel_id).cost_usd;
        if spent < p.ch.daily_budget_usd {
            return true;
        }
        if !self.budget_warned.swap(true, Ordering::Relaxed) {
            self.keyed(
                "error",
                "budget",
                json!({ "amount": format!("{:.2}", p.ch.daily_budget_usd) }),
                format!(
                    "Daily AI budget of ${:.2} reached — AI features pause until tomorrow (local moderation rules keep working).",
                    p.ch.daily_budget_usd
                ),
            );
        }
        false
    }

    async fn ai_call(&self, profile: &AiProfile, req: AiRequest) -> AppResult<String> {
        let key = self.st.ai_key(profile);
        let resp = ai::complete(&self.st.http, profile, key.as_deref(), &req).await?;
        let cost = resp.cost(profile);
        self.st
            .usage
            .lock()
            .add_ai(&self.channel_id, resp.input_tokens, resp.output_tokens, cost);
        Ok(resp.text)
    }

    // ------------------------------------------------------------------
    // Stream info, emotes, research
    // ------------------------------------------------------------------

    async fn refresh_stream(self: &Arc<Self>) {
        if !self.has_twitch() {
            return;
        }
        let helix = self.st.helix();
        let b = self.broadcaster_id.clone();
        let (channel, stream) = tokio::join!(helix.channel(&b, &b), helix.stream(&b, &b));
        let mut game_changed = None;
        {
            let mut s = self.stream.lock();
            if let Ok(Some(c)) = channel {
                if c.game_name != s.game {
                    game_changed = Some(c.game_name.clone());
                }
                s.game = c.game_name;
                s.title = c.title;
                s.language = c.broadcaster_language;
            }
            match stream {
                Ok(Some(info)) => {
                    s.live = true;
                    s.viewers = info.viewer_count;
                    s.started_at = chrono::DateTime::parse_from_rfc3339(&info.started_at)
                        .ok()
                        .map(|d| d.with_timezone(&chrono::Utc));
                }
                Ok(None) => {
                    s.live = false;
                    s.viewers = 0;
                    s.started_at = None;
                }
                Err(e) => log::warn!("stream info failed: {e}"),
            }
        }
        if let Some(g) = game_changed {
            self.maybe_research(g);
        }
        self.emit_status();
    }

    async fn refresh_emotes(self: &Arc<Self>) {
        if !self.has_twitch() {
            return;
        }
        let cat = emotes::fetch(&self.st.helix(), &self.broadcaster_id).await;
        let n = cat.total();
        let urls = Arc::new(cat.urls.clone());
        *self.emotes.write() = cat;
        *self.st.emote_urls.write() = urls.clone();
        let _ = self.app.emit(super::EVT_EMOTES, &*urls);
        log::info!("loaded {n} emotes");
        self.emit_status();
    }

    fn maybe_research(self: &Arc<Self>, game: String) {
        let p = self.profiles();
        if !p.comp.research.enabled || research::is_non_game(&game) {
            *self.knowledge.lock() = None;
            return;
        }
        if *self.researched_game.lock() == game && self.knowledge.lock().is_some() {
            return;
        }
        if let Some(k) = research::load_cached(&self.st.data_dir, &game, p.comp.research.refresh_days) {
            *self.knowledge.lock() = Some(k);
            *self.researched_game.lock() = game;
            self.emit_status();
            return;
        }
        let Some(ai_profile) = p.ai.clone() else {
            return;
        };
        if !self.budget_ok(&p) || self.researching.swap(true, Ordering::SeqCst) {
            return;
        }
        let me = self.clone();
        tokio::spawn(async move {
            me.emit_status();
            let key = me.st.ai_key(&ai_profile);
            let res = research::research(
                &me.st.http,
                &me.st.data_dir,
                &game,
                &ai_profile,
                key.as_deref(),
                p.comp.research.web_search,
                p.comp.research.wikipedia,
            )
            .await;
            match res {
                Ok((k, cost)) => {
                    me.st.usage.lock().add_ai(&me.channel_id, 0, 0, cost);
                    me.keyed("system", "researched", json!({ "game": game, "facts": k.facts.len(), "tips": k.tips.len() }), format!("Researched “{game}”: {} facts and {} tips ready.", k.facts.len(), k.tips.len()));
                    *me.knowledge.lock() = Some(k);
                    *me.researched_game.lock() = game;
                }
                Err(e) => me.keyed("error", "researchFailed", json!({ "game": game, "error": e }), format!("Research for “{game}” failed: {e}")),
            }
            me.researching.store(false, Ordering::SeqCst);
            me.emit_status();
        });
    }

    // ------------------------------------------------------------------
    // Event handling
    // ------------------------------------------------------------------

    fn handle_event(self: &Arc<Self>, ev: TwitchEvent) {
        match ev {
            TwitchEvent::Connected => {
                let first = !self.connected.swap(true, Ordering::Relaxed);
                if first {
                    self.keyed("system", "connected", json!({ "channel": self.broadcaster_login }), format!("Connected to #{}.", self.broadcaster_login));
                }
                self.last_error.lock().clear();
                self.emit_status();
            }
            TwitchEvent::Disconnected(reason) => {
                self.connected.store(false, Ordering::Relaxed);
                self.keyed("error", "disconnected", json!({ "reason": reason }), format!("Chat connection lost ({reason}). Reconnecting…"));
                self.emit_status();
            }
            TwitchEvent::SubscriptionFailed { kind, error } => {
                self.keyed("error", "subFailed", json!({ "kind": kind, "error": error }), format!("Twitch event “{kind}” unavailable: {error}"));
            }
            TwitchEvent::Chat(m) => self.on_chat(m),
            TwitchEvent::Notification(n) => self.on_notification(n),
            TwitchEvent::Follow { name, .. } => {
                self.keyed("event", "follow", json!({ "user": name }), format!("{name} followed"));
                let p = self.profiles();
                if p.ch.companion_enabled && p.comp.events.follow {
                    let mut f = self.follows.lock();
                    if f.0.is_empty() {
                        f.1 = now_ms() + p.comp.events.follow_batch_secs as i64 * 1000;
                    }
                    if !f.0.contains(&name) {
                        f.0.push(name);
                    }
                }
            }
            TwitchEvent::StreamOnline => {
                self.stream.lock().live = true;
                self.stream_seen.lock().clear();
                self.keyed("system", "live", json!({}), "Stream is live.".into());
                let me = self.clone();
                tokio::spawn(async move { me.refresh_stream().await });
            }
            TwitchEvent::StreamOffline => {
                {
                    let mut s = self.stream.lock();
                    s.live = false;
                    s.viewers = 0;
                    s.started_at = None;
                }
                self.keyed("system", "offline", json!({}), "Stream went offline.".into());
                self.emit_status();
            }
            TwitchEvent::ChannelUpdate { title, category, language } => {
                let changed = {
                    let mut s = self.stream.lock();
                    let changed = s.game != category;
                    s.title = title;
                    s.game = category.clone();
                    if !language.is_empty() {
                        s.language = language;
                    }
                    changed
                };
                if changed {
                    self.keyed("system", "category", json!({ "category": category }), format!("Category changed to “{category}”."));
                    self.maybe_research(category);
                }
                self.emit_status();
            }
        }
    }

    fn is_known_bot(&self, login: &str) -> bool {
        KNOWN_BOTS.contains(&login) || (self.via_bot && login == self.sender_login)
    }

    fn on_chat(self: &Arc<Self>, m: ChatMessage) {
        let now = m.ts;
        // Our own message (sent through the streamer or the bot account)?
        if m.user_id == self.sender_id && self.ctx.lock().is_own_message(&m.id, &m.text, now) {
            return;
        }
        let p = self.profiles();
        let other_bot = self.is_known_bot(&m.login);
        let role: &'static str = if m.is_broadcaster {
            "broadcaster"
        } else if other_bot {
            "bot"
        } else if m.is_mod {
            "mod"
        } else if m.is_vip {
            "vip"
        } else if m.is_sub {
            "sub"
        } else {
            ""
        };
        {
            let emotes = self.emotes.read();
            let mut ctx = self.ctx.lock();
            let line = Line {
                ts: now,
                user_id: m.user_id.clone(),
                login: m.login.clone(),
                name: m.display_name.clone(),
                text: m.text.clone(),
                role: if other_bot { "other bot" } else { role },
            };
            if other_bot {
                ctx.lines.push_back(line);
            } else {
                ctx.push_human(line, &m.emotes);
                ctx.count_text_emotes(&m.text, |w| {
                    !m.emotes.iter().any(|e| e == w) && emotes.contains(w)
                });
            }
        }
        let mut item = FeedItem::new("chat", m.text.clone());
        item.ts = now;
        item.user = m.display_name.clone();
        item.login = m.login.clone();
        item.user_id = m.user_id.clone();
        item.message_id = m.id.clone();
        item.color = m.color.clone();
        item.badges = m.badges.clone();
        // Native emotes can come from any channel (subs), so they are sent per message.
        let emote_urls: serde_json::Map<String, serde_json::Value> = m
            .twitch_emotes
            .iter()
            .map(|(name, id)| (name.clone(), json!(emotes::twitch_emote_url(id))))
            .collect();
        // Shared chat: messages from the other channels are shown but not moderated or welcomed.
        let foreign = m.source_broadcaster_id.as_deref().is_some_and(|s| s != self.broadcaster_id);
        item.meta = json!({ "bits": m.bits, "firstMsg": m.message_type == "user_intro" && !foreign, "emotes": emote_urls, "shared": foreign });
        self.feed(item);

        if other_bot {
            return;
        }

        // --- Language ---------------------------------------------------
        // The streamer's own messages are never translated.
        let lang = if p.ch.translation.active() && !m.is_broadcaster {
            let detector = translate::Detector::new(&p.ch.translation.languages);
            let emotes = self.emotes.read();
            detector.assess(&m.text, |w| m.emotes.iter().any(|e| e == w) || emotes.contains(w))
        } else {
            Foreign::No
        };

        // --- Moderation -------------------------------------------------
        let mut sent_to_ai = false;
        if !foreign && p.ch.moderation_enabled && p.modp.mode != "off" && !moderation::is_exempt(&p.modp, &m) {
            let local = self.rules.lock().check(&p.modp, &m, m.emote_count);
            if let Some(f) = local {
                let acted = self.handle_finding(&p, &m, f);
                if acted {
                    return;
                }
            } else if p.modp.ai_check == "all"
                // The suspicious-word list only knows German and English, so
                // foreign messages always get the AI check.
                || (p.modp.ai_check == "suspicious" && (lang != Foreign::No || moderation::is_suspicious(&m.text)))
            {
                sent_to_ai = p.mod_ai.is_some() && self.mod_tx.send(ModJob { m: m.clone(), lang }).is_ok();
            }
        }
        if !sent_to_ai && lang == Foreign::Yes {
            let _ = self.tr_tx.send(m.clone());
        }

        if m.is_broadcaster && !self.via_bot && !p.ch.companion_enabled {
            return;
        }

        // --- First-time chatter bookkeeping --------------------------------
        let (first_ever, first_stream, learning) = {
            let mut seen = self.seen.lock();
            let first_ever = seen.ids.insert(m.user_id.clone());
            if first_ever {
                self.seen_dirty.store(true, Ordering::Relaxed);
            }
            let learning = seen.learning(chrono::Utc::now().timestamp());
            let first_stream = self.stream_seen.lock().insert(m.user_id.clone());
            (first_ever, first_stream, learning)
        };

        if !p.ch.companion_enabled {
            return;
        }
        let c = &p.comp;

        // --- Commands ---------------------------------------------------------
        if c.commands.enabled && !c.commands.prefix.trim().is_empty() {
            let prefix = c.commands.prefix.trim().to_lowercase();
            let lower = m.text.trim().to_lowercase();
            if lower == prefix || lower.starts_with(&format!("{prefix} ")) {
                // Skip by characters: lowercasing can change byte lengths (e.g. "İ").
                let skip = c.commands.prefix.trim().chars().count();
                let rest = m.text.trim().chars().skip(skip).collect::<String>().trim().to_string();
                self.on_command(&p, &m, rest);
                return;
            }
        }

        // --- Mentions -------------------------------------------------------
        let is_new = !m.is_broadcaster
            && !foreign
            && (m.message_type == "user_intro"
                || match c.welcome.scope.as_str() {
                    "firstThisStream" => first_stream,
                    _ => first_ever && !learning,
                });
        if c.mentions.enabled && self.is_mention(&p, &m) {
            if m.role_rank() >= role_requirement(&c.mentions.min_role)
                && self.cooldown(&format!("mention:{}", m.user_id), c.mentions.user_cooldown_secs, now)
                && self.cooldown("mention", c.mentions.global_cooldown_secs, now)
            {
                self.spawn_task(Task::Mention(m.clone(), is_new && c.welcome.enabled));
            }
            return;
        }

        // --- Welcome ----------------------------------------------------------
        if c.welcome.enabled && is_new {
            let ok = {
                let mut w = self.welcomes.lock();
                while w.front().is_some_and(|t| now - t > 60_000) {
                    w.pop_front();
                }
                if (w.len() as u32) < c.welcome.max_per_minute.max(1) {
                    w.push_back(now);
                    true
                } else {
                    false
                }
            };
            if ok {
                let me = self.clone();
                let delay = c.welcome.delay_secs as u64;
                let msg = m.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_secs(delay)).await;
                    me.spawn_task(Task::Welcome(msg));
                });
                return;
            }
        }

        // --- Cheers -----------------------------------------------------------
        if c.events.cheer && m.bits >= c.events.min_cheer_bits.max(1) {
            let text: String = m.text.split_whitespace().filter(|w| !w.to_lowercase().starts_with("cheer")).collect::<Vec<_>>().join(" ");
            self.spawn_task(Task::Event(
                "cheer".into(),
                format!("{} just cheered {} bits{}", m.display_name, m.bits, if text.is_empty() { String::new() } else { format!(" with the message \"{text}\"") }),
            ));
        }
    }

    fn is_mention(&self, p: &Profiles, m: &ChatMessage) -> bool {
        let lower = m.text.to_lowercase();
        let mut names: Vec<String> = vec![p.ch.bot_name.trim().to_lowercase()];
        names.extend(p.comp.mentions.trigger_words.iter().map(|w| w.trim().to_lowercase()));
        if self.via_bot {
            names.push(self.sender_login.to_lowercase());
            if m.mentions.iter().any(|l| l.eq_ignore_ascii_case(&self.sender_login)) {
                return true;
            }
        }
        let word_hit = names.iter().filter(|n| n.len() >= 2).any(|n| {
            lower.match_indices(n.as_str()).any(|(i, _)| {
                let before = lower[..i].chars().next_back();
                let after = lower[i + n.len()..].chars().next();
                before.is_none_or(|c| !c.is_alphanumeric()) && after.is_none_or(|c| !c.is_alphanumeric())
            })
        });
        if word_hit {
            return true;
        }
        p.comp.mentions.replies
            && m.reply_parent_id.as_deref().is_some_and(|id| self.ctx.lock().is_reply_to_bot(id))
    }

    /// Returns true (and starts the cooldown) if `key` is not on cooldown.
    fn cooldown(&self, key: &str, secs: u32, now: i64) -> bool {
        let mut cd = self.cooldowns.lock();
        if let Some(last) = cd.get(key)
            && now - last < secs as i64 * 1000 {
                return false;
            }
        cd.insert(key.to_string(), now);
        true
    }

    fn on_command(self: &Arc<Self>, p: &Profiles, m: &ChatMessage, rest: String) {
        let c = &p.comp.commands;
        let (sub, arg) = match rest.split_once(char::is_whitespace) {
            Some((s, a)) => (s.to_lowercase(), a.trim().to_string()),
            None => (rest.to_lowercase(), String::new()),
        };
        let (name, cfg) = match sub.as_str() {
            "ask" | "frage" | "q" => ("ask", &c.ask),
            "fact" | "fakt" => ("fact", &c.fact),
            "joke" | "witz" => ("joke", &c.joke),
            "" | "about" | "help" | "hilfe" | "info" => ("about", &c.about),
            // Anything else is treated as a question.
            _ => ("ask", &c.ask),
        };
        let arg = if name == "ask" && !matches!(sub.as_str(), "ask" | "frage" | "q") { rest.clone() } else { arg };
        if !cfg.enabled || m.role_rank() < role_requirement(&cfg.min_role) {
            return;
        }
        let now = m.ts;
        if !self.cooldown(&format!("cmd:{name}"), cfg.global_cooldown_secs, now) {
            return;
        }
        if !self.cooldown(&format!("cmd:{name}:{}", m.user_id), cfg.user_cooldown_secs, now) {
            return;
        }
        if name == "about" {
            let de = prompt::fixed_text_lang(&p.comp, &self.stream.lock().language) == "de";
            let names: [&str; 3] = if de { ["frage <Frage>", "fakt", "witz"] } else { ["ask <question>", "fact", "joke"] };
            let mut list = Vec::new();
            for (n, cc) in names.into_iter().zip([&c.ask, &c.fact, &c.joke]) {
                if cc.enabled {
                    list.push(format!("{} {n}", c.prefix.trim()));
                }
            }
            let bot = if p.ch.bot_name.trim().is_empty() { "HighBazar" } else { p.ch.bot_name.trim() };
            let (intro, try_) = if de { ("ich bin {bot}, ein KI-Chat-Begleiter.", "Probier:") } else { ("I'm {bot}, an AI chat companion.", "Try:") };
            let text = format!(
                "@{} {} {}",
                m.display_name,
                intro.replace("{bot}", bot),
                if list.is_empty() { String::new() } else { format!("{try_} {}", list.join(" | ")) }
            );
            let _ = self.out_tx.send(Outgoing { text, reason: "command".into() });
            return;
        }
        if name == "ask" && arg.trim().is_empty() {
            return;
        }
        self.spawn_task(Task::Command(name.into(), m.clone(), arg));
    }

    fn on_notification(self: &Arc<Self>, n: ChatNotification) {
        let who = if n.anonymous || n.user_name.is_empty() { "An anonymous gifter".to_string() } else { n.user_name.clone() };
        let tier = match n.tier.as_str() {
            "2000" => " (Tier 2)",
            "3000" => " (Tier 3)",
            _ => "",
        };
        let p = self.profiles();
        let e = &p.comp.events;
        let (enabled, desc) = match n.notice_type.as_str() {
            "sub" => (e.sub, format!("{who} just subscribed{tier}")),
            "resub" => (
                e.resub,
                format!(
                    "{who} resubscribed{tier} for {} months{}",
                    n.months,
                    if n.text.is_empty() { String::new() } else { format!(" and wrote: \"{}\"", n.text) }
                ),
            ),
            "sub_gift" => (e.gift, format!("{who} gifted a sub{tier} to {}", n.recipient)),
            "community_sub_gift" => (e.gift, format!("{who} gifted {} subs{tier} to the community", n.gift_count)),
            "raid" => (e.raid, format!("{who} is raiding the channel with {} viewers", n.raid_viewers)),
            "gift_paid_upgrade" | "prime_paid_upgrade" => (e.sub, format!("{who} upgraded their gifted or Prime sub to a paid sub{tier}")),
            "pay_it_forward" => (e.gift, format!("{who} paid a gifted sub forward")),
            "watch_streak" if n.streak > 0 => (e.watch_streak, format!("{who} has watched {} streams in a row (watch streak)", n.streak)),
            "bits_badge_tier" if n.bits_tier > 0 => (e.cheer, format!("{who} unlocked the {} Bits badge", n.bits_tier)),
            "charity_donation" => (false, format!("{who} donated to the charity campaign")),
            _ => return,
        };
        self.keyed(
            "event",
            match n.notice_type.as_str() {
                "community_sub_gift" => "communityGift",
                "sub_gift" => "gift",
                "gift_paid_upgrade" | "prime_paid_upgrade" => "upgrade",
                "pay_it_forward" => "payForward",
                "watch_streak" => "watchStreak",
                "bits_badge_tier" => "bitsBadge",
                "charity_donation" => "charity",
                other => other,
            },
            json!({
                "user": who,
                "streak": n.streak,
                "bitsTier": n.bits_tier,
                "tier": tier.trim().trim_matches(['(', ')']),
                "months": n.months,
                "count": n.gift_count,
                "recipient": n.recipient,
                "viewers": n.raid_viewers,
            }),
            if n.system_message.is_empty() { desc.clone() } else { n.system_message.clone() },
        );
        if !p.ch.companion_enabled || !enabled {
            return;
        }
        if n.notice_type == "raid" && self.has_twitch() {
            // Mention what the raider was playing.
            let me = self.clone();
            let login = n.user_login.clone();
            tokio::spawn(async move {
                let helix = me.st.helix();
                let mut d = desc;
                if let Ok(Some(u)) = helix.user_by_login(&me.broadcaster_id, &login).await
                    && let Ok(Some(c)) = helix.channel(&me.broadcaster_id, &u.id).await
                        && !c.game_name.is_empty() {
                            d.push_str(&format!(" (they were streaming {})", c.game_name));
                        }
                me.spawn_task(Task::Event("raid".into(), d));
            });
            return;
        }
        self.spawn_task(Task::Event(n.notice_type, desc));
    }

    // ------------------------------------------------------------------
    // Generating companion messages
    // ------------------------------------------------------------------

    fn spawn_task(self: &Arc<Self>, task: Task) {
        let me = self.clone();
        tokio::spawn(async move {
            if let Err(e) = me.respond(task).await {
                me.error(format!("AI error: {e}"));
            }
        });
    }

    fn knowledge_items(&self, n: usize, mark_used: bool) -> Vec<String> {
        let k = self.knowledge.lock();
        let Some(k) = k.as_ref() else {
            return Vec::new();
        };
        let mut ctx = self.ctx.lock();
        let mut items: Vec<String> = k.items().filter(|i| !ctx.used_facts.contains(*i)).cloned().collect();
        if items.is_empty() {
            ctx.used_facts.clear();
            items = k.items().cloned().collect();
        }
        items.shuffle(&mut rand::rng());
        items.truncate(n);
        if mark_used {
            for i in &items {
                ctx.used_facts.insert(i.clone());
            }
        }
        items
    }

    fn emote_list(&self, c: &CompanionProfile) -> Vec<String> {
        if c.emote_level == 0 {
            return Vec::new();
        }
        let cat = self.emotes.read();
        let allowed: HashSet<&String> = cat.allowed(&c.emotes).into_iter().collect();
        let mut out: Vec<String> = Vec::new();
        if c.emotes.prefer_popular {
            for e in self.ctx.lock().popular_emotes(20) {
                if allowed.contains(&e) {
                    out.push(e);
                }
            }
        }
        // Channel emotes come first in each list; take a sample of the rest.
        let mut rest: Vec<&String> = allowed.into_iter().filter(|e| !out.contains(e)).collect();
        rest.sort();
        let mut rng = rand::rng();
        rest.shuffle(&mut rng);
        for e in rest.into_iter().take(45usize.saturating_sub(out.len())) {
            out.push(e.clone());
        }
        if out.is_empty() {
            // Global Twitch emotes everyone has.
            out = ["LUL", "Kappa", "PogChamp", "NotLikeThis", "BibleThump", "SeemsGood", "HeyGuys", "<3", "VoHiYo", "Kreygasm"]
                .iter()
                .map(|s| s.to_string())
                .collect();
        }
        out
    }

    async fn respond(self: &Arc<Self>, task: Task) -> AppResult<()> {
        let p = self.profiles();
        let Some(ai_profile) = p.ai.clone() else {
            if !self.no_ai_warned.swap(true, Ordering::Relaxed) {
                self.keyed("error", "noAi", json!({}), "No AI profile selected for this channel — choose one under Channels.".into());
            }
            return Ok(());
        };
        if !self.budget_ok(&p) {
            return Ok(());
        }
        // Wait for a free slot instead of silently dropping replies when busy.
        let Ok(Ok(permit)) = tokio::time::timeout(Duration::from_secs(60), self.inflight.acquire()).await else {
            log::warn!("AI busy, dropped {}", task.label());
            return Ok(());
        };
        let result = self.respond_inner(&p, &ai_profile, &task).await;
        drop(permit);
        let text = result?;
        let Some(text) = text else {
            return Ok(());
        };
        if let Task::Interval(kind) = &task {
            self.ctx.lock().note_kind(kind);
        }
        self.deliver(&p, text, task.label());
        Ok(())
    }

    async fn respond_inner(&self, p: &Profiles, ai_profile: &AiProfile, task: &Task) -> AppResult<Option<String>> {
        let c = &p.comp;
        let (live, title, game, stream_lang, viewers, started) = {
            let s = self.stream.lock();
            (s.live, s.title.clone(), s.game.clone(), s.language.clone(), s.viewers, s.started_at)
        };
        let has_knowledge = self.knowledge.lock().is_some();
        let knowledge_n = match task {
            Task::Interval(k) if k == "game_fact" || k == "game_tip" => 6,
            Task::Command(name, ..) if name == "fact" => 6,
            Task::Mention(..) | Task::Command(..) => 4,
            _ => 0,
        };
        let mark_used = matches!(task, Task::Interval(_)) || matches!(task, Task::Command(n, ..) if n == "fact");
        let knowledge = self.knowledge_items(knowledge_n, mark_used);

        let task_text = match task {
            Task::Interval(kind) => prompt::interval_task(kind, has_knowledge, &game),
            Task::Mention(m, new) => format!(
                "@{name} wrote to you: \"{text}\"\nReply directly to them, starting with @{name}. Be helpful, fun and concise. If they ask something you cannot know, say so playfully.{new}",
                name = m.display_name,
                text = m.text,
                new = if *new { " This is their first message in this channel, so also welcome them briefly." } else { "" }
            ),
            Task::Welcome(m) => format!(
                "{name} is chatting in this channel for the first time. Their message: \"{text}\"\nWelcome them warmly in one short message starting with @{name}; reference their message if it fits naturally.{extra}",
                name = m.display_name,
                text = m.text,
                extra = if c.welcome.instructions.trim().is_empty() { String::new() } else { format!(" {}", c.welcome.instructions.trim()) }
            ),
            Task::Event(kind, desc) => {
                let how = match kind.as_str() {
                    "raid" => "Welcome the raiders warmly and thank the raider by @mentioning them.",
                    "follow" => "Thank them briefly and warmly in one message (mention them with @).",
                    _ => "Celebrate and thank them in one short, genuine message (mention them with @).",
                };
                format!("Event: {desc}.\n{how}")
            }
            Task::Command(name, m, arg) => match name.as_str() {
                "fact" => format!(
                    "@{n} asked for a fun fact. Share one surprising, true fact about {topic}, starting with @{n}.",
                    n = m.display_name,
                    topic = if game.is_empty() || research::is_non_game(&game) { "gaming or the stream topic".to_string() } else { format!("\"{game}\"") }
                ),
                "joke" => format!("@{n} asked for a joke. Tell one short, clever joke fitting the stream, starting with @{n}.", n = m.display_name),
                _ => format!(
                    "@{n} asks you: \"{arg}\"\nAnswer briefly and correctly, starting with @{n}. If you don't know or can't know, say so honestly and playfully.",
                    n = m.display_name
                ),
            },
        };

        let now = now_ms();
        // Before taking `ctx`: emote_list locks it too (parking_lot is not re-entrant).
        let emotes = self.emote_list(c);
        let user_prompt = {
            let ctx = self.ctx.lock();
            let chat = ctx.recent(c.context_messages.clamp(0, 200) as usize, c.context_max_age_mins.max(1) as i64 * 60_000, now);
            let bot_recent: Vec<String> = ctx.bot_recent.iter().rev().take(8).rev().cloned().collect();
            let uptime = started.map(|s| {
                let mins = (chrono::Utc::now() - s).num_minutes().max(0);
                if mins >= 60 { format!("{}h {}m", mins / 60, mins % 60) } else { format!("{mins}m") }
            });
            let block = prompt::ContextBlock {
                stream: prompt::StreamFacts {
                    channel_name: &self.broadcaster_name,
                    live: live || self.sandbox,
                    uptime,
                    game: &game,
                    title: &title,
                    language: &stream_lang,
                    viewers: if live { Some(viewers) } else { None },
                },
                knowledge,
                emotes,
                bot_recent,
                chat,
                language_rule: prompt::language_rule(c, &stream_lang),
            };
            let task_text = format!(
                "{task_text}\n\nNow write the chat message itself, exactly as it should appear in chat: no label or heading (e.g. no \"Hype moment:\"), no preamble, and nothing about these instructions or the task. Keep it under {} characters.",
                c.max_length.clamp(60, 480) * 9 / 10
            );
            prompt::user_prompt(&block, &task_text)
        };
        let system = prompt::system_prompt(&p.ch, c, &self.broadcaster_name, self.via_bot);
        let raw = self
            .ai_call(ai_profile, AiRequest { system, user: user_prompt, max_tokens: None, web_search: false })
            .await?;
        let prefix = if p.ch.prefix_enabled { p.ch.prefix.as_str() } else { "" };
        let Some(text) = output::sanitize(&raw, &p.ch.bot_name, prefix, c.max_length as usize) else {
            log::info!("model skipped ({})", task.label());
            return Ok(None);
        };
        let emotes = self.emotes.read();
        let text = output::limit_emotes(&text, |w| emotes.contains(w), c.emotes.max_per_message as usize);
        Ok(Some(text))
    }

    fn deliver(self: &Arc<Self>, p: &Profiles, text: String, reason: &str) {
        match p.ch.send_mode.as_str() {
            "preview" | "approve" => {
                let send_at = (p.ch.send_mode == "preview").then(|| now_ms() + p.ch.preview_delay_secs.max(1) as i64 * 1000);
                self.pending.lock().push(PendingMessage {
                    id: new_id(),
                    text,
                    reason: reason.into(),
                    created: now_ms(),
                    send_at,
                });
                self.emit_pending();
            }
            _ => {
                let _ = self.out_tx.send(Outgoing { text, reason: reason.into() });
            }
        }
    }

    // ------------------------------------------------------------------
    // Moderation
    // ------------------------------------------------------------------

    /// Applies a finding. Returns true if the message was removed.
    fn handle_finding(self: &Arc<Self>, p: &Profiles, m: &ChatMessage, f: Finding) -> bool {
        let window = p.modp.escalation.window_mins.max(1) as i64 * 60_000;
        let prior = {
            let mut s = self.strikes.lock();
            let list = s.entry(m.user_id.clone()).or_default();
            list.retain(|t| m.ts - t < window);
            list.len()
        };
        let Some(action) = moderation::decide(&p.modp, &f, prior) else {
            return false;
        };
        let punitive = matches!(action.kind.as_str(), "delete" | "warn" | "timeout" | "ban");
        let mut entry = ModLogEntry {
            id: new_id(),
            ts: now_ms(),
            user_id: m.user_id.clone(),
            login: m.login.clone(),
            display_name: m.display_name.clone(),
            message_id: m.id.clone(),
            text: m.text.clone(),
            category: f.category.clone(),
            verdict: f.verdict.clone(),
            confidence: f.confidence,
            reason: f.reason.clone(),
            source: f.source.clone(),
            action: action.clone(),
            status: String::new(),
            error: String::new(),
            source_lang: f.translation.as_ref().map(|t| t.lang.clone()).unwrap_or_default(),
            translation: f.translation.as_ref().map(|t| t.text.clone()).unwrap_or_default(),
            meaning: f.translation.as_ref().map(|t| t.meaning.clone()).unwrap_or_default(),
        };
        let auto = p.modp.mode == "auto" && punitive;
        entry.status = match p.modp.mode.as_str() {
            "suggest" if punitive => "suggested".into(),
            "auto" if punitive => "pending".into(),
            _ => "logged".into(),
        };
        if f.verdict == "violation" && (auto || p.modp.mode != "auto") {
            self.strikes.lock().entry(m.user_id.clone()).or_default().push(m.ts);
        }
        self.record_mod(entry.clone());
        if auto {
            let me = self.clone();
            let announce = p.modp.announce_actions;
            let template = p.modp.reason_template.clone();
            tokio::spawn(async move {
                me.execute_mod(entry, &template, announce).await;
            });
            return true;
        }
        false
    }

    fn record_mod(&self, entry: ModLogEntry) {
        let is_new = {
            let mut log = self.modlog.lock();
            if let Some(existing) = log.iter_mut().find(|e| e.id == entry.id) {
                *existing = entry.clone();
                false
            } else {
                log.push(entry.clone());
                if log.len() > 500 {
                    log.remove(0);
                }
                true
            }
        };
        let path = self.st.data_dir.join("channels").join(&self.broadcaster_id).join("modlog.jsonl");
        if let Err(e) = storage::append_jsonl(&path, &entry) {
            log::warn!("modlog write failed: {e}");
        }
        let _ = self.app.emit(EVT_MODLOG, &entry);
        // Status updates of an entry reach the feed item through EVT_MODLOG.
        if !is_new {
            return;
        }
        let mut item = FeedItem::new(
            "mod",
            format!(
                "{} · {} · {}{}",
                entry.display_name,
                moderation::describe_action(&entry.action),
                entry.category,
                if entry.reason.is_empty() { String::new() } else { format!(" — {}", entry.reason) }
            ),
        );
        item.user = entry.display_name.clone();
        item.login = entry.login.clone();
        item.user_id = entry.user_id.clone();
        item.message_id = entry.message_id.clone();
        item.meta = serde_json::to_value(&entry).unwrap_or_default();
        self.feed(item);
    }

    async fn execute_mod(self: &Arc<Self>, mut entry: ModLogEntry, template: &str, announce: bool) {
        let reason = template.replace("{reason}", if entry.reason.is_empty() { &entry.category } else { &entry.reason });
        // Translated messages: the translation goes to the mods as the reason of a
        // timeout/ban (Twitch shows it to mods, the broadcaster and the user only).
        // Warnings are shown to the viewer prominently, so they keep the short reason.
        let tr = self.profiles().ch.translation;
        let noted = if entry.translation.is_empty() {
            reason.clone()
        } else {
            let t = Translation { lang: entry.source_lang.clone(), text: entry.translation.clone(), meaning: entry.meaning.clone() };
            format!("{reason} · {}", translate::mod_note(&t, tr.target() == "de"))
        };
        // A plain delete has no reason field; a 1-second timeout removes the message
        // the same way and carries the note.
        let purge = entry.action.kind == "delete" && !entry.translation.is_empty() && tr.note_on_delete;
        let res = if self.sandbox || !self.has_twitch() {
            entry.status = "simulated".into();
            Ok(())
        } else {
            let helix = self.st.helix();
            let b = self.broadcaster_id.as_str();
            let r = match entry.action.kind.as_str() {
                "delete" if purge => helix.ban(b, b, &entry.user_id, Some(1), &noted).await,
                "delete" => helix.delete_message(b, b, &entry.message_id).await,
                "warn" => {
                    let _ = helix.delete_message(b, b, &entry.message_id).await;
                    helix.warn(b, b, &entry.user_id, &reason).await
                }
                "timeout" => helix.ban(b, b, &entry.user_id, Some(entry.action.duration_secs.max(1)), &noted).await,
                "ban" => helix.ban(b, b, &entry.user_id, None, &noted).await,
                _ => Ok(()),
            };
            entry.status = if r.is_ok() { "executed".into() } else { "failed".into() };
            r
        };
        if let Err(e) = res {
            entry.error = e.to_string();
            self.error(format!("Moderation action against {} failed: {e}", entry.display_name));
        } else {
            self.st.usage.lock().add_mod_action(&self.channel_id);
            if announce && matches!(entry.action.kind.as_str(), "timeout" | "ban" | "warn") {
                let de = prompt::fixed_text_lang(&self.profiles().comp, &self.stream.lock().language) == "de";
                let action = match (de, entry.action.kind.as_str()) {
                    (true, "timeout") => format!("Timeout {}", moderation::human_duration(entry.action.duration_secs)),
                    (true, "ban") => "gebannt".into(),
                    (true, _) => "Verwarnung".into(),
                    _ => moderation::describe_action(&entry.action),
                };
                let text = format!(
                    "@{} {action} — {}",
                    entry.display_name,
                    moderation::public_reason(&entry.source, &entry.reason, &entry.category, de)
                );
                let _ = self.out_tx.send(Outgoing { text, reason: "moderation".into() });
            }
        }
        self.record_mod(entry);
    }

    async fn undo_mod(self: &Arc<Self>, id: &str) -> AppResult<()> {
        let mut entry = self.modlog.lock().iter().find(|e| e.id == id).cloned();
        if entry.is_none() {
            // Entries from earlier sessions live in the log file.
            let path = self.st.data_dir.join("channels").join(&self.broadcaster_id).join("modlog.jsonl");
            entry = storage::read_jsonl_tail::<ModLogEntry>(&path, 5000)
                .into_iter().rfind(|e| e.id == id);
        }
        let Some(mut entry) = entry else {
            return Err(AppError::msg("Moderation entry not found."));
        };
        if !matches!(entry.action.kind.as_str(), "timeout" | "ban") {
            return Err(AppError::msg("Only timeouts and bans can be undone."));
        }
        if !self.sandbox && self.has_twitch() {
            let b = self.broadcaster_id.as_str();
            self.st.helix().unban(b, b, &entry.user_id).await?;
        }
        entry.status = "undone".into();
        // Undoing also forgives the strike.
        if let Some(list) = self.strikes.lock().get_mut(&entry.user_id) {
            list.pop();
        }
        self.record_mod(entry);
        Ok(())
    }

    /// Shows a translation under the chat message in the feed.
    fn publish_translation(&self, m: &ChatMessage, t: &Translation) {
        let _ = self.app.emit(EVT_TRANSLATION, json!({ "messageId": m.id, "lang": t.lang, "text": t.text }));
    }

    async fn moderation_worker(self: Arc<Self>, mut rx: mpsc::UnboundedReceiver<ModJob>, mut shutdown: watch::Receiver<bool>) {
        let mut calls: VecDeque<i64> = VecDeque::new();
        loop {
            let first = tokio::select! {
                m = rx.recv() => m,
                _ = shutdown.changed() => return,
            };
            let Some(first) = first else { return };
            let mut batch = vec![first];
            let deadline = tokio::time::Instant::now() + Duration::from_millis(2500);
            while batch.len() < 15 {
                match tokio::time::timeout_at(deadline, rx.recv()).await {
                    Ok(Some(m)) => batch.push(m),
                    _ => break,
                }
            }
            // Messages this worker can't judge right now still get translated.
            let hand_over = |batch: &[ModJob]| {
                for j in batch.iter().filter(|j| j.lang == Foreign::Yes) {
                    let _ = self.tr_tx.send(j.m.clone());
                }
            };
            let p = self.profiles();
            let Some(ai_profile) = p.mod_ai.clone() else {
                hand_over(&batch);
                continue;
            };
            if !p.ch.moderation_enabled || p.modp.mode == "off" || p.modp.ai_check == "off" {
                hand_over(&batch);
                continue;
            }
            if !self.budget_ok(&p) {
                continue;
            }
            let now = now_ms();
            while calls.front().is_some_and(|t| now - t > 60_000) {
                calls.pop_front();
            }
            if calls.len() as u32 >= p.modp.max_ai_calls_per_min.max(1) {
                {
                    let mut warned = self.mod_rate_warned.lock();
                    if now - *warned > 300_000 {
                        *warned = now;
                        self.keyed("system", "modRateLimit", json!({}), "AI moderation is at its per-minute limit; some messages were only checked by local rules.".into());
                    }
                }
                hand_over(&batch);
                continue;
            }
            calls.push_back(now);

            let lang = {
                let s = self.stream.lock();
                let l = prompt::language_name(&s.language);
                if l.is_empty() { "English".to_string() } else { l.to_string() }
            };
            let mut system = prompt::moderation_system(p.modp.strictness, &p.modp.channel_rules, &lang);
            let tr = &p.ch.translation;
            let translating = tr.active() && batch.iter().any(|j| j.lang != Foreign::No);
            if translating {
                let target = prompt::language_name(tr.target());
                system.push_str(&translate::moderation_section(&translate::names(&tr.languages), target));
            }
            let user = {
                let ctx = self.ctx.lock();
                let first_ts = batch[0].m.ts;
                let context: Vec<String> = ctx
                    .lines
                    .iter()
                    .filter(|l| l.ts < first_ts)
                    .rev()
                    .take(12)
                    .map(|l| format!("{}: {}", l.name, l.text.chars().take(200).collect::<String>()))
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect();
                let mut u = String::from("Recent chat for context:\n");
                if context.is_empty() {
                    u.push_str("(none)\n");
                }
                for c in context {
                    u.push_str(&c);
                    u.push('\n');
                }
                u.push_str("\nMessages to judge:\n");
                for (i, j) in batch.iter().enumerate() {
                    let mark = if translating && j.lang != Foreign::No { " (translate)" } else { "" };
                    u.push_str(&format!("[{i}]{mark} {}: {}\n", j.m.display_name, j.m.text.chars().take(400).collect::<String>()));
                }
                u
            };
            let req = AiRequest { system, user, max_tokens: None, web_search: false };
            match self.ai_call(&ai_profile, req).await {
                Ok(text) => {
                    let mut translations: HashMap<usize, Translation> = if translating {
                        translate::parse(&text, batch.len())
                            .into_iter()
                            .filter(|(i, _)| batch[*i].lang != Foreign::No)
                            .map(|(i, t)| (i, translate::without_name(t, &batch[i].m.display_name)))
                            .collect()
                    } else {
                        HashMap::new()
                    };
                    for (i, t) in &translations {
                        self.publish_translation(&batch[*i].m, t);
                    }
                    for (i, mut f) in moderation::parse_ai_findings(&text, batch.len()) {
                        f.translation = translations.remove(&i);
                        let p = self.profiles();
                        self.handle_finding(&p, &batch[i].m, f);
                    }
                }
                Err(e) => self.error(format!("AI moderation failed: {e}")),
            }
        }
    }

    /// Translates foreign messages that don't go through AI moderation
    /// (moderation off, exempt users, shared chat, rate limit).
    async fn translation_worker(self: Arc<Self>, mut rx: mpsc::UnboundedReceiver<ChatMessage>, mut shutdown: watch::Receiver<bool>) {
        const MAX_CALLS_PER_MIN: usize = 10;
        let mut calls: VecDeque<i64> = VecDeque::new();
        loop {
            let first = tokio::select! {
                m = rx.recv() => m,
                _ = shutdown.changed() => return,
            };
            let Some(first) = first else { return };
            let mut batch = vec![first];
            let deadline = tokio::time::Instant::now() + Duration::from_millis(2000);
            while batch.len() < 10 {
                match tokio::time::timeout_at(deadline, rx.recv()).await {
                    Ok(Some(m)) => batch.push(m),
                    _ => break,
                }
            }
            let p = self.profiles();
            let tr = &p.ch.translation;
            let Some(ai_profile) = p.mod_ai.clone() else { continue };
            if !tr.active() || !self.budget_ok(&p) {
                continue;
            }
            let now = now_ms();
            while calls.front().is_some_and(|t| now - t > 60_000) {
                calls.pop_front();
            }
            if calls.len() >= MAX_CALLS_PER_MIN {
                continue;
            }
            calls.push_back(now);
            let system = translate::translation_system(&translate::names(&tr.languages), prompt::language_name(tr.target()));
            let mut user = String::from("Messages:\n");
            for (i, m) in batch.iter().enumerate() {
                user.push_str(&format!("[{i}] {}: {}\n", m.display_name, m.text.chars().take(400).collect::<String>()));
            }
            let req = AiRequest { system, user, max_tokens: None, web_search: false };
            match self.ai_call(&ai_profile, req).await {
                Ok(text) => {
                    for (i, t) in translate::parse(&text, batch.len()) {
                        self.publish_translation(&batch[i], &translate::without_name(t, &batch[i].display_name));
                    }
                }
                Err(e) => log::warn!("translation failed: {e}"),
            }
        }
    }

    // ------------------------------------------------------------------
    // Sending
    // ------------------------------------------------------------------

    async fn sender(self: Arc<Self>, mut rx: mpsc::UnboundedReceiver<Outgoing>, mut shutdown: watch::Receiver<bool>) {
        let mut last_sent = 0i64;
        loop {
            let msg = tokio::select! {
                m = rx.recv() => m,
                _ = shutdown.changed() => return,
            };
            let Some(msg) = msg else { return };
            let wait = 1500 - (now_ms() - last_sent);
            if wait > 0 {
                tokio::time::sleep(Duration::from_millis(wait as u64)).await;
            }
            let p = self.profiles();
            let full = output::with_prefix(&msg.text, p.ch.prefix_enabled, &p.ch.prefix);
            let full = output::truncate_words(&full, output::TWITCH_MAX);
            let ts = now_ms();
            self.ctx.lock().expect_own(&full, ts);
            let result = if self.sandbox || !self.has_twitch() {
                Ok(None)
            } else {
                match self.st.helix().send_chat(&self.sender_id, &self.broadcaster_id, &full, None).await {
                    Ok(r) if r.is_sent => Ok(Some(r.message_id)),
                    Ok(r) => Err(format!(
                        "Twitch dropped the message: {}",
                        r.drop_reason.map(|d| format!("{} ({})", d.message, d.code)).unwrap_or_default()
                    )),
                    Err(e) => Err(e.to_string()),
                }
            };
            last_sent = now_ms();
            match result {
                Ok(id) => {
                    self.ctx.lock().push_bot(
                        Line {
                            ts,
                            user_id: self.sender_id.clone(),
                            login: self.sender_login.clone(),
                            name: if p.ch.bot_name.is_empty() { "HighBazar".into() } else { p.ch.bot_name.clone() },
                            text: msg.text.clone(),
                            role: "bot",
                        },
                        id.clone(),
                    );
                    self.st.usage.lock().add_message(&self.channel_id);
                    let mut item = FeedItem::new("bot", msg.text.clone());
                    item.user = p.ch.bot_name.clone();
                    item.login = self.sender_login.clone();
                    item.message_id = id.unwrap_or_default();
                    item.meta = json!({ "reason": msg.reason, "sandbox": self.sandbox, "fullText": full });
                    self.feed(item);
                    self.emit_status();
                }
                Err(e) => self.error(e),
            }
        }
    }

    // ------------------------------------------------------------------
    // Timers
    // ------------------------------------------------------------------

    fn schedule_next(&self, c: &CompanionProfile, factor: f64) {
        let min = c.interval_min_secs.max(30) as i64;
        let max = (c.interval_max_secs as i64).max(min);
        let secs = rand::rng().random_range(min..=max) as f64 * factor;
        *self.next_interval.lock() = now_ms() + (secs * 1000.0) as i64;
    }

    fn tick(self: &Arc<Self>, n: u64) {
        let now = now_ms();
        let p = self.profiles();

        // Pending messages whose preview time ran out.
        let due: Vec<PendingMessage> = {
            let mut pending = self.pending.lock();
            let (due, keep): (Vec<_>, Vec<_>) = pending.drain(..).partition(|m| m.send_at.is_some_and(|t| t <= now));
            *pending = keep;
            due
        };
        if !due.is_empty() {
            for m in due {
                let _ = self.out_tx.send(Outgoing { text: m.text, reason: m.reason });
            }
            self.emit_pending();
        }

        // Batched follow thanks.
        let follows = {
            let mut f = self.follows.lock();
            if !f.0.is_empty() && now >= f.1 { Some(std::mem::take(&mut f.0)) } else { None }
        };
        if let Some(names) = follows {
            let shown: Vec<String> = names.iter().take(5).cloned().collect();
            let more = names.len().saturating_sub(5);
            let desc = format!(
                "These viewers just followed: {}{}",
                shown.join(", "),
                if more > 0 { format!(" and {more} more") } else { String::new() }
            );
            self.spawn_task(Task::Event("follow".into(), desc));
        }

        self.interval_check(&p, now);

        if n.is_multiple_of(5) {
            self.emit_status();
        }
        if n.is_multiple_of(60) {
            self.save_seen();
            self.st.usage.lock().save_if_dirty();
        }
        if n > 0 && n.is_multiple_of(120) {
            let me = self.clone();
            tokio::spawn(async move { me.refresh_stream().await });
        }
        if n > 0 && n.is_multiple_of(1800) {
            let me = self.clone();
            tokio::spawn(async move { me.refresh_emotes().await });
        }
        if n > 0 && n.is_multiple_of(3600) && self.has_twitch() {
            // Twitch asks apps to validate tokens hourly.
            let me = self.clone();
            tokio::spawn(async move {
                let client_id = me.st.config.read().twitch_client_id();
                if let Ok(token) = me.st.tokens.access_token(&client_id, &me.broadcaster_id).await
                    && let Ok(None) = crate::twitch::auth::validate(&me.st.http, &token).await {
                        let _ = me.st.tokens.force_refresh(&client_id, &me.broadcaster_id, &token).await;
                    }
            });
        }
    }

    fn interval_check(self: &Arc<Self>, p: &Profiles, now: i64) {
        let c = &p.comp;
        let set_paused = |r: &str| {
            let mut pr = self.paused_reason.lock();
            if pr.as_str() != r {
                *pr = r.to_string();
            }
        };
        if !p.ch.companion_enabled || !c.interval_enabled {
            *self.next_interval.lock() = 0;
            set_paused("");
            return;
        }
        let next = *self.next_interval.lock();
        if next == 0 {
            self.schedule_next(c, 0.5);
            return;
        }
        // Show the offline pause right away instead of only when the timer fires.
        let offline = c.only_when_live && !self.stream.lock().live && !self.sandbox;
        if offline {
            set_paused("offline");
        } else if self.paused_reason.lock().as_str() == "offline" {
            set_paused("");
        }
        if now < next {
            return;
        }
        if offline {
            self.schedule_next(c, 0.5);
            return;
        }
        let (rate, unanswered) = {
            let mut ctx = self.ctx.lock();
            (ctx.rate_per_min(now), ctx.unanswered)
        };
        if c.adaptive && c.busy_threshold > 0 && rate >= c.busy_threshold {
            set_paused("busy");
            self.schedule_next(c, 0.5);
            return;
        }
        if c.max_unanswered > 0 && unanswered >= c.max_unanswered {
            set_paused("waiting");
            // Check again soon; any human message resets the counter.
            *self.next_interval.lock() = now + 30_000;
            return;
        }
        set_paused("");
        let factor = if c.adaptive && c.busy_threshold > 0 { 1.0 + rate as f64 / c.busy_threshold as f64 } else { 1.0 };
        self.schedule_next(c, factor);
        let game = self.stream.lock().game.clone();
        let kind = {
            let mut ctx = self.ctx.lock();
            ctx.unanswered += 1;
            let recent: Vec<String> = ctx.recent_kinds.iter().cloned().collect();
            prompt::pick_interval_kind(c, &recent, !game.is_empty() && !research::is_non_game(&game))
        };
        self.spawn_task(Task::Interval(kind));
    }

    fn save_seen(&self) {
        if !self.seen_dirty.swap(false, Ordering::Relaxed) || !self.has_twitch() || self.sandbox {
            return;
        }
        let seen = self.seen.lock().clone();
        let path = self.st.data_dir.join("channels").join(&self.broadcaster_id).join("seen.json");
        if let Err(e) = storage::write_json_atomic(&path, &seen) {
            log::warn!("saving seen chatters failed: {e}");
        }
    }

    // ------------------------------------------------------------------
    // Controls from the UI
    // ------------------------------------------------------------------

    async fn handle_control(self: &Arc<Self>, c: Control) {
        match c {
            Control::SayNow => {
                let p = self.profiles();
                let recent: Vec<String> = self.ctx.lock().recent_kinds.iter().cloned().collect();
                let game = self.stream.lock().game.clone();
                let kind = prompt::pick_interval_kind(&p.comp, &recent, !game.is_empty() && !research::is_non_game(&game));
                self.spawn_task(Task::Interval(kind));
            }
            Control::Approve(id) => {
                let m = {
                    let mut pending = self.pending.lock();
                    let pos = pending.iter().position(|m| m.id == id);
                    pos.map(|i| pending.remove(i))
                };
                if let Some(m) = m {
                    let _ = self.out_tx.send(Outgoing { text: m.text, reason: m.reason });
                }
                self.emit_pending();
            }
            Control::Reject(id) => {
                self.pending.lock().retain(|m| m.id != id);
                self.emit_pending();
            }
            Control::EditPending(id, text) => {
                if let Some(m) = self.pending.lock().iter_mut().find(|m| m.id == id) {
                    m.text = text;
                    m.send_at = None;
                }
                self.emit_pending();
            }
            Control::Simulate(sim) => self.simulate(sim),
            Control::SimulateEvent(kind, user) => {
                let user = if user.trim().is_empty() { "TestViewer".to_string() } else { user };
                match kind.as_str() {
                    "follow" => self.handle_event(TwitchEvent::Follow { login: user.to_lowercase(), name: user }),
                    "raid" => self.on_notification(ChatNotification { notice_type: "raid".into(), user_name: user.clone(), user_login: user.to_lowercase(), raid_viewers: 42, ..Default::default() }),
                    "sub" => self.on_notification(ChatNotification { notice_type: "sub".into(), user_name: user, tier: "1000".into(), months: 1, ..Default::default() }),
                    "resub" => self.on_notification(ChatNotification { notice_type: "resub".into(), user_name: user, tier: "1000".into(), months: 7, text: "Still here!".into(), ..Default::default() }),
                    "gift" => self.on_notification(ChatNotification { notice_type: "community_sub_gift".into(), user_name: user, tier: "1000".into(), gift_count: 5, ..Default::default() }),
                    _ => {}
                }
            }
            Control::ApplySuggestion(id) => {
                let entry = self.modlog.lock().iter().find(|e| e.id == id).cloned();
                if let Some(e) = entry {
                    let p = self.profiles();
                    self.execute_mod(e, &p.modp.reason_template, p.modp.announce_actions).await;
                }
            }
            Control::Dismiss(id) => {
                let entry = self.modlog.lock().iter().find(|e| e.id == id).cloned();
                if let Some(mut e) = entry {
                    e.status = "dismissed".into();
                    if let Some(list) = self.strikes.lock().get_mut(&e.user_id) {
                        list.pop();
                    }
                    self.record_mod(e);
                }
            }
            Control::Undo(id) => {
                if let Err(e) = self.undo_mod(&id).await {
                    self.error(e.to_string());
                }
            }
            Control::Manual(a) => self.manual(a).await,
            Control::RefreshStream => {
                self.refresh_stream().await;
                self.refresh_emotes().await;
            }
        }
    }

    fn simulate(self: &Arc<Self>, sim: SimMessage) {
        let name = if sim.user.trim().is_empty() { "TestViewer".to_string() } else { sim.user.trim().to_string() };
        let login = name.to_lowercase().replace(' ', "_");
        let badges: Vec<String> = match sim.role.as_str() {
            "broadcaster" => vec!["broadcaster".into()],
            "mod" => vec!["moderator".into()],
            "vip" => vec!["vip".into()],
            "sub" => vec!["subscriber".into()],
            _ => vec![],
        };
        let m = ChatMessage {
            id: format!("sim-{}", new_id()),
            user_id: format!("sim-{login}"),
            login,
            display_name: name,
            text: sim.text,
            color: "#9147FF".into(),
            is_broadcaster: sim.role == "broadcaster",
            is_mod: sim.role == "mod",
            is_vip: sim.role == "vip",
            is_sub: sim.role == "sub",
            badges,
            message_type: "text".into(),
            bits: sim.bits,
            ts: now_ms(),
            ..Default::default()
        };
        self.on_chat(m);
    }

    async fn manual(self: &Arc<Self>, a: ManualAction) {
        let entry = ModLogEntry {
            id: new_id(),
            ts: now_ms(),
            user_id: a.user_id,
            login: a.login,
            display_name: a.display_name,
            message_id: a.message_id,
            text: a.text,
            category: "manual".into(),
            verdict: "violation".into(),
            confidence: 1.0,
            reason: "manual action".into(),
            source: "manual".into(),
            action: ModAction::new(&a.kind, a.duration_secs),
            status: "pending".into(),
            error: String::new(),
            ..Default::default()
        };
        let p = self.profiles();
        self.execute_mod(entry, &p.modp.reason_template, false).await;
    }
}

pub async fn run(session: Arc<Session>, mut ctrl: mpsc::UnboundedReceiver<Control>, mut shutdown: watch::Receiver<bool>) {
    let s = session;
    if s.sandbox {
        s.keyed("system", "startedTest", json!({}), "Bot started in test mode — nothing is sent to Twitch.".into());
    } else {
        s.keyed("system", "starting", json!({ "channel": s.broadcaster_login }), format!("Bot starting for #{}…", s.broadcaster_login));
    }
    {
        let p = s.profiles();
        if p.ai.is_none() {
            s.keyed("error", "noAi", json!({}), "No AI profile selected — companion messages and AI moderation are disabled.".into());
            s.no_ai_warned.store(true, Ordering::Relaxed);
        }
    }

    let (tw_tx, mut tw_rx) = mpsc::channel::<TwitchEvent>(512);
    if s.has_twitch() {
        // Stream info and emotes first so the first messages have context.
        let me = s.clone();
        tokio::spawn(async move {
            me.refresh_stream().await;
            let game = me.stream.lock().game.clone();
            me.maybe_research(game);
            me.refresh_emotes().await;
        });
    }
    if !s.sandbox {
        let cfg = eventsub::EventSubConfig { helix: s.st.helix(), broadcaster_id: s.broadcaster_id.clone() };
        tokio::spawn(eventsub::run(cfg, tw_tx, shutdown.clone()));
    } else {
        drop(tw_tx);
    }
    if let Some(rx) = s.out_rx.lock().take() {
        tokio::spawn(s.clone().sender(rx, shutdown.clone()));
    }
    if let Some(rx) = s.mod_rx.lock().take() {
        tokio::spawn(s.clone().moderation_worker(rx, shutdown.clone()));
    }
    if let Some(rx) = s.tr_rx.lock().take() {
        tokio::spawn(s.clone().translation_worker(rx, shutdown.clone()));
    }
    s.emit_status();

    let mut ticker = tokio::time::interval(Duration::from_secs(1));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let mut n: u64 = 0;
    let mut twitch_open = !s.sandbox;
    loop {
        tokio::select! {
            ev = tw_rx.recv(), if twitch_open => {
                match ev {
                    Some(ev) => s.handle_event(ev),
                    None => twitch_open = false,
                }
            }
            c = ctrl.recv() => {
                match c {
                    Some(c) => s.handle_control(c).await,
                    None => break,
                }
            }
            _ = ticker.tick() => {
                s.tick(n);
                n += 1;
            }
            _ = shutdown.changed() => break,
        }
    }
    s.save_seen();
    s.st.usage.lock().save_if_dirty();
    s.keyed("system", "stopped", json!({}), "Bot stopped.".into());
}
