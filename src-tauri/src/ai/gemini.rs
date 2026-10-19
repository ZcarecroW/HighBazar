//! Google Gemini: Interactions API (recommended) with automatic fallback to
//! the older generateContent API.

use serde_json::{Value, json};

use super::{AiRequest, AiResponse, apply_headers, base_url, error_from, max_tokens, timeout};
use crate::config::AiProfile;
use crate::error::{AppError, AppResult};

pub async fn complete(http: &reqwest::Client, p: &AiProfile, key: &str, req: &AiRequest) -> AppResult<AiResponse> {
    if p.gemini_interactions {
        match interactions(http, p, key, req).await {
            Ok(r) if !r.text.trim().is_empty() => return Ok(r),
            Ok(_) => log::warn!("Gemini Interactions returned no text, retrying with generateContent"),
            Err(AppError::Ai { status, message, .. }) if status == 404 || status == 400 => {
                log::warn!("Gemini Interactions failed ({status}: {message}), retrying with generateContent");
            }
            Err(e) => return Err(e),
        }
    }
    generate_content(http, p, key, req).await
}

async fn interactions(http: &reqwest::Client, p: &AiProfile, key: &str, req: &AiRequest) -> AppResult<AiResponse> {
    let mut gen_cfg = json!({ "max_output_tokens": max_tokens(p, req) });
    if !p.reasoning.is_empty() {
        gen_cfg["thinking_level"] = json!(p.reasoning);
    }
    let mut b = json!({
        "model": p.model,
        "input": req.user,
        "system_instruction": req.system,
        "generation_config": gen_cfg,
        "store": false,
    });
    if req.web_search {
        b["tools"] = json!([{ "type": "google_search" }]);
    }
    let resp = apply_headers(
        http.post(format!("{}/interactions", base_url(p)))
            .timeout(timeout(p))
            .header("x-goog-api-key", key)
            .json(&b),
        p,
    )
    .send()
    .await?;
    if !resp.status().is_success() {
        return Err(error_from("gemini", resp).await);
    }
    let v: Value = resp.json().await?;
    Ok(parse_interaction(&v))
}

pub fn parse_interaction(v: &Value) -> AiResponse {
    let mut text = String::new();
    if let Some(steps) = v["steps"].as_array() {
        for step in steps.iter().filter(|s| s["type"] == "model_output") {
            if let Some(content) = step["content"].as_array() {
                for c in content.iter().filter(|c| c["type"] == "text") {
                    text.push_str(c["text"].as_str().unwrap_or_default());
                }
            }
        }
    }
    // Older response schema used "outputs".
    if text.is_empty()
        && let Some(outputs) = v["outputs"].as_array() {
            for o in outputs.iter().filter(|o| o["type"] == "text") {
                text.push_str(o["text"].as_str().unwrap_or_default());
            }
        }
    let u = &v["usage"];
    AiResponse {
        text,
        input_tokens: u["total_input_tokens"].as_u64().unwrap_or(0),
        output_tokens: u["total_output_tokens"].as_u64().unwrap_or(0) + u["total_thought_tokens"].as_u64().unwrap_or(0),
        reported_cost_usd: None,
        tool_cost_usd: 0.0,
        model: v["model"].as_str().unwrap_or_default().to_string(),
    }
}

async fn generate_content(http: &reqwest::Client, p: &AiProfile, key: &str, req: &AiRequest) -> AppResult<AiResponse> {
    let mut gen_cfg = json!({ "maxOutputTokens": max_tokens(p, req) });
    if !p.reasoning.is_empty() {
        gen_cfg["thinkingConfig"] = json!({ "thinkingLevel": p.reasoning });
    }
    let mut b = json!({
        "systemInstruction": { "parts": [{ "text": req.system }] },
        "contents": [{ "role": "user", "parts": [{ "text": req.user }] }],
        "generationConfig": gen_cfg,
    });
    if req.web_search {
        b["tools"] = json!([{ "google_search": {} }]);
    }
    let model = p.model.trim_start_matches("models/");
    let resp = apply_headers(
        http.post(format!("{}/models/{model}:generateContent", base_url(p)))
            .timeout(timeout(p))
            .header("x-goog-api-key", key)
            .json(&b),
        p,
    )
    .send()
    .await?;
    if !resp.status().is_success() {
        return Err(error_from("gemini", resp).await);
    }
    let v: Value = resp.json().await?;
    parse_generate(&v)
}

pub fn parse_generate(v: &Value) -> AppResult<AiResponse> {
    let cand = &v["candidates"][0];
    if cand.is_null() {
        let reason = v["promptFeedback"]["blockReason"].as_str().unwrap_or("no candidates");
        return Err(AppError::msg(format!("Gemini returned no answer ({reason}).")));
    }
    let text = cand["content"]["parts"]
        .as_array()
        .map(|parts| {
            parts
                .iter()
                .filter(|p| !p["thought"].as_bool().unwrap_or(false))
                .filter_map(|p| p["text"].as_str())
                .collect::<Vec<_>>()
                .join("")
        })
        .unwrap_or_default();
    let u = &v["usageMetadata"];
    Ok(AiResponse {
        text,
        input_tokens: u["promptTokenCount"].as_u64().unwrap_or(0),
        output_tokens: u["candidatesTokenCount"].as_u64().unwrap_or(0) + u["thoughtsTokenCount"].as_u64().unwrap_or(0),
        reported_cost_usd: None,
        tool_cost_usd: 0.0,
        model: v["modelVersion"].as_str().unwrap_or_default().to_string(),
    })
}

pub async fn list_models(http: &reqwest::Client, p: &AiProfile, key: &str) -> AppResult<Vec<String>> {
    let resp = apply_headers(
        http.get(format!("{}/models?pageSize=1000", base_url(p)))
            .header("x-goog-api-key", key),
        p,
    )
    .send()
    .await?;
    if !resp.status().is_success() {
        return Err(error_from("gemini", resp).await);
    }
    let v: Value = resp.json().await?;
    Ok(v["models"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter(|m| {
                    m["supportedGenerationMethods"]
                        .as_array()
                        .is_none_or(|ms| ms.iter().any(|x| x == "generateContent"))
                })
                .filter_map(|m| m["name"].as_str().map(|n| n.trim_start_matches("models/").to_string()))
                .collect()
        })
        .unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_interaction_steps() {
        let v = json!({
            "model": "gemini-3.8-flash",
            "status": "completed",
            "steps": [
                {"type": "thought", "summary": []},
                {"type": "model_output", "content": [{"type": "text", "text": "Hi!"}]}
            ],
            "usage": {"total_input_tokens": 12, "total_output_tokens": 3, "total_thought_tokens": 5}
        });
        let r = parse_interaction(&v);
        assert_eq!(r.text, "Hi!");
        assert_eq!(r.output_tokens, 8);
    }

    #[test]
    fn parses_generate_content_without_thoughts() {
        let v = json!({
            "candidates": [{"content": {"parts": [{"text": "plan", "thought": true}, {"text": "Answer"}]}}],
            "usageMetadata": {"promptTokenCount": 3, "candidatesTokenCount": 1}
        });
        assert_eq!(parse_generate(&v).unwrap().text, "Answer");
    }
}
