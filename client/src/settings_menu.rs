//! Меню налаштувань — сучасний темний Gaming UI дизайн (Cyber-Clean / Glassmorphism)

use macroquad::prelude::*;
use crate::i18n::Texts;
use crate::settings::{Language, Settings, RebindAction, key_name, save_settings};
use crate::art::{ha, hx, text, text_c};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SettingsCategory {
    General,
    Audio,
    Controls,
}

pub struct SettingsMenu {
    pub category: SettingsCategory,
    pub rebinding: Option<RebindAction>,
}

fn is_button_hovered(x: f32, y: f32, w: f32, h: f32) -> bool {
    let (mx, my) = mouse_position();
    if mx >= x && mx <= x + w && my >= y && my <= y + h {
        return true;
    }
    touches().iter().any(|t| t.position.x >= x && t.position.x <= x + w && t.position.y >= y && t.position.y <= y + h)
}

/// Малює сучасну темну скляну панель з неоновим акцентом
fn draw_modern_panel(x: f32, y: f32, w: f32, h: f32) {
    // М'яка темна тінь
    draw_rectangle(x + 6.0, y + 8.0, w, h, ha(0x000000, 0.6));
    // Глибокий темний обсидіановий фон
    draw_rectangle(x, y, w, h, Color::from_rgba(13, 17, 26, 248));
    // Легкий градієнт/блік зверху
    draw_rectangle(x, y, w, h * 0.3, ha(0xffffff, 0.02));
    // Тонка акцентна рамка
    draw_rectangle_lines(x, y, w, h, 1.5, Color::from_rgba(56, 189, 248, 110));
    draw_rectangle_lines(x + 1.0, y + 1.0, w - 2.0, h - 2.0, 1.0, ha(0xffffff, 0.04));
    // Верхня неонова лінія
    draw_rectangle(x, y, w, 3.0, Color::from_rgba(56, 189, 248, 230));
}

/// Сучасна кнопка
fn draw_modern_button(font: &Font, x: f32, y: f32, w: f32, h: f32, label: &str, fs: f32, hovered: bool, primary: bool) {
    let by = y + if hovered { -2.0 } else { 0.0 };
    // Тінь
    draw_rectangle(x + 2.0, by + 3.0, w, h, ha(0x000000, 0.35));

    let bg = if primary {
        if hovered { Color::from_rgba(14, 165, 233, 255) } else { Color::from_rgba(2, 132, 199, 230) }
    } else {
        if hovered { Color::from_rgba(51, 65, 85, 240) } else { Color::from_rgba(30, 41, 59, 220) }
    };
    draw_rectangle(x, by, w, h, bg);

    // Рамка кнопки
    let border = if primary {
        if hovered { Color::from_rgba(125, 211, 252, 255) } else { Color::from_rgba(56, 189, 248, 200) }
    } else {
        if hovered { Color::from_rgba(148, 163, 184, 180) } else { Color::from_rgba(71, 85, 105, 150) }
    };
    draw_rectangle_lines(x, by, w, h, 1.5, border);

    // Текст кнопки
    let text_color = if primary {
        WHITE
    } else if hovered {
        Color::from_rgba(241, 245, 249, 255)
    } else {
        Color::from_rgba(203, 213, 225, 255)
    };
    let measure = measure_text(label, Some(font), fs as u16, 1.0);
    let tx = x + (w - measure.width) / 2.0;
    let ty = by + (h + measure.height) / 2.0;

    draw_text_ex(label, tx, ty, TextParams {
        font: Some(font),
        font_size: fs as u16,
        color: text_color,
        ..Default::default()
    });
}

impl SettingsMenu {
    pub fn new() -> Self {
        Self {
            category: SettingsCategory::General,
            rebinding: None,
        }
    }

