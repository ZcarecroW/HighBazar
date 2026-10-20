//! Chat page for the Steam overlay browser (and any browser source).
//!
//! A tiny HTTP server on 127.0.0.1 serves a self-contained page and streams
//! new chat lines as Server-Sent Events. It lives in the Rust core, so it keeps
//! working while the window is closed in performance mode. Nothing runs when
//! the overlay is switched off.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use parking_lot::Mutex;
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, watch};

use crate::bot::FeedItem;
use crate::config::AppSettings;
use crate::state::AppState;

const PAGE: &str = include_str!("../../overlay/index.html");
const SCRIPT: &str = include_str!("../../overlay/overlay.js");
const STYLE: &str = include_str!("../../overlay/overlay.css");

/// Feed entries kept for the overlay and for a window that is reopened from the tray.
const RECENT: usize = 200;
const MAX_CLIENTS: usize = 6;

/// Live feed fan-out. Everything is cheap when nobody listens.
pub struct Hub {
    tx: broadcast::Sender<Arc<str>>,
    recent: Mutex<VecDeque<FeedItem>>,
    translations: Mutex<VecDeque<(String, Value)>>,
}

impl Default for Hub {
    fn default() -> Self {
        Self {
            tx: broadcast::channel(256).0,
            recent: Mutex::new(VecDeque::with_capacity(RECENT)),
            translations: Mutex::new(VecDeque::new()),
        }
    }
}

fn shown(kind: &str) -> bool {
    matches!(kind, "chat" | "bot" | "event")
}

impl Hub {
    fn send(&self, event: &str, data: &Value) {
        if self.tx.receiver_count() > 0 {
            let _ = self.tx.send(format!("event: {event}\ndata: {data}\n\n").into());
        }
    }

    pub fn push(&self, item: &FeedItem) {
        {
            let mut r = self.recent.lock();
            if r.len() >= RECENT {
                r.pop_front();
            }
            r.push_back(item.clone());
        }
        if shown(&item.kind) && self.tx.receiver_count() > 0 {
            self.send("msg", &line(item));
        }
    }

    pub fn translation(&self, message_id: &str, lang: &str, text: &str) {
        let v = json!({ "messageId": message_id, "lang": lang, "text": text });
        {
            let mut t = self.translations.lock();
            if t.len() >= RECENT {
                t.pop_front();
            }
            t.push_back((message_id.to_string(), v.clone()));
        }
        self.send("tr", &v);
    }

    /// A moderation action removed a message (or all messages of a user).
    pub fn removed(&self, message_id: &str, user_id: &str, whole_user: bool) {
        self.recent.lock().retain(|i| {
            !(i.kind == "chat" && (i.message_id == message_id || (whole_user && i.user_id == user_id)))
        });
        self.send("remove", &json!({ "messageId": message_id, "userId": if whole_user { user_id } else { "" } }));
    }

    pub fn emotes(&self, urls: &HashMap<String, String>) {
        self.send("emotes", &json!(urls));
    }

    pub fn settings(&self, s: &AppSettings) {
        self.send("settings", &page_settings(s));
    }

    pub fn clear(&self) {
        self.recent.lock().clear();
        self.translations.lock().clear();
        self.send("clear", &json!({}));
    }

    /// Last feed entries (all kinds) for a window that was reopened.
    pub fn recent(&self) -> Vec<FeedItem> {
        self.recent.lock().iter().cloned().collect()
    }

    pub fn recent_translations(&self) -> Vec<Value> {
        self.translations.lock().iter().map(|(_, v)| v.clone()).collect()
    }

    fn snapshot(&self, st: &AppState) -> Value {
        let settings = page_settings(&st.config.read().settings);
        let items: Vec<Value> = self.recent.lock().iter().filter(|i| shown(&i.kind)).map(line).collect();
        let emotes = st.emote_urls.read().clone();
        json!({ "settings": settings, "emotes": *emotes, "items": items, "translations": self.recent_translations() })
    }
}

