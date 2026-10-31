//! Emotes from Twitch, BetterTTV, FrankerFaceZ and 7TV: names, image URLs and
//! which one wins when several providers have an emote with the same name.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;

use super::helix::Helix;
use crate::chat::{EmoteInfo, Scope};
use crate::config::EmoteSettings;

const BTTV_GLOBAL: &str = "https://api.betterttv.net/3/cached/emotes/global";
const FFZ_GLOBAL: &str = "https://api.betterttv.net/3/cached/frankerfacez/emotes/global";
const STV_GLOBAL: &str = "https://7tv.io/v3/emote-sets/global";

/// BTTV draws these global emotes over the emote before them (hats, masks);
/// the list is built into the BTTV extension, the API does not mark them.
const BTTV_ZERO_WIDTH: &[&str] = &["SoSnowy", "IceCold", "SantaHat", "TopHat", "ReinDeer", "CandyCane", "cvMask", "cvHazmat"];

/// One emote as a provider lists it.
#[derive(Debug, Clone, PartialEq)]
pub struct Emote {
    /// The provider's id (7TV live updates remove and rename by id).
    pub id: String,
    pub name: String,
    pub url: String,
    /// Drawn over the emote before it (7TV zero-width, BTTV hats, FFZ modifiers).
    pub zero_width: bool,
}

/// The lists a channel's emotes come from. `None` = not loaded yet; a request
/// that fails keeps the last good list, so a hiccup at one provider does not
/// make its emotes vanish until the next refresh.
#[derive(Debug, Clone, Default)]
pub struct Sources {
    pub twitch_channel: Option<Vec<Emote>>,
    pub twitch_global: Option<Vec<Emote>>,
    /// Twitch emotes the streamer may use anywhere (subscriptions, Prime, Turbo…).
    pub twitch_user: Option<Vec<Emote>>,
    pub stv_channel: Option<Vec<Emote>>,
    pub stv_global: Option<Vec<Emote>>,
    pub bttv_channel: Option<Vec<Emote>>,
    pub bttv_global: Option<Vec<Emote>>,
    pub ffz_channel: Option<Vec<Emote>>,
    pub ffz_global: Option<Vec<Emote>>,
    /// 7TV ids of the channel's account and active emote set (for live updates).
    pub stv_user_id: String,
    pub stv_set_id: String,
    /// The streamer's own emote list needs `user:read:emotes` (logins before 1.6.0 lack it).
    pub user_emotes_missing: bool,
}

/// A change in the channel's 7TV emote set (for the live feed).
#[derive(Debug, Clone, PartialEq)]
pub enum StvChange {
    Added(String),
    Removed(String),
    Renamed(String, String),
}

impl Sources {
    /// Takes every list of `new` that loaded and keeps the others.
    pub fn merge(&mut self, new: Sources) {
        macro_rules! take {
            ($($f:ident),*) => {$( if new.$f.is_some() { self.$f = new.$f; } )*};
        }
        if new.stv_channel.is_some() {
            self.stv_user_id = new.stv_user_id;
            self.stv_set_id = new.stv_set_id;
        }
        take!(twitch_channel, twitch_global, twitch_user, stv_channel, stv_global, bttv_channel, bttv_global, ffz_channel, ffz_global);
        self.user_emotes_missing = new.user_emotes_missing;
    }

