//! Налаштування гри: мова, звуки, клавіші
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Language {
    Ukrainian,
    English,
    Russian,
}

impl Language {
    pub fn code(&self) -> &str {
        match self {
            Language::Ukrainian => "uk",
            Language::English => "en",
            Language::Russian => "ru",
        }
    }

    pub fn next(&self) -> Language {
        match self {
            Language::Ukrainian => Language::English,
            Language::English => Language::Russian,
            Language::Russian => Language::Ukrainian,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KeyBindings {
    pub move_up: String,
    pub move_down: String,
    pub move_left: String,
    pub move_right: String,
    pub action: String,
    pub eat: String,
    pub drink: String,
    pub dig: String,
    pub menu: String,
    pub quick1: String,
    pub quick2: String,
    pub quick3: String,
    pub quick4: String,
    pub quick5: String,
}

impl Default for KeyBindings {
    fn default() -> Self {
        KeyBindings {
            move_up: "W".to_string(),
            move_down: "S".to_string(),
            move_left: "A".to_string(),
            move_right: "D".to_string(),
            action: "Space".to_string(),
            eat: "F".to_string(),
            drink: "R".to_string(),
            dig: "G".to_string(),
            menu: "Tab".to_string(),
            quick1: "1".to_string(),
            quick2: "2".to_string(),
            quick3: "3".to_string(),
            quick4: "4".to_string(),
            quick5: "5".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Settings {
    pub language: Language,
    pub music_volume: f32,
    pub sound_volume: f32,
    pub music_enabled: bool,
    pub sounds_enabled: bool,
    pub keybindings: KeyBindings,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            language: Language::Ukrainian,
            music_volume: 0.5,
            sound_volume: 0.7,
            music_enabled: true,
            sounds_enabled: true,
            keybindings: KeyBindings::default(),
        }
    }
}

fn settings_dir() -> PathBuf {
    if let Ok(d) = std::env::var("PV_SAVE_DIR") {
        return PathBuf::from(d);
    }
    #[cfg(target_os = "android")]
    {
        PathBuf::from("/data/data/com.gertyn.pixelvillage/files")
    }
    #[cfg(not(target_os = "android"))]
    {
        if let Ok(a) = std::env::var("APPDATA") {
            return PathBuf::from(a).join("PixelVillage");
        }
        if let Ok(h) = std::env::var("HOME") {
            return PathBuf::from(h).join(".pixel_village");
        }
        PathBuf::from(".")
    }
}

pub fn load_settings() -> Settings {
    let path = settings_dir().join("settings.json");
    if let Ok(s) = std::fs::read_to_string(&path) {
        if let Ok(settings) = serde_json::from_str(&s) {
            return settings;
        }
    }
    Settings::default()
}

pub fn save_settings(settings: &Settings) {
    let dir = settings_dir();
    let _ = std::fs::create_dir_all(&dir);
    if let Ok(json) = serde_json::to_string_pretty(settings) {
        let _ = std::fs::write(dir.join("settings.json"), json);
    }
}
