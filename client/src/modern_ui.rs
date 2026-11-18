//! Нова система UI компонентів з єдиним дизайном
//! Замінює старий ui.rs на сучасний компонентний підхід

use macroquad::prelude::*;
use crate::design_system::*;

/// Покращений UI менеджер з дизайн-системою
pub struct ModernUI {
    pub hovered_button: Option<String>,
    pub pressed_button: Option<String>,
}

impl ModernUI {
    pub fn new() -> Self {
        ModernUI {
            hovered_button: None,
            pressed_button: None,
        }
    }

    /// Малює сучасну панель з тінню та правильними відступами
    pub fn draw_panel(&self, x: f32, y: f32, w: f32, h: f32, title: Option<&str>, font: &Font) -> Rect {
        let style = PanelStyle::DEFAULT;

        // Тінь
        if style.shadow {
            draw_rectangle(x + 4.0, y + 4.0, w, h, ColorPalette::SHADOW_MEDIUM);
        }

        // Фон панелі
        draw_rectangle(x, y, w, h, style.bg_color);

        // Заголовок якщо є
        if let Some(title_text) = title {
            let title_h = 48.0;
            let title_bg = ColorPalette::PANEL_BG_DARK;

            // Фон заголовка
            draw_rectangle(x, y, w, title_h, title_bg);

            // Лінія під заголовком
            draw_rectangle(x, y + title_h, w, 2.0, ColorPalette::BORDER_ACCENT);

            // Текст заголовка
            let font_size = Typography::scale_font_size(Typography::SIZE_HEADING, screen_height());
            let text_params = TextParams {
                font: Some(font),
                font_size,
                color: ColorPalette::TEXT_PRIMARY,
                ..Default::default()
            };

            draw_text_ex(
                title_text,
                x + Spacing::PANEL_PADDING,
                y + title_h / 2.0 + font_size as f32 / 3.0,
                text_params
            );
        }

        // Зовнішня рамка
        draw_rectangle_lines(x, y, w, h, style.border_width, style.border_color);

        // Внутрішня світла рамка для об'єму
        draw_rectangle_lines(
            x + style.border_width,
            y + style.border_width,
            w - style.border_width * 2.0,
            h - style.border_width * 2.0,
            1.0,
            ColorPalette::BORDER_LIGHT
        );

        // Повертаємо область контенту (з відступами)
        let content_y = if title.is_some() { y + 50.0 } else { y + style.padding };
        Rect::new(
            x + style.padding,
            content_y,
            w - style.padding * 2.0,
            h - style.padding - (content_y - y)
        )
    }

    /// Малює сучасну кнопку з hover ефектами
    pub fn draw_button(
        &mut self,
        id: &str,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        text: &str,
        font: &Font,
        style: ButtonStyle,
        enabled: bool
    ) -> bool {
        if !enabled {
            // Disabled стан
            draw_rectangle(x, y, w, h, ColorPalette::BUTTON_DISABLED);
            draw_rectangle_lines(x, y, w, h, Spacing::BORDER_THIN, ColorPalette::BORDER_DARK);

            let font_size = Typography::scale_font_size(Typography::SIZE_BODY, screen_height());
            let text_params = TextParams {
                font: Some(font),
                font_size,
                color: ColorPalette::TEXT_DISABLED,
                ..Default::default()
            };
            let text_measure = measure_text(text, Some(font), font_size, 1.0);
            draw_text_ex(
                text,
                x + (w - text_measure.width) / 2.0,
                y + (h + text_measure.height) / 2.0,
                text_params
            );
            return false;
        }

        let is_hovered = is_hovered(x, y, w, h);
        let is_pressed = is_hovered && is_mouse_button_down(MouseButton::Left);
        let clicked = is_hovered && is_mouse_button_released(MouseButton::Left);

        // Оновлюємо стан
        if is_hovered {
            self.hovered_button = Some(id.to_string());
        }
        if is_pressed {
            self.pressed_button = Some(id.to_string());
        }

        // Вибір кольору
        let bg_color = if is_pressed {
            style.active_color
        } else if is_hovered {
            style.hover_color
        } else {
            style.bg_color
        };

        // Ефект натискання (зміщення)
        let offset = if is_pressed { 2.0 } else { 0.0 };

        // Тінь кнопки
        if !is_pressed {
            draw_rectangle(x + 3.0, y + 3.0, w, h, ColorPalette::SHADOW_LIGHT);
        }

        // Фон кнопки
        draw_rectangle(x, y + offset, w, h, bg_color);

        // Світлий край зверху (3D ефект)
        draw_rectangle(x, y + offset, w, 2.0, ColorPalette::BORDER_LIGHT);

        // Темний край знизу (3D ефект)
        draw_rectangle(x, y + offset + h - 2.0, w, 2.0, ColorPalette::BORDER_DARK);

        // Зовнішня рамка
        draw_rectangle_lines(x, y + offset, w, h, style.border_width, style.border_color);

        // Текст
        let font_size = Typography::scale_font_size(Typography::SIZE_BODY, screen_height());
        let text_params = TextParams {
            font: Some(font),
            font_size,
            color: style.text_color,
            ..Default::default()
        };
        let text_measure = measure_text(text, Some(font), font_size, 1.0);
        draw_text_ex(
            text,
            x + (w - text_measure.width) / 2.0,
            y + offset + (h + text_measure.height) / 2.0,
            text_params
        );

        clicked
    }

