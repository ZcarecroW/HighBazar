//! AI usage and cost bookkeeping per day and channel profile.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::storage;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct UsageStats {
    pub calls: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost_usd: f64,
    pub messages_sent: u64,
    pub mod_actions: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct UsageBook {
    /// date (YYYY-MM-DD, local) → channel id → stats
    pub days: BTreeMap<String, BTreeMap<String, UsageStats>>,
    #[serde(skip)]
    path: PathBuf,
    #[serde(skip)]
    dirty: bool,
}

pub fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

impl UsageBook {
    pub fn load(dir: &Path) -> Self {
        let path = dir.join("usage.json");
        let mut b: UsageBook = storage::read_json(&path).unwrap_or_default();
        b.path = path;
        b
    }

    fn entry(&mut self, channel: &str) -> &mut UsageStats {
        self.dirty = true;
        self.days.entry(today()).or_default().entry(channel.to_string()).or_default()
    }

    pub fn add_ai(&mut self, channel: &str, input: u64, output: u64, cost: f64) {
        let e = self.entry(channel);
        e.calls += 1;
        e.input_tokens += input;
        e.output_tokens += output;
        e.cost_usd += cost;
    }

    pub fn add_message(&mut self, channel: &str) {
        self.entry(channel).messages_sent += 1;
    }

    pub fn add_mod_action(&mut self, channel: &str) {
        self.entry(channel).mod_actions += 1;
    }

    pub fn today_for(&self, channel: &str) -> UsageStats {
        self.days.get(&today()).and_then(|d| d.get(channel)).cloned().unwrap_or_default()
    }

    pub fn save_if_dirty(&mut self) {
        if !self.dirty {
            return;
        }
        // Keep roughly the last 90 days.
        while self.days.len() > 90 {
            let first = self.days.keys().next().cloned();
            if let Some(k) = first {
                self.days.remove(&k);
            }
        }
        if storage::write_json_atomic(&self.path, self).is_ok() {
            self.dirty = false;
        }
    }
}