/// What the page needs of a feed entry.
fn line(i: &FeedItem) -> Value {
    json!({
        "id": i.id,
        "ts": i.ts,
        "kind": i.kind,
        "user": i.user,
        "userId": i.user_id,
        "messageId": i.message_id,
        "color": i.color,
        "text": i.text,
        "emotes": i.meta.get("emotes").cloned().unwrap_or(Value::Null),
        "key": i.meta.get("key").cloned().unwrap_or(Value::Null),
        "params": i.meta.get("params").cloned().unwrap_or(Value::Null),
        "shared": i.meta.get("shared").cloned().unwrap_or(Value::Null),
    })
}

fn page_settings(s: &AppSettings) -> Value {
    let o = &s.overlay;
    json!({
        "showBot": o.show_bot,
        "showEvents": o.show_events,
        "showTranslations": o.show_translations,
        "hideAfterSecs": o.hide_after_secs,
        "background": (f64::from(o.background.clamp(0.0, 1.0)) * 100.0).round() / 100.0,
        "showEmotes": s.show_emotes,
        "chatZoom": s.chat_zoom.clamp(0.6, 2.5),
        "chatScroll": s.chat_scroll,
        "language": s.language,
        "chatInput": o.chat_input,
    })
}

// ---------------------------------------------------------------------------
// Server
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct Server {
    running: Mutex<Option<Running>>,
    error: Mutex<String>,
    clients: Arc<AtomicUsize>,
}

struct Running {
    port: u16,
    key: String,
    stop: watch::Sender<bool>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub running: bool,
    pub url: String,
    pub error: String,
    pub clients: usize,
}

impl Server {
    pub fn status(&self) -> Status {
        let r = self.running.lock();
        Status {
            running: r.is_some(),
            url: r.as_ref().map(|r| format!("http://127.0.0.1:{}/?k={}", r.port, r.key)).unwrap_or_default(),
            error: self.error.lock().clone(),
            clients: self.clients.load(Ordering::Relaxed),
        }
    }

    fn stop(&self) {
        if let Some(r) = self.running.lock().take() {
            let _ = r.stop.send(true);
        }
    }
}

/// Starts, restarts or stops the server to match the settings.
pub fn sync(st: &Arc<AppState>) {
    let o = st.config.read().settings.overlay.clone();
    let srv = &st.overlay_server;
    let wanted = o.enabled && o.port >= 1024 && o.key.len() >= 16;
    {
        let r = srv.running.lock();
        if let Some(r) = r.as_ref()
            && wanted
            && r.port == o.port
            && r.key == o.key
        {
            return;
        }
    }
    srv.stop();
    srv.error.lock().clear();
    if !wanted {
        return;
    }
    // Bind synchronously so a busy port is reported right away.
    let std_listener = match std::net::TcpListener::bind(("127.0.0.1", o.port)) {
        Ok(l) => l,
        Err(e) => {
            *srv.error.lock() = format!("Port {} is not available: {e}", o.port);
            return;
        }
    };
    let _ = std_listener.set_nonblocking(true);
    let (stop_tx, stop_rx) = watch::channel(false);
    *srv.running.lock() = Some(Running { port: o.port, key: o.key.clone(), stop: stop_tx });
    let st = st.clone();
    tauri::async_runtime::spawn(async move {
        let listener = match TcpListener::from_std(std_listener) {
            Ok(l) => l,
            Err(e) => {
                *st.overlay_server.error.lock() = e.to_string();
                return;
            }
        };
        accept_loop(listener, st, o.port, o.key, stop_rx).await;
    });
}

async fn accept_loop(listener: TcpListener, st: Arc<AppState>, port: u16, key: String, mut stop: watch::Receiver<bool>) {
    loop {
        tokio::select! {
            r = listener.accept() => {
                let Ok((stream, _)) = r else { continue };
                let st = st.clone();
                let key = key.clone();
                let stop = stop.clone();
                tokio::spawn(async move {
                    let _ = handle(stream, st, port, &key, stop).await;
                });
            }
            _ = stop.changed() => return,
        }
    }
}

struct Request {
    method: String,
    path: String,
    query: HashMap<String, String>,
    host: String,
    /// Origin header of a POST (empty when the browser sent none).
    origin: String,
    content_type: String,
    body: Vec<u8>,
}

