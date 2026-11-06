//! Launcher для Pixel Village з автоматичним оновленням

use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

const CURRENT_VERSION: &str = "1.2.0";
const GITHUB_API: &str = "https://api.github.com/repos/gertyn230/game/releases/latest";
const GAME_EXE: &str = "PixelVillage.exe";

#[derive(Deserialize, Serialize, Debug)]
struct GitHubRelease {
    tag_name: String,
    name: String,
    body: String,
    assets: Vec<Asset>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Asset {
    name: String,
    browser_download_url: String,
    size: u64,
}

#[derive(Clone)]
enum LauncherState {
    CheckingUpdate,
    UpdateAvailable(String, String, u64), // version, url, size
    Downloading(f32), // progress 0.0-1.0
    Installing,
    Ready,
    Error(String),
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Pixel Village Launcher".to_owned(),
        window_width: 800,
        window_height: 450,
        window_resizable: false,
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut state = LauncherState::CheckingUpdate;
    let font = load_ttf_font("assets/font.ttf").await.ok();

    let progress = Arc::new(Mutex::new(0.0f32));
    let mut download_handle: Option<thread::JoinHandle<Result<PathBuf, String>>> = None;

    // Перевірка оновлень
    match check_for_updates().await {
        Ok(Some((version, download_url, size))) => {
            state = LauncherState::UpdateAvailable(version, download_url, size);
        }
        Ok(None) => {
            state = LauncherState::Ready;
        }
        Err(e) => {
            state = LauncherState::Error(format!("Помилка перевірки: {}", e));
        }
    }

