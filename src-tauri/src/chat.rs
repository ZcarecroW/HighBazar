//! Typing help for the streamer's own chat input (in the app and in the
//! overlay): suggestions for ":" (emotes of every provider and Unicode emojis),
//! "@" (people in chat) and Tab completion, ranked by favorites, what you use
//! yourself and what chat uses. One implementation for both inputs; a query
//! takes a fraction of a millisecond, and the emoji data is only loaded when
//! the first ":" is typed.

use std::cmp::Ordering as CmpOrdering;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str, Utf32String};
use parking_lot::{Mutex, MutexGuard, RwLock};
use serde::{Deserialize, Serialize};

use crate::storage;

const EMOJI_TSV: &str = include_str!("../data/emoji.tsv");
const MAX_PEOPLE: usize = 500;
/// Emoji sequences are at most this many characters long (families, kisses
/// with two skin tones).
const MAX_EMOJI_CHARS: usize = 12;

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

impl Scope {
    fn as_str(self) -> &'static str {
        match self {
            Scope::Channel => "channel",
            Scope::Sub => "sub",
            Scope::Global => "global",
        }
    }
}

/// An emote the streamer can use, with where it comes from.
#[derive(Debug, Clone)]
pub struct EmoteInfo {
    pub name: String,
    /// "twitch" | "bttv" | "ffz" | "7tv"
    pub provider: &'static str,
    pub url: String,
    pub scope: Scope,
    /// The emote chat shows for this name. Several providers can have an
    /// emote with the same name; typing it gives the same text, so it is
    /// suggested once – except when a provider is picked (":b", ":7" …).
    pub primary: bool,
}

struct Emote {
    info: EmoteInfo,
    hay: Utf32String,
    lower: String,
}

#[derive(Default)]
struct EmoteIndex {
    list: Vec<Emote>,
    /// Every name, for counting the emotes of sent messages.
    names: HashSet<String>,
    /// Different names.
    unique: usize,
}

struct Emoji {
    ch: String,
    en: String,
    de: String,
    hay: Utf32String,
    /// All words with spaces around them, for exact keyword hits.
    words: String,
    /// Bonus for the emojis chats use most.
    popular: f64,
}

/// Without U+FE0F: "❤" and "❤️" are the same heart in chat.
fn base(s: &str) -> String {
    s.replace('\u{fe0f}', "")
}

fn emojis() -> &'static [Emoji] {
    static DATA: OnceLock<Vec<Emoji>> = OnceLock::new();
    DATA.get_or_init(|| {
        let rank: HashMap<String, usize> = POPULAR.iter().enumerate().map(|(i, p)| (base(p), i)).collect();
        EMOJI_TSV
            .lines()
            .filter(|l| !l.starts_with('#'))
            .filter_map(|l| {
                let mut f = l.split('\t');
                let (ch, en, en_words, de, de_words) = (f.next()?, f.next()?, f.next()?, f.next()?, f.next()?);
                // Shortcodes and names first: they decide most matches.
                let hay = format!("{} {} {} {}", en.to_lowercase(), de.to_lowercase(), en_words, de_words);
                let words = format!(" {} ", hay.replace(['_', '-'], " "));
                let popular = rank.get(&base(ch)).map_or(0.0, |r| 30.0 - *r as f64 * 0.3);
                Some(Emoji { ch: ch.into(), en: en.into(), de: de.into(), hay: Utf32String::from(hay.as_str()), words, popular })
            })
            .collect()
    })
}

/// Emoji (without U+FE0F) → its index, for finding the emojis in a message.
fn emoji_lookup() -> &'static HashMap<String, usize> {
    static DATA: OnceLock<HashMap<String, usize>> = OnceLock::new();
    DATA.get_or_init(|| emojis().iter().enumerate().map(|(i, e)| (base(&e.ch), i)).collect())
}

/// The emojis in a text, longest match first: "❤️‍🔥" counts as itself, not as
/// ❤️ and 🔥; a skin tone falls back to the plain emoji ("👍🏽" → 👍).
fn emojis_in(text: &str) -> Vec<usize> {
    let lookup = emoji_lookup();
    let plain = base(text);
    let chars: Vec<(usize, char)> = plain.char_indices().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].1.is_ascii() {
            i += 1;
            continue;
        }
        let start = chars[i].0;
        let found = (1..=MAX_EMOJI_CHARS.min(chars.len() - i)).rev().find_map(|n| {
            let end = chars.get(i + n).map_or(plain.len(), |c| c.0);
            lookup.get(&plain[start..end]).map(|idx| (n, *idx))
        });
        match found {
            Some((n, idx)) => {
                out.push(idx);
                i += n;
            }
            None => i += 1,
        }
    }
    out
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

