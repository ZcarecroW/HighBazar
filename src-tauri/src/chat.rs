//! Typing help for the streamer's own chat input (in the app and in the
//! overlay): suggestions for ":" (emotes of every provider and Unicode emojis),
//! "@" (people in chat) and Tab completion, ranked by favorites, what you use
//! yourself and what chat uses. One implementation for both inputs; a query
//! takes well under a millisecond, and the emoji data is only loaded when the
//! first ":" is typed.

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str, Utf32String};
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};

use crate::storage;

const EMOJI_TSV: &str = include_str!("../data/emoji.tsv");
const MAX_PEOPLE: usize = 500;

/// The emojis used most in chats, most popular first: among similar matches
/// they come first (the official order groups emojis by topic, not by use).
const POPULAR: &[&str] = &[
    "😂", "❤️", "🤣", "😭", "🔥", "👍", "😍", "🥺", "✨", "🙏", "😊", "💀", "😅", "😎", "🎉", "💯", "👀", "😳", "🤔", "😢",
    "🥰", "😘", "😉", "😏", "🙄", "😡", "😤", "😱", "🤯", "😴", "👏", "🙌", "💪", "👋", "🤝", "💜", "💙", "💚", "💛", "🧡",
    "🖤", "🤍", "💔", "⭐", "🌟", "⚡", "🎮", "🏆", "🍕", "🍿", "☕", "🍺", "🎂", "👑", "💩", "🤡", "👻", "🐐", "🗿", "😬",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    /// Emote of the streamer's channel.
    Channel,
    /// Twitch emote from a channel the streamer is subscribed to.
    Sub,
    Global,
}

/// An emote the streamer can use, with where it comes from.
#[derive(Debug, Clone)]
pub struct EmoteInfo {
    pub name: String,
    /// "twitch" | "bttv" | "ffz" | "7tv"
    pub provider: &'static str,
    pub url: String,
    pub scope: Scope,
}

struct Emote {
    info: EmoteInfo,
    hay: Utf32String,
}

struct Emoji {
    ch: String,
    en: String,
    de: String,
    hay: Utf32String,
    /// All words with spaces around them, for exact keyword hits.
    words: String,
}