    /// Малює меню налаштувань, повертає (close_settings, disconnect)
    pub fn draw(&mut self, font: &Font, texts: &Texts, settings: &mut Settings) -> (bool, bool) {
        let (sw, sh) = (screen_width(), screen_height());
        let fs = (sw.min(sh) * 0.038).clamp(15.0, 24.0);

        // Обробка зміни клавіші в режимі ребінду
        if let Some(action) = self.rebinding {
            if let Some(key) = get_last_key_pressed() {
                if key == KeyCode::Escape {
                    // Скасування зміни
                    self.rebinding = None;
                } else {
                    let k_name = key_name(key);
                    settings.keybindings.set_key_for_action(action, k_name.to_string());
                    save_settings(settings);
                    self.rebinding = None;
                }
            }
        }

        // Затемнення фону гри
        draw_rectangle(0.0, 0.0, sw, sh, ha(0x060911, 0.78));

        // Головна панель налаштувань
        let pw = (sw * 0.72).clamp(520.0, 780.0);
        let ph = (sh * 0.84).clamp(460.0, 680.0);
        let px = (sw - pw) / 2.0;
        let py = (sh - ph) / 2.0;

        draw_modern_panel(px, py, pw, ph);

        // Заголовок та підзаголовок
        let title_y = py + fs * 1.8;
        text(font, &format!("⚙️ {}", texts.settings), px + 35.0, title_y, fs * 1.35, Color::from_rgba(241, 245, 249, 255));
        text(font, texts.keybindings, px + 35.0, title_y + fs * 1.0, fs * 0.75, Color::from_rgba(148, 163, 184, 200));

        // Кнопка закриття (✕) у верхньому правому кутку
        let close_btn_size = fs * 1.8;
        let close_btn_x = px + pw - close_btn_size - 25.0;
        let close_btn_y = py + 20.0;
        let close_hovered = is_button_hovered(close_btn_x, close_btn_y, close_btn_size, close_btn_size);

        if close_hovered {
            draw_rectangle(close_btn_x, close_btn_y, close_btn_size, close_btn_size, ha(0xef4444, 0.25));
            draw_rectangle_lines(close_btn_x, close_btn_y, close_btn_size, close_btn_size, 1.5, hx(0xef4444));
        }
        text_c(font, "✕", close_btn_x + close_btn_size / 2.0, close_btn_y + close_btn_size * 0.72, fs * 1.1, if close_hovered { hx(0xff6b6b) } else { ha(0xffffff, 0.6) });

        let mut should_close = close_hovered && is_mouse_button_pressed(MouseButton::Left);

        // Сучасні вкладки налаштувань (Tabs)
        let tab_y = title_y + fs * 1.8;
        let tab_h = fs * 2.2;
        let tabs = [
            (SettingsCategory::General, "🌐", texts.general_tab),
            (SettingsCategory::Audio, "🎵", texts.audio_tab),
            (SettingsCategory::Controls, "⌨️", texts.controls_tab),
        ];

        let tab_w = (pw - 70.0 - 20.0) / tabs.len() as f32;

        for (i, (cat, icon, label)) in tabs.iter().enumerate() {
            let tx = px + 35.0 + i as f32 * (tab_w + 10.0);
            let active = self.category == *cat;
            let hovered = is_button_hovered(tx, tab_y, tab_w, tab_h);

            let bg = if active {
                Color::from_rgba(30, 41, 59, 255)
            } else if hovered {
                Color::from_rgba(20, 27, 41, 220)
            } else {
                Color::from_rgba(15, 23, 42, 160)
            };
            draw_rectangle(tx, tab_y, tab_w, tab_h, bg);

            // Нижня смужка активної вкладки
            if active {
                draw_rectangle(tx, tab_y + tab_h - 2.5, tab_w, 2.5, Color::from_rgba(56, 189, 248, 255));
                draw_rectangle_lines(tx, tab_y, tab_w, tab_h, 1.0, Color::from_rgba(56, 189, 248, 140));
            } else {
                draw_rectangle_lines(tx, tab_y, tab_w, tab_h, 1.0, ha(0xffffff, if hovered { 0.12 } else { 0.05 }));
            }

            let tab_title = format!("{} {}", icon, label);
            let tab_color = if active {
                Color::from_rgba(56, 189, 248, 255)
            } else if hovered {
                Color::from_rgba(241, 245, 249, 255)
            } else {
                Color::from_rgba(148, 163, 184, 200)
            };
            text_c(font, &tab_title, tx + tab_w / 2.0, tab_y + tab_h * 0.65, fs * 0.95, tab_color);

            if hovered && is_mouse_button_pressed(MouseButton::Left) {
                self.category = *cat;
                self.rebinding = None;
            }
        }

        // Область вмісту
        let content_y = tab_y + tab_h + fs * 1.0;
        let content_h = ph - (content_y - py) - fs * 3.5;
        let content_rect = Rect::new(px + 35.0, content_y, pw - 70.0, content_h);

        match self.category {
            SettingsCategory::General => self.draw_modern_general(font, texts, settings, content_rect, fs),
            SettingsCategory::Audio => self.draw_modern_audio(font, texts, settings, content_rect, fs),
            SettingsCategory::Controls => self.draw_modern_controls(font, texts, settings, content_rect, fs),
        }

        // Нижня панель: Кнопка "Закрити"
        let bottom_btn_w = fs * 9.0;
        let bottom_btn_h = fs * 2.1;
        let bottom_btn_x = px + (pw - bottom_btn_w) / 2.0;
        let bottom_btn_y = py + ph - bottom_btn_h - fs * 0.8;
        let btn_hovered = is_button_hovered(bottom_btn_x, bottom_btn_y, bottom_btn_w, bottom_btn_h);
        draw_modern_button(font, bottom_btn_x, bottom_btn_y, bottom_btn_w, bottom_btn_h, texts.close, fs * 0.95, btn_hovered, false);

        if btn_hovered && is_mouse_button_pressed(MouseButton::Left) {
            should_close = true;
        }

        if is_key_pressed(KeyCode::Escape) && self.rebinding.is_none() {
            should_close = true;
        }

        (should_close, false)
    }

