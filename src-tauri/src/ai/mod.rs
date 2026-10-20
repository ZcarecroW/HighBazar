//! Provider-neutral AI completion layer.

mod anthropic;
mod claude_cli;
mod codex_cli;
mod compat;
mod gemini;
mod openai;

use std::time::Duration;

use serde::Serialize;
use serde_json::Value;

use crate::config::{AiProfile, WebSearchSettings};
use crate::error::{AppError, AppResult};

pub use claude_cli::detect as detect_claude_cli;
pub use codex_cli::{detect as detect_codex_cli, login_status as codex_login_status};

#[derive(Debug, Clone, Default)]
pub struct AiRequest {
    pub system: String,
    pub user: String,
    pub max_tokens: Option<u32>,
    /// Let the model look things up on the web (see `complete_searching`).
    pub web_search: bool,
    /// Also let it open web pages (only for requests not triggered by chat).
    pub web_fetch: bool,
    /// Known search query: HighBazar's own search runs it up front instead of
    /// letting the model ask for one.
    pub search_query: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiResponse {
    pub text: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    /// Exact cost if the provider reports it.
    pub reported_cost_usd: Option<f64>,
    /// Extra cost of server-side tools (e.g. web searches), in USD.
    pub tool_cost_usd: f64,
    pub model: String,
    /// Number of web searches made for this answer.
    pub searches: u32,
}

impl AiResponse {
    pub fn cost(&self, p: &AiProfile) -> f64 {
        if let Some(c) = self.reported_cost_usd {
            return c;
        }
        self.input_tokens as f64 * p.price_input_per_mtok / 1e6
            + self.output_tokens as f64 * p.price_output_per_mtok / 1e6
            + self.tool_cost_usd
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub base_url: &'static str,
    pub needs_key: bool,
    /// The provider searches the web itself (otherwise HighBazar's own search is used).
    pub web_search: bool,
    pub key_url: &'static str,
    pub local: bool,
    /// Hint for the "reasoning" field.
    pub reasoning_options: &'static [&'static str],
}

pub fn providers() -> Vec<ProviderInfo> {
    const EFFORT: &[&str] = &["", "low", "medium", "high"];
    vec![
        ProviderInfo { id: "anthropic", name: "Anthropic (Claude)", base_url: "https://api.anthropic.com", needs_key: true, web_search: true, key_url: "https://platform.claude.com/settings/keys", local: false, reasoning_options: &["", "low", "medium", "high", "xhigh", "max"] },
        ProviderInfo { id: "openai", name: "OpenAI", base_url: "https://api.openai.com/v1", needs_key: true, web_search: true, key_url: "https://platform.openai.com/api-keys", local: false, reasoning_options: &["", "minimal", "low", "medium", "high"] },
        ProviderInfo { id: "gemini", name: "Google Gemini", base_url: "https://generativelanguage.googleapis.com/v1beta", needs_key: true, web_search: true, key_url: "https://aistudio.google.com/apikey", local: false, reasoning_options: &["", "minimal", "low", "medium", "high"] },
        ProviderInfo { id: "openrouter", name: "OpenRouter", base_url: "https://openrouter.ai/api/v1", needs_key: true, web_search: true, key_url: "https://openrouter.ai/settings/keys", local: false, reasoning_options: EFFORT },
        ProviderInfo { id: "mistral", name: "Mistral AI", base_url: "https://api.mistral.ai/v1", needs_key: true, web_search: false, key_url: "https://console.mistral.ai/api-keys", local: false, reasoning_options: &[""] },
        ProviderInfo { id: "groq", name: "Groq", base_url: "https://api.groq.com/openai/v1", needs_key: true, web_search: false, key_url: "https://console.groq.com/keys", local: false, reasoning_options: EFFORT },
        ProviderInfo { id: "xai", name: "xAI (Grok)", base_url: "https://api.x.ai/v1", needs_key: true, web_search: true, key_url: "https://console.x.ai", local: false, reasoning_options: EFFORT },
        ProviderInfo { id: "deepseek", name: "DeepSeek", base_url: "https://api.deepseek.com/v1", needs_key: true, web_search: false, key_url: "https://platform.deepseek.com/api_keys", local: false, reasoning_options: &[""] },
        ProviderInfo { id: "perplexity", name: "Perplexity", base_url: "https://api.perplexity.ai", needs_key: true, web_search: true, key_url: "https://www.perplexity.ai/account/api/keys", local: false, reasoning_options: &[""] },
        ProviderInfo { id: "together", name: "Together AI", base_url: "https://api.together.xyz/v1", needs_key: true, web_search: false, key_url: "https://api.together.ai/settings/api-keys", local: false, reasoning_options: &[""] },
        ProviderInfo { id: "ollama", name: "Ollama (local)", base_url: "http://localhost:11434/v1", needs_key: false, web_search: false, key_url: "https://ollama.com/download", local: true, reasoning_options: &[""] },
        ProviderInfo { id: "lmstudio", name: "LM Studio (local)", base_url: "http://localhost:1234/v1", needs_key: false, web_search: false, key_url: "https://lmstudio.ai", local: true, reasoning_options: &[""] },
        ProviderInfo { id: "openaiCompatible", name: "Custom (OpenAI-compatible)", base_url: "", needs_key: false, web_search: false, key_url: "", local: false, reasoning_options: EFFORT },
        ProviderInfo { id: "claudeCli", name: "Claude Code CLI (subscription)", base_url: "", needs_key: false, web_search: true, key_url: "https://code.claude.com/docs/en/setup", local: true, reasoning_options: &["", "low", "medium", "high", "xhigh", "max"] },
        ProviderInfo { id: "codexCli", name: "Codex CLI (ChatGPT subscription)", base_url: "", needs_key: false, web_search: true, key_url: "https://learn.chatgpt.com/docs/codex-cli", local: true, reasoning_options: &["", "low", "medium", "high", "xhigh", "max", "ultra"] },
    ]
}

pub fn provider_info(id: &str) -> Option<ProviderInfo> {
    providers().into_iter().find(|p| p.id == id)
}

pub fn base_url(p: &AiProfile) -> String {
    let custom = p.base_url.trim().trim_end_matches('/');
    if !custom.is_empty() {
        return custom.to_string();
    }
    provider_info(&p.provider).map(|i| i.base_url.to_string()).unwrap_or_default()
}

/// The key reference a profile uses (`provider` unless overridden).
pub fn key_ref(p: &AiProfile) -> String {
    if p.key_ref.trim().is_empty() { p.provider.clone() } else { p.key_ref.trim().to_string() }
}

/// Whether this profile's provider searches the web itself.
pub fn native_search(p: &AiProfile) -> bool {
    match p.provider.as_str() {
        // Only the Compound systems have Groq's built-in search.
        "groq" => p.model.trim().starts_with("groq/compound"),
        id => provider_info(id).is_some_and(|i| i.web_search),
    }
}

/// HighBazar's own web search, for providers without one.
pub struct OwnSearch<'a> {
    pub cfg: &'a WebSearchSettings,
    pub key: Option<String>,
    /// Preferred result language (e.g. the stream language).
    pub lang: String,
}

