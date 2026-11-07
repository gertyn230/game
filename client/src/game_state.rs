//! Обгортка для управління станами гри з меню
use crate::menu::{Menu, MenuAction, MenuState};
use crate::settings::{Settings, save_settings};
use crate::i18n::get_texts;
use crate::updater::Updater;
use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum UpdateAction {
    Install,
    Later,
}

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

    /// Перевірити оновлення у фоновому режимі (викликати після завантаження)
    pub fn check_updates_in_background(&mut self) {
        self.updater.check_for_updates_async();
    }

    /// Чи є доступне оновлення
    pub fn has_update(&self) -> bool {
        self.updater.update_info.is_some()
    }

    /// Отримати інформацію про оновлення
    pub fn get_update_info(&self) -> Option<&crate::updater::UpdateInfo> {
        self.updater.update_info.as_ref()
    }

    /// Встановити оновлення (показати UI з кнопкою)
    pub fn install_update(&mut self) -> Result<(), String> {
        self.updater.download_and_install()
    }

    /// Показати вікно оновлення з кнопками
    pub fn draw_update_window(&mut self, font: &Font) -> Option<UpdateAction> {
        if self.updater.update_info.is_none() {
            return None;
        }

        let (sw, sh) = (screen_width(), screen_height());

        // Напівпрозорий фон
        draw_rectangle(0.0, 0.0, sw, sh, Color::from_rgba(0, 0, 0, 180));

        // Вікно
        let w = 600.0;
        let h = 400.0;
        let x = (sw - w) / 2.0;
        let y = (sh - h) / 2.0;

        draw_rectangle(x, y, w, h, Color::from_rgba(30, 30, 40, 255));
        draw_rectangle_lines(x, y, w, h, 3.0, Color::from_rgba(100, 150, 255, 255));

        let info = self.updater.update_info.as_ref().unwrap();

        // Заголовок
        let title = format!("Доступна версія {}", info.version);
        draw_text_ex(
            &title,
            x + 20.0,
            y + 40.0,
            TextParams {
                font: Some(font),
                font_size: 28,
                color: Color::from_rgba(100, 200, 255, 255),
                ..Default::default()
            },
        );

        // Розмір
        let size_text = format!("Розмір: {:.1} МБ", info.size_mb);
        draw_text_ex(
            &size_text,
            x + 20.0,
            y + 80.0,
            TextParams {
                font: Some(font),
                font_size: 18,
                color: WHITE,
                ..Default::default()
            },
        );

        // Changelog (перші 5 рядків)
        let changelog_lines: Vec<&str> = info.changelog.lines().take(5).collect();
        for (i, line) in changelog_lines.iter().enumerate() {
            draw_text_ex(
                line,
                x + 20.0,
                y + 120.0 + i as f32 * 25.0,
                TextParams {
                    font: Some(font),
                    font_size: 16,
                    color: Color::from_rgba(200, 200, 200, 255),
                    ..Default::default()
                },
            );
        }

        // Статус завантаження
        if self.updater.downloading {
            let progress = self.updater.get_progress();
            let bar_y = y + h - 100.0;
            let bar_w = w - 40.0;
            let bar_h = 30.0;

            draw_rectangle(x + 20.0, bar_y, bar_w, bar_h, Color::from_rgba(50, 50, 60, 255));
            draw_rectangle(x + 20.0, bar_y, bar_w * progress, bar_h, Color::from_rgba(100, 200, 100, 255));

            let progress_text = format!("Завантаження... {:.0}%", progress * 100.0);
            draw_text_ex(
                &progress_text,
                x + 20.0,
                bar_y - 10.0,
                TextParams {
                    font: Some(font),
                    font_size: 18,
                    color: WHITE,
                    ..Default::default()
                },
            );
        } else {
            // Кнопки
            let btn_w = 200.0;
            let btn_h = 50.0;
            let btn_y = y + h - 80.0;
            let install_x = x + (w - btn_w * 2.0 - 20.0) / 2.0;
            let later_x = install_x + btn_w + 20.0;

            // Кнопка "Встановити"
            let mouse_pos = macroquad::input::mouse_position();
            let mouse_on_install = mouse_pos.0 >= install_x && mouse_pos.0 <= install_x + btn_w
                && mouse_pos.1 >= btn_y && mouse_pos.1 <= btn_y + btn_h;

            let install_color = if mouse_on_install {
                Color::from_rgba(80, 180, 80, 255)
            } else {
                Color::from_rgba(60, 140, 60, 255)
            };

            draw_rectangle(install_x, btn_y, btn_w, btn_h, install_color);
            draw_rectangle_lines(install_x, btn_y, btn_w, btn_h, 2.0, WHITE);
            draw_text_ex(
                "Встановити",
                install_x + 45.0,
                btn_y + 33.0,
                TextParams {
                    font: Some(font),
                    font_size: 20,
                    color: WHITE,
                    ..Default::default()
                },
            );

            if mouse_on_install && macroquad::input::is_mouse_button_pressed(macroquad::input::MouseButton::Left) {
                return Some(UpdateAction::Install);
            }

            // Кнопка "Пізніше"
            let mouse_on_later = mouse_pos.0 >= later_x && mouse_pos.0 <= later_x + btn_w
                && mouse_pos.1 >= btn_y && mouse_pos.1 <= btn_y + btn_h;

            let later_color = if mouse_on_later {
                Color::from_rgba(100, 100, 120, 255)
            } else {
                Color::from_rgba(70, 70, 90, 255)
            };

            draw_rectangle(later_x, btn_y, btn_w, btn_h, later_color);
            draw_rectangle_lines(later_x, btn_y, btn_w, btn_h, 2.0, WHITE);
            draw_text_ex(
                "Пізніше",
                later_x + 60.0,
                btn_y + 33.0,
                TextParams {
                    font: Some(font),
                    font_size: 20,
                    color: WHITE,
                    ..Default::default()
                },
            );

            if mouse_on_later && macroquad::input::is_mouse_button_pressed(macroquad::input::MouseButton::Left) {
                return Some(UpdateAction::Later);
            }
        }

        None
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
