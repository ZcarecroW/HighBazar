//! In-memory chat context: recent messages, activity, emote usage and the
//! bot's own recent output.

use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};

const MAX_LINES: usize = 300;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Line {
    pub ts: i64,
    pub user_id: String,
    pub login: String,
    pub name: String,
    pub text: String,
    /// "broadcaster" | "mod" | "vip" | "sub" | "" | "bot"
    pub role: &'static str,
}

#[derive(Default)]
pub struct ChatContext {
    pub lines: VecDeque<Line>,
    /// Timestamps (ms) of human messages, for the activity rate.
    human_ts: VecDeque<i64>,
    pub last_human_ts: i64,
    /// Interval messages since the last human message.
    pub unanswered: u32,
    pub emote_usage: HashMap<String, u32>,
    pub bot_recent: VecDeque<String>,
    pub recent_kinds: VecDeque<String>,
    pub used_facts: HashSet<String>,
    /// Texts the bot sent recently (to recognise its own messages when it
    /// posts through the streamer account).
    sent_texts: VecDeque<(String, i64)>,
    sent_ids: VecDeque<String>,
}

impl ChatContext {
    pub fn push_human(&mut self, line: Line, emotes: &[String]) {
        self.human_ts.push_back(line.ts);
        self.last_human_ts = line.ts;
        self.unanswered = 0;
        for e in emotes {
            *self.emote_usage.entry(e.clone()).or_insert(0) += 1;
        }
        self.push(line);
    }

    /// Registers a message that is about to be sent, before the request is
    /// made, because the chat event may arrive before the API answers.
    pub fn expect_own(&mut self, full_text: &str, ts: i64) {
        self.sent_texts.push_back((normalize(full_text), ts));
        while self.sent_texts.len() > 30 {
            self.sent_texts.pop_front();
        }
    }

    pub fn push_bot(&mut self, line: Line, message_id: Option<String>) {
        self.bot_recent.push_back(line.text.clone());
        while self.bot_recent.len() > 15 {
            self.bot_recent.pop_front();
        }
        if let Some(id) = message_id {
            self.sent_ids.push_back(id);
            while self.sent_ids.len() > 200 {
                self.sent_ids.pop_front();
            }
        }
        self.push(line);
    }

    fn push(&mut self, line: Line) {
        self.lines.push_back(line);
        while self.lines.len() > MAX_LINES {
            self.lines.pop_front();
        }
    }

    /// Counts words that appear in the emote catalog (third-party emotes are
    /// plain words in the message text).
    pub fn count_text_emotes(&mut self, text: &str, is_emote: impl Fn(&str) -> bool) {
        for w in text.split_whitespace() {
            if is_emote(w) {
                *self.emote_usage.entry(w.to_string()).or_insert(0) += 1;
            }
        }
    }

    pub fn is_own_message(&self, message_id: &str, text: &str, now_ms: i64) -> bool {
        if self.sent_ids.iter().any(|id| id == message_id) {
            return true;
        }
        let n = normalize(text);
        self.sent_texts.iter().any(|(t, ts)| *t == n && now_ms - ts < 30_000)
    }

    pub fn is_reply_to_bot(&self, parent_id: &str) -> bool {
        self.sent_ids.iter().any(|id| id == parent_id)
    }

    /// Human messages per minute over the last minute.
    pub fn rate_per_min(&mut self, now_ms: i64) -> u32 {
        while self.human_ts.front().is_some_and(|t| now_ms - t > 60_000) {
            self.human_ts.pop_front();
        }
        self.human_ts.len() as u32
    }

    pub fn recent(&self, max: usize, max_age_ms: i64, now_ms: i64) -> Vec<&Line> {
        let mut out: Vec<&Line> = self
            .lines
            .iter()
            .rev()
            .filter(|l| now_ms - l.ts <= max_age_ms)
            .take(max)
            .collect();
        out.reverse();
        out
    }

    pub fn popular_emotes(&self, n: usize) -> Vec<String> {
        let mut v: Vec<(&String, &u32)> = self.emote_usage.iter().collect();
        v.sort_by(|a, b| b.1.cmp(a.1));
        v.into_iter().take(n).map(|(e, _)| e.clone()).collect()
    }

    pub fn note_kind(&mut self, kind: &str) {
        self.recent_kinds.push_back(kind.to_string());
        while self.recent_kinds.len() > 4 {
            self.recent_kinds.pop_front();
        }
    }
}

fn normalize(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

/// Chatters seen in a channel, persisted between sessions so that
/// "first time in this channel" welcomes work.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SeenChatters {
    /// Unix seconds when tracking started for this channel.
    pub since: i64,
    pub ids: HashSet<String>,
}

impl SeenChatters {
    /// During the first 20 minutes of tracking we can't tell regulars from
    /// newcomers, so only Twitch's own "first chat" intro counts.
    pub fn learning(&self, now: i64) -> bool {
        now - self.since < 20 * 60
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(ts: i64, text: &str) -> Line {
        Line { ts, user_id: "1".into(), login: "a".into(), name: "A".into(), text: text.into(), role: "" }
    }

    #[test]
    fn recognises_own_messages_by_text() {
        let mut c = ChatContext::default();
        c.expect_own("Hello  Chat", 1000);
        assert!(c.is_own_message("x", "hello chat", 2000));
        assert!(!c.is_own_message("x", "hello chat", 40_000));
    }

    #[test]
    fn rate_and_unanswered() {
        let mut c = ChatContext::default();
        c.unanswered = 3;
        c.push_human(line(0, "a"), &[]);
        c.push_human(line(30_000, "b"), &["Kappa".into()]);
        assert_eq!(c.unanswered, 0);
        assert_eq!(c.rate_per_min(70_000), 1);
        assert_eq!(c.popular_emotes(1), vec!["Kappa".to_string()]);
    }
}
