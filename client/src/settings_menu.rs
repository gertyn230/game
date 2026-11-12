//! Меню налаштувань з категоріями

use macroquad::prelude::*;
use crate::i18n::Texts;
use crate::settings::{Language, Settings};

#[derive(Clone, Copy, PartialEq)]
pub enum SettingsCategory {
    Main,       // Головне меню з 4 кнопками
    Language,   // Вибір мови
    Audio,      // Музика та звук
    Controls,   // Керування
    Graphics,   // Графіка
}

pub struct SettingsMenu {
    pub category: SettingsCategory,
}

impl SettingsMenu {
    pub fn new() -> Self {
        Self {
            category: SettingsCategory::Main,
        }
    }

    /// Малює меню налаштувань
    pub fn draw(&mut self, font: &Font, texts: &Texts, settings: &mut Settings) -> bool {
        match self.category {
            SettingsCategory::Main => self.draw_main_categories(font, texts),
            SettingsCategory::Language => self.draw_language(font, texts, settings),
            SettingsCategory::Audio => self.draw_audio(font, texts, settings),
            SettingsCategory::Controls => self.draw_controls(font, texts, settings),
            SettingsCategory::Graphics => self.draw_graphics(font, texts, settings),
        }
    }

    /// Головне меню з 4 категоріями
    fn draw_main_categories(&mut self, font: &Font, texts: &Texts) -> bool {
        let (sw, sh) = (screen_width(), screen_height());
        let fs = (sw.min(sh) * 0.04).max(16.0);
        let time = get_time() as f32;

        // Темний фон з анімацією
        let backdrop_pulse = ((time * 0.5).sin() * 10.0) as u8;
        draw_rectangle(0.0, 0.0, sw, sh, Color::from_rgba(10 + backdrop_pulse, 10, 20 + backdrop_pulse, 235));

        // Анімовані частинки
        for i in 0..8 {
            let phase = (time * 0.5 + i as f32).sin();
            let x = sw * 0.15 + i as f32 * (sw * 0.12) + phase * 50.0;
            let y = sh * 0.3 + (time * 0.3 + i as f32 * 0.7).cos() * 100.0;
            let size = 3.0 + phase.abs() * 4.0;
            let alpha = (30.0 + phase.abs() * 40.0) as u8;
            draw_circle(x, y, size, Color::from_rgba(168, 85, 247, alpha));
        }

        // Головна панель
        let panel_width = (sw * 0.6).min(700.0).max(500.0);
        let panel_height = (sh * 0.7).min(600.0).max(450.0);
        let panel_x = (sw - panel_width) / 2.0;
        let panel_y = (sh - panel_height) / 2.0;

        // Тінь під панеллю
        for i in 0..3 {
            let offset = (i + 1) as f32 * 3.0;
            let alpha = 30 - i * 8;
            draw_rectangle(
                panel_x + offset,
                panel_y + offset,
                panel_width,
                panel_height,
                Color::from_rgba(0, 0, 0, alpha as u8)
            );
        }

        // Фон панелі
        draw_rectangle(panel_x, panel_y, panel_width, panel_height, Color::from_rgba(20, 24, 35, 245));

        // Рамка
        let glow_pulse = ((time * 2.5).sin() * 0.3 + 0.7) * 255.0;
        draw_rectangle_lines(panel_x, panel_y, panel_width, panel_height, 3.0,
            Color::from_rgba(124, 58, 237, glow_pulse as u8));

        // Заголовок
        let title_y = panel_y + 55.0;
        let title_size = fs * 2.0;

        draw_text_ex(
            texts.settings,
            panel_x + 50.0,
            title_y,
            TextParams {
                font: Some(font),
                font_size: title_size as u16,
                color: Color::from_rgba(255, 255, 255, 255),
                ..Default::default()
            },
        );

        // Лінія під заголовком
        draw_rectangle(
            panel_x + 40.0,
            title_y + 18.0,
            panel_width - 80.0,
            2.0,
            Color::from_rgba(124, 58, 237, 180)
        );

        // 4 великі кнопки категорій
        let categories = [
            ("🌍", "Мова", SettingsCategory::Language),
            ("🔊", "Аудіо", SettingsCategory::Audio),
            ("🎮", "Керування", SettingsCategory::Controls),
            ("🎨", "Графіка", SettingsCategory::Graphics),
        ];

        let btn_width = (panel_width - 120.0) / 2.0;
        let btn_height = 100.0;
        let btn_spacing = 20.0;
        let start_y = title_y + 80.0;

        for (idx, (emoji, label, category)) in categories.iter().enumerate() {
            let col = idx % 2;
            let row = idx / 2;

            let btn_x = panel_x + 50.0 + col as f32 * (btn_width + btn_spacing);
            let btn_y = start_y + row as f32 * (btn_height + btn_spacing);

            let hovered = is_button_hovered(btn_x, btn_y, btn_width, btn_height);
            let hover_lift = if hovered { -5.0 } else { 0.0 };

            // Тінь
            if hovered {
                draw_rectangle(
                    btn_x + 5.0,
                    btn_y + hover_lift + 5.0,
                    btn_width,
                    btn_height,
                    Color::from_rgba(0, 0, 0, 80)
                );
            }

            // Фон кнопки з градієнтом
            let bg_color = if hovered {
                Color::from_rgba(75, 60, 180, 220)
            } else {
                Color::from_rgba(45, 45, 70, 200)
            };

            draw_rectangle(btn_x, btn_y + hover_lift, btn_width, btn_height, bg_color);

            // Рамка
            let border_color = if hovered {
                Color::from_rgba(168, 85, 247, 255)
            } else {
                Color::from_rgba(100, 100, 120, 150)
            };

            draw_rectangle_lines(btn_x, btn_y + hover_lift, btn_width, btn_height, 2.0, border_color);

            // Емодзі іконка
            let emoji_size = fs * 2.5;
            let emoji_y = btn_y + hover_lift + 35.0;

            draw_text_ex(
                emoji,
                btn_x + (btn_width - emoji_size * 0.6) / 2.0,
                emoji_y,
                TextParams {
                    font: Some(font),
                    font_size: emoji_size as u16,
                    color: Color::from_rgba(255, 255, 255, 255),
                    ..Default::default()
                },
            );

            // Текст
            let text_size = fs * 1.2;
            let text_measure = measure_text(label, Some(font), text_size as u16, 1.0);
            let text_y = btn_y + hover_lift + 75.0;

            draw_text_ex(
                label,
                btn_x + (btn_width - text_measure.width) / 2.0,
                text_y,
                TextParams {
                    font: Some(font),
                    font_size: text_size as u16,
                    color: Color::from_rgba(220, 220, 255, 255),
                    ..Default::default()
                },
            );

            // Обробка кліка
            if hovered && is_mouse_button_pressed(MouseButton::Left) {
                self.category = *category;
            }
        }

        // Кнопка "Назад"
        let back_btn_width = 150.0;
        let back_btn_height = 50.0;
        let back_btn_x = panel_x + (panel_width - back_btn_width) / 2.0;
        let back_btn_y = panel_y + panel_height - back_btn_height - 30.0;

        let back_hovered = is_button_hovered(back_btn_x, back_btn_y, back_btn_width, back_btn_height);
        let back_lift = if back_hovered { -3.0 } else { 0.0 };

        // Фон кнопки назад
        let back_bg = if back_hovered {
            Color::from_rgba(65, 55, 90, 220)
        } else {
            Color::from_rgba(45, 45, 65, 180)
        };

        draw_rectangle(back_btn_x, back_btn_y + back_lift, back_btn_width, back_btn_height, back_bg);
        draw_rectangle_lines(back_btn_x, back_btn_y + back_lift, back_btn_width, back_btn_height, 2.0,
            Color::from_rgba(100, 100, 120, 180));

        let back_text = "← Назад";
        let back_measure = measure_text(back_text, Some(font), fs as u16, 1.0);
        draw_text_ex(
            back_text,
            back_btn_x + (back_btn_width - back_measure.width) / 2.0,
            back_btn_y + back_lift + (back_btn_height + back_measure.height) / 2.0,
            TextParams {
                font: Some(font),
                font_size: fs as u16,
                color: Color::from_rgba(200, 200, 220, 255),
                ..Default::default()
            },
        );

        // Повернення до головного меню
        if (back_hovered && is_mouse_button_pressed(MouseButton::Left)) || is_key_pressed(KeyCode::Escape) {
            return true; // Закрити налаштування
        }

        false
    }

