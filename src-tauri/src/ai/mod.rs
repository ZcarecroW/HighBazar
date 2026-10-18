//! Provider-neutral AI completion layer.

mod anthropic;
mod claude_cli;
mod compat;
mod gemini;
mod openai;

use std::time::Duration;

use serde::Serialize;
use serde_json::Value;

use crate::config::AiProfile;
use crate::error::{AppError, AppResult};

pub use claude_cli::detect as detect_claude_cli;

#[derive(Debug, Clone)]
pub struct AiRequest {
    pub system: String,
    pub user: String,
    pub max_tokens: Option<u32>,
    pub web_search: bool,
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
        ProviderInfo { id: "xai", name: "xAI (Grok)", base_url: "https://api.x.ai/v1", needs_key: true, web_search: false, key_url: "https://console.x.ai", local: false, reasoning_options: EFFORT },
        ProviderInfo { id: "deepseek", name: "DeepSeek", base_url: "https://api.deepseek.com/v1", needs_key: true, web_search: false, key_url: "https://platform.deepseek.com/api_keys", local: false, reasoning_options: &[""] },
        ProviderInfo { id: "perplexity", name: "Perplexity", base_url: "https://api.perplexity.ai", needs_key: true, web_search: true, key_url: "https://www.perplexity.ai/account/api/keys", local: false, reasoning_options: &[""] },
        ProviderInfo { id: "together", name: "Together AI", base_url: "https://api.together.xyz/v1", needs_key: true, web_search: false, key_url: "https://api.together.ai/settings/api-keys", local: false, reasoning_options: &[""] },
        ProviderInfo { id: "ollama", name: "Ollama (local)", base_url: "http://localhost:11434/v1", needs_key: false, web_search: false, key_url: "https://ollama.com/download", local: true, reasoning_options: &[""] },
        ProviderInfo { id: "lmstudio", name: "LM Studio (local)", base_url: "http://localhost:1234/v1", needs_key: false, web_search: false, key_url: "https://lmstudio.ai", local: true, reasoning_options: &[""] },
        ProviderInfo { id: "openaiCompatible", name: "Custom (OpenAI-compatible)", base_url: "", needs_key: false, web_search: false, key_url: "", local: false, reasoning_options: EFFORT },
        ProviderInfo { id: "claudeCli", name: "Claude Code CLI (subscription)", base_url: "", needs_key: false, web_search: false, key_url: "https://code.claude.com/docs/en/setup", local: true, reasoning_options: &["", "low", "medium", "high"] },
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
            "openai" if p.openai_responses => openai::complete(http, p, key, req).await,
            "gemini" => gemini::complete(http, p, key, req).await,
            "claudeCli" => claude_cli::complete(p, req).await,
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

pub async fn list_models(http: &reqwest::Client, p: &AiProfile, key: Option<&str>) -> AppResult<Vec<String>> {
    let key = key.unwrap_or_default();
    let mut models = match p.provider.as_str() {
        "anthropic" => anthropic::list_models(http, p, key).await?,
        "gemini" => gemini::list_models(http, p, key).await?,
        "claudeCli" => vec!["haiku".into(), "sonnet".into(), "opus".into(), "fable".into()],
        _ => compat::list_models(http, p, key).await?,
    };
    models.sort();
    models.dedup();
    Ok(models)
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