/// Messages are short; anything bigger is not from the overlay page.
const MAX_BODY: usize = 4096;

async fn read_request(stream: &mut TcpStream) -> Option<Request> {
    let mut buf = Vec::with_capacity(1024);
    let mut chunk = [0u8; 1024];
    let read = async {
        loop {
            let n = stream.read(&mut chunk).await.ok()?;
            if n == 0 {
                return None;
            }
            buf.extend_from_slice(&chunk[..n]);
            if buf.windows(4).any(|w| w == b"\r\n\r\n") {
                return Some(());
            }
            if buf.len() > 8192 {
                return None;
            }
        }
    };
    tokio::time::timeout(Duration::from_secs(5), read).await.ok()??;
    let head_end = buf.windows(4).position(|w| w == b"\r\n\r\n")? + 4;
    let text = String::from_utf8_lossy(&buf[..head_end]).into_owned();
    let mut lines = text.split("\r\n");
    let mut first = lines.next()?.split(' ');
    let method = first.next()?.to_string();
    if method != "GET" && method != "POST" {
        return None;
    }
    let target = first.next()?;
    let (path, qs) = target.split_once('?').unwrap_or((target, ""));
    let query = url::form_urlencoded::parse(qs.as_bytes()).into_owned().collect();
    let headers: Vec<(String, String)> = lines
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.trim().to_lowercase(), v.trim().to_string()))
        .collect();
    let header = |name: &str| headers.iter().find(|(k, _)| k == name).map(|(_, v)| v.clone()).unwrap_or_default();
    let mut body = buf[head_end..].to_vec();
    if method == "POST" {
        let len: usize = header("content-length").parse().ok()?;
        if len > MAX_BODY {
            return None;
        }
        let rest = async {
            while body.len() < len {
                let n = stream.read(&mut chunk).await.ok()?;
                if n == 0 {
                    return None;
                }
                body.extend_from_slice(&chunk[..n]);
            }
            Some(())
        };
        tokio::time::timeout(Duration::from_secs(5), rest).await.ok()??;
        body.truncate(len);
    }
    Some(Request {
        method,
        path: path.to_string(),
        query,
        host: header("host").to_lowercase(),
        origin: header("origin").to_lowercase(),
        content_type: header("content-type").to_lowercase(),
        body,
    })
}

async fn respond(stream: &mut TcpStream, status: &str, ctype: &str, body: &str) -> std::io::Result<()> {
    let csp = "default-src 'none'; img-src https://static-cdn.jtvnw.net https://cdn.betterttv.net https://cdn.frankerfacez.com https://cdn.7tv.app; style-src 'self'; script-src 'self'; connect-src 'self'";
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nContent-Security-Policy: {csp}\r\nX-Content-Type-Options: nosniff\r\nReferrer-Policy: no-referrer\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(head.as_bytes()).await?;
    stream.write_all(body.as_bytes()).await?;
    stream.shutdown().await
}