    /// Малює слот для предмета з рамкою
    pub fn draw_item_slot(
        &self,
        x: f32,
        y: f32,
        item_icon: Option<&str>,
        count: u32,
        rarity: ItemRarity,
        is_selected: bool,
        font: &Font
    ) -> bool {
        let size = Spacing::SLOT_SIZE;
        let is_hovered = is_hovered(x, y, size, size);

        // Фон слоту
        let bg_color = if is_selected {
            ColorPalette::PANEL_BG_LIGHT
        } else if is_hovered {
            ColorPalette::EARTH_DARK
        } else {
            ColorPalette::PANEL_BG_DARK
        };

        draw_rectangle(x, y, size, size, bg_color);

        // Рамка рідкості
        let rarity_color = match rarity {
            ItemRarity::Common => ColorPalette::RARITY_COMMON,
            ItemRarity::Uncommon => ColorPalette::RARITY_UNCOMMON,
            ItemRarity::Rare => ColorPalette::RARITY_RARE,
            ItemRarity::Epic => ColorPalette::RARITY_EPIC,
            ItemRarity::Legendary => ColorPalette::RARITY_LEGENDARY,
        };

        draw_rectangle_lines(x, y, size, size, Spacing::BORDER_MEDIUM, rarity_color);

        // Внутрішня рамка
        draw_rectangle_lines(
            x + Spacing::BORDER_MEDIUM,
            y + Spacing::BORDER_MEDIUM,
            size - Spacing::BORDER_MEDIUM * 2.0,
            size - Spacing::BORDER_MEDIUM * 2.0,
            1.0,
            ColorPalette::BORDER_DARK
        );

        // Іконка предмета (поки текст)
        if let Some(icon_text) = item_icon {
            let font_size = Typography::scale_font_size(Typography::SIZE_LARGE, screen_height());
            let text_params = TextParams {
                font: Some(font),
                font_size,
                color: ColorPalette::TEXT_PRIMARY,
                ..Default::default()
            };
            let text_measure = measure_text(icon_text, Some(font), font_size, 1.0);
            draw_text_ex(
                icon_text,
                x + (size - text_measure.width) / 2.0,
                y + (size + text_measure.height) / 2.0,
                text_params
            );
        }

        // Кількість
        if count > 1 {
            let count_text = count.to_string();
            let font_size = Typography::scale_font_size(Typography::SIZE_SMALL, screen_height());

            // Тінь для тексту
            draw_text_ex(
                &count_text,
                x + size - 15.0,
                y + size - 4.0,
                TextParams {
                    font: Some(font),
                    font_size,
                    color: Color::new(0.0, 0.0, 0.0, 1.0),
                    ..Default::default()
                }
            );

            // Основний текст
            draw_text_ex(
                &count_text,
                x + size - 16.0,
                y + size - 5.0,
                TextParams {
                    font: Some(font),
                    font_size,
                    color: ColorPalette::TEXT_PRIMARY,
                    ..Default::default()
                }
            );
        }

        // Підсвічування при hover
        if is_hovered {
            draw_rectangle(x, y, size, size, Color::new(1.0, 1.0, 1.0, 0.1));
        }

        is_hovered && is_mouse_button_pressed(MouseButton::Left)
    }

