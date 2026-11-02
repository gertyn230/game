//! Головне меню гри з екраном завантаження

use macroquad::prelude::*;
use crate::i18n::Texts;
use crate::settings::{Language, Settings};

#[derive(Clone, Copy, PartialEq)]
pub enum MenuState {
    Loading,
    MainMenu,
    Settings,
    InGame,
}

pub struct Menu {
    pub state: MenuState,
    pub loading_progress: f32,
    pub loading_message: String,
    pub selected_button: usize,
    pub has_save: bool,
    particles: Vec<Particle>,
}

struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    size: f32,
    alpha: f32,
    life: f32,
}

impl Menu {
    pub fn new() -> Self {
        let mut particles = Vec::new();
        for _ in 0..50 {
            particles.push(Particle {
                x: (get_time() * 1000.0) as f32 % 1920.0,
                y: (get_time() * 2000.0) as f32 % 1080.0,
                vx: (get_time() * 10.0).sin() as f32 * 20.0,
                vy: -20.0 - (get_time() * 5.0).cos() as f32 * 10.0,
                size: 2.0 + (get_time() * 3.0).sin() as f32 * 2.0,
                alpha: 0.3 + (get_time() * 2.0).cos() as f32 * 0.2,
                life: 1.0,
            });
        }

        Menu {
            state: MenuState::Loading,
            loading_progress: 0.0,
            loading_message: String::from("Initializing..."),
            selected_button: 0,
            has_save: false,
            particles,
        }
    }

    pub fn update_loading(&mut self, progress: f32, message: &str) {
        self.loading_progress = progress.clamp(0.0, 1.0);
        self.loading_message = message.to_string();
    }

    pub fn update_particles(&mut self, dt: f32, sw: f32, sh: f32) {
        // Оновлення частинок
        for p in &mut self.particles {
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.life -= dt * 0.2;

            // Респавн частинок
            if p.life <= 0.0 || p.y < -10.0 || p.x < -10.0 || p.x > sw + 10.0 {
                p.x = rand::gen_range(0.0, sw);
                p.y = sh + 10.0;
                p.vx = rand::gen_range(-20.0, 20.0);
                p.vy = rand::gen_range(-60.0, -20.0);
                p.size = rand::gen_range(2.0, 6.0);
                p.alpha = rand::gen_range(0.2, 0.6);
                p.life = 1.0;
            }
        }
    }

    fn draw_particles(&self) {
        for p in &self.particles {
            let alpha = (p.alpha * p.life * 255.0) as u8;
            draw_circle(p.x, p.y, p.size, Color::from_rgba(236, 232, 220, alpha));
        }
    }

    pub fn finish_loading(&mut self) {
        self.state = MenuState::MainMenu;
    }

    pub fn draw_loading(&mut self, font: &Font, _texts: &Texts) {
        let (sw, sh) = (screen_width(), screen_height());
        let fs = (sw.min(sh) * 0.04).max(16.0);

        // Темний фон з градієнтом
        clear_background(Color::from_rgba(29, 31, 27, 255));

        // Декоративний фон
        for i in 0..20 {
            let x = (i as f32 / 20.0) * sw;
            let alpha = 0.02 + (get_time() * 0.3 + i as f64 * 0.5).sin().abs() * 0.03;
            draw_rectangle(x, 0.0, sw / 20.0, sh, Color::from_rgba(236, 232, 220, (alpha * 255.0) as u8));
        }

        // Логотип/назва гри
        let title_y = sh * 0.35;
        let title = "🏡 Pixel Village";
        draw_text_ex(
            title,
            sw / 2.0 - measure_text(title, Some(font), fs as u16 * 2, 1.0).width / 2.0,
            title_y,
            TextParams {
                font: Some(font),
                font_size: (fs * 2.5) as u16,
                color: Color::from_rgba(236, 232, 220, 255),
                ..Default::default()
            },
        );

        // Версія
        let version = format!("v{}", env!("CARGO_PKG_VERSION"));
        draw_text_ex(
            &version,
            sw / 2.0 - measure_text(&version, Some(font), fs as u16, 1.0).width / 2.0,
            title_y + fs * 2.0,
            TextParams {
                font: Some(font),
                font_size: (fs * 0.9) as u16,
                color: Color::from_rgba(236, 232, 220, 180),
                ..Default::default()
            },
        );

        // Прогрес бар
        let bar_y = sh * 0.6;
        let bar_width = sw * 0.5;
        let bar_height = 8.0;
        let bar_x = (sw - bar_width) / 2.0;

        // Фон прогрес бару
        draw_rectangle(bar_x, bar_y, bar_width, bar_height, Color::from_rgba(67, 71, 62, 255));

        // Прогрес
        let progress_width = bar_width * self.loading_progress;
        draw_rectangle(bar_x, bar_y, progress_width, bar_height, Color::from_rgba(42, 157, 143, 255));

        // Відсоток
        let percent_text = format!("{}%", (self.loading_progress * 100.0) as u32);
        draw_text_ex(
            &percent_text,
            sw / 2.0 - measure_text(&percent_text, Some(font), fs as u16, 1.0).width / 2.0,
            bar_y - 10.0,
            TextParams {
                font: Some(font),
                font_size: (fs * 0.8) as u16,
                color: Color::from_rgba(236, 232, 220, 200),
                ..Default::default()
            },
        );

        // Повідомлення завантаження
        draw_text_ex(
            &self.loading_message,
            sw / 2.0 - measure_text(&self.loading_message, Some(font), fs as u16, 1.0).width / 2.0,
            bar_y + bar_height + 30.0,
            TextParams {
                font: Some(font),
                font_size: (fs * 1.1) as u16,
                color: Color::from_rgba(236, 232, 220, 230),
                ..Default::default()
            },
        );

        // Анімований індикатор
        let dots_count = ((get_time() * 2.0) as usize % 4) as usize;
        let dots = ".".repeat(dots_count);
        draw_text_ex(
            &dots,
            sw / 2.0 - measure_text(&dots, Some(font), (fs * 1.5) as u16, 1.0).width / 2.0,
            bar_y + bar_height + 60.0,
            TextParams {
                font: Some(font),
                font_size: (fs * 1.5) as u16,
                color: Color::from_rgba(236, 232, 220, 150),
                ..Default::default()
            },
        );
    }

