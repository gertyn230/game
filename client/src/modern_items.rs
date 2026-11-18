//! Покращена система відображення предметів для Pixel Village
//! Система рідкості, детальні іконки, ефекти сяйва

use macroquad::prelude::*;
use crate::design_system::*;
use crate::data::{It, Tl, NI, NT};

/// Рівень рідкості предмета
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ItemRarity {
    Common,    // Звичайний (сірий)
    Uncommon,  // Незвичайний (зелений)
    Rare,      // Рідкісний (синій)
    Epic,      // Епічний (фіолетовий)
    Legendary, // Легендарний (помаранчевий)
}

impl ItemRarity {
    /// Колір рамки для рівня рідкості
    pub fn frame_color(&self) -> u32 {
        match self {
            ItemRarity::Common => 0x8a8a8a,
            ItemRarity::Uncommon => 0x3a9a3a,
            ItemRarity::Rare => 0x3a7ad9,
            ItemRarity::Epic => 0x9a3ad9,
            ItemRarity::Legendary => 0xd9823a,
        }
    }

    /// Ефект сяйва (pulse) для рідкісних предметів
    pub fn has_glow(&self) -> bool {
        matches!(self, ItemRarity::Rare | ItemRarity::Epic | ItemRarity::Legendary)
    }

    /// Інтенсивність сяйва (0.0 - 1.0)
    pub fn glow_intensity(&self) -> f32 {
        match self {
            ItemRarity::Common | ItemRarity::Uncommon => 0.0,
            ItemRarity::Rare => 0.3,
            ItemRarity::Epic => 0.5,
            ItemRarity::Legendary => 0.7,
        }
    }
}

/// Отримати рідкість предмета за його індексом
pub fn get_item_rarity(item_idx: usize) -> ItemRarity {
    match item_idx {
        // Звичайні (Common) - базові ресурси
        0..=6 => ItemRarity::Common, // Wood, Stone, Food, Stick, Pebble, Fiber, Plank

        // Незвичайні (Uncommon) - оброблені ресурси та їжа
        7..=10 => ItemRarity::Uncommon, // Meal, Meat, Roast, Hide

        // Рідкісні (Rare) - риба, вугілля, руди
        11..=16 => ItemRarity::Rare, // Worm, Fish, RFish, Coal, CuOre, FeOre

        // Епічні (Epic) - метали, ягоди, гриби
        17..=22 => ItemRarity::Epic, // Cu, Fe, Berry, Mushroom, Herb, AgOre

        // Легендарні (Legendary) - дорогоцінні метали
        23..=25 => ItemRarity::Legendary, // AuOre, Ag, Au

        _ => ItemRarity::Common,
    }
}

/// Отримати рідкість інструмента за його індексом
pub fn get_tool_rarity(tool_idx: usize) -> ItemRarity {
    match tool_idx {
        0..=1 => ItemRarity::Uncommon,  // Axe, Pick (камінь)
        2..=3 => ItemRarity::Rare,      // Axe2, Pick2 (мідь)
        4..=5 => ItemRarity::Epic,      // Axe3, Pick3 (залізо)
        6..=9 => ItemRarity::Uncommon,  // Spear, Knife, Shovel, Rod
        10..=11 => ItemRarity::Rare,    // Basket, Boat
        12..=13 => ItemRarity::Epic,    // Coat, Mail (броня)
        _ => ItemRarity::Common,
    }
}

