//! Єдина дизайн-система для Pixel Village
//!
//! Цей модуль містить всі константи для узгодженого дизайну:
//! - Кольорова палітра
//! - Розміри та відступи
//! - Стилі рамок та панелей
//! - Типографіка

use macroquad::prelude::*;

// ============================================================================
// КОЛЬОРОВА ПАЛІТРА - Природна піксельна гама
// ============================================================================

pub struct ColorPalette;

impl ColorPalette {
    // === ОСНОВНІ КОЛЬОРИ ===

    // Природні зелені (трава, рослини)
    pub const GRASS_DARK: Color = Color::new(0.22, 0.38, 0.22, 1.0);      // #385e38
    pub const GRASS_MAIN: Color = Color::new(0.33, 0.56, 0.33, 1.0);      // #558f55
    pub const GRASS_LIGHT: Color = Color::new(0.47, 0.70, 0.47, 1.0);     // #78b378

    // Земля та камінь
    pub const EARTH_DARK: Color = Color::new(0.29, 0.23, 0.18, 1.0);      // #4a3a2e
    pub const EARTH_MAIN: Color = Color::new(0.42, 0.34, 0.25, 1.0);      // #6b5640
    pub const EARTH_LIGHT: Color = Color::new(0.58, 0.49, 0.38, 1.0);     // #947d61

    // Вода
    pub const WATER_DARK: Color = Color::new(0.20, 0.35, 0.55, 1.0);      // #33598c
    pub const WATER_MAIN: Color = Color::new(0.30, 0.52, 0.75, 1.0);      // #4d85bf
    pub const WATER_LIGHT: Color = Color::new(0.45, 0.68, 0.88, 1.0);     // #73ade0

    // Дерево
    pub const WOOD_DARK: Color = Color::new(0.35, 0.25, 0.15, 1.0);       // #594026
    pub const WOOD_MAIN: Color = Color::new(0.52, 0.38, 0.24, 1.0);       // #85613d
    pub const WOOD_LIGHT: Color = Color::new(0.68, 0.53, 0.35, 1.0);      // #ae8759

    // Камінь
    pub const STONE_DARK: Color = Color::new(0.30, 0.30, 0.33, 1.0);      // #4d4d54
    pub const STONE_MAIN: Color = Color::new(0.45, 0.45, 0.48, 1.0);      // #73737a
    pub const STONE_LIGHT: Color = Color::new(0.60, 0.60, 0.63, 1.0);     // #9999a1

    // === UI КОЛЬОРИ ===

    // Фони панелей
    pub const PANEL_BG: Color = Color::new(0.12, 0.14, 0.16, 0.95);       // #1f2329
    pub const PANEL_BG_LIGHT: Color = Color::new(0.16, 0.18, 0.21, 0.95); // #292e35
    pub const PANEL_BG_DARK: Color = Color::new(0.08, 0.10, 0.12, 0.95);  // #14191f

    // Рамки
    pub const BORDER_MAIN: Color = Color::new(0.42, 0.36, 0.26, 1.0);     // #6b5c43
    pub const BORDER_LIGHT: Color = Color::new(0.58, 0.51, 0.39, 1.0);    // #948263
    pub const BORDER_DARK: Color = Color::new(0.28, 0.24, 0.18, 1.0);     // #473d2e
    pub const BORDER_ACCENT: Color = Color::new(0.78, 0.65, 0.38, 1.0);   // #c7a561

    // Кнопки
    pub const BUTTON_DEFAULT: Color = Color::new(0.42, 0.34, 0.25, 1.0);  // #6b5640
    pub const BUTTON_HOVER: Color = Color::new(0.52, 0.43, 0.32, 1.0);    // #856e52
    pub const BUTTON_ACTIVE: Color = Color::new(0.62, 0.52, 0.40, 1.0);   // #9e8566
    pub const BUTTON_DISABLED: Color = Color::new(0.25, 0.25, 0.28, 0.6); // #404047