    loop {
        clear_background(Color::from_rgba(30, 35, 40, 255));

        let (sw, sh) = (screen_width(), screen_height());
        let cx = sw / 2.0;
        let cy = sh / 2.0;

        // Заголовок
        draw_text_ex(
            "🎮 PIXEL VILLAGE",
            cx - 200.0,
            100.0,
            TextParams {
                font: font.as_ref(),
                font_size: 48,
                color: Color::from_rgba(255, 255, 255, 255),
                ..Default::default()
            },
        );

        // Версія
        let version_text = format!("v{}", CURRENT_VERSION);
        draw_text_ex(
            &version_text,
            cx - 30.0,
            140.0,
            TextParams {
                font: font.as_ref(),
                font_size: 20,
                color: Color::from_rgba(150, 150, 150, 255),
                ..Default::default()
            },
        );

        // Стан
        match &state {
            LauncherState::CheckingUpdate => {
                draw_text_ex(
                    "🔍 Перевірка оновлень...",
                    cx - 150.0,
                    cy,
                    TextParams {
                        font: font.as_ref(),
                        font_size: 24,
                        color: WHITE,
                        ..Default::default()
                    },
                );
            }
            LauncherState::UpdateAvailable(version, url, size) => {
                draw_text_ex(
                    &format!("✨ Доступна нова версія {}!", version),
                    cx - 200.0,
                    cy - 60.0,
                    TextParams {
                        font: font.as_ref(),
                        font_size: 28,
                        color: Color::from_rgba(255, 215, 0, 255),
                        ..Default::default()
                    },
                );

                let size_mb = *size as f32 / 1024.0 / 1024.0;
                draw_text_ex(
                    &format!("Розмір: {:.1} MB", size_mb),
                    cx - 80.0,
                    cy - 20.0,
                    TextParams {
                        font: font.as_ref(),
                        font_size: 20,
                        color: Color::from_rgba(200, 200, 200, 255),
                        ..Default::default()
                    },
                );

                // Кнопка завантаження
                let button_rect = Rect::new(cx - 120.0, cy + 20.0, 240.0, 50.0);
                let mouse_pos = mouse_position();
                let is_hover = button_rect.contains(Vec2::new(mouse_pos.0, mouse_pos.1));

                let button_color = if is_hover {
                    Color::from_rgba(70, 150, 70, 255)
                } else {
                    Color::from_rgba(50, 130, 50, 255)
                };

                draw_rectangle(button_rect.x, button_rect.y, button_rect.w, button_rect.h, button_color);
                draw_rectangle_lines(button_rect.x, button_rect.y, button_rect.w, button_rect.h, 2.0, WHITE);
                draw_text_ex(
                    "📥 ОНОВИТИ",
                    cx - 75.0,
                    cy + 55.0,
                    TextParams {
                        font: font.as_ref(),
                        font_size: 24,
                        color: WHITE,
                        ..Default::default()
                    },
                );

                // Кнопка пропустити
                let skip_rect = Rect::new(cx - 100.0, cy + 80.0, 200.0, 40.0);
                let skip_hover = skip_rect.contains(Vec2::new(mouse_pos.0, mouse_pos.1));

                draw_text_ex(
                    "Запустити без оновлення",
                    cx - 95.0,
                    cy + 110.0,
                    TextParams {
                        font: font.as_ref(),
                        font_size: 18,
                        color: if skip_hover { Color::from_rgba(255, 200, 100, 255) } else { Color::from_rgba(150, 150, 150, 255) },
                        ..Default::default()
                    },
                );

                if skip_hover && is_mouse_button_pressed(MouseButton::Left) {
                    launch_game();
                    break;
                }

                // Натиснули на кнопку завантаження
                if is_hover && is_mouse_button_pressed(MouseButton::Left) {
                    // Клонувати дані перед зміною state
                    let url_clone = url.clone();
                    let progress_clone = Arc::clone(&progress);

                    download_handle = Some(thread::spawn(move || {
                        download_update(&url_clone, &progress_clone)
                    }));

                    state = LauncherState::Downloading(0.0);
                }
            }
            LauncherState::Downloading(_) => {
                draw_text_ex(
                    "📥 Завантаження оновлення...",
                    cx - 180.0,
                    cy - 40.0,
                    TextParams {
                        font: font.as_ref(),
                        font_size: 24,
                        color: WHITE,
                        ..Default::default()
                    },
                );

                // Оновити прогрес з потоку
                let current_progress = *progress.lock().unwrap();

                // Прогрес бар
                let bar_width = 400.0;
                let bar_height = 40.0;
                let bar_x = cx - bar_width / 2.0;
                let bar_y = cy;

                draw_rectangle(bar_x, bar_y, bar_width, bar_height, Color::from_rgba(50, 50, 50, 255));
                draw_rectangle(bar_x, bar_y, bar_width * current_progress, bar_height, Color::from_rgba(70, 150, 255, 255));
                draw_rectangle_lines(bar_x, bar_y, bar_width, bar_height, 2.0, WHITE);

                let percent_text = format!("{:.0}%", current_progress * 100.0);
                draw_text_ex(
                    &percent_text,
                    cx - 20.0,
                    cy + 28.0,
                    TextParams {
                        font: font.as_ref(),
                        font_size: 20,
                        color: WHITE,
                        ..Default::default()
                    },
                );

                // Перевірити чи завантаження завершилось
                if let Some(ref handle) = download_handle {
                    if handle.is_finished() {
                        let handle = download_handle.take().unwrap();
                        match handle.join().unwrap() {
                            Ok(temp_path) => {
                                state = LauncherState::Installing;
                                // Встановити оновлення
                                match install_update(&temp_path) {
                                    Ok(_) => state = LauncherState::Ready,
                                    Err(e) => state = LauncherState::Error(format!("Помилка встановлення: {}", e)),
                                }
                            }
                            Err(e) => {
                                state = LauncherState::Error(format!("Помилка завантаження: {}", e));
                            }
                        }
                    }
                }
            }
            LauncherState::Installing => {
                draw_text_ex(
                    "⚙️ Встановлення оновлення...",
                    cx - 180.0,
                    cy,
                    TextParams {
                        font: font.as_ref(),
                        font_size: 24,
                        color: WHITE,
                        ..Default::default()
                    },
                );
            }
            LauncherState::Ready => {
                draw_text_ex(
                    "✅ Гра готова до запуску!",
                    cx - 160.0,
                    cy - 40.0,
                    TextParams {
                        font: font.as_ref(),
                        font_size: 24,
                        color: Color::from_rgba(100, 255, 100, 255),
                        ..Default::default()
                    },
                );

                // Кнопка запуску
                let button_rect = Rect::new(cx - 100.0, cy + 20.0, 200.0, 60.0);
                let mouse_pos = mouse_position();
                let is_hover = button_rect.contains(Vec2::new(mouse_pos.0, mouse_pos.1));

                let button_color = if is_hover {
                    Color::from_rgba(70, 150, 255, 255)
                } else {
                    Color::from_rgba(50, 100, 200, 255)
                };

                draw_rectangle(button_rect.x, button_rect.y, button_rect.w, button_rect.h, button_color);
                draw_rectangle_lines(button_rect.x, button_rect.y, button_rect.w, button_rect.h, 3.0, WHITE);
                draw_text_ex(
                    "▶️ ГРАТИ",
                    cx - 50.0,
                    cy + 60.0,
                    TextParams {
                        font: font.as_ref(),
                        font_size: 28,
                        color: WHITE,
                        ..Default::default()
                    },
                );

                if is_hover && is_mouse_button_pressed(MouseButton::Left) {
                    launch_game();
                    break;
                }
            }
            LauncherState::Error(msg) => {
                draw_text_ex(
                    "❌ Помилка:",
                    cx - 80.0,
                    cy - 40.0,
                    TextParams {
                        font: font.as_ref(),
                        font_size: 24,
                        color: Color::from_rgba(255, 100, 100, 255),
                        ..Default::default()
                    },
                );
                draw_text_ex(
                    msg,
                    cx - 200.0,
                    cy,
                    TextParams {
                        font: font.as_ref(),
                        font_size: 18,
                        color: WHITE,
                        ..Default::default()
                    },
                );

                draw_text_ex(
                    "Натисніть для запуску гри",
                    cx - 120.0,
                    cy + 50.0,
                    TextParams {
                        font: font.as_ref(),
                        font_size: 16,
                        color: Color::from_rgba(200, 200, 200, 255),
                        ..Default::default()
                    },
                );

                if is_mouse_button_pressed(MouseButton::Left) {
                    launch_game();
                    break;
                }
            }
        }

        // Копірайт
        draw_text_ex(
            "© 2026 Pixel Village",
            cx - 80.0,
            sh - 30.0,
            TextParams {
                font: font.as_ref(),
                font_size: 16,
                color: Color::from_rgba(100, 100, 100, 255),
                ..Default::default()
            },
        );

        next_frame().await;
    }
}

async fn check_for_updates() -> Result<Option<(String, String, u64)>, String> {
    let client = reqwest::Client::builder()
        .user_agent("PixelVillage-Launcher/1.2.0")
        .build()
        .map_err(|e| format!("HTTP error: {}", e))?;

    let response = client.get(GITHUB_API)
        .send()
        .await
        .map_err(|e| format!("Request error: {}", e))?;

    if !response.status().is_success() {
        return Err("GitHub API не відповідає".to_string());
    }

    let release: GitHubRelease = response.json()
        .await
        .map_err(|e| format!("JSON parse error: {}", e))?;

    let latest_version = release.tag_name.trim_start_matches('v');

    if is_newer_version(latest_version, CURRENT_VERSION) {
        if let Some(asset) = release.assets.iter().find(|a| a.name.ends_with(".exe")) {
            return Ok(Some((
                latest_version.to_string(),
                asset.browser_download_url.clone(),
                asset.size
            )));
        }
    }

    Ok(None)
}

fn download_update(url: &str, progress: &Arc<Mutex<f32>>) -> Result<PathBuf, String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("PixelVillage-Launcher/1.2.0")
        .build()
        .map_err(|e| format!("HTTP error: {}", e))?;