/// Малює іконку предмета з покращеною графікою
pub fn draw_modern_item_icon(
    item_idx: usize,
    x: f32,
    y: f32,
    size: f32,
    now: f64,
    show_rarity: bool,
) {
    let rarity = get_item_rarity(item_idx);

    // Тінь іконки
    draw_rectangle(
        x + 2.0,
        y + 2.0,
        size,
        size,
        Color::new(0.0, 0.0, 0.0, 0.3),
    );

    // Фон іконки
    draw_rectangle(
        x,
        y,
        size,
        size,
        ColorPalette::PANEL_BG,
    );

    // Рамка рідкості
    if show_rarity {
        let frame_col = rarity.frame_color();
        let thickness = 2.0;

        // Зовнішня рамка
        draw_rectangle_lines(
            x,
            y,
            size,
            size,
            thickness,
            Color::from_rgba(
                ((frame_col >> 16) & 0xFF) as u8,
                ((frame_col >> 8) & 0xFF) as u8,
                (frame_col & 0xFF) as u8,
                255,
            ),
        );

        // Ефект сяйва для рідкісних предметів
        if rarity.has_glow() {
            let pulse = ((now * 3.0).sin() * 0.5 + 0.5) as f32;
            let glow_alpha = pulse * rarity.glow_intensity();

            draw_rectangle(
                x,
                y,
                size,
                size,
                Color::new(1.0, 1.0, 1.0, glow_alpha * 0.2),
            );
        }
    }

    // Малюємо саму іконку предмета (Pixel Art)
    draw_item_pixels(item_idx, x + size * 0.1, y + size * 0.1, size * 0.8);
}

/// Малює іконку інструмента з покращеною графікою
pub fn draw_modern_tool_icon(
    tool_idx: usize,
    x: f32,
    y: f32,
    size: f32,
    now: f64,
    show_rarity: bool,
    durability: i32,
    max_durability: i32,
) {
    let rarity = get_tool_rarity(tool_idx);

    // Тінь іконки
    draw_rectangle(
        x + 2.0,
        y + 2.0,
        size,
        size,
        Color::new(0.0, 0.0, 0.0, 0.3),
    );

    // Фон іконки
    draw_rectangle(
        x,
        y,
        size,
        size,
        ColorPalette::PANEL_BG,
    );

    // Рамка рідкості
    if show_rarity {
        let frame_col = rarity.frame_color();
        let thickness = 2.0;

        draw_rectangle_lines(
            x,
            y,
            size,
            size,
            thickness,
            Color::from_rgba(
                ((frame_col >> 16) & 0xFF) as u8,
                ((frame_col >> 8) & 0xFF) as u8,
                (frame_col & 0xFF) as u8,
                255,
            ),
        );

        // Ефект сяйва
        if rarity.has_glow() {
            let pulse = ((now * 3.0).sin() * 0.5 + 0.5) as f32;
            let glow_alpha = pulse * rarity.glow_intensity();

            draw_rectangle(
                x,
                y,
                size,
                size,
                Color::new(1.0, 1.0, 1.0, glow_alpha * 0.2),
            );
        }
    }

    // Малюємо саму іконку інструмента
    draw_tool_pixels(tool_idx, x + size * 0.1, y + size * 0.1, size * 0.8);

    // Індикатор міцності (якщо є)
    if max_durability > 1 {
        let durability_ratio = durability as f32 / max_durability as f32;
        let bar_height = 3.0;
        let bar_y = y + size - bar_height - 2.0;

        // Фон індикатора
        draw_rectangle(
            x + 2.0,
            bar_y,
            size - 4.0,
            bar_height,
            Color::new(0.2, 0.2, 0.2, 0.7),
        );

        // Заповнення індикатора (градієнт зелений -> жовтий -> червоний)
        let color = if durability_ratio > 0.5 {
            Color::new(0.3, 0.8, 0.3, 1.0)
        } else if durability_ratio > 0.25 {
            Color::new(0.9, 0.7, 0.2, 1.0)
        } else {
            Color::new(0.9, 0.2, 0.2, 1.0)
        };

        draw_rectangle(
            x + 2.0,
            bar_y,
            (size - 4.0) * durability_ratio,
            bar_height,
            color,
        );
    }
}

