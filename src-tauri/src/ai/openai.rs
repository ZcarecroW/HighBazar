//! OpenAI Responses API.

use serde_json::{Value, json};

use super::{AiRequest, AiResponse, apply_headers, base_url, error_from, max_tokens, timeout};
use crate::config::AiProfile;
use crate::error::{AppError, AppResult};

pub async fn complete(http: &reqwest::Client, p: &AiProfile, key: &str, req: &AiRequest) -> AppResult<AiResponse> {
    let mut b = json!({
        "model": p.model,
        "instructions": req.system,
        "input": req.user,
        "max_output_tokens": max_tokens(p, req),
        "store": false,
    });
    if !p.reasoning.is_empty() {
        b["reasoning"] = json!({ "effort": p.reasoning });
    }
    if let Some(t) = p.temperature {
        b["temperature"] = json!(t);
    }
    if req.web_search {
        b["tools"] = json!([{ "type": "web_search" }]);
    }
    let resp = apply_headers(
        http.post(format!("{}/responses", base_url(p)))
            .timeout(timeout(p))
            .bearer_auth(key)
            .json(&b),
        p,
    )
    .send()
    .await?;
    if !resp.status().is_success() {
        return Err(error_from("openai", resp).await);
    }
    let v: Value = resp.json().await?;
    parse(&v)
}

pub fn parse(v: &Value) -> AppResult<AiResponse> {
    let mut text = String::new();
    let mut searches = 0u64;
    if let Some(items) = v["output"].as_array() {
        for item in items {
            match item["type"].as_str() {
                Some("message") => {
                    if let Some(parts) = item["content"].as_array() {
                        for part in parts {
                            match part["type"].as_str() {
                                Some("output_text") => text.push_str(part["text"].as_str().unwrap_or_default()),
                                Some("refusal") => {
                                    return Err(AppError::msg(format!(
                                        "The model refused: {}",
                                        part["refusal"].as_str().unwrap_or_default()
                                    )));
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Some("web_search_call") => searches += 1,
                _ => {}
            }
        }
    }
    let u = &v["usage"];
    Ok(AiResponse {
        text,
        input_tokens: u["input_tokens"].as_u64().unwrap_or(0),
        output_tokens: u["output_tokens"].as_u64().unwrap_or(0),
        reported_cost_usd: None,
        tool_cost_usd: searches as f64 * 0.01,
        model: v["model"].as_str().unwrap_or_default().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_output_text() {
        let v = json!({
            "model": "gpt-6-luna",
            "output": [
                {"type": "reasoning", "summary": []},
                {"type": "message", "role": "assistant", "content": [{"type": "output_text", "text": "GG chat"}]}
            ],
            "usage": {"input_tokens": 50, "output_tokens": 4}
        });
        let r = parse(&v).unwrap();
        assert_eq!(r.text, "GG chat");
        assert_eq!(r.output_tokens, 4);
    }
}
