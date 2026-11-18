//! Сучасне меню торгівлі/ринку для Pixel Village
//! Використовує єдину дизайн-систему

use macroquad::prelude::*;
use crate::design_system::*;
use crate::i18n::Texts;
use crate::game::Game;
use shared::Offer;
use crate::ui::{Ui, Cmd};
use crate::data::NI;
use std::collections::HashMap;

/// Малює сучасне меню торгівлі
#[allow(clippy::too_many_arguments)]
pub fn draw_modern_market_menu(
    ui: &mut Ui,
    font: &Font,
    g: &Game,
    body: Rect,
    texts: &Texts,
    fs: f32,
    market: &[Offer],
    posted: &HashMap<u32, (u8, u32)>,
    my_id: Option<u32>,
) {
    let pad = Spacing::MEDIUM;

    // Верхня секція - форма створення оголошення
    let form_h = fs * 12.0;
    let form_rect = Rect::new(body.x, body.y, body.w, form_h);
    draw_sell_form(ui, font, g, form_rect, texts, fs);

    // Нижня секція - список оголошень
    let list_y = body.y + form_h + pad;
    let list_h = body.h - form_h - pad;
    let list_rect = Rect::new(body.x, list_y, body.w, list_h);
    draw_offers_list(ui, font, g, list_rect, texts, fs, market, posted, my_id);
}

/// Малює форму створення оголошення
fn draw_sell_form(
    ui: &mut Ui,
    font: &Font,
    g: &Game,
    form: Rect,
    texts: &Texts,
    fs: f32,
) {
    let pad = Spacing::SMALL;

    // Фон форми
    draw_rectangle(form.x, form.y, form.w, form.h, ColorPalette::PANEL_BG_DARK);

    // Заголовок
    draw_rectangle(form.x, form.y, form.w, fs * 2.0, ColorPalette::PANEL_BG_LIGHT);
    draw_rectangle(form.x, form.y + fs * 2.0, form.w, 2.0, ColorPalette::BORDER_ACCENT);

    let title_size = Typography::scale_font_size(Typography::SIZE_BODY, screen_height());
    draw_text_ex(
        &format!("💰 {}", texts.sell_item),
        form.x + pad,
        form.y + fs * 1.3,
        TextParams {
            font: Some(font),
            font_size: title_size,
            color: ColorPalette::TEXT_ACCENT,
            ..Default::default()
        }
    );

    // Рамка форми
    draw_rectangle_lines(form.x, form.y, form.w, form.h, Spacing::BORDER_MEDIUM, ColorPalette::BORDER_MAIN);

    // Контент форми
    let content_y = form.y + fs * 2.5;
    let bh = fs * 2.3;
    let sq = fs * 2.3;

    // Ряд 1: Вибір предмета
    let y1 = content_y;
    let label_size = Typography::scale_font_size(Typography::SIZE_SMALL, screen_height());

    draw_text_ex(
        &format!("{}:", texts.sell_item),
        form.x + pad,
        y1 + bh * 0.6,
        TextParams {
            font: Some(font),
            font_size: label_size,
            color: ColorPalette::TEXT_PRIMARY,
            ..Default::default()
        }
    );

    // Кнопки вибору предмета
    let item_x = form.x + fs * 5.0;
    draw_selector_button(ui, font, item_x, y1, sq, bh, "<", Cmd::MktItem(-1), fs);

    let item_name = texts.item_names[ui.mk_item];
    let name_w = fs * 9.0;
    draw_text_ex(
        item_name,
        item_x + sq + (name_w - measure_text(item_name, Some(font), title_size, 1.0).width) / 2.0,
        y1 + bh * 0.65,
        TextParams {
            font: Some(font),
            font_size: title_size,
            color: ColorPalette::TEXT_PRIMARY,
            ..Default::default()
        }
    );

    draw_selector_button(ui, font, item_x + sq + name_w, y1, sq, bh, ">", Cmd::MktItem(1), fs);

    // Ряд 2: Кількість
    let y2 = y1 + bh + pad;
    draw_text_ex(
        texts.quantity,
        form.x + pad,
        y2 + bh * 0.6,
        TextParams {
            font: Some(font),
            font_size: label_size,
            color: ColorPalette::TEXT_PRIMARY,
            ..Default::default()
        }
    );

    let qty_x = form.x + fs * 5.0;
    draw_selector_button(ui, font, qty_x, y2, sq, bh, "-", Cmd::MktQty(-1), fs);

    let qty_text = format!("{}", ui.mk_qty);
    draw_text_ex(
        &qty_text,
        qty_x + sq + fs * 1.5,
        y2 + bh * 0.65,
        TextParams {
            font: Some(font),
            font_size: title_size,
            color: ColorPalette::TEXT_ACCENT,
            ..Default::default()
        }
    );

    draw_selector_button(ui, font, qty_x + sq + fs * 3.0, y2, sq, bh, "+", Cmd::MktQty(1), fs);

    // Ряд 3: Ціна
    let y3 = y2 + bh + pad;
    draw_text_ex(
        texts.price,
        form.x + pad,
        y3 + bh * 0.6,
        TextParams {
            font: Some(font),
            font_size: label_size,
            color: ColorPalette::TEXT_PRIMARY,
            ..Default::default()
        }
    );

    let price_x = form.x + fs * 5.0;
    draw_selector_button(ui, font, price_x, y3, sq, bh, "-", Cmd::MktPrice(-1), fs);

    let price_text = format!("{} 💰", ui.mk_price);
    draw_text_ex(
        &price_text,
        price_x + sq + fs * 1.5,
        y3 + bh * 0.65,
        TextParams {
            font: Some(font),
            font_size: title_size,
            color: ColorPalette::TEXT_ACCENT,
            ..Default::default()
        }
    );

    draw_selector_button(ui, font, price_x + sq + fs * 3.0, y3, sq, bh, "+", Cmd::MktPrice(1), fs);

    // Кнопка "Виставити"
    let can_post = g.inv[ui.mk_item] >= ui.mk_qty;
    let post_x = form.x + fs * 18.0;
    let post_w = fs * 10.0;
    let post_h = bh * 2.0 + pad;

    draw_action_button(
        ui,
        font,
        post_x,
        y2,
        post_w,
        post_h,
        texts.post_offer,
        Cmd::MktPost,
        can_post,
        ButtonType::Success,
    );

    // Інфо про золото та наявність
    let info_text = format!("{}: {}   {}: {}",
        texts.gold_label, g.gold,
        texts.you_have, g.inv[ui.mk_item]
    );
    draw_text_ex(
        &info_text,
        form.x + fs * 30.0,
        y2 + bh * 0.65,
        TextParams {
            font: Some(font),
            font_size: label_size,
            color: ColorPalette::TEXT_SECONDARY,
            ..Default::default()
        }
    );
}

