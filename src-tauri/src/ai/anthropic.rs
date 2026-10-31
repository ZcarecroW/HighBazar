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

/// The web fetch tool version a model takes (Fable has its own dynamic one).
fn web_fetch_tool(model: &str) -> &'static str {
    if model.starts_with("claude-fable") || model.starts_with("claude-mythos") {
        "web_fetch_20260318"
    } else if supports_dynamic_search(model) {
        "web_fetch_20260209"
    } else {
        "web_fetch_20250910"
    }
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

fn body(p: &AiProfile, req: &AiRequest, with_fallbacks: bool, with_fetch: bool) -> Value {
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
        let mut tools = vec![json!({ "type": kind, "name": "web_search", "max_uses": 3 })];
        // Reading a found page (patch notes, wikis) – only for requests not
        // triggered by chat, and capped so one long page can't get expensive.
        if with_fetch {
            tools.push(json!({ "type": web_fetch_tool(&p.model), "name": "web_fetch", "max_uses": 3, "max_content_tokens": 8000 }));
        }
        b["tools"] = json!(tools);
    }
    if with_fallbacks {
        b["fallbacks"] = json!("default");
    }
    b
}

pub async fn complete(http: &reqwest::Client, p: &AiProfile, key: &str, req: &AiRequest) -> AppResult<AiResponse> {
    let url = format!("{}/v1/messages", base_url(p));
    let mut use_fallbacks = supports_fallbacks(&p.model) && p.base_url.trim().is_empty();
    let mut use_fetch = req.web_search && req.web_fetch;
    loop {
        let mut rb = http
            .post(&url)
            .timeout(timeout(p))
            .header("x-api-key", key)
            .header("anthropic-version", VERSION)
            .json(&body(p, req, use_fallbacks, use_fetch));
        if use_fallbacks {
            rb = rb.header("anthropic-beta", FALLBACK_BETA);
        }
        let resp = apply_headers(rb, p).send().await?;
        if !resp.status().is_success() {
            let err = error_from("anthropic", resp).await;
            if let AppError::Ai { status: 400, message, .. } = &err {
                let m = message.to_lowercase();
                // Older deployments / proxies may not know the fallback beta …
                if use_fallbacks && (m.contains("fallback") || m.contains("beta")) {
                    use_fallbacks = false;
                    continue;
                }
                // … or a model the web fetch tool (search still works without it).
                if use_fetch && m.contains("web_fetch") {
                    use_fetch = false;
                    continue;
                }
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
    // Thinking counts toward max_tokens on the 5.x models: it can use up the
    // whole budget before any text is written.
    if text.trim().is_empty() && v["stop_reason"].as_str() == Some("max_tokens") {
        return Err(AppError::msg("The model used up “Max. output tokens” before writing an answer. Raise it in the AI profile or choose a lower effort."));
    }
    let u = &v["usage"];
    let (write, read) = (u["cache_creation_input_tokens"].as_u64().unwrap_or(0), u["cache_read_input_tokens"].as_u64().unwrap_or(0));
    let input = u["input_tokens"].as_u64().unwrap_or(0) + write + read;
    let searches = u["server_tool_use"]["web_search_requests"].as_u64().unwrap_or(0);
    Ok(AiResponse {
        text,
        input_tokens: input,
        cache_read_tokens: read,
        cache_write_tokens: write,
        output_tokens: u["output_tokens"].as_u64().unwrap_or(0),
        reported_cost_usd: None,
        tool_cost_usd: searches as f64 * 0.01,
        searches: searches as u32,
        model: v["model"].as_str().unwrap_or_default().to_string(),
    })
}

pub async fn list_models(http: &reqwest::Client, p: &AiProfile, key: &str) -> AppResult<Vec<super::ModelInfo>> {
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
        .map(|a| {
            a.iter()
                .filter_map(|m| m["id"].as_str().map(|id| super::ModelInfo::new(id, m["display_name"].as_str().unwrap_or_default())))
                .collect()
        })
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
        assert_eq!(web_fetch_tool("claude-fable-5-1"), "web_fetch_20260318");
        assert_eq!(web_fetch_tool("claude-sonnet-5-5"), "web_fetch_20260209");
        assert_eq!(web_fetch_tool("claude-haiku-5-5"), "web_fetch_20250910");
    }

    #[test]
    fn fetch_only_with_search_and_capped() {
        let p = AiProfile { model: "claude-opus-5-5".into(), ..Default::default() };
        let req = AiRequest { web_search: true, web_fetch: true, ..Default::default() };
        let b = body(&p, &req, false, true);
        let tools = b["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[1]["type"], "web_fetch_20260209");
        assert_eq!(tools[1]["max_content_tokens"], 8000);
        assert_eq!(body(&p, &req, false, false)["tools"].as_array().unwrap().len(), 1);
        // No sampling parameters for current models, effort only when set.
        assert!(b.get("temperature").is_none() && b.get("output_config").is_none());
    }

    #[test]
    fn counts_cache_tokens_and_explains_empty_answers() {
        let v = json!({
            "stop_reason": "end_turn",
            "content": [{"type": "text", "text": "hi"}],
            "usage": {"input_tokens": 100, "cache_creation_input_tokens": 50, "cache_read_input_tokens": 2000, "output_tokens": 3}
        });
        let r = parse(&v).unwrap();
        assert_eq!((r.input_tokens, r.cache_read_tokens, r.cache_write_tokens), (2150, 2000, 50));
        let cut = json!({"stop_reason": "max_tokens", "content": [{"type": "thinking", "thinking": ""}], "usage": {}});
        assert!(parse(&cut).unwrap_err().to_string().contains("Max. output tokens"));
    }
}