/// Малює Pixel Art іконку предмета
fn draw_item_pixels(item_idx: usize, x: f32, y: f32, size: f32) {
    let pixel_size = size / 8.0; // 8x8 піксельна іконка

    let p = |px: i32, py: i32, col: u32| {
        draw_rectangle(
            x + px as f32 * pixel_size,
            y + py as f32 * pixel_size,
            pixel_size,
            pixel_size,
            Color::from_rgba(
                ((col >> 16) & 0xFF) as u8,
                ((col >> 8) & 0xFF) as u8,
                (col & 0xFF) as u8,
                255,
            ),
        );
    };

    // Малюємо іконку в залежності від типу предмета
    match item_idx {
        0 => draw_wood_icon(p, pixel_size), // Wood
        1 => draw_stone_icon(p), // Stone
        2 => draw_food_icon(p), // Food
        3 => draw_stick_icon(p), // Stick
        4 => draw_pebble_icon(p), // Pebble
        5 => draw_fiber_icon(p), // Fiber
        6 => draw_plank_icon(p), // Plank
        7 => draw_meal_icon(p), // Meal
        8 => draw_meat_icon(p), // Meat
        9 => draw_roast_icon(p), // Roast
        10 => draw_hide_icon(p), // Hide
        11 => draw_worm_icon(p), // Worm
        12 => draw_fish_icon(p), // Fish
        13 => draw_rfish_icon(p), // RFish (Roasted Fish)
        14 => draw_coal_icon(p), // Coal
        15 => draw_ore_icon(p, 0xd9823a), // CuOre (мідна руда)
        16 => draw_ore_icon(p, 0xb5613a), // FeOre (залізна руда)
        17 => draw_ingot_icon(p, 0xe08a3a), // Cu (мідь)
        18 => draw_ingot_icon(p, 0xc8ccd4), // Fe (залізо)
        19 => draw_berry_icon(p), // Berry
        20 => draw_mushroom_icon(p), // Mushroom
        21 => draw_herb_icon(p), // Herb
        22 => draw_ore_icon(p, 0xd0d0d8), // AgOre (срібна руда)
        23 => draw_ore_icon(p, 0xffcc33), // AuOre (золота руда)
        24 => draw_ingot_icon(p, 0xe0e0e8), // Ag (срібло)
        25 => draw_ingot_icon(p, 0xffdd00), // Au (золото)
        _ => {}
    }
}

/// Малює Pixel Art іконку інструмента
fn draw_tool_pixels(tool_idx: usize, x: f32, y: f32, size: f32) {
    let pixel_size = size / 8.0;

    let p = |px: i32, py: i32, col: u32| {
        draw_rectangle(
            x + px as f32 * pixel_size,
            y + py as f32 * pixel_size,
            pixel_size,
            pixel_size,
            Color::from_rgba(
                ((col >> 16) & 0xFF) as u8,
                ((col >> 8) & 0xFF) as u8,
                (col & 0xFF) as u8,
                255,
            ),
        );
    };

    match tool_idx {
        0 | 2 | 4 => draw_axe_icon(p, tool_idx / 2), // Axe, Axe2, Axe3
        1 | 3 | 5 => draw_pick_icon(p, tool_idx / 2), // Pick, Pick2, Pick3
        6 => draw_spear_icon(p), // Spear
        7 => draw_knife_icon(p), // Knife
        8 => draw_shovel_icon(p), // Shovel
        9 => draw_rod_icon(p), // Rod
        10 => draw_basket_icon(p), // Basket
        11 => draw_boat_icon(p), // Boat
        12 => draw_coat_icon(p), // Coat
        13 => draw_mail_icon(p), // Mail
        _ => {}
    }
}

// Далі йдуть функції малювання конкретних іконок (8x8 пікселів)

fn draw_wood_icon<F>(p: F, _pixel_size: f32)
where
    F: Fn(i32, i32, u32),
{
    // Дерев'яна колода (коричнева)
    let wood = 0x8b5a2b;
    let dark = 0x6a4a1a;

    // Торець колоди
    for py in 1..7 {
        for px in 2..6 {
            p(px, py, wood);
        }
    }
    // Кора
    p(2, 1, dark); p(3, 1, dark); p(4, 1, dark); p(5, 1, dark);
    p(2, 6, dark); p(3, 6, dark); p(4, 6, dark); p(5, 6, dark);
    p(1, 2, dark); p(1, 3, dark); p(1, 4, dark); p(1, 5, dark);
    p(6, 2, dark); p(6, 3, dark); p(6, 4, dark); p(6, 5, dark);
    // Кільця дерева
    p(3, 3, dark); p(4, 3, dark);
    p(3, 4, dark); p(4, 4, dark);
}