/// Ranking boosts of one query: the locks are taken once, not per candidate.
struct Boosts<'a> {
    favorites: &'a [String],
    usage: MutexGuard<'a, Usage>,
    community: MutexGuard<'a, HashMap<String, u32>>,
}

impl Boosts<'_> {
    fn get(&self, key: &str) -> (f64, bool) {
        let fav = self.favorites.iter().any(|f| f == key);
        let own = self.usage.uses.get(key).copied().unwrap_or(0);
        let chat = self.community.get(key).copied().unwrap_or(0);
        let b = if fav { 300.0 } else { 0.0 } + 40.0 * f64::from(own).ln_1p() + 12.0 * f64::from(chat).ln_1p();
        (b, fav)
    }
}

/// A match before its suggestion is built (only the best few are).
#[derive(Clone, Copy)]
enum Cand {
    Emote(usize),
    Emoji(usize),
}

pub struct ChatAssist {
    emotes: RwLock<Arc<EmoteIndex>>,
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
            emotes: RwLock::new(Arc::default()),
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

    /// Every emote the streamer can use in the channel (replaces the previous
    /// list); one entry per provider and name, see [`EmoteInfo::primary`].
    pub fn set_emotes(&self, list: Vec<EmoteInfo>) {
        let names: HashSet<String> = list.iter().map(|i| i.name.clone()).collect();
        let unique = names.len();
        let list = list
            .into_iter()
            .map(|info| Emote { hay: Utf32String::from(info.name.as_str()), lower: info.name.to_lowercase(), info })
            .collect();
        *self.emotes.write() = Arc::new(EmoteIndex { list, names, unique });
    }