    /// Малює індикатор стану (здоров'я, голод, спрага)
    pub fn draw_status_bar(
        &self,
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

        // Фон бару
        draw_rectangle(x, y, w, h, ColorPalette::PANEL_BG_DARK);

        // Критичний стан (мигання)
        if progress < 0.25 {
            let pulse = ((get_time() * 3.0).sin() * 0.3 + 0.7) as f32;
            let warning_color = Color::new(
                fg_color.r * pulse,
                fg_color.g * pulse,
                fg_color.b * pulse,
                1.0
            );
            draw_rectangle(x, y, w * progress, h, warning_color);
        } else {
            draw_rectangle(x, y, w * progress, h, fg_color);
        }

        // Градієнт для об'єму
        for i in 0..10 {
            let alpha = (i as f32 / 10.0) * 0.3;
            let grad_h = h * (1.0 - i as f32 / 10.0);
            draw_rectangle(
                x,
                y,
                w * progress,
                grad_h,
                Color::new(1.0, 1.0, 1.0, alpha)
            );
        }

        // Рамка
        draw_rectangle_lines(x, y, w, h, Spacing::BORDER_MEDIUM, ColorPalette::BORDER_MAIN);

        // Внутрішня світла рамка
        draw_rectangle_lines(
            x + 1.0,
            y + 1.0,
            w - 2.0,
            h - 2.0,
            1.0,
            ColorPalette::BORDER_LIGHT
        );

        // Іконка та текст
        let font_size = Typography::scale_font_size(Typography::SIZE_SMALL, screen_height());
        let text = format!("{} {}: {:.0}/{:.0}", icon, label, value, max_value);

        // Тінь тексту
        draw_text_ex(
            &text,
            x + 6.0,
            y + h / 2.0 + font_size as f32 / 3.0 + 1.0,
            TextParams {
                font: Some(font),
                font_size,
                color: Color::new(0.0, 0.0, 0.0, 1.0),
                ..Default::default()
            }
        );

        // Основний текст
        draw_text_ex(
            &text,
            x + 5.0,
            y + h / 2.0 + font_size as f32 / 3.0,
            TextParams {
                font: Some(font),
                font_size,
                color: ColorPalette::TEXT_PRIMARY,
                ..Default::default()
            }
        );
    }

    /// Малює tooltip при наведенні
    pub fn draw_tooltip(&self, text: &str, font: &Font) {
        let (mx, my) = mouse_position();
        let font_size = Typography::scale_font_size(Typography::SIZE_SMALL, screen_height());
        let text_measure = measure_text(text, Some(font), font_size, 1.0);

        let padding = Spacing::SMALL;
        let w = text_measure.width + padding * 2.0;
        let h = text_measure.height + padding * 2.0;

        let x = (mx + 15.0).min(screen_width() - w);
        let y = (my - h - 10.0).max(0.0);

        // Тінь
        draw_rectangle(x + 3.0, y + 3.0, w, h, ColorPalette::SHADOW_DARK);

        // Фон
        draw_rectangle(x, y, w, h, ColorPalette::PANEL_BG);

        // Рамка
        draw_rectangle_lines(x, y, w, h, Spacing::BORDER_THIN, ColorPalette::BORDER_ACCENT);

        // Текст
        draw_text_ex(
            text,
            x + padding,
            y + padding + text_measure.height / 2.0,
            TextParams {
                font: Some(font),
                font_size,
                color: ColorPalette::TEXT_PRIMARY,
                ..Default::default()
            }
        );
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum ItemRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl Default for ItemRarity {
    fn default() -> Self {
        ItemRarity::Common
    }
}
