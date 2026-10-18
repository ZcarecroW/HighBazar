//! Emote names from Twitch, BetterTTV, FrankerFaceZ and 7TV.

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

async fn get_json(http: &reqwest::Client, url: &str) -> Option<Value> {
    let resp = http.get(url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    resp.json().await.ok()
}

fn names(v: Option<&Value>, key: &str) -> Vec<String> {
    v.and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|e| e[key].as_str().map(str::to_string)).collect())
        .unwrap_or_default()
}

pub async fn fetch(helix: &Helix, broadcaster_id: &str) -> EmoteCatalog {
    let http = &helix.http;
    let bttv_url = format!("https://api.betterttv.net/3/cached/users/twitch/{broadcaster_id}");
    let ffz_url = format!("https://api.betterttv.net/3/cached/frankerfacez/users/twitch/{broadcaster_id}");
    let stv_url = format!("https://7tv.io/v3/users/twitch/{broadcaster_id}");
    let (tw_global, tw_channel, bttv_g, bttv_c, ffz_g, ffz_c, stv_g, stv_c) = tokio::join!(
        helix.global_emotes(broadcaster_id),
        helix.channel_emotes(broadcaster_id, broadcaster_id),
        get_json(http, "https://api.betterttv.net/3/cached/emotes/global"),
        get_json(http, &bttv_url),
        get_json(http, "https://api.betterttv.net/3/cached/frankerfacez/emotes/global"),
        get_json(http, &ffz_url),
        get_json(http, "https://7tv.io/v3/emote-sets/global"),
        get_json(http, &stv_url),
    );

    let mut cat = EmoteCatalog::default();
    // Channel emotes first: they matter most for the community.
    cat.twitch.extend(tw_channel.unwrap_or_default());
    cat.twitch.extend(tw_global.unwrap_or_default());

    if let Some(c) = &bttv_c {
        cat.bttv.extend(names(c.get("channelEmotes"), "code"));
        cat.bttv.extend(names(c.get("sharedEmotes"), "code"));
    }
    cat.bttv.extend(names(bttv_g.as_ref(), "code"));

    cat.ffz.extend(names(ffz_c.as_ref(), "code"));
    cat.ffz.extend(names(ffz_g.as_ref(), "code"));

    if let Some(c) = &stv_c {
        cat.seventv.extend(names(c["emote_set"].get("emotes"), "name"));
    }
    if let Some(g) = &stv_g {
        cat.seventv.extend(names(g.get("emotes"), "name"));
    }

    for list in [&mut cat.twitch, &mut cat.bttv, &mut cat.ffz, &mut cat.seventv] {
        let mut seen = std::collections::HashSet::new();
        list.retain(|e| !e.is_empty() && seen.insert(e.clone()));
    }
    cat
}
