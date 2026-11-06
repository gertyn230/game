//! Система автоматичного оновлення гри

use std::fs;
use std::path::PathBuf;

pub struct Updater {
    pub current_version: String,
    pub latest_version: Option<String>,
    pub update_available: bool,
    pub download_url: Option<String>,
    pub changelog: Vec<String>,
}

impl Updater {
    pub fn new(current_version: &str) -> Self {
        Self {
            current_version: current_version.to_string(),
            latest_version: None,
            update_available: false,
            download_url: None,
            changelog: vec![],
        }
    }

    /// Перевірити наявність оновлень через GitHub API
    pub async fn check_for_updates(&mut self) -> Result<bool, String> {
        // Запит до GitHub API для перевірки останнього релізу
        let url = "https://api.github.com/repos/gertyn230/game/releases/latest";

        match self.fetch_latest_release(url).await {
            Ok((version, download_url, changelog)) => {
                self.latest_version = Some(version.clone());
                self.download_url = Some(download_url);
                self.changelog = changelog;
                self.update_available = self.is_newer_version(&version);
                Ok(self.update_available)
            }
            Err(e) => Err(format!("Помилка перевірки оновлень: {}", e)),
        }
    }

    /// Завантажити та встановити оновлення
    pub async fn download_and_install(&self) -> Result<(), String> {
        if !self.update_available {
            return Err("Немає доступних оновлень".to_string());
        }

        let download_url = self.download_url.as_ref()
            .ok_or("Відсутнє посилання для завантаження")?;

        // Визначити шлях для завантаження
        let temp_path = self.get_temp_download_path()?;

        // Завантажити файл
        self.download_file(download_url, &temp_path).await?;

        // Встановити оновлення
        self.install_update(&temp_path)?;

        Ok(())
    }

    /// Отримати інформацію про останній реліз з GitHub
    async fn fetch_latest_release(&self, url: &str) -> Result<(String, String, Vec<String>), String> {
        // Для Windows використовуємо reqwest або інший HTTP клієнт
        // Тут заглушка - потрібно додати залежність reqwest в Cargo.toml

        // Поки що повертаємо дані з локального API сервера
        let local_url = "http://127.0.0.1:8080/api/version";

        match self.fetch_json(local_url).await {
            Ok(data) => {
                let version = data.get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("1.2.0")
                    .to_string();

                let download = data.get("download")
                    .and_then(|v| v.as_str())
                    .unwrap_or("https://github.com/gertyn230/game/releases/latest")
                    .to_string();

                let changes = data.get("changes")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect())
                    .unwrap_or_default();

                Ok((version, download, changes))
            }
            Err(e) => Err(e),
        }
    }

    /// Завантажити JSON з URL
    async fn fetch_json(&self, url: &str) -> Result<serde_json::Value, String> {
        // Тут має бути реальний HTTP запит
        // Поки що заглушка
        Err("HTTP клієнт не реалізований".to_string())
    }

    /// Завантажити файл з URL
    async fn download_file(&self, url: &str, path: &PathBuf) -> Result<(), String> {
        // Тут має бути завантаження файлу
        Err("Завантаження не реалізоване".to_string())
    }

    /// Встановити оновлення
    fn install_update(&self, update_path: &PathBuf) -> Result<(), String> {
        // Логіка встановлення оновлення
        // 1. Зберегти поточний exe як backup
        // 2. Замінити поточний exe на новий
        // 3. Перезапустити гру

        let current_exe = std::env::current_exe()
            .map_err(|e| format!("Не вдалося знайти поточний exe: {}", e))?;

        let backup_path = current_exe.with_extension("exe.backup");

        // Створити резервну копію
        fs::copy(&current_exe, &backup_path)
            .map_err(|e| format!("Помилка створення backup: {}", e))?;

        // Замінити exe (потрібні права адміністратора на Windows)
        fs::copy(update_path, &current_exe)
            .map_err(|e| format!("Помилка заміни exe: {}", e))?;

        Ok(())
    }

    /// Отримати шлях для тимчасового завантаження
    fn get_temp_download_path(&self) -> Result<PathBuf, String> {
        let temp_dir = std::env::temp_dir();
        Ok(temp_dir.join("pixel_village_update.exe"))
    }

    /// Перевірити чи нова версія новіша за поточну
    fn is_newer_version(&self, new_version: &str) -> bool {
        // Проста перевірка версій (1.2.0 > 1.1.0)
        let current_parts: Vec<u32> = self.current_version
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

    /// Отримати повідомлення про оновлення
    pub fn get_update_message(&self) -> String {
        if !self.update_available {
            return String::new();
        }

        let version = self.latest_version.as_ref().unwrap();
        let mut msg = format!("🎮 Доступна нова версія {}!\n\n", version);

        if !self.changelog.is_empty() {
            msg.push_str("📝 Що нового:\n");
            for change in &self.changelog {
                msg.push_str(&format!("  • {}\n", change));
            }
        }

        msg.push_str("\n💾 Натисніть U для автоматичного оновлення");
        msg
    }
}
