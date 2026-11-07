//! Обгортка для управління станами гри з меню
use crate::menu::{Menu, MenuAction, MenuState};
use crate::settings::{Settings, save_settings};
use crate::i18n::get_texts;
use crate::updater::Updater;
use macroquad::prelude::*;

pub struct GameStateManager {
    pub menu: Menu,
    pub in_game: bool,
    pub should_quit: bool,
    pub updater: Updater,
}

impl GameStateManager {
    pub fn new(has_save: bool, current_version: &str) -> Self {
        let mut menu = Menu::new();
        menu.has_save = has_save;
        GameStateManager {
            menu,
            in_game: false,
            should_quit: false,
            updater: Updater::new(current_version),
        }
    }

    /// Перевірити оновлення і автоматично встановити (викликати після завантаження)
    pub async fn check_and_auto_update(&mut self) {
        // Перевірити наявність оновлень
        match self.updater.check_for_updates().await {
            Ok(has_update) => {
                if has_update {
                    // Автоматично завантажити і встановити
                    let _ = self.updater.download_and_install().await;
                }
            }
            Err(_) => {
                // Тихо ігноруємо помилки перевірки оновлень
            }
        }
    }

    /// Показати статус оновлення внизу екрану (якщо завантажується)
    pub fn draw_update_status(&self, font: &Font) {
        let status = self.updater.get_status_text();
        if !status.is_empty() {
            let (sw, sh) = (screen_width(), screen_height());

            // Малий бар внизу екрану
            let bar_w = 300.0;
            let bar_h = 40.0;
            let bar_x = (sw - bar_w) / 2.0;
            let bar_y = sh - 60.0;

            draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color::from_rgba(30, 30, 40, 200));
            draw_rectangle_lines(bar_x, bar_y, bar_w, bar_h, 2.0, Color::from_rgba(100, 150, 255, 255));

            draw_text_ex(
                &status,
                bar_x + 10.0,
                bar_y + 26.0,
                TextParams {
                    font: Some(font),
                    font_size: 16,
                    color: WHITE,
                    ..Default::default()
                },
            );
        }
    }

    pub fn update(&mut self, font: &Font, settings: &mut Settings) -> bool {
        let texts = get_texts(&settings.language);

        match self.menu.state {
            MenuState::Loading => {
                self.menu.draw_loading(font, texts);
                false
            }
            MenuState::MainMenu => {
                if let Some(action) = self.menu.draw_main_menu(font, texts) {
                    match action {
                        MenuAction::Play | MenuAction::Continue => {
                            self.menu.state = MenuState::InGame;
                            self.in_game = true;
                            return true;
                        }
                        MenuAction::NewGame => {
                            self.menu.state = MenuState::InGame;
                            self.in_game = true;
                            return true;
                        }
                        MenuAction::OpenSettings => {
                            self.menu.state = MenuState::Settings;
                        }
                        MenuAction::Quit => {
                            self.should_quit = true;
                        }
                        _ => {}
                    }
                }
                false
            }
            MenuState::Settings => {
                if let Some(action) = self.menu.draw_settings_menu(font, texts, settings) {
                    if action == MenuAction::CloseSettings {
                        self.menu.state = MenuState::MainMenu;
                        save_settings(settings);
                    }
                }
                false
            }
            MenuState::InGame => true,
        }
    }

    pub fn show_menu(&mut self) {
        self.menu.state = MenuState::MainMenu;
        self.in_game = false;
    }

    pub fn update_loading(&mut self, progress: f32, message: &str) {
        self.menu.update_loading(progress, message);
    }

    pub fn finish_loading(&mut self) {
        self.menu.finish_loading();
    }
}
