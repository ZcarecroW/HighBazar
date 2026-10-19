//! Emotes from Twitch, BetterTTV, FrankerFaceZ and 7TV (names + image URLs).

use std::collections::HashMap;

use serde::Serialize;
use serde_json::Value;

use super::helix::Helix;
use crate::config::EmoteSettings;

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmoteCatalog {
    pub twitch: Vec<String>,
    pub bttv: Vec<String>,
    pub ffz: Vec<String>,
    pub seventv: Vec<String>,
    /// Emote name → image URL (1x) for showing emotes in the UI.
    #[serde(skip)]
    pub urls: HashMap<String, String>,
}

impl EmoteCatalog {
    pub fn all(&self) -> impl Iterator<Item = &String> {
        self.twitch.iter().chain(&self.bttv).chain(&self.ffz).chain(&self.seventv)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.all().any(|e| e == name)
    }

    pub fn total(&self) -> usize {
        self.twitch.len() + self.bttv.len() + self.ffz.len() + self.seventv.len()
    }

    /// Emotes the bot may use, honoring the enabled sources.
    pub fn allowed(&self, s: &EmoteSettings) -> Vec<&String> {
        let mut out: Vec<&String> = Vec::new();
        if s.twitch {
            out.extend(&self.twitch);
        }
        if s.bttv {
            out.extend(&self.bttv);
        }
        if s.ffz {
            out.extend(&self.ffz);
        }
        if s.seventv {
            out.extend(&self.seventv);
        }
        out
    }
}

/// Image of a native Twitch emote (`default` = animated if available).
pub fn twitch_emote_url(id: &str) -> String {
    format!("https://static-cdn.jtvnw.net/emoticons/v2/{id}/default/dark/1.0")
}

async fn get_json(http: &reqwest::Client, url: &str) -> Option<Value> {
    let resp = http.get(url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    resp.json().await.ok()
}

/// (name, url) pairs from a JSON array of emote objects.
fn entries(v: Option<&Value>, name_key: &str, url: impl Fn(&Value) -> Option<String>) -> Vec<(String, String)> {
    v.and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|e| Some((e[name_key].as_str()?.to_string(), url(e).unwrap_or_default())))
                .collect()
        })
        .unwrap_or_default()
}

fn bttv_url(e: &Value) -> Option<String> {
    Some(format!("https://cdn.betterttv.net/emote/{}/1x.webp", e["id"].as_str()?))
}

fn ffz_url(e: &Value) -> Option<String> {
    e["images"]["1x"].as_str().map(str::to_string)
}

fn seventv_url(e: &Value) -> Option<String> {
    Some(format!("https://cdn.7tv.app/emote/{}/1x.webp", e["id"].as_str()?))
}

pub async fn fetch(helix: &Helix, broadcaster_id: &str) -> EmoteCatalog {
    let http = &helix.http;
    let bttv_c_url = format!("https://api.betterttv.net/3/cached/users/twitch/{broadcaster_id}");
    let ffz_c_url = format!("https://api.betterttv.net/3/cached/frankerfacez/users/twitch/{broadcaster_id}");
    let stv_c_url = format!("https://7tv.io/v3/users/twitch/{broadcaster_id}");
    let (tw_global, tw_channel, bttv_g, bttv_c, ffz_g, ffz_c, stv_g, stv_c) = tokio::join!(
        helix.global_emotes(broadcaster_id),
        helix.channel_emotes(broadcaster_id, broadcaster_id),
        get_json(http, "https://api.betterttv.net/3/cached/emotes/global"),
        get_json(http, &bttv_c_url),
        get_json(http, "https://api.betterttv.net/3/cached/frankerfacez/emotes/global"),
        get_json(http, &ffz_c_url),
        get_json(http, "https://7tv.io/v3/emote-sets/global"),
        get_json(http, &stv_c_url),
    );

    let mut twitch: Vec<(String, String)> = Vec::new();
    // Channel emotes first: they matter most for the community.
    for e in tw_channel.unwrap_or_default().into_iter().chain(tw_global.unwrap_or_default()) {
        twitch.push((e.name, twitch_emote_url(&e.id)));
    }
    let mut bttv = Vec::new();
    if let Some(c) = &bttv_c {
        bttv.extend(entries(c.get("channelEmotes"), "code", bttv_url));
        bttv.extend(entries(c.get("sharedEmotes"), "code", bttv_url));
    }
    bttv.extend(entries(bttv_g.as_ref(), "code", bttv_url));
    let mut ffz = entries(ffz_c.as_ref(), "code", ffz_url);
    ffz.extend(entries(ffz_g.as_ref(), "code", ffz_url));
    let mut stv = Vec::new();
    if let Some(c) = &stv_c {
        stv.extend(entries(c["emote_set"].get("emotes"), "name", seventv_url));
    }
    if let Some(g) = &stv_g {
        stv.extend(entries(g.get("emotes"), "name", seventv_url));
    }

    let mut cat = EmoteCatalog::default();
    for (list, out) in [(twitch, &mut cat.twitch), (bttv, &mut cat.bttv), (ffz, &mut cat.ffz), (stv, &mut cat.seventv)] {
        let mut seen = std::collections::HashSet::new();
        for (name, url) in list {
            if name.is_empty() || !seen.insert(name.clone()) {
                continue;
            }
            // Channel emotes come first, so they win over globals with the same name.
            if !url.is_empty() {
                cat.urls.entry(name.clone()).or_insert(url);
            }
            out.push(name);
        }
    }
    cat
}