    pub fn clear(&self) {
        *self.emotes.write() = Arc::default();
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
            emotes: self.emotes.read().unique,
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
        let now = chrono::Utc::now().timestamp();
        let mut used: Vec<String> = {
            let idx = self.emotes.read();
            text.split_whitespace().filter(|w| idx.names.contains(*w)).map(str::to_string).collect()
        };
        if !text.is_ascii() {
            let list = emojis();
            used.extend(emojis_in(text).into_iter().map(|i| list[i].ch.clone()));
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

    fn boosts<'a>(&'a self, favorites: &'a [String]) -> Boosts<'a> {
        Boosts { favorites, usage: self.usage.lock(), community: self.community.lock() }
    }

    /// Suggestions for what is typed. `german` picks the emoji names shown.
    pub fn suggest(&self, query: &str, mode: Mode, favorites: &[String], german: bool, limit: usize) -> Vec<Suggestion> {
        let limit = limit.clamp(1, 400);
        let mut out = match mode {
            Mode::Mention => self.people_matching(query, limit, false),
            Mode::Tab => {
                let mut out = self.emotes_matching(query, favorites, limit);
                out.extend(self.people_matching(query, limit, true));
                out
            }
            Mode::Colon | Mode::Browse => {
                let (provider, q) = provider_filter(query);
                let with_emoji = matches!(provider, None | Some("emoji"));
                let idx = self.emotes.read().clone();
                let mut m = self.matcher.lock();
                let boosts = self.boosts(favorites);
                let mut scored: Vec<(f64, usize, Cand)> = Vec::new();
                if provider != Some("emoji") {
                    scored.extend(score_emotes(&idx, q, provider, &boosts, false, &mut m).map(|(s, i)| (s, idx.list[i].info.name.len(), Cand::Emote(i))));
                }
                if with_emoji && (mode == Mode::Colon || provider == Some("emoji") || !q.is_empty()) {
                    let list = emojis();
                    let label = |i: usize| if german { list[i].de.len() } else { list[i].en.len() };
                    scored.extend(score_emojis(q, &boosts, provider == Some("emoji"), &mut m).map(|(s, i)| (s, label(i), Cand::Emoji(i))));
                }
                drop(m);
                top(&mut scored, limit, |a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
                scored
                    .into_iter()
                    .map(|(_, _, c)| match c {
                        Cand::Emote(i) => emote_suggestion(&idx.list[i].info, boosts.get(&idx.list[i].info.name).1),
                        Cand::Emoji(i) => emoji_suggestion(&emojis()[i], german, boosts.get(&emojis()[i].ch).1),
                    })
                    .collect()
            }
        };
        // The same text never twice.
        let mut seen = HashSet::new();
        out.retain(|s| seen.insert(s.insert.clone()));
        out.truncate(limit);
        out
    }

    /// Tab: emotes that start with the word.
    fn emotes_matching(&self, q: &str, favorites: &[String], limit: usize) -> Vec<Suggestion> {
        let idx = self.emotes.read().clone();
        let mut m = self.matcher.lock();
        let boosts = self.boosts(favorites);
        let mut v: Vec<(f64, usize)> = score_emotes(&idx, q, None, &boosts, true, &mut m).collect();
        drop(m);
        let name = |i: usize| idx.list[i].info.name.as_str();
        top(&mut v, limit, |a, b| b.0.total_cmp(&a.0).then(name(a.1).len().cmp(&name(b.1).len())).then(name(a.1).cmp(name(b.1))));
        v.into_iter().map(|(_, i)| emote_suggestion(&idx.list[i].info, boosts.get(name(i)).1)).collect()
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
        let mut hay = String::new();
        let mut seen: HashSet<&str> = HashSet::new();
        let mut v: Vec<(f64, &str, &str)> = Vec::new();
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
                hay.clear();
                hay.push_str(display);
                hay.push(' ');
                hay.push_str(login);
                let Some(s) = pattern.score(Utf32Str::new(&hay, &mut buf), &mut m) else { continue };
                f64::from(s) + recency + if login.starts_with(&ql) { 80.0 } else { 0.0 }
            };
            v.push((score, display, login));
        }
        top(&mut v, limit, |a, b| b.0.total_cmp(&a.0).then_with(|| a.1.to_lowercase().cmp(&b.1.to_lowercase())));
        v.into_iter()
            .map(|(_, display, login)| {
                // Twitch resolves mentions by login: a display name in another
                // script ("小明") would not reach the person.
                let same = display.eq_ignore_ascii_case(login);
                Suggestion {
                    insert: format!("@{}", if same { display } else { login }),
                    label: if same { display.to_string() } else { format!("{display} ({login})") },
                    kind: "user",
                    provider: "user",
                    url: String::new(),
                    scope: "",
                    favorite: false,
                }
            })
            .collect()
    }
}

/// Keeps the best `limit` entries, best first, without sorting all of them.
fn top<T>(v: &mut Vec<T>, limit: usize, mut cmp: impl FnMut(&T, &T) -> CmpOrdering) {
    if v.len() > limit {
        v.select_nth_unstable_by(limit, &mut cmp);
        v.truncate(limit);
    }
    v.sort_by(cmp);
}

fn emote_suggestion(e: &EmoteInfo, favorite: bool) -> Suggestion {
    Suggestion {
        insert: e.name.clone(),
        label: e.name.clone(),
        kind: "emote",
        provider: e.provider,
        url: e.url.clone(),
        scope: e.scope.as_str(),
        favorite,
    }
}

fn emoji_suggestion(e: &Emoji, german: bool, favorite: bool) -> Suggestion {
    Suggestion {
        insert: e.ch.clone(),
        label: if german { e.de.clone() } else { e.en.clone() },
        kind: "emoji",
        provider: "emoji",
        url: String::new(),
        scope: "",
        favorite,
    }
}

/// (score, index) of every emote that matches.
fn score_emotes<'a>(
    idx: &'a EmoteIndex,
    q: &'a str,
    provider: Option<&'a str>,
    boosts: &'a Boosts<'a>,
    prefix_only: bool,
    m: &'a mut Matcher,
) -> impl Iterator<Item = (f64, usize)> + 'a {
    let ql = q.to_lowercase();
    let pattern = Pattern::new(q, CaseMatching::Ignore, Normalization::Smart, AtomKind::Fuzzy);
    idx.list.iter().enumerate().filter_map(move |(i, e)| {
        // A provider shortcut lists that provider's emotes; otherwise each name once.
        match provider {
            Some(p) if p != e.info.provider => return None,
            None if !e.info.primary => return None,
            _ => {}
        }
        let name = &e.info.name;
        let base = if q.is_empty() {
            0.0
        } else if prefix_only {
            if !e.lower.starts_with(&ql) {
                return None;
            }
            100.0 + if name.starts_with(q) { 5.0 } else { 0.0 }
        } else {
            let s = f64::from(pattern.score(e.hay.slice(..), m)?);
            s + if e.lower == ql {
                60.0
            } else if name.starts_with(q) {
                40.0
            } else if e.lower.starts_with(&ql) {
                30.0
            } else {
                0.0
            }
        };
        let (b, _) = boosts.get(name);
        // Without a query (and no provider shortcut): what you or chat use and
        // the channel's own – not every global smiley.
        if q.is_empty() && provider.is_none() && b <= 0.0 && e.info.scope != Scope::Channel {
            return None;
        }
        // Channel emotes first – but without a query, what you use comes first.
        let scope_bonus = match (e.info.scope, q.is_empty()) {
            (Scope::Channel, false) => 25.0,
            (Scope::Sub, false) => 15.0,
            (Scope::Channel, true) => 2.0,
            (Scope::Sub, true) => 1.0,
            (Scope::Global, _) => 0.0,
        };
        Some((base + b + scope_bonus, i))
    })
}

