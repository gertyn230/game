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
    clouds: Vec<Cloud>,
    stars: Vec<Star>,
    background: Option<Texture2D>,
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

struct Cloud {
    x: f32,
    y: f32,
    speed: f32,
    scale: f32,
}

struct Star {
    x: f32,
    y: f32,
    brightness: f32,
    twinkle_speed: f32,
}

impl Menu {
    pub fn new() -> Self {
        let mut particles = Vec::new();
        for _ in 0..30 {
            particles.push(Particle {
                x: rand::gen_range(0.0, 1920.0),
                y: rand::gen_range(0.0, 1080.0),
                vx: rand::gen_range(-10.0, 10.0),
                vy: rand::gen_range(-30.0, -10.0),
                size: rand::gen_range(1.0, 3.0),
                alpha: rand::gen_range(0.3, 0.7),
                life: 1.0,
            });
        }

        let mut clouds = Vec::new();
        for i in 0..5 {
            clouds.push(Cloud {
                x: rand::gen_range(0.0, 1920.0),
                y: 100.0 + i as f32 * 80.0,
                speed: rand::gen_range(5.0, 15.0),
                scale: rand::gen_range(0.8, 1.5),
            });
        }

        let mut stars = Vec::new();
        for _ in 0..100 {
            stars.push(Star {
                x: rand::gen_range(0.0, 1920.0),
                y: rand::gen_range(0.0, 600.0),
                brightness: rand::gen_range(0.3, 1.0),
                twinkle_speed: rand::gen_range(1.0, 3.0),
            });
        }

        Menu {
            state: MenuState::Loading,
            loading_progress: 0.0,
            loading_message: String::from(""),
            selected_button: 0,
            has_save: false,
            particles,
            clouds,
            stars,
            background: None,
        }
    }