    // Спеціальні кнопки
    pub const BUTTON_SUCCESS: Color = Color::new(0.33, 0.56, 0.33, 1.0);  // #558f55
    pub const BUTTON_DANGER: Color = Color::new(0.70, 0.28, 0.28, 1.0);   // #b34747
    pub const BUTTON_INFO: Color = Color::new(0.30, 0.52, 0.75, 1.0);     // #4d85bf
    pub const BUTTON_WARNING: Color = Color::new(0.85, 0.65, 0.30, 1.0);  // #d9a64d

    // Текст
    pub const TEXT_PRIMARY: Color = Color::new(0.95, 0.93, 0.88, 1.0);    // #f2ede0
    pub const TEXT_SECONDARY: Color = Color::new(0.75, 0.73, 0.68, 1.0);  // #bfbaad
    pub const TEXT_DISABLED: Color = Color::new(0.45, 0.45, 0.48, 1.0);   // #73737a
    pub const TEXT_ACCENT: Color = Color::new(0.95, 0.82, 0.48, 1.0);     // #f2d17a

    // Статус-бари
    pub const HEALTH_FULL: Color = Color::new(0.85, 0.33, 0.33, 1.0);     // #d95454
    pub const HEALTH_LOW: Color = Color::new(0.60, 0.20, 0.20, 1.0);      // #993333
    pub const HUNGER_FULL: Color = Color::new(0.85, 0.55, 0.30, 1.0);     // #d98c4d
    pub const THIRST_FULL: Color = Color::new(0.40, 0.68, 0.88, 1.0);     // #66ade0
    pub const ENERGY_FULL: Color = Color::new(0.95, 0.82, 0.38, 1.0);     // #f2d161

    // Рідкість предметів
    pub const RARITY_COMMON: Color = Color::new(0.75, 0.73, 0.68, 1.0);   // #bfbaad
    pub const RARITY_UNCOMMON: Color = Color::new(0.45, 0.75, 0.45, 1.0); // #73bf73
    pub const RARITY_RARE: Color = Color::new(0.45, 0.65, 0.85, 1.0);     // #73a6d9
    pub const RARITY_EPIC: Color = Color::new(0.70, 0.45, 0.85, 1.0);     // #b373d9
    pub const RARITY_LEGENDARY: Color = Color::new(0.95, 0.72, 0.30, 1.0);// #f2b84d

    // Тіні та оверлеї
    pub const SHADOW_LIGHT: Color = Color::new(0.0, 0.0, 0.0, 0.2);
    pub const SHADOW_MEDIUM: Color = Color::new(0.0, 0.0, 0.0, 0.4);
    pub const SHADOW_DARK: Color = Color::new(0.0, 0.0, 0.0, 0.6);
    pub const OVERLAY_LIGHT: Color = Color::new(0.0, 0.0, 0.0, 0.3);
    pub const OVERLAY_MEDIUM: Color = Color::new(0.0, 0.0, 0.0, 0.5);
    pub const OVERLAY_DARK: Color = Color::new(0.0, 0.0, 0.0, 0.7);
}

// ============================================================================
// РОЗМІРИ ТА ВІДСТУПИ
// ============================================================================

pub struct Spacing;

impl Spacing {
    // Базові відступи (в пікселях при 1080p)
    pub const TINY: f32 = 4.0;
    pub const SMALL: f32 = 8.0;
    pub const MEDIUM: f32 = 16.0;
    pub const LARGE: f32 = 24.0;
    pub const XLARGE: f32 = 32.0;
    pub const XXLARGE: f32 = 48.0;

    // Відступи для UI
    pub const PANEL_PADDING: f32 = 20.0;
    pub const BUTTON_PADDING_X: f32 = 16.0;
    pub const BUTTON_PADDING_Y: f32 = 8.0;
    pub const ITEM_SPACING: f32 = 12.0;
    pub const SECTION_SPACING: f32 = 24.0;

