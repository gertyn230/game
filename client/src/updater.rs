//! Система автоматичного оновлення гри через GitHub Releases

use std::fs;
use std::sync::{Arc, Mutex};
use std::thread;

/// Інформація про доступне оновлення
#[derive(Clone, Debug)]
pub struct UpdateInfo {
    pub version: String,
    pub download_url: String,
    pub sha256_url: String,
    pub changelog: String,
    pub size_mb: f32,
}

/// Стан оновлювача
pub struct Updater {
    pub current_version: String,
    pub update_info: Option<UpdateInfo>,
    pub checking: bool,
    pub downloading: bool,
    pub download_progress: Arc<Mutex<f32>>,
}

impl Updater {
    pub fn new(current_version: &str) -> Self {
        Self {
            current_version: current_version.to_string(),
            update_info: None,
            checking: false,
            downloading: false,
            download_progress: Arc::new(Mutex::new(0.0)),
        }
    }

    /// Перевірити наявність оновлень у фоновому потоці
    pub fn check_for_updates_async(&mut self) {
        if self.checking {
            return;
        }

        self.checking = true;
        let current_version = self.current_version.clone();

        thread::spawn(move || {
            match Self::fetch_github_release(&current_version) {
                Ok(Some(info)) => {
                    println!("✅ Знайдено оновлення: {}", info.version);
                    // TODO: передати info назад в основний потік
                }
                Ok(None) => {
                    println!("✅ Оновлень немає");
                }
                Err(e) => {
                    eprintln!("⚠️ Помилка перевірки оновлень: {}", e);
                }
            }
        });
    }

    /// Отримати інформацію про останній реліз з GitHub
    fn fetch_github_release(current_version: &str) -> Result<Option<UpdateInfo>, String> {
        let client = reqwest::blocking::Client::builder()
            .user_agent("PixelVillage-Game")
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| format!("HTTP клієнт: {}", e))?;

        let url = "https://api.github.com/repos/gertyn230/game/releases/latest";
        let response = client.get(url)
            .send()
            .map_err(|e| format!("GitHub API запит: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("GitHub API помилка: {}", response.status()));
        }

        let json: serde_json::Value = response.json()
            .map_err(|e| format!("Парсинг JSON: {}", e))?;

        let tag_name = json["tag_name"].as_str()
            .ok_or("Немає поля tag_name")?;
        let version = tag_name.trim_start_matches('v');

        // Перевірка чи нова версія новіша
        if !Self::is_newer_version(current_version, version) {
            return Ok(None);
        }

        // Знайти файл для Windows
        let assets = json["assets"].as_array()
            .ok_or("Немає assets")?;

        let asset = assets.iter()
            .find(|a| {
                a["name"].as_str()
                    .map(|n| n == "PixelVillage.exe")
                    .unwrap_or(false)
            })
            .ok_or("Не знайдено PixelVillage.exe")?;

        let download_url = asset["browser_download_url"].as_str()
            .ok_or("Немає download URL")?
            .to_string();

        let size = asset["size"].as_u64().unwrap_or(0);
        let size_mb = size as f32 / 1024.0 / 1024.0;

        // SHA256 файл має бути поряд
        let sha256_url = download_url.replace(".exe", ".exe.sha256");

        let changelog = json["body"].as_str().unwrap_or("").to_string();

        Ok(Some(UpdateInfo {
            version: version.to_string(),
            download_url,
            sha256_url,
            changelog,
            size_mb,
        }))
    }

    /// Порівняти версії (semver)
    fn is_newer_version(current: &str, new: &str) -> bool {
        match (semver::Version::parse(current), semver::Version::parse(new)) {
            (Ok(cur), Ok(new_ver)) => new_ver > cur,
            _ => {
                // Fallback на просте порівняння рядків
                new > current
            }
        }
    }