    /// Order in which an emote name is claimed: Twitch draws its own emotes for
    /// everyone, then a channel's emotes beat global ones (as in the browser
    /// extensions and Chatterino).
    fn ranked(&self) -> [(&Option<Vec<Emote>>, &'static str, Scope); 9] {
        [
            (&self.twitch_channel, "twitch", Scope::Channel),
            (&self.twitch_global, "twitch", Scope::Global),
            (&self.twitch_user, "twitch", Scope::Sub),
            (&self.stv_channel, "7tv", Scope::Channel),
            (&self.bttv_channel, "bttv", Scope::Channel),
            (&self.ffz_channel, "ffz", Scope::Channel),
            (&self.stv_global, "7tv", Scope::Global),
            (&self.bttv_global, "bttv", Scope::Global),
            (&self.ffz_global, "ffz", Scope::Global),
        ]
    }

    /// Applies a 7TV `emote_set.update` to the channel's 7TV list.
    pub fn apply_7tv(&mut self, body: &Value) -> Vec<StvChange> {
        let list = self.stv_channel.get_or_insert_with(Vec::new);
        let emotes = |k: &str| body[k].as_array().into_iter().flatten().filter(|c| c["key"] == "emotes");
        let same = |e: &Emote, old: &Value| old["id"].as_str() == Some(e.id.as_str()) && old["name"].as_str().is_none_or(|n| n == e.name);
        let mut out = Vec::new();
        for c in emotes("pulled") {
            let old = &c["old_value"];
            if let Some(i) = list.iter().position(|e| same(e, old)) {
                out.push(StvChange::Removed(list.remove(i).name));
            }
        }
        for c in emotes("updated") {
            let (old, Some(new)) = (&c["old_value"], seventv_emote(&c["value"])) else { continue };
            if let Some(e) = list.iter_mut().find(|e| same(e, old)) {
                if e.name != new.name {
                    out.push(StvChange::Renamed(e.name.clone(), new.name.clone()));
                }
                *e = new;
            }
        }
        for c in emotes("pushed") {
            if let Some(e) = seventv_emote(&c["value"])
                && !list.iter().any(|x| x.id == e.id && x.name == e.name)
            {
                out.push(StvChange::Added(e.name.clone()));
                list.push(e);
            }
        }
        out
    }
}

/// What the UI needs to draw emotes: the image of the emote that wins each
/// name, and which of them are zero-width.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmoteMap {
    pub urls: HashMap<String, String>,
    pub zero_width: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct EmoteCatalog {
    /// Names per provider (channel emotes first), for the emotes the bot may use.
    pub twitch: Vec<String>,
    pub bttv: Vec<String>,
    pub ffz: Vec<String>,
    pub seventv: Vec<String>,
    /// The channel's own emotes of every provider (the bot prefers them).
    channel: HashSet<String>,
    /// Every name chat sees as an emote, for fast lookups of chat words.
    names: HashSet<String>,
    pub map: EmoteMap,
    /// Every emote per provider; `primary` marks the one that wins its name
    /// (the chat input suggests a name once).
    pub infos: Vec<EmoteInfo>,
}

impl EmoteCatalog {
    pub fn build(s: &Sources) -> Self {
        let mut cat = Self::default();
        let mut claimed: HashSet<&str> = HashSet::new();
        let mut per_provider: HashMap<&str, HashSet<&str>> = HashMap::new();
        for (list, provider, scope) in s.ranked() {
            let seen = per_provider.entry(provider).or_default();
            for e in list.iter().flatten() {
                // The same provider lists some names twice (7TV global + channel set).
                if e.name.is_empty() || !seen.insert(&e.name) {
                    continue;
                }
                let primary = claimed.insert(&e.name);
                cat.infos.push(EmoteInfo { name: e.name.clone(), provider, url: e.url.clone(), scope, primary });
                // Emotes of other channels' subscriptions only work for the streamer.
                if scope == Scope::Sub {
                    continue;
                }
                if scope == Scope::Channel {
                    cat.channel.insert(e.name.clone());
                }
                cat.names.insert(e.name.clone());
                if !e.url.is_empty() && !cat.map.urls.contains_key(&e.name) {
                    cat.map.urls.insert(e.name.clone(), e.url.clone());
                    if e.zero_width {
                        cat.map.zero_width.push(e.name.clone());
                    }
                }
                match provider {
                    "twitch" => &mut cat.twitch,
                    "bttv" => &mut cat.bttv,
                    "ffz" => &mut cat.ffz,
                    _ => &mut cat.seventv,
                }
                .push(e.name.clone());
            }
        }
        cat
    }

    pub fn contains(&self, name: &str) -> bool {
        self.names.contains(name)
    }

    pub fn is_channel(&self, name: &str) -> bool {
        self.channel.contains(name)
    }