    /// Вкладка "Загальні" (Мова + Сервер)
    fn draw_modern_general(&mut self, font: &Font, texts: &Texts, settings: &mut Settings, rect: Rect, fs: f32) {
        // Секція вибору мови
        text(font, texts.language, rect.x, rect.y + fs * 0.9, fs * 1.05, Color::from_rgba(241, 245, 249, 255));

        let languages = [
            ("🇺🇦 Українська", Language::Ukrainian),
            ("🇬🇧 English", Language::English),
            ("🇷🇺 Русский", Language::Russian),
        ];

        let lang_btn_w = (rect.w - 20.0) / 3.0;
        let lang_btn_h = fs * 3.2;
        let lang_y = rect.y + fs * 1.6;

        for (i, (name, lang)) in languages.iter().enumerate() {
            let bx = rect.x + i as f32 * (lang_btn_w + 10.0);
            let selected = settings.language == *lang;
            let hovered = is_button_hovered(bx, lang_y, lang_btn_w, lang_btn_h);

            let bg = if selected {
                Color::from_rgba(14, 116, 144, 200)
            } else if hovered {
                Color::from_rgba(30, 41, 59, 230)
            } else {
                Color::from_rgba(15, 23, 42, 180)
            };
            draw_rectangle(bx, lang_y, lang_btn_w, lang_btn_h, bg);

            let border = if selected {
                Color::from_rgba(56, 189, 248, 255)
            } else if hovered {
                Color::from_rgba(100, 116, 139, 200)
            } else {
                Color::from_rgba(51, 65, 85, 140)
            };
            draw_rectangle_lines(bx, lang_y, lang_btn_w, lang_btn_h, if selected { 2.0 } else { 1.0 }, border);

            text_c(font, name, bx + lang_btn_w / 2.0, lang_y + lang_btn_h * 0.6, fs * 0.95, if selected { WHITE } else { Color::from_rgba(203, 213, 225, 255) });

            if hovered && is_mouse_button_pressed(MouseButton::Left) {
                settings.language = *lang;
                save_settings(settings);
            }
        }

        // Секція вибору сервера
        let srv_section_y = lang_y + lang_btn_h + fs * 1.8;
        text(font, texts.server_label, rect.x, srv_section_y + fs * 0.9, fs * 1.05, Color::from_rgba(241, 245, 249, 255));

        let srv_card_w = rect.w;
        let srv_card_h = fs * 4.2;
        let srv_card_y = srv_section_y + fs * 1.6;
        let srv_hovered = is_button_hovered(rect.x, srv_card_y, srv_card_w, srv_card_h);

        let srv_bg = if srv_hovered { Color::from_rgba(30, 41, 59, 220) } else { Color::from_rgba(15, 23, 42, 180) };
        draw_rectangle(rect.x, srv_card_y, srv_card_w, srv_card_h, srv_bg);
        draw_rectangle_lines(rect.x, srv_card_y, srv_card_w, srv_card_h, 1.5, if srv_hovered { Color::from_rgba(56, 189, 248, 200) } else { Color::from_rgba(51, 65, 85, 150) });

        // Іконка та назва сервера
        text(font, "🌐", rect.x + 20.0, srv_card_y + srv_card_h * 0.6, fs * 1.6, WHITE);
        text(font, settings.server_type.name(), rect.x + 70.0, srv_card_y + fs * 1.6, fs * 1.1, Color::from_rgba(56, 189, 248, 255));
        text(font, settings.server_type.url(), rect.x + 70.0, srv_card_y + fs * 2.8, fs * 0.8, Color::from_rgba(148, 163, 184, 200));

        // Кнопка перемикання справа
        let change_btn_w = fs * 8.0;
        let change_btn_h = fs * 2.2;
        let change_btn_x = rect.x + srv_card_w - change_btn_w - 20.0;
        let change_btn_y = srv_card_y + (srv_card_h - change_btn_h) / 2.0;
        let ch_hovered = is_button_hovered(change_btn_x, change_btn_y, change_btn_w, change_btn_h);
        draw_modern_button(font, change_btn_x, change_btn_y, change_btn_w, change_btn_h, texts.click_to_change, fs * 0.8, ch_hovered, false);

        if (srv_hovered || ch_hovered) && is_mouse_button_pressed(MouseButton::Left) {
            settings.server_type = settings.server_type.next();
            save_settings(settings);
        }

        // Секція мобільного інтерфейсу (для сенсорних екранів або тестування)
        let mob_section_y = srv_card_y + srv_card_h + fs * 1.2;
        let mob_card_w = rect.w;
        let mob_card_h = fs * 3.6;
        let mob_bg = Color::from_rgba(15, 23, 42, 180);
        draw_rectangle(rect.x, mob_section_y, mob_card_w, mob_card_h, mob_bg);
        draw_rectangle_lines(rect.x, mob_section_y, mob_card_w, mob_card_h, 1.0, Color::from_rgba(51, 65, 85, 150));
        text(font, "📱", rect.x + 20.0, mob_section_y + mob_card_h * 0.6, fs * 1.5, WHITE);
        text(font, texts.mobile_mode, rect.x + 65.0, mob_section_y + fs * 1.4, fs * 1.0, Color::from_rgba(241, 245, 249, 255));
        text(font, texts.mobile_mode_desc, rect.x + 65.0, mob_section_y + fs * 2.6, fs * 0.78, Color::from_rgba(148, 163, 184, 180));

        let mob_toggle_x = rect.x + mob_card_w - 75.0;
        let mob_toggle_y = mob_section_y + (mob_card_h - 26.0) / 2.0;
        let (mob_toggled, mob_val) = self.draw_modern_toggle(font, texts, mob_toggle_x, mob_toggle_y, settings.mobile_mode);
        if mob_toggled {
            settings.mobile_mode = mob_val;
            save_settings(settings);
        }
    }