    /// Меню вибору мови
    fn draw_language(&mut self, font: &Font, texts: &Texts, settings: &mut Settings) -> bool {
        let (sw, sh) = (screen_width(), screen_height());
        let fs = (sw.min(sh) * 0.04).max(16.0);
        let time = get_time() as f32;

        // Фон
        let backdrop_pulse = ((time * 0.5).sin() * 10.0) as u8;
        draw_rectangle(0.0, 0.0, sw, sh, Color::from_rgba(10 + backdrop_pulse, 10, 20 + backdrop_pulse, 235));

        // Панель
        let panel_width = (sw * 0.5).min(600.0).max(450.0);
        let panel_height = (sh * 0.6).min(500.0).max(400.0);
        let panel_x = (sw - panel_width) / 2.0;
        let panel_y = (sh - panel_height) / 2.0;

        draw_rectangle(panel_x, panel_y, panel_width, panel_height, Color::from_rgba(20, 24, 35, 245));
        draw_rectangle_lines(panel_x, panel_y, panel_width, panel_height, 3.0,
            Color::from_rgba(124, 58, 237, 200));

        // Заголовок
        let title = "🌍 Мова";
        draw_text_ex(
            title,
            panel_x + 40.0,
            panel_y + 50.0,
            TextParams {
                font: Some(font),
                font_size: (fs * 1.8) as u16,
                color: WHITE,
                ..Default::default()
            },
        );

        draw_rectangle(panel_x + 40.0, panel_y + 65.0, panel_width - 80.0, 2.0,
            Color::from_rgba(124, 58, 237, 180));

        // Кнопки мов
        let languages = [
            ("🇺🇦", "Українська", Language::Ukrainian),
            ("🇬🇧", "English", Language::English),
            ("🇷🇺", "Русский", Language::Russian),
        ];

        let btn_width = panel_width - 100.0;
        let btn_height = 70.0;
        let start_y = panel_y + 120.0;

        for (idx, (flag, name, lang)) in languages.iter().enumerate() {
            let btn_x = panel_x + 50.0;
            let btn_y = start_y + idx as f32 * (btn_height + 15.0);

            let selected = settings.language == *lang;
            let hovered = is_button_hovered(btn_x, btn_y, btn_width, btn_height);
            let hover_lift = if hovered { -3.0 } else { 0.0 };

            // Фон
            let bg_color = if selected {
                Color::from_rgba(124, 58, 237, 255)
            } else if hovered {
                Color::from_rgba(65, 55, 163, 200)
            } else {
                Color::from_rgba(45, 45, 65, 180)
            };

            draw_rectangle(btn_x, btn_y + hover_lift, btn_width, btn_height, bg_color);
            draw_rectangle_lines(btn_x, btn_y + hover_lift, btn_width, btn_height, 2.0,
                if selected {
                    Color::from_rgba(168, 85, 247, 255)
                } else {
                    Color::from_rgba(100, 100, 120, 120)
                });

            // Прапор та назва
            let text = format!("{} {}", flag, name);
            let text_measure = measure_text(&text, Some(font), (fs * 1.3) as u16, 1.0);

            draw_text_ex(
                &text,
                btn_x + (btn_width - text_measure.width) / 2.0,
                btn_y + hover_lift + (btn_height + text_measure.height) / 2.0,
                TextParams {
                    font: Some(font),
                    font_size: (fs * 1.3) as u16,
                    color: if selected { WHITE } else { Color::from_rgba(220, 220, 240, 255) },
                    ..Default::default()
                },
            );

            if hovered && is_mouse_button_pressed(MouseButton::Left) {
                settings.language = *lang;
            }
        }

        // Кнопка назад
        self.draw_back_button(font, panel_x, panel_y, panel_width, panel_height, fs)
    }

