use serde::{Deserialize, Deserializer};
use std::path::PathBuf;

fn deserialize_color<'de, D: Deserializer<'de>>(d: D) -> Result<u32, D::Error> {
    let s = String::deserialize(d)?;
    let s = s.trim_start_matches("0x").trim_start_matches('#');
    u32::from_str_radix(s, 16).map_err(serde::de::Error::custom)
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    pub gap: u32,
    pub border_width: u32,
    #[serde(deserialize_with = "deserialize_color")]
    pub border_color: u32,
    #[serde(deserialize_with = "deserialize_color")]
    pub border_focus_color: u32,
    pub master_ratio: f32,
    pub terminal: String,
    pub keybinds: KeybindsConfig,
    pub bar: BarConfig,
    pub autostart: Vec<String>,
    pub custom_keys: Vec<CustomKey>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct BarConfig {
    pub enabled: bool,
    pub height: u32,
    pub position: BarPosition,
    #[serde(deserialize_with = "deserialize_color")]
    pub bg_color: u32,
    #[serde(deserialize_with = "deserialize_color")]
    pub fg_color: u32,
    #[serde(deserialize_with = "deserialize_color")]
    pub workspace_active_color: u32,
    #[serde(deserialize_with = "deserialize_color")]
    pub workspace_inactive_color: u32,
    #[serde(deserialize_with = "deserialize_color")]
    pub workspace_fg_color: u32,
    pub show_time: bool,
    pub time_format: TimeFormat,
    pub font: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(rename_all = "lowercase")]
pub enum BarPosition {
    #[default]
    Top,
    Bottom,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub enum TimeFormat {
    #[default]
    #[serde(rename = "24h")]
    H24,
    #[serde(rename = "12h")]
    H12,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct KeybindsConfig {
    pub close_window: String,
    pub quit: String,
    pub swap_master: String,
    pub focus_next: String,
    pub focus_prev: String,
    pub terminal: String,
    pub restart: String,
    pub workspace_switch: String,
    pub workspace_move: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CustomKey {
    pub binding: String,
    pub action: String,
    pub arg: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            gap: 8,
            border_width: 2,
            border_color: 0x444444,
            border_focus_color: 0x8888ff,
            master_ratio: 0.5,
            terminal: "kitty".to_string(),
            keybinds: KeybindsConfig::default(),
            bar: BarConfig::default(),
            autostart: vec![],
            custom_keys: vec![],
        }
    }
}

impl Default for KeybindsConfig {
    fn default() -> Self {
        Self {
            close_window: "super+q".to_string(),
            quit: "super+shift+q".to_string(),
            swap_master: "super+w".to_string(),
            focus_next: "super+j".to_string(),
            focus_prev: "super+k".to_string(),
            terminal: "super+return".to_string(),
            restart: "super+shift+r".to_string(),
            workspace_switch: "super".to_string(),
            workspace_move: "super+shift".to_string(),
        }
    }
}

impl Default for BarConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            height: 24,
            position: BarPosition::Top,
            bg_color: 0x1a1a2e,
            fg_color: 0xffffff,
            workspace_active_color: 0x8888ff,
            workspace_inactive_color: 0x2a2a3e,
            workspace_fg_color: 0xffffff,
            show_time: true,
            time_format: TimeFormat::H24,
            font: "-misc-fixed-medium-r-normal--13-*".to_string(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = config_path();

        if path.as_os_str().is_empty() || !path.exists() {
            println!("No config found, using defaults");
            return Self::default();
        }

        println!("Loading config from {:?}", path);

        let contents = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!("Failed to read config: {e}");
            String::new()
        });

        toml::from_str(&contents).unwrap_or_else(|e| {
            eprintln!("Failed to parse config: {e}");
            Self::default()
        })
    }
}

fn config_path() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        let p = PathBuf::from(xdg).join("orbitwm/config.toml");
        if p.exists() {
            return p;
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let p = PathBuf::from(home).join(".config/orbitwm/config.toml");
        if p.exists() {
            return p;
        }
    }
    let system = PathBuf::from("/etc/orbitwm/config.toml");
    if system.exists() {
        return system;
    }
    PathBuf::from("")
}
