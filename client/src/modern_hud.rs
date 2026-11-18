//! Сучасний HUD (Heads-Up Display) для Pixel Village
//! Використовує єдину дизайн-систему

use macroquad::prelude::*;
use crate::design_system::*;
use crate::i18n::Texts;
use crate::game::Game;

/// Малює повний HUD гри
pub fn draw_modern_hud(
    game: &Game,
    font: &Font,
    texts: &Texts,
    connected: bool,
    online: usize,
    others_names: &[String],
    px: f32,
    py: f32,
) {
    let sw = screen_width();
    let sh = screen_height();

    // Статус-бари персонажа (верхній лівий кут)
    draw_character_stats(game, font, texts);

    // Інформація про підключення
    draw_connection_info(font, texts, connected, online, others_names);

    // Міні-карта та статистика (верхній правий кут)
    draw_minimap_and_stats(game, font, texts, px, py, sw);

    // Інвентар та інструменти (нижня частина)
    draw_inventory_bar(game, font, texts, sw, sh);
}

/// Малює статус-бари персонажа
fn draw_character_stats(game: &Game, font: &Font, texts: &Texts) {
    let padding = Spacing::MEDIUM;
    let bar_width = 220.0;
    let bar_height = 28.0;
    let bar_spacing = 8.0;

    let stats = [
        (game.hp, 100.0, "❤️", texts.health, ColorPalette::HEALTH_FULL),
        (game.hunger, 100.0, "🍖", texts.hunger, ColorPalette::HUNGER_FULL),
        (game.thirst, 100.0, "💧", texts.thirst, ColorPalette::THIRST_FULL),
        (game.energy, 100.0, "⚡", texts.energy, ColorPalette::ENERGY_FULL),
    ];

    for (i, (value, max_value, icon, label, color)) in stats.iter().enumerate() {
        let y = padding + i as f32 * (bar_height + bar_spacing);

        draw_stat_bar(
            padding,
            y,
            bar_width,
            bar_height,
            *value,
            *max_value,
            label,
            icon,
            *color,
            font
        );
    }
}

/// Малює один статус-бар з градієнтом та ефектами
fn draw_stat_bar(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    value: f32,
    max_value: f32,
    label: &str,
    icon: &str,
    fg_color: Color,
    font: &Font
) {
    let progress = (value / max_value).clamp(0.0, 1.0);

    // Тінь бару
    draw_rectangle(
        x + 3.0,
        y + 3.0,
        w,
        h,
        ColorPalette::SHADOW_MEDIUM
    );

    // Темний фон
    draw_rectangle(x, y, w, h, ColorPalette::PANEL_BG_DARK);

    // Критичний стан (мигання червоним)
    if progress < 0.25 {
        let pulse = ((get_time() * 3.0).sin() * 0.3 + 0.7) as f32;
        let warning_color = Color::new(
            fg_color.r * pulse,
            fg_color.g * 0.5,
            fg_color.b * 0.5,
            1.0
        );
        draw_rectangle(x + 2.0, y + 2.0, (w - 4.0) * progress, h - 4.0, warning_color);
    } else {
        // Нормальний стан
        draw_rectangle(x + 2.0, y + 2.0, (w - 4.0) * progress, h - 4.0, fg_color);
    }

    // Світлий градієнт зверху для об'єму
    for i in 0..5 {
        let alpha = (1.0 - i as f32 / 5.0) * 0.25;
        let grad_h = (h - 4.0) * (1.0 - i as f32 / 5.0);
        draw_rectangle(
            x + 2.0,
            y + 2.0,
            (w - 4.0) * progress,
            grad_h,
            Color::new(1.0, 1.0, 1.0, alpha)
        );
    }

    // Зовнішня рамка
    draw_rectangle_lines(x, y, w, h, Spacing::BORDER_MEDIUM, ColorPalette::BORDER_MAIN);

    // Внутрішня світла рамка
    draw_rectangle_lines(
        x + 2.0,
        y + 2.0,
        w - 4.0,
        h - 4.0,
        1.0,
        ColorPalette::BORDER_LIGHT
    );

    // Поділки на барі (кожні 25%)
    for i in 1..4 {
        let tick_x = x + 2.0 + (w - 4.0) * (i as f32 * 0.25);
        draw_line(
            tick_x,
            y + 2.0,
            tick_x,
            y + h - 2.0,
            1.0,
            Color::new(0.0, 0.0, 0.0, 0.3)
        );
    }

    // Іконка та текст з тінню
    let font_size = Typography::scale_font_size(Typography::SIZE_SMALL, screen_height());
    let text = format!("{} {}: {:.0}/{:.0}", icon, label, value, max_value);

    // Тінь тексту
    draw_text_ex(
        &text,
        x + 7.0,
        y + h / 2.0 + font_size as f32 / 3.0 + 1.0,
        TextParams {
            font: Some(font),
            font_size,
            color: Color::new(0.0, 0.0, 0.0, 0.8),
            ..Default::default()
        }
    );

    // Основний текст
    draw_text_ex(
        &text,
        x + 6.0,
        y + h / 2.0 + font_size as f32 / 3.0,
        TextParams {
            font: Some(font),
            font_size,
            color: ColorPalette::TEXT_PRIMARY,
            ..Default::default()
        }
    );
}

