//! Built-in, read-only profiles. They are regenerated on every start so that
//! app updates can improve them; users clone them to customize.

use super::types::*;

pub const DEFAULT_COMPANION: &str = "builtin-companion-balanced";
pub const DEFAULT_MODERATION: &str = "builtin-mod-balanced";

fn companion(id: &str, name: &str, description: &str, f: impl FnOnce(&mut CompanionProfile)) -> CompanionProfile {
    let mut p = CompanionProfile {
        id: id.into(),
        name: name.into(),
        description: description.into(),
        builtin: true,
        ..Default::default()
    };
    f(&mut p);
    p
}

pub fn companion_presets() -> Vec<CompanionProfile> {
    vec![
        companion(
            DEFAULT_COMPANION,
            "Balanced Companion",
            "Friendly, witty all-rounder. Reacts to chat, shares game facts, asks questions. Works for most streams.",
            |_| {},
        ),
        companion(
            "builtin-companion-booster",
            "Small Stream Booster",
            "Talks a bit more often and asks lots of questions to get lurkers chatting. Ideal for streams with few viewers.",
            |p| {
                p.interval_min_secs = 120;
                p.interval_max_secs = 210;
                p.max_unanswered = 6;
                p.energy = 65;
                p.mix.question = 9;
                p.mix.chat_reaction = 8;
                p.mix.stream_moment = 5;
                p.welcome.scope = "firstThisStream".into();
            },
        ),
        companion(
            "builtin-companion-cozy",
            "Cozy & Chill",
            "Calm, warm and laid-back. Fewer messages, gentle humor. Great for cozy games, art and just chatting.",
            |p| {
                p.persona = "A calm, warm and cozy chat companion. Speaks softly, appreciates the little things, makes everyone feel welcome.".into();
                p.humor = 45;
                p.energy = 25;
                p.snark = 5;
                p.emote_level = 1;
                p.interval_min_secs = 240;
                p.interval_max_secs = 420;
                p.mix.hype = 0;
                p.mix.joke = 2;
                p.mix.question = 7;
                p.mix.stream_moment = 5;
            },
        ),
        companion(
            "builtin-companion-hype",
            "Hype Buddy",
            "High energy, loves big moments, celebrates every play and every new chatter.",
            |p| {
                p.persona = "An enthusiastic hype buddy who gets excited about great plays, cheers people on and keeps the energy up — but never spams.".into();
                p.humor = 60;
                p.energy = 90;
                p.snark = 10;
                p.emote_level = 3;
                p.interval_min_secs = 150;
                p.interval_max_secs = 270;
                p.mix.hype = 8;
                p.mix.stream_moment = 7;
            },
        ),
        companion(
            "builtin-companion-expert",
            "Game Expert",
            "Knows the game inside out: facts, lore, tips and trivia. Researches the current game online.",
            |p| {
                p.persona = "A knowledgeable gaming nerd who loves lore, speedrun trivia, development history and useful tips — explains things briefly and with fun.".into();
                p.humor = 40;
                p.energy = 45;
                p.mix.game_fact = 9;
                p.mix.game_tip = 8;
                p.mix.trivia = 5;
                p.mix.joke = 2;
                p.mix.hype = 1;
            },
        ),
        companion(
            "builtin-companion-comedian",
            "Comedian",
            "Quick-witted and playful with light teasing. Jokes, puns and funny takes on what happens in chat.",
            |p| {
                p.persona = "A quick-witted comedian with a playful, slightly cheeky sense of humor. Teases lightly, never punches down.".into();
                p.humor = 90;
                p.energy = 65;
                p.snark = 45;
                p.mix.joke = 9;
                p.mix.chat_reaction = 9;
                p.mix.game_tip = 1;
            },
        ),
        companion(
            "builtin-companion-quiet",
            "Quiet Helper",
            "No timed messages. Only answers when addressed, welcomes newcomers, reacts to events and commands.",
            |p| {
                p.interval_enabled = false;
                p.energy = 40;
            },
        ),
    ]
}

fn moderation(id: &str, name: &str, description: &str, f: impl FnOnce(&mut ModerationProfile)) -> ModerationProfile {
    let mut p = ModerationProfile {
        id: id.into(),
        name: name.into(),
        description: description.into(),
        builtin: true,
        ..Default::default()
    };
    f(&mut p);
    p
}

fn set_cat(p: &mut ModerationProfile, cat: &str, clear: ModAction, border: ModAction) {
    if let Some(r) = p.categories.iter_mut().find(|r| r.category == cat) {
        r.clear_action = clear;
        r.borderline_action = border;
    }
}