    /// Меню аудіо
    fn draw_audio(&mut self, font: &Font, texts: &Texts, settings: &mut Settings) -> bool {
        let (sw, sh) = (screen_width(), screen_height());
        let fs = (sw.min(sh) * 0.04).max(16.0);
        let time = get_time() as f32;

        // Фон
        let backdrop_pulse = ((time * 0.5).sin() * 10.0) as u8;
        draw_rectangle(0.0, 0.0, sw, sh, Color::from_rgba(10 + backdrop_pulse, 10, 20 + backdrop_pulse, 235));

        // Панель
        let panel_width = (sw * 0.6).min(700.0).max(500.0);
        let panel_height = (sh * 0.7).min(600.0).max(450.0);
        let panel_x = (sw - panel_width) / 2.0;
        let panel_y = (sh - panel_height) / 2.0;

        draw_rectangle(panel_x, panel_y, panel_width, panel_height, Color::from_rgba(20, 24, 35, 245));
        draw_rectangle_lines(panel_x, panel_y, panel_width, panel_height, 3.0,
            Color::from_rgba(124, 58, 237, 200));

        // Заголовок
        draw_text_ex(
            "🔊 Аудіо",
            panel_x + 40.0,
            panel_y + 50.0,
            TextParams {
                font: Some(font),
                font_size: (fs * 1.8) as u16,
                color: WHITE,
                ..Default::default()
            },
        );

        draw_rectangle(panel_x + 40.0, panel_y + 65.0, panel_width - 80.0, 2.0,
            Color::from_rgba(124, 58, 237, 180));

        let mut y = panel_y + 130.0;
        let left = panel_x + 60.0;

        // Музика
        let (music_changed, new_music) = self.draw_toggle(font, "🎵 Музика", left, y, fs, settings.music_enabled);
        if music_changed {
            settings.music_enabled = new_music;
        }
        y += 60.0;

        if settings.music_enabled {
            settings.music_volume = self.draw_slider(font, "Гучність", left + 20.0, y, panel_width - 160.0, fs, settings.music_volume);
            y += 70.0;
        }

        // Звуки
        let (sound_changed, new_sound) = self.draw_toggle(font, "🔔 Звуки", left, y, fs, settings.sounds_enabled);
        if sound_changed {
            settings.sounds_enabled = new_sound;
        }
        y += 60.0;

        if settings.sounds_enabled {
            settings.sound_volume = self.draw_slider(font, "Гучність", left + 20.0, y, panel_width - 160.0, fs, settings.sound_volume);
        }

        self.draw_back_button(font, panel_x, panel_y, panel_width, panel_height, fs)
    }