    // Розміри елементів
    pub const ICON_SMALL: f32 = 24.0;
    pub const ICON_MEDIUM: f32 = 32.0;
    pub const ICON_LARGE: f32 = 48.0;
    pub const ICON_XLARGE: f32 = 64.0;

    pub const BUTTON_HEIGHT_SMALL: f32 = 32.0;
    pub const BUTTON_HEIGHT_MEDIUM: f32 = 40.0;
    pub const BUTTON_HEIGHT_LARGE: f32 = 56.0;

    pub const SLOT_SIZE: f32 = 48.0;
    pub const SLOT_SPACING: f32 = 4.0;

    // Рамки
    pub const BORDER_THIN: f32 = 2.0;
    pub const BORDER_MEDIUM: f32 = 3.0;
    pub const BORDER_THICK: f32 = 4.0;

    // Заокруглення (для піксельного стилю - мінімальні)
    pub const CORNER_RADIUS: f32 = 0.0; // Піксельний стиль без заокруглень
}

// ============================================================================
// ТИПОГРАФІКА
// ============================================================================

pub struct Typography;

impl Typography {
    // Базові розміри шрифтів (масштабуються від роздільності)
    pub fn scale_font_size(base_size: f32, screen_height: f32) -> u16 {
        let scale = (screen_height / 1080.0).max(0.5).min(2.0);
        (base_size * scale).round() as u16
    }

    // Розміри
    pub const SIZE_TINY: f32 = 12.0;
    pub const SIZE_SMALL: f32 = 14.0;
    pub const SIZE_BODY: f32 = 16.0;
    pub const SIZE_LARGE: f32 = 20.0;
    pub const SIZE_TITLE: f32 = 24.0;
    pub const SIZE_HEADING: f32 = 32.0;
    pub const SIZE_DISPLAY: f32 = 48.0;
}

// ============================================================================
// СТИЛІ КОМПОНЕНТІВ
// ============================================================================

#[derive(Clone, Copy)]
pub struct PanelStyle {
    pub bg_color: Color,
    pub border_color: Color,
    pub border_width: f32,
    pub padding: f32,
    pub shadow: bool,
}

impl PanelStyle {
    pub const DEFAULT: PanelStyle = PanelStyle {
        bg_color: ColorPalette::PANEL_BG,
        border_color: ColorPalette::BORDER_MAIN,
        border_width: Spacing::BORDER_MEDIUM,
        padding: Spacing::PANEL_PADDING,
        shadow: true,
    };

    pub const DARK: PanelStyle = PanelStyle {
        bg_color: ColorPalette::PANEL_BG_DARK,
        border_color: ColorPalette::BORDER_DARK,
        border_width: Spacing::BORDER_MEDIUM,
        padding: Spacing::PANEL_PADDING,
        shadow: true,
    };

    pub const LIGHT: PanelStyle = PanelStyle {
        bg_color: ColorPalette::PANEL_BG_LIGHT,
        border_color: ColorPalette::BORDER_LIGHT,
        border_width: Spacing::BORDER_MEDIUM,
        padding: Spacing::PANEL_PADDING,
        shadow: false,
    };
}

#[derive(Clone, Copy)]
pub struct ButtonStyle {
    pub bg_color: Color,
    pub hover_color: Color,
    pub active_color: Color,
    pub text_color: Color,
    pub border_color: Color,
    pub border_width: f32,
    pub padding_x: f32,
    pub padding_y: f32,
}

impl ButtonStyle {
    pub const DEFAULT: ButtonStyle = ButtonStyle {
        bg_color: ColorPalette::BUTTON_DEFAULT,
        hover_color: ColorPalette::BUTTON_HOVER,
        active_color: ColorPalette::BUTTON_ACTIVE,
        text_color: ColorPalette::TEXT_PRIMARY,
        border_color: ColorPalette::BORDER_DARK,
        border_width: Spacing::BORDER_MEDIUM,
        padding_x: Spacing::BUTTON_PADDING_X,
        padding_y: Spacing::BUTTON_PADDING_Y,
    };