fn draw_stone_icon<F>(p: F)
where
    F: Fn(i32, i32, u32),
{
    let stone = 0x8d8d96;
    let dark = 0x6a6a70;

    // Камінь (неправильна форма)
    p(3, 1, stone); p(4, 1, stone);
    p(2, 2, stone); p(3, 2, stone); p(4, 2, stone); p(5, 2, stone);
    p(1, 3, stone); p(2, 3, stone); p(3, 3, stone); p(4, 3, stone); p(5, 3, stone);
    p(2, 4, stone); p(3, 4, stone); p(4, 4, stone); p(5, 4, stone); p(6, 4, stone);
    p(2, 5, stone); p(3, 5, stone); p(4, 5, stone); p(5, 5, stone);
    p(3, 6, stone); p(4, 6, stone);
    // Тіні
    p(2, 4, dark); p(3, 5, dark); p(4, 6, dark);
}

fn draw_food_icon<F>(p: F)
where
    F: Fn(i32, i32, u32),
{
    // Морква (помаранчева)
    let carrot = 0xd94f4f;
    let green = 0x3a9a3a;

    // Листя
    p(3, 1, green); p(4, 1, green);
    p(3, 2, green);
    // Морква
    p(3, 3, carrot); p(4, 3, carrot);
    p(2, 4, carrot); p(3, 4, carrot); p(4, 4, carrot); p(5, 4, carrot);
    p(3, 5, carrot); p(4, 5, carrot);
    p(3, 6, carrot);
}

fn draw_stick_icon<F>(p: F)
where
    F: Fn(i32, i32, u32),
{
    let stick = 0xb07a45;

    // Палиця (діагональ)
    for i in 0..7 {
        p(2 + i / 2, 1 + i, stick);
    }
}

fn draw_pebble_icon<F>(p: F)
where
    F: Fn(i32, i32, u32),
{
    let pebble = 0x9a9aa4;

    // Маленький камінчик
    p(3, 3, pebble); p(4, 3, pebble);
    p(2, 4, pebble); p(3, 4, pebble); p(4, 4, pebble); p(5, 4, pebble);
    p(3, 5, pebble); p(4, 5, pebble);
}

fn draw_fiber_icon<F>(p: F)
where
    F: Fn(i32, i32, u32),
{
    let fiber = 0x9ad06a;

    // Волокна (хвилясті лінії)
    p(2, 2, fiber); p(3, 1, fiber); p(4, 2, fiber); p(5, 1, fiber);
    p(2, 4, fiber); p(3, 3, fiber); p(4, 4, fiber); p(5, 3, fiber);
    p(2, 6, fiber); p(3, 5, fiber); p(4, 6, fiber); p(5, 5, fiber);
}

fn draw_plank_icon<F>(p: F)
where
    F: Fn(i32, i32, u32),
{
    let plank = 0xc9a06a;
    let dark = 0x8a6a3a;

    // Дошка
    for py in 2..6 {
        for px in 1..7 {
            p(px, py, plank);
        }
    }
    // Лінії дерева
    for px in 1..7 {
        p(px, 3, dark);
        p(px, 5, dark);
    }
}

fn draw_meal_icon<F>(p: F)
where
    F: Fn(i32, i32, u32),
{
    let bread = 0xe0b070;
    let crust = 0xb08040;

    // Хліб/пайок
    p(2, 2, crust); p(3, 2, crust); p(4, 2, crust); p(5, 2, crust);
    p(2, 3, crust); p(3, 3, bread); p(4, 3, bread); p(5, 3, crust);
    p(2, 4, crust); p(3, 4, bread); p(4, 4, bread); p(5, 4, crust);
    p(2, 5, crust); p(3, 5, crust); p(4, 5, crust); p(5, 5, crust);
}