    /// Different emote names.
    pub fn total(&self) -> usize {
        self.names.len()
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

// ---------------------------------------------------------------------------
// Provider responses (only the fields used; a big channel's 7TV answer is
// megabytes of data otherwise)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct BttvEmote {
    id: String,
    code: String,
    /// "w!", "h!", "z!" …: text modifiers for the next emote, not emotes.
    #[serde(default)]
    modifier: bool,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct BttvChannel {
    channel_emotes: Vec<BttvEmote>,
    shared_emotes: Vec<BttvEmote>,
}

#[derive(Deserialize)]
struct FfzEmote {
    code: String,
    #[serde(default)]
    images: HashMap<String, Option<String>>,
    /// Overlays and effects for the emote before them.
    #[serde(default)]
    modifier: bool,
}

#[derive(Deserialize)]
struct StvActive {
    id: String,
    name: String,
    #[serde(default)]
    flags: u64,
    #[serde(default)]
    data: Option<StvData>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct StvData {
    flags: u64,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct StvSet {
    id: String,
    emotes: Option<Vec<StvActive>>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct StvUser {
    emote_set: Option<StvSet>,
    user: Option<StvAccount>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct StvAccount {
    id: String,
}

fn bttv(list: Vec<BttvEmote>) -> Vec<Emote> {
    list.into_iter()
        .filter(|e| !e.modifier)
        .map(|e| Emote {
            url: format!("https://cdn.betterttv.net/emote/{}/1x.webp", e.id),
            zero_width: BTTV_ZERO_WIDTH.contains(&e.code.as_str()),
            name: e.code,
            id: e.id,
        })
        .collect()
}

fn ffz(list: Vec<FfzEmote>) -> Vec<Emote> {
    list.into_iter()
        .map(|mut e| Emote {
            id: String::new(),
            url: e.images.remove("1x").flatten().unwrap_or_default(),
            zero_width: e.modifier,
            name: e.code,
        })
        .collect()
}

fn stv(a: StvActive) -> Emote {
    // Set entry flag 1 or the emote's own flag 1 << 8: zero-width.
    let zero_width = a.flags & 1 != 0 || a.data.is_some_and(|d| d.flags & (1 << 8) != 0);
    Emote { url: format!("https://cdn.7tv.app/emote/{}/1x.webp", a.id), id: a.id, name: a.name, zero_width }
}

fn stv_list(set: Option<StvSet>) -> Vec<Emote> {
    set.and_then(|s| s.emotes).unwrap_or_default().into_iter().map(stv).collect()
}

/// A 7TV set entry from a live update.
pub fn seventv_emote(v: &Value) -> Option<Emote> {
    StvActive::deserialize(v).ok().map(stv)
}

/// Some(value) on success, Some(default) when the channel has no account
/// there (404), None when the request failed.
async fn get<T: DeserializeOwned + Default>(http: &reqwest::Client, url: &str) -> Option<T> {
    let resp = match http.get(url).send().await {
        Ok(r) => r,
        Err(e) => {
            log::warn!("emotes: {url} failed: {e}");
            return None;
        }
    };
    match resp.status().as_u16() {
        404 => Some(T::default()),
        s if (200..300).contains(&s) => match resp.json::<T>().await {
            Ok(v) => Some(v),
            Err(e) => {
                log::warn!("emotes: {url} sent unreadable data: {e}");
                None
            }
        },
        s => {
            log::warn!("emotes: {url} answered {s}");
            None
        }
    }
}

/// The 7TV emote set of a channel: (7TV account id, set id, emotes).
pub async fn fetch_7tv_channel(http: &reqwest::Client, broadcaster_id: &str) -> Option<(String, String, Vec<Emote>)> {
    let u = get::<StvUser>(http, &format!("https://7tv.io/v3/users/twitch/{broadcaster_id}")).await?;
    let user_id = u.user.map(|a| a.id).unwrap_or_default();
    let set_id = u.emote_set.as_ref().map(|s| s.id.clone()).unwrap_or_default();
    Some((user_id, set_id, stv_list(u.emote_set)))
}

/// Loads every list at once; lists whose request failed stay `None`.
pub async fn fetch(helix: &Helix, broadcaster_id: &str) -> Sources {
    let http = &helix.http;
    let twitch = |r: crate::error::AppResult<Vec<super::helix::Emote>>| {
        r.map_err(|e| log::warn!("Twitch emotes: {e}"))
            .ok()
            .map(|l| l.into_iter().map(|e| Emote { url: twitch_emote_url(&e.id), id: e.id, name: e.name, zero_width: false }).collect())
    };
    let bttv_c_url = format!("https://api.betterttv.net/3/cached/users/twitch/{broadcaster_id}");
    let ffz_c_url = format!("https://api.betterttv.net/3/cached/frankerfacez/users/twitch/{broadcaster_id}");
    let (tw_global, tw_channel, tw_user, bttv_g, bttv_c, ffz_g, ffz_c, stv_g, stv_c) = tokio::join!(
        helix.global_emotes(broadcaster_id),
        helix.channel_emotes(broadcaster_id, broadcaster_id),
        helix.user_emotes(broadcaster_id),
        get::<Vec<BttvEmote>>(http, BTTV_GLOBAL),
        get::<BttvChannel>(http, &bttv_c_url),
        get::<Vec<FfzEmote>>(http, FFZ_GLOBAL),
        get::<Vec<FfzEmote>>(http, &ffz_c_url),
        get::<StvSet>(http, STV_GLOBAL),
        fetch_7tv_channel(http, broadcaster_id),
    );
    // Logins from before 1.6.0 lack the permission: the chat input says so.
    let user_emotes_missing = tw_user.as_ref().is_err_and(|e| e.to_string().contains("user:read:emotes"));
    let (stv_user_id, stv_set_id, stv_channel) = match stv_c {
        Some((u, s, l)) => (u, s, Some(l)),
        None => (String::new(), String::new(), None),
    };
    Sources {
        twitch_channel: twitch(tw_channel),
        twitch_global: twitch(tw_global),
        twitch_user: if user_emotes_missing { Some(Vec::new()) } else { twitch(tw_user) },
        stv_channel,
        stv_global: stv_g.map(|s| stv_list(Some(s))),
        bttv_channel: bttv_c.map(|c| bttv(c.channel_emotes.into_iter().chain(c.shared_emotes).collect())),
        bttv_global: bttv_g.map(bttv),
        ffz_channel: ffz_c.map(ffz),
        ffz_global: ffz_g.map(ffz),
        stv_user_id,
        stv_set_id,
        user_emotes_missing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn e(name: &str, url: &str) -> Emote {
        Emote { id: format!("id-{name}"), name: name.into(), url: url.into(), zero_width: false }
    }

    fn sources() -> Sources {
        Sources {
            twitch_channel: Some(vec![e("zcarecHype", "tw/hype")]),
            twitch_global: Some(vec![e("Kappa", "tw/kappa"), e("BibleThump", "tw/bible")]),
            twitch_user: Some(vec![e("Kappa", "tw/kappa"), e("otherSub", "tw/sub")]),
            stv_channel: Some(vec![e("AngelThump", "7/angel"), e("catJAM", "7/cat"), e("catJAM", "7/cat2"), Emote { zero_width: true, ..e("RainTime", "7/rain") }]),
            stv_global: Some(vec![e("Clap", "7/clap"), e("RainTime", "7/rain-g"), e("BibleThump", "7/bible")]),
            bttv_channel: Some(vec![e("catJAM", "b/cat")]),
            bttv_global: Some(vec![e("AngelThump", "b/angel"), e("Clap", "b/clap")]),
            ffz_channel: Some(vec![]),
            ffz_global: Some(vec![e("WideHard", "f/wide")]),
            ..Default::default()
        }
    }

    #[test]
    fn one_winner_per_name() {
        let cat = EmoteCatalog::build(&sources());
        // A 7TV channel emote beats BTTV's global one; Twitch beats everyone.
        assert_eq!(cat.map.urls["AngelThump"], "7/angel");
        assert_eq!(cat.map.urls["BibleThump"], "tw/bible");
        assert_eq!(cat.map.urls["catJAM"], "7/cat");
        assert_eq!(cat.map.urls["Clap"], "7/clap");
        assert_eq!(cat.map.zero_width, vec!["RainTime"]);
        // Each name is suggested once …
        let primary: Vec<&str> = cat.infos.iter().filter(|i| i.primary).map(|i| i.name.as_str()).collect();
        let unique: HashSet<&str> = primary.iter().copied().collect();
        assert_eq!(primary.len(), unique.len(), "{primary:?}");
        // … but stays listed under every provider that has it (for ":b", ":7" …).
        let angel: Vec<&str> = cat.infos.iter().filter(|i| i.name == "AngelThump").map(|i| i.provider).collect();
        assert_eq!(angel, vec!["7tv", "bttv"]);
        // The same provider never lists a name twice.
        assert_eq!(cat.infos.iter().filter(|i| i.name == "catJAM" && i.provider == "7tv").count(), 1);
        // Subscription emotes are only for the streamer's own input.
        assert!(!cat.contains("otherSub") && cat.infos.iter().any(|i| i.name == "otherSub" && i.scope == Scope::Sub && i.primary));
        assert!(cat.is_channel("catJAM") && !cat.is_channel("Clap"));
        assert_eq!(cat.total(), 8);
    }

    #[test]
    fn failed_requests_keep_the_last_list() {
        let mut s = sources();
        s.stv_set_id = "set1".into();
        s.merge(Sources { bttv_global: Some(vec![e("NewOne", "b/new")]), ..Default::default() });
        assert_eq!(s.stv_channel.as_ref().unwrap().len(), 4, "7TV failed: keep it");
        assert_eq!(s.stv_set_id, "set1");
        assert_eq!(s.bttv_global.as_ref().unwrap()[0].name, "NewOne");
    }

    #[test]
    fn parses_provider_answers() {
        let b: Vec<BttvEmote> = serde_json::from_value(json!([
            {"id": "a1", "code": "TopHat", "imageType": "png", "modifier": false},
            {"id": "a2", "code": "w!", "modifier": true},
            {"id": "a3", "code": "monkaS"}
        ]))
        .unwrap();
        let b = bttv(b);
        assert_eq!(b.iter().map(|e| e.name.as_str()).collect::<Vec<_>>(), vec!["TopHat", "monkaS"]);
        assert!(b[0].zero_width && !b[1].zero_width);
        assert_eq!(b[1].url, "https://cdn.betterttv.net/emote/a3/1x.webp");

        let f: Vec<FfzEmote> = serde_json::from_value(json!([
            {"id": 9, "code": "ZrehplaR", "images": {"1x": "https://cdn.betterttv.net/frankerfacez_emote/9/1", "2x": null}},
            {"id": 10, "code": "ffzHat", "images": {"1x": "u"}, "modifier": true}
        ]))
        .unwrap();
        let f = ffz(f);
        assert_eq!(f[0].url, "https://cdn.betterttv.net/frankerfacez_emote/9/1");
        assert!(f[1].zero_width);

        let u: StvUser = serde_json::from_value(json!({
            "id": "x", "emote_set": {"id": "set1", "emotes": [
                {"id": "01A", "name": "catJAM", "flags": 0, "data": {"flags": 0, "host": {"url": "//cdn.7tv.app/emote/01A"}}},
                {"id": "01B", "name": "SteerR", "flags": 0, "data": {"flags": 256}},
                {"id": "01C", "name": "Overlay", "flags": 1}
            ]},
            "user": {"id": "u1", "username": "streamer"}
        }))
        .unwrap();
        let set_id = u.emote_set.as_ref().unwrap().id.clone();
        let list = stv_list(u.emote_set);
        assert_eq!(set_id, "set1");
        assert_eq!(list[0].url, "https://cdn.7tv.app/emote/01A/1x.webp");
        assert_eq!(list.iter().map(|e| e.zero_width).collect::<Vec<_>>(), vec![false, true, true]);
        // A channel without a set.
        let empty: StvUser = serde_json::from_value(json!({"emote_set": null})).unwrap();
        assert!(stv_list(empty.emote_set).is_empty());
    }

    #[test]
    fn applies_7tv_live_updates() {
        let mut s = sources();
        let body = json!({
            "id": "set1",
            "actor": {"display_name": "Anna"},
            "pushed": [{"key": "emotes", "index": 4, "value": {"id": "01N", "name": "peepoHey", "flags": 0, "data": {"flags": 0}}}],
            "pulled": [{"key": "emotes", "index": 0, "old_value": {"id": "id-AngelThump", "name": "AngelThump"}}],
            "updated": [{"key": "emotes", "index": 1, "old_value": {"id": "id-catJAM", "name": "catJAM"}, "value": {"id": "id-catJAM", "name": "catJAMMER", "flags": 0}}]
        });
        let changes = s.apply_7tv(&body);
        assert_eq!(
            changes,
            vec![StvChange::Removed("AngelThump".into()), StvChange::Renamed("catJAM".into(), "catJAMMER".into()), StvChange::Added("peepoHey".into())]
        );
        let cat = EmoteCatalog::build(&s);
        // BTTV's AngelThump takes over again; the set's second "catJAM" entry stays.
        assert_eq!(cat.map.urls["AngelThump"], "b/angel");
        assert_eq!(cat.map.urls["catJAM"], "7/cat2");
        assert!(cat.map.urls.contains_key("catJAMMER") && cat.map.urls.contains_key("peepoHey"));
        // Applying the same update twice changes nothing.
        assert!(s.apply_7tv(&body).iter().all(|c| !matches!(c, StvChange::Added(_))));
    }
}