    /// Завантажує фонове зображення для меню
    pub async fn load_background(&mut self) {
        // Спробувати завантажити заставку кілька разів
        for _ in 0..3 {
            if let Ok(texture) = load_texture("assets/menu_background.png").await {
                self.background = Some(texture);
                println!("✅ Background loaded successfully!");
                return;
            }
            println!("⚠️ Failed to load background, retrying...");
        }
        println!("❌ Failed to load background after 3 attempts, using fallback");
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

    fn draw_pixel_background(&mut self, sw: f32, sh: f32) {
        let time = get_time() as f32;

        // Небо з градієнтом (день/ніч)
        let is_day = (time / 30.0) % 2.0 < 1.0;
        let sky_top = if is_day {
            Color::from_rgba(135, 206, 235, 255) // Блакитне небо
        } else {
            Color::from_rgba(25, 25, 50, 255) // Нічне небо
        };
        let sky_bottom = if is_day {
            Color::from_rgba(200, 220, 240, 255)
        } else {
            Color::from_rgba(40, 40, 70, 255)
        };

        // Градієнт неба
        for i in 0..100 {
            let y = (i as f32 / 100.0) * sh * 0.7;
            let t = i as f32 / 100.0;
            let r = (sky_top.r as f32 * (1.0 - t) + sky_bottom.r as f32 * t) as u8;
            let g = (sky_top.g as f32 * (1.0 - t) + sky_bottom.g as f32 * t) as u8;
            let b = (sky_top.b as f32 * (1.0 - t) + sky_bottom.b as f32 * t) as u8;
            draw_rectangle(0.0, y, sw, sh * 0.7 / 100.0, Color::from_rgba(r, g, b, 255));
        }

        // Зірки вночі
        if !is_day {
            for star in &self.stars {
                let twinkle = ((time * star.twinkle_speed).sin() * 0.3 + 0.7) * star.brightness;
                let alpha = (twinkle * 255.0) as u8;
                draw_rectangle(star.x, star.y, 2.0, 2.0, Color::from_rgba(255, 255, 200, alpha));
            }
        }

        // Сонце/Місяць
        let celestial_y = sh * 0.15;
        if is_day {
            // Сонце
            draw_circle(sw * 0.85, celestial_y, 40.0, Color::from_rgba(255, 220, 100, 255));
            draw_circle(sw * 0.85, celestial_y, 35.0, Color::from_rgba(255, 240, 150, 255));
        } else {
            // Місяць
            draw_circle(sw * 0.85, celestial_y, 35.0, Color::from_rgba(240, 240, 255, 255));
            draw_circle(sw * 0.85 - 10.0, celestial_y - 5.0, 30.0, sky_top);
        }

        // Оновлення та малювання хмар
        for cloud in &mut self.clouds {
            cloud.x += cloud.speed * 0.016;
            if cloud.x > sw + 200.0 {
                cloud.x = -200.0;
            }

            // Піксельна хмара
            let cx = cloud.x;
            let cy = cloud.y;
            let s = cloud.scale * 8.0;
            let cloud_color = if is_day {
                Color::from_rgba(255, 255, 255, 200)
            } else {
                Color::from_rgba(80, 80, 100, 150)
            };

            // Хмара з піксельними блоками
            draw_rectangle(cx + s * 2.0, cy, s * 4.0, s, cloud_color);
            draw_rectangle(cx + s, cy + s, s * 6.0, s, cloud_color);
            draw_rectangle(cx, cy + s * 2.0, s * 8.0, s, cloud_color);
            draw_rectangle(cx + s, cy + s * 3.0, s * 6.0, s, cloud_color);
        }

        // Далекі гори (задній план)
        let mountain_color_far = if is_day {
            Color::from_rgba(120, 140, 160, 255)
        } else {
            Color::from_rgba(40, 45, 60, 255)
        };
        for i in 0..10 {
            let x = i as f32 * (sw / 10.0);
            let peak_height = 150.0 + ((i as f32 * 0.7).sin() * 80.0);
            let base_y = sh * 0.5;

            // Трикутник гори
            for y in 0..peak_height as i32 {
                let width = (peak_height - y as f32) * 2.0;
                draw_rectangle(x + sw / 20.0 - width / 2.0, base_y - y as f32, width, 1.0, mountain_color_far);
            }
        }

        // Середні гори
        let mountain_color_mid = if is_day {
            Color::from_rgba(80, 120, 100, 255)
        } else {
            Color::from_rgba(30, 40, 45, 255)
        };
        for i in 0..8 {
            let x = i as f32 * (sw / 8.0) + 50.0;
            let peak_height = 200.0 + ((i as f32 * 1.2).sin() * 100.0);
            let base_y = sh * 0.6;

            for y in 0..peak_height as i32 {
                let width = (peak_height - y as f32) * 1.8;
                draw_rectangle(x - width / 2.0, base_y - y as f32, width, 1.0, mountain_color_mid);
            }
        }

        // Передні гори
        let mountain_color_front = if is_day {
            Color::from_rgba(60, 100, 80, 255)
        } else {
            Color::from_rgba(25, 35, 40, 255)
        };
        for i in 0..6 {
            let x = i as f32 * (sw / 6.0);
            let peak_height = 250.0 + ((i as f32 * 0.5).sin() * 120.0);
            let base_y = sh * 0.7;

            for y in 0..peak_height as i32 {
                let width = (peak_height - y as f32) * 1.5;
                draw_rectangle(x + sw / 12.0 - width / 2.0, base_y - y as f32, width, 1.0, mountain_color_front);
            }
        }

        // Земля/трава
        let ground_color = if is_day {
            Color::from_rgba(90, 140, 90, 255)
        } else {
            Color::from_rgba(40, 50, 40, 255)
        };
        draw_rectangle(0.0, sh * 0.7, sw, sh * 0.3, ground_color);

        // Піксельні дерева
        let tree_color = if is_day {
            Color::from_rgba(50, 80, 50, 255)
        } else {
            Color::from_rgba(30, 40, 30, 255)
        };
        let trunk_color = if is_day {
            Color::from_rgba(100, 70, 50, 255)
        } else {
            Color::from_rgba(60, 50, 40, 255)
        };

        for i in 0..15 {
            let tx = i as f32 * (sw / 15.0) + (time * 0.01).sin() * 5.0;
            let ty = sh * 0.7 - 10.0;
            let tree_size = 12.0 + (i as f32 * 0.5).sin() * 6.0;

            // Стовбур
            draw_rectangle(tx - tree_size * 0.2, ty - tree_size, tree_size * 0.4, tree_size, trunk_color);

            // Крона (піксельна)
            draw_rectangle(tx - tree_size * 0.8, ty - tree_size * 2.0, tree_size * 1.6, tree_size * 0.3, tree_color);
            draw_rectangle(tx - tree_size * 0.6, ty - tree_size * 2.3, tree_size * 1.2, tree_size * 0.3, tree_color);
            draw_rectangle(tx - tree_size * 0.4, ty - tree_size * 2.6, tree_size * 0.8, tree_size * 0.3, tree_color);
            draw_rectangle(tx - tree_size * 0.2, ty - tree_size * 2.9, tree_size * 0.4, tree_size * 0.3, tree_color);
        }

        // Трава (піксельна)
        for i in 0..200 {
            let gx = (i as f32 * sw / 200.0 + time * 2.0).rem_euclid(sw);
            let gh = 5.0 + ((i as f32 * 0.3 + time).sin() * 3.0);
            draw_rectangle(gx, sh * 0.7 - gh, 2.0, gh, if is_day {
                Color::from_rgba(100, 180, 100, 200)
            } else {
                Color::from_rgba(50, 70, 50, 150)
            });
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

        // Малюємо фон (або згенеровану заставку, або піксельний фон)
        if let Some(bg) = &self.background {
            draw_texture_ex(
                bg,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(sw, sh)),
                    ..Default::default()
                },
            );
        } else {
            // Якщо заставка не завантажилась, малюємо піксельний фон
            self.draw_pixel_background(sw, sh);
        }

        // Кнопки меню (без чорної панелі!)
        let button_width = 400.0; // Збільшено для довгих слів
        let button_height = 70.0;
        let button_spacing = 25.0;
        let start_y = sh * 0.55;

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
            let clicked = hovered && (is_mouse_button_pressed(MouseButton::Left) ||
                touches().iter().any(|t| t.phase == TouchPhase::Started));

            // Піксельна тінь
            draw_rectangle(x + 6.0, y + 6.0, button_width, button_height, Color::from_rgba(20, 20, 30, 180));

            // Дерев'яна кнопка з градієнтом
            let btn_color = if hovered {
                Color::from_rgba(210, 180, 140, 255) // Світле дерево при наведенні
            } else {
                Color::from_rgba(139, 90, 43, 255) // Темне дерево
            };
            draw_rectangle(x, y, button_width, button_height, btn_color);

            // Внутрішня рамка (світліша)
            draw_rectangle_lines(x + 3.0, y + 3.0, button_width - 6.0, button_height - 6.0, 2.0,
                Color::from_rgba(210, 180, 140, 120));

            // Зовнішня рамка (темніша)
            draw_rectangle_lines(x, y, button_width, button_height, 3.0,
                Color::from_rgba(80, 50, 20, 255));

            // Декоративні кутові пікселі
            let corner_size = 4.0;
            let corner_color = Color::from_rgba(255, 220, 150, 255);
            // Лівий верхній
            draw_rectangle(x + 8.0, y + 8.0, corner_size, corner_size, corner_color);
            // Правий верхній
            draw_rectangle(x + button_width - 12.0, y + 8.0, corner_size, corner_size, corner_color);
            // Лівий нижній
            draw_rectangle(x + 8.0, y + button_height - 12.0, corner_size, corner_size, corner_color);
            // Правий нижній
            draw_rectangle(x + button_width - 12.0, y + button_height - 12.0, corner_size, corner_size, corner_color);

            // Текст кнопки з тінню
            let text_color = if hovered {
                Color::from_rgba(50, 30, 10, 255) // Темний текст на світлій кнопці
            } else {
                Color::from_rgba(255, 240, 200, 255) // Світлий текст на темній кнопці
            };

            let text_measure = measure_text(label, Some(font), (fs * 1.4) as u16, 1.0);
            let text_x = x + (button_width - text_measure.width) / 2.0;
            let text_y = y + (button_height + text_measure.height) / 2.0;

            // Тінь тексту
            draw_text_ex(
                label,
                text_x + 2.0,
                text_y + 2.0,
                TextParams {
                    font: Some(font),
                    font_size: (fs * 1.4) as u16,
                    color: Color::from_rgba(0, 0, 0, 100),
                    ..Default::default()
                },
            );

            // Основний текст
            draw_text_ex(
                label,
                text_x,
                text_y,
                TextParams {
                    font: Some(font),
                    font_size: (fs * 1.4) as u16,
                    color: text_color,
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