fn emojis() -> &'static [Emoji] {
    static DATA: OnceLock<Vec<Emoji>> = OnceLock::new();
    DATA.get_or_init(|| {
        EMOJI_TSV
            .lines()
            .filter(|l| !l.starts_with('#'))
            .filter_map(|l| {
                let mut f = l.split('\t');
                let (ch, en, en_words, de, de_words) = (f.next()?, f.next()?, f.next()?, f.next()?, f.next()?);
                // Shortcodes and names first: they decide most matches.
                let hay = format!("{} {} {} {}", en.to_lowercase(), de.to_lowercase(), en_words, de_words);
                let words = format!(" {} ", hay.replace(['_', '-'], " "));
                Some(Emoji { ch: ch.into(), en: en.into(), de: de.into(), hay: Utf32String::from(hay.as_str()), words })
            })
            .collect()
    })
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Suggestion {
    /// Text that goes into the message.
    pub insert: String,
    pub label: String,
    /// "emote" | "emoji" | "user"
    pub kind: &'static str,
    /// "twitch" | "bttv" | "ffz" | "7tv" | "emoji" | "user"
    pub provider: &'static str,
    pub url: String,
    /// "channel" | "sub" | "global" | ""
    pub scope: &'static str,
    pub favorite: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistStatus {
    pub sub_emotes_missing: bool,
    pub emotes: usize,
    pub people: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// After ":" – emotes and emojis, fuzzy.
    Colon,
    /// "@" – people in chat.
    Mention,
    /// Tab on a word – emotes and people that start with it.
    Tab,
    /// The emote picker grid.
    Browse,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct Usage {
    uses: HashMap<String, u32>,
    last: HashMap<String, i64>,
}

pub struct ChatAssist {
    emotes: RwLock<Arc<Vec<Emote>>>,
    usage: Mutex<Usage>,
    /// Emote uses in chat this session.
    community: Mutex<HashMap<String, u32>>,
    /// Recent chatters (display name, login), newest last.
    people: Mutex<VecDeque<(String, String)>>,
    /// Everyone in chat right now according to Twitch (display name, login),
    /// including people who have not written anything yet.
    present: Mutex<Vec<(String, String)>>,
    /// The streamer's login, never suggested for "@".
    own_login: Mutex<String>,
    /// The emotes of the streamer's subscriptions could not be loaded because
    /// the Twitch login lacks `user:read:emotes` (reconnect fixes it).
    subs_missing: AtomicBool,
    matcher: Mutex<Matcher>,
    path: PathBuf,
}

impl ChatAssist {
    pub fn new(data_dir: &std::path::Path) -> Self {
        let path = data_dir.join("chat-input.json");
        let mut config = Config::DEFAULT;
        config.prefer_prefix = true;
        Self {
            emotes: RwLock::new(Arc::new(Vec::new())),
            usage: Mutex::new(storage::read_json(&path).unwrap_or_default()),
            community: Mutex::new(HashMap::new()),
            people: Mutex::new(VecDeque::new()),
            present: Mutex::new(Vec::new()),
            own_login: Mutex::new(String::new()),
            subs_missing: AtomicBool::new(false),
            matcher: Mutex::new(Matcher::new(config)),
            path,
        }
    }

    /// The emotes of the channel the bot runs in (replaces the previous list).
    pub fn set_emotes(&self, list: Vec<EmoteInfo>) {
        let list = list.into_iter().map(|info| Emote { hay: Utf32String::from(info.name.as_str()), info }).collect();
        *self.emotes.write() = Arc::new(list);
    }

    /// Adds emotes (e.g. those of the streamer's subscriptions) that are not known yet.
    pub fn add_emotes(&self, extra: Vec<EmoteInfo>) {
        let cur = self.emotes.read().clone();
        let known: HashSet<&str> = cur.iter().map(|e| e.info.name.as_str()).collect();
        let mut list: Vec<Emote> = cur.iter().map(|e| Emote { info: e.info.clone(), hay: e.hay.clone() }).collect();
        for info in extra {
            if !known.contains(info.name.as_str()) {
                list.push(Emote { hay: Utf32String::from(info.name.as_str()), info });
            }
        }
        *self.emotes.write() = Arc::new(list);
    }

    pub fn clear(&self) {
        *self.emotes.write() = Arc::new(Vec::new());
        self.community.lock().clear();
        self.people.lock().clear();
        self.present.lock().clear();
        self.subs_missing.store(false, Ordering::Relaxed);
    }

    /// The people in chat right now (Twitch's chatter list) and the streamer's own login.
    pub fn set_present(&self, own_login: &str, list: Vec<(String, String)>) {
        *self.own_login.lock() = own_login.to_lowercase();
        *self.present.lock() = list;
    }

    pub fn set_subs_missing(&self, missing: bool) {
        self.subs_missing.store(missing, Ordering::Relaxed);
    }

    /// What the input can tell the streamer about missing suggestions.
    pub fn status(&self) -> AssistStatus {
        AssistStatus {
            sub_emotes_missing: self.subs_missing.load(Ordering::Relaxed),
            emotes: self.emotes.read().len(),
            people: self.people.lock().len().max(self.present.lock().len()),
        }
    }

    /// A chat message from someone else: remember the person and its emotes.
    pub fn observe(&self, display: &str, login: &str, emotes: impl IntoIterator<Item = String>) {
        {
            let mut p = self.people.lock();
            if let Some(i) = p.iter().position(|(_, l)| l == login) {
                p.remove(i);
            }
            p.push_back((display.to_string(), login.to_string()));
            while p.len() > MAX_PEOPLE {
                p.pop_front();
            }
        }
        let mut c = self.community.lock();
        for e in emotes {
            *c.entry(e).or_insert(0) += 1;
        }
    }

    /// The streamer sent a message: count its emotes and emojis for the ranking.
    pub fn record_sent(&self, text: &str) {
        let names: HashSet<String> = self.emotes.read().iter().map(|e| e.info.name.clone()).collect();
        let now = chrono::Utc::now().timestamp();
        let mut used: Vec<String> = text.split_whitespace().filter(|w| names.contains(*w)).map(str::to_string).collect();
        if !text.is_ascii() {
            used.extend(emojis().iter().filter(|e| text.contains(e.ch.as_str())).map(|e| e.ch.clone()));
        }
        if used.is_empty() {
            return;
        }
        let snapshot = {
            let mut u = self.usage.lock();
            for k in used {
                *u.uses.entry(k.clone()).or_insert(0) += 1;
                u.last.insert(k, now);
            }
            // Keep the file small: the 400 most used.
            if u.uses.len() > 500 {
                let mut v: Vec<(String, u32)> = u.uses.iter().map(|(k, n)| (k.clone(), *n)).collect();
                v.sort_by_key(|x| std::cmp::Reverse(x.1));
                let keep: HashSet<String> = v.into_iter().take(400).map(|(k, _)| k).collect();
                u.uses.retain(|k, _| keep.contains(k));
                u.last.retain(|k, _| keep.contains(k));
            }
            serde_json::to_value(&*u).ok()
        };
        if let Some(v) = snapshot {
            let _ = storage::write_json_atomic(&self.path, &v);
        }
    }

    /// Suggestions for what is typed. `german` picks the emoji names shown.
    pub fn suggest(&self, query: &str, mode: Mode, favorites: &[String], german: bool, limit: usize) -> Vec<Suggestion> {
        let limit = limit.clamp(1, 400);
        match mode {
            Mode::Mention => self.people_matching(query, limit, false),
            Mode::Tab => {
                let mut out = self.emotes_matching(query, None, favorites, limit, true);
                out.extend(self.people_matching(query, limit, true));
                out.truncate(limit);
                out
            }
            Mode::Colon | Mode::Browse => {
                let (provider, q) = provider_filter(query);
                let with_emoji = matches!(provider, None | Some("emoji"));
                let mut scored: Vec<(f64, Suggestion)> = Vec::new();
                if provider != Some("emoji") {
                    scored.extend(self.score_emotes(q, provider, favorites, false));
                }
                if with_emoji && (mode == Mode::Colon || provider == Some("emoji") || !q.is_empty()) {
                    scored.extend(self.score_emojis(q, favorites, german, provider == Some("emoji")));
                }
                scored.sort_by(|a, b| b.0.total_cmp(&a.0).then_with(|| a.1.label.len().cmp(&b.1.label.len())));
                scored.into_iter().take(limit).map(|(_, s)| s).collect()
            }
        }
    }

    fn boost(&self, key: &str, favorites: &[String]) -> (f64, bool) {
        let fav = favorites.iter().any(|f| f == key);
        let own = self.usage.lock().uses.get(key).copied().unwrap_or(0);
        let chat = self.community.lock().get(key).copied().unwrap_or(0);
        let b = if fav { 300.0 } else { 0.0 } + 40.0 * f64::from(own).ln_1p() + 12.0 * f64::from(chat).ln_1p();
        (b, fav)
    }

    fn score_emotes(&self, q: &str, provider: Option<&str>, favorites: &[String], prefix_only: bool) -> Vec<(f64, Suggestion)> {
        let emotes = self.emotes.read().clone();
        let ql = q.to_lowercase();
        let pattern = Pattern::new(q, CaseMatching::Ignore, Normalization::Smart, AtomKind::Fuzzy);
        let mut m = self.matcher.lock();
        let mut out = Vec::new();
        for e in emotes.iter() {
            if provider.is_some_and(|p| p != e.info.provider) {
                continue;
            }
            let name = &e.info.name;
            let lower = name.to_lowercase();
            let base = if q.is_empty() {
                0.0
            } else if prefix_only {
                if !lower.starts_with(&ql) {
                    continue;
                }
                100.0 + if name.starts_with(q) { 5.0 } else { 0.0 }
            } else {
                let Some(s) = pattern.score(e.hay.slice(..), &mut m) else { continue };
                let mut s = f64::from(s);
                if lower == ql {
                    s += 60.0;
                } else if name.starts_with(q) {
                    s += 40.0;
                } else if lower.starts_with(&ql) {
                    s += 30.0;
                }
                s
            };
            let (b, fav) = self.boost(name, favorites);
            // Without a query (and no provider shortcut): what you or chat use and
            // the channel's own – not every global smiley.
            if q.is_empty() && provider.is_none() && b <= 0.0 && e.info.scope != Scope::Channel {
                continue;
            }
            // Channel emotes first – but without a query, what you use comes first.
            let scope_bonus = match (e.info.scope, q.is_empty()) {
                (Scope::Channel, false) => 25.0,
                (Scope::Sub, false) => 15.0,
                (Scope::Channel, true) => 2.0,
                (Scope::Sub, true) => 1.0,
                (Scope::Global, _) => 0.0,
            };
            out.push((
                base + b + scope_bonus,
                Suggestion {
                    insert: name.clone(),
                    label: name.clone(),
                    kind: "emote",
                    provider: e.info.provider,
                    url: e.info.url.clone(),
                    scope: match e.info.scope {
                        Scope::Channel => "channel",
                        Scope::Sub => "sub",
                        Scope::Global => "global",
                    },
                    favorite: fav,
                },
            ));
        }
        out
    }

    fn emotes_matching(&self, q: &str, provider: Option<&str>, favorites: &[String], limit: usize, prefix_only: bool) -> Vec<Suggestion> {
        let mut v = self.score_emotes(q, provider, favorites, prefix_only);
        v.sort_by(|a, b| b.0.total_cmp(&a.0).then_with(|| a.1.label.len().cmp(&b.1.label.len())).then_with(|| a.1.label.cmp(&b.1.label)));
        v.into_iter().take(limit).map(|(_, s)| s).collect()
    }

    fn score_emojis(&self, q: &str, favorites: &[String], german: bool, all: bool) -> Vec<(f64, Suggestion)> {
        let list = emojis();
        let mut out = Vec::new();
        let make = |e: &Emoji, fav: bool| Suggestion {
            insert: e.ch.clone(),
            label: if german { e.de.clone() } else { e.en.clone() },
            kind: "emoji",
            provider: "emoji",
            url: String::new(),
            scope: "",
            favorite: fav,
        };
        if q.is_empty() {
            // Without a query only favorites and emojis you use (or all of them in the picker).
            for (i, e) in list.iter().enumerate() {
                let (b, fav) = self.boost(&e.ch, favorites);
                if all || b > 0.0 {
                    out.push((b - i as f64 * 0.001, make(e, fav)));
                }
            }
            return out;
        }
        // The words of an emoji are a long list, so scattered letters would match
        // almost anything: the query has to appear as written (any case, umlauts optional).
        let pattern = Pattern::new(q, CaseMatching::Ignore, Normalization::Smart, AtomKind::Substring);
        let ql = q.to_lowercase();
        let (exact, start) = (format!(" {ql} "), format!(" {ql}"));
        let mut m = self.matcher.lock();
        for (i, e) in list.iter().enumerate() {
            let Some(s) = pattern.score(e.hay.slice(..), &mut m) else { continue };
            let (b, fav) = self.boost(&e.ch, favorites);
            // A whole keyword ("joy", "herz") beats a word that merely starts like it.
            let word = if e.words.contains(&exact) {
                60.0
            } else if e.words.contains(&start) {
                15.0
            } else {
                0.0
            };
            // Emotes win a tie: in Twitch chat they are what people usually mean.
            // Popular emojis first among similar matches; then the official order.
            // Compared without the emoji variation selector (U+FE0F).
            let base = e.ch.trim_end_matches('\u{fe0f}');
            let popular = POPULAR.iter().position(|p| p.trim_end_matches('\u{fe0f}') == base).map_or(0.0, |r| 30.0 - r as f64 * 0.3);
            out.push((f64::from(s) * 0.9 + word + popular + b - 5.0 - i as f64 * 0.002, make(e, fav)));
        }
        out
    }

    /// People who wrote recently come first (newest first), then everyone else
    /// Twitch lists as being in chat. The streamer is never suggested.
    fn people_matching(&self, q: &str, limit: usize, prefix_only: bool) -> Vec<Suggestion> {
        let q = q.trim_start_matches('@');
        let ql = q.to_lowercase();
        let own = self.own_login.lock().clone();
        let people = self.people.lock();
        let present = self.present.lock();
        let pattern = Pattern::new(q, CaseMatching::Ignore, Normalization::Smart, AtomKind::Fuzzy);
        let mut m = self.matcher.lock();
        let mut buf = Vec::new();
        let mut seen: HashSet<&str> = HashSet::new();
        let mut v: Vec<(f64, Suggestion)> = Vec::new();
        let n = people.len().max(1) as f64;
        let recent = people.iter().enumerate().rev().map(|(i, p)| (p, 20.0 + i as f64 / n * 30.0));
        for ((display, login), recency) in recent.chain(present.iter().map(|p| (p, 0.0))) {
            if login == &own || !seen.insert(login.as_str()) {
                continue;
            }
            let score = if q.is_empty() {
                recency
            } else if prefix_only {
                if !display.to_lowercase().starts_with(&ql) && !login.starts_with(&ql) {
                    continue;
                }
                100.0 + recency
            } else {
                let hay = format!("{display} {login}");
                let Some(s) = pattern.score(Utf32Str::new(&hay, &mut buf), &mut m) else { continue };
                f64::from(s) + recency + if login.starts_with(&ql) { 80.0 } else { 0.0 }
            };
            v.push((
                score,
                Suggestion {
                    insert: format!("@{display}"),
                    label: display.clone(),
                    kind: "user",
                    provider: "user",
                    url: String::new(),
                    scope: "",
                    favorite: false,
                },
            ));
        }
        v.sort_by(|a, b| b.0.total_cmp(&a.0).then_with(|| a.1.label.to_lowercase().cmp(&b.1.label.to_lowercase())));
        v.into_iter().take(limit).map(|(_, s)| s).collect()
    }
}

/// Sends a message as the streamer through the running bot and waits for Twitch.
pub async fn send(st: &crate::state::AppState, text: String) -> crate::error::AppResult<()> {
    use crate::error::AppError;
    let (tx, rx) = tokio::sync::oneshot::channel();
    st.bot.control(crate::bot::Control::Send(text, tx)).await?;
    tokio::time::timeout(std::time::Duration::from_secs(20), rx)
        .await
        .map_err(|_| AppError::msg("Twitch did not answer in time."))?
        .map_err(|_| AppError::msg("The bot stopped."))?
}

/// Suggestions with the saved favorites.
pub fn suggest_for(st: &crate::state::AppState, query: &str, mode: Mode, german: bool, limit: usize) -> Vec<Suggestion> {
    let favorites = st.config.read().settings.chat_input.favorites.clone();
    st.chat.suggest(query, mode, &favorites, german, limit)
}

/// ":7 pog" → (7TV, "pog"); also "b" BTTV, "f" FFZ, "t" Twitch, "e" emoji.
fn provider_filter(query: &str) -> (Option<&'static str>, &str) {
    let q = query.trim_start();
    let mut chars = q.chars();
    let p = match chars.next().map(|c| c.to_ascii_lowercase()) {
        Some('7') => "7tv",
        Some('b') => "bttv",
        Some('f') => "ffz",
        Some('t') => "twitch",
        Some('e') => "emoji",
        _ => return (None, q.trim()),
    };
    match chars.next() {
        None => (Some(p), ""),
        Some(' ') => (Some(p), q[2..].trim()),
        _ => (None, q.trim()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assist() -> ChatAssist {
        let dir = std::env::temp_dir().join(format!("hb-chat-{}", uuid::Uuid::new_v4().simple()));
        std::fs::create_dir_all(&dir).unwrap();
        let a = ChatAssist::new(&dir);
        let e = |name: &str, provider: &'static str, scope| EmoteInfo { name: name.into(), provider, url: format!("u/{name}"), scope };
        a.set_emotes(vec![
            e("zcarecHype", "twitch", Scope::Channel),
            e("Kappa", "twitch", Scope::Global),
            e("PogChamp", "twitch", Scope::Global),
            e("catJAM", "7tv", Scope::Channel),
            e("catJAM2", "bttv", Scope::Channel),
            e("Pog", "7tv", Scope::Channel),
            e("POGGERS", "ffz", Scope::Global),
            e("peepoClap", "bttv", Scope::Global),
        ]);
        a
    }

    fn labels(v: &[Suggestion]) -> Vec<&str> {
        v.iter().map(|s| s.label.as_str()).collect()
    }

    #[test]
    fn provider_shortcuts() {
        assert_eq!(provider_filter("7 pog"), (Some("7tv"), "pog"));
        assert_eq!(provider_filter("b"), (Some("bttv"), ""));
        assert_eq!(provider_filter("pog"), (None, "pog"));
        assert_eq!(provider_filter("bttvx"), (None, "bttvx"));
    }

    #[test]
    fn colon_ranks_exact_and_prefix_first() {
        let a = assist();
        let r = a.suggest("pog", Mode::Colon, &[], false, 5);
        assert_eq!(r[0].label, "Pog");
        assert!(labels(&r).contains(&"PogChamp"));
        let r = a.suggest("7 pog", Mode::Colon, &[], false, 5);
        assert!(r.iter().all(|s| s.provider == "7tv"), "{r:?}");
        let r = a.suggest("catjm", Mode::Colon, &[], false, 3);
        assert_eq!(r[0].label, "catJAM");
    }

    #[test]
    fn favorites_and_own_use_win() {
        let a = assist();
        a.record_sent("hello POGGERS POGGERS");
        a.record_sent("POGGERS");
        let r = a.suggest("pog", Mode::Colon, &[], false, 3);
        assert_eq!(r[0].label, "POGGERS", "{r:?}");
        let r = a.suggest("pog", Mode::Colon, &["PogChamp".into()], false, 3);
        assert_eq!(r[0].label, "PogChamp");
        assert!(r[0].favorite);
    }

    #[test]
    fn emojis_in_english_and_german() {
        let a = assist();
        let r = a.suggest("herz", Mode::Colon, &[], true, 5);
        assert!(r.iter().any(|s| s.insert == "❤️"), "{r:?}");
        let r = a.suggest("joy", Mode::Colon, &[], false, 3);
        assert_eq!(r[0].insert, "😂");
        let r = a.suggest("e feuer", Mode::Colon, &[], true, 3);
        assert_eq!(r[0].insert, "🔥");
        assert!(r.iter().all(|s| s.kind == "emoji"));
    }

    #[test]
    fn tab_completes_prefixes_and_people() {
        let a = assist();
        a.observe("Peter", "peter", Vec::new());
        a.observe("Max", "max", vec!["Kappa".into()]);
        let r = a.suggest("Kap", Mode::Tab, &[], false, 5);
        assert_eq!(r[0].label, "Kappa");
        let r = a.suggest("pog", Mode::Colon, &[], false, 8);
        assert!(r.iter().all(|s| s.kind == "emote"), "no emoji should match pog: {r:?}");
        let r = a.suggest("", Mode::Colon, &[], false, 20);
        // Unused global emotes stay out; Kappa was used in chat above, so it may show.
        assert!(!r.iter().any(|s| s.label == "PogChamp" || s.label == "POGGERS"), "{r:?}");
        // With a provider shortcut, all of its emotes are listed.
        assert!(a.suggest("f", Mode::Colon, &[], false, 5).iter().any(|s| s.label == "POGGERS"));
        let r = a.suggest("pe", Mode::Tab, &[], false, 5);
        assert_eq!(labels(&r), vec!["peepoClap", "Peter"]);
        let r = a.suggest("@ma", Mode::Mention, &[], false, 5);
        assert_eq!(r[0].insert, "@Max");
    }

    #[test]
    fn mentions_include_everyone_in_chat() {
        let a = assist();
        a.set_present("streamer", vec![("Streamer".into(), "streamer".into()), ("Anna".into(), "anna".into()), ("Max".into(), "max".into())]);
        // Nobody wrote yet: everyone Twitch lists, without the streamer.
        let r = a.suggest("", Mode::Mention, &[], false, 10);
        assert_eq!(labels(&r), vec!["Anna", "Max"]);
        // Who wrote recently comes first, and only once.
        a.observe("Max", "max", Vec::new());
        let r = a.suggest("", Mode::Mention, &[], false, 10);
        assert_eq!(labels(&r), vec!["Max", "Anna"]);
        assert_eq!(a.suggest("ann", Mode::Mention, &[], false, 5)[0].label, "Anna");
        assert!(a.suggest("stre", Mode::Mention, &[], false, 5).is_empty());
    }

    #[test]
    fn sent_messages_count_emojis() {
        let a = assist();
        a.record_sent("gg 🔥🔥");
        let r = a.suggest("", Mode::Colon, &[], false, 5);
        assert_eq!(r[0].insert, "🔥");
    }

    #[test]
    fn fast_enough_for_every_keystroke() {
        let a = assist();
        let many: Vec<EmoteInfo> = (0..5000)
            .map(|i| EmoteInfo { name: format!("emote{i}Pepe{}", i % 97), provider: "7tv", url: String::new(), scope: Scope::Global })
            .collect();
        a.add_emotes(many);
        let _ = a.suggest("x", Mode::Colon, &[], false, 10); // load the emoji data
        let t = std::time::Instant::now();
        for q in ["p", "pe", "pep", "pepe4", "catjam", "herz"] {
            let _ = a.suggest(q, Mode::Colon, &[], true, 10);
        }
        // Six queries over ~7000 entries (debug builds are ~10x slower).
        eprintln!("6 queries: {:?}", t.elapsed());
        assert!(t.elapsed().as_millis() < 600, "{:?}", t.elapsed());
    }
}
