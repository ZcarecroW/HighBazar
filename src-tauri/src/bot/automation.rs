//! Automations around the stream: Shield Mode when chat is attacked, ad
//! breaks (pre-roll guard, announcements, snooze on raids) and channel point
//! rewards the bot answers. Only the decisions live here; the engine calls
//! Twitch and posts the messages.

use std::collections::{HashSet, VecDeque};

use serde_json::Value;

use crate::config::{AdSettings, AutoShieldSettings};

/// Finding categories that count as an attack on the chat.
pub fn is_attack(category: &str) -> bool {
    matches!(category, "hate" | "harassment" | "threat" | "sexual" | "extremism" | "doxxing" | "selfHarm")
}

// ---------------------------------------------------------------------------
// Auto Shield Mode
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct Shield {
    /// (time ms, user id) of recent attacks.
    attacks: VecDeque<(i64, String)>,
    last_attack: i64,
    /// Shield Mode was turned on by HighBazar (only then it turns it off again).
    pub ours: bool,
    /// A request to Twitch is running.
    pub busy: bool,
}

impl Shield {
    /// Records an attack; true when Shield Mode should go on now.
    pub fn on_attack(&mut self, s: &AutoShieldSettings, user_id: &str, now: i64) -> bool {
        self.last_attack = now;
        let window = i64::from(s.window_secs.max(10)) * 1000;
        self.attacks.push_back((now, user_id.to_string()));
        while self.attacks.front().is_some_and(|(t, _)| now - t > window) {
            self.attacks.pop_front();
        }
        let accounts: HashSet<&str> = self.attacks.iter().map(|(_, u)| u.as_str()).collect();
        s.enabled && !self.ours && !self.busy && accounts.len() as u32 >= s.attackers.max(2)
    }

    /// True when Shield Mode (turned on by us) can go off again.
    pub fn calm(&self, s: &AutoShieldSettings, now: i64) -> bool {
        self.ours && !self.busy && now - self.last_attack > i64::from(s.calm_mins.max(1)) * 60_000
    }

    pub fn activated(&mut self) {
        self.ours = true;
        self.attacks.clear();
    }
}

// ---------------------------------------------------------------------------
// Ads
// ---------------------------------------------------------------------------

/// Twitch's ad schedule (Get Ad Schedule), times as Unix seconds.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AdSchedule {
    pub next_ad_at: i64,
    pub last_ad_at: i64,
    pub duration: i64,
    pub preroll_free_secs: i64,
    pub snooze_count: i64,
}