    /// Меню керування
    fn draw_controls(&mut self, font: &Font, texts: &Texts, settings: &mut Settings) -> bool {
        let (sw, sh) = (screen_width(), screen_height());
        let fs = (sw.min(sh) * 0.04).max(16.0);
        let time = get_time() as f32;

        // Фон
        let backdrop_pulse = ((time * 0.5).sin() * 10.0) as u8;
        draw_rectangle(0.0, 0.0, sw, sh, Color::from_rgba(10 + backdrop_pulse, 10, 20 + backdrop_pulse, 235));

        // Панель
        let panel_width = (sw * 0.6).min(700.0).max(500.0);
        let panel_height = (sh * 0.7).min(600.0).max(450.0);
        let panel_x = (sw - panel_width) / 2.0;
        let panel_y = (sh - panel_height) / 2.0;

        draw_rectangle(panel_x, panel_y, panel_width, panel_height, Color::from_rgba(20, 24, 35, 245));
        draw_rectangle_lines(panel_x, panel_y, panel_width, panel_height, 3.0,
            Color::from_rgba(124, 58, 237, 200));

        // Заголовок
        draw_text_ex(
            "🎮 Керування",
            panel_x + 40.0,
            panel_y + 50.0,
            TextParams {
                font: Some(font),
                font_size: (fs * 1.8) as u16,
                color: WHITE,
                ..Default::default()
            },
        );

        draw_rectangle(panel_x + 40.0, panel_y + 65.0, panel_width - 80.0, 2.0,
            Color::from_rgba(124, 58, 237, 180));

        // Список клавіш
        let controls = [
            ("WASD", "Рух"),
            ("E", "Взаємодія"),
            ("I", "Інвентар"),
            ("C", "Крафт"),
            ("ESC", "Меню"),
        ];

        let mut y = panel_y + 130.0;
        let left = panel_x + 60.0;

        for (key, action) in controls.iter() {
            // Клавіша
            let key_width = 80.0;
            let key_height = 40.0;

            draw_rectangle(left, y, key_width, key_height, Color::from_rgba(45, 45, 65, 200));
            draw_rectangle_lines(left, y, key_width, key_height, 2.0,
                Color::from_rgba(124, 58, 237, 180));

            let key_measure = measure_text(key, Some(font), fs as u16, 1.0);
            draw_text_ex(
                key,
                left + (key_width - key_measure.width) / 2.0,
                y + (key_height + key_measure.height) / 2.0,
                TextParams {
                    font: Some(font),
                    font_size: fs as u16,
                    color: Color::from_rgba(168, 85, 247, 255),
                    ..Default::default()
                },
            );

            // Опис
            draw_text_ex(
                action,
                left + key_width + 30.0,
                y + (key_height + key_measure.height) / 2.0,
                TextParams {
                    font: Some(font),
                    font_size: fs as u16,
                    color: Color::from_rgba(200, 200, 220, 255),
                    ..Default::default()
                },
            );

            y += key_height + 20.0;
        }

        self.draw_back_button(font, panel_x, panel_y, panel_width, panel_height, fs)
    }

