//! Background knowledge about the current game: Wikipedia summary plus an
//! AI research pass (optionally with the provider's web search). Results are
//! cached on disk per game.

use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::moderation::extract_json;
use crate::ai::{self, AiRequest};
use crate::config::AiProfile;
use crate::storage;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Knowledge {
    pub game: String,
    pub created_at: i64,
    pub summary: String,
    pub facts: Vec<String>,
    pub tips: Vec<String>,
    pub sources: Vec<String>,
}

impl Knowledge {
    pub fn items(&self) -> impl Iterator<Item = &String> {
        self.facts.iter().chain(self.tips.iter())
    }
}

/// Categories that are not games – no research for these.
pub fn is_non_game(category: &str) -> bool {
    const NON_GAMES: &[&str] = &[
        "just chatting", "music", "art", "talk shows & podcasts", "irl", "asmr", "software and game development",
        "pools, hot tubs, and beaches", "special events", "sports", "travel & outdoors", "food & drink",
        "makers & crafting", "science & technology", "beauty & body art", "fitness & health", "politics",
        "co-working & studying", "animals, aquariums, and zoos", "games + demos", "",
    ];
    NON_GAMES.contains(&category.trim().to_lowercase().as_str())
}

fn cache_path(dir: &Path, game: &str) -> std::path::PathBuf {
    dir.join("research").join(format!("{}.json", storage::slug(game)))
}

pub fn load_cached(dir: &Path, game: &str, max_age_days: u32) -> Option<Knowledge> {
    let k: Knowledge = storage::read_json(&cache_path(dir, game))?;
    let age = chrono::Utc::now().timestamp() - k.created_at;
    (age < max_age_days.max(1) as i64 * 86_400 && !k.facts.is_empty()).then_some(k)
}

async fn wikipedia(http: &reqwest::Client, game: &str) -> Option<(String, String)> {
    let search: Value = http
        .get("https://en.wikipedia.org/w/api.php")
        .query(&[
            ("action", "query"),
            ("list", "search"),
            ("srsearch", &format!("{game} video game")),
            ("srlimit", "1"),
            ("format", "json"),
        ])
        .header("User-Agent", "HighBazar/1.0 (https://github.com/ZcarecroW/HighBazar)")
        .send()
        .await
        .ok()?
        .json()
        .await
        .ok()?;
    let title = search["query"]["search"][0]["title"].as_str()?.to_string();
    let summary: Value = http
        .get(format!(
            "https://en.wikipedia.org/api/rest_v1/page/summary/{}",
            url::form_urlencoded::byte_serialize(title.replace(' ', "_").as_bytes()).collect::<String>()
        ))
        .header("User-Agent", "HighBazar/1.0 (https://github.com/ZcarecroW/HighBazar)")
        .send()
        .await
        .ok()?
        .json()
        .await
        .ok()?;
    let extract = summary["extract"].as_str()?.to_string();
    let url = summary["content_urls"]["desktop"]["page"].as_str().unwrap_or_default().to_string();
    Some((extract, url))
}

fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .take(25)
                .collect()
        })
        .unwrap_or_default()
}

