//! Secret storage. Uses the OS credential store (Windows Credential Manager,
//! macOS Keychain, Secret Service on Linux). If that is not available (e.g. a
//! Linux desktop without a Secret Service provider) it falls back to a file
//! in the app data directory that only the current user can read.

use std::collections::BTreeMap;
use std::path::PathBuf;

use parking_lot::Mutex;

use crate::error::{AppError, AppResult};

const SERVICE: &str = "HighBazar";

pub struct Secrets {
    fallback_path: PathBuf,
    use_fallback: bool,
    lock: Mutex<()>,
}

impl Secrets {
    pub fn new(data_dir: PathBuf) -> Self {
        let use_fallback = keyring::Entry::store_status().is_err();
        if use_fallback {
            log::warn!("OS credential store unavailable, using file fallback");
        }
        Self {
            fallback_path: data_dir.join("secrets.json"),
            use_fallback,
            lock: Mutex::new(()),
        }
    }

    pub fn backend(&self) -> &'static str {
        if self.use_fallback { "file" } else { "os" }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        if self.use_fallback {
            let _g = self.lock.lock();
            return self.read_file().remove(key);
        }
        let entry = keyring::Entry::new(SERVICE, key).ok()?;
        match entry.get_password() {
            Ok(v) => Some(v),
            Err(keyring::Error::NoEntry) => None,
            Err(e) => {
                log::warn!("reading secret {key} failed: {e}");
                None
            }
        }
    }

    pub fn set(&self, key: &str, value: &str) -> AppResult<()> {
        if self.use_fallback {
            let _g = self.lock.lock();
            let mut map = self.read_file();
            map.insert(key.to_string(), value.to_string());
            return self.write_file(&map);
        }
        let entry = keyring::Entry::new(SERVICE, key).map_err(|e| AppError::Keyring(e.to_string()))?;
        entry.set_password(value).map_err(|e| AppError::Keyring(e.to_string()))
    }

    pub fn delete(&self, key: &str) -> AppResult<()> {
        if self.use_fallback {
            let _g = self.lock.lock();
            let mut map = self.read_file();
            map.remove(key);
            return self.write_file(&map);
        }
        let entry = keyring::Entry::new(SERVICE, key).map_err(|e| AppError::Keyring(e.to_string()))?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(AppError::Keyring(e.to_string())),
        }
    }

    fn read_file(&self) -> BTreeMap<String, String> {
        crate::storage::read_json(&self.fallback_path).unwrap_or_default()
    }

    fn write_file(&self, map: &BTreeMap<String, String>) -> AppResult<()> {
        crate::storage::write_json_atomic(&self.fallback_path, map)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&self.fallback_path, std::fs::Permissions::from_mode(0o600));
        }
        Ok(())
    }
}

pub fn ai_key_name(key_ref: &str) -> String {
    format!("ai:{key_ref}")
}

pub fn twitch_key_name(user_id: &str) -> String {
    format!("twitch:{user_id}")
}
