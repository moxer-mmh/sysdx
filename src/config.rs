use anyhow::Result;
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub display: DisplayOptions,
    pub keybinds: KeyBinds,
    pub colors: Colors,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct DisplayOptions {
    pub journal_lines: usize,
    pub tick_rate_ms: u64,
    pub show_description: bool,
    pub list_width_pct: u16,
    pub date_format: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct KeyBinds {
    pub move_down: String,
    pub move_up: String,
    pub page_down: String,
    pub page_up: String,
    pub go_top: String,
    pub go_bottom: String,
    pub filter: String,
    pub action_menu: String,
    pub switch_scope: String,
    pub open_logs: String,
    pub quit: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct Colors {
    pub background: Option<String>,
    pub surface: Option<String>,
    pub border: Option<String>,
    pub border_focused: Option<String>,
    pub text: Option<String>,
    pub text_dim: Option<String>,
    pub selection_bg: Option<String>,
    pub selection_fg: Option<String>,
    pub active: Option<String>,
    pub inactive: Option<String>,
    pub failed: Option<String>,
    pub filter_bar: Option<String>,
    pub header: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            display: DisplayOptions::default(),
            keybinds: KeyBinds::default(),
            colors: Colors::default(),
        }
    }
}

impl Default for DisplayOptions {
    fn default() -> Self {
        Self {
            journal_lines: 50,
            tick_rate_ms: 250,
            show_description: true,
            list_width_pct: 40,
            date_format: "%H:%M:%S".to_string(),
        }
    }
}

impl Default for KeyBinds {
    fn default() -> Self {
        Self {
            move_down: "j".to_string(),
            move_up: "k".to_string(),
            page_down: "ctrl-d".to_string(),
            page_up: "ctrl-u".to_string(),
            go_top: "g".to_string(),
            go_bottom: "G".to_string(),
            filter: "/".to_string(),
            action_menu: "enter".to_string(),
            switch_scope: "tab".to_string(),
            open_logs: "l".to_string(),
            quit: "q".to_string(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = std::fs::read_to_string(&path)?;
        let partial: toml::Value = toml::from_str(&raw)?;
        // Merge: deserialize partial over default so missing keys don't panic
        let merged = merge_with_default(partial)?;
        Ok(merged)
    }

    pub fn config_path() -> PathBuf {
        config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("sysdx")
            .join("config.toml")
    }
}

fn merge_with_default(partial: toml::Value) -> Result<Config> {
    let default_str = toml::to_string(&Config::default())?;
    let mut default_val: toml::Value = toml::from_str(&default_str)?;
    merge_toml(&mut default_val, partial);
    let merged: Config = default_val.try_into()?;
    Ok(merged)
}

fn merge_toml(base: &mut toml::Value, override_val: toml::Value) {
    if let (toml::Value::Table(base_map), toml::Value::Table(override_map)) = (base, override_val) {
        for (k, v) in override_map {
            let entry = base_map.entry(k).or_insert(toml::Value::Boolean(false));
            if v.is_table() && entry.is_table() {
                merge_toml(entry, v);
            } else {
                *entry = v;
            }
        }
    }
}