    /// Меню графіки
    fn draw_graphics(&mut self, font: &Font, texts: &Texts, settings: &mut Settings) -> bool {
        let (sw, sh) = (screen_width(), screen_height());
        let fs = (sw.min(sh) * 0.04).max(16.0);
        let time = get_time() as f32;

        // Фон
        let backdrop_pulse = ((time * 0.5).sin() * 10.0) as u8;
        draw_rectangle(0.0, 0.0, sw, sh, Color::from_rgba(10 + backdrop_pulse, 10, 20 + backdrop_pulse, 235));

        // Панель
        let panel_width = (sw * 0.6).min(700.0).max(500.0);
        let panel_height = (sh * 0.7).min(600.0).max(450.0);
        let panel_x = (sw - panel_width) / 2.0;
        let panel_y = (sh - panel_height) / 2.0;

        draw_rectangle(panel_x, panel_y, panel_width, panel_height, Color::from_rgba(20, 24, 35, 245));
        draw_rectangle_lines(panel_x, panel_y, panel_width, panel_height, 3.0,
            Color::from_rgba(124, 58, 237, 200));

        // Заголовок
        draw_text_ex(
            "🎨 Графіка",
            panel_x + 40.0,
            panel_y + 50.0,
            TextParams {
                font: Some(font),
                font_size: (fs * 1.8) as u16,
                color: WHITE,
                ..Default::default()
            },
        );

        draw_rectangle(panel_x + 40.0, panel_y + 65.0, panel_width - 80.0, 2.0,
            Color::from_rgba(124, 58, 237, 180));

        let left = panel_x + 60.0;
        let y = panel_y + 150.0;

        // Інфо про графіку
        let info_text = "Налаштування графіки";
        draw_text_ex(
            info_text,
            left,
            y,
            TextParams {
                font: Some(font),
                font_size: (fs * 1.2) as u16,
                color: Color::from_rgba(200, 200, 220, 255),
                ..Default::default()
            },
        );

        let desc = "(Буде додано пізніше)";
        draw_text_ex(
            desc,
            left,
            y + 50.0,
            TextParams {
                font: Some(font),
                font_size: fs as u16,
                color: Color::from_rgba(150, 150, 170, 255),
                ..Default::default()
            },
        );

        self.draw_back_button(font, panel_x, panel_y, panel_width, panel_height, fs)
    }

    /// Малює кнопку "Назад"
    fn draw_back_button(&mut self, font: &Font, panel_x: f32, panel_y: f32, panel_width: f32, panel_height: f32, fs: f32) -> bool {
        let btn_width = 150.0;
        let btn_height = 50.0;
        let btn_x = panel_x + (panel_width - btn_width) / 2.0;
        let btn_y = panel_y + panel_height - btn_height - 30.0;

        let hovered = is_button_hovered(btn_x, btn_y, btn_width, btn_height);
        let lift = if hovered { -3.0 } else { 0.0 };

        let bg = if hovered {
            Color::from_rgba(65, 55, 90, 220)
        } else {
            Color::from_rgba(45, 45, 65, 180)
        };

        draw_rectangle(btn_x, btn_y + lift, btn_width, btn_height, bg);
        draw_rectangle_lines(btn_x, btn_y + lift, btn_width, btn_height, 2.0,
            Color::from_rgba(100, 100, 120, 180));

        let text = "← Назад";
        let measure = measure_text(text, Some(font), fs as u16, 1.0);
        draw_text_ex(
            text,
            btn_x + (btn_width - measure.width) / 2.0,
            btn_y + lift + (btn_height + measure.height) / 2.0,
            TextParams {
                font: Some(font),
                font_size: fs as u16,
                color: Color::from_rgba(200, 200, 220, 255),
                ..Default::default()
            },
        );

        if (hovered && is_mouse_button_pressed(MouseButton::Left)) || is_key_pressed(KeyCode::Escape) {
            self.category = SettingsCategory::Main;
            true
        } else {
            false
        }
    }