/// Малює список оголошень
#[allow(clippy::too_many_arguments)]
fn draw_offers_list(
    ui: &mut Ui,
    font: &Font,
    g: &Game,
    list: Rect,
    texts: &Texts,
    fs: f32,
    market: &[Offer],
    posted: &HashMap<u32, (u8, u32)>,
    my_id: Option<u32>,
) {
    let pad = Spacing::SMALL;

    // Заголовок списку
    let header_h = fs * 2.0;
    draw_rectangle(list.x, list.y, list.w, header_h, ColorPalette::PANEL_BG_LIGHT);
    draw_rectangle(list.x, list.y + header_h, list.w, 2.0, ColorPalette::BORDER_ACCENT);

    let title_size = Typography::scale_font_size(Typography::SIZE_BODY, screen_height());
    draw_text_ex(
        "🛒 Ринок",
        list.x + pad,
        list.y + header_h * 0.65,
        TextParams {
            font: Some(font),
            font_size: title_size,
            color: ColorPalette::TEXT_ACCENT,
            ..Default::default()
        }
    );

    // Рамка списку
    draw_rectangle_lines(list.x, list.y, list.w, list.h, Spacing::BORDER_MEDIUM, ColorPalette::BORDER_MAIN);

    // Область контенту
    let content_y = list.y + header_h + 4.0;
    let content_h = list.h - header_h - 8.0;
    let content_rect = Rect::new(list.x + 4.0, content_y, list.w - 8.0, content_h);

    // Якщо ринок порожній
    if market.is_empty() {
        let empty_size = Typography::scale_font_size(Typography::SIZE_SMALL, screen_height());
        draw_text_ex(
            "Поки порожньо (ринок працює онлайн)",
            content_rect.x + pad,
            content_rect.y + fs * 2.0,
            TextParams {
                font: Some(font),
                font_size: empty_size,
                color: ColorPalette::TEXT_DISABLED,
                ..Default::default()
            }
        );
        return;
    }

    // Прокрутка
    let rh = fs * 3.5;
    let content_height = market.len() as f32 * rh;
    ui.scroll = ui.scroll.clamp(0.0, (content_height - content_h).max(0.0));

    // Малюємо оголошення
    for (row, offer) in market.iter().enumerate() {
        let y = content_rect.y + row as f32 * rh - ui.scroll;

        // Пропускаємо невидимі
        if y < content_rect.y - rh || y + rh > content_rect.y + content_h {
            continue;
        }

        let is_mine = Some(offer.seller) == my_id || posted.contains_key(&offer.id);

        draw_offer_card(
            ui,
            font,
            content_rect.x,
            y,
            content_rect.w,
            rh,
            offer,
            is_mine,
            g.gold >= offer.price,
            texts,
            fs,
        );
    }

    // Маска прокрутки
    draw_scroll_gradient(content_rect);
}