const ASK_FOR_SEARCH: &str = "\n\nWeb search: if a correct answer needs current or specific facts you do not reliably know (patches, release dates, news, events, results, prices, records), reply with only one line \"SEARCH: <short web search query>\" and nothing else. Otherwise answer right away.";

/// Like `complete`, but with web access when `req.web_search` is set: the
/// provider's own search where it has one, otherwise HighBazar searches with
/// its configured source and passes the results to the model.
pub async fn complete_searching(
    http: &reqwest::Client,
    p: &AiProfile,
    key: Option<&str>,
    req: &AiRequest,
    own: &OwnSearch<'_>,
) -> AppResult<AiResponse> {
    if !req.web_search || native_search(p) {
        return complete(http, p, key, req).await;
    }
    let plain = AiRequest { web_search: false, web_fetch: false, search_query: None, ..req.clone() };
    let run_search = |q: String| async move {
        let hits = match crate::search::search(http, own.cfg, own.key.as_deref(), &q, &own.lang).await {
            Ok(h) => h,
            Err(e) => {
                log::warn!("web search for {q:?} failed: {e}");
                Vec::new()
            }
        };
        crate::search::prompt_block(&q, &hits)
    };
    if let Some(q) = req.search_query.clone().filter(|q| !q.trim().is_empty()) {
        let block = run_search(q).await;
        let mut r = complete(http, p, key, &AiRequest { user: format!("{}\n\n{block}", req.user), ..plain }).await?;
        r.searches = 1;
        return Ok(r);
    }
    // Let the model decide: it either answers or asks for one search.
    let first = complete(http, p, key, &AiRequest { user: format!("{}{ASK_FOR_SEARCH}", req.user), ..plain.clone() }).await?;
    let Some(q) = search_request(&first.text) else {
        return Ok(first);
    };
    let block = run_search(q).await;
    let user = format!("{}\n\n{block}\n\nNow write your answer using these results where they help. Do not ask for another search.", req.user);
    let second = complete(http, p, key, &AiRequest { user, ..plain }).await?;
    Ok(AiResponse {
        input_tokens: first.input_tokens + second.input_tokens,
        output_tokens: first.output_tokens + second.output_tokens,
        reported_cost_usd: match (first.reported_cost_usd, second.reported_cost_usd) {
            (Some(a), Some(b)) => Some(a + b),
            _ => None,
        },
        tool_cost_usd: first.tool_cost_usd + second.tool_cost_usd,
        searches: 1,
        ..second
    })
}

