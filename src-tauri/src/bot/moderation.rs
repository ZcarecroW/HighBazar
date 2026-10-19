//! Moderation logic: instant local rules, AI verdict parsing and action
//! selection with escalation. Executing actions happens in the engine.

use std::collections::{HashMap, VecDeque};

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::{ModAction, ModerationProfile};
use crate::twitch::eventsub::ChatMessage;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ModLogEntry {
    pub id: String,
    pub ts: i64,
    pub user_id: String,
    pub login: String,
    pub display_name: String,
    pub message_id: String,
    pub text: String,
    pub category: String,
    /// "violation" | "borderline"
    pub verdict: String,
    pub confidence: f32,
    pub reason: String,
    /// "ai" | "rule:<name>" | "manual"
    pub source: String,
    pub action: ModAction,
    /// "logged" | "suggested" | "executed" | "simulated" | "failed" | "dismissed" | "undone"
    pub status: String,
    pub error: String,
}

#[derive(Debug, Clone)]
pub struct Finding {
    pub category: String,
    pub verdict: String,
    pub confidence: f32,
    pub reason: String,
    pub source: String,
    /// Set for local rules, which carry their own action.
    pub action: Option<ModAction>,
}

pub fn is_exempt(p: &ModerationProfile, m: &ChatMessage) -> bool {
    m.is_broadcaster
        || (p.exempt.moderators && m.is_mod)
        || (p.exempt.vips && m.is_vip)
        || (p.exempt.subscribers && m.is_sub)
        || p.exempt.users.iter().any(|u| u.trim().eq_ignore_ascii_case(&m.login))
}

/// Folds common obfuscations so blocked terms still match ("h3ll0" → "hello").
pub fn fold(text: &str) -> String {
    let chars: Vec<char> = text.to_lowercase().chars().collect();
    // Symbols only count as letters inside a word ("k!ll", "a$$"), so
    // punctuation and @mentions keep working as word boundaries.
    let inside = |i: usize| {
        i > 0
            && i + 1 < chars.len()
            && chars[i - 1].is_alphanumeric()
            && (chars[i + 1].is_alphanumeric() || "!|@$".contains(chars[i + 1]))
    };
    chars
        .iter()
        .enumerate()
        .map(|(i, &c)| match c {
            '!' | '|' if inside(i) => 'i',
            '@' if inside(i) => 'a',
            '$' if inside(i) => 's',
            '0' => 'o',
            '1' => 'i',
            '3' => 'e',
            '4' => 'a',
            '5' => 's',
            '7' => 't',
            '8' => 'b',
            'á' | 'à' | 'â' => 'a',
            'é' | 'è' | 'ê' => 'e',
            'í' | 'ì' | 'î' => 'i',
            'ó' | 'ò' | 'ô' => 'o',
            'ú' | 'ù' | 'û' => 'u',
            c => c,
        })
        .collect()
}

/// Compiles a blocked term with `*` wildcards into a word-boundary regex.
fn term_regex(term: &str) -> Option<Regex> {
    let t = fold(term.trim());
    if t.is_empty() {
        return None;
    }
    let pattern = t.split('*').map(regex::escape).collect::<Vec<_>>().join(r"\S*");
    Regex::new(&format!(r"(?:^|[^\p{{L}}\p{{N}}]){pattern}(?:$|[^\p{{L}}\p{{N}}])")).ok()
}

pub struct RuleState {
    terms: Vec<(String, Regex)>,
    terms_source: Vec<String>,
    url_re: Regex,
    /// Recent message texts per user for repeat detection: (ts_ms, text).
    history: HashMap<String, VecDeque<(i64, String)>>,
}

impl Default for RuleState {
    fn default() -> Self {
        Self {
            terms: Vec::new(),
            terms_source: Vec::new(),
            url_re: Regex::new(r"(?i)\b((?:https?://)?(?:[a-z0-9-]+\.)+(?:com|net|org|io|gg|tv|de|ru|xyz|ly|me|co|info|biz|app|link|click|top|shop|store|live|site|online|uk|us|eu|fr|be|nl|at|ch|pl|cc|to|in)\b(?:/\S*)?)").unwrap(),
            history: HashMap::new(),
        }
    }
}

impl RuleState {
    fn sync_terms(&mut self, terms: &[String]) {
        if self.terms_source != terms {
            self.terms = terms
                .iter()
                .filter_map(|t| term_regex(t).map(|r| (t.clone(), r)))
                .collect();
            self.terms_source = terms.to_vec();
        }
    }

