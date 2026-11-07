//! Система автоматичного оновлення гри через GitHub Releases

use std::fs::{self, File};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

/// Стан оновлювача
#[derive(Clone, Debug, PartialEq)]
pub enum UpdateState {
    Idle,
    Checking,
    Available(UpdateInfo),
    Downloading(f32), // прогрес 0.0-1.0
    Installing,
    Failed(String),
}

/// Інформація про доступне оновлення
#[derive(Clone, Debug, PartialEq)]
pub struct UpdateInfo {
    pub version: String,
    pub sha256: String,
    pub size: u64,
    pub notes: String,
}

pub struct Updater {
    pub current_version: String,
    pub state: Arc<Mutex<UpdateState>>,
    base_url: String,
}

impl Updater {
    pub fn new(current_version: &str) -> Self {
        #[cfg(debug_assertions)]
        let base_url = std::env::var("UPDATE_BASE_URL")
            .unwrap_or_else(|_| "https://github.com/gertyn230/game-releases/releases/latest/download".to_string());

        #[cfg(not(debug_assertions))]
        let base_url = "https://github.com/gertyn230/game-releases/releases/latest/download".to_string();

        Self {
            current_version: current_version.to_string(),
            state: Arc::new(Mutex::new(UpdateState::Idle)),
            base_url,
        }
    }

    /// Перевірити оновлення у фоновому потоці (викликати один раз)
    pub fn check_for_updates(&self) {
        let state = Arc::clone(&self.state);
        {
            let current = state.lock().unwrap();
            if *current != UpdateState::Idle {
                return; // вже перевіряємо або є результат
            }
        }

        *state.lock().unwrap() = UpdateState::Checking;

        let current_version = self.current_version.clone();
        let base_url = self.base_url.clone();

        thread::spawn(move || {
            match Self::fetch_update_info(&base_url, &current_version) {
                Ok(Some(info)) => {
                    *state.lock().unwrap() = UpdateState::Available(info);
                }
                Ok(None) => {
                    *state.lock().unwrap() = UpdateState::Idle;
                }
                Err(e) => {
                    eprintln!("⚠️ Update check failed: {}", e);
                    *state.lock().unwrap() = UpdateState::Idle; // тихо, без паніки
                }
            }
        });
    }

    fn fetch_update_info(base_url: &str, current_version: &str) -> Result<Option<UpdateInfo>, String> {
        let client = reqwest::blocking::Client::builder()
            .user_agent("PixelVillage-Game")
            .timeout(std::time::Duration::from_secs(10))
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
            .map_err(|e| format!("HTTP client: {}", e))?;

        let url = format!("{}/latest.json", base_url);
        let response = client.get(&url)
            .send()
            .map_err(|e| format!("HTTP request: {}", e))?;

        if !response.status().is_success() {
            return if response.status().as_u16() == 404 {
                Ok(None) // немає оновлень
            } else {
                Err(format!("HTTP {}", response.status()))
            };
        }

        #[derive(serde::Deserialize)]
        struct LatestJson {
            version: String,
            sha256: String,
            size: u64,
            notes: String,
        }

        let json: LatestJson = response.json()
            .map_err(|e| format!("Invalid JSON: {}", e))?;

        // Порівняти версії
        if !Self::is_newer_version(current_version, &json.version) {
            return Ok(None);
        }

        Ok(Some(UpdateInfo {
            version: json.version,
            sha256: json.sha256.to_lowercase(),
            size: json.size,
            notes: json.notes,
        }))
    }

    fn is_newer_version(current: &str, new: &str) -> bool {
        match (semver::Version::parse(current), semver::Version::parse(new)) {
            (Ok(cur), Ok(new_ver)) => new_ver > cur,
            _ => new > current, // fallback
        }
    }

