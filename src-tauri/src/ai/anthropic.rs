//! Anthropic Messages API (raw HTTP).

use serde_json::{Value, json};

use super::{AiRequest, AiResponse, apply_headers, base_url, error_from, max_tokens, timeout};
use crate::config::AiProfile;
use crate::error::{AppError, AppResult};

const VERSION: &str = "2023-06-01";
const FALLBACK_BETA: &str = "server-side-fallback-2026-07-01";

/// Models that accept the dynamic-filtering web search tool.
fn supports_dynamic_search(model: &str) -> bool {
    ["claude-opus-5", "claude-opus-4-8", "claude-opus-4-7", "claude-opus-4-6", "claude-sonnet-5", "claude-sonnet-4-6", "claude-fable"]
        .iter()
        .any(|m| model.starts_with(m))
}

/// Server-side refusal fallbacks are offered on these models.
fn supports_fallbacks(model: &str) -> bool {
    ["claude-opus-5-5", "claude-opus-5", "claude-sonnet-5-5", "claude-fable-5-1"].contains(&model)
}

/// Current models reject sampling parameters; only send them to old ones.
fn supports_temperature(model: &str) -> bool {
    ["claude-3", "claude-haiku-4-5", "claude-sonnet-4-5", "claude-opus-4-5", "claude-opus-4-6", "claude-sonnet-4-6", "claude-opus-4-1", "claude-sonnet-4-0", "claude-opus-4-0"]
        .iter()
        .any(|m| model.starts_with(m))
}

fn body(p: &AiProfile, req: &AiRequest, with_fallbacks: bool) -> Value {
    let mut b = json!({
        "model": p.model,
        "max_tokens": max_tokens(p, req),
        "system": [{ "type": "text", "text": req.system, "cache_control": { "type": "ephemeral" } }],
        "messages": [{ "role": "user", "content": req.user }],
    });
    if !p.reasoning.is_empty() {
        b["output_config"] = json!({ "effort": p.reasoning });
    }
    if let Some(t) = p.temperature
        && supports_temperature(&p.model) {
            b["temperature"] = json!(t);
        }
    if req.web_search {
        let kind = if supports_dynamic_search(&p.model) { "web_search_20260209" } else { "web_search_20250305" };
        b["tools"] = json!([{ "type": kind, "name": "web_search", "max_uses": 3 }]);
    }
    if with_fallbacks {
        b["fallbacks"] = json!("default");
    }
    b
}

pub async fn complete(http: &reqwest::Client, p: &AiProfile, key: &str, req: &AiRequest) -> AppResult<AiResponse> {
    let url = format!("{}/v1/messages", base_url(p));
    let mut use_fallbacks = supports_fallbacks(&p.model) && p.base_url.trim().is_empty();
    loop {
        let mut rb = http
            .post(&url)
            .timeout(timeout(p))
            .header("x-api-key", key)
            .header("anthropic-version", VERSION)
            .json(&body(p, req, use_fallbacks));
        if use_fallbacks {
            rb = rb.header("anthropic-beta", FALLBACK_BETA);
        }
        let resp = apply_headers(rb, p).send().await?;
        if !resp.status().is_success() {
            let err = error_from("anthropic", resp).await;
            // Older deployments / proxies may not know the fallback beta.
            if use_fallbacks
                && let AppError::Ai { status: 400, message, .. } = &err
                    && (message.to_lowercase().contains("fallback") || message.to_lowercase().contains("beta")) {
                        use_fallbacks = false;
                        continue;
                    }
            return Err(err);
        }
        let v: Value = resp.json().await?;
        return parse(&v);
    }
}

pub fn parse(v: &Value) -> AppResult<AiResponse> {
    if v["stop_reason"].as_str() == Some("refusal") {
        return Err(AppError::msg("The model declined this request (refusal)."));
    }
    let text = v["content"]
        .as_array()
        .map(|blocks| {
            blocks
                .iter()
                .filter(|b| b["type"] == "text")
                .filter_map(|b| b["text"].as_str())
                .collect::<Vec<_>>()
                .join("")
        })
        .unwrap_or_default();
    let u = &v["usage"];
    let input = u["input_tokens"].as_u64().unwrap_or(0)
        + u["cache_creation_input_tokens"].as_u64().unwrap_or(0)
        + u["cache_read_input_tokens"].as_u64().unwrap_or(0);
    let searches = u["server_tool_use"]["web_search_requests"].as_u64().unwrap_or(0);
    Ok(AiResponse {
        text,
        input_tokens: input,
        output_tokens: u["output_tokens"].as_u64().unwrap_or(0),
        reported_cost_usd: None,
        tool_cost_usd: searches as f64 * 0.01,
        model: v["model"].as_str().unwrap_or_default().to_string(),
    })
}

pub async fn list_models(http: &reqwest::Client, p: &AiProfile, key: &str) -> AppResult<Vec<String>> {
    let resp = apply_headers(
        http.get(format!("{}/v1/models?limit=1000", base_url(p)))
            .header("x-api-key", key)
            .header("anthropic-version", VERSION),
        p,
    )
    .send()
    .await?;
    if !resp.status().is_success() {
        return Err(error_from("anthropic", resp).await);
    }
    let v: Value = resp.json().await?;
    Ok(v["data"]
        .as_array()
        .map(|a| a.iter().filter_map(|m| m["id"].as_str().map(str::to_string)).collect())
        .unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_text_and_usage() {
        let v = json!({
            "model": "claude-haiku-5-5",
            "stop_reason": "end_turn",
            "content": [{"type": "thinking", "thinking": ""}, {"type": "text", "text": "Hello "}, {"type": "text", "text": "chat"}],
            "usage": {"input_tokens": 10, "cache_read_input_tokens": 5, "output_tokens": 3, "server_tool_use": {"web_search_requests": 2}}
        });
        let r = parse(&v).unwrap();
        assert_eq!(r.text, "Hello chat");
        assert_eq!(r.input_tokens, 15);
        assert!((r.tool_cost_usd - 0.02).abs() < 1e-9);
    }

    #[test]
    fn web_search_variant() {
        assert!(supports_dynamic_search("claude-opus-5-5"));
        assert!(!supports_dynamic_search("claude-haiku-5-5"));
    }
}
