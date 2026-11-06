//! Система автоматичного оновлення гри

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
struct GitHubRelease {
    tag_name: String,
    name: String,
    body: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Deserialize, Serialize, Debug)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

pub struct UpdateInfo {
    pub version: String,
    pub download_url: String,
    pub changelog: String,
    pub size_mb: f32,
    pub is_apk: bool,
}

pub struct Updater {
    pub current_version: String,
    pub update_info: Option<UpdateInfo>,
    pub checking: bool,
    pub error: Option<String>,
}

impl Updater {
    pub fn new(current_version: &str) -> Self {
        Self {
            current_version: current_version.to_string(),
            update_info: None,
            checking: false,
            error: None,
        }
    }

    /// Перевірити наявність оновлень через GitHub API
    pub async fn check_for_updates(&mut self) -> Result<bool, String> {
        self.checking = true;
        self.error = None;

        let url = "https://api.github.com/repos/gertyn230/game/releases/latest";

        match self.fetch_latest_release(url).await {
            Ok(info) => {
                if self.is_newer_version(&info.version) {
                    self.update_info = Some(info);
                    self.checking = false;
                    Ok(true)
                } else {
                    self.checking = false;
                    Ok(false)
                }
            }
            Err(e) => {
                self.error = Some(e.clone());
                self.checking = false;
                Err(e)
            }
        }
    }

    /// Отримати інформацію про останній реліз з GitHub
    async fn fetch_latest_release(&self, url: &str) -> Result<UpdateInfo, String> {
        // Визначити платформу
        let is_android = cfg!(target_os = "android");
        let file_extension = if is_android { ".apk" } else { ".exe" };

        // HTTP запит до GitHub API
        let response = macroquad::prelude::load_string(url)
            .await
            .map_err(|e| format!("Помилка завантаження: {:?}", e))?;

        let release: GitHubRelease = serde_json::from_str(&response)
            .map_err(|e| format!("Помилка парсингу JSON: {}", e))?;

        let version = release.tag_name.trim_start_matches('v').to_string();

        // Знайти відповідний файл для платформи
        let asset = release
            .assets
            .iter()
            .find(|a| a.name.ends_with(file_extension))
            .ok_or_else(|| format!("Немає файлу {} в релізі", file_extension))?;

        let size_mb = asset.size as f32 / 1024.0 / 1024.0;

        Ok(UpdateInfo {
            version,
            download_url: asset.browser_download_url.clone(),
            changelog: release.body,
            size_mb,
            is_apk: is_android,
        })
    }

    /// Перевірити чи нова версія новіша за поточну
    fn is_newer_version(&self, new_version: &str) -> bool {
        let current_parts: Vec<u32> = self
            .current_version
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();

        let new_parts: Vec<u32> = new_version
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();

        for i in 0..3 {
            let cur = current_parts.get(i).unwrap_or(&0);
            let new = new_parts.get(i).unwrap_or(&0);

            if new > cur {
                return true;
            } else if new < cur {
                return false;
            }
        }

        false
    }

    /// Отримати текст для показу в UI
    pub fn get_update_text(&self) -> Option<String> {
        if let Some(ref info) = self.update_info {
            let platform = if info.is_apk { "APK" } else { "EXE" };
            Some(format!(
                "🎮 Нова версія {}!\n\n📦 Розмір: {:.1} MB\n💾 Тип: {}\n\n📝 Зміни:\n{}",
                info.version, info.size_mb, platform, info.changelog
            ))
        } else {
            None
        }
    }

    /// Відкрити браузер для завантаження (для Desktop та Android)
    pub fn open_download_page(&self) {
        if let Some(ref info) = self.update_info {
            // Для Android та Desktop - відкрити посилання в браузері
            if let Err(e) = open_url(&info.download_url) {
                eprintln!("Не вдалось відкрити браузер: {:?}", e);
            }
        }
    }
}

/// Відкрити URL в браузері (кросплатформно)
#[cfg(target_os = "windows")]
fn open_url(url: &str) -> Result<(), String> {
    std::process::Command::new("cmd")
        .args(&["/c", "start", url])
        .spawn()
        .map_err(|e| format!("Помилка відкриття браузера: {}", e))?;
    Ok(())
}

#[cfg(target_os = "android")]
fn open_url(url: &str) -> Result<(), String> {
    // На Android використовуємо Intent для відкриття браузера
    use macroquad::prelude::*;
    // macroquad має вбудовану функцію для Android
    if let Err(e) = miniquad::window::open_url(url) {
        return Err(format!("Помилка: {:?}", e));
    }
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "android")))]
fn open_url(url: &str) -> Result<(), String> {
    // Linux/MacOS
    std::process::Command::new("xdg-open")
        .arg(url)
        .spawn()
        .map_err(|e| format!("Помилка відкриття браузера: {}", e))?;
    Ok(())
}