    pub const SUCCESS: ButtonStyle = ButtonStyle {
        bg_color: ColorPalette::BUTTON_SUCCESS,
        hover_color: Color::new(0.40, 0.65, 0.40, 1.0),
        active_color: Color::new(0.48, 0.73, 0.48, 1.0),
        text_color: ColorPalette::TEXT_PRIMARY,
        border_color: ColorPalette::GRASS_DARK,
        border_width: Spacing::BORDER_MEDIUM,
        padding_x: Spacing::BUTTON_PADDING_X,
        padding_y: Spacing::BUTTON_PADDING_Y,
    };

    pub const DANGER: ButtonStyle = ButtonStyle {
        bg_color: ColorPalette::BUTTON_DANGER,
        hover_color: Color::new(0.78, 0.35, 0.35, 1.0),
        active_color: Color::new(0.85, 0.42, 0.42, 1.0),
        text_color: ColorPalette::TEXT_PRIMARY,
        border_color: Color::new(0.50, 0.20, 0.20, 1.0),
        border_width: Spacing::BORDER_MEDIUM,
        padding_x: Spacing::BUTTON_PADDING_X,
        padding_y: Spacing::BUTTON_PADDING_Y,
    };
}

// ============================================================================
// ДОПОМІЖНІ ФУНКЦІЇ
// ============================================================================

/// Малює панель з тінню
pub fn draw_panel(x: f32, y: f32, w: f32, h: f32, style: PanelStyle) {
    // Тінь
    if style.shadow {
        draw_rectangle(
            x + 4.0,
            y + 4.0,
            w,
            h,
            ColorPalette::SHADOW_MEDIUM
        );
    }

    // Фон
    draw_rectangle(x, y, w, h, style.bg_color);

    // Рамка
    draw_rectangle_lines(x, y, w, h, style.border_width, style.border_color);
}

/// Малює кнопку з правильним станом
pub fn draw_button(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    text: &str,
    font: &Font,
    font_size: u16,
    style: ButtonStyle,
    is_hovered: bool,
    is_pressed: bool,
) {
    // Вибір кольору залежно від стану
    let bg_color = if is_pressed {
        style.active_color
    } else if is_hovered {
        style.hover_color
    } else {
        style.bg_color
    };

    // Ефект натискання
    let offset = if is_pressed { 2.0 } else { 0.0 };

    // Фон кнопки
    draw_rectangle(x, y + offset, w, h, bg_color);

    // Рамка
    draw_rectangle_lines(x, y + offset, w, h, style.border_width, style.border_color);

    // Текст (центрований)
    let text_params = TextParams {
        font: Some(font),
        font_size,
        color: style.text_color,
        ..Default::default()
    };
    let text_measure = measure_text(text, Some(font), font_size, 1.0);
    let text_x = x + (w - text_measure.width) / 2.0;
    let text_y = y + offset + (h + text_measure.height) / 2.0 - text_measure.offset_y / 2.0;

    draw_text_ex(text, text_x, text_y, text_params);
}

/// Перевіряє чи курсор над прямокутником
pub fn is_hovered(x: f32, y: f32, w: f32, h: f32) -> bool {
    let (mx, my) = mouse_position();
    mx >= x && mx <= x + w && my >= y && my <= y + h
}

/// Малює індикатор прогресу (здоров'я, голод і т.д.)
pub fn draw_progress_bar(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    value: f32,
    max_value: f32,
    fg_color: Color,
    bg_color: Color,
    border_color: Color,
) {
    let progress = (value / max_value).clamp(0.0, 1.0);

    // Фон
    draw_rectangle(x, y, w, h, bg_color);

    // Заповнення
    draw_rectangle(x, y, w * progress, h, fg_color);

    // Рамка
    draw_rectangle_lines(x, y, w, h, Spacing::BORDER_THIN, border_color);
}