/// Малює картку оголошення
#[allow(clippy::too_many_arguments)]
fn draw_offer_card(
    ui: &mut Ui,
    font: &Font,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    offer: &Offer,
    is_mine: bool,
    can_afford: bool,
    texts: &Texts,
    fs: f32,
) {
    let pad = Spacing::SMALL;
    let inner_h = h - 6.0;

    // Фон картки
    let bg_color = if is_mine {
        Color::new(0.35, 0.28, 0.22, 0.9) // Моє оголошення (коричневий)
    } else {
        ColorPalette::PANEL_BG_DARK
    };

    // Тінь
    draw_rectangle(x + 2.0, y + 2.0, w, inner_h, ColorPalette::SHADOW_LIGHT);

    // Фон
    draw_rectangle(x, y, w, inner_h, bg_color);

    // Акцентна рамка для моїх оголошень
    if is_mine {
        draw_rectangle(x, y, 4.0, inner_h, ColorPalette::BUTTON_WARNING);
    }

    // Рамка
    let border_color = if is_mine {
        ColorPalette::BUTTON_WARNING
    } else {
        ColorPalette::BORDER_MAIN
    };
    draw_rectangle_lines(x, y, w, inner_h, Spacing::BORDER_THIN, border_color);

    // Текст оголошення
    let item_name = texts.item_names[(offer.item as usize).min(NI - 1)];
    let offer_text = format!(
        "{}: {} ×{} {} {} 💰",
        offer.seller_name, item_name, offer.qty, texts.for_gold, offer.price
    );

    let text_size = Typography::scale_font_size(Typography::SIZE_SMALL, screen_height());
    draw_text_ex(
        &offer_text,
        x + pad + if is_mine { 8.0 } else { 0.0 },
        y + inner_h * 0.6,
        TextParams {
            font: Some(font),
            font_size: text_size,
            color: ColorPalette::TEXT_PRIMARY,
            ..Default::default()
        }
    );

    // Кнопка дії
    let button_w = fs * 7.0;
    let button_h = inner_h - 8.0;
    let button_x = x + w - button_w - 4.0;
    let button_y = y + 4.0;

    if is_mine {
        draw_action_button(
            ui,
            font,
            button_x,
            button_y,
            button_w,
            button_h,
            texts.withdraw,
            Cmd::MktWithdraw(offer.id),
            true,
            ButtonType::Danger,
        );
    } else {
        draw_action_button(
            ui,
            font,
            button_x,
            button_y,
            button_w,
            button_h,
            texts.buy,
            Cmd::MktBuy(offer.id),
            can_afford,
            ButtonType::Success,
        );
    }
}

/// Типи кнопок
enum ButtonType {
    Success,
    Danger,
}

