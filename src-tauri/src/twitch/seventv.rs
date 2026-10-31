//! 7TV live updates: emotes the streamer or an editor adds, removes or renames
//! in the channel's 7TV set show up right away instead of with the next
//! refresh – like in the 7TV extension. 7TV's EventAPI over Server-Sent
//! Events: one long GET request, no extra dependency.

use std::time::Duration;

use serde_json::Value;
use tokio::sync::watch;

const EVENTS: &str = "https://events.7tv.io/v3";

/// Why listening stopped.
#[derive(Debug, PartialEq)]
pub enum Stop {
    /// The bot stops.
    Shutdown,
    /// The channel switched to another emote set: load it and listen again.
    SetChanged,
    /// Connection lost or closed by 7TV: try again after a pause.
    Lost,
}

/// One subscription request for the set's changes and the account (set switches).
pub fn url(user_id: &str, set_id: &str) -> String {
    let mut subs = format!("emote_set.update<object_id={set_id}>");
    if !user_id.is_empty() {
        subs.push_str(&format!(",user.update<object_id={user_id}>"));
    }
    format!("{EVENTS}@{}", url::form_urlencoded::byte_serialize(subs.as_bytes()).collect::<String>())
}

/// Listens until the bot stops, the connection breaks or the active set
/// changes. `on_set_update` gets the body of every `emote_set.update`.
pub async fn listen(http: &reqwest::Client, url: &str, shutdown: &mut watch::Receiver<bool>, mut on_set_update: impl FnMut(&Value)) -> Stop {
    let request = http
        .get(url)
        .header("Accept", "text/event-stream")
        // The stream stays open for hours; heartbeats show it is alive.
        .timeout(Duration::from_secs(12 * 3600))
        .send();
    let mut resp = tokio::select! {
        r = request => match r {
            Ok(r) if r.status().is_success() => r,
            Ok(r) => {
                log::info!("7TV events: HTTP {}", r.status());
                return Stop::Lost;
            }
            Err(e) => {
                log::info!("7TV events: {e}");
                return Stop::Lost;
            }
        },
        _ = shutdown.changed() => return Stop::Shutdown,
    };
    let mut heartbeat = Duration::from_secs(60);
    let mut buf: Vec<u8> = Vec::new();
    loop {
        let chunk = tokio::select! {
            c = tokio::time::timeout(heartbeat * 3, resp.chunk()) => c,
            _ = shutdown.changed() => return Stop::Shutdown,
        };
        match chunk {
            Ok(Ok(Some(bytes))) => buf.extend_from_slice(&bytes),
            Ok(Ok(None)) => return Stop::Lost,
            Ok(Err(e)) => {
                log::info!("7TV events: {e}");
                return Stop::Lost;
            }
            Err(_) => {
                log::info!("7TV events: no heartbeat");
                return Stop::Lost;
            }
        }
        while let Some((event, data, used)) = next_event(&buf) {
            buf.drain(..used);
            let Ok(v) = serde_json::from_str::<Value>(&data) else { continue };
            match event.as_str() {
                "hello" => {
                    if let Some(ms) = v["heartbeat_interval"].as_u64() {
                        heartbeat = Duration::from_millis(ms.clamp(5_000, 120_000));
                    }
                }
                "dispatch" => match v["type"].as_str() {
                    Some("emote_set.update") => on_set_update(&v["body"]),
                    Some("user.update") if switches_set(&v["body"]) => return Stop::SetChanged,
                    _ => {}
                },
                "reconnect" | "end_of_stream" => return Stop::Lost,
                _ => {}
            }
        }
    }
}

/// The next complete event in the buffer: (name, data, bytes used).
fn next_event(buf: &[u8]) -> Option<(String, String, usize)> {
    let end = buf.windows(2).position(|w| w == b"\n\n").map(|p| (p, p + 2));
    let end_crlf = buf.windows(4).position(|w| w == b"\r\n\r\n").map(|p| (p, p + 4));
    let (block_end, used) = match (end, end_crlf) {
        (Some(a), Some(b)) => a.min(b),
        (a, b) => a.or(b)?,
    };
    let block = String::from_utf8_lossy(&buf[..block_end]);
    let mut event = String::from("message");
    let mut data = String::new();
    for line in block.lines() {
        if let Some(v) = line.strip_prefix("event:") {
            event = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("data:") {
            if !data.is_empty() {
                data.push('\n');
            }
            data.push_str(v.strip_prefix(' ').unwrap_or(v));
        }
    }
    Some((event, data, used))
}

/// A `user.update` that changes the active emote set of a connection.
fn switches_set(body: &Value) -> bool {
    body["updated"].as_array().into_iter().flatten().filter(|c| c["key"] == "connections").any(|c| {
        c["value"].as_array().into_iter().flatten().any(|f| f["key"] == "emote_set" || f["key"] == "emote_set_id")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn builds_the_subscription_url() {
        assert_eq!(
            url("u1", "s1"),
            "https://events.7tv.io/v3@emote_set.update%3Cobject_id%3Ds1%3E%2Cuser.update%3Cobject_id%3Du1%3E"
        );
        assert_eq!(url("", "s1"), "https://events.7tv.io/v3@emote_set.update%3Cobject_id%3Ds1%3E");
    }

    #[test]
    fn reads_server_sent_events() {
        let raw = b"event: hello\ndata: {\"heartbeat_interval\":46979}\nid: 0\n\nevent: dispatch\r\ndata: {\"type\":\"x\"}\r\n\r\nevent: heart";
        let (e, d, used) = next_event(raw).unwrap();
        assert_eq!((e.as_str(), d.as_str()), ("hello", "{\"heartbeat_interval\":46979}"));
        let rest = &raw[used..];
        let (e, d, used) = next_event(rest).unwrap();
        assert_eq!((e.as_str(), d.as_str()), ("dispatch", "{\"type\":\"x\"}"));
        // An incomplete event waits for more data.
        assert!(next_event(&rest[used..]).is_none());
    }

    #[test]
    fn notices_set_switches() {
        let switch = json!({"id": "u1", "updated": [{"key": "connections", "index": 0, "nested": true, "value": [
            {"key": "emote_set", "old_value": {"id": "a"}, "value": {"id": "b"}},
            {"key": "emote_set_id", "old_value": "a", "value": "b"}
        ]}]});
        assert!(switches_set(&switch));
        let other = json!({"id": "u1", "updated": [{"key": "style", "value": {}}]});
        assert!(!switches_set(&other));
    }
}