    /// Завантажити та встановити оновлення (Windows)
    #[cfg(target_os = "windows")]
    pub fn download_and_install(&self) -> Result<(), String> {
        let info = {
            let state = self.state.lock().unwrap();
            match &*state {
                UpdateState::Available(info) => info.clone(),
                _ => return Err("No update available".to_string()),
            }
        };

        *self.state.lock().unwrap() = UpdateState::Downloading(0.0);

        let url = format!("{}/PixelVillage.exe", self.base_url);
        let temp_path = std::env::temp_dir().join("PixelVillage.exe.new");

        // Завантажити потоково на диск
        let client = reqwest::blocking::Client::builder()
            .user_agent("PixelVillage-Game")
            .timeout(std::time::Duration::from_secs(300))
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
            .map_err(|e| format!("HTTP client: {}", e))?;

        let mut response = client.get(&url)
            .send()
            .map_err(|e| format!("Download failed: {}", e))?;

        if !response.status().is_success() {
            *self.state.lock().unwrap() = UpdateState::Failed(format!("HTTP {}", response.status()));
            return Err(format!("HTTP {}", response.status()));
        }

        let total_size = response.content_length().unwrap_or(info.size);
        let mut file = File::create(&temp_path)
            .map_err(|e| format!("Create temp file: {}", e))?;

        let mut downloaded = 0u64;
        let mut buffer = [0u8; 8192];
        let state = Arc::clone(&self.state);

        loop {
            match response.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    file.write_all(&buffer[..n])
                        .map_err(|e| format!("Write error: {}", e))?;
                    downloaded += n as u64;
                    let progress = if total_size > 0 {
                        (downloaded as f32 / total_size as f32).min(1.0)
                    } else {
                        0.0
                    };
                    *state.lock().unwrap() = UpdateState::Downloading(progress);
                }
                Err(e) => {
                    let _ = fs::remove_file(&temp_path);
                    *self.state.lock().unwrap() = UpdateState::Failed(format!("Download error: {}", e));
                    return Err(format!("Download error: {}", e));
                }
            }
        }

        drop(file); // закрити файл

        // Перевірити розмір
        let actual_size = fs::metadata(&temp_path)
            .map_err(|e| format!("Stat file: {}", e))?
            .len();

        if actual_size != info.size {
            let _ = fs::remove_file(&temp_path);
            *self.state.lock().unwrap() = UpdateState::Failed(format!("Size mismatch: {} != {}", actual_size, info.size));
            return Err(format!("Size mismatch"));
        }

        // Перевірити SHA256
        *self.state.lock().unwrap() = UpdateState::Installing;

        let actual_hash = Self::calculate_sha256(&temp_path)?;
        if actual_hash != info.sha256 {
            let _ = fs::remove_file(&temp_path);
            *self.state.lock().unwrap() = UpdateState::Failed("SHA256 mismatch".to_string());
            return Err("SHA256 mismatch".to_string());
        }

        // Замінити exe
        Self::replace_exe(&temp_path)?;

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn calculate_sha256(path: &std::path::Path) -> Result<String, String> {
        use sha2::{Sha256, Digest};

        let mut file = File::open(path)
            .map_err(|e| format!("Open file: {}", e))?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            match file.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => hasher.update(&buffer[..n]),
                Err(e) => return Err(format!("Read error: {}", e)),
            }
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    #[cfg(target_os = "windows")]
    fn replace_exe(new_exe_path: &std::path::Path) -> Result<(), String> {
        let current_exe = std::env::current_exe()
            .map_err(|e| format!("Get current exe: {}", e))?;

        let parent = current_exe.parent()
            .ok_or("No parent directory")?;

        let old_exe = parent.join("PixelVillage.exe.old");

        // Перейменувати поточний → .old
        fs::rename(&current_exe, &old_exe)
            .map_err(|e| format!("Rename current: {}", e))?;

        // Перейменувати new → поточний
        match fs::rename(new_exe_path, &current_exe) {
            Ok(_) => {}
            Err(e) => {
                // Rollback
                let _ = fs::rename(&old_exe, &current_exe);
                return Err(format!("Rename new: {}", e));
            }
        }

        // Запустити новий процес
        std::process::Command::new(&current_exe)
            .spawn()
            .map_err(|e| format!("Start new process: {}", e))?;

        // Завершити поточний
        std::process::exit(0);
    }

    #[cfg(not(target_os = "windows"))]
    pub fn download_and_install(&self) -> Result<(), String> {
        Err("Auto-update not supported on this platform".to_string())
    }

    /// Видалити старий backup при старті
    pub fn cleanup_old_version() {
        if let Ok(current_exe) = std::env::current_exe() {
            if let Some(parent) = current_exe.parent() {
                let old_exe = parent.join("PixelVillage.exe.old");
                let _ = fs::remove_file(&old_exe);
            }
        }
    }

    pub fn get_state(&self) -> UpdateState {
        self.state.lock().unwrap().clone()
    }

    pub fn retry(&self) {
        *self.state.lock().unwrap() = UpdateState::Idle;
        self.check_for_updates();
    }

    pub fn dismiss(&self) {
        *self.state.lock().unwrap() = UpdateState::Idle;
    }
}
