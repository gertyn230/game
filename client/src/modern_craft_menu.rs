//! Сучасне меню крафту для Pixel Village
//! Використовує єдину дизайн-систему

use macroquad::prelude::*;
use crate::design_system::*;
use crate::i18n::Texts;
use crate::game::Game;
use crate::data::{RECIPES, Recipe, Out};
use crate::ui::{Ui, Cmd};

/// Малює сучасне меню крафту
pub fn draw_modern_craft_menu(
    ui: &mut Ui,
    font: &Font,
    g: &Game,
    now: f64,
    body: Rect,
    texts: &Texts,
    fs: f32,
) {
    let rh = fs * 3.5; // Висота рядка рецепту

    // Отримуємо список рецептів (швидкі зверху)
    let ql = g.quick_list(now);
    let mut order: Vec<usize> = ql.clone();
    for i in 0..RECIPES.len() {
        if !order.contains(&i) {
            order.push(i);
        }
    }

    // Прокрутка
    let content_height = order.len() as f32 * rh;
    ui.scroll = ui.scroll.clamp(0.0, (content_height - body.h).max(0.0));

    // Малюємо рецепти
    let mut last_was_quick = false;
    for (row, &ri) in order.iter().enumerate() {
        let is_quick = ql.contains(&ri);

        // Заголовок "Всі рецепти" після швидких
        if last_was_quick && !is_quick {
            let header_y = body.y + row as f32 * rh - ui.scroll;
            if header_y >= body.y - rh && header_y < body.y + body.h {
                draw_section_header(font, body.x, header_y, body.w, "📋 Всі рецепти", fs);
            }
        }
        last_was_quick = is_quick;

        let y = body.y + row as f32 * rh - ui.scroll;

        // Пропускаємо невидимі рецепти
        if y < body.y - rh || y + rh > body.y + body.h {
            continue;
        }

        let r = &RECIPES[ri];
        let can = g.can_craft(r, now);
        let has_tool = matches!(r.out, Out::Tool(t) if g.tool(t));

        draw_recipe_card(
            ui,
            font,
            body.x,
            y,
            body.w,
            rh,
            ri,
            r,
            can,
            has_tool,
            is_quick,
            texts,
            fs,
        );
    }

    // Оверлей для обрізки контенту
    draw_scroll_mask(body);
}

/// Малює заголовок секції
fn draw_section_header(font: &Font, x: f32, y: f32, w: f32, title: &str, fs: f32) {
    let h = fs * 1.8;

    // Фон
    draw_rectangle(x, y, w, h, ColorPalette::PANEL_BG_LIGHT);

    // Декоративна лінія зліва
    draw_rectangle(x, y, 4.0, h, ColorPalette::BORDER_ACCENT);

    // Текст
    let font_size = Typography::scale_font_size(Typography::SIZE_SMALL, screen_height());
    draw_text_ex(
        title,
        x + Spacing::MEDIUM,
        y + h * 0.65,
        TextParams {
            font: Some(font),
            font_size,
            color: ColorPalette::TEXT_ACCENT,
            ..Default::default()
        }
    );
}

/// Малює картку рецепту
#[allow(clippy::too_many_arguments)]
fn draw_recipe_card(
    ui: &mut Ui,
    font: &Font,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    recipe_idx: usize,
    recipe: &Recipe,
    can_craft: bool,
    has_tool: bool,
    is_quick: bool,
    texts: &Texts,
    fs: f32,
) {
    let pad = Spacing::SMALL;
    let inner_h = h - 6.0;

    // Фон картки (різні кольори для різних станів)
    let bg_color = if has_tool {
        Color::new(0.25, 0.30, 0.25, 0.9) // Вже є
    } else if can_craft {
        ColorPalette::BUTTON_SUCCESS // Можна створити
    } else {
        ColorPalette::PANEL_BG_DARK // Не можна
    };

    // Тінь
    draw_rectangle(x + 3.0, y + 3.0, w, inner_h, ColorPalette::SHADOW_LIGHT);

    // Фон
    draw_rectangle(x, y, w, inner_h, bg_color);

    // Акцентна рамка зліва для швидких рецептів
    if is_quick {
        draw_rectangle(x, y, 5.0, inner_h, ColorPalette::ENERGY_FULL);
    }

    // Рамка
    let border_color = if can_craft && !has_tool {
        ColorPalette::BORDER_ACCENT
    } else {
        ColorPalette::BORDER_MAIN
    };
    draw_rectangle_lines(x, y, w, inner_h, Spacing::BORDER_MEDIUM, border_color);

    // Назва рецепту
    let name_size = Typography::scale_font_size(Typography::SIZE_BODY, screen_height());
    let name_color = if can_craft && !has_tool {
        ColorPalette::TEXT_PRIMARY
    } else {
        ColorPalette::TEXT_SECONDARY
    };

    draw_text_ex(
        texts.recipe_names[recipe_idx],
        x + pad + if is_quick { 10.0 } else { 0.0 },
        y + inner_h * 0.35,
        TextParams {
            font: Some(font),
            font_size: name_size,
            color: name_color,
            ..Default::default()
        }
    );

    // Вимоги та статус
    let mut status_parts = Vec::new();

    // Додаємо вартість
    let cost = cost_text(recipe, texts);
    if !cost.is_empty() {
        status_parts.push(cost);
    }

    // Додаємо умови
    if recipe.fire {
        status_parts.push(format!("🔥 {}", texts.at_campfire));
    }
    if recipe.furnace {
        status_parts.push(format!("⚒️ {}", texts.at_furnace));
    }

    // Статус наявності
    if has_tool {
        status_parts.push(format!("✓ {}", texts.already_have));
    }

    let status_text = status_parts.join(" · ");
    let status_size = Typography::scale_font_size(Typography::SIZE_SMALL, screen_height());
    let status_color = if has_tool {
        ColorPalette::TEXT_ACCENT
    } else if can_craft {
        Color::new(0.8, 0.9, 0.8, 1.0)
    } else {
        ColorPalette::TEXT_DISABLED
    };

    draw_text_ex(
        &status_text,
        x + pad + if is_quick { 10.0 } else { 0.0 },
        y + inner_h * 0.75,
        TextParams {
            font: Some(font),
            font_size: status_size,
            color: status_color,
            ..Default::default()
        }
    );

    // Кнопка "Створити"
    let button_w = fs * 6.5;
    let button_h = inner_h - 8.0;
    let button_x = x + w - button_w - 6.0;
    let button_y = y + 4.0;

    draw_craft_button(
        ui,
        font,
        button_x,
        button_y,
        button_w,
        button_h,
        texts.make,
        recipe_idx,
        can_craft && !has_tool,
    );
}