fn draw_meat_icon<F>(p: F)
where
    F: Fn(i32, i32, u32),
{
    let meat = 0xb03a3a;
    let bone = 0xffffff;

    // М'ясо з кісткою
    p(2, 2, bone); p(5, 2, bone);
    p(2, 3, meat); p(3, 3, meat); p(4, 3, meat); p(5, 3, meat);
    p(2, 4, meat); p(3, 4, meat); p(4, 4, meat); p(5, 4, meat);
    p(2, 5, meat); p(3, 5, meat); p(4, 5, meat); p(5, 5, meat);
    p(2, 6, bone); p(5, 6, bone);
}

fn draw_roast_icon<F>(p: F)
where
    F: Fn(i32, i32, u32),
{
    let roast = 0xc26a2a;

    // Смажене м'ясо
    p(2, 2, roast); p(3, 2, roast); p(4, 2, roast); p(5, 2, roast);
    p(2, 3, roast); p(3, 3, roast); p(4, 3, roast); p(5, 3, roast);
    p(2, 4, roast); p(3, 4, roast); p(4, 4, roast); p(5, 4, roast);
    p(3, 5, roast); p(4, 5, roast);
}

fn draw_hide_icon<F>(p: F)
where
    F: Fn(i32, i32, u32),
{
    let hide = 0x7a5a3a;

    // Шкура (неправильна форма)
    p(3, 1, hide);
    p(2, 2, hide); p(3, 2, hide); p(4, 2, hide);
    p(1, 3, hide); p(2, 3, hide); p(3, 3, hide); p(4, 3, hide); p(5, 3, hide);
    p(2, 4, hide); p(3, 4, hide); p(4, 4, hide); p(5, 4, hide); p(6, 4, hide);
    p(2, 5, hide); p(3, 5, hide); p(4, 5, hide); p(5, 5, hide);
    p(3, 6, hide); p(4, 6, hide);
}

fn draw_worm_icon<F>(p: F)
where
    F: Fn(i32, i32, u32),
{
    let worm = 0xd98a8a;

    // Черв'як (хвиляста лінія)
    p(2, 2, worm);
    p(3, 3, worm);
    p(4, 2, worm);
    p(5, 3, worm);
    p(4, 4, worm);
    p(3, 5, worm);
    p(4, 6, worm);
}

fn draw_fish_icon<F>(p: F)
where
    F: Fn(i32, i32, u32),
{
    let fish = 0x6aa0d0;
    let eye = 0x000000;

    // Риба
    p(2, 3, fish); p(3, 2, fish); p(4, 2, fish); p(5, 2, fish); p(6, 3, fish);
    p(2, 4, fish); p(3, 3, fish); p(4, 3, fish); p(5, 3, fish); p(6, 4, fish);
    p(2, 5, fish); p(3, 4, fish); p(4, 4, fish); p(5, 4, fish); p(6, 5, fish);
    p(3, 5, fish); p(4, 5, fish); p(5, 5, fish);
    // Око
    p(5, 3, eye);
    // Хвіст
    p(1, 3, fish); p(1, 4, fish); p(1, 5, fish);
}

fn draw_rfish_icon<F>(p: F)
where
    F: Fn(i32, i32, u32),
{
    let rfish = 0xe0a050;
    let eye = 0x000000;

    // Смажена риба (аналогічно fish, але помаранчева)
    p(2, 3, rfish); p(3, 2, rfish); p(4, 2, rfish); p(5, 2, rfish); p(6, 3, rfish);
    p(2, 4, rfish); p(3, 3, rfish); p(4, 3, rfish); p(5, 3, rfish); p(6, 4, rfish);
    p(2, 5, rfish); p(3, 4, rfish); p(4, 4, rfish); p(5, 4, rfish); p(6, 5, rfish);
    p(3, 5, rfish); p(4, 5, rfish); p(5, 5, rfish);
    p(5, 3, eye);
    p(1, 3, rfish); p(1, 4, rfish); p(1, 5, rfish);
}

fn draw_coal_icon<F>(p: F)
where
    F: Fn(i32, i32, u32),
{
    let coal = 0x222228;

    // Вугілля (чорний шматок)
    p(3, 2, coal); p(4, 2, coal);
    p(2, 3, coal); p(3, 3, coal); p(4, 3, coal); p(5, 3, coal);
    p(2, 4, coal); p(3, 4, coal); p(4, 4, coal); p(5, 4, coal);
    p(3, 5, coal); p(4, 5, coal); p(5, 5, coal);
}

