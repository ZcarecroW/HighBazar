//! Tauri commands called by the frontend.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter, State};

use crate::ai::{self, AiRequest, ProviderInfo};
use crate::bot::moderation::ModLogEntry;
use crate::bot::usage::UsageStats;
use crate::bot::{Control, ManualAction, SimMessage};
use crate::config::{AiProfile, Config, TwitchAccount, new_id};
use crate::error::{AppError, AppResult};
use crate::secrets::ai_key_name;
use crate::state::AppState;
use crate::storage;
use crate::twitch::auth::{self, BOT_SCOPES, PollResult};

type St<'a> = State<'a, Arc<AppState>>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bootstrap {
    config: Config,
    providers: Vec<ProviderInfo>,
    version: String,
    secrets_backend: String,
    data_dir: String,
    client_id_configured: bool,
    key_status: BTreeMap<String, bool>,
    account_status: BTreeMap<String, bool>,
    running: Option<RunningInfo>,
    os: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunningInfo {
    channel_id: String,
    sandbox: bool,
}

fn key_status(st: &AppState, cfg: &Config) -> BTreeMap<String, bool> {
    let mut refs: Vec<String> = ai::providers().iter().map(|p| p.id.to_string()).collect();
    refs.extend(cfg.ai_profiles.iter().map(ai::key_ref));
    refs.extend(crate::search::SOURCES.iter().filter(|(_, needs_key)| *needs_key).map(|(id, _)| crate::search::key_ref(id)));
    refs.sort();
    refs.dedup();
    refs.into_iter()
        .map(|r| {
            let has = st.secrets.get(&ai_key_name(&r)).is_some_and(|k| !k.is_empty());
            (r, has)
        })
        .collect()
}

#[tauri::command]
pub async fn get_bootstrap(st: St<'_>) -> AppResult<Bootstrap> {
    let cfg = st.config.read().clone();
    let account_status = cfg.accounts.iter().map(|a| (a.id.clone(), st.tokens.has(&a.id))).collect();
    let running = st.bot.current().await.map(|(channel_id, sandbox)| RunningInfo { channel_id, sandbox });
    Ok(Bootstrap {
        key_status: key_status(&st, &cfg),
        client_id_configured: !cfg.twitch_client_id().is_empty(),
        config: cfg,
        providers: ai::providers(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        secrets_backend: st.secrets.backend().to_string(),
        data_dir: st.data_dir.to_string_lossy().to_string(),
        account_status,
        running,
        os: std::env::consts::OS.to_string(),
    })
}

#[tauri::command]
pub async fn save_config(app: AppHandle, st: St<'_>, config: Config) -> AppResult<Config> {
    let mut cfg = config;
    // Accounts are managed by the backend only.
    cfg.accounts = st.config.read().accounts.clone();
    cfg.normalize();
    *st.config.write() = cfg.clone();
    st.save_config()?;
    crate::shell::apply_settings(&app, st.inner());
    Ok(cfg)
}

#[tauri::command]
pub async fn get_key_status(st: St<'_>) -> AppResult<BTreeMap<String, bool>> {
    let cfg = st.config.read().clone();
    Ok(key_status(&st, &cfg))
}

#[tauri::command]
pub async fn set_ai_key(st: St<'_>, key_ref: String, key: String) -> AppResult<()> {
    let k = key.trim();
    if k.is_empty() {
        st.secrets.delete(&ai_key_name(&key_ref))
    } else {
        st.secrets.set(&ai_key_name(&key_ref), k)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiTestResult {
    text: String,
    input_tokens: u64,
    output_tokens: u64,
    cost_usd: f64,
    millis: u128,
    model: String,
    searches: u32,
}

#[tauri::command]
pub async fn ai_test(st: St<'_>, profile: AiProfile, prompt: Option<String>, web_search: Option<bool>) -> AppResult<AiTestResult> {
    let key = st.ai_key(&profile);
    let web_search = web_search.unwrap_or(false);
    let mut system = String::from("You are HighBazar, a friendly Twitch chat companion. Reply with one short, fun chat message (max 150 characters).");
    if web_search {
        system.push_str(" Look up current facts on the web when the message needs them. No links or source lists.");
    }
    let req = AiRequest {
        system,
        user: prompt.unwrap_or_else(|| "Say hello to the chat and introduce yourself in one sentence.".into()),
        web_search,
        ..Default::default()
    };
    let start = Instant::now();
    let cfg = st.config.read().settings.web_search.clone();
    let own = ai::OwnSearch { key: st.search_key(&cfg.source), cfg: &cfg, lang: String::new() };
    let resp = ai::complete_searching(&st.http, &profile, key.as_deref(), &req, &own).await?;
    Ok(AiTestResult {
        cost_usd: resp.cost(&profile),
        text: resp.text,
        input_tokens: resp.input_tokens,
        output_tokens: resp.output_tokens,
        millis: start.elapsed().as_millis(),
        model: resp.model,
        searches: resp.searches,
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CliStatus {
    path: Option<String>,
    /// e.g. "Logged in using ChatGPT"; `None` = not logged in or unknown.
    login: Option<String>,
    error: String,
}

/// Finds the Codex CLI and checks its login.
#[tauri::command]
pub async fn codex_cli_status(path: Option<String>) -> CliStatus {
    let path = path.unwrap_or_default();
    let Some(exe) = ai::detect_codex_cli(&path) else {
        return CliStatus { path: None, login: None, error: String::new() };
    };
    let (login, error) = match ai::codex_login_status(&path).await {
        Ok(l) => (l, String::new()),
        Err(e) => (None, e.to_string()),
    };
    CliStatus { path: Some(exe.to_string_lossy().to_string()), login, error }
}

/// Runs one query with HighBazar's own web search (settings → web search).
#[tauri::command]
pub async fn web_search_test(st: St<'_>, settings: crate::config::WebSearchSettings, query: String) -> AppResult<Vec<crate::search::Hit>> {
    let key = st.search_key(&settings.source);
    crate::search::search(&st.http, &settings, key.as_deref(), &query, "").await
}

#[tauri::command]
pub async fn ai_list_models(st: St<'_>, profile: AiProfile) -> AppResult<Vec<ai::ModelInfo>> {
    let key = st.ai_key(&profile);
    ai::list_models(&st.http, &profile, key.as_deref()).await
}

#[tauri::command]
pub fn detect_claude_cli(path: Option<String>) -> Option<String> {
    ai::detect_claude_cli(path.as_deref().unwrap_or("")).map(|p| p.to_string_lossy().to_string())
}

// ---------------------------------------------------------------------------
// Twitch login (device code flow)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginStart {
    login_id: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct LoginEvent {
    login_id: String,
    /// done | error | expired | cancelled
    status: String,
    account: Option<TwitchAccount>,
    error: String,
}

#[tauri::command]
pub async fn twitch_login_start(app: AppHandle, st: St<'_>, kind: String, level: Option<String>) -> AppResult<LoginStart> {
    let client_id = st.config.read().twitch_client_id();
    if client_id.is_empty() {
        return Err(AppError::msg("No Twitch Client ID configured. Enter one under Settings → Advanced."));
    }
    let level = level.unwrap_or_else(|| st.config.read().settings.channel_manager_level.clone());
    let scopes: Vec<&'static str> = if kind == "bot" { BOT_SCOPES.to_vec() } else { auth::streamer_scopes(&level) };
    let dc = auth::start_device_flow(&st.http, &client_id, &scopes).await?;
    let login_id = new_id();
    let cancelled = Arc::new(AtomicBool::new(false));
    st.logins.lock().insert(login_id.clone(), cancelled.clone());

    let state = st.inner().clone();
    let lid = login_id.clone();
    let device_code = dc.device_code.clone();
    let expires = dc.expires_in;
    let mut interval = dc.interval.max(1);
    tokio::spawn(async move {
        let started = Instant::now();
        let emit = |status: &str, account: Option<TwitchAccount>, error: String| {
            let _ = app.emit(
                "hb:login",
                LoginEvent { login_id: lid.clone(), status: status.into(), account, error },
            );
        };
        loop {
            tokio::time::sleep(Duration::from_secs(interval)).await;
            if cancelled.load(Ordering::Relaxed) {
                emit("cancelled", None, String::new());
                break;
            }
            if started.elapsed().as_secs() > expires {
                emit("expired", None, String::new());
                break;
            }
            match auth::poll_device_token(&state.http, &client_id, &scopes, &device_code).await {
                Ok(PollResult::Pending) => {}
                Ok(PollResult::SlowDown) => interval += 2,
                Ok(PollResult::Done(tokens)) => {
                    match finish_login(&state, &client_id, tokens, &kind).await {
                        Ok(acc) => emit("done", Some(acc), String::new()),
                        Err(e) => emit("error", None, e.to_string()),
                    }
                    break;
                }
                Err(e) => {
                    emit("error", None, e.to_string());
                    break;
                }
            }
        }
        state.logins.lock().remove(&lid);
    });

    Ok(LoginStart {
        login_id,
        user_code: dc.user_code,
        verification_uri: dc.verification_uri,
        expires_in: dc.expires_in,
    })
}

async fn finish_login(st: &Arc<AppState>, client_id: &str, tokens: auth::TokenSet, kind: &str) -> AppResult<TwitchAccount> {
    let info = auth::validate(&st.http, &tokens.access_token)
        .await?
        .ok_or_else(|| AppError::msg("Twitch rejected the new token."))?;
    st.tokens.store(&info.user_id, &tokens)?;
    let helix = crate::twitch::helix::Helix {
        http: st.http.clone(),
        client_id: client_id.to_string(),
        tokens: st.tokens.clone(),
    };
    let user = helix
        .users_by_id(&info.user_id, &[info.user_id.as_str()])
        .await?
        .into_iter()
        .next()
        .unwrap_or_default();
    let account = TwitchAccount {
        id: info.user_id.clone(),
        login: if user.login.is_empty() { info.login.clone() } else { user.login },
        display_name: if user.display_name.is_empty() { info.login.clone() } else { user.display_name },
        profile_image_url: user.profile_image_url,
        scopes: info.scopes,
    };
    {
        let mut cfg = st.config.write();
        cfg.accounts.retain(|a| a.id != account.id);
        cfg.accounts.push(account.clone());
        let active = cfg.settings.active_channel_id.clone();
        if let Some(ch) = cfg.channels.iter_mut().find(|c| Some(&c.id) == active.as_ref()) {
            if kind == "bot" {
                ch.bot_account_id = Some(account.id.clone());
            } else {
                ch.streamer_account_id = Some(account.id.clone());
                if ch.name.is_empty() || ch.name == "My channel" {
                    ch.name = account.display_name.clone();
                }
            }
        }
        cfg.normalize();
    }
    st.save_config()?;
    Ok(account)
}

#[tauri::command]
pub fn twitch_login_cancel(st: St<'_>, login_id: String) {
    if let Some(flag) = st.logins.lock().get(&login_id) {
        flag.store(true, Ordering::Relaxed);
    }
}

#[tauri::command]
pub async fn twitch_remove_account(st: St<'_>, id: String) -> AppResult<Config> {
    let client_id = st.config.read().twitch_client_id();
    if let Some(t) = st.tokens.load(&id) {
        auth::revoke(&st.http, &client_id, &t.access_token).await;
    }
    st.tokens.remove(&id)?;
    {
        let mut cfg = st.config.write();
        cfg.accounts.retain(|a| a.id != id);
        cfg.normalize();
    }
    st.save_config()?;
    Ok(st.config.read().clone())
}

// ---------------------------------------------------------------------------
// Bot control
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn bot_start(app: AppHandle, st: St<'_>, channel_id: String, sandbox: bool) -> AppResult<()> {
    let state = st.inner().clone();
    st.bot.start(app, state, channel_id, sandbox).await
}

#[tauri::command]
pub async fn bot_stop(app: AppHandle, st: St<'_>) -> AppResult<()> {
    st.bot.stop(&app).await;
    Ok(())
}

#[tauri::command]
pub async fn bot_running(st: St<'_>) -> AppResult<Option<RunningInfo>> {
    Ok(st.bot.current().await.map(|(channel_id, sandbox)| RunningInfo { channel_id, sandbox }))
}

#[tauri::command]
pub async fn bot_control(st: St<'_>, action: String, payload: Option<Value>) -> AppResult<()> {
    let p = payload.unwrap_or(Value::Null);
    let s = |k: &str| p.get(k).and_then(|v| v.as_str()).unwrap_or_default().to_string();
    let c = match action.as_str() {
        "sayNow" => Control::SayNow,
        "approve" => Control::Approve(s("id")),
        "reject" => Control::Reject(s("id")),
        "edit" => Control::EditPending(s("id"), s("text")),
        "simulate" => Control::Simulate(serde_json::from_value::<SimMessage>(p.clone())?),
        "simulateEvent" => Control::SimulateEvent(s("kind"), s("user")),
        "modApply" => Control::ApplySuggestion(s("id")),
        "modDismiss" => Control::Dismiss(s("id")),
        "modUndo" => Control::Undo(s("id")),
        "modManual" => Control::Manual(serde_json::from_value::<ManualAction>(p.clone())?),
        "refresh" => Control::RefreshStream,
        other => return Err(AppError::msg(format!("Unknown action {other}"))),
    };
    st.bot.control(c).await
}

#[tauri::command]
pub fn get_modlog(st: St<'_>, channel_id: String, limit: Option<usize>) -> AppResult<Vec<ModLogEntry>> {
    let broadcaster = {
        let cfg = st.config.read();
        cfg.channel(&channel_id).and_then(|c| c.streamer_account_id.clone())
    };
    let Some(b) = broadcaster else {
        return Ok(Vec::new());
    };
    let path = st.data_dir.join("channels").join(b).join("modlog.jsonl");
    let raw: Vec<ModLogEntry> = storage::read_jsonl_tail(&path, 5000);
    // Later lines update earlier ones with the same id.
    let mut order: Vec<String> = Vec::new();
    let mut latest: HashMap<String, ModLogEntry> = HashMap::new();
    for e in raw {
        if !latest.contains_key(&e.id) {
            order.push(e.id.clone());
        }
        latest.insert(e.id.clone(), e);
    }
    let mut out: Vec<ModLogEntry> = order.into_iter().filter_map(|id| latest.remove(&id)).collect();
    out.reverse();
    out.truncate(limit.unwrap_or(500));
    Ok(out)
}

#[tauri::command]
pub fn get_usage(st: St<'_>) -> BTreeMap<String, BTreeMap<String, UsageStats>> {
    let u = st.usage.lock();
    u.days.iter().rev().take(31).map(|(k, v)| (k.clone(), v.clone())).collect()
}

/// Suggestions while the streamer types (":" emotes and emojis, "@" people, Tab).
#[tauri::command]
pub fn chat_suggest(st: St<'_>, query: String, mode: crate::chat::Mode, german: bool, limit: Option<usize>) -> Vec<crate::chat::Suggestion> {
    crate::chat::suggest_for(&st, &query, mode, german, limit.unwrap_or(10))
}

/// Sends a chat message as the streamer.
#[tauri::command]
pub async fn chat_send(st: St<'_>, text: String) -> AppResult<()> {
    crate::chat::send(&st, text).await
}

#[tauri::command]
pub fn get_emote_urls(st: St<'_>) -> HashMap<String, String> {
    (**st.emote_urls.read()).clone()
}

// ---------------------------------------------------------------------------
// Channel manager: allow-listed Helix calls with a connected account's token
// ---------------------------------------------------------------------------

const CHANNEL_API: &[(&str, &str)] = &[
    ("GET", "/users"),
    ("PUT", "/users"),
    ("GET", "/channels"),
    ("PATCH", "/channels"),
    ("GET", "/streams"),
    ("GET", "/search/categories"),
    ("GET", "/search/channels"),
    ("GET", "/streams/followed"),
    ("GET", "/chat/chatters"),
    ("GET", "/games"),
    ("GET", "/content_classification_labels"),
    ("POST", "/streams/markers"),
    ("GET", "/chat/settings"),
    ("PATCH", "/chat/settings"),
    ("POST", "/chat/announcements"),
    ("POST", "/chat/shoutouts"),
    ("GET", "/chat/color"),
    ("PUT", "/chat/color"),
    ("GET", "/chat/pins"),
    ("PUT", "/chat/pins"),
    ("DELETE", "/chat/pins"),
    ("POST", "/raids"),
    ("DELETE", "/raids"),
    ("POST", "/clips"),
    ("GET", "/polls"),
    ("POST", "/polls"),
    ("PATCH", "/polls"),
    ("GET", "/predictions"),
    ("POST", "/predictions"),
    ("PATCH", "/predictions"),
    ("GET", "/channel_points/custom_rewards"),
    ("POST", "/channel_points/custom_rewards"),
    ("PATCH", "/channel_points/custom_rewards"),
    ("DELETE", "/channel_points/custom_rewards"),
    ("GET", "/moderation/moderators"),
    ("POST", "/moderation/moderators"),
    ("DELETE", "/moderation/moderators"),
    ("GET", "/channels/vips"),
    ("POST", "/channels/vips"),
    ("DELETE", "/channels/vips"),
    ("GET", "/moderation/automod/settings"),
    ("PUT", "/moderation/automod/settings"),
    ("GET", "/moderation/blocked_terms"),
    ("POST", "/moderation/blocked_terms"),
    ("DELETE", "/moderation/blocked_terms"),
    ("GET", "/moderation/shield_mode"),
    ("PUT", "/moderation/shield_mode"),
    ("GET", "/moderation/banned"),
    ("DELETE", "/moderation/bans"),
    ("GET", "/moderation/unban_requests"),
    ("PATCH", "/moderation/unban_requests"),
    ("POST", "/moderation/suspicious_users"),
    ("DELETE", "/moderation/suspicious_users"),
    ("GET", "/hypetrain/status"),
    ("POST", "/channels/commercial"),
    ("GET", "/channels/ads"),
    ("POST", "/channels/ads/schedule/snooze"),
    ("GET", "/schedule"),
    ("POST", "/schedule/segment"),
    ("DELETE", "/schedule/segment"),
    ("PATCH", "/schedule/settings"),
    ("GET", "/channels/followers"),
    ("GET", "/subscriptions"),
    ("GET", "/channels/editors"),
];

#[tauri::command]
pub async fn twitch_api(
    st: St<'_>,
    account_id: String,
    method: String,
    path: String,
    query: Option<Vec<(String, String)>>,
    body: Option<Value>,
) -> AppResult<Value> {
    let method = method.to_uppercase();
    if !CHANNEL_API.iter().any(|(m, p)| *m == method && *p == path) {
        return Err(AppError::msg(format!("Twitch API call not allowed: {method} {path}")));
    }
    if st.config.read().account(&account_id).is_none() {
        return Err(AppError::msg("Unknown Twitch account — connect it first."));
    }
    let m = reqwest::Method::from_bytes(method.as_bytes()).map_err(|e| AppError::msg(e.to_string()))?;
    let query = query.unwrap_or_default();
    let q: Vec<(&str, &str)> = query.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    st.helix().raw(m, &path, &account_id, &q, body).await
}

#[tauri::command]
pub fn clear_research_cache(st: St<'_>) -> AppResult<()> {
    let dir = st.data_dir.join("research");
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    Ok(())
}

#[tauri::command]
pub fn open_data_dir(app: AppHandle, st: St<'_>) -> AppResult<()> {
    use tauri_plugin_opener::OpenerExt;
    std::fs::create_dir_all(&st.data_dir)?;
    app.opener()
        .open_path(st.data_dir.to_string_lossy().to_string(), None::<&str>)
        .map_err(|e| AppError::msg(e.to_string()))
}

// ---------------------------------------------------------------- shell & overlay

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentFeed {
    pub items: Vec<crate::bot::FeedItem>,
    pub translations: Vec<Value>,
}

/// Feed entries of the running session, for a window reopened from the tray.
#[tauri::command]
pub fn get_recent_feed(st: St<'_>) -> RecentFeed {
    RecentFeed { items: st.overlay.recent(), translations: st.overlay.recent_translations() }
}

#[tauri::command]
pub fn overlay_status(st: St<'_>) -> crate::overlay::Status {
    st.overlay_server.status()
}

/// Performance mode: close the interface now and keep running in the tray.
#[tauri::command]
pub fn app_to_tray(app: AppHandle) {
    crate::shell::to_tray(&app);
}