/// Малює інформацію про підключення
fn draw_connection_info(
    font: &Font,
    texts: &Texts,
    connected: bool,
    online: usize,
    others_names: &[String],
) {
    let padding = Spacing::MEDIUM;
    let y_start = padding + 4.0 * (28.0 + 8.0) + 10.0; // Після статус-барів

    let font_size = Typography::scale_font_size(Typography::SIZE_SMALL, screen_height());

    // Статус підключення
    let (status_text, status_color) = if connected {
        (texts.online, ColorPalette::BUTTON_SUCCESS)
    } else {
        ("❌ Offline", ColorPalette::BUTTON_DANGER)
    };

    // Індикатор підключення
    draw_circle(
        padding + 8.0,
        y_start + 2.0,
        6.0,
        status_color
    );

    // Обводка індикатора
    draw_circle_lines(
        padding + 8.0,
        y_start + 2.0,
        6.0,
        2.0,
        ColorPalette::BORDER_DARK
    );

    // Текст статусу
    let online_count = if connected { online.max(1) } else { 0 };
    let status_full = format!("{}: {} ({})", texts.online, online_count, status_text);

    draw_text_ex(
        &status_full,
        padding + 20.0,
        y_start + font_size as f32 / 2.0,
        TextParams {
            font: Some(font),
            font_size,
            color: ColorPalette::TEXT_PRIMARY,
            ..Default::default()
        }
    );

    // Список гравців онлайн
    for (i, name) in others_names.iter().take(5).enumerate() {
        let y = y_start + (i + 1) as f32 * (font_size as f32 * 1.3);

        draw_text_ex(
            &format!("• {}", name),
            padding + 10.0,
            y,
            TextParams {
                font: Some(font),
                font_size: (font_size as f32 * 0.9) as u16,
                color: ColorPalette::TEXT_SECONDARY,
                ..Default::default()
            }
        );
    }
}

