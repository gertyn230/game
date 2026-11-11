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
        // Вбудована заставка - працює на всіх платформах без папки assets!
        const BACKGROUND_PNG: &[u8] = include_bytes!("../../assets/menu_background.png");

        match image::load_from_memory(BACKGROUND_PNG) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let width = rgba.width() as u16;
                let height = rgba.height() as u16;
                let bytes = rgba.into_raw();

                let texture = Texture2D::from_rgba8(width, height, &bytes);
                self.background = Some(texture);
                println!("✅ Background loaded from embedded data!");
            }
            Err(e) => {
                println!("❌ Failed to load embedded background: {:?}", e);
            }
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

        // Версія гри в правому нижньому куті
        let version = env!("CARGO_PKG_VERSION");
        let version_text = format!("v{}", version);
        let version_fs = (fs * 0.7).max(12.0);
        let version_measure = measure_text(&version_text, Some(font), version_fs as u16, 1.0);
        let version_x = sw - version_measure.width - 20.0;
        let version_y = sh - 20.0;

        // Тінь версії
        draw_text_ex(
            &version_text,
            version_x + 1.0,
            version_y + 1.0,
            TextParams {
                font: Some(font),
                font_size: version_fs as u16,
                color: Color::from_rgba(0, 0, 0, 100),
                ..Default::default()
            },
        );

        // Текст версії
        draw_text_ex(
            &version_text,
            version_x,
            version_y,
            TextParams {
                font: Some(font),
                font_size: version_fs as u16,
                color: Color::from_rgba(200, 200, 200, 180),
                ..Default::default()
            },
        );

        action
    }

    pub fn draw_settings_menu(&mut self, font: &Font, texts: &Texts, settings: &mut Settings) -> Option<MenuAction> {
        let (sw, sh) = (screen_width(), screen_height());
        let fs = (sw.min(sh) * 0.04).max(16.0);

        // Тьмавий фон з легким blur ефектом
        draw_rectangle(0.0, 0.0, sw, sh, Color::from_rgba(10, 10, 20, 230));

        // Головна панель (сучасні пропорції)
        let panel_width = (sw * 0.75).min(800.0).max(600.0);
        let panel_height = (sh * 0.8).min(650.0).max(500.0);
        let panel_x = (sw - panel_width) / 2.0;
        let panel_y = (sh - panel_height) / 2.0;

        // М'яка тінь
        draw_rectangle(
            panel_x + 8.0,
            panel_y + 8.0,
            panel_width,
            panel_height,
            Color::from_rgba(0, 0, 0, 100)
        );

        // Glassmorphism фон панелі з градієнтом
        for i in 0..30 {
            let progress = i as f32 / 30.0;
            let y_step = panel_height / 30.0;
            let r = (25.0 + progress * 10.0) as u8;
            let g = (28.0 + progress * 8.0) as u8;
            let b = (35.0 + progress * 15.0) as u8;
            draw_rectangle(
                panel_x,
                panel_y + i as f32 * y_step,
                panel_width,
                y_step,
                Color::from_rgba(r, g, b, 245)
            );
        }

        // Тонка світла рамка
        draw_rectangle_lines(panel_x, panel_y, panel_width, panel_height, 2.0,
            Color::from_rgba(124, 58, 237, 180));

        // Внутрішня підсвітка
        draw_rectangle_lines(panel_x + 2.0, panel_y + 2.0, panel_width - 4.0, panel_height - 4.0, 1.0,
            Color::from_rgba(255, 255, 255, 30));

        // Заголовок (сучасний стиль)
        let title_y = panel_y + 50.0;
        let title_icon = "⚙️";
        let title_size = fs * 1.8;

        draw_text_ex(
            title_icon,
            panel_x + 40.0,
            title_y,
            TextParams {
                font: Some(font),
                font_size: title_size as u16,
                color: Color::from_rgba(168, 85, 247, 255),
                ..Default::default()
            },
        );

        draw_text_ex(
            texts.settings,
            panel_x + 85.0,
            title_y,
            TextParams {
                font: Some(font),
                font_size: title_size as u16,
                color: Color::from_rgba(255, 255, 255, 255),
                ..Default::default()
            },
        );

        // Тонка лінія під заголовком
        draw_rectangle(
            panel_x + 40.0,
            title_y + 15.0,
            panel_width - 80.0,
            1.0,
            Color::from_rgba(124, 58, 237, 100)
        );

        let mut y_offset = title_y + 60.0;
        let left_margin = panel_x + 50.0;

        // ===== СЕКЦІЯ: МОВА =====
        draw_text_ex(
            "🌍  Мова",
            left_margin,
            y_offset,
            TextParams {
                font: Some(font),
                font_size: (fs * 1.3) as u16,
                color: Color::from_rgba(220, 220, 230, 255),
                ..Default::default()
            },
        );
        y_offset += 40.0;

        // Кнопки мови (карткового стилю)
        let lang_buttons = [
            ("🇺🇦 Українська", Language::Ukrainian),
            ("🇬🇧 English", Language::English),
            ("🇷🇺 Русский", Language::Russian),
        ];

        let btn_width = 150.0;
        let btn_height = 48.0;
        let btn_spacing = 12.0;

        for (i, (label, lang)) in lang_buttons.iter().enumerate() {
            let btn_x = left_margin + i as f32 * (btn_width + btn_spacing);
            let hovered = is_button_hovered(btn_x, y_offset, btn_width, btn_height);
            let selected = settings.language == *lang;

            // Фон кнопки
            let bg_color = if selected {
                Color::from_rgba(124, 58, 237, 255)
            } else if hovered {
                Color::from_rgba(55, 48, 163, 180)
            } else {
                Color::from_rgba(40, 40, 60, 180)
            };

            // Тінь кнопки
            if hovered || selected {
                draw_rectangle(
                    btn_x + 2.0,
                    y_offset + 2.0,
                    btn_width,
                    btn_height,
                    Color::from_rgba(0, 0, 0, 80)
                );
            }

            // Основна кнопка з округленими кутами (імітація)
            draw_rectangle(btn_x, y_offset, btn_width, btn_height, bg_color);

            // Рамка
            let border_color = if selected {
                Color::from_rgba(168, 85, 247, 255)
            } else {
                Color::from_rgba(100, 100, 120, 100)
            };
            draw_rectangle_lines(btn_x, y_offset, btn_width, btn_height, 1.5, border_color);

            // Текст кнопки
            let text_color = if selected {
                Color::from_rgba(255, 255, 255, 255)
            } else {
                Color::from_rgba(200, 200, 220, 255)
            };

            let text_measure = measure_text(label, Some(font), (fs * 1.0) as u16, 1.0);
            draw_text_ex(
                label,
                btn_x + (btn_width - text_measure.width) / 2.0,
                y_offset + (btn_height + text_measure.height) / 2.0,
                TextParams {
                    font: Some(font),
                    font_size: (fs * 1.0) as u16,
                    color: text_color,
                    ..Default::default()
                },
            );

            if hovered && is_mouse_button_pressed(MouseButton::Left) {
                settings.language = *lang;
            }
        }

        y_offset += btn_height + 45.0;

        // ===== СЕКЦІЯ: АУДІО =====
        draw_text_ex(
            "🔊  Аудіо",
            left_margin,
            y_offset,
            TextParams {
                font: Some(font),
                font_size: (fs * 1.3) as u16,
                color: Color::from_rgba(220, 220, 230, 255),
                ..Default::default()
            },
        );
        y_offset += 45.0;

        // Музика toggle (сучасний стиль)
        let (music_changed, new_music_enabled) = draw_modern_toggle(
            font, "🎵  Музика", left_margin, y_offset, fs, settings.music_enabled
        );
        if music_changed {
            settings.music_enabled = new_music_enabled;
        }
        y_offset += 40.0;

        // Слайдер музики
        if settings.music_enabled {
            let slider_width = panel_width - 120.0;
            let new_volume = draw_modern_slider(
                font, left_margin + 40.0, y_offset,
                slider_width, fs, settings.music_volume
            );
            settings.music_volume = new_volume;
            y_offset += 50.0;
        } else {
            y_offset += 10.0;
        }

        // Звуки toggle
        let (sound_changed, new_sound_enabled) = draw_modern_toggle(
            font, "🔔  Звуки", left_margin, y_offset, fs, settings.sounds_enabled
        );
        if sound_changed {
            settings.sounds_enabled = new_sound_enabled;
        }
        y_offset += 40.0;

        // Слайдер звуків
        if settings.sounds_enabled {
            let slider_width = panel_width - 120.0;
            let new_volume = draw_modern_slider(
                font, left_margin + 40.0, y_offset,
                slider_width, fs, settings.sound_volume
            );
            settings.sound_volume = new_volume;
            y_offset += 50.0;
        }

        // Кнопка "Зберегти" (сучасна)
        let save_btn_width = 200.0;
        let save_btn_height = 52.0;
        let save_btn_x = (sw - save_btn_width) / 2.0;
        let save_btn_y = panel_y + panel_height - save_btn_height - 30.0;

        let save_hovered = is_button_hovered(save_btn_x, save_btn_y, save_btn_width, save_btn_height);

        // Тінь кнопки
        if save_hovered {
            draw_rectangle(
                save_btn_x + 3.0,
                save_btn_y + 3.0,
                save_btn_width,
                save_btn_height,
                Color::from_rgba(0, 0, 0, 100)
            );
        }

        // Градієнт кнопки
        let btn_color = if save_hovered {
            Color::from_rgba(139, 92, 246, 255)
        } else {
            Color::from_rgba(124, 58, 237, 255)
        };

        draw_rectangle(save_btn_x, save_btn_y, save_btn_width, save_btn_height, btn_color);
        draw_rectangle_lines(save_btn_x, save_btn_y, save_btn_width, save_btn_height, 2.0,
            Color::from_rgba(168, 85, 247, 255));

        // Текст кнопки
        let save_text = "✓  Зберегти";
        let save_measure = measure_text(save_text, Some(font), (fs * 1.4) as u16, 1.0);
        draw_text_ex(
            save_text,
            save_btn_x + (save_btn_width - save_measure.width) / 2.0,
            save_btn_y + (save_btn_height + save_measure.height) / 2.0,
            TextParams {
                font: Some(font),
                font_size: (fs * 1.4) as u16,
                color: Color::from_rgba(255, 255, 255, 255),
                ..Default::default()
            },
        );

        if save_hovered && is_mouse_button_pressed(MouseButton::Left) {
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
    // Перевірка миші (ПК)
    let (mx, my) = mouse_position();
    if mx >= x && mx <= x + width && my >= y && my <= y + height {
        return true;
    }

    // Перевірка тачів (Android)
    for touch in touches() {
        let tx = touch.position.x;
        let ty = touch.position.y;
        if tx >= x && tx <= x + width && ty >= y && ty <= y + height {
            return true;
        }
    }

    false
}

// Малює заголовок секції
fn draw_section_header(font: &Font, icon: &str, text: &str, x: f32, y: f32, fs: f32) {
    let full_text = format!("{}{}", icon, text);
    draw_text_ex(
        &full_text,
        x,
        y,
        TextParams {
            font: Some(font),
            font_size: (fs * 1.5) as u16,
            color: Color::from_rgba(42, 157, 143, 255),
            ..Default::default()
        },
    );
    // Декоративна лінія під заголовком
    draw_rectangle(x, y + 8.0, 250.0, 2.0, Color::from_rgba(42, 157, 143, 100));
}

// Малює сучасну кнопку
fn draw_modern_button(
    x: f32, y: f32, width: f32, height: f32,
    label: &str, font: &Font, fs: f32,
    selected: bool, hovered: bool
) {
    // Тінь
    if hovered || selected {
        draw_rectangle(x + 3.0, y + 3.0, width, height, Color::from_rgba(0, 0, 0, 80));
    }

    // Фон кнопки
    let btn_color = if selected {
        Color::from_rgba(42, 157, 143, 255)
    } else if hovered {
        Color::from_rgba(67, 71, 62, 230)
    } else {
        Color::from_rgba(67, 71, 62, 180)
    };
    draw_rectangle(x, y, width, height, btn_color);

    // Внутрішня обводка (якщо обрано)
    if selected {
        draw_rectangle_lines(x + 3.0, y + 3.0, width - 6.0, height - 6.0, 2.0, Color::from_rgba(255, 255, 255, 100));
    }

    // Зовнішня рамка
    let border_color = if selected {
        Color::from_rgba(42, 157, 143, 255)
    } else {
        Color::from_rgba(236, 232, 220, 120)
    };
    draw_rectangle_lines(x, y, width, height, 2.0, border_color);

    // Текст
    let text_measure = measure_text(label, Some(font), (fs * 1.05) as u16, 1.0);
    let text_color = if selected || hovered {
        Color::from_rgba(236, 232, 220, 255)
    } else {
        Color::from_rgba(236, 232, 220, 200)
    };
    draw_text_ex(
        label,
        x + (width - text_measure.width) / 2.0,
        y + (height + text_measure.height) / 2.0,
        TextParams {
            font: Some(font),
            font_size: (fs * 1.05) as u16,
            color: text_color,
            ..Default::default()
        },
    );
}

// Малює перемикач (toggle)
fn draw_toggle(font: &Font, label: &str, x: f32, y: f32, fs: f32, enabled: bool) -> (bool, bool) {
    // Текст
    draw_text_ex(
        label,
        x,
        y + 15.0,
        TextParams {
            font: Some(font),
            font_size: (fs * 1.1) as u16,
            color: Color::from_rgba(236, 232, 220, 255),
            ..Default::default()
        },
    );

    // Перемикач справа від тексту
    let label_width = measure_text(label, Some(font), (fs * 1.1) as u16, 1.0).width;
    let toggle_x = x + label_width + 20.0;
    let toggle_y = y;
    let toggle_width = 60.0;
    let toggle_height = 28.0;
    let toggle_radius = toggle_height / 2.0;

    let hovered = is_button_hovered(toggle_x, toggle_y, toggle_width, toggle_height);
    let clicked = hovered && is_mouse_button_pressed(MouseButton::Left);

    // Фон перемикача
    let bg_color = if enabled {
        Color::from_rgba(42, 157, 143, 255)
    } else {
        Color::from_rgba(80, 80, 80, 255)
    };

    // Округлений прямокутник (імітація)
    draw_rectangle(toggle_x + toggle_radius, toggle_y, toggle_width - toggle_height, toggle_height, bg_color);
    draw_circle(toggle_x + toggle_radius, toggle_y + toggle_radius, toggle_radius, bg_color);
    draw_circle(toggle_x + toggle_width - toggle_radius, toggle_y + toggle_radius, toggle_radius, bg_color);

    // Рамка
    draw_circle_lines(toggle_x + toggle_radius, toggle_y + toggle_radius, toggle_radius, 2.0, Color::from_rgba(236, 232, 220, 100));
    draw_circle_lines(toggle_x + toggle_width - toggle_radius, toggle_y + toggle_radius, toggle_radius, 2.0, Color::from_rgba(236, 232, 220, 100));
    draw_line(toggle_x + toggle_radius, toggle_y, toggle_x + toggle_width - toggle_radius, toggle_y, 2.0, Color::from_rgba(236, 232, 220, 100));
    draw_line(toggle_x + toggle_radius, toggle_y + toggle_height, toggle_x + toggle_width - toggle_radius, toggle_y + toggle_height, 2.0, Color::from_rgba(236, 232, 220, 100));

    // Кружечок всередині
    let knob_radius = toggle_radius - 4.0;
    let knob_x = if enabled {
        toggle_x + toggle_width - toggle_radius
    } else {
        toggle_x + toggle_radius
    };
    let knob_y = toggle_y + toggle_radius;

    // Тінь кружечка
    draw_circle(knob_x + 1.0, knob_y + 1.0, knob_radius, Color::from_rgba(0, 0, 0, 80));
    // Кружечок
    draw_circle(knob_x, knob_y, knob_radius, Color::from_rgba(255, 255, 255, 255));
    draw_circle_lines(knob_x, knob_y, knob_radius, 2.0, Color::from_rgba(200, 200, 200, 255));

    (clicked, if clicked { !enabled } else { enabled })
}

// Малює слайдер
fn draw_slider(font: &Font, label: &str, x: f32, y: f32, width: f32, fs: f32, value: f32) -> f32 {
    // Текст
    draw_text_ex(
        label,
        x,
        y + 12.0,
        TextParams {
            font: Some(font),
            font_size: (fs * 1.0) as u16,
            color: Color::from_rgba(200, 200, 200, 255),
            ..Default::default()
        },
    );

    // Слайдер
    let slider_x = x + 120.0;
    let slider_y = y;
    let slider_width = width - 250.0;
    let slider_height = 8.0;
    let knob_radius = 12.0;

    // Трек слайдера
    draw_rectangle(slider_x, slider_y + 8.0, slider_width, slider_height, Color::from_rgba(67, 71, 62, 255));
    // Заповнена частина
    let filled_width = slider_width * value;
    draw_rectangle(slider_x, slider_y + 8.0, filled_width, slider_height, Color::from_rgba(42, 157, 143, 255));

    // Кружечок
    let knob_x = slider_x + filled_width;
    let knob_y = slider_y + 12.0;

    let (mx, my) = mouse_position();
    let dragging = is_mouse_button_down(MouseButton::Left) &&
        (mx - knob_x).abs() < 200.0 && (my - knob_y).abs() < 50.0;

    let new_value = if dragging {
        ((mx - slider_x) / slider_width).clamp(0.0, 1.0)
    } else {
        value
    };

    // Малюємо кружечок
    draw_circle(knob_x + 1.0, knob_y + 1.0, knob_radius, Color::from_rgba(0, 0, 0, 80));
    draw_circle(knob_x, knob_y, knob_radius, Color::from_rgba(236, 232, 220, 255));
    draw_circle_lines(knob_x, knob_y, knob_radius, 2.0, Color::from_rgba(42, 157, 143, 255));

    // Відсоток
    let percent_text = format!("{}%", (new_value * 100.0) as i32);
    draw_text_ex(
        &percent_text,
        slider_x + slider_width + 15.0,
        y + 15.0,
        TextParams {
            font: Some(font),
            font_size: (fs * 0.95) as u16,
            color: Color::from_rgba(236, 232, 220, 255),
            ..Default::default()
        },
    );

    new_value
}

// Малює велику action кнопку
fn draw_action_button(
    x: f32, y: f32, width: f32, height: f32,
    label: &str, font: &Font, fs: f32, hovered: bool
) {
    // М'яка тінь
    for i in 0..4 {
        let offset = (4 - i) as f32;
        draw_rectangle(
            x + offset,
            y + offset,
            width,
            height,
            Color::from_rgba(0, 0, 0, 40 - i * 8)
        );
    }

    // Градієнтний фон
    let base_color = if hovered {
        Color::from_rgba(42, 157, 143, 255)
    } else {
        Color::from_rgba(35, 130, 120, 255)
    };

    for i in 0..height as i32 {
        let progress = i as f32 / height;
        let alpha_mod = (1.0 - progress * 0.3) as f32;
        let color = Color::from_rgba(
            (base_color.r as f32 * alpha_mod) as u8,
            (base_color.g as f32 * (alpha_mod * 1.1).min(1.0)) as u8,
            (base_color.b as f32 * alpha_mod) as u8,
            255
        );
        draw_rectangle(x, y + i as f32, width, 1.0, color);
    }

    // Рамки
    draw_rectangle_lines(x, y, width, height, 3.0, Color::from_rgba(42, 157, 143, 255));
    draw_rectangle_lines(x + 3.0, y + 3.0, width - 6.0, height - 6.0, 1.0, Color::from_rgba(255, 255, 255, 80));

    // Текст з тінню
    let text_measure = measure_text(label, Some(font), fs as u16, 1.0);
    let text_x = x + (width - text_measure.width) / 2.0;
    let text_y = y + (height + text_measure.height) / 2.0;

    draw_text_ex(
        label,
        text_x + 2.0,
        text_y + 2.0,
        TextParams {
            font: Some(font),
            font_size: fs as u16,
            color: Color::from_rgba(0, 0, 0, 100),
            ..Default::default()
        },
    );

    draw_text_ex(
        label,
        text_x,
        text_y,
        TextParams {
            font: Some(font),
            font_size: fs as u16,
            color: Color::from_rgba(255, 255, 255, 255),
            ..Default::default()
        },
    );
}

// Сучасний toggle перемикач (iOS стиль)
fn draw_modern_toggle(font: &Font, label: &str, x: f32, y: f32, fs: f32, enabled: bool) -> (bool, bool) {
    // Текст
    draw_text_ex(
        label,
        x,
        y + 12.0,
        TextParams {
            font: Some(font),
            font_size: (fs * 1.1) as u16,
            color: Color::from_rgba(220, 220, 230, 255),
            ..Default::default()
        },
    );

    // Перемикач справа від тексту
    let label_width = measure_text(label, Some(font), (fs * 1.1) as u16, 1.0).width;
    let toggle_x = x + label_width + 25.0;
    let toggle_y = y;
    let toggle_width = 52.0;
    let toggle_height = 28.0;
    let toggle_radius = toggle_height / 2.0;

    let hovered = is_button_hovered(toggle_x, toggle_y, toggle_width, toggle_height);
    let clicked = hovered && is_mouse_button_pressed(MouseButton::Left);

    // Фон перемикача (сучасні кольори)
    let bg_color = if enabled {
        Color::from_rgba(124, 58, 237, 255) // Фіолетовий коли ввімкнено
    } else {
        Color::from_rgba(60, 60, 80, 220) // Темно-сірий коли вимкнено
    };

    // Ефект hover
    let glow_alpha = if hovered { 40 } else { 0 };
    if glow_alpha > 0 {
        draw_circle(toggle_x + toggle_radius, toggle_y + toggle_radius, toggle_radius + 4.0, Color::from_rgba(124, 58, 237, glow_alpha));
        draw_circle(toggle_x + toggle_width - toggle_radius, toggle_y + toggle_radius, toggle_radius + 4.0, Color::from_rgba(124, 58, 237, glow_alpha));
    }

    // Округлений прямокутник (трек)
    draw_rectangle(toggle_x + toggle_radius, toggle_y, toggle_width - toggle_height, toggle_height, bg_color);
    draw_circle(toggle_x + toggle_radius, toggle_y + toggle_radius, toggle_radius, bg_color);
    draw_circle(toggle_x + toggle_width - toggle_radius, toggle_y + toggle_radius, toggle_radius, bg_color);

    // Тонка рамка
    draw_circle_lines(toggle_x + toggle_radius, toggle_y + toggle_radius, toggle_radius, 1.5, Color::from_rgba(255, 255, 255, 30));
    draw_circle_lines(toggle_x + toggle_width - toggle_radius, toggle_y + toggle_radius, toggle_radius, 1.5, Color::from_rgba(255, 255, 255, 30));

    // Кружечок (knob)
    let knob_radius = toggle_radius - 3.0;
    let knob_x = if enabled {
        toggle_x + toggle_width - toggle_radius
    } else {
        toggle_x + toggle_radius
    };
    let knob_y = toggle_y + toggle_radius;

    // М'яка тінь кружечка
    draw_circle(knob_x + 1.0, knob_y + 2.0, knob_radius, Color::from_rgba(0, 0, 0, 120));

    // Білий кружечок з легким градієнтом
    draw_circle(knob_x, knob_y, knob_radius, Color::from_rgba(255, 255, 255, 255));
    draw_circle_lines(knob_x, knob_y, knob_radius, 1.0, Color::from_rgba(220, 220, 230, 150));

    (clicked, if clicked { !enabled } else { enabled })
}

// Сучасний слайдер
fn draw_modern_slider(font: &Font, x: f32, y: f32, width: f32, fs: f32, value: f32) -> f32 {
    let slider_height = 6.0;
    let knob_radius = 10.0;

    // Трек слайдера (темний фон)
    draw_rectangle(
        x,
        y + 10.0,
        width,
        slider_height,
        Color::from_rgba(50, 50, 70, 200)
    );

    // Заповнена частина (градієнт фіолетовий)
    let filled_width = width * value;

    // Градієнт для заповненої частини
    for i in 0..(filled_width as i32) {
        let progress = i as f32 / filled_width;
        let r = (124.0 + progress * 44.0) as u8; // 124 -> 168
        let g = (58.0 + progress * 27.0) as u8;  // 58 -> 85
        let b = 237;
        draw_rectangle(
            x + i as f32,
            y + 10.0,
            1.0,
            slider_height,
            Color::from_rgba(r, g, b, 255)
        );
    }

    // Кружечок (knob)
    let knob_x = x + filled_width;
    let knob_y = y + 13.0;

    let (mx, my) = mouse_position();
    let hovered = (mx - knob_x).abs() < knob_radius * 2.0 && (my - knob_y).abs() < knob_radius * 2.0;
    let dragging = is_mouse_button_down(MouseButton::Left) &&
        (mx - knob_x).abs() < 200.0 && (my - knob_y).abs() < 50.0;

    let new_value = if dragging {
        ((mx - x) / width).clamp(0.0, 1.0)
    } else {
        value
    };

    // Світіння при hover
    if hovered || dragging {
        draw_circle(knob_x, knob_y, knob_radius + 6.0, Color::from_rgba(124, 58, 237, 40));
    }

    // Тінь кружечка
    draw_circle(knob_x + 1.0, knob_y + 2.0, knob_radius, Color::from_rgba(0, 0, 0, 120));

    // Білий кружечок
    draw_circle(knob_x, knob_y, knob_radius, Color::from_rgba(255, 255, 255, 255));
    draw_circle_lines(knob_x, knob_y, knob_radius, 2.0, Color::from_rgba(124, 58, 237, 200));

    // Внутрішній кружечок
    draw_circle(knob_x, knob_y, knob_radius - 4.0, Color::from_rgba(124, 58, 237, hovered as u8 * 100));

    // Відсоток
    let percent_text = format!("{}%", (new_value * 100.0) as i32);
    draw_text_ex(
        &percent_text,
        x + width + 20.0,
        y + 18.0,
        TextParams {
            font: Some(font),
            font_size: (fs * 0.95) as u16,
            color: Color::from_rgba(180, 180, 200, 255),
            ..Default::default()
        },
    );

    new_value
}

