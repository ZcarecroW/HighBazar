//! The bot runtime: one session at a time, bound to a channel profile.

pub mod context;
mod engine;
pub mod moderation;
pub mod output;
pub mod prompt;
pub mod research;
pub mod usage;

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc, watch};

use crate::error::{AppError, AppResult};
use crate::state::AppState;

pub const EVT_FEED: &str = "hb:feed";
pub const EVT_STATUS: &str = "hb:status";
pub const EVT_PENDING: &str = "hb:pending";
pub const EVT_MODLOG: &str = "hb:modlog";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedItem {
    pub id: String,
    pub ts: i64,
    /// chat | bot | mod | event | system | error
    pub kind: String,
    pub user: String,
    pub login: String,
    pub user_id: String,
    pub message_id: String,
    pub color: String,
    pub badges: Vec<String>,
    pub text: String,
    pub meta: Value,
}

impl FeedItem {
    pub fn new(kind: &str, text: impl Into<String>) -> Self {
        Self {
            id: crate::config::new_id(),
            ts: chrono::Utc::now().timestamp_millis(),
            kind: kind.into(),
            user: String::new(),
            login: String::new(),
            user_id: String::new(),
            message_id: String::new(),
            color: String::new(),
            badges: Vec::new(),
            text: text.into(),
            meta: Value::Null,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BotStatus {
    pub running: bool,
    pub sandbox: bool,
    pub connected: bool,
    pub channel_id: String,
    pub channel_login: String,
    pub channel_name: String,
    pub sender_login: String,
    pub live: bool,
    pub game: String,
    pub title: String,
    pub language: String,
    pub viewers: u64,
    pub started_at: Option<String>,
    /// Unix ms of the next timed message (None = not scheduled).
    pub next_message_at: Option<i64>,
    /// Why timed messages are currently paused (empty = not paused).
    pub paused_reason: String,
    pub chat_rate: u32,
    pub emote_count: usize,
    pub knowledge_items: usize,
    pub researching: bool,
    pub cost_today: f64,
    pub budget: f64,
    pub messages_today: u64,
    pub last_error: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingMessage {
    pub id: String,
    pub text: String,
    pub reason: String,
    pub created: i64,
    /// Unix ms when it will be sent automatically (None = needs approval).
    pub send_at: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimMessage {
    pub user: String,
    pub text: String,
    /// "" | "sub" | "vip" | "mod" | "broadcaster"
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub bits: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualAction {
    pub user_id: String,
    pub login: String,
    pub display_name: String,
    pub message_id: String,
    pub text: String,
    pub kind: String,
    #[serde(default)]
    pub duration_secs: u32,
}

#[derive(Debug)]
pub enum Control {
    SayNow,
    Approve(String),
    Reject(String),
    EditPending(String, String),
    Simulate(SimMessage),
    SimulateEvent(String, String),
    ApplySuggestion(String),
    Dismiss(String),
    Undo(String),
    Manual(ManualAction),
    RefreshStream,
}

struct Running {
    shutdown: watch::Sender<bool>,
    ctrl: mpsc::UnboundedSender<Control>,
    handle: tokio::task::JoinHandle<()>,
    channel_id: String,
    sandbox: bool,
}

#[derive(Default)]
pub struct BotManager {
    running: tokio::sync::Mutex<Option<Running>>,
}

impl BotManager {
    pub async fn start(&self, app: AppHandle, state: Arc<AppState>, channel_id: String, sandbox: bool) -> AppResult<()> {
        self.stop(&app).await;
        let session = engine::Session::create(app.clone(), state, &channel_id, sandbox).await?;
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let (ctrl_tx, ctrl_rx) = mpsc::unbounded_channel();
        let handle = tokio::spawn(engine::run(session, ctrl_rx, shutdown_rx));
        *self.running.lock().await = Some(Running {
            shutdown: shutdown_tx,
            ctrl: ctrl_tx,
            handle,
            channel_id,
            sandbox,
        });
        Ok(())
    }

    pub async fn stop(&self, app: &AppHandle) {
        let running = self.running.lock().await.take();
        if let Some(r) = running {
            let _ = r.shutdown.send(true);
            let _ = tokio::time::timeout(std::time::Duration::from_secs(5), r.handle).await;
            let _ = app.emit(EVT_STATUS, BotStatus::default());
        }
    }

    pub async fn control(&self, c: Control) -> AppResult<()> {
        let guard = self.running.lock().await;
        let r = guard.as_ref().ok_or_else(|| AppError::msg("The bot is not running."))?;
        r.ctrl.send(c).map_err(|_| AppError::msg("The bot is shutting down."))
    }

    pub async fn current(&self) -> Option<(String, bool)> {
        let guard = self.running.lock().await;
        guard
            .as_ref()
            .filter(|r| !r.handle.is_finished())
            .map(|r| (r.channel_id.clone(), r.sandbox))
    }
}