/// Малює міні-карту та статистику
fn draw_minimap_and_stats(
    game: &Game,
    font: &Font,
    texts: &Texts,
    px: f32,
    py: f32,
    sw: f32,
) {
    let padding = Spacing::MEDIUM;
    let map_size = 180.0;
    let x = sw - map_size - padding;
    let y = padding;

    // Панель для міні-карти
    draw_rectangle(
        x - 4.0,
        y - 4.0,
        map_size + 8.0,
        map_size + 8.0,
        ColorPalette::PANEL_BG_DARK
    );

    // Рамка карти
    draw_rectangle_lines(
        x - 4.0,
        y - 4.0,
        map_size + 8.0,
        map_size + 8.0,
        Spacing::BORDER_MEDIUM,
        ColorPalette::BORDER_ACCENT
    );

    // Фон карти (темно-зелений)
    draw_rectangle(x, y, map_size, map_size, ColorPalette::GRASS_DARK);

    // Сітка карти
    let grid_size = 20.0;
    for i in 0..10 {
        let grid_pos = i as f32 * grid_size;
        // Вертикальні лінії
        draw_line(
            x + grid_pos,
            y,
            x + grid_pos,
            y + map_size,
            1.0,
            Color::new(0.0, 0.0, 0.0, 0.15)
        );
        // Горизонтальні лінії
        draw_line(
            x,
            y + grid_pos,
            x + map_size,
            y + grid_pos,
            1.0,
            Color::new(0.0, 0.0, 0.0, 0.15)
        );
    }

    // Вогнища
    for fire in &game.fires {
        let map_x = x + (fire.0 / 80.0) * map_size;
        let map_y = y + (fire.1 / 80.0) * map_size;
        draw_circle(map_x, map_y, 3.0, Color::new(1.0, 0.5, 0.0, 0.8));
    }

    // Позиція гравця
    let player_x = x + (px / 80.0) * map_size;
    let player_y = y + (py / 80.0) * map_size;

    // Тінь гравця
    draw_circle(player_x + 1.0, player_y + 1.0, 5.0, Color::new(0.0, 0.0, 0.0, 0.5));

    // Гравець (пульсуючий)
    let pulse = ((get_time() * 2.0).sin() * 0.2 + 0.8) as f32;
    draw_circle(player_x, player_y, 5.0 * pulse, Color::new(1.0, 1.0, 0.0, 1.0));
    draw_circle_lines(player_x, player_y, 5.0 * pulse, 2.0, ColorPalette::BORDER_DARK);

    // Статистика під картою
    let stats_y = y + map_size + 12.0;
    let font_size = Typography::scale_font_size(Typography::SIZE_SMALL, screen_height());

    let stats = [
        (format!("💰 {}", game.gold), ColorPalette::TEXT_ACCENT),
        (format!("📊 {}", game.score()), ColorPalette::TEXT_PRIMARY),
    ];

    for (i, (stat_text, color)) in stats.iter().enumerate() {
        let stat_y = stats_y + i as f32 * (font_size as f32 * 1.4);

        // Фон для статистики
        let text_width = measure_text(&stat_text, Some(font), font_size, 1.0).width;
        draw_rectangle(
            x - 4.0,
            stat_y - font_size as f32 * 0.7,
            map_size + 8.0,
            font_size as f32 * 1.2,
            ColorPalette::PANEL_BG_DARK
        );

        // Рамка
        draw_rectangle_lines(
            x - 4.0,
            stat_y - font_size as f32 * 0.7,
            map_size + 8.0,
            font_size as f32 * 1.2,
            1.0,
            ColorPalette::BORDER_MAIN
        );

        // Текст статистики
        draw_text_ex(
            &stat_text,
            x + (map_size - text_width) / 2.0,
            stat_y,
            TextParams {
                font: Some(font),
                font_size,
                color: *color,
                ..Default::default()
            }
        );
    }
}

/// Малює панель інвентаря внизу
fn draw_inventory_bar(
    game: &Game,
    font: &Font,
    texts: &Texts,
    sw: f32,
    sh: f32,
) {
    let padding = Spacing::MEDIUM;
    let chip_height = 32.0;
    let chip_spacing = 6.0;
    let max_width = sw * 0.5;

    let mut x = padding;
    let y = sh - padding - chip_height;

    // Збираємо всі предмети
    let mut items: Vec<(Color, String)> = Vec::new();

    // Предмети з інвентаря
    for (i, &count) in game.inv.iter().enumerate() {
        if count > 0 {
            let color = Color::from_rgba(
                ((crate::data::IT_COLOR[i] >> 16) & 0xFF) as u8,
                ((crate::data::IT_COLOR[i] >> 8) & 0xFF) as u8,
                (crate::data::IT_COLOR[i] & 0xFF) as u8,
                255
            );
            items.push((color, format!("{} ×{}", texts.item_names[i], count)));
        }
    }

    // Інструменти
    for (t, &dur) in game.tools.iter().enumerate() {
        if dur > 0 {
            let label = if crate::data::TL_DUR[t] > 1 {
                format!("{} ({})", texts.tool_names[t], dur)
            } else {
                texts.tool_names[t].to_string()
            };
            items.push((ColorPalette::TEXT_PRIMARY, label));
        }
    }

    // Малюємо чіпси справа наліво
    for (color, label) in items.iter().rev() {
        let font_size = Typography::scale_font_size(Typography::SIZE_SMALL, screen_height());
        let text_width = measure_text(&label, Some(font), font_size, 1.0).width;
        let chip_width = text_width + Spacing::MEDIUM * 2.0;

        // Перевірка чи влізає
        if x + chip_width > max_width {
            break;
        }

        // Фон чіпа
        draw_rectangle(x, y, chip_width, chip_height, ColorPalette::PANEL_BG);

        // Кольорова смужка зліва
        draw_rectangle(x, y, 4.0, chip_height, *color);

        // Рамка
        draw_rectangle_lines(
            x,
            y,
            chip_width,
            chip_height,
            Spacing::BORDER_THIN,
            ColorPalette::BORDER_MAIN
        );

        // Текст
        draw_text_ex(
            &label,
            x + Spacing::MEDIUM,
            y + chip_height / 2.0 + font_size as f32 / 3.0,
            TextParams {
                font: Some(font),
                font_size,
                color: ColorPalette::TEXT_PRIMARY,
                ..Default::default()
            }
        );

        x += chip_width + chip_spacing;
    }
}