    fn domain_allowed(domain: &str, allowed: &[String]) -> bool {
        let d = domain.to_lowercase();
        let host = d
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .split('/')
            .next()
            .unwrap_or("")
            .trim_start_matches("www.");
        allowed.iter().any(|a| {
            let a = a.trim().to_lowercase();
            !a.is_empty() && (host == a || host.ends_with(&format!(".{a}")))
        })
    }

    /// Instant, free checks. Returns the first rule that matches.
    pub fn check(&mut self, p: &ModerationProfile, m: &ChatMessage, emote_count: u32) -> Option<Finding> {
        let r = &p.rules;
        let found = |name: &str, category: &str, reason: String, action: &ModAction| Finding {
            category: category.into(),
            verdict: "violation".into(),
            confidence: 1.0,
            reason,
            source: format!("rule:{name}"),
            action: Some(action.clone()),
        };

        // Repeat detection keeps its history even if no rule fires.
        let now = m.ts;
        let hist = self.history.entry(m.user_id.clone()).or_default();
        while hist.front().is_some_and(|(t, _)| now - t > 60_000) {
            hist.pop_front();
        }
        let norm = m.text.trim().to_lowercase();
        let repeats = hist.iter().filter(|(_, t)| *t == norm).count() as u32 + 1;
        hist.push_back((now, norm));
        while hist.len() > 20 {
            hist.pop_front();
        }
        if self.history.len() > 5000 {
            self.history.retain(|_, h| h.back().is_some_and(|(t, _)| now - t < 60_000));
        }

        self.sync_terms(&r.blocked_terms);
        let folded = fold(&m.text);
        for (term, re) in &self.terms {
            if re.is_match(&folded) {
                return Some(found("blockedTerm", "blockedTerm", format!("blocked term \"{term}\""), &r.blocked_terms_action));
            }
        }
        if r.scam_phrases_enabled
            && let Some(phrase) = SCAM_PHRASES.iter().find(|s| folded.contains(*s)) {
                return Some(found("scam", "scam", format!("known spam phrase \"{phrase}\""), &r.scam_phrases_action));
            }
        if r.links == "block"
            && let Some(cap) = self.url_re.captures(&m.text) {
                let url = cap.get(1).map(|x| x.as_str()).unwrap_or_default();
                if !Self::domain_allowed(url, &r.allowed_domains) {
                    return Some(found("links", "spam", format!("link not allowed ({url})"), &r.links_action));
                }
            }
        if r.repeat_enabled && r.repeat_count > 1 && repeats >= r.repeat_count {
            return Some(found("repeat", "spam", format!("same message {repeats}× within a minute"), &r.repeat_action));
        }
        if r.caps_enabled {
            let letters: Vec<char> = m.text.chars().filter(|c| c.is_alphabetic()).collect();
            if letters.len() as u32 >= r.caps_min_length {
                let upper = letters.iter().filter(|c| c.is_uppercase()).count() as f32;
                if upper / letters.len() as f32 >= r.caps_ratio {
                    return Some(found("caps", "spam", "excessive caps".into(), &r.caps_action));
                }
            }
        }
        if r.emote_spam_enabled && emote_count > r.max_emotes {
            return Some(found("emoteSpam", "spam", format!("{emote_count} emotes"), &r.emote_spam_action));
        }
        if r.max_length > 0 && m.text.chars().count() as u32 > r.max_length {
            return Some(found("length", "spam", "message too long".into(), &r.length_action));
        }
        None
    }
}

/// Phrases of the viewer-bot and gift-card spam that hits almost every
/// channel (matched against the folded text, so case and leetspeak don't matter).
const SCAM_PHRASES: &[&str] = &[
    "cheap viewers", "cheap followers", "best viewers", "buy viewers", "buy followers", "free followers",
    "free viewers", "primes and viewers", "primes, viewers", "wanna become famous", "want to become famous",
    "streamboo", "dogehype", "bigfollows", "free nitro", "nitro for free", "steam gift card for free",
    "zuschauer kaufen", "follower kaufen", "günstige zuschauer", "gunstige zuschauer",
];

