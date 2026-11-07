//! Обгортка для управління станами гри з меню
use crate::menu::{Menu, MenuAction, MenuState};
use crate::settings::{Settings, save_settings};
use crate::i18n::get_texts;
use crate::updater::{Updater, UpdateState, UpdateInfo};
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

    /// Перевірити оновлення у фоновому режимі (викликати один раз)
    pub fn check_updates_once(&mut self) {
        self.updater.check_for_updates();
    }

    /// Показати вікно оновлення з кнопками
    pub fn draw_update_window(&mut self, font: &Font) {
        let state = self.updater.get_state();

        match state {
            UpdateState::Available(ref info) => {
                self.draw_available_window(font, info);
            }
            UpdateState::Downloading(progress) => {
                self.draw_downloading_window(font, progress);
            }
            UpdateState::Installing => {
                self.draw_installing_window(font);
            }
            UpdateState::Failed(ref err) => {
                self.draw_failed_window(font, err);
            }
            _ => {}
        }
    }

    fn draw_available_window(&mut self, font: &Font, info: &UpdateInfo) {
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
        let size_mb = info.size as f32 / 1024.0 / 1024.0;
        let size_text = format!("Розмір: {:.1} МБ", size_mb);
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

        // Notes (перші 5 рядків)
        let lines: Vec<&str> = info.notes.lines().take(5).collect();
        for (i, line) in lines.iter().enumerate() {
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

        // Кнопки
        let btn_w = 200.0;
        let btn_h = 50.0;
        let btn_y = y + h - 80.0;
        let install_x = x + (w - btn_w * 2.0 - 20.0) / 2.0;
        let later_x = install_x + btn_w + 20.0;

        // Кнопка "Встановити"
        let mouse_pos = mouse_position();
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

        if mouse_on_install && is_mouse_button_pressed(MouseButton::Left) {
            let _ = self.updater.download_and_install();
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

        if mouse_on_later && is_mouse_button_pressed(MouseButton::Left) {
            self.updater.dismiss();
        }
    }

    fn draw_downloading_window(&self, font: &Font, progress: f32) {
        let (sw, sh) = (screen_width(), screen_height());
        let w = 400.0;
        let h = 150.0;
        let x = (sw - w) / 2.0;
        let y = (sh - h) / 2.0;

        draw_rectangle(0.0, 0.0, sw, sh, Color::from_rgba(0, 0, 0, 180));
        draw_rectangle(x, y, w, h, Color::from_rgba(30, 30, 40, 255));
        draw_rectangle_lines(x, y, w, h, 3.0, Color::from_rgba(100, 150, 255, 255));

        draw_text_ex(
            "Завантаження оновлення...",
            x + 20.0,
            y + 40.0,
            TextParams {
                font: Some(font),
                font_size: 20,
                color: WHITE,
                ..Default::default()
            },
        );

        let bar_w = w - 40.0;
        let bar_h = 30.0;
        let bar_x = x + 20.0;
        let bar_y = y + 70.0;

        draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color::from_rgba(50, 50, 60, 255));
        draw_rectangle(bar_x, bar_y, bar_w * progress, bar_h, Color::from_rgba(100, 200, 100, 255));

        let progress_text = format!("{:.0}%", progress * 100.0);
        draw_text_ex(
            &progress_text,
            bar_x + bar_w / 2.0 - 20.0,
            bar_y + 20.0,
            TextParams {
                font: Some(font),
                font_size: 18,
                color: WHITE,
                ..Default::default()
            },
        );
    }

    fn draw_installing_window(&self, font: &Font) {
        let (sw, sh) = (screen_width(), screen_height());
        let w = 400.0;
        let h = 100.0;
        let x = (sw - w) / 2.0;
        let y = (sh - h) / 2.0;

        draw_rectangle(0.0, 0.0, sw, sh, Color::from_rgba(0, 0, 0, 180));
        draw_rectangle(x, y, w, h, Color::from_rgba(30, 30, 40, 255));
        draw_rectangle_lines(x, y, w, h, 3.0, Color::from_rgba(100, 150, 255, 255));

        draw_text_ex(
            "Встановлення...",
            x + w / 2.0 - 70.0,
            y + h / 2.0 + 8.0,
            TextParams {
                font: Some(font),
                font_size: 24,
                color: WHITE,
                ..Default::default()
            },
        );
    }

    fn draw_failed_window(&mut self, font: &Font, error: &str) {
        let (sw, sh) = (screen_width(), screen_height());
        let w = 500.0;
        let h = 200.0;
        let x = (sw - w) / 2.0;
        let y = (sh - h) / 2.0;

        draw_rectangle(0.0, 0.0, sw, sh, Color::from_rgba(0, 0, 0, 180));
        draw_rectangle(x, y, w, h, Color::from_rgba(30, 30, 40, 255));
        draw_rectangle_lines(x, y, w, h, 3.0, Color::from_rgba(200, 50, 50, 255));

        draw_text_ex(
            "Помилка оновлення",
            x + 20.0,
            y + 40.0,
            TextParams {
                font: Some(font),
                font_size: 24,
                color: Color::from_rgba(255, 100, 100, 255),
                ..Default::default()
            },
        );

        // Обрізати помилку до 50 символів
        let error_text = if error.len() > 50 {
            format!("{}...", &error[..50])
        } else {
            error.to_string()
        };

        draw_text_ex(
            &error_text,
            x + 20.0,
            y + 80.0,
            TextParams {
                font: Some(font),
                font_size: 16,
                color: Color::from_rgba(200, 200, 200, 255),
                ..Default::default()
            },
        );

        // Кнопки
        let btn_w = 150.0;
        let btn_h = 40.0;
        let btn_y = y + h - 60.0;
        let retry_x = x + (w - btn_w * 2.0 - 20.0) / 2.0;
        let cancel_x = retry_x + btn_w + 20.0;

        let mouse_pos = mouse_position();

        // Кнопка "Повторити"
        let mouse_on_retry = mouse_pos.0 >= retry_x && mouse_pos.0 <= retry_x + btn_w
            && mouse_pos.1 >= btn_y && mouse_pos.1 <= btn_y + btn_h;

        let retry_color = if mouse_on_retry {
            Color::from_rgba(80, 180, 80, 255)
        } else {
            Color::from_rgba(60, 140, 60, 255)
        };

        draw_rectangle(retry_x, btn_y, btn_w, btn_h, retry_color);
        draw_rectangle_lines(retry_x, btn_y, btn_w, btn_h, 2.0, WHITE);
        draw_text_ex(
            "Повторити",
            retry_x + 35.0,
            btn_y + 26.0,
            TextParams {
                font: Some(font),
                font_size: 18,
                color: WHITE,
                ..Default::default()
            },
        );

        if mouse_on_retry && is_mouse_button_pressed(MouseButton::Left) {
            self.updater.retry();
        }

        // Кнопка "Пізніше"
        let mouse_on_cancel = mouse_pos.0 >= cancel_x && mouse_pos.0 <= cancel_x + btn_w
            && mouse_pos.1 >= btn_y && mouse_pos.1 <= btn_y + btn_h;

        let cancel_color = if mouse_on_cancel {
            Color::from_rgba(100, 100, 120, 255)
        } else {
            Color::from_rgba(70, 70, 90, 255)
        };

        draw_rectangle(cancel_x, btn_y, btn_w, btn_h, cancel_color);
        draw_rectangle_lines(cancel_x, btn_y, btn_w, btn_h, 2.0, WHITE);
        draw_text_ex(
            "Пізніше",
            cancel_x + 45.0,
            btn_y + 26.0,
            TextParams {
                font: Some(font),
                font_size: 18,
                color: WHITE,
                ..Default::default()
            },
        );

        if mouse_on_cancel && is_mouse_button_pressed(MouseButton::Left) {
            self.updater.dismiss();
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