pub fn moderation_presets() -> Vec<ModerationProfile> {
    vec![
        moderation(
            DEFAULT_MODERATION,
            "Balanced",
            "Acts only on clear hate, harassment, threats, scams and spam. Banter, swearing and criticism stay allowed. Borderline cases are only logged for you.",
            |_| {},
        ),
        moderation(
            "builtin-mod-relaxed",
            "Relaxed",
            "Maximum freedom. Only the most severe cases (hate, threats, doxxing, scams) are acted on; everything else is logged.",
            |p| {
                p.strictness = 25;
                p.min_confidence = 0.9;
                set_cat(p, "harassment", ModAction::new("delete", 0), ModAction::new("none", 0));
                set_cat(p, "sexual", ModAction::new("delete", 0), ModAction::new("none", 0));
                set_cat(p, "spam", ModAction::new("log", 0), ModAction::new("none", 0));
                set_cat(p, "selfHarm", ModAction::new("log", 0), ModAction::new("none", 0));
                p.rules.repeat_enabled = false;
                p.escalation.steps = vec![ModAction::new("timeout", 3600), ModAction::new("timeout", 86_400)];
            },
        ),
        moderation(
            "builtin-mod-family",
            "Family Friendly",
            "Strict. Also removes borderline content, sexual references, excessive caps and links from non-exempt users.",
            |p| {
                p.strictness = 80;
                p.min_confidence = 0.7;
                for r in p.categories.iter_mut() {
                    if r.borderline_action.kind == "log" {
                        r.borderline_action = ModAction::new("delete", 0);
                    }
                }
                set_cat(p, "sexual", ModAction::new("timeout", 1800), ModAction::new("delete", 0));
                p.rules.caps_enabled = true;
                p.rules.links = "block".into();
                p.rules.emote_spam_enabled = true;
            },
        ),
        moderation(
            "builtin-mod-suggest",
            "Suggest Only",
            "Never acts on its own. Flags problematic messages in the dashboard with a suggested action you can apply with one click.",
            |p| {
                p.mode = "suggest".into();
            },
        ),
        moderation(
            "builtin-mod-log",
            "Watch & Log",
            "Only observes and writes a moderation log. No actions, no suggestions.",
            |p| {
                p.mode = "log".into();
            },
        ),
    ]
}

fn ai(
    id: &str,
    name: &str,
    description: &str,
    provider: &str,
    model: &str,
    price_in: f64,
    price_out: f64,
    f: impl FnOnce(&mut AiProfile),
) -> AiProfile {
    let mut p = AiProfile {
        id: id.into(),
        name: name.into(),
        description: description.into(),
        builtin: true,
        provider: provider.into(),
        model: model.into(),
        price_input_per_mtok: price_in,
        price_output_per_mtok: price_out,
        ..Default::default()
    };
    f(&mut p);
    p
}

pub fn ai_presets() -> Vec<AiProfile> {
    vec![
        ai(
            "builtin-ai-claude-opus",
            "Claude Opus 5.5",
            "Anthropic's flagship. Best quality and humor; higher cost.",
            "anthropic",
            "claude-opus-5-5",
            4.0,
            20.0,
            |p| p.reasoning = "low".into(),
        ),
        ai(
            "builtin-ai-claude-sonnet",
            "Claude Sonnet 5.5",
            "Great quality at a moderate price.",
            "anthropic",
            "claude-sonnet-5-5",
            2.0,
            10.0,
            |p| p.reasoning = "low".into(),
        ),
        ai(
            "builtin-ai-claude-haiku",
            "Claude Haiku 5.5",
            "Fast and very cheap. Recommended for moderation and busy chats.",
            "anthropic",
            "claude-haiku-5-5",
            0.10,
            0.50,
            |p| p.reasoning = "low".into(),
        ),
        ai(
            "builtin-ai-openai-sol",
            "OpenAI GPT-6.1 Sol",
            "OpenAI's balanced model (Responses API).",
            "openai",
            "gpt-6.1-sol",
            2.0,
            10.0,
            |p| p.reasoning = "low".into(),
        ),
        ai(
            "builtin-ai-openai-luna",
            "OpenAI GPT-6 Luna",
            "OpenAI's fast, low-cost model.",
            "openai",
            "gpt-6-luna",
            0.10,
            0.50,
            |p| p.reasoning = "low".into(),
        ),
        ai(
            "builtin-ai-gemini-flash",
            "Gemini 3.8 Flash",
            "Google's recommended general-purpose model with Google Search grounding.",
            "gemini",
            "gemini-3.8-flash",
            0.75,
            3.75,
            |p| p.reasoning = "low".into(),
        ),
        ai(
            "builtin-ai-gemini-lite",
            "Gemini 3.5 Flash-Lite",
            "Google's cheapest high-volume model.",
            "gemini",
            "gemini-3.5-flash-lite",
            0.30,
            2.50,
            |p| p.reasoning = "minimal".into(),
        ),
        ai(
            "builtin-ai-openrouter-auto",
            "OpenRouter Auto",
            "Lets OpenRouter pick a model. One key for hundreds of models.",
            "openrouter",
            "openrouter/auto",
            0.0,
            0.0,
            |_| {},
        ),
        ai(
            "builtin-ai-mistral-small",
            "Mistral Small",
            "Mistral's efficient model (EU provider).",
            "mistral",
            "mistral-small-latest",
            0.0,
            0.0,
            |p| p.web_search = false,
        ),
        ai(
            "builtin-ai-deepseek",
            "DeepSeek Chat",
            "Low-cost model by DeepSeek.",
            "deepseek",
            "deepseek-chat",
            0.0,
            0.0,
            |p| p.web_search = false,
        ),
        ai(
            "builtin-ai-perplexity-sonar",
            "Perplexity Sonar",
            "Answers with built-in live web search.",
            "perplexity",
            "sonar",
            0.0,
            0.0,
            |_| {},
        ),
        ai(
            "builtin-ai-ollama",
            "Ollama (local)",
            "Runs models on your own PC for free. Pick an installed model in a copy of this profile.",
            "ollama",
            "llama3.2",
            0.0,
            0.0,
            |p| p.web_search = false,
        ),
        ai(
            "builtin-ai-lmstudio",
            "LM Studio (local)",
            "Uses the model currently loaded in LM Studio.",
            "lmstudio",
            "local-model",
            0.0,
            0.0,
            |p| p.web_search = false,
        ),
        ai(
            "builtin-ai-claude-cli",
            "Claude Code CLI (subscription)",
            "Uses your locally installed and logged-in Claude Code CLI. Whether automated bot use is covered by your plan's terms is your responsibility.",
            "claudeCli",
            "haiku",
            0.0,
            0.0,
            |p| p.web_search = false,
        ),
    ]
}