fn draw_ore_icon<F>(p: F, ore_color: u32)
where
    F: Fn(i32, i32, u32),
{
    let stone = 0x8d8d96;

    // Руда (камінь з вкрапленнями металу)
    p(3, 1, stone); p(4, 1, stone);
    p(2, 2, stone); p(3, 2, ore_color); p(4, 2, stone); p(5, 2, stone);
    p(1, 3, stone); p(2, 3, stone); p(3, 3, stone); p(4, 3, ore_color); p(5, 3, stone);
    p(2, 4, stone); p(3, 4, ore_color); p(4, 4, stone); p(5, 4, stone); p(6, 4, stone);
    p(2, 5, stone); p(3, 5, stone); p(4, 5, stone); p(5, 5, ore_color);
    p(3, 6, stone); p(4, 6, stone);
}

fn draw_ingot_icon<F>(p: F, metal_color: u32)
where
    F: Fn(i32, i32, u32),
{
    // Злиток металу
    p(2, 2, metal_color); p(3, 2, metal_color); p(4, 2, metal_color); p(5, 2, metal_color);
    p(2, 3, metal_color); p(3, 3, metal_color); p(4, 3, metal_color); p(5, 3, metal_color);
    p(2, 4, metal_color); p(3, 4, metal_color); p(4, 4, metal_color); p(5, 4, metal_color);
    p(3, 5, metal_color); p(4, 5, metal_color);
}

fn draw_berry_icon<F>(p: F)
where
    F: Fn(i32, i32, u32),
{
    let berry = 0xff3366;
    let leaf = 0x3a9a3a;

    // Ягоди
    p(3, 2, leaf);
    p(2, 3, berry); p(4, 3, berry);
    p(3, 4, berry); p(5, 4, berry);
    p(4, 5, berry);
}

fn draw_mushroom_icon<F>(p: F)
where
    F: Fn(i32, i32, u32),
{
    let cap = 0xcc6633;
    let stem = 0xffffff;
    let spot = 0xffffff;

    // Гриб
    p(2, 2, cap); p(3, 2, cap); p(4, 2, cap); p(5, 2, cap);
    p(2, 3, cap); p(3, 3, cap); p(4, 3, cap); p(5, 3, cap);
    p(3, 4, stem); p(4, 4, stem);
    p(3, 5, stem); p(4, 5, stem);
    // Плями на шапці
    p(2, 2, spot); p(5, 2, spot);
}

fn draw_herb_icon<F>(p: F)
where
    F: Fn(i32, i32, u32),
{
    let herb = 0x66cc66;

    // Трава (кілька стебел)
    p(2, 2, herb); p(2, 3, herb); p(2, 4, herb); p(2, 5, herb);
    p(4, 1, herb); p(4, 2, herb); p(4, 3, herb); p(4, 4, herb); p(4, 5, herb);
    p(6, 2, herb); p(6, 3, herb); p(6, 4, herb);
}

// Функції малювання іконок інструментів

fn draw_axe_icon<F>(p: F, tier: usize)
where
    F: Fn(i32, i32, u32),
{
    let handle = 0xb07a45;
    let blade = match tier {
        0 => 0x9a9aa4, // Камінь
        1 => 0xe08a3a, // Мідь
        _ => 0xc8ccd4, // Залізо
    };

    // Ручка
    for i in 0..6 {
        p(1 + i / 2, 2 + i, handle);
    }
    // Лезо
    p(4, 1, blade); p(5, 1, blade);
    p(5, 2, blade); p(6, 2, blade);
    p(6, 3, blade);
}

