//! OpenAI-compatible Chat Completions (OpenRouter, Mistral, Groq, xAI,
//! DeepSeek, Perplexity, Together, Ollama, LM Studio, custom endpoints and
//! OpenAI itself when the Responses API is turned off).

use serde_json::{Value, json};

use super::{AiRequest, AiResponse, apply_headers, base_url, error_from, max_tokens, timeout};
use crate::config::AiProfile;
use crate::error::{AppError, AppResult};

pub async fn complete(http: &reqwest::Client, p: &AiProfile, key: &str, req: &AiRequest) -> AppResult<AiResponse> {
    let base = base_url(p);
    if base.is_empty() {
        return Err(AppError::msg("This AI profile needs a base URL."));
    }
    let mut b = json!({
        "model": p.model,
        "messages": [
            { "role": "system", "content": req.system },
            { "role": "user", "content": req.user },
        ],
    });
    let tokens = max_tokens(p, req);
    if p.provider == "openai" {
        b["max_completion_tokens"] = json!(tokens);
    } else {
        b["max_tokens"] = json!(tokens);
    }
    if let Some(t) = p.temperature {
        b["temperature"] = json!(t);
    }
    if !p.reasoning.is_empty() {
        if p.provider == "openrouter" {
            b["reasoning"] = json!({ "effort": p.reasoning });
        } else {
            b["reasoning_effort"] = json!(p.reasoning);
        }
    }
    if p.provider == "openrouter" {
        b["usage"] = json!({ "include": true });
        if req.web_search {
            b["plugins"] = json!([{ "id": "web", "max_results": 5 }]);
        }
    }
    let mut rb = http
        .post(format!("{base}/chat/completions"))
        .timeout(timeout(p))
        .json(&b);
    if !key.is_empty() {
        rb = rb.bearer_auth(key);
    }
    if p.provider == "openrouter" {
        rb = rb
            .header("HTTP-Referer", "https://github.com/ZcarecroW/HighBazar")
            .header("X-Title", "HighBazar");
    }
    let resp = apply_headers(rb, p).send().await.map_err(|e| {
        if e.is_connect() && (p.provider == "ollama" || p.provider == "lmstudio") {
            AppError::msg(format!("Cannot reach {base}. Is the local model server running?"))
        } else {
            AppError::Http(e)
        }
    })?;
    if !resp.status().is_success() {
        return Err(error_from(&p.provider, resp).await);
    }
    let v: Value = resp.json().await?;
    parse(&v)
}

pub fn parse(v: &Value) -> AppResult<AiResponse> {
    let msg = &v["choices"][0]["message"];
    let text = match &msg["content"] {
        Value::String(s) => s.clone(),
        Value::Array(parts) => parts
            .iter()
            .filter_map(|p| p["text"].as_str())
            .collect::<Vec<_>>()
            .join(""),
        _ => String::new(),
    };
    if text.is_empty()
        && let Some(r) = msg["refusal"].as_str() {
            return Err(AppError::msg(format!("The model refused: {r}")));
        }
    // Some reasoning models wrap their thoughts in <think> tags.
    let text = strip_think(&text);
    let u = &v["usage"];
    Ok(AiResponse {
        text,
        input_tokens: u["prompt_tokens"].as_u64().unwrap_or(0),
        output_tokens: u["completion_tokens"].as_u64().unwrap_or(0),
        reported_cost_usd: u["cost"].as_f64(),
        tool_cost_usd: 0.0,
        model: v["model"].as_str().unwrap_or_default().to_string(),
    })
}

fn strip_think(s: &str) -> String {
    let mut out = s.to_string();
    while let (Some(a), Some(b)) = (out.find("<think>"), out.find("</think>")) {
        if b < a {
            break;
        }
        out.replace_range(a..b + "</think>".len(), "");
    }
    out.trim().to_string()
}

pub async fn list_models(http: &reqwest::Client, p: &AiProfile, key: &str) -> AppResult<Vec<String>> {
    let base = base_url(p);
    if base.is_empty() {
        return Err(AppError::msg("This AI profile needs a base URL."));
    }
    let mut rb = http.get(format!("{base}/models"));
    if !key.is_empty() {
        rb = rb.bearer_auth(key);
    }
    let resp = apply_headers(rb, p).send().await?;
    if !resp.status().is_success() {
        return Err(error_from(&p.provider, resp).await);
    }
    let v: Value = resp.json().await?;
    let list = v["data"].as_array().or_else(|| v.as_array());
    Ok(list
        .map(|a| a.iter().filter_map(|m| m["id"].as_str().map(str::to_string)).collect())
        .unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_completion_and_strips_think() {
        let v = json!({
            "model": "x",
            "choices": [{"message": {"role": "assistant", "content": "<think>hmm</think> Hello!"}}],
            "usage": {"prompt_tokens": 7, "completion_tokens": 2, "cost": 0.0001}
        });
        let r = parse(&v).unwrap();
        assert_eq!(r.text, "Hello!");
        assert_eq!(r.reported_cost_usd, Some(0.0001));
    }
}
