//! Система автоматичного оновлення гри

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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
}

pub struct Updater {
    pub current_version: String,
    pub update_info: Option<UpdateInfo>,
    pub downloading: bool,
    pub download_progress: f32,
    pub installing: bool,
    pub error: Option<String>,
}

impl Updater {
    pub fn new(current_version: &str) -> Self {
        Self {
            current_version: current_version.to_string(),
            update_info: None,
            downloading: false,
            download_progress: 0.0,
            installing: false,
            error: None,
        }
    }

    /// Перевірити наявність оновлень через GitHub API
    pub async fn check_for_updates(&mut self) -> Result<bool, String> {
        let url = "https://api.github.com/repos/gertyn230/game/releases/latest";

        match self.fetch_latest_release(url).await {
            Ok(info) => {
                if self.is_newer_version(&info.version) {
                    self.update_info = Some(info);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Err(e) => {
                self.error = Some(e.clone());
                Err(e)
            }
        }
    }

    /// Отримати інформацію про останній реліз з GitHub
    async fn fetch_latest_release(&self, url: &str) -> Result<UpdateInfo, String> {
        // Визначити платформу
        let file_extension = if cfg!(target_os = "android") {
            ".apk"
        } else {
            ".exe"
        };

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
        })
    }

    /// Завантажити та встановити оновлення автоматично
    #[cfg(target_os = "windows")]
    pub async fn download_and_install(&mut self) -> Result<(), String> {
        if self.update_info.is_none() {
            return Err("Немає інформації про оновлення".to_string());
        }

        let info = self.update_info.as_ref().unwrap();
        self.downloading = true;
        self.download_progress = 0.0;

        // Завантажити файл
        let bytes = macroquad::prelude::load_file(&info.download_url)
            .await
            .map_err(|e| format!("Помилка завантаження: {:?}", e))?;

        self.download_progress = 1.0;
        self.downloading = false;
        self.installing = true;

        // Зберегти у temp
        let temp_path = std::env::temp_dir().join("PixelVillage_update.exe");
        fs::write(&temp_path, bytes)
            .map_err(|e| format!("Помилка збереження: {}", e))?;

        // Встановити
        self.install_update(&temp_path)?;

        self.installing = false;
        Ok(())
    }

    /// Встановити оновлення (Windows) через батник-скрипт
    #[cfg(target_os = "windows")]
    fn install_update(&self, update_path: &PathBuf) -> Result<(), String> {
        let current_exe = std::env::current_exe()
            .map_err(|e| format!("Не вдалося знайти поточний exe: {}", e))?;

        // Створити батник-скрипт для оновлення
        let batch_path = current_exe.parent().unwrap().join("_update.bat");

        let batch_content = format!(
            r#"@echo off
chcp 65001 >nul
timeout /t 2 /nobreak >nul
del /f /q "{current_exe}"
move /y "{update_path}" "{current_exe}"
start "" "{current_exe}"
del /f /q "%~f0"
"#,
            current_exe = current_exe.display(),
            update_path = update_path.display()
        );

        fs::write(&batch_path, batch_content)
            .map_err(|e| format!("Помилка створення батника: {}", e))?;

        // Запустити батник і вийти
        std::process::Command::new("cmd")
            .args(&["/c", "start", "/min", batch_path.to_str().unwrap()])
            .spawn()
            .map_err(|e| format!("Помилка запуску батника: {}", e))?;

        // Вийти з поточного процесу
        std::process::exit(0);
    }

    /// Заглушка для Android - просто відкриває браузер
    #[cfg(target_os = "android")]
    pub async fn download_and_install(&mut self) -> Result<(), String> {
        if let Some(ref info) = self.update_info {
            // На Android просто показуємо повідомлення
            Err(format!("Оновлення доступне! Версія: {}. Завантажте APK вручну.", info.version))
        } else {
            Err("Немає інформації про оновлення".to_string())
        }
    }

    /// Заглушка для інших платформ
    #[cfg(not(any(target_os = "windows", target_os = "android")))]
    pub async fn download_and_install(&mut self) -> Result<(), String> {
        Err("Автооновлення не підтримується на цій платформі".to_string())
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

    /// Отримати статус оновлення для показу
    pub fn get_status_text(&self) -> String {
        if self.downloading {
            format!("⬇️ Завантаження оновлення... {:.0}%", self.download_progress * 100.0)
        } else if self.installing {
            "⚙️ Встановлення оновлення...".to_string()
        } else if let Some(ref info) = self.update_info {
            format!("✨ Доступна версія {}", info.version)
        } else {
            String::new()
        }
    }
}