    /// Малює toggle перемикач
    fn draw_toggle(&self, font: &Font, label: &str, x: f32, y: f32, fs: f32, enabled: bool) -> (bool, bool) {
        // Текст
        draw_text_ex(
            label,
            x,
            y,
            TextParams {
                font: Some(font),
                font_size: fs as u16,
                color: Color::from_rgba(220, 220, 240, 255),
                ..Default::default()
            },
        );

        // Toggle
        let toggle_x = x + 250.0;
        let toggle_y = y - 20.0;
        let toggle_width = 60.0;
        let toggle_height = 30.0;

        let hovered = is_button_hovered(toggle_x, toggle_y, toggle_width, toggle_height);

        let bg = if enabled {
            Color::from_rgba(124, 58, 237, 255)
        } else {
            Color::from_rgba(60, 60, 80, 200)
        };

        draw_rectangle(toggle_x, toggle_y, toggle_width, toggle_height, bg);
        draw_rectangle_lines(toggle_x, toggle_y, toggle_width, toggle_height, 2.0,
            Color::from_rgba(100, 100, 120, 180));

        // Кружечок
        let circle_x = if enabled {
            toggle_x + toggle_width - 18.0
        } else {
            toggle_x + 18.0
        };

        draw_circle(circle_x, toggle_y + toggle_height / 2.0, 12.0, WHITE);

        if hovered && is_mouse_button_pressed(MouseButton::Left) {
            (true, !enabled)
        } else {
            (false, enabled)
        }
    }

    /// Малює слайдер
    fn draw_slider(&self, font: &Font, label: &str, x: f32, y: f32, width: f32, fs: f32, value: f32) -> f32 {
        // Текст
        draw_text_ex(
            label,
            x,
            y - 10.0,
            TextParams {
                font: Some(font),
                font_size: (fs * 0.9) as u16,
                color: Color::from_rgba(180, 180, 200, 255),
                ..Default::default()
            },
        );

        // Відсоток
        let percent = format!("{}%", (value * 100.0) as i32);
        let percent_measure = measure_text(&percent, Some(font), (fs * 0.9) as u16, 1.0);
        draw_text_ex(
            &percent,
            x + width - percent_measure.width,
            y - 10.0,
            TextParams {
                font: Some(font),
                font_size: (fs * 0.9) as u16,
                color: Color::from_rgba(168, 85, 247, 255),
                ..Default::default()
            },
        );

        // Лінія слайдера
        let slider_y = y + 10.0;
        let slider_height = 6.0;

        draw_rectangle(x, slider_y, width, slider_height, Color::from_rgba(60, 60, 80, 200));

        // Заповнення
        let fill_width = width * value;
        draw_rectangle(x, slider_y, fill_width, slider_height, Color::from_rgba(124, 58, 237, 255));

        // Кружечок
        let handle_x = x + fill_width;
        let handle_y = slider_y + slider_height / 2.0;

        draw_circle(handle_x, handle_y, 10.0, Color::from_rgba(168, 85, 247, 255));
        draw_circle(handle_x, handle_y, 8.0, WHITE);

        // Обробка перетягування
        if is_mouse_button_down(MouseButton::Left) {
            let (mx, _) = mouse_position();
            if mx >= x && mx <= x + width {
                let slider_rect = Rect::new(x - 10.0, slider_y - 10.0, width + 20.0, slider_height + 20.0);
                if slider_rect.contains(vec2(mx, mouse_position().1)) {
                    return ((mx - x) / width).clamp(0.0, 1.0);
                }
            }
        }

        value
    }
}

fn is_button_hovered(x: f32, y: f32, w: f32, h: f32) -> bool {
    let (mx, my) = mouse_position();
    mx >= x && mx <= x + w && my >= y && my <= y + h
}
