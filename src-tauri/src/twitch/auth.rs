//! Twitch OAuth via the Device Code Grant Flow (public client, no secret).

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::secrets::{Secrets, twitch_key_name};

pub const STREAMER_SCOPES: &[&str] = &[
    "user:read:chat",
    "user:write:chat",
    "moderator:read:followers",
    "moderator:manage:banned_users",
    "moderator:manage:chat_messages",
    "moderator:manage:warnings",
    "moderator:read:chatters",
];

pub const BOT_SCOPES: &[&str] = &["user:write:chat"];

/// Extra scopes of the channel manager, per level (each level adds to the previous).
pub const CHANNEL_SIMPLE_SCOPES: &[&str] = &["channel:manage:broadcast"];
pub const CHANNEL_ADVANCED_SCOPES: &[&str] = &[
    "moderator:manage:chat_settings",
    "moderator:manage:announcements",
    "moderator:manage:shoutouts",
    "channel:manage:raids",
    "clips:edit",
];
pub const CHANNEL_EXPERT_SCOPES: &[&str] = &[
    "channel:manage:polls",
    "channel:manage:predictions",
    "channel:manage:redemptions",
    "channel:manage:moderators",
    "channel:manage:vips",
    "moderator:manage:automod_settings",
    "moderator:manage:blocked_terms",
    "moderator:manage:shield_mode",
    "moderation:read",
    "channel:edit:commercial",
    "channel:read:ads",
    "channel:manage:ads",
    "channel:manage:schedule",
    "channel:read:subscriptions",
    "channel:read:editors",
    "user:edit",
    "user:manage:chat_color",
    "moderator:manage:unban_requests",
    "moderator:manage:suspicious_users",
    "channel:read:hype_train",
];

/// Scopes for a streamer login with the channel manager at `level`
/// ("simple" | "advanced" | "expert").
pub fn streamer_scopes(level: &str) -> Vec<&'static str> {
    let mut s: Vec<&'static str> = STREAMER_SCOPES.to_vec();
    s.extend(CHANNEL_SIMPLE_SCOPES);
    if matches!(level, "advanced" | "expert") {
        s.extend(CHANNEL_ADVANCED_SCOPES);
    }
    if level == "expert" {
        s.extend(CHANNEL_EXPERT_SCOPES);
    }
    s
}

/// Twitch's answer to `POST /oauth2/device` (snake_case on the wire).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCode {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSet {
    pub access_token: String,
    pub refresh_token: String,
    /// Unix seconds.
    pub expires_at: i64,
    pub scopes: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<i64>,
    #[serde(default)]
    scope: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    #[serde(default)]
    message: String,
}

pub enum PollResult {
    Pending,
    SlowDown,
    Done(TokenSet),
}

#[derive(Debug, Clone, Deserialize)]
pub struct ValidateInfo {
    pub login: String,
    pub user_id: String,
    #[serde(default)]
    pub scopes: Vec<String>,
}

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

fn token_from(resp: TokenResponse, previous_refresh: Option<&str>) -> TokenSet {
    TokenSet {
        access_token: resp.access_token,
        refresh_token: resp
            .refresh_token
            .or_else(|| previous_refresh.map(str::to_string))
            .unwrap_or_default(),
        expires_at: now() + resp.expires_in.unwrap_or(3600),
        scopes: resp.scope,
    }
}

async fn twitch_error(resp: reqwest::Response) -> AppError {
    let status = resp.status().as_u16();
    let text = resp.text().await.unwrap_or_default();
    let message = serde_json::from_str::<ErrorResponse>(&text)
        .map(|e| e.message)
        .unwrap_or(text);
    AppError::Twitch { status, message }
}

pub async fn start_device_flow(http: &reqwest::Client, client_id: &str, scopes: &[&str]) -> AppResult<DeviceCode> {
    let resp = http
        .post("https://id.twitch.tv/oauth2/device")
        .form(&[("client_id", client_id), ("scopes", &scopes.join(" "))])
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(twitch_error(resp).await);
    }
    resp.json::<DeviceCode>().await.map_err(|e| AppError::msg(format!("Unexpected device code response: {e}")))
}

pub async fn poll_device_token(
    http: &reqwest::Client,
    client_id: &str,
    scopes: &[&str],
    device_code: &str,
) -> AppResult<PollResult> {
    let resp = http
        .post("https://id.twitch.tv/oauth2/token")
        .form(&[
            ("client_id", client_id),
            ("scopes", &scopes.join(" ")),
            ("device_code", device_code),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ])
        .send()
        .await?;
    if resp.status().is_success() {
        let t: TokenResponse = resp.json().await?;
        return Ok(PollResult::Done(token_from(t, None)));
    }
    let status = resp.status().as_u16();
    let text = resp.text().await.unwrap_or_default();
    let msg = serde_json::from_str::<ErrorResponse>(&text)
        .map(|e| e.message)
        .unwrap_or_else(|_| text.clone());
    match msg.as_str() {
        "authorization_pending" => Ok(PollResult::Pending),
        "slow_down" => Ok(PollResult::SlowDown),
        _ => Err(AppError::Twitch { status, message: msg }),
    }
}