    /// Вкладка "Звук" (Музика та Ефекти)
    fn draw_modern_audio(&mut self, font: &Font, texts: &Texts, settings: &mut Settings, rect: Rect, fs: f32) {
        let card_w = rect.w;
        let card_h = fs * 4.6;

        // 1) Музика
        let y_music = rect.y + fs * 0.5;
        draw_rectangle(rect.x, y_music, card_w, card_h, Color::from_rgba(15, 23, 42, 180));
        draw_rectangle_lines(rect.x, y_music, card_w, card_h, 1.0, Color::from_rgba(51, 65, 85, 150));

        text(font, &format!("🎵 {}", texts.music), rect.x + 20.0, y_music + fs * 1.5, fs * 1.1, Color::from_rgba(241, 245, 249, 255));

        // Toggle перемикач
        let toggle_x = rect.x + card_w - 75.0;
        let toggle_y = y_music + fs * 0.7;
        let (toggled, new_val) = self.draw_modern_toggle(font, texts, toggle_x, toggle_y, settings.music_enabled);
        if toggled {
            settings.music_enabled = new_val;
            save_settings(settings);
        }

        // Слайдер гучності музики
        if settings.music_enabled {
            let slider_y = y_music + fs * 2.8;
            let slider_w = card_w - 40.0;
            settings.music_volume = self.draw_modern_slider(font, texts.music_volume, rect.x + 20.0, slider_y, slider_w, fs, settings.music_volume);
        }

        // 2) Звуки
        let y_sound = y_music + card_h + fs * 1.2;
        draw_rectangle(rect.x, y_sound, card_w, card_h, Color::from_rgba(15, 23, 42, 180));
        draw_rectangle_lines(rect.x, y_sound, card_w, card_h, 1.0, Color::from_rgba(51, 65, 85, 150));

        text(font, &format!("🔊 {}", texts.sounds), rect.x + 20.0, y_sound + fs * 1.5, fs * 1.1, Color::from_rgba(241, 245, 249, 255));

        let toggle2_x = rect.x + card_w - 75.0;
        let toggle2_y = y_sound + fs * 0.7;
        let (toggled2, new_val2) = self.draw_modern_toggle(font, texts, toggle2_x, toggle2_y, settings.sounds_enabled);
        if toggled2 {
            settings.sounds_enabled = new_val2;
            save_settings(settings);
        }

        // Слайдер гучності звуків
        if settings.sounds_enabled {
            let slider_y = y_sound + fs * 2.8;
            let slider_w = card_w - 40.0;
            settings.sound_volume = self.draw_modern_slider(font, texts.sound_volume, rect.x + 20.0, slider_y, slider_w, fs, settings.sound_volume);
        }
    }

