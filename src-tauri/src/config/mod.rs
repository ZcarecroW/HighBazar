pub mod presets;
pub mod types;

use std::path::{Path, PathBuf};

pub use types::*;

use crate::error::AppResult;

// Built-in Twitch application (public client, device code flow). The ID is
// injected at build time from `HIGHBAZAR_TWITCH_CLIENT_ID` and stored masked,
// see build.rs. Users can override it in the advanced settings.
include!(concat!(env!("OUT_DIR"), "/twitch_client_id.rs"));

fn builtin_twitch_client_id() -> String {
    MASKED_CLIENT_ID
        .iter()
        .zip(CLIENT_ID_KEY)
        .map(|(b, k)| (b ^ k) as char)
        .collect()
}

impl Config {
    /// Empty if neither a built-in nor a custom Client ID is available.
    pub fn twitch_client_id(&self) -> String {
        let custom = self.settings.twitch_client_id.trim();
        if custom.is_empty() {
            builtin_twitch_client_id()
        } else {
            custom.to_string()
        }
    }

    pub fn channel(&self, id: &str) -> Option<&ChannelProfile> {
        self.channels.iter().find(|c| c.id == id)
    }

    pub fn companion(&self, id: &str) -> CompanionProfile {
        self.companion_profiles
            .iter()
            .find(|p| p.id == id)
            .or_else(|| self.companion_profiles.iter().find(|p| p.id == presets::DEFAULT_COMPANION))
            .cloned()
            .unwrap_or_default()
    }

    pub fn moderation(&self, id: &str) -> ModerationProfile {
        self.moderation_profiles
            .iter()
            .find(|p| p.id == id)
            .or_else(|| self.moderation_profiles.iter().find(|p| p.id == presets::DEFAULT_MODERATION))
            .cloned()
            .unwrap_or_default()
    }

    pub fn ai_profile(&self, id: &str) -> Option<AiProfile> {
        self.ai_profiles.iter().find(|p| p.id == id).cloned()
    }

    pub fn account(&self, id: &str) -> Option<&TwitchAccount> {
        self.accounts.iter().find(|a| a.id == id)
    }

    /// Replaces built-in profiles with their canonical definitions, keeps user
    /// profiles and makes sure every reference points somewhere valid.
    pub fn normalize(&mut self) {
        fn merge<T: Clone>(user: &mut Vec<T>, builtin: Vec<T>, is_builtin: impl Fn(&T) -> bool) {
            user.retain(|p| !is_builtin(p));
            let mut all = builtin;
            all.append(user);
            *user = all;
        }
        merge(&mut self.companion_profiles, presets::companion_presets(), |p| p.builtin);
        merge(&mut self.moderation_profiles, presets::moderation_presets(), |p| p.builtin);
        merge(&mut self.ai_profiles, presets::ai_presets(), |p| p.builtin);

        for p in self.moderation_profiles.iter_mut() {
            // Make sure every known category exists exactly once.
            for cat in CATEGORIES {
                if !p.categories.iter().any(|r| r.category == *cat)
                    && let Some(def) = default_categories().into_iter().find(|r| r.category == *cat) {
                        p.categories.push(def);
                    }
            }
        }

        let companion_ids: Vec<String> = self.companion_profiles.iter().map(|p| p.id.clone()).collect();
        let moderation_ids: Vec<String> = self.moderation_profiles.iter().map(|p| p.id.clone()).collect();
        let ai_ids: Vec<String> = self.ai_profiles.iter().map(|p| p.id.clone()).collect();
        let account_ids: Vec<String> = self.accounts.iter().map(|a| a.id.clone()).collect();
        for c in self.channels.iter_mut() {
            if c.id.is_empty() {
                c.id = new_id();
            }
            if !companion_ids.contains(&c.companion_profile_id) {
                c.companion_profile_id = presets::DEFAULT_COMPANION.into();
            }
            if !moderation_ids.contains(&c.moderation_profile_id) {
                c.moderation_profile_id = presets::DEFAULT_MODERATION.into();
            }
            if !c.ai_profile_id.is_empty() && !ai_ids.contains(&c.ai_profile_id) {
                c.ai_profile_id.clear();
            }
            if !c.moderation_ai_profile_id.is_empty() && !ai_ids.contains(&c.moderation_ai_profile_id) {
                c.moderation_ai_profile_id.clear();
            }
            if let Some(id) = &c.streamer_account_id
                && !account_ids.contains(id) {
                    c.streamer_account_id = None;
                }
            if let Some(id) = &c.bot_account_id
                && !account_ids.contains(id) {
                    c.bot_account_id = None;
                }
        }
        if self.channels.is_empty() {
            self.channels.push(ChannelProfile {
                id: new_id(),
                ..Default::default()
            });
        }
        let active_ok = self
            .settings
            .active_channel_id
            .as_ref()
            .is_some_and(|id| self.channels.iter().any(|c| &c.id == id));
        if !active_ok {
            self.settings.active_channel_id = self.channels.first().map(|c| c.id.clone());
        }
        self.settings.feed_limit = self.settings.feed_limit.clamp(50, 5000);
    }
}

pub fn new_id() -> String {
    uuid::Uuid::new_v4().simple().to_string()[..12].to_string()
}

pub fn config_path(dir: &Path) -> PathBuf {
    dir.join("config.json")
}

pub fn load(dir: &Path) -> Config {
    let path = config_path(dir);
    let mut cfg = match std::fs::read_to_string(&path) {
        Ok(text) => match serde_json::from_str::<Config>(&text) {
            Ok(c) => c,
            Err(e) => {
                log::error!("config.json is invalid ({e}); keeping a backup and starting fresh");
                let _ = std::fs::copy(&path, dir.join("config.invalid.json"));
                Config::default()
            }
        },
        Err(_) => Config::default(),
    };
    cfg.normalize();
    cfg
}

pub fn save(dir: &Path, cfg: &Config) -> AppResult<()> {
    crate::storage::write_json_atomic(&config_path(dir), cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_id_mask_roundtrip() {
        let expected = option_env!("HIGHBAZAR_TWITCH_CLIENT_ID").unwrap_or("").trim();
        assert_eq!(builtin_twitch_client_id(), expected);
    }

    #[test]
    fn normalize_creates_defaults() {
        let mut c = Config::default();
        c.normalize();
        assert_eq!(c.channels.len(), 1);
        assert!(c.companion_profiles.iter().any(|p| p.id == presets::DEFAULT_COMPANION));
        assert!(c.moderation_profiles.iter().all(|p| p.categories.len() >= CATEGORIES.len()));
        assert_eq!(c.settings.active_channel_id.as_deref(), Some(c.channels[0].id.as_str()));
    }

    #[test]
    fn builtins_are_restored_and_user_profiles_kept() {
        let mut c = Config::default();
        c.normalize();
        c.companion_profiles[0].name = "hacked".into();
        let mut mine = c.companion_profiles[0].clone();
        mine.id = "mine".into();
        mine.builtin = false;
        c.companion_profiles.push(mine);
        c.normalize();
        assert_ne!(c.companion_profiles[0].name, "hacked");
        assert!(c.companion_profiles.iter().any(|p| p.id == "mine"));
    }

    /// Writes the default config for the frontend's browser preview mock:
    /// `cargo test dump_default_config -- --ignored`
    #[test]
    #[ignore]
    fn dump_default_config() {
        let mut c = Config::default();
        c.normalize();
        c.channels[0].ai_profile_id = "builtin-ai-claude-opus".into();
        let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/lib/mock-config.json");
        std::fs::write(out, serde_json::to_string_pretty(&c).unwrap()).unwrap();
    }
}