    pub fn draw_main_menu(&mut self, font: &Font, texts: &Texts) -> Option<MenuAction> {
        let (sw, sh) = (screen_width(), screen_height());
        let fs = (sw.min(sh) * 0.04).max(16.0);

        // Фон
        clear_background(Color::from_rgba(29, 31, 27, 255));

        // Декоративні елементи
        for i in 0..15 {
            let offset = (get_time() * 0.2 + i as f64 * 0.3).sin() * 20.0;
            let x = (i as f32 / 15.0) * sw + offset as f32;
            draw_rectangle(x, 0.0, 2.0, sh, Color::from_rgba(236, 232, 220, 15));
        }

        // Назва гри
        let title = "🏡 Pixel Village";
        draw_text_ex(
            title,
            sw / 2.0 - measure_text(title, Some(font), (fs * 3.0) as u16, 1.0).width / 2.0,
            sh * 0.25,
            TextParams {
                font: Some(font),
                font_size: (fs * 3.0) as u16,
                color: Color::from_rgba(236, 232, 220, 255),
                ..Default::default()
            },
        );

        // Кнопки меню
        let button_width = 300.0;
        let button_height = 60.0;
        let button_spacing = 20.0;
        let start_y = sh * 0.45;

        let buttons = if self.has_save {
            vec![
                (texts.continue_game, MenuAction::Continue),
                (texts.new_game, MenuAction::NewGame),
                (texts.settings, MenuAction::OpenSettings),
                (texts.quit, MenuAction::Quit),
            ]
        } else {
            vec![
                (texts.play, MenuAction::Play),
                (texts.settings, MenuAction::OpenSettings),
                (texts.quit, MenuAction::Quit),
            ]
        };

        let mut action = None;

        for (i, (label, btn_action)) in buttons.iter().enumerate() {
            let y = start_y + i as f32 * (button_height + button_spacing);
            let x = (sw - button_width) / 2.0;

            let hovered = is_button_hovered(x, y, button_width, button_height);
            let clicked = hovered && is_mouse_button_pressed(MouseButton::Left);

            // Тінь кнопки
            draw_rectangle(x + 4.0, y + 4.0, button_width, button_height, Color::from_rgba(0, 0, 0, 80));

            // Кнопка
            let btn_color = if hovered {
                Color::from_rgba(42, 157, 143, 255)
            } else {
                Color::from_rgba(67, 71, 62, 255)
            };
            draw_rectangle(x, y, button_width, button_height, btn_color);

            // Обводка
            draw_rectangle_lines(x, y, button_width, button_height, 2.0, Color::from_rgba(236, 232, 220, 150));

            // Текст кнопки
            let text_measure = measure_text(label, Some(font), (fs * 1.3) as u16, 1.0);
            draw_text_ex(
                label,
                x + (button_width - text_measure.width) / 2.0,
                y + (button_height + text_measure.height) / 2.0,
                TextParams {
                    font: Some(font),
                    font_size: (fs * 1.3) as u16,
                    color: Color::from_rgba(236, 232, 220, 255),
                    ..Default::default()
                },
            );

            if clicked {
                action = Some(*btn_action);
            }
        }

        action
    }