    /// Вкладка "Керування та Бінди"
    fn draw_modern_controls(&mut self, font: &Font, texts: &Texts, settings: &mut Settings, rect: Rect, fs: f32) {
        // Підказка
        text(font, texts.rebind_hint, rect.x, rect.y + fs * 0.6, fs * 0.85, Color::from_rgba(148, 163, 184, 220));

        let actions = RebindAction::ALL;
        let cols = 2;
        let item_w = (rect.w - 15.0) / cols as f32;
        let item_h = fs * 2.5;
        let start_y = rect.y + fs * 1.2;

        for (idx, &action) in actions.iter().enumerate() {
            let col = idx % cols;
            let row = idx / cols;

            let bx = rect.x + col as f32 * (item_w + 15.0);
            let by = start_y + row as f32 * (item_h + 8.0);

            let is_rebinding_this = self.rebinding == Some(action);
            let current_key = settings.keybindings.get_key_for_action(action);

            let card_hovered = is_button_hovered(bx, by, item_w, item_h);

            // Фон картки
            let bg = if is_rebinding_this {
                Color::from_rgba(30, 58, 138, 220)
            } else if card_hovered {
                Color::from_rgba(30, 41, 59, 200)
            } else {
                Color::from_rgba(15, 23, 42, 160)
            };
            draw_rectangle(bx, by, item_w, item_h, bg);

            // Рамка картки
            let border = if is_rebinding_this {
                let pulse = (macroquad::miniquad::date::now() * 6.0).sin().abs() as f32;
                Color::new(0.2, 0.8, 1.0, 0.6 + 0.4 * pulse)
            } else if card_hovered {
                Color::from_rgba(56, 189, 248, 180)
            } else {
                Color::from_rgba(51, 65, 85, 140)
            };
            draw_rectangle_lines(bx, by, item_w, item_h, if is_rebinding_this { 2.0 } else { 1.0 }, border);

            // Іконка та назва дії
            let action_name = match action {
                RebindAction::Inventory => texts.inventory,
                RebindAction::Craft => texts.craft,
                RebindAction::Action => texts.action,
                RebindAction::Eat => texts.eat,
                RebindAction::Drink => texts.drink,
                RebindAction::Dig => texts.dig,
                RebindAction::Menu => texts.menu,
                RebindAction::MoveUp => texts.move_up,
                RebindAction::MoveDown => texts.move_down,
                RebindAction::MoveLeft => texts.move_left,
                RebindAction::MoveRight => texts.move_right,
            };
            let title = format!("{} {}", action.icon(), action_name);
            text(font, &title, bx + 12.0, by + item_h * 0.62, fs * 0.85, Color::from_rgba(241, 245, 249, 255));

            // Пігулка клавіші справа
            let key_box_w = if is_rebinding_this { fs * 7.5 } else { fs * 4.2 };
            let key_box_h = item_h - 10.0;
            let key_box_x = bx + item_w - key_box_w - 6.0;
            let key_box_y = by + 5.0;

            let k_bg = if is_rebinding_this {
                Color::from_rgba(14, 165, 233, 240)
            } else {
                Color::from_rgba(30, 41, 59, 255)
            };
            draw_rectangle(key_box_x, key_box_y, key_box_w, key_box_h, k_bg);
            draw_rectangle_lines(key_box_x, key_box_y, key_box_w, key_box_h, 1.0, if is_rebinding_this { WHITE } else { Color::from_rgba(100, 116, 139, 180) });

            let key_label = if is_rebinding_this {
                "..."
            } else {
                current_key
            };
            text_c(font, key_label, key_box_x + key_box_w / 2.0, key_box_y + key_box_h * 0.68, fs * 0.88, if is_rebinding_this { WHITE } else { Color::from_rgba(56, 189, 248, 255) });

            if card_hovered && is_mouse_button_pressed(MouseButton::Left) {
                if self.rebinding == Some(action) {
                    self.rebinding = None;
                } else {
                    self.rebinding = Some(action);
                }
            }
        }

        // Кнопка "Скинути бінди за замовчуванням"
        let reset_btn_w = fs * 11.0;
        let reset_btn_h = fs * 2.0;
        let reset_btn_x = rect.x + (rect.w - reset_btn_w) / 2.0;
        let reset_btn_y = start_y + (actions.len() as f32 / cols as f32).ceil() * (item_h + 8.0) + fs * 0.4;

        let r_hovered = is_button_hovered(reset_btn_x, reset_btn_y, reset_btn_w, reset_btn_h);
        draw_modern_button(font, reset_btn_x, reset_btn_y, reset_btn_w, reset_btn_h, &format!("🔄 {}", texts.reset_defaults), fs * 0.82, r_hovered, false);

        if r_hovered && is_mouse_button_pressed(MouseButton::Left) {
            settings.keybindings = Default::default();
            save_settings(settings);
            self.rebinding = None;
        }
    }

