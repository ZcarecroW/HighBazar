//! OpenAI Responses API.

use serde_json::{Value, json};

use super::{AiRequest, AiResponse, apply_headers, base_url, error_from, max_tokens, openai_reasoning_model, timeout};
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
        b["reasoning"] = json!({ "effort": effort(&p.model, &p.reasoning) });
    }
    if let Some(t) = p.temperature.filter(|_| !openai_reasoning_model(&p.model)) {
        b["temperature"] = json!(t);
    }
    if req.web_search {
        b["tools"] = json!([{ "type": "web_search" }]);
    }
    let resp = apply_headers(
        http.post(format!("{}/responses", base_url(p)))
            .timeout(timeout(p, req))
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

/// GPT-6 models know "none" … "max"; "minimal" (GPT-5 era, saved in older
/// profiles) would be rejected there.
fn effort<'a>(model: &str, effort: &'a str) -> &'a str {
    if effort == "minimal" && model.starts_with("gpt-6") { "none" } else { effort }
}

pub fn parse(v: &Value) -> AppResult<AiResponse> {
    let mut text = String::new();
    let mut searches = 0u64;
    if let Some(items) = v["output"].as_array() {
        for item in items {
            match item["type"].as_str() {
                Some("message") => {
                    // Only the last message is the answer; earlier ones are
                    // commentary around tool calls ("Let me look that up").
                    text.clear();
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
    if text.trim().is_empty() && v["incomplete_details"]["reason"].as_str() == Some("max_output_tokens") {
        return Err(AppError::msg("The model used up “Max. output tokens” before writing an answer. Raise it in the AI profile or choose a lower effort."));
    }
    let u = &v["usage"];
    Ok(AiResponse {
        text,
        input_tokens: u["input_tokens"].as_u64().unwrap_or(0),
        cache_read_tokens: u["input_tokens_details"]["cached_tokens"].as_u64().unwrap_or(0),
        cache_write_tokens: 0,
        output_tokens: u["output_tokens"].as_u64().unwrap_or(0),
        reported_cost_usd: None,
        tool_cost_usd: searches as f64 * 0.01,
        searches: searches as u32,
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
                {"type": "message", "role": "assistant", "content": [{"type": "output_text", "text": "Let me check. "}]},
                {"type": "web_search_call", "status": "completed"},
                {"type": "message", "role": "assistant", "content": [{"type": "output_text", "text": "GG chat"}]}
            ],
            "usage": {"input_tokens": 50, "output_tokens": 4}
        });
        let r = parse(&v).unwrap();
        assert_eq!(r.text, "GG chat");
        assert_eq!(r.output_tokens, 4);
    }

    #[test]
    fn cached_input_and_truncated_answers() {
        let v = json!({
            "output": [{"type": "message", "content": [{"type": "output_text", "text": "hi"}]}],
            "usage": {"input_tokens": 3000, "input_tokens_details": {"cached_tokens": 2048}, "output_tokens": 2}
        });
        assert_eq!(parse(&v).unwrap().cache_read_tokens, 2048);
        let cut = json!({"status": "incomplete", "incomplete_details": {"reason": "max_output_tokens"}, "output": [{"type": "reasoning"}], "usage": {}});
        assert!(parse(&cut).is_err());
    }

    #[test]
    fn maps_old_effort_names() {
        assert_eq!(effort("gpt-6-luna", "minimal"), "none");
        assert_eq!(effort("gpt-6.1-terra", "low"), "low");
        assert_eq!(effort("gpt-5-mini", "minimal"), "minimal");
    }
}