async fn handle(mut stream: TcpStream, st: Arc<AppState>, port: u16, key: &str, mut stop: watch::Receiver<bool>) -> std::io::Result<()> {
    let Some(req) = read_request(&mut stream).await else {
        return respond(&mut stream, "400 Bad Request", "text/plain", "Bad request").await;
    };
    // Only the local machine, and no DNS-rebinding tricks from websites.
    if req.host != format!("127.0.0.1:{port}") && req.host != format!("localhost:{port}") {
        return respond(&mut stream, "403 Forbidden", "text/plain", "Forbidden").await;
    }
    // The page, the event stream and the chat input need the key; the static script and style don't.
    let needs_key = matches!(req.path.as_str(), "/" | "/events" | "/suggest" | "/send");
    if needs_key && req.query.get("k").map(String::as_str) != Some(key) {
        return respond(&mut stream, "403 Forbidden", "text/plain", "This overlay link is outdated. Copy the current one from HighBazar.").await;
    }
    if req.method == "POST" && req.path != "/send" {
        return respond(&mut stream, "405 Method Not Allowed", "text/plain", "Method not allowed").await;
    }
    match req.path.as_str() {
        "/suggest" | "/send" if !st.config.read().settings.overlay.chat_input => {
            respond(&mut stream, "403 Forbidden", "application/json", r#"{"error":"The chat input of the overlay is switched off in HighBazar."}"#).await
        }
        "/suggest" => {
            let q = req.query.get("q").map(String::as_str).unwrap_or("");
            let mode = match req.query.get("m").map(String::as_str) {
                Some("mention") => crate::chat::Mode::Mention,
                Some("tab") => crate::chat::Mode::Tab,
                Some("browse") => crate::chat::Mode::Browse,
                _ => crate::chat::Mode::Colon,
            };
            let german = req.query.get("lang").is_some_and(|l| l.starts_with("de"));
            let limit = if mode == crate::chat::Mode::Browse { 200 } else { 10 };
            let list = crate::chat::suggest_for(&st, q, mode, german, limit);
            respond(&mut stream, "200 OK", "application/json", &serde_json::to_string(&list).unwrap_or_default()).await
        }
        "/send" => {
            // Only the overlay page itself: same origin and a JSON body, which a
            // form on another website cannot send.
            let own = [format!("http://127.0.0.1:{port}"), format!("http://localhost:{port}")];
            if req.method != "POST" || !req.content_type.starts_with("application/json") || (!req.origin.is_empty() && !own.contains(&req.origin)) {
                return respond(&mut stream, "403 Forbidden", "application/json", r#"{"error":"Forbidden"}"#).await;
            }
            let text = serde_json::from_slice::<Value>(&req.body).ok().and_then(|v| v["text"].as_str().map(str::to_string)).unwrap_or_default();
            let body = match crate::chat::send(&st, text).await {
                Ok(()) => json!({ "ok": true }),
                Err(e) => json!({ "error": e.to_string() }),
            };
            respond(&mut stream, "200 OK", "application/json", &body.to_string()).await
        }
        "/" => respond(&mut stream, "200 OK", "text/html; charset=utf-8", PAGE).await,
        "/overlay.js" => respond(&mut stream, "200 OK", "text/javascript; charset=utf-8", SCRIPT).await,
        "/overlay.css" => respond(&mut stream, "200 OK", "text/css; charset=utf-8", STYLE).await,
        "/events" => {
            let clients = st.overlay_server.clients.clone();
            if clients.load(Ordering::Relaxed) >= MAX_CLIENTS {
                return respond(&mut stream, "503 Service Unavailable", "text/plain", "Too many overlay windows").await;
            }
            clients.fetch_add(1, Ordering::Relaxed);
            let r = stream_events(&mut stream, &st, &mut stop).await;
            clients.fetch_sub(1, Ordering::Relaxed);
            r
        }
        _ => respond(&mut stream, "404 Not Found", "text/plain", "Not found").await,
    }
}

async fn stream_events(stream: &mut TcpStream, st: &AppState, stop: &mut watch::Receiver<bool>) -> std::io::Result<()> {
    let (mut rd, mut stream) = stream.split();
    let mut probe = [0u8; 64];
    let head = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nConnection: keep-alive\r\n\r\nretry: 2000\n\n";
    stream.write_all(head.as_bytes()).await?;
    let mut rx = st.overlay.tx.subscribe();
    let state = st.overlay.snapshot(st);
    stream.write_all(format!("event: state\ndata: {state}\n\n").as_bytes()).await?;
    let mut ping = tokio::time::interval(Duration::from_secs(20));
    ping.tick().await;
    loop {
        tokio::select! {
            m = rx.recv() => match m {
                Ok(m) => stream.write_all(m.as_bytes()).await?,
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    let state = st.overlay.snapshot(st);
                    stream.write_all(format!("event: state\ndata: {state}\n\n").as_bytes()).await?;
                }
                Err(_) => return Ok(()),
            },
            _ = ping.tick() => stream.write_all(b": ping\n\n").await?,
            // The browser closed the page: free the slot right away.
            r = rd.read(&mut probe) => if matches!(r, Ok(0) | Err(_)) { return Ok(()) },
            _ = stop.changed() => return Ok(()),
        }
    }
}