/// Researches a game. Returns the knowledge, the AI cost in USD and the
/// number of web searches.
#[allow(clippy::too_many_arguments)]
pub async fn research(
    http: &reqwest::Client,
    dir: &Path,
    game: &str,
    profile: &AiProfile,
    key: Option<&str>,
    use_web: bool,
    use_wiki: bool,
    own: &ai::OwnSearch<'_>,
) -> Result<(Knowledge, f64, u32), String> {
    let wiki = if use_wiki { wikipedia(http, game).await } else { None };
    let (system, user) = super::prompt::research_prompt(game, wiki.as_ref().map(|w| w.0.as_str()).unwrap_or(""), use_web);
    let req = AiRequest {
        system,
        user,
        max_tokens: Some(4000),
        web_search: use_web,
        // Not triggered by chat, so reading patch notes etc. is fine.
        web_fetch: true,
        // Without the provider's own search, HighBazar looks this up first;
        // the year keeps old patch notes from ranking first.
        search_query: Some(format!("{game} game latest update patch notes {}", chrono::Local::now().format("%Y"))),
        background: true,
    };
    let resp = ai::complete_searching(http, profile, key, &req, own).await.map_err(|e| e.to_string())?;
    let cost = resp.cost(profile);
    let v = extract_json(&resp.text).ok_or_else(|| "research answer was not valid JSON".to_string())?;
    let mut k = Knowledge {
        game: game.to_string(),
        created_at: chrono::Utc::now().timestamp(),
        summary: v["summary"].as_str().unwrap_or_default().to_string(),
        facts: strings(&v["facts"]),
        tips: strings(&v["tips"]),
        sources: Vec::new(),
    };
    if let Some((_, url)) = wiki
        && !url.is_empty() {
            k.sources.push(url);
        }
    if k.facts.is_empty() {
        return Err("research returned no facts".into());
    }
    let _ = storage::write_json_atomic(&cache_path(dir, game), &k);
    Ok((k, cost, resp.searches))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Network test: `cargo test wikipedia_lookup -- --ignored`
    #[tokio::test]
    #[ignore]
    async fn wikipedia_lookup() {
        let http = reqwest::Client::builder().user_agent("HighBazar-test").build().unwrap();
        let (extract, url) = wikipedia(&http, "Elden Ring").await.expect("wikipedia");
        assert!(extract.to_lowercase().contains("elden"));
        assert!(url.starts_with("https://"));
    }

    #[test]
    fn non_games() {
        assert!(is_non_game("Just Chatting"));
        assert!(!is_non_game("Elden Ring"));
    }

    // Live tests with a real AI – they use your subscription or key:
    //   cargo test live_ -- --ignored --nocapture --test-threads 1
    // HIGHBAZAR_LIVE_PROVIDER (default claudeCli), HIGHBAZAR_LIVE_MODEL (default
    // haiku, like the built-in Claude CLI profile), HIGHBAZAR_LIVE_GAME.

    fn live_profile() -> AiProfile {
        AiProfile {
            provider: std::env::var("HIGHBAZAR_LIVE_PROVIDER").unwrap_or_else(|_| "claudeCli".into()),
            model: std::env::var("HIGHBAZAR_LIVE_MODEL").unwrap_or_else(|_| "haiku".into()),
            ..Default::default()
        }
    }

    fn live_game() -> String {
        std::env::var("HIGHBAZAR_LIVE_GAME").unwrap_or_else(|_| "ELDEN RING NIGHTREIGN".into())
    }

    /// A chatter @mentions the bot with a question that needs a search.
    #[tokio::test]
    #[ignore]
    async fn live_chat_question() {
        use crate::bot::context::Line;
        use crate::bot::prompt::{self, ContextBlock, StreamFacts};
        use crate::config::{ChannelProfile, CompanionProfile, WebSearchSettings};
        let game = live_game();
        // HIGHBAZAR_LIVE_QUESTION / HIGHBAZAR_LIVE_LANG for other questions and stream languages.
        let text = std::env::var("HIGHBAZAR_LIVE_QUESTION").unwrap_or_else(|_| "what was in the latest patch?".into());
        let lang = std::env::var("HIGHBAZAR_LIVE_LANG").unwrap_or_else(|_| "en".into());
        let c = CompanionProfile::default();
        let line = Line { ts: chrono::Utc::now().timestamp_millis(), user_id: "1".into(), login: "viewer42".into(), name: "viewer42".into(), text: format!("@HighBazar {text}"), role: "" };
        let block = ContextBlock {
            stream: StreamFacts { channel_name: "ZcarecroW", live: true, uptime: Some("1h 5m".into()), game: &game, title: "chill runs", language: &lang, viewers: Some(12) },
            knowledge: Vec::new(),
            emotes: Vec::new(),
            bot_recent: Vec::new(),
            chat: vec![&line],
            language_rule: prompt::language_rule(&c, &lang),
        };
        let task = prompt::finish_task(&prompt::mention_task("viewer42", &text, false, true), true, c.max_length);
        let req = AiRequest {
            system: prompt::system_prompt(&ChannelProfile::default(), &c, "ZcarecroW", false),
            user: prompt::user_prompt(&block, &task),
            web_search: true,
            ..Default::default()
        };
        let cfg = WebSearchSettings::default();
        let own = ai::OwnSearch { cfg: &cfg, key: None, lang: "en".into() };
        let start = std::time::Instant::now();
        let r = ai::complete_searching(&reqwest::Client::new(), &live_profile(), None, &req, &own).await.expect("answer");
        println!("{:.1} s, {} search(es): {}", start.elapsed().as_secs_f32(), r.searches, r.text);
        assert!(r.searches > 0, "a patch question must be looked up");
    }

    /// The game research at stream start.
    #[tokio::test]
    #[ignore]
    async fn live_research() {
        let dir = std::env::temp_dir().join(format!("highbazar-research-{}", uuid::Uuid::new_v4().simple()));
        let cfg = crate::config::WebSearchSettings::default();
        let own = ai::OwnSearch { cfg: &cfg, key: None, lang: "en".into() };
        let http = reqwest::Client::builder().user_agent("HighBazar-test").build().unwrap();
        let start = std::time::Instant::now();
        let res = research(&http, &dir, &live_game(), &live_profile(), None, true, true, &own).await;
        let _ = std::fs::remove_dir_all(&dir);
        let (k, _, searches) = res.expect("research");
        println!("{:.1} s, {searches} search(es), {} facts, {} tips", start.elapsed().as_secs_f32(), k.facts.len(), k.tips.len());
        for f in k.facts.iter().filter(|f| f.chars().any(|c| c.is_ascii_digit())) {
            println!("  - {f}");
        }
        assert!(searches > 0 && k.facts.len() >= 10);
    }
}
