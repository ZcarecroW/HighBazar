//! Minimal Twitch Helix API client.

use std::sync::Arc;

use reqwest::Method;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

use super::auth::TokenManager;
use crate::error::{AppError, AppResult};

const API: &str = "https://api.twitch.tv/helix";

#[derive(Clone)]
pub struct Helix {
    pub http: reqwest::Client,
    pub client_id: String,
    pub tokens: Arc<TokenManager>,
}

#[derive(Debug, Deserialize)]
struct DataList<T> {
    data: Vec<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct User {
    pub id: String,
    pub login: String,
    pub display_name: String,
    #[serde(default)]
    pub profile_image_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChannelInfo {
    #[serde(default)]
    pub broadcaster_language: String,
    #[serde(default)]
    pub game_id: String,
    #[serde(default)]
    pub game_name: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StreamInfo {
    #[serde(default)]
    pub viewer_count: u64,
    #[serde(default)]
    pub started_at: String,
    #[serde(default)]
    pub game_name: String,
    #[serde(default)]
    pub title: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SendResult {
    pub message_id: String,
    pub is_sent: bool,
    pub drop_reason: Option<DropReason>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DropReason {
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Emote {
    pub id: String,
    pub name: String,
}

impl Helix {
    async fn request<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        user_id: &str,
        query: &[(&str, &str)],
        body: Option<Value>,
    ) -> AppResult<Option<T>> {
        let mut token = self.tokens.access_token(&self.client_id, user_id).await?;
        for attempt in 0..2 {
            let mut req = self
                .http
                .request(method.clone(), format!("{API}{path}"))
                .header("Client-Id", &self.client_id)
                .bearer_auth(&token)
                .query(query);
            if let Some(b) = &body {
                req = req.json(b);
            }
            let resp = req.send().await?;
            let status = resp.status().as_u16();
            if status == 401 && attempt == 0 {
                token = self.tokens.force_refresh(&self.client_id, user_id, &token).await?;
                continue;
            }
            if status == 204 {
                return Ok(None);
            }
            let text = resp.text().await.unwrap_or_default();
            if !(200..300).contains(&status) {
                let message = serde_json::from_str::<Value>(&text)
                    .ok()
                    .and_then(|v| v.get("message").and_then(|m| m.as_str()).map(str::to_string))
                    .unwrap_or(text);
                return Err(AppError::Twitch { status, message });
            }
            if text.trim().is_empty() {
                return Ok(None);
            }
            return Ok(Some(serde_json::from_str(&text)?));
        }
        Err(AppError::msg("Twitch authentication failed"))
    }

    pub async fn users_by_id(&self, as_user: &str, ids: &[&str]) -> AppResult<Vec<User>> {
        let q: Vec<(&str, &str)> = ids.iter().map(|id| ("id", *id)).collect();
        let r: Option<DataList<User>> = self.request(Method::GET, "/users", as_user, &q, None).await?;
        Ok(r.map(|d| d.data).unwrap_or_default())
    }

    pub async fn user_by_login(&self, as_user: &str, login: &str) -> AppResult<Option<User>> {
        let r: Option<DataList<User>> = self
            .request(Method::GET, "/users", as_user, &[("login", login)], None)
            .await?;
        Ok(r.and_then(|d| d.data.into_iter().next()))
    }

    pub async fn channel(&self, as_user: &str, broadcaster_id: &str) -> AppResult<Option<ChannelInfo>> {
        let r: Option<DataList<ChannelInfo>> = self
            .request(Method::GET, "/channels", as_user, &[("broadcaster_id", broadcaster_id)], None)
            .await?;
        Ok(r.and_then(|d| d.data.into_iter().next()))
    }

    pub async fn stream(&self, as_user: &str, user_id: &str) -> AppResult<Option<StreamInfo>> {
        let r: Option<DataList<StreamInfo>> = self
            .request(Method::GET, "/streams", as_user, &[("user_id", user_id)], None)
            .await?;
        Ok(r.and_then(|d| d.data.into_iter().next()))
    }

    pub async fn send_chat(
        &self,
        sender_id: &str,
        broadcaster_id: &str,
        message: &str,
        reply_to: Option<&str>,
    ) -> AppResult<SendResult> {
        let mut body = json!({
            "broadcaster_id": broadcaster_id,
            "sender_id": sender_id,
            "message": message,
        });
        if let Some(parent) = reply_to {
            body["reply_parent_message_id"] = json!(parent);
        }
        let r: Option<DataList<SendResult>> = self
            .request(Method::POST, "/chat/messages", sender_id, &[], Some(body))
            .await?;
        r.and_then(|d| d.data.into_iter().next())
            .ok_or_else(|| AppError::msg("Twitch returned no send result"))
    }

    pub async fn delete_message(&self, moderator_id: &str, broadcaster_id: &str, message_id: &str) -> AppResult<()> {
        self.request::<Value>(
            Method::DELETE,
            "/moderation/chat",
            moderator_id,
            &[
                ("broadcaster_id", broadcaster_id),
                ("moderator_id", moderator_id),
                ("message_id", message_id),
            ],
            None,
        )
        .await?;
        Ok(())
    }

    /// `duration_secs` = None bans permanently.
    pub async fn ban(
        &self,
        moderator_id: &str,
        broadcaster_id: &str,
        user_id: &str,
        duration_secs: Option<u32>,
        reason: &str,
    ) -> AppResult<()> {
        let mut data = json!({ "user_id": user_id, "reason": truncate(reason, 500) });
        if let Some(d) = duration_secs {
            data["duration"] = json!(d.clamp(1, 1_209_600));
        }
        self.request::<Value>(
            Method::POST,
            "/moderation/bans",
            moderator_id,
            &[("broadcaster_id", broadcaster_id), ("moderator_id", moderator_id)],
            Some(json!({ "data": data })),
        )
        .await?;
        Ok(())
    }

    pub async fn unban(&self, moderator_id: &str, broadcaster_id: &str, user_id: &str) -> AppResult<()> {
        self.request::<Value>(
            Method::DELETE,
            "/moderation/bans",
            moderator_id,
            &[
                ("broadcaster_id", broadcaster_id),
                ("moderator_id", moderator_id),
                ("user_id", user_id),
            ],
            None,
        )
        .await?;
        Ok(())
    }

    pub async fn warn(&self, moderator_id: &str, broadcaster_id: &str, user_id: &str, reason: &str) -> AppResult<()> {
        self.request::<Value>(
            Method::POST,
            "/moderation/warnings",
            moderator_id,
            &[("broadcaster_id", broadcaster_id), ("moderator_id", moderator_id)],
            Some(json!({ "data": { "user_id": user_id, "reason": truncate(reason, 500) } })),
        )
        .await?;
        Ok(())
    }

    pub async fn create_subscription(
        &self,
        as_user: &str,
        kind: &str,
        version: &str,
        condition: Value,
        session_id: &str,
    ) -> AppResult<()> {
        let body = json!({
            "type": kind,
            "version": version,
            "condition": condition,
            "transport": { "method": "websocket", "session_id": session_id },
        });
        self.request::<Value>(Method::POST, "/eventsub/subscriptions", as_user, &[], Some(body))
            .await?;
        Ok(())
    }

    pub async fn global_emotes(&self, as_user: &str) -> AppResult<Vec<Emote>> {
        let r: Option<DataList<Emote>> = self
            .request(Method::GET, "/chat/emotes/global", as_user, &[], None)
            .await?;
        Ok(r.map(|d| d.data).unwrap_or_default())
    }

    pub async fn channel_emotes(&self, as_user: &str, broadcaster_id: &str) -> AppResult<Vec<Emote>> {
        let r: Option<DataList<Emote>> = self
            .request(Method::GET, "/chat/emotes", as_user, &[("broadcaster_id", broadcaster_id)], None)
            .await?;
        Ok(r.map(|d| d.data).unwrap_or_default())
    }

    /// Generic Helix call for the channel manager (path must be allow-listed by the caller).
    pub async fn raw(
        &self,
        method: Method,
        path: &str,
        as_user: &str,
        query: &[(&str, &str)],
        body: Option<Value>,
    ) -> AppResult<Value> {
        Ok(self.request::<Value>(method, path, as_user, query, body).await?.unwrap_or(Value::Null))
    }
}

pub fn truncate(s: &str, max_chars: usize) -> String {
    s.chars().take(max_chars).collect()
}
