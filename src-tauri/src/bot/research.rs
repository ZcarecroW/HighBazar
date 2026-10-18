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

/// Researches a game. Returns the knowledge and the AI cost in USD.
pub async fn research(
    http: &reqwest::Client,
    dir: &Path,
    game: &str,
    profile: &AiProfile,
    key: Option<&str>,
    use_web: bool,
    use_wiki: bool,
) -> Result<(Knowledge, f64), String> {
    let wiki = if use_wiki { wikipedia(http, game).await } else { None };
    let (system, user) = super::prompt::research_prompt(game, wiki.as_ref().map(|w| w.0.as_str()).unwrap_or(""));
    let supports_web = ai::provider_info(&profile.provider).is_some_and(|i| i.web_search) && profile.web_search;
    let req = AiRequest { system, user, max_tokens: Some(4000), web_search: use_web && supports_web };
    let resp = ai::complete(http, profile, key, &req).await.map_err(|e| e.to_string())?;
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
    if let Some((_, url)) = wiki {
        if !url.is_empty() {
            k.sources.push(url);
        }
    }
    if k.facts.is_empty() {
        return Err("research returned no facts".into());
    }
    let _ = storage::write_json_atomic(&cache_path(dir, game), &k);
    Ok((k, cost))
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
}