/// "SEARCH: query" as the model's whole answer.
fn search_request(text: &str) -> Option<String> {
    let t = text.trim().trim_matches(|c| c == '`' || c == '"').trim();
    let first = t.lines().next()?.trim();
    let (head, rest) = first.split_at_checked(7)?;
    if !head.eq_ignore_ascii_case("search:") {
        return None;
    }
    let q: String = rest.trim().trim_matches(|c| c == '"' || c == '<' || c == '>' || c == '`').trim().chars().take(200).collect();
    (!q.is_empty()).then_some(q)
}

pub async fn complete(http: &reqwest::Client, p: &AiProfile, key: Option<&str>, req: &AiRequest) -> AppResult<AiResponse> {
    let needs_key = provider_info(&p.provider).map(|i| i.needs_key).unwrap_or(false);
    if needs_key && key.is_none_or(|k| k.trim().is_empty()) {
        return Err(AppError::msg(format!("No API key stored for “{}”. Add it under AI → API keys.", p.provider)));
    }
    let key = key.unwrap_or_default();
    let mut attempt = 0;
    loop {
        let result = match p.provider.as_str() {
            "anthropic" => anthropic::complete(http, p, key, req).await,
            "openai" if p.openai_responses || req.web_search => openai::complete(http, p, key, req).await,
            // xAI's web search only exists in its Responses API.
            "xai" if req.web_search => openai::complete(http, p, key, req).await,
            "gemini" => gemini::complete(http, p, key, req).await,
            "claudeCli" => claude_cli::complete(p, req).await,
            "codexCli" => codex_cli::complete(p, req).await,
            _ => compat::complete(http, p, key, req).await,
        };
        match result {
            Err(AppError::Ai { status, .. }) if attempt < 2 && (status == 429 || status == 529 || status >= 500) => {
                attempt += 1;
                tokio::time::sleep(Duration::from_secs(2u64.pow(attempt) + 1)).await;
            }
            Err(AppError::Http(e)) if attempt < 2 && (e.is_timeout() || e.is_connect()) => {
                attempt += 1;
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
            other => return other,
        }
    }
}

/// A model offered in the model picker.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub id: String,
    /// Display name from the provider (empty = use the id).
    pub name: String,
    /// USD per million tokens, when the provider publishes prices (OpenRouter).
    pub price_in: Option<f64>,
    pub price_out: Option<f64>,
}

