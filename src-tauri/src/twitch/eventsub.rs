//! EventSub over WebSocket: connection handling, reconnects and parsing of
//! the events HighBazar cares about.

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::{mpsc, watch};
use tokio::time::{Instant, timeout};
use tokio_tungstenite::tungstenite::Message;

use super::helix::Helix;

const EVENTSUB_URL: &str = "wss://eventsub.wss.twitch.tv/ws";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub id: String,
    pub user_id: String,
    pub login: String,
    pub display_name: String,
    pub text: String,
    pub color: String,
    pub badges: Vec<String>,
    pub is_broadcaster: bool,
    pub is_mod: bool,
    pub is_vip: bool,
    pub is_sub: bool,
    pub reply_parent_id: Option<String>,
    pub reply_parent_login: Option<String>,
    pub message_type: String,
    pub emote_count: u32,
    pub emotes: Vec<String>,
    /// Native Twitch emotes in this message: (name, emote id).
    pub twitch_emotes: Vec<(String, String)>,
    pub bits: u32,
    pub mentions: Vec<String>,
    /// Unix millis.
    pub ts: i64,
}

impl ChatMessage {
    pub fn role_rank(&self) -> u8 {
        if self.is_broadcaster {
            4
        } else if self.is_mod {
            3
        } else if self.is_vip {
            2
        } else if self.is_sub {
            1
        } else {
            0
        }
    }
}