    /// Сучасний toggle перемикач
    fn draw_modern_toggle(&self, _font: &Font, _texts: &Texts, x: f32, y: f32, enabled: bool) -> (bool, bool) {
        let tw = 50.0;
        let th = 26.0;
        let hovered = is_button_hovered(x, y, tw, th);

        // Капсула
        let bg = if enabled {
            Color::from_rgba(16, 185, 129, 240) // Неоновий смарагдовий
        } else {
            Color::from_rgba(51, 65, 85, 200) // Спокійний темний
        };
        draw_rectangle(x, y, tw, th, bg);
        draw_rectangle_lines(x, y, tw, th, 1.5, if enabled { Color::from_rgba(52, 211, 153, 255) } else { Color::from_rgba(100, 116, 139, 180) });

        // Повзунок-коло
        let r = th * 0.38;
        let cx = if enabled { x + tw - r - 4.0 } else { x + r + 4.0 };
        let cy = y + th / 2.0;
        draw_circle(cx, cy, r, WHITE);

        if hovered && is_mouse_button_pressed(MouseButton::Left) {
            (true, !enabled)
        } else {
            (false, enabled)
        }
    }

    /// Сучасний повзунок (Slider)
    fn draw_modern_slider(&self, font: &Font, label: &str, x: f32, y: f32, width: f32, fs: f32, mut value: f32) -> f32 {
        let sh = 7.0;
        let sy = y + fs * 0.4;

        // Фон доріжки
        draw_rectangle(x, sy, width, sh, Color::from_rgba(30, 41, 59, 255));
        draw_rectangle_lines(x, sy, width, sh, 1.0, Color::from_rgba(51, 65, 85, 180));

        // Заповнення (неоновий градієнт)
        let fill_w = (width * value).clamp(0.0, width);
        if fill_w > 0.0 {
            draw_rectangle(x, sy, fill_w, sh, Color::from_rgba(56, 189, 248, 255));
            draw_rectangle(x, sy, fill_w, sh * 0.4, ha(0xffffff, 0.25));
        }

        // Повзунок
        let thumb_r = 8.0;
        let thumb_x = x + fill_w;
        let thumb_y = sy + sh / 2.0;
        draw_circle(thumb_x, thumb_y, thumb_r, WHITE);
        draw_circle_lines(thumb_x, thumb_y, thumb_r, 2.0, Color::from_rgba(56, 189, 248, 255));

        // Відсоток справа над слайдером
        let pct_str = format!("{}: {}%", label, (value * 100.0) as i32);
        text(font, &pct_str, x, y - 2.0, fs * 0.8, Color::from_rgba(203, 213, 225, 255));

        // Клік або перетягування
        let hit_rect = Rect::new(x - 10.0, sy - 15.0, width + 20.0, sh + 30.0);
        if is_mouse_button_down(MouseButton::Left) {
            let (mx, my) = mouse_position();
            if hit_rect.contains(vec2(mx, my)) {
                value = ((mx - x) / width).clamp(0.0, 1.0);
            }
        }

        value
    }
}
