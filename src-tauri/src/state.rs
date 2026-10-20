use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use parking_lot::RwLock;

use crate::bot::BotManager;
use crate::config::{self, Config};
use crate::error::AppResult;
use crate::secrets::Secrets;
use crate::twitch::auth::TokenManager;
use crate::twitch::helix::Helix;

pub struct AppState {
    pub config: RwLock<Config>,
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub secrets: Arc<Secrets>,
    pub http: reqwest::Client,
    pub tokens: Arc<TokenManager>,
    pub bot: BotManager,
    pub usage: parking_lot::Mutex<crate::bot::usage::UsageBook>,
    /// Device-code logins in progress (id → cancelled flag).
    pub logins: parking_lot::Mutex<std::collections::HashMap<String, Arc<std::sync::atomic::AtomicBool>>>,
    /// Emote name → image URL of the channel the bot last loaded (for the UI).
    pub emote_urls: RwLock<Arc<std::collections::HashMap<String, String>>>,
    /// Live feed fan-out (Steam overlay page, reopened windows).
    pub overlay: crate::overlay::Hub,
    /// Suggestions for the streamer's own chat input.
    pub chat: crate::chat::ChatAssist,
    pub overlay_server: crate::overlay::Server,
}

impl AppState {
    pub fn new(config_dir: PathBuf, data_dir: PathBuf) -> Self {
        let chat_dir = data_dir.clone();
        let http = reqwest::Client::builder()
            .user_agent(concat!("HighBazar/", env!("CARGO_PKG_VERSION")))
            .connect_timeout(Duration::from_secs(15))
            .timeout(Duration::from_secs(120))
            .pool_idle_timeout(Duration::from_secs(90))
            .build()
            .expect("HTTP client");
        let secrets = Arc::new(Secrets::new(data_dir.clone()));
        let tokens = Arc::new(TokenManager::new(http.clone(), secrets.clone()));
        let cfg = config::load(&config_dir);
        let usage = parking_lot::Mutex::new(crate::bot::usage::UsageBook::load(&data_dir));
        Self {
            config: RwLock::new(cfg),
            config_dir,
            usage,
            data_dir,
            secrets,
            http,
            tokens,
            bot: BotManager::default(),
            logins: Default::default(),
            emote_urls: Default::default(),
            overlay: Default::default(),
            chat: crate::chat::ChatAssist::new(&chat_dir),
            overlay_server: Default::default(),
        }
    }

    pub fn helix(&self) -> Helix {
        Helix {
            http: self.http.clone(),
            client_id: self.config.read().twitch_client_id(),
            tokens: self.tokens.clone(),
        }
    }

    pub fn save_config(&self) -> AppResult<()> {
        let cfg = self.config.read().clone();
        config::save(&self.config_dir, &cfg)
    }

    pub fn ai_key(&self, profile: &config::AiProfile) -> Option<String> {
        self.secrets.get(&crate::secrets::ai_key_name(&crate::ai::key_ref(profile)))
    }

    /// Key of a web search source (Ollama, Tavily, Brave).
    pub fn search_key(&self, source: &str) -> Option<String> {
        self.secrets.get(&crate::secrets::ai_key_name(&crate::search::key_ref(source)))
    }
}