pub fn role_requirement(role: &str) -> u8 {
    match role {
        "subscriber" => 1,
        "vip" => 2,
        "moderator" => 3,
        "broadcaster" => 4,
        _ => 0,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ChatNotification {
    pub notice_type: String,
    pub user_login: String,
    pub user_name: String,
    pub anonymous: bool,
    pub system_message: String,
    pub text: String,
    pub tier: String,
    pub months: u32,
    pub gift_count: u32,
    pub recipient: String,
    pub raid_viewers: u64,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum TwitchEvent {
    Connected,
    Disconnected(String),
    Chat(ChatMessage),
    Notification(ChatNotification),
    Follow { login: String, name: String },
    StreamOnline,
    StreamOffline,
    ChannelUpdate { title: String, category: String, language: String },
    SubscriptionFailed { kind: String, error: String },
}

pub struct EventSubConfig {
    pub helix: Helix,
    pub broadcaster_id: String,
}

#[derive(Deserialize)]
struct Frame {
    metadata: FrameMeta,
    #[serde(default)]
    payload: Value,
}

#[derive(Deserialize)]
struct FrameMeta {
    message_type: String,
    #[serde(default)]
    subscription_type: Option<String>,
}

type Ws = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

enum SessionEnd {
    Shutdown,
    Reconnect(String),
    Failed(String),
}

/// Runs until `shutdown` turns true. Reconnects with exponential backoff.
pub async fn run(cfg: EventSubConfig, tx: mpsc::Sender<TwitchEvent>, mut shutdown: watch::Receiver<bool>) {
    let mut backoff = 1u64;
    let mut url = EVENTSUB_URL.to_string();
    // When Twitch asks us to reconnect, subscriptions carry over.
    let mut resubscribe = true;
    loop {
        if *shutdown.borrow() {
            return;
        }
        let end = match tokio_tungstenite::connect_async(url.as_str()).await {
            Ok((ws, _)) => session(&cfg, ws, &tx, &mut shutdown, resubscribe).await,
            Err(e) => SessionEnd::Failed(format!("WebSocket connect failed: {e}")),
        };
        match end {
            SessionEnd::Shutdown => return,
            SessionEnd::Reconnect(new_url) => {
                url = new_url;
                resubscribe = false;
                backoff = 1;
                continue;
            }
            SessionEnd::Failed(reason) => {
                let _ = tx.send(TwitchEvent::Disconnected(reason)).await;
                url = EVENTSUB_URL.to_string();
                resubscribe = true;
            }
        }
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(backoff)) => {}
            _ = shutdown.changed() => return,
        }
        backoff = (backoff * 2).min(60);
    }
}

async fn session(
    cfg: &EventSubConfig,
    mut ws: Ws,
    tx: &mpsc::Sender<TwitchEvent>,
    shutdown: &mut watch::Receiver<bool>,
    subscribe: bool,
) -> SessionEnd {
    let mut keepalive = Duration::from_secs(30);
    let mut deadline = Instant::now() + Duration::from_secs(15);
    let mut welcomed = false;
    loop {
        let wait = deadline.saturating_duration_since(Instant::now());
        let msg = tokio::select! {
            _ = shutdown.changed() => {
                let _ = ws.close(None).await;
                return SessionEnd::Shutdown;
            }
            m = timeout(wait, ws.next()) => m,
        };
        let msg = match msg {
            Err(_) => return SessionEnd::Failed("EventSub keepalive timeout".into()),
            Ok(None) => return SessionEnd::Failed("EventSub connection closed".into()),
            Ok(Some(Err(e))) => return SessionEnd::Failed(format!("EventSub error: {e}")),
            Ok(Some(Ok(m))) => m,
        };
        deadline = Instant::now() + keepalive + Duration::from_secs(10);
        let text = match msg {
            Message::Text(t) => t.to_string(),
            Message::Ping(p) => {
                let _ = ws.send(Message::Pong(p)).await;
                continue;
            }
            Message::Close(frame) => {
                let reason = frame.map(|f| format!("{} {}", f.code, f.reason)).unwrap_or_default();
                return SessionEnd::Failed(format!("EventSub closed the connection {reason}"));
            }
            _ => continue,
        };
        let Ok(frame) = serde_json::from_str::<Frame>(&text) else {
            continue;
        };
        match frame.metadata.message_type.as_str() {
            "session_welcome" => {
                let session = &frame.payload["session"];
                let id = session["id"].as_str().unwrap_or_default().to_string();
                if let Some(k) = session["keepalive_timeout_seconds"].as_u64() {
                    keepalive = Duration::from_secs(k.max(10));
                    deadline = Instant::now() + keepalive + Duration::from_secs(10);
                }
                if !welcomed {
                    welcomed = true;
                    if subscribe {
                        subscribe_all(cfg, &id, tx).await;
                    }
                    let _ = tx.send(TwitchEvent::Connected).await;
                }
            }
            "session_keepalive" => {}
            "session_reconnect" => {
                if let Some(u) = frame.payload["session"]["reconnect_url"].as_str() {
                    return SessionEnd::Reconnect(u.to_string());
                }
            }
            "revocation" => {
                let kind = frame.payload["subscription"]["type"].as_str().unwrap_or_default().to_string();
                let status = frame.payload["subscription"]["status"].as_str().unwrap_or_default().to_string();
                let _ = tx.send(TwitchEvent::SubscriptionFailed { kind, error: format!("revoked: {status}") }).await;
            }
            "notification" => {
                let kind = frame.metadata.subscription_type.unwrap_or_default();
                if let Some(ev) = parse_event(&kind, &frame.payload["event"])
                    && tx.send(ev).await.is_err() {
                        return SessionEnd::Shutdown;
                    }
            }
            _ => {}
        }
    }
}

async fn subscribe_all(cfg: &EventSubConfig, session_id: &str, tx: &mpsc::Sender<TwitchEvent>) {
    let b = cfg.broadcaster_id.as_str();
    let subs: Vec<(&str, &str, Value)> = vec![
        ("channel.chat.message", "1", json!({ "broadcaster_user_id": b, "user_id": b })),
        ("channel.chat.notification", "1", json!({ "broadcaster_user_id": b, "user_id": b })),
        ("channel.follow", "2", json!({ "broadcaster_user_id": b, "moderator_user_id": b })),
        ("stream.online", "1", json!({ "broadcaster_user_id": b })),
        ("stream.offline", "1", json!({ "broadcaster_user_id": b })),
        ("channel.update", "2", json!({ "broadcaster_user_id": b })),
    ];
    for (kind, version, condition) in subs {
        if let Err(e) = cfg.helix.create_subscription(b, kind, version, condition, session_id).await {
            log::warn!("EventSub subscription {kind} failed: {e}");
            let _ = tx
                .send(TwitchEvent::SubscriptionFailed { kind: kind.to_string(), error: e.to_string() })
                .await;
        }
    }
}

fn s(v: &Value) -> String {
    v.as_str().unwrap_or_default().to_string()
}

pub fn parse_event(kind: &str, e: &Value) -> Option<TwitchEvent> {
    match kind {
        "channel.chat.message" => Some(TwitchEvent::Chat(parse_chat(e))),
        "channel.chat.notification" => {
            let notice = s(&e["notice_type"]);
            let mut n = ChatNotification {
                notice_type: notice.clone(),
                user_login: s(&e["chatter_user_login"]),
                user_name: s(&e["chatter_user_name"]),
                anonymous: e["chatter_is_anonymous"].as_bool().unwrap_or(false),
                system_message: s(&e["system_message"]),
                text: s(&e["message"]["text"]),
                ..Default::default()
            };
            match notice.as_str() {
                "sub" => {
                    n.tier = s(&e["sub"]["sub_tier"]);
                    n.months = e["sub"]["duration_months"].as_u64().unwrap_or(1) as u32;
                }
                "resub" => {
                    n.tier = s(&e["resub"]["sub_tier"]);
                    n.months = e["resub"]["cumulative_months"].as_u64().unwrap_or(0) as u32;
                }
                "sub_gift" => {
                    n.tier = s(&e["sub_gift"]["sub_tier"]);
                    n.gift_count = 1;
                    n.recipient = s(&e["sub_gift"]["recipient_user_name"]);
                    // Part of a community gift: the summary event covers it.
                    if e["sub_gift"]["community_gift_id"].as_str().is_some_and(|x| !x.is_empty()) {
                        return None;
                    }
                }
                "community_sub_gift" => {
                    n.tier = s(&e["community_sub_gift"]["sub_tier"]);
                    n.gift_count = e["community_sub_gift"]["total"].as_u64().unwrap_or(1) as u32;
                }
                "raid" => {
                    n.user_login = s(&e["raid"]["user_login"]);
                    n.user_name = s(&e["raid"]["user_name"]);
                    n.raid_viewers = e["raid"]["viewer_count"].as_u64().unwrap_or(0);
                }
                _ => {}
            }
            Some(TwitchEvent::Notification(n))
        }
        "channel.follow" => Some(TwitchEvent::Follow {
            login: s(&e["user_login"]),
            name: s(&e["user_name"]),
        }),
        "stream.online" => Some(TwitchEvent::StreamOnline),
        "stream.offline" => Some(TwitchEvent::StreamOffline),
        "channel.update" => Some(TwitchEvent::ChannelUpdate {
            title: s(&e["title"]),
            category: s(&e["category_name"]),
            language: s(&e["language"]),
        }),
        _ => None,
    }
}

pub fn parse_chat(e: &Value) -> ChatMessage {
    let badges: Vec<String> = e["badges"]
        .as_array()
        .map(|a| a.iter().map(|b| s(&b["set_id"])).collect())
        .unwrap_or_default();
    let has = |id: &str| badges.iter().any(|b| b == id);
    let mut emotes = Vec::new();
    let mut twitch_emotes = Vec::new();
    let mut mentions = Vec::new();
    if let Some(frags) = e["message"]["fragments"].as_array() {
        for f in frags {
            match f["type"].as_str() {
                Some("emote") => {
                    emotes.push(s(&f["text"]));
                    twitch_emotes.push((s(&f["text"]), s(&f["emote"]["id"])));
                }
                Some("mention") => mentions.push(s(&f["mention"]["user_login"])),
                _ => {}
            }
        }
    }
    let reply = &e["reply"];
    ChatMessage {
        id: s(&e["message_id"]),
        user_id: s(&e["chatter_user_id"]),
        login: s(&e["chatter_user_login"]),
        display_name: s(&e["chatter_user_name"]),
        text: s(&e["message"]["text"]),
        color: s(&e["color"]),
        is_broadcaster: has("broadcaster"),
        is_mod: has("moderator") || has("lead_moderator"),
        is_vip: has("vip"),
        is_sub: has("subscriber") || has("founder"),
        badges,
        reply_parent_id: reply["parent_message_id"].as_str().map(str::to_string),
        reply_parent_login: reply["parent_user_login"].as_str().map(str::to_string),
        message_type: s(&e["message_type"]),
        emote_count: emotes.len() as u32,
        emotes,
        twitch_emotes,
        bits: e["cheer"]["bits"].as_u64().unwrap_or(0) as u32,
        mentions,
        ts: chrono::Utc::now().timestamp_millis(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_chat_message() {
        let e = json!({
            "broadcaster_user_id": "1", "chatter_user_id": "2", "chatter_user_login": "bob",
            "chatter_user_name": "Bob", "message_id": "m1",
            "message": { "text": "hi @alice Kappa", "fragments": [
                {"type": "text", "text": "hi "},
                {"type": "mention", "text": "@alice", "mention": {"user_id": "3", "user_login": "alice", "user_name": "Alice"}},
                {"type": "emote", "text": "Kappa", "emote": {"id": "25"}}
            ]},
            "color": "#FF0000",
            "badges": [{"set_id": "moderator", "id": "1", "info": ""}, {"set_id": "subscriber", "id": "12", "info": "12"}],
            "message_type": "text",
            "reply": null
        });
        let m = parse_chat(&e);
        assert_eq!(m.login, "bob");
        assert!(m.is_mod && m.is_sub && !m.is_vip);
        assert_eq!(m.emotes, vec!["Kappa"]);
        assert_eq!(m.twitch_emotes, vec![("Kappa".to_string(), "25".to_string())]);
        assert_eq!(m.mentions, vec!["alice"]);
        assert_eq!(m.role_rank(), 3);
    }

    /// Network test: `cargo test eventsub_welcome -- --ignored`
    #[tokio::test]
    #[ignore]
    async fn eventsub_welcome() {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let (mut ws, _) = tokio_tungstenite::connect_async(EVENTSUB_URL).await.expect("connect");
        let text = loop {
            let msg = tokio::time::timeout(Duration::from_secs(15), ws.next()).await.unwrap().unwrap().unwrap();
            match msg {
                Message::Text(t) => break t.to_string(),
                other => println!("non-text frame: {other:?}"),
            }
        };
        let frame: Frame = serde_json::from_str(&text).unwrap();
        assert_eq!(frame.metadata.message_type, "session_welcome");
        assert!(frame.payload["session"]["id"].as_str().is_some());
    }

    #[test]
    fn skips_community_gift_parts() {
        let e = json!({ "notice_type": "sub_gift", "sub_gift": { "community_gift_id": "abc", "sub_tier": "1000" } });
        assert!(parse_event("channel.chat.notification", &e).is_none());
    }
}