/// (score, index) of every emoji that matches.
fn score_emojis<'a>(q: &'a str, boosts: &'a Boosts<'a>, all: bool, m: &'a mut Matcher) -> Box<dyn Iterator<Item = (f64, usize)> + 'a> {
    let list = emojis();
    if q.is_empty() {
        // Without a query only favorites and emojis you use (or all of them in the picker).
        return Box::new(list.iter().enumerate().filter_map(move |(i, e)| {
            let (b, _) = boosts.get(&e.ch);
            (all || b > 0.0).then_some((b - i as f64 * 0.001, i))
        }));
    }
    // The words of an emoji are a long list, so scattered letters would match
    // almost anything: the query has to appear as written (any case, umlauts optional).
    let pattern = Pattern::new(q, CaseMatching::Ignore, Normalization::Smart, AtomKind::Substring);
    let ql = q.to_lowercase();
    let (exact, start) = (format!(" {ql} "), format!(" {ql}"));
    Box::new(list.iter().enumerate().filter_map(move |(i, e)| {
        let s = pattern.score(e.hay.slice(..), m)?;
        let (b, _) = boosts.get(&e.ch);
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
        Some((f64::from(s) * 0.9 + word + e.popular + b - 5.0 - i as f64 * 0.002, i))
    }))
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

    fn info(name: &str, provider: &'static str, scope: Scope) -> EmoteInfo {
        EmoteInfo { name: name.into(), provider, url: format!("u/{name}"), scope, primary: true }
    }

    fn assist() -> ChatAssist {
        let dir = std::env::temp_dir().join(format!("hb-chat-{}", uuid::Uuid::new_v4().simple()));
        std::fs::create_dir_all(&dir).unwrap();
        let a = ChatAssist::new(&dir);
        a.set_emotes(vec![
            info("zcarecHype", "twitch", Scope::Channel),
            info("Kappa", "twitch", Scope::Global),
            info("PogChamp", "twitch", Scope::Global),
            info("catJAM", "7tv", Scope::Channel),
            info("catJAM2", "bttv", Scope::Channel),
            info("Pog", "7tv", Scope::Channel),
            info("POGGERS", "ffz", Scope::Global),
            info("peepoClap", "bttv", Scope::Global),
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

    /// The same emote name from several providers (7TV and BTTV both have
    /// "AngelThump") is suggested once – the one chat shows.
    #[test]
    fn same_name_from_several_providers_once() {
        let a = assist();
        let dup = |provider, scope, primary| EmoteInfo { primary, ..info("AngelThump", provider, scope) };
        let mut list = vec![dup("7tv", Scope::Channel, true), dup("bttv", Scope::Global, false), dup("ffz", Scope::Global, false)];
        list.push(info("Angel", "twitch", Scope::Global));
        a.set_emotes(list);
        for mode in [Mode::Colon, Mode::Tab, Mode::Browse] {
            let r = a.suggest("angel", mode, &[], false, 10);
            assert_eq!(r.iter().filter(|s| s.insert == "AngelThump").count(), 1, "{mode:?}: {r:?}");
            assert_eq!(r.iter().find(|s| s.insert == "AngelThump").unwrap().provider, "7tv");
        }
        // A provider shortcut shows that provider's emote.
        let r = a.suggest("b angel", Mode::Colon, &[], false, 10);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].provider, "bttv");
        assert_eq!(a.status().emotes, 2);
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
    fn finds_emojis_in_messages() {
        let list = emojis();
        let found = |t: &str| emojis_in(t).into_iter().map(|i| base(&list[i].ch)).collect::<Vec<_>>();
        assert_eq!(found("gg 🔥🔥"), vec!["🔥", "🔥"]);
        // A ZWJ sequence counts as itself, not as its parts.
        assert_eq!(found("❤️‍🔥"), vec![base("❤️‍🔥")]);
        // With or without U+FE0F, and a skin tone falls back to the emoji.
        assert_eq!(found("❤ 👍🏽"), vec!["❤", "👍"]);
        assert!(found("plain text äöü").is_empty());
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
    fn mentions_use_the_login_for_names_in_other_scripts() {
        let a = assist();
        a.observe("小明", "xiaoming", Vec::new());
        a.observe("小明", "xiaoming_2", Vec::new());
        let r = a.suggest("xiao", Mode::Mention, &[], false, 5);
        let inserts: Vec<&str> = r.iter().map(|s| s.insert.as_str()).collect();
        assert!(inserts.contains(&"@xiaoming") && inserts.contains(&"@xiaoming_2"), "{r:?}");
        assert!(r.iter().any(|s| s.label == "小明 (xiaoming)"));
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
            .map(|i| EmoteInfo { name: format!("emote{i}Pepe{}", i % 97), provider: "7tv", url: String::new(), scope: Scope::Global, primary: true })
            .collect();
        a.set_emotes(many);
        let _ = a.suggest("x", Mode::Colon, &[], false, 10); // load the emoji data
        let t = std::time::Instant::now();
        for q in ["p", "pe", "pep", "pepe4", "catjam", "herz"] {
            let _ = a.suggest(q, Mode::Colon, &[], true, 10);
        }
        // Six queries over ~7000 entries (debug builds are ~10x slower).
        eprintln!("6 queries: {:?}", t.elapsed());
        assert!(t.elapsed().as_millis() < 600, "{:?}", t.elapsed());
    }

    /// Per-keystroke cost with a big channel's catalog. Run in an optimized build:
    /// `cargo test --release --lib suggest_bench -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn suggest_bench() {
        let dir = std::env::temp_dir().join(format!("hb-bench-{}", uuid::Uuid::new_v4().simple()));
        std::fs::create_dir_all(&dir).unwrap();
        let a = ChatAssist::new(&dir);
        let words = ["pog", "cat", "pepe", "kek", "lul", "jam", "hype", "sad", "wide", "clap", "dance", "monka", "omega", "peepo", "ez"];
        let mut list = Vec::new();
        for i in 0..6400 {
            let (provider, scope) = match i % 8 {
                0 => ("twitch", Scope::Global),
                1 => ("twitch", Scope::Sub),
                2 | 3 => ("7tv", Scope::Channel),
                4 => ("7tv", Scope::Global),
                5 => ("bttv", Scope::Channel),
                6 => ("bttv", Scope::Global),
                _ => ("ffz", Scope::Channel),
            };
            let name = format!("{}{}{}", words[i % words.len()], ["", "ers", "Wide", "JAM", "Clap"][i % 5], i / 7);
            list.push(EmoteInfo { name, provider, url: format!("https://cdn.example/{i}"), scope, primary: true });
        }
        a.set_emotes(list);
        for i in 0..300 {
            a.observe(&format!("Viewer{i}"), &format!("viewer{i}"), vec![format!("{}0", words[i % words.len()])]);
        }
        let _ = a.suggest("x", Mode::Colon, &[], false, 8); // load the emoji data
        let queries = ["p", "po", "pog", "pogg", "c", "ca", "cat", "catj", "h", "he", "her", "herz", "lul", "kekw", "7 pe", "e fire"];
        let favs: Vec<String> = vec!["pog0".into(), "catJAM3".into()];
        let rounds = 200;
        let t = std::time::Instant::now();
        for _ in 0..rounds {
            for q in queries {
                std::hint::black_box(a.suggest(q, Mode::Colon, &favs, true, 8));
            }
        }
        let colon = t.elapsed() / (rounds * queries.len() as u32);
        let t = std::time::Instant::now();
        for _ in 0..rounds {
            for q in ["pe", "ca", "vi", "lu"] {
                std::hint::black_box(a.suggest(q, Mode::Tab, &favs, false, 20));
            }
        }
        let tab = t.elapsed() / (rounds * 4);
        let t = std::time::Instant::now();
        for _ in 0..rounds {
            std::hint::black_box(a.suggest("7", Mode::Browse, &favs, false, 200));
        }
        let browse = t.elapsed() / rounds;
        let t = std::time::Instant::now();
        for i in 0..rounds {
            a.record_sent(&format!("pog{i} catJAM3 gg 🔥❤️ nice"));
        }
        let sent = t.elapsed() / rounds;
        eprintln!("BENCH colon {colon:?}/query · tab {tab:?}/query · browse {browse:?}/query · record_sent {sent:?}/msg");
    }
}