    /// Завантажити та встановити оновлення (Windows)
    #[cfg(target_os = "windows")]
    pub fn download_and_install(&mut self) -> Result<(), String> {
        if self.update_info.is_none() {
            return Err("Немає інформації про оновлення".to_string());
        }

        let info = self.update_info.as_ref().unwrap();
        self.downloading = true;

        let download_url = info.download_url.clone();
        let sha256_url = info.sha256_url.clone();
        let progress = Arc::clone(&self.download_progress);

        // Завантаження у фоновому потоці
        thread::spawn(move || {
            match Self::download_with_progress(&download_url, &progress) {
                Ok(bytes) => {
                    // Перевірка SHA256
                    match Self::download_sha256(&sha256_url) {
                        Ok(expected_hash) => {
                            let actual_hash = Self::calculate_sha256(&bytes);
                            if actual_hash != expected_hash {
                                eprintln!("❌ SHA256 не співпадає!");
                                return;
                            }

                            // Зберегти новий exe
                            if let Err(e) = Self::save_and_replace(bytes) {
                                eprintln!("❌ Помилка встановлення: {}", e);
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Не вдалось завантажити SHA256: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ Помилка завантаження: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Завантажити файл з прогрес-баром
    #[cfg(target_os = "windows")]
    fn download_with_progress(url: &str, progress: &Arc<Mutex<f32>>) -> Result<Vec<u8>, String> {
        use std::io::Read;

        let client = reqwest::blocking::Client::builder()
            .user_agent("PixelVillage-Updater")
            .build()
            .map_err(|e| format!("Помилка створення HTTP клієнта: {}", e))?;

        let mut response = client.get(url)
            .send()
            .map_err(|e| format!("Помилка HTTP запиту: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP помилка: {}", response.status()));
        }

        let total_size = response.content_length().unwrap_or(0);
        let mut bytes = Vec::new();
        let mut downloaded = 0u64;

        let mut buffer = [0u8; 8192];
        loop {
            match response.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    bytes.extend_from_slice(&buffer[..n]);
                    downloaded += n as u64;

                    if total_size > 0 {
                        let prog = downloaded as f32 / total_size as f32;
                        if let Ok(mut p) = progress.lock() {
                            *p = prog;
                        }
                    }
                }
                Err(e) => return Err(format!("Помилка читання: {}", e)),
            }
        }

        Ok(bytes)
    }

    /// Завантажити SHA256 контрольну суму
    #[cfg(target_os = "windows")]
    fn download_sha256(url: &str) -> Result<String, String> {
        let client = reqwest::blocking::Client::builder()
            .user_agent("PixelVillage-Updater")
            .build()
            .map_err(|e| format!("HTTP клієнт: {}", e))?;

        let text = client.get(url)
            .send()
            .map_err(|e| format!("HTTP запит: {}", e))?
            .text()
            .map_err(|e| format!("Читання тексту: {}", e))?;

        // SHA256 файл має формат: "<hash>  <filename>"
        Ok(text.split_whitespace().next().unwrap_or("").to_string())
    }

    /// Обчислити SHA256 хеш
    #[cfg(target_os = "windows")]
    fn calculate_sha256(data: &[u8]) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// Зберегти та замінити exe
    #[cfg(target_os = "windows")]
    fn save_and_replace(bytes: Vec<u8>) -> Result<(), String> {
        let current_exe = std::env::current_exe()
            .map_err(|e| format!("Не вдалось знайти поточний exe: {}", e))?;

        let parent = current_exe.parent()
            .ok_or("Не вдалось знайти батьківську папку")?;

        let new_exe = parent.join("PixelVillage.exe.new");

        // Зберегти новий файл
        fs::write(&new_exe, bytes)
            .map_err(|e| format!("Помилка запису файлу: {}", e))?;

        // Перейменувати поточний exe (Windows дозволяє це для запущеного процесу)
        let old_exe = parent.join("PixelVillage.exe.old");
        fs::rename(&current_exe, &old_exe)
            .map_err(|e| format!("Помилка перейменування поточного exe: {}", e))?;

        // Перейменувати новий на основну назву
        fs::rename(&new_exe, &current_exe)
            .map_err(|e| {
                // Rollback при помилці
                let _ = fs::rename(&old_exe, &current_exe);
                format!("Помилка встановлення нового exe: {}", e)
            })?;

        // Запустити новий процес
        std::process::Command::new(&current_exe)
            .spawn()
            .map_err(|e| format!("Помилка запуску оновленої гри: {}", e))?;

        // Завершити поточний процес
        std::process::exit(0);
    }

    /// Видалити старий backup при старті
    pub fn cleanup_old_version() {
        if let Ok(current_exe) = std::env::current_exe() {
            if let Some(parent) = current_exe.parent() {
                let old_exe = parent.join("PixelVillage.exe.old");
                let _ = fs::remove_file(&old_exe); // Ігноруємо помилки
            }
        }
    }

    /// Заглушка для Android
    #[cfg(target_os = "android")]
    pub fn download_and_install(&mut self) -> Result<(), String> {
        Err("Автооновлення на Android поки не реалізовано".to_string())
    }

    /// Заглушка для інших платформ
    #[cfg(not(any(target_os = "windows", target_os = "android")))]
    pub fn download_and_install(&mut self) -> Result<(), String> {
        Err("Автооновлення не підтримується на цій платформі".to_string())
    }

    /// Отримати прогрес завантаження (0.0 - 1.0)
    pub fn get_progress(&self) -> f32 {
        self.download_progress.lock().map(|p| *p).unwrap_or(0.0)
    }

    /// Отримати статус для UI
    pub fn get_status_text(&self) -> String {
        if self.checking {
            "Перевірка оновлень...".to_string()
        } else if self.downloading {
            let progress = self.get_progress();
            format!("⬇️ Завантаження... {:.0}%", progress * 100.0)
        } else if let Some(ref info) = self.update_info {
            format!("✨ Доступна версія {}", info.version)
        } else {
            String::new()
        }
    }
}