impl AdSchedule {
    pub fn parse(v: &Value) -> Option<Self> {
        let d = v["data"].get(0)?;
        // Timestamps come as RFC 3339 strings or as Unix seconds (strings or numbers).
        let ts = |x: &Value| -> i64 {
            match x {
                Value::Number(n) => n.as_i64().unwrap_or(0),
                Value::String(s) if s.is_empty() || s == "0" => 0,
                Value::String(s) => s
                    .parse::<i64>()
                    .ok()
                    .or_else(|| chrono::DateTime::parse_from_rfc3339(s).ok().map(|d| d.timestamp()))
                    .unwrap_or(0),
                _ => 0,
            }
        };
        let num = |x: &Value| x.as_i64().or_else(|| x.as_str().and_then(|s| s.parse().ok())).unwrap_or(0);
        Some(Self {
            next_ad_at: ts(&d["next_ad_at"]),
            last_ad_at: ts(&d["last_ad_at"]),
            duration: num(&d["duration"]),
            preroll_free_secs: num(&d["preroll_free_time"]),
            snooze_count: num(&d["snooze_count"]),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AdStep {
    /// Announce an ad in this many seconds.
    Announce(i64),
    /// Start an ad of this length now.
    Run(u32),
}

#[derive(Default)]
pub struct Ads {
    pub schedule: Option<AdSchedule>,
    /// The ad (its start, Unix s) that was already announced.
    announced: i64,
    /// An ad runs until then (Unix s).
    pub running_until: i64,
    /// When HighBazar last started an ad itself (Unix s).
    last_run: i64,
}

/// Twitch allows a new ad only 8 minutes after the last one.
const AD_GAP_SECS: i64 = 8 * 60;

impl Ads {
    pub fn update(&mut self, sched: AdSchedule) {
        if sched.last_ad_at > 0 && sched.duration > 0 {
            self.running_until = self.running_until.max(sched.last_ad_at + sched.duration);
        }
        self.schedule = Some(sched);
    }

    pub fn ad_running(&self, now: i64) -> bool {
        now < self.running_until
    }

    pub fn started(&mut self, length: u32, now: i64) {
        self.running_until = now + i64::from(length);
        self.last_run = now;
    }

    /// What to do now, checked every few seconds while live.
    pub fn plan(&mut self, s: &AdSettings, now: i64) -> Vec<AdStep> {
        let mut steps = Vec::new();
        let Some(sch) = self.schedule.clone() else { return steps };
        if !s.auto {
            return steps;
        }
        if self.ad_running(now) {
            // Whatever was announced is happening now.
            self.announced = self.announced.max(0);
            return steps;
        }
        let since_last = now - sch.last_ad_at.max(self.last_run);
        let can_run = since_last >= AD_GAP_SECS;
        // Pre-roll guard: when the pre-roll free time is about to end, an ad now
        // keeps newcomers from getting a pre-roll – unless Twitch runs one soon anyway.
        let next_in = if sch.next_ad_at > 0 { sch.next_ad_at - now } else { i64::MAX };
        if s.preroll_guard && can_run && sch.preroll_free_secs <= 90 && next_in > 120 {
            let lead = if s.announce { i64::from(s.announce_secs.clamp(15, 300)) } else { 0 };
            // Announce first, then run when the announcement time is over.
            if lead > 0 && self.announced != -1 {
                self.announced = -1;
                steps.push(AdStep::Announce(lead));
                self.last_run = now - AD_GAP_SECS + lead; // runs `lead` seconds from now
                return steps;
            }
            self.announced = 0;
            self.started(s.length.clamp(30, 180), now);
            steps.push(AdStep::Run(s.length.clamp(30, 180)));
            return steps;
        }
        // Announce Twitch's scheduled ads.
        if s.announce && sch.next_ad_at > now && next_in <= i64::from(s.announce_secs.clamp(15, 300)) && self.announced != sch.next_ad_at {
            self.announced = sch.next_ad_at;
            steps.push(AdStep::Announce(next_in));
        }
        steps
    }

    /// Whether a raid should push the next ad back.
    pub fn snooze_for_raid(&self, s: &AdSettings, now: i64) -> bool {
        let Some(sch) = &self.schedule else { return false };
        s.auto && s.snooze_on_raid && sch.snooze_count > 0 && sch.next_ad_at > now && sch.next_ad_at - now < 10 * 60
    }
}

/// The chat message that announces an ad.
pub fn ad_announcement(s: &AdSettings, german: bool, secs: i64) -> String {
    let secs = secs.max(1);
    let custom = s.announce_text.trim();
    if !custom.is_empty() {
        return custom.replace("{s}", &secs.to_string());
    }
    if german {
        format!("Gleich kommt Werbung (in {secs} s) – kurz Wasser holen, wir sind sofort zurück! Danke, dass ihr den Stream unterstützt.")
    } else {
        format!("Ad break in {secs} s – grab a drink, we'll be right back! Thanks for supporting the stream.")
    }
}

// ---------------------------------------------------------------------------
// Channel point rewards
// ---------------------------------------------------------------------------

/// A redemption of a channel point reward (EventSub).
#[derive(Debug, Clone, Default)]
pub struct Redemption {
    pub id: String,
    pub reward_id: String,
    pub reward_title: String,
    pub user_name: String,
    pub input: String,
}

/// Task for the AI for a reward the bot answers; None for actions without AI.
pub fn reward_task(action: &str, r: &Redemption, instructions: &str, game: &str) -> Option<String> {
    let n = &r.user_name;
    let input = r.input.trim();
    let quoted = if input.is_empty() { String::new() } else { format!(" Their text: \"{input}\".") };
    let topic = if game.is_empty() { "gaming or the stream topic".to_string() } else { format!("\"{game}\"") };
    let text = match action {
        "ask" if input.is_empty() => return None,
        "ask" => format!(
            "@{n} redeemed the channel point reward \"{t}\" and asks you: \"{input}\"\nAnswer briefly and correctly, starting with @{n}. If you don't know or can't know, say so honestly and playfully.",
            t = r.reward_title
        ),
        "roast" => format!(
            "@{n} redeemed the channel point reward \"{t}\" to get roasted.{quoted}\nRoast them in one short, witty and good-natured line, starting with @{n}: tease harmless things (their name, their channel point spending, their timing), never looks, identity, family or anything hurtful.",
            t = r.reward_title
        ),
        "rhyme" => format!(
            "@{n} redeemed the channel point reward \"{t}\".{quoted}\nWrite a short, funny rhyme (two to four short lines in one message, separated by \" / \") about @{n} and the stream, starting with @{n}.",
            t = r.reward_title
        ),
        "fact" => format!(
            "@{n} redeemed the channel point reward \"{t}\". Share one surprising, true fact about {topic}, starting with @{n}.",
            t = r.reward_title
        ),
        "custom" => format!(
            "@{n} redeemed the channel point reward \"{t}\".{quoted}\nWhat the streamer wants you to do: {instr}\nStart with @{n}.",
            t = r.reward_title,
            instr = if instructions.trim().is_empty() { "Thank them in a fun way." } else { instructions.trim() }
        ),
        _ => return None,
    };
    Some(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn shield() -> AutoShieldSettings {
        AutoShieldSettings { enabled: true, attackers: 3, window_secs: 60, calm_mins: 10 }
    }

    #[test]
    fn shield_needs_several_accounts() {
        let mut s = Shield::default();
        let cfg = shield();
        assert!(!s.on_attack(&cfg, "a", 0));
        assert!(!s.on_attack(&cfg, "a", 1000), "one account is a case for a ban, not for the shield");
        assert!(!s.on_attack(&cfg, "b", 2000));
        assert!(s.on_attack(&cfg, "c", 3000));
        s.activated();
        assert!(!s.on_attack(&cfg, "d", 4000), "already on");
        assert!(!s.calm(&cfg, 4000 + 9 * 60_000));
        assert!(s.calm(&cfg, 4000 + 11 * 60_000));
    }

    #[test]
    fn shield_window_and_switch() {
        let mut s = Shield::default();
        let mut cfg = shield();
        s.on_attack(&cfg, "a", 0);
        s.on_attack(&cfg, "b", 1000);
        assert!(!s.on_attack(&cfg, "c", 120_000), "the first two are outside the window");
        cfg.enabled = false;
        let mut s = Shield::default();
        for (i, u) in ["a", "b", "c", "d"].iter().enumerate() {
            assert!(!s.on_attack(&cfg, u, i as i64));
        }
    }

    #[test]
    fn parses_ad_schedule() {
        let v = json!({"data":[{"next_ad_at":"2026-10-20T20:00:00Z","last_ad_at":"1760990000","duration":"90","preroll_free_time":600,"snooze_count":"2"}]});
        let s = AdSchedule::parse(&v).unwrap();
        assert_eq!(s.next_ad_at, chrono::DateTime::parse_from_rfc3339("2026-10-20T20:00:00Z").unwrap().timestamp());
        assert_eq!(s.last_ad_at, 1_760_990_000);
        assert_eq!((s.duration, s.preroll_free_secs, s.snooze_count), (90, 600, 2));
        let empty = json!({"data":[{"next_ad_at":"","last_ad_at":"","duration":0,"preroll_free_time":0,"snooze_count":0}]});
        assert_eq!(AdSchedule::parse(&empty).unwrap().next_ad_at, 0);
    }

    #[test]
    fn ads_announce_scheduled_breaks_once() {
        let s = AdSettings { auto: true, preroll_guard: false, ..Default::default() };
        let mut a = Ads::default();
        a.update(AdSchedule { next_ad_at: 1000 + 50, last_ad_at: 0, duration: 90, preroll_free_secs: 600, snooze_count: 1 });
        assert_eq!(a.plan(&s, 1000), vec![AdStep::Announce(50)]);
        assert!(a.plan(&s, 1010).is_empty(), "only once per ad");
        assert!(a.snooze_for_raid(&s, 1000));
        assert!(!a.snooze_for_raid(&AdSettings { snooze_on_raid: false, ..s.clone() }, 1000));
    }

    #[test]
    fn preroll_guard_announces_then_runs() {
        let s = AdSettings { auto: true, preroll_guard: true, announce: true, announce_secs: 60, length: 90, ..Default::default() };
        let mut a = Ads::default();
        let now = 100_000;
        a.update(AdSchedule { next_ad_at: now + 3600, last_ad_at: now - 3000, duration: 90, preroll_free_secs: 60, snooze_count: 3 });
        assert_eq!(a.plan(&s, now), vec![AdStep::Announce(60)]);
        assert!(a.plan(&s, now + 30).is_empty(), "waits for the announced time");
        assert_eq!(a.plan(&s, now + 61), vec![AdStep::Run(90)]);
        assert!(a.ad_running(now + 100));
        assert!(a.plan(&s, now + 120).is_empty());
        // Too soon after the last ad: nothing.
        let mut b = Ads::default();
        b.update(AdSchedule { next_ad_at: now + 3600, last_ad_at: now - 100, duration: 30, preroll_free_secs: 0, snooze_count: 0 });
        assert!(b.plan(&s, now).is_empty());
        // Switched off: nothing.
        let mut c = Ads::default();
        c.update(AdSchedule { next_ad_at: now + 30, last_ad_at: 0, duration: 30, preroll_free_secs: 0, snooze_count: 0 });
        assert!(c.plan(&AdSettings::default(), now).is_empty());
    }

    #[test]
    fn reward_tasks() {
        let r = Redemption { user_name: "Max".into(), reward_title: "Frag den Bot".into(), input: "Wer gewinnt?".into(), ..Default::default() };
        assert!(reward_task("ask", &r, "", "").unwrap().contains("Wer gewinnt?"));
        assert!(reward_task("ask", &Redemption { input: " ".into(), ..r.clone() }, "", "").is_none());
        assert!(reward_task("fact", &r, "", "Elden Ring").unwrap().contains("Elden Ring"));
        assert!(reward_task("custom", &r, "Sing a song", "").unwrap().contains("Sing a song"));
        assert!(reward_task("emoteOnly", &r, "", "").is_none());
        assert_eq!(ad_announcement(&AdSettings { announce_text: "Ads in {s}s".into(), ..Default::default() }, true, 60), "Ads in 60s");
    }
}