    pub fn draw_settings_menu(&mut self, font: &Font, texts: &Texts, settings: &mut Settings) -> Option<MenuAction> {
        let (sw, sh) = (screen_width(), screen_height());
        let fs = (sw.min(sh) * 0.04).max(16.0);

        // Напівпрозорий фон
        draw_rectangle(0.0, 0.0, sw, sh, Color::from_rgba(0, 0, 0, 200));

        // Панель налаштувань
        let panel_width = sw * 0.7;
        let panel_height = sh * 0.8;
        let panel_x = (sw - panel_width) / 2.0;
        let panel_y = (sh - panel_height) / 2.0;

        // Тінь панелі
        draw_rectangle(panel_x + 6.0, panel_y + 6.0, panel_width, panel_height, Color::from_rgba(0, 0, 0, 120));

        // Панель
        draw_rectangle(panel_x, panel_y, panel_width, panel_height, Color::from_rgba(29, 31, 27, 255));
        draw_rectangle_lines(panel_x, panel_y, panel_width, panel_height, 3.0, Color::from_rgba(42, 157, 143, 255));

        // Заголовок
        draw_text_ex(
            texts.settings,
            panel_x + 30.0,
            panel_y + 50.0,
            TextParams {
                font: Some(font),
                font_size: (fs * 2.0) as u16,
                color: Color::from_rgba(236, 232, 220, 255),
                ..Default::default()
            },
        );

        // Налаштування мови
        let mut y_offset = panel_y + 120.0;
        draw_text_ex(
            texts.language,
            panel_x + 50.0,
            y_offset,
            TextParams {
                font: Some(font),
                font_size: (fs * 1.2) as u16,
                color: Color::from_rgba(236, 232, 220, 255),
                ..Default::default()
            },
        );

        // Кнопки вибору мови
        let lang_buttons = [
            ("🇺🇦 Українська", Language::Ukrainian),
            ("🇬🇧 English", Language::English),
            ("🇷🇺 Русский", Language::Russian),
        ];

        y_offset += 40.0;
        for (label, lang) in lang_buttons.iter() {
            let btn_x = panel_x + 50.0;
            let btn_width = 200.0;
            let btn_height = 45.0;
            let hovered = is_button_hovered(btn_x, y_offset, btn_width, btn_height);
            let selected = settings.language == *lang;

            let btn_color = if selected {
                Color::from_rgba(42, 157, 143, 255)
            } else if hovered {
                Color::from_rgba(67, 71, 62, 200)
            } else {
                Color::from_rgba(67, 71, 62, 150)
            };

            draw_rectangle(btn_x, y_offset, btn_width, btn_height, btn_color);
            draw_rectangle_lines(btn_x, y_offset, btn_width, btn_height, 2.0, Color::from_rgba(236, 232, 220, 100));

            let text_measure = measure_text(label, Some(font), fs as u16, 1.0);
            draw_text_ex(
                label,
                btn_x + (btn_width - text_measure.width) / 2.0,
                y_offset + (btn_height + text_measure.height) / 2.0,
                TextParams {
                    font: Some(font),
                    font_size: fs as u16,
                    color: Color::from_rgba(236, 232, 220, 255),
                    ..Default::default()
                },
            );

            if hovered && is_mouse_button_pressed(MouseButton::Left) {
                settings.language = *lang;
            }

            y_offset += btn_height + 10.0;
        }

        // Кнопка "Назад"
        let back_btn_width = 150.0;
        let back_btn_height = 50.0;
        let back_btn_x = panel_x + panel_width - back_btn_width - 30.0;
        let back_btn_y = panel_y + panel_height - back_btn_height - 30.0;

        let back_hovered = is_button_hovered(back_btn_x, back_btn_y, back_btn_width, back_btn_height);
        let back_color = if back_hovered {
            Color::from_rgba(42, 157, 143, 255)
        } else {
            Color::from_rgba(67, 71, 62, 255)
        };

        draw_rectangle(back_btn_x, back_btn_y, back_btn_width, back_btn_height, back_color);
        draw_rectangle_lines(back_btn_x, back_btn_y, back_btn_width, back_btn_height, 2.0, Color::from_rgba(236, 232, 220, 150));

        let back_text = texts.close;
        let back_measure = measure_text(back_text, Some(font), (fs * 1.2) as u16, 1.0);
        draw_text_ex(
            back_text,
            back_btn_x + (back_btn_width - back_measure.width) / 2.0,
            back_btn_y + (back_btn_height + back_measure.height) / 2.0,
            TextParams {
                font: Some(font),
                font_size: (fs * 1.2) as u16,
                color: Color::from_rgba(236, 232, 220, 255),
                ..Default::default()
            },
        );

        if back_hovered && is_mouse_button_pressed(MouseButton::Left) {
            return Some(MenuAction::CloseSettings);
        }

        None
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum MenuAction {
    Play,
    Continue,
    NewGame,
    OpenSettings,
    CloseSettings,
    Quit,
}

fn is_button_hovered(x: f32, y: f32, width: f32, height: f32) -> bool {
    let (mx, my) = mouse_position();
    mx >= x && mx <= x + width && my >= y && my <= y + height
}
