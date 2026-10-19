//! All persisted configuration types. Every struct uses `#[serde(default)]`
//! so that older config files keep loading after new fields are added.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Root
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Config {
    pub version: u32,
    pub settings: AppSettings,
    pub accounts: Vec<TwitchAccount>,
    pub channels: Vec<ChannelProfile>,
    pub companion_profiles: Vec<CompanionProfile>,
    pub moderation_profiles: Vec<ModerationProfile>,
    pub ai_profiles: Vec<AiProfile>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            settings: AppSettings::default(),
            accounts: Vec::new(),
            channels: Vec::new(),
            companion_profiles: Vec::new(),
            moderation_profiles: Vec::new(),
            ai_profiles: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// App settings
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppSettings {
    /// "system" | "dark" | "light"
    pub theme: String,
    /// "auto" | "en" | "de"
    pub language: String,
    pub check_updates_on_start: bool,
    pub wizard_completed: bool,
    pub active_channel_id: Option<String>,
    pub start_bot_on_launch: bool,
    /// Overrides the built-in Twitch client id (advanced).
    pub twitch_client_id: String,
    /// Maximum number of entries kept in the live feed.
    pub feed_limit: u32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "system".into(),
            language: "auto".into(),
            check_updates_on_start: true,
            wizard_completed: false,
            active_channel_id: None,
            start_bot_on_launch: false,
            twitch_client_id: String::new(),
            feed_limit: 400,
        }
    }
}

// ---------------------------------------------------------------------------
// Twitch accounts (tokens live in the OS credential store)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TwitchAccount {
    pub id: String,
    pub login: String,
    pub display_name: String,
    pub profile_image_url: String,
    pub scopes: Vec<String>,
}

// ---------------------------------------------------------------------------
// Channel profile (= "global profile")
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ChannelProfile {
    pub id: String,
    pub name: String,
    /// Account that owns the channel. Chat is read and moderated with it.
    pub streamer_account_id: Option<String>,
    /// Optional separate account that sends the bot messages.
    pub bot_account_id: Option<String>,
    pub bot_name: String,
    pub prefix_enabled: bool,
    pub prefix: String,
    pub companion_enabled: bool,
    pub moderation_enabled: bool,
    pub companion_profile_id: String,
    pub moderation_profile_id: String,
    pub ai_profile_id: String,
    /// Empty = use `ai_profile_id`.
    pub moderation_ai_profile_id: String,
    /// "auto" | "preview" | "approve"
    pub send_mode: String,
    pub preview_delay_secs: u32,
    /// Free text about the streamer/stream the AI may use.
    pub about_stream: String,
    /// 0 = unlimited.
    pub daily_budget_usd: f64,
}

impl Default for ChannelProfile {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: "My channel".into(),
            streamer_account_id: None,
            bot_account_id: None,
            bot_name: "HighBazar".into(),
            prefix_enabled: true,
            prefix: "[HighBazar]".into(),
            companion_enabled: true,
            moderation_enabled: true,
            companion_profile_id: "builtin-companion-balanced".into(),
            moderation_profile_id: "builtin-mod-balanced".into(),
            ai_profile_id: String::new(),
            moderation_ai_profile_id: String::new(),
            send_mode: "auto".into(),
            preview_delay_secs: 10,
            about_stream: String::new(),
            daily_budget_usd: 2.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Companion profile
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CompanionProfile {
    pub id: String,
    pub name: String,
    pub description: String,
    pub builtin: bool,

    // Personality
    pub persona: String,
    pub humor: u8,
    pub energy: u8,
    pub snark: u8,
    /// 0 = never, 1 = rarely, 2 = sometimes, 3 = often
    pub emote_level: u8,
    pub custom_instructions: String,
    pub avoid_topics: String,
    pub max_length: u32,

    // Timing
    pub interval_enabled: bool,
    pub interval_min_secs: u32,
    pub interval_max_secs: u32,
    pub only_when_live: bool,
    pub adaptive: bool,
    /// Human messages per minute above which interval messages are skipped.
    pub busy_threshold: u32,
    /// Stop interval messages after this many in a row without any human
    /// chat message in between (0 = never stop).
    pub max_unanswered: u32,

    // Context
    pub context_messages: u32,
    pub context_max_age_mins: u32,

    pub mix: ContentMix,
    pub language: LanguageSettings,
    pub emotes: EmoteSettings,
    pub mentions: MentionSettings,
    pub welcome: WelcomeSettings,
    pub events: EventSettings,
    pub commands: CommandSettings,
    pub research: ResearchSettings,
}

impl Default for CompanionProfile {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: "New profile".into(),
            description: String::new(),
            builtin: false,
            persona: "A friendly, witty and curious chat companion who loves games and the community. Warm, playful, never mean.".into(),
            humor: 60,
            energy: 55,
            snark: 20,
            emote_level: 2,
            custom_instructions: String::new(),
            avoid_topics: "politics, religion, real-world tragedies, viewer counts, begging for follows or subs".into(),
            max_length: 220,
            interval_enabled: true,
            interval_min_secs: 180,
            interval_max_secs: 300,
            only_when_live: true,
            adaptive: true,
            busy_threshold: 25,
            max_unanswered: 4,
            context_messages: 40,
            context_max_age_mins: 20,
            mix: ContentMix::default(),
            language: LanguageSettings::default(),
            emotes: EmoteSettings::default(),
            mentions: MentionSettings::default(),
            welcome: WelcomeSettings::default(),
            events: EventSettings::default(),
            commands: CommandSettings::default(),
            research: ResearchSettings::default(),
        }
    }
}