pub async fn refresh(http: &reqwest::Client, client_id: &str, refresh_token: &str) -> AppResult<TokenSet> {
    let resp = http
        .post("https://id.twitch.tv/oauth2/token")
        .form(&[
            ("client_id", client_id),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
        ])
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(twitch_error(resp).await);
    }
    let t: TokenResponse = resp.json().await?;
    Ok(token_from(t, Some(refresh_token)))
}

/// Twitch requires apps to validate tokens on start and hourly afterwards.
pub async fn validate(http: &reqwest::Client, access_token: &str) -> AppResult<Option<ValidateInfo>> {
    let resp = http
        .get("https://id.twitch.tv/oauth2/validate")
        .header("Authorization", format!("OAuth {access_token}"))
        .send()
        .await?;
    if resp.status().as_u16() == 401 {
        return Ok(None);
    }
    if !resp.status().is_success() {
        return Err(twitch_error(resp).await);
    }
    Ok(Some(resp.json().await?))
}

pub async fn revoke(http: &reqwest::Client, client_id: &str, access_token: &str) {
    let _ = http
        .post("https://id.twitch.tv/oauth2/revoke")
        .form(&[("client_id", client_id), ("token", access_token)])
        .send()
        .await;
}

/// Hands out valid access tokens and refreshes them transparently. Refresh
/// tokens of public clients are single-use, so refreshes are serialized.
pub struct TokenManager {
    http: reqwest::Client,
    secrets: Arc<Secrets>,
    cache: parking_lot::Mutex<HashMap<String, TokenSet>>,
    refresh_lock: tokio::sync::Mutex<()>,
}

impl TokenManager {
    pub fn new(http: reqwest::Client, secrets: Arc<Secrets>) -> Self {
        Self {
            http,
            secrets,
            cache: parking_lot::Mutex::new(HashMap::new()),
            refresh_lock: tokio::sync::Mutex::new(()),
        }
    }

    pub fn store(&self, user_id: &str, tokens: &TokenSet) -> AppResult<()> {
        self.secrets
            .set(&twitch_key_name(user_id), &serde_json::to_string(tokens)?)?;
        self.cache.lock().insert(user_id.to_string(), tokens.clone());
        Ok(())
    }

    pub fn load(&self, user_id: &str) -> Option<TokenSet> {
        if let Some(t) = self.cache.lock().get(user_id) {
            return Some(t.clone());
        }
        let raw = self.secrets.get(&twitch_key_name(user_id))?;
        let t: TokenSet = serde_json::from_str(&raw).ok()?;
        self.cache.lock().insert(user_id.to_string(), t.clone());
        Some(t)
    }

    pub fn remove(&self, user_id: &str) -> AppResult<()> {
        self.cache.lock().remove(user_id);
        self.secrets.delete(&twitch_key_name(user_id))
    }

    pub fn has(&self, user_id: &str) -> bool {
        self.load(user_id).is_some()
    }

    /// Returns a token valid for at least five more minutes.
    pub async fn access_token(&self, client_id: &str, user_id: &str) -> AppResult<String> {
        let current = self
            .load(user_id)
            .ok_or_else(|| AppError::msg("Twitch account is not connected. Please log in again."))?;
        if current.expires_at - now() > 300 {
            return Ok(current.access_token);
        }
        self.force_refresh(client_id, user_id, &current.access_token).await
    }

    /// Refreshes the token unless another task already did so since
    /// `stale_token` was handed out.
    pub async fn force_refresh(&self, client_id: &str, user_id: &str, stale_token: &str) -> AppResult<String> {
        let _guard = self.refresh_lock.lock().await;
        let current = self
            .load(user_id)
            .ok_or_else(|| AppError::msg("Twitch account is not connected. Please log in again."))?;
        if current.access_token != stale_token && current.expires_at - now() > 300 {
            return Ok(current.access_token);
        }
        match refresh(&self.http, client_id, &current.refresh_token).await {
            Ok(mut fresh) => {
                if fresh.scopes.is_empty() {
                    fresh.scopes = current.scopes.clone();
                }
                self.store(user_id, &fresh)?;
                Ok(fresh.access_token)
            }
            // Twitch rejected the refresh token (revoked, password change, 30 days unused).
            Err(e @ AppError::Twitch { .. }) => Err(AppError::msg(format!(
                "Twitch session expired ({e}). Please reconnect the account in Channels."
            ))),
            // Network trouble: the session itself may still be fine.
            Err(e) => Err(AppError::msg(format!("Twitch is not reachable right now ({e})."))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_twitch_device_code_response() {
        let body = r#"{"device_code":"abc","expires_in":1800,"interval":5,"user_code":"QBQLNSHC","verification_uri":"https://www.twitch.tv/activate?device-code=QBQLNSHC"}"#;
        let dc: DeviceCode = serde_json::from_str(body).unwrap();
        assert_eq!(dc.user_code, "QBQLNSHC");
        assert_eq!(dc.interval, 5);
    }

    #[test]
    fn parses_token_response() {
        let body = r#"{"access_token":"a","expires_in":14400,"refresh_token":"r","scope":["user:read:chat"],"token_type":"bearer"}"#;
        let t = token_from(serde_json::from_str(body).unwrap(), None);
        assert_eq!(t.refresh_token, "r");
        assert_eq!(t.scopes, vec!["user:read:chat"]);
    }
}