/// Малює кнопку дії (купити/зняти)
fn draw_action_button(
    ui: &mut Ui,
    font: &Font,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    text: &str,
    cmd: Cmd,
    enabled: bool,
    button_type: ButtonType,
) {
    let is_hovered = is_mouse_over(x, y, w, h);
    let is_pressed = is_hovered && is_mouse_button_down(MouseButton::Left);

    // Колір кнопки
    let (base_color, hover_color, active_color) = match button_type {
        ButtonType::Success => (
            ColorPalette::BUTTON_SUCCESS,
            Color::new(0.40, 0.65, 0.40, 1.0),
            Color::new(0.25, 0.50, 0.25, 1.0),
        ),
        ButtonType::Danger => (
            ColorPalette::BUTTON_DANGER,
            Color::new(0.78, 0.35, 0.35, 1.0),
            Color::new(0.60, 0.20, 0.20, 1.0),
        ),
    };

    let bg_color = if !enabled {
        ColorPalette::BUTTON_DISABLED
    } else if is_pressed {
        active_color
    } else if is_hovered {
        hover_color
    } else {
        base_color
    };

    let offset = if is_pressed && enabled { 2.0 } else { 0.0 };

    // Тінь
    if !is_pressed && enabled {
        draw_rectangle(x + 2.0, y + 2.0, w, h, ColorPalette::SHADOW_MEDIUM);
    }

    // Фон
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

    // Реєструємо кнопку
    if enabled {
        ui.btns_push_circle(Rect::new(x, y, w, h), cmd);
    }
}

/// Малює кнопку-селектор (+/-)
fn draw_selector_button(
    ui: &mut Ui,
    font: &Font,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    text: &str,
    cmd: Cmd,
    fs: f32,
) {
    let is_hovered = is_mouse_over(x, y, w, h);
    let is_pressed = is_hovered && is_mouse_button_down(MouseButton::Left);

    let bg_color = if is_pressed {
        ColorPalette::EARTH_DARK
    } else if is_hovered {
        ColorPalette::EARTH_MAIN
    } else {
        Color::new(0.35, 0.30, 0.25, 1.0)
    };

    let offset = if is_pressed { 1.0 } else { 0.0 };

    // Тінь
    if !is_pressed {
        draw_rectangle(x + 1.0, y + 1.0, w, h, ColorPalette::SHADOW_LIGHT);
    }

    // Фон
    draw_rectangle(x, y + offset, w, h, bg_color);

    // Рамка
    draw_rectangle_lines(x, y + offset, w, h, Spacing::BORDER_THIN, ColorPalette::BORDER_DARK);

    // Текст
    let font_size = Typography::scale_font_size(Typography::SIZE_BODY, screen_height());
    let text_measure = measure_text(text, Some(font), font_size, 1.0);
    draw_text_ex(
        text,
        x + (w - text_measure.width) / 2.0,
        y + offset + (h + text_measure.height) / 2.0,
        TextParams {
            font: Some(font),
            font_size,
            color: ColorPalette::TEXT_PRIMARY,
            ..Default::default()
        }
    );

    // Реєструємо кнопку
    ui.btns_push_circle(Rect::new(x, y, w, h), cmd);
}

/// Малює градієнт прокрутки
fn draw_scroll_gradient(rect: Rect) {
    let fade_h = 15.0;

    // Верхній градієнт
    for i in 0..8 {
        let alpha = (i as f32 / 8.0) * 0.4;
        let y = rect.y + i as f32 * (fade_h / 8.0);
        draw_rectangle(
            rect.x,
            y,
            rect.w,
            fade_h / 8.0,
            Color::new(0.12, 0.14, 0.16, alpha)
        );
    }

    // Нижній градієнт
    for i in 0..8 {
        let alpha = ((8 - i) as f32 / 8.0) * 0.4;
        let y = rect.y + rect.h - fade_h + i as f32 * (fade_h / 8.0);
        draw_rectangle(
            rect.x,
            y,
            rect.w,
            fade_h / 8.0,
            Color::new(0.12, 0.14, 0.16, alpha)
        );
    }
}

/// Перевіряє чи курсор над областю
fn is_mouse_over(x: f32, y: f32, w: f32, h: f32) -> bool {
    let (mx, my) = mouse_position();
    mx >= x && mx <= x + w && my >= y && my <= y + h
}