/// Relative weights (0–10) for the kind of interval message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ContentMix {
    pub chat_reaction: u8,
    pub game_fact: u8,
    pub game_tip: u8,
    pub trivia: u8,
    pub joke: u8,
    pub question: u8,
    pub hype: u8,
    pub stream_moment: u8,
}

impl Default for ContentMix {
    fn default() -> Self {
        Self {
            chat_reaction: 8,
            game_fact: 6,
            game_tip: 3,
            trivia: 3,
            joke: 4,
            question: 6,
            hype: 3,
            stream_moment: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LanguageSettings {
    /// "stream" (Twitch stream language) | "fixed" | "chat" (majority of chat)
    pub mode: String,
    /// ISO code used for "fixed" and as fallback.
    pub fixed: String,
    /// Reply to a chatter in the language they wrote in.
    pub reply_in_user_language: bool,
    /// Allow English gaming slang / memes even in other languages.
    pub allow_slang: bool,
}

impl Default for LanguageSettings {
    fn default() -> Self {
        Self {
            mode: "stream".into(),
            fixed: "en".into(),
            reply_in_user_language: true,
            allow_slang: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EmoteSettings {
    pub twitch: bool,
    pub bttv: bool,
    pub ffz: bool,
    pub seventv: bool,
    /// Prefer emotes that this chat actually uses.
    pub prefer_popular: bool,
    pub max_per_message: u8,
}

impl Default for EmoteSettings {
    fn default() -> Self {
        Self {
            twitch: true,
            bttv: true,
            ffz: true,
            seventv: true,
            prefer_popular: true,
            max_per_message: 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MentionSettings {
    pub enabled: bool,
    /// Extra words that count as addressing the bot (bot name always counts).
    pub trigger_words: Vec<String>,
    /// Treat replies to the bot's messages as mentions.
    pub replies: bool,
    pub user_cooldown_secs: u32,
    pub global_cooldown_secs: u32,
    /// "everyone" | "subscriber" | "vip" | "moderator"
    pub min_role: String,
}

impl Default for MentionSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            trigger_words: Vec::new(),
            replies: true,
            user_cooldown_secs: 30,
            global_cooldown_secs: 4,
            min_role: "everyone".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WelcomeSettings {
    pub enabled: bool,
    /// "firstEver" | "firstThisStream"
    pub scope: String,
    pub delay_secs: u32,
    pub max_per_minute: u32,
    pub instructions: String,
}

impl Default for WelcomeSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            scope: "firstEver".into(),
            delay_secs: 4,
            max_per_minute: 3,
            instructions: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EventSettings {
    pub follow: bool,
    pub sub: bool,
    pub resub: bool,
    pub gift: bool,
    pub raid: bool,
    pub cheer: bool,
    pub min_cheer_bits: u32,
    /// Collect follows for this many seconds and thank them together.
    pub follow_batch_secs: u32,
}

impl Default for EventSettings {
    fn default() -> Self {
        Self {
            follow: true,
            sub: true,
            resub: true,
            gift: true,
            raid: true,
            cheer: true,
            min_cheer_bits: 100,
            follow_batch_secs: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CommandSettings {
    pub enabled: bool,
    pub prefix: String,
    pub ask: CommandConfig,
    pub fact: CommandConfig,
    pub joke: CommandConfig,
    pub about: CommandConfig,
}

impl Default for CommandSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            prefix: "!hb".into(),
            ask: CommandConfig::new(60, 5, "everyone"),
            fact: CommandConfig::new(120, 20, "everyone"),
            joke: CommandConfig::new(120, 20, "everyone"),
            about: CommandConfig::new(60, 30, "everyone"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CommandConfig {
    pub enabled: bool,
    pub user_cooldown_secs: u32,
    pub global_cooldown_secs: u32,
    pub min_role: String,
}

impl CommandConfig {
    pub fn new(user: u32, global: u32, role: &str) -> Self {
        Self {
            enabled: true,
            user_cooldown_secs: user,
            global_cooldown_secs: global,
            min_role: role.into(),
        }
    }
}

impl Default for CommandConfig {
    fn default() -> Self {
        Self::new(60, 10, "everyone")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ResearchSettings {
    pub enabled: bool,
    /// Ask the AI provider to use its built-in web search (if supported).
    pub web_search: bool,
    pub wikipedia: bool,
    pub refresh_days: u32,
}

impl Default for ResearchSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            web_search: true,
            wikipedia: true,
            refresh_days: 14,
        }
    }
}

// ---------------------------------------------------------------------------
// Moderation profile
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ModerationProfile {
    pub id: String,
    pub name: String,
    pub description: String,
    pub builtin: bool,

    /// "off" | "log" | "suggest" | "auto"
    pub mode: String,
    /// "off" | "suspicious" | "all"
    pub ai_check: String,
    /// 0 (very lenient) … 100 (very strict)
    pub strictness: u8,
    /// Auto actions on "clear" verdicts need at least this confidence (0–1).
    pub min_confidence: f32,
    /// Max AI moderation calls per minute (messages are batched).
    pub max_ai_calls_per_min: u32,
    /// Channel rules in plain language for the AI.
    pub channel_rules: String,

    pub exempt: Exemptions,
    pub categories: Vec<CategoryRule>,
    pub escalation: Escalation,
    pub rules: LocalRules,
    /// Reason sent to Twitch; `{reason}` is replaced.
    pub reason_template: String,
    /// Post a short, neutral note in chat after an automatic action.
    pub announce_actions: bool,
}

impl Default for ModerationProfile {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: "New profile".into(),
            description: String::new(),
            builtin: false,
            mode: "auto".into(),
            ai_check: "all".into(),
            strictness: 50,
            min_confidence: 0.8,
            max_ai_calls_per_min: 12,
            channel_rules: String::new(),
            exempt: Exemptions::default(),
            categories: default_categories(),
            escalation: Escalation::default(),
            rules: LocalRules::default(),
            reason_template: "[HighBazar] {reason}".into(),
            announce_actions: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Exemptions {
    pub moderators: bool,
    pub vips: bool,
    pub subscribers: bool,
    /// Logins (lowercase) that are never moderated.
    pub users: Vec<String>,
}

impl Default for Exemptions {
    fn default() -> Self {
        Self {
            moderators: true,
            vips: true,
            subscribers: false,
            users: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct ModAction {
    /// "none" | "log" | "delete" | "warn" | "timeout" | "ban"
    pub kind: String,
    pub duration_secs: u32,
}

impl ModAction {
    pub fn new(kind: &str, duration_secs: u32) -> Self {
        Self {
            kind: kind.into(),
            duration_secs,
        }
    }

    /// Severity ranking used for escalation comparisons.
    pub fn rank(&self) -> u32 {
        match self.kind.as_str() {
            "log" => 1,
            "delete" => 2,
            "warn" => 3,
            "timeout" => 10 + self.duration_secs.min(1_209_600) / 60,
            "ban" => 1_000_000,
            _ => 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CategoryRule {
    /// hate | harassment | threat | sexual | selfHarm | spam | scam | doxxing | extremism
    pub category: String,
    pub enabled: bool,
    pub clear_action: ModAction,
    pub borderline_action: ModAction,
}

impl Default for CategoryRule {
    fn default() -> Self {
        Self {
            category: String::new(),
            enabled: true,
            clear_action: ModAction::new("timeout", 600),
            borderline_action: ModAction::new("log", 0),
        }
    }
}

pub const CATEGORIES: &[&str] = &[
    "hate",
    "harassment",
    "threat",
    "sexual",
    "selfHarm",
    "spam",
    "scam",
    "doxxing",
    "extremism",
];

pub fn default_categories() -> Vec<CategoryRule> {
    let rule = |c: &str, clear: ModAction, border: ModAction| CategoryRule {
        category: c.into(),
        enabled: true,
        clear_action: clear,
        borderline_action: border,
    };
    vec![
        rule("hate", ModAction::new("timeout", 1800), ModAction::new("log", 0)),
        rule("harassment", ModAction::new("timeout", 600), ModAction::new("log", 0)),
        rule("threat", ModAction::new("timeout", 3600), ModAction::new("log", 0)),
        rule("sexual", ModAction::new("timeout", 600), ModAction::new("log", 0)),
        // Category = telling others to hurt themselves ("kys"), not people in distress.
        rule("selfHarm", ModAction::new("timeout", 600), ModAction::new("log", 0)),
        rule("spam", ModAction::new("delete", 0), ModAction::new("log", 0)),
        rule("scam", ModAction::new("ban", 0), ModAction::new("delete", 0)),
        rule("doxxing", ModAction::new("timeout", 3600), ModAction::new("delete", 0)),
        rule("extremism", ModAction::new("timeout", 3600), ModAction::new("log", 0)),
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Escalation {
    pub enabled: bool,
    pub window_mins: u32,
    /// Action for the 2nd, 3rd, … strike within the window.
    pub steps: Vec<ModAction>,
}

impl Default for Escalation {
    fn default() -> Self {
        Self {
            enabled: true,
            window_mins: 120,
            steps: vec![
                ModAction::new("timeout", 1800),
                ModAction::new("timeout", 86_400),
                ModAction::new("ban", 0),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LocalRules {
    /// One term per entry; `*` is a wildcard. Matching ignores case and
    /// common character substitutions.
    pub blocked_terms: Vec<String>,
    pub blocked_terms_action: ModAction,
    /// Built-in list of typical viewer-bot / phishing spam phrases.
    pub scam_phrases_enabled: bool,
    pub scam_phrases_action: ModAction,
    /// "allow" | "block" (non-exempt users)
    pub links: String,
    pub allowed_domains: Vec<String>,
    pub links_action: ModAction,
    pub caps_enabled: bool,
    pub caps_min_length: u32,
    pub caps_ratio: f32,
    pub caps_action: ModAction,
    pub repeat_enabled: bool,
    /// Same message from the same user this many times within 60 s.
    pub repeat_count: u32,
    pub repeat_action: ModAction,
    pub emote_spam_enabled: bool,
    pub max_emotes: u32,
    pub emote_spam_action: ModAction,
    pub max_length: u32,
    pub length_action: ModAction,
}

impl Default for LocalRules {
    fn default() -> Self {
        Self {
            blocked_terms: Vec::new(),
            blocked_terms_action: ModAction::new("delete", 0),
            scam_phrases_enabled: true,
            scam_phrases_action: ModAction::new("delete", 0),
            links: "allow".into(),
            allowed_domains: vec![
                "twitch.tv".into(),
                "clips.twitch.tv".into(),
                "youtube.com".into(),
                "youtu.be".into(),
            ],
            links_action: ModAction::new("delete", 0),
            caps_enabled: false,
            caps_min_length: 20,
            caps_ratio: 0.8,
            caps_action: ModAction::new("delete", 0),
            repeat_enabled: true,
            repeat_count: 4,
            repeat_action: ModAction::new("timeout", 60),
            emote_spam_enabled: false,
            max_emotes: 15,
            emote_spam_action: ModAction::new("delete", 0),
            max_length: 0,
            length_action: ModAction::new("delete", 0),
        }
    }
}

// ---------------------------------------------------------------------------
// AI profile
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AiProfile {
    pub id: String,
    pub name: String,
    pub description: String,
    pub builtin: bool,
    /// anthropic | openai | gemini | openrouter | mistral | groq | xai |
    /// deepseek | perplexity | together | ollama | lmstudio | openaiCompatible | claudeCli
    pub provider: String,
    /// Empty = provider default.
    pub base_url: String,
    pub model: String,
    /// Which stored API key to use. Empty = the provider's shared key.
    pub key_ref: String,
    pub max_output_tokens: u32,
    /// None = provider default. Ignored by models that don't support it.
    pub temperature: Option<f32>,
    /// Provider specific: Anthropic effort, OpenAI reasoning effort,
    /// Gemini thinking level. Empty = provider default.
    pub reasoning: String,
    pub web_search: bool,
    pub timeout_secs: u32,
    pub price_input_per_mtok: f64,
    pub price_output_per_mtok: f64,
    pub extra_headers: Vec<HeaderPair>,
    /// Use the OpenAI Responses API (true) or Chat Completions (false).
    pub openai_responses: bool,
    /// Gemini: use the Interactions API (true) or generateContent (false).
    pub gemini_interactions: bool,
    /// Path to the `claude` executable (Claude Code CLI bridge).
    pub cli_path: String,
}

impl Default for AiProfile {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: "New AI profile".into(),
            description: String::new(),
            builtin: false,
            provider: "anthropic".into(),
            base_url: String::new(),
            model: String::new(),
            key_ref: String::new(),
            max_output_tokens: 2048,
            temperature: None,
            reasoning: String::new(),
            web_search: true,
            timeout_secs: 90,
            price_input_per_mtok: 0.0,
            price_output_per_mtok: 0.0,
            extra_headers: Vec::new(),
            openai_responses: true,
            gemini_interactions: true,
            cli_path: String::new(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct HeaderPair {
    pub name: String,
    pub value: String,
}
