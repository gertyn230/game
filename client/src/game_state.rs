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
    pub show_update_popup: bool,
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
            show_update_popup: false,
        }
    }

    /// Перевірити оновлення (викликати після завантаження)
    pub async fn check_updates(&mut self) {
        match self.updater.check_for_updates().await {
            Ok(has_update) => {
                if has_update {
                    self.show_update_popup = true;
                }
            }
            Err(_) => {
                // Тихо ігноруємо помилки перевірки оновлень
            }
        }
    }

    /// Намалювати popup з оновленням
    pub fn draw_update_popup(&mut self, font: &Font) -> bool {
        if !self.show_update_popup {
            return false;
        }

        let (sw, sh) = (screen_width(), screen_height());

        // Напівпрозорий фон
        draw_rectangle(0.0, 0.0, sw, sh, Color::from_rgba(0, 0, 0, 180));

        // Вікно повідомлення
        let box_w = 600.0;
        let box_h = 400.0;
        let box_x = (sw - box_w) / 2.0;
        let box_y = (sh - box_h) / 2.0;

        // Фон вікна
        draw_rectangle(box_x, box_y, box_w, box_h, Color::from_rgba(30, 30, 40, 255));
        draw_rectangle_lines(box_x, box_y, box_w, box_h, 3.0, Color::from_rgba(100, 150, 255, 255));

        if let Some(text) = self.updater.get_update_text() {
            // Заголовок
            draw_text_ex(
                "🎮 Оновлення доступне!",
                box_x + 20.0,
                box_y + 50.0,
                TextParams {
                    font: Some(font),
                    font_size: 32,
                    color: Color::from_rgba(255, 215, 0, 255),
                    ..Default::default()
                },
            );

            // Текст оновлення
            let lines: Vec<&str> = text.lines().collect();
            let mut y = box_y + 90.0;
            for line in lines.iter().take(10) {
                draw_text_ex(
                    line,
                    box_x + 20.0,
                    y,
                    TextParams {
                        font: Some(font),
                        font_size: 18,
                        color: WHITE,
                        ..Default::default()
                    },
                );
                y += 25.0;
            }

            // Кнопка "Завантажити"
            let btn_w = 250.0;
            let btn_h = 50.0;
            let btn_x = box_x + (box_w - btn_w) / 2.0;
            let btn_y = box_y + box_h - 100.0;

            let mouse_pos = mouse_position();
            let is_hover = mouse_pos.0 >= btn_x && mouse_pos.0 <= btn_x + btn_w
                && mouse_pos.1 >= btn_y && mouse_pos.1 <= btn_y + btn_h;

            let btn_color = if is_hover {
                Color::from_rgba(70, 150, 70, 255)
            } else {
                Color::from_rgba(50, 130, 50, 255)
            };

            draw_rectangle(btn_x, btn_y, btn_w, btn_h, btn_color);
            draw_rectangle_lines(btn_x, btn_y, btn_w, btn_h, 2.0, WHITE);
            draw_text_ex(
                "📥 Завантажити оновлення",
                btn_x + 20.0,
                btn_y + 33.0,
                TextParams {
                    font: Some(font),
                    font_size: 22,
                    color: WHITE,
                    ..Default::default()
                },
            );

            // Кнопка "Пізніше"
            let skip_y = btn_y + 60.0;
            let skip_hover = mouse_pos.1 >= skip_y && mouse_pos.1 <= skip_y + 30.0
                && mouse_pos.0 >= btn_x && mouse_pos.0 <= btn_x + btn_w;

            draw_text_ex(
                "Пізніше",
                btn_x + 95.0,
                skip_y + 20.0,
                TextParams {
                    font: Some(font),
                    font_size: 18,
                    color: if skip_hover {
                        Color::from_rgba(255, 200, 100, 255)
                    } else {
                        Color::from_rgba(150, 150, 150, 255)
                    },
                    ..Default::default()
                },
            );

            // Обробка кліків
            if is_mouse_button_pressed(MouseButton::Left) {
                if is_hover {
                    // Відкрити сторінку завантаження
                    self.updater.open_download_page();
                    self.show_update_popup = false;
                    return true;
                } else if skip_hover {
                    self.show_update_popup = false;
                    return true;
                }
            }
        }

        true // Показуємо popup
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