fn draw_pick_icon<F>(p: F, tier: usize)
where
    F: Fn(i32, i32, u32),
{
    let handle = 0xb07a45;
    let head = match tier {
        0 => 0x9a9aa4,
        1 => 0xe08a3a,
        _ => 0xc8ccd4,
    };

    // Ручка
    p(3, 3, handle);
    p(3, 4, handle);
    p(3, 5, handle);
    p(3, 6, handle);
    // Голова кірки
    p(1, 2, head); p(2, 2, head); p(3, 2, head); p(4, 2, head); p(5, 2, head);
    p(2, 3, head); p(3, 3, head); p(4, 3, head);
}

fn draw_spear_icon<F>(p: F)
where
    F: Fn(i32, i32, u32),
{
    let shaft = 0xb07a45;
    let tip = 0x9a9aa4;

    // Древко
    for i in 1..7 {
        p(3, i, shaft);
    }
    // Наконечник
    p(3, 0, tip);
    p(2, 1, tip); p(4, 1, tip);
}

fn draw_knife_icon<F>(p: F)
where
    F: Fn(i32, i32, u32),
{
    let handle = 0xb07a45;
    let blade = 0x9a9aa4;

    // Ручка
    p(2, 4, handle); p(2, 5, handle); p(2, 6, handle);
    // Лезо
    p(3, 1, blade);
    p(3, 2, blade); p(4, 2, blade);
    p(3, 3, blade); p(4, 3, blade);
    p(2, 3, blade); p(3, 4, blade);
}

fn draw_shovel_icon<F>(p: F)
where
    F: Fn(i32, i32, u32),
{
    let handle = 0xb07a45;
    let blade = 0x9a9aa4;

    // Ручка
    for i in 0..5 {
        p(3, i, handle);
    }
    // Лопата
    p(2, 5, blade); p(3, 5, blade); p(4, 5, blade);
    p(2, 6, blade); p(3, 6, blade); p(4, 6, blade);
}

fn draw_rod_icon<F>(p: F)
where
    F: Fn(i32, i32, u32),
{
    let rod = 0xb07a45;
    let line = 0x6a6a70;

    // Вудка
    for i in 0..7 {
        p(2, i, rod);
    }
    // Волосінь
    p(3, 0, line); p(4, 0, line); p(5, 0, line); p(6, 0, line);
    p(6, 1, line);
}

fn draw_basket_icon<F>(p: F)
where
    F: Fn(i32, i32, u32),
{
    let basket = 0xc9a06a;

    // Корзина
    p(2, 3, basket); p(5, 3, basket);
    p(2, 4, basket); p(3, 4, basket); p(4, 4, basket); p(5, 4, basket);
    p(2, 5, basket); p(3, 5, basket); p(4, 5, basket); p(5, 5, basket);
    p(2, 6, basket); p(3, 6, basket); p(4, 6, basket); p(5, 6, basket);
}

fn draw_boat_icon<F>(p: F)
where
    F: Fn(i32, i32, u32),
{
    let boat = 0x8b5a2b;

    // Човен
    p(2, 4, boat); p(3, 4, boat); p(4, 4, boat); p(5, 4, boat);
    p(2, 5, boat); p(3, 5, boat); p(4, 5, boat); p(5, 5, boat);
    p(3, 6, boat); p(4, 6, boat);
}

fn draw_coat_icon<F>(p: F)
where
    F: Fn(i32, i32, u32),
{
    let coat = 0x7a5a3a;

    // Плащ/пальто
    p(2, 2, coat); p(3, 2, coat); p(4, 2, coat); p(5, 2, coat);
    p(2, 3, coat); p(3, 3, coat); p(4, 3, coat); p(5, 3, coat);
    p(2, 4, coat); p(3, 4, coat); p(4, 4, coat); p(5, 4, coat);
    p(2, 5, coat); p(5, 5, coat);
    p(2, 6, coat); p(5, 6, coat);
}

fn draw_mail_icon<F>(p: F)
where
    F: Fn(i32, i32, u32),
{
    let mail = 0xc8ccd4;
    let dark = 0x8a8a90;

    // Кольчуга (візерунок кілець)
    for py in 2..6 {
        for px in 2..6 {
            if (px + py) % 2 == 0 {
                p(px, py, mail);
            } else {
                p(px, py, dark);
            }
        }
    }
}
