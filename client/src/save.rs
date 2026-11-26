//! Сохранение прогресса в файл (ПК и Android).
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::data::{NI, NT};

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct Save {
    pub name: String,
    pub inv: Vec<u32>,
    pub tools: Vec<i32>,
    pub gold: u32,
    pub x: f32,
    pub y: f32,
    pub hunger: f32,
    pub thirst: f32,
    pub energy: f32,
    pub hp: f32,
    /// (x, y, сколько секунд ещё горит)
    pub fires: Vec<(f32, f32, f64)>,
    pub furn: Vec<(f32, f32)>,
    pub bridges: Vec<(i32, i32)>,
    pub houses: u32,
    pub made: u32,
    pub got: u32,
    pub caught: u32,
    pub biggest: f32,
    pub has_pet: bool,
    pub in_boat: bool,
}

fn dir() -> PathBuf {
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

pub fn load() -> Option<Save> {
    let s = std::fs::read_to_string(dir().join("save.json")).ok()?;
    let mut save: Save = serde_json::from_str(&s).ok()?;
    if save.inv.len() < NI { save.inv.resize(NI, 0); }
    if save.tools.len() < NT { save.tools.resize(NT, 0); }
    save.gold = save.gold.min(999_999);
    save.hp = save.hp.clamp(0.0, 100.0);
    save.hunger = save.hunger.clamp(0.0, 100.0);
    save.thirst = save.thirst.clamp(0.0, 100.0);
    save.energy = save.energy.clamp(0.0, 100.0);
    for count in &mut save.inv {
        *count = (*count).min(999);
    }
    Some(save)
}

pub fn store(s: &Save) {
    let d = dir();
    let _ = std::fs::create_dir_all(&d);
    if let Ok(json) = serde_json::to_string(s) {
        let tmp = d.join("save.tmp");
        if std::fs::write(&tmp, json).is_ok() {
            let _ = std::fs::rename(&tmp, d.join("save.json"));
        }
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct AccountCredentials {
    pub email: String,
    pub password: String,
    pub username: String,
}

pub fn load_account_creds() -> Option<AccountCredentials> {
    let s = std::fs::read_to_string(dir().join("account.json")).ok()?;
    serde_json::from_str(&s).ok()
}

pub fn store_account_creds(creds: &AccountCredentials) {
    let d = dir();
    let _ = std::fs::create_dir_all(&d);
    if let Ok(json) = serde_json::to_string(creds) {
        let tmp = d.join("account.tmp");
        if std::fs::write(&tmp, json).is_ok() {
            let _ = std::fs::rename(&tmp, d.join("account.json"));
        }
    }
}

pub fn clear_account_creds() {
    let _ = std::fs::remove_file(dir().join("account.json"));
}
