//! Per-app config (`config.json`). Holds the shared checklist directory.
//!
//! The path is read from config.json, not hardcoded in logic. On first run it defaults to the
//! project's shared store; every app keeps its own config.json so each can point elsewhere.

use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Shared location of the checklist `*.json` files.
    pub checklist_dir: PathBuf,
    /// UI theme id (also persisted by the GUI).
    pub theme: String,
    /// Play the completion chime.
    pub chime: bool,
    /// Play a tick sound when an agent ticks an item.
    pub tick: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            checklist_dir: default_checklist_dir(),
            theme: "checklist-default".to_string(),
            chime: true,
            tick: true,
        }
    }
}

impl Config {
    /// Load config.json, or defaults if absent/unreadable. Missing keys fall back to defaults.
    pub fn load() -> Self {
        match fs::read_to_string(config_path()) {
            Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
            Err(_) => Config::default(),
        }
    }

    /// Persist to config.json (creating the config dir).
    pub fn save(&self) -> Result<()> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("creating config dir {}", parent.display()))?;
        }
        let json = serde_json::to_string_pretty(self)? + "\n";
        fs::write(&path, json).with_context(|| format!("writing {}", path.display()))
    }
}

/// Where this app keeps its own config.json (per-user).
pub fn config_dir() -> PathBuf {
    if cfg!(windows) {
        let base = env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home().join("AppData").join("Roaming"));
        base.join("checklists-app")
    } else {
        let base = env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home().join(".config"));
        base.join("checklists-app")
    }
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

/// First-run default for the shared store, overridden by config.json once saved.
/// Precedence: `$CHECKLISTS_DIR`, else `~/checklists`.
pub fn default_checklist_dir() -> PathBuf {
    if let Ok(p) = env::var("CHECKLISTS_DIR") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    home().join("checklists")
}

fn home() -> PathBuf {
    env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_override_wins() {
        env::set_var("CHECKLISTS_DIR", "/tmp/some-checklists");
        assert_eq!(default_checklist_dir(), PathBuf::from("/tmp/some-checklists"));
        env::remove_var("CHECKLISTS_DIR");
    }

    #[test]
    fn config_roundtrips_via_serde() {
        let cfg = Config { checklist_dir: PathBuf::from("/x/y"), theme: "nord".into(), chime: false, tick: true };
        let text = serde_json::to_string(&cfg).unwrap();
        let back: Config = serde_json::from_str(&text).unwrap();
        assert_eq!(back.checklist_dir, PathBuf::from("/x/y"));
        assert_eq!(back.theme, "nord");
        assert!(!back.chime);
    }

    #[test]
    fn defaults_theme_and_chime() {
        let back: Config = serde_json::from_str("{}").unwrap();
        assert_eq!(back.theme, "checklist-default");
        assert!(back.chime);
    }

    #[test]
    fn missing_keys_use_defaults() {
        // An empty object must still deserialize (serde(default) on the struct).
        let back: Config = serde_json::from_str("{}").unwrap();
        assert_eq!(back.checklist_dir, default_checklist_dir());
    }
}