impl ModelInfo {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self { id: id.into(), name: name.into(), price_in: None, price_out: None }
    }
}

pub async fn list_models(http: &reqwest::Client, p: &AiProfile, key: Option<&str>) -> AppResult<Vec<ModelInfo>> {
    let key = key.unwrap_or_default();
    let mut models = match p.provider.as_str() {
        // Anthropic lists the newest models first; keep that order.
        "anthropic" => return Ok(dedup(anthropic::list_models(http, p, key).await?)),
        "gemini" => gemini::list_models(http, p, key).await?,
        // Aliases of the Claude Code CLI always point to the latest model of each family.
        "claudeCli" => return Ok(["haiku", "sonnet", "opus", "fable"].iter().map(|a| ModelInfo::new(*a, "")).collect()),
        // The CLI's own catalog, in its order (newest first).
        "codexCli" => return Ok(dedup(codex_cli::list_models(p).await?)),
        _ => compat::list_models(http, p, key).await?,
    };
    models.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(dedup(models))
}

fn dedup(models: Vec<ModelInfo>) -> Vec<ModelInfo> {
    let mut seen = std::collections::HashSet::new();
    models.into_iter().filter(|m| !m.id.is_empty() && seen.insert(m.id.clone())).collect()
}

pub(crate) fn timeout(p: &AiProfile) -> Duration {
    Duration::from_secs(p.timeout_secs.clamp(10, 600) as u64)
}

pub(crate) fn max_tokens(p: &AiProfile, req: &AiRequest) -> u32 {
    req.max_tokens.unwrap_or(0).max(p.max_output_tokens).clamp(64, 64_000)
}

/// Turns a non-2xx response into a readable error.
pub(crate) async fn error_from(provider: &str, resp: reqwest::Response) -> AppError {
    let status = resp.status().as_u16();
    let text = resp.text().await.unwrap_or_default();
    let message = serde_json::from_str::<Value>(&text)
        .ok()
        .and_then(|v| {
            v.pointer("/error/message")
                .or_else(|| v.get("message"))
                .or_else(|| v.get("error"))
                .and_then(|m| m.as_str().map(str::to_string))
        })
        .unwrap_or_else(|| text.chars().take(400).collect());
    AppError::Ai { provider: provider.to_string(), status, message }
}

pub(crate) fn apply_headers(mut rb: reqwest::RequestBuilder, p: &AiProfile) -> reqwest::RequestBuilder {
    for h in &p.extra_headers {
        if !h.name.trim().is_empty() {
            rb = rb.header(h.name.trim(), h.value.trim());
        }
    }
    rb
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_search_requests() {
        assert_eq!(search_request("SEARCH: elden ring nightreign patch"), Some("elden ring nightreign patch".into()));
        assert_eq!(search_request("  `search: \"dlc release date\"`"), Some("dlc release date".into()));
        assert_eq!(search_request("@max The patch came out yesterday!"), None);
        assert_eq!(search_request("SEARCH:"), None);
        assert_eq!(search_request("Süß"), None);
    }

    #[test]
    fn groq_searches_only_with_compound() {
        let mut p = AiProfile { provider: "groq".into(), model: "llama-3.3-70b-versatile".into(), ..Default::default() };
        assert!(!native_search(&p));
        p.model = "groq/compound-mini".into();
        assert!(native_search(&p));
        p.provider = "ollama".into();
        assert!(!native_search(&p));
        p.provider = "codexCli".into();
        assert!(native_search(&p));
    }
}