/// Cheap pre-filter for the "suspicious only" AI mode.
pub fn is_suspicious(text: &str) -> bool {
    const MARKERS: &[&str] = &[
        "kys", "kill yourself", "kill urself", "die ", "hope you die", "idiot", "stupid", "dumb", "retard", "loser",
        "trash", "ugly", "hate ", "nazi", "hitler", "rape", "porn", "nude", "onlyfans", "f*ck", "fuck", "shit", "bitch",
        "whore", "slut", "cunt", "fag", "gay", "kanake", "hurensohn", "wichser", "fotze", "spast", "behindert",
        "missgeburt", "schlampe", "neger", "opfer", "stirb", "verreck", "cheap viewers", "free followers", "best viewers",
        "giveaway", "free nitro", "steam gift", "bit.ly", "discord.gg", "http", "www.", ".com", "@", "address", "adresse",
        "wohnt in", "lives in", "phone", "telefon",
    ];
    let f = fold(text);
    if MARKERS.iter().any(|m| f.contains(m)) {
        return true;
    }
    let letters: Vec<char> = text.chars().filter(|c| c.is_alphabetic()).collect();
    letters.len() > 15 && letters.iter().filter(|c| c.is_uppercase()).count() * 10 > letters.len() * 8
}

#[derive(Debug, Clone, Deserialize)]
struct AiResult {
    i: usize,
    #[serde(default)]
    verdict: String,
    #[serde(default)]
    category: String,
    #[serde(default)]
    confidence: f32,
    #[serde(default)]
    reason: String,
}

/// Extracts the first JSON object from a model answer (handles code fences
/// and chatter around the JSON).
pub fn extract_json(text: &str) -> Option<Value> {
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    if end <= start {
        return None;
    }
    serde_json::from_str(&text[start..=end]).ok()
}

pub fn parse_ai_findings(text: &str, count: usize) -> Vec<(usize, Finding)> {
    let Some(v) = extract_json(text) else {
        return Vec::new();
    };
    let Ok(results) = serde_json::from_value::<Vec<AiResult>>(v["results"].clone()) else {
        return Vec::new();
    };
    results
        .into_iter()
        .filter(|r| r.i < count && (r.verdict == "violation" || r.verdict == "borderline"))
        .map(|r| {
            (
                r.i,
                Finding {
                    category: normalize_category(&r.category),
                    verdict: r.verdict,
                    confidence: r.confidence.clamp(0.0, 1.0),
                    reason: r.reason.chars().take(200).collect(),
                    source: "ai".into(),
                    action: None,
                },
            )
        })
        .collect()
}

fn normalize_category(c: &str) -> String {
    let c = c.trim().to_lowercase().replace(['-', '_', ' '], "");
    match c.as_str() {
        "selfharm" | "suicide" => "selfHarm".into(),
        "threats" | "violence" => "threat".into(),
        "hatespeech" => "hate".into(),
        "bullying" | "insult" => "harassment".into(),
        "phishing" => "scam".into(),
        other => other.to_string(),
    }
}

/// Picks the action for a finding, including escalation for repeat
/// offenders. `prior_strikes` = strikes of this user inside the window.
pub fn decide(p: &ModerationProfile, f: &Finding, prior_strikes: usize) -> Option<ModAction> {
    let base = if let Some(a) = &f.action {
        a.clone()
    } else {
        let rule = p.categories.iter().find(|r| r.category == f.category && r.enabled)?;
        if f.verdict == "violation" && f.confidence >= p.min_confidence {
            rule.clear_action.clone()
        } else {
            rule.borderline_action.clone()
        }
    };
    if base.kind.is_empty() || base.kind == "none" {
        return None;
    }
    let punitive = matches!(base.kind.as_str(), "delete" | "warn" | "timeout" | "ban");
    if punitive && p.escalation.enabled && prior_strikes > 0 && !p.escalation.steps.is_empty() {
        let idx = (prior_strikes - 1).min(p.escalation.steps.len() - 1);
        let step = &p.escalation.steps[idx];
        if step.rank() > base.rank() {
            return Some(step.clone());
        }
    }
    Some(base)
}

pub fn describe_action(a: &ModAction) -> String {
    match a.kind.as_str() {
        "timeout" => format!("timeout {}", human_duration(a.duration_secs)),
        other => other.to_string(),
    }
}