    let mut response = client.get(url)
        .send()
        .map_err(|e| format!("Download error: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    let total_size = response.content_length().unwrap_or(0);
    let temp_path = std::env::temp_dir().join("pixel_village_update.exe");

    let mut file = fs::File::create(&temp_path)
        .map_err(|e| format!("File create error: {}", e))?;

    let mut downloaded = 0u64;
    let mut buffer = vec![0; 8192];

    loop {
        match response.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                file.write_all(&buffer[..n])
                    .map_err(|e| format!("Write error: {}", e))?;

                downloaded += n as u64;

                if total_size > 0 {
                    let prog = downloaded as f32 / total_size as f32;
                    *progress.lock().unwrap() = prog;
                }
            }
            Err(e) => return Err(format!("Read error: {}", e)),
        }
    }

    file.flush().map_err(|e| format!("Flush error: {}", e))?;
    *progress.lock().unwrap() = 1.0;

    Ok(temp_path)
}

fn install_update(temp_path: &PathBuf) -> Result<(), String> {
    let game_exe = get_game_exe_path();

    // Створити backup
    let backup_path = game_exe.with_extension("exe.backup");
    if game_exe.exists() {
        fs::copy(&game_exe, &backup_path)
            .map_err(|e| format!("Backup error: {}", e))?;
    }

    // Замінити exe
    fs::copy(temp_path, &game_exe)
        .map_err(|e| {
            // Спроба відновити з backup
            if backup_path.exists() {
                let _ = fs::copy(&backup_path, &game_exe);
            }
            format!("Install error: {}", e)
        })?;

    // Видалити тимчасові файли
    let _ = fs::remove_file(temp_path);
    let _ = fs::remove_file(&backup_path);

    Ok(())
}

fn is_newer_version(new: &str, current: &str) -> bool {
    let new_parts: Vec<u32> = new.split('.').filter_map(|s| s.parse().ok()).collect();
    let cur_parts: Vec<u32> = current.split('.').filter_map(|s| s.parse().ok()).collect();

    for i in 0..3 {
        let n = new_parts.get(i).unwrap_or(&0);
        let c = cur_parts.get(i).unwrap_or(&0);
        if n > c {
            return true;
        } else if n < c {
            return false;
        }
    }
    false
}

fn launch_game() {
    let exe_path = get_game_exe_path();
    if exe_path.exists() {
        Command::new(exe_path)
            .spawn()
            .expect("Не вдалося запустити гру");
    }
}

fn get_game_exe_path() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop();
    path.push(GAME_EXE);
    path
}