/// Малює кнопку крафту
fn draw_craft_button(
    ui: &mut Ui,
    font: &Font,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    text: &str,
    recipe_idx: usize,
    enabled: bool,
) {
    let is_hovered = is_mouse_over(x, y, w, h);
    let is_pressed = is_hovered && is_mouse_button_down(MouseButton::Left);

    // Колір кнопки
    let bg_color = if !enabled {
        ColorPalette::BUTTON_DISABLED
    } else if is_pressed {
        Color::new(0.25, 0.60, 0.25, 1.0)
    } else if is_hovered {
        Color::new(0.35, 0.70, 0.35, 1.0)
    } else {
        Color::new(0.33, 0.56, 0.33, 1.0)
    };

    let offset = if is_pressed && enabled { 2.0 } else { 0.0 };

    // Тінь (тільки якщо не натиснута)
    if !is_pressed && enabled {
        draw_rectangle(x + 2.0, y + 2.0, w, h, ColorPalette::SHADOW_MEDIUM);
    }

    // Фон кнопки
    draw_rectangle(x, y + offset, w, h, bg_color);

    // 3D ефект
    if enabled {
        draw_rectangle(x, y + offset, w, 2.0, ColorPalette::BORDER_LIGHT);
        draw_rectangle(x, y + offset + h - 2.0, w, 2.0, ColorPalette::BORDER_DARK);
    }

    // Рамка
    let border_color = if enabled {
        ColorPalette::BORDER_DARK
    } else {
        ColorPalette::BORDER_MAIN
    };
    draw_rectangle_lines(x, y + offset, w, h, Spacing::BORDER_MEDIUM, border_color);

    // Текст
    let font_size = Typography::scale_font_size(Typography::SIZE_SMALL, screen_height());
    let text_color = if enabled {
        ColorPalette::TEXT_PRIMARY
    } else {
        ColorPalette::TEXT_DISABLED
    };

    let text_measure = measure_text(text, Some(font), font_size, 1.0);
    draw_text_ex(
        text,
        x + (w - text_measure.width) / 2.0,
        y + offset + (h + text_measure.height) / 2.0,
        TextParams {
            font: Some(font),
            font_size,
            color: text_color,
            ..Default::default()
        }
    );

    // Реєструємо кнопку в UI системі (використовуємо публічний метод)
    if enabled {
        ui.btns_push_circle(Rect::new(x, y, w, h), Cmd::Craft(recipe_idx));
    }
}

/// Формує текст вартості рецепту
fn cost_text(r: &Recipe, texts: &Texts) -> String {
    let mut parts = Vec::new();
    for (item, count) in r.cost.iter() {
        if *count > 0 {
            let item_idx = *item as usize;
            parts.push(format!("{} ×{}", texts.item_names[item_idx], count));
        }
    }
    parts.join(", ")
}

/// Малює маску прокрутки (затемнення країв)
fn draw_scroll_mask(body: Rect) {
    let fade_h = 20.0;

    // Верхній градієнт
    for i in 0..10 {
        let alpha = (i as f32 / 10.0) * 0.5;
        let y = body.y + i as f32 * (fade_h / 10.0);
        draw_rectangle(
            body.x,
            y,
            body.w,
            fade_h / 10.0,
            Color::new(0.12, 0.14, 0.16, alpha)
        );
    }

    // Нижній градієнт
    for i in 0..10 {
        let alpha = ((10 - i) as f32 / 10.0) * 0.5;
        let y = body.y + body.h - fade_h + i as f32 * (fade_h / 10.0);
        draw_rectangle(
            body.x,
            y,
            body.w,
            fade_h / 10.0,
            Color::new(0.12, 0.14, 0.16, alpha)
        );
    }
}

/// Перевіряє чи курсор над областю
fn is_mouse_over(x: f32, y: f32, w: f32, h: f32) -> bool {
    let (mx, my) = mouse_position();
    mx >= x && mx <= x + w && my >= y && my <= y + h
}