/// Reason for a chat announcement. Rule reasons are replaced by a short
/// generic text so a blocked term or a spam link is never repeated in chat.
pub fn public_reason(source: &str, reason: &str, category: &str, de: bool) -> String {
    let rule = source.strip_prefix("rule:");
    let text = match (rule, de) {
        (Some("repeat"), false) => "repeated messages",
        (Some("repeat"), true) => "wiederholte Nachrichten",
        (Some("links"), false) => "links are not allowed",
        (Some("links"), true) => "Links sind nicht erlaubt",
        (Some("caps"), false) => "too many caps",
        (Some("caps"), true) => "zu viele Großbuchstaben",
        (Some("emoteSpam"), false) => "emote spam",
        (Some("emoteSpam"), true) => "Emote-Spam",
        (Some("length"), false) => "message too long",
        (Some("length"), true) => "Nachricht zu lang",
        (Some("blockedTerm"), false) => "blocked term",
        (Some("blockedTerm"), true) => "gesperrter Begriff",
        (Some(_), false) => "spam",
        (Some(_), true) => "Spam",
        (None, _) if !reason.is_empty() => return reason.to_string(),
        (None, _) => return category.to_string(),
    };
    text.to_string()
}

pub fn human_duration(secs: u32) -> String {
    match secs {
        0..=59 => format!("{secs}s"),
        60..=3599 => format!("{}m", secs / 60),
        3600..=86_399 => format!("{}h", secs / 3600),
        _ => format!("{}d", secs / 86_400),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(text: &str) -> ChatMessage {
        ChatMessage { id: "m".into(), user_id: "u".into(), login: "bob".into(), text: text.into(), ts: 0, ..Default::default() }
    }

    #[test]
    fn blocked_terms_with_wildcards_and_leetspeak() {
        let mut p = ModerationProfile::default();
        p.rules.blocked_terms = vec!["badword".into(), "spam*bot".into()];
        let mut s = RuleState::default();
        assert!(s.check(&p, &msg("you B4DW0RD!"), 0).is_some());
        assert!(s.check(&p, &msg("buy spamxxbot now"), 0).is_some());
        assert!(s.check(&p, &msg("notbadwordish"), 0).is_none());
    }

    #[test]
    fn links_respect_allowlist() {
        let mut p = ModerationProfile::default();
        p.rules.links = "block".into();
        p.rules.repeat_enabled = false;
        let mut s = RuleState::default();
        assert!(s.check(&p, &msg("look clips.twitch.tv/abc"), 0).is_none());
        assert!(s.check(&p, &msg("free stuff at scam-site.xyz/win"), 0).is_some());
    }

    #[test]
    fn scam_phrases_without_ai() {
        let mut p = ModerationProfile::default();
        assert!(!p.rules.scam_phrases_enabled, "off by default");
        p.rules.scam_phrases_enabled = true;
        let mut s = RuleState::default();
        let f = s.check(&p, &msg("Cheap VIEWERS on streamboo .com"), 0).unwrap();
        assert_eq!(f.source, "rule:scam");
        assert!(s.check(&p, &msg("FR33 F0LL0WERS here"), 0).is_some());
        assert!(s.check(&p, &msg("we have 12 viewers and a free evening"), 0).is_none());
        assert_eq!(public_reason("rule:blockedTerm", "blocked term \"x\"", "blockedTerm", true), "gesperrter Begriff");
    }

    #[test]
    fn repeat_detection() {
        let mut p = ModerationProfile::default();
        p.rules.repeat_count = 3;
        let mut s = RuleState::default();
        assert!(s.check(&p, &msg("same"), 0).is_none());
        assert!(s.check(&p, &msg("same"), 0).is_none());
        assert!(s.check(&p, &msg("same"), 0).is_some());
    }

    #[test]
    fn parses_ai_output_in_code_fence() {
        let out = "```json\n{\"results\":[{\"i\":1,\"verdict\":\"violation\",\"category\":\"Hate Speech\",\"confidence\":0.95,\"reason\":\"slur\"},{\"i\":9,\"verdict\":\"violation\",\"category\":\"spam\"}]}\n```";
        let f = parse_ai_findings(out, 3);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].0, 1);
        assert_eq!(f[0].1.category, "hate");
    }

    #[test]
    fn escalation_raises_action() {
        let p = ModerationProfile::default();
        let f = Finding { category: "harassment".into(), verdict: "violation".into(), confidence: 0.9, reason: String::new(), source: "ai".into(), action: None };
        assert_eq!(decide(&p, &f, 0).unwrap(), ModAction::new("timeout", 600));
        assert_eq!(decide(&p, &f, 1).unwrap(), ModAction::new("timeout", 1800));
        assert_eq!(decide(&p, &f, 5).unwrap(), ModAction::new("ban", 0));
        let low = Finding { confidence: 0.5, ..f };
        assert_eq!(decide(&p, &low, 0).unwrap().kind, "log");
    }
}
