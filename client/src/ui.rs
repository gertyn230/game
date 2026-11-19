//! Мини-«інтерфейс»: кнопки в піксельному дерев'яному стилі.
use macroquad::prelude::*;

use crate::art::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Cmd {
    Act,
    Eat,
    Drink,
    Dig,
    Menu,
    Close,
    Tab(u8),
    Craft(usize),
    Quick(usize),
    Bridge,
    Boat,
    Tame,
    MktBuy(u32),
    MktWithdraw(u32),
    MktPost,
    MktItem(i32),
    MktQty(i32),
    MktPrice(i32),
    NameOpen,
    NameOk,
    Key(char),
    Space,
    Back,
    Lang,
    // Команди налаштувань
    ToggleMusic,
    ToggleSound,
    MusicVolume(f32),
    SoundVolume(f32),
    ChangeLanguage,
    SettingsSubtab(u8),
    Rebind(crate::settings::RebindAction),
    ResetKeybinds,
    Inventory,
    CraftMenu,
    Settings,
    Quit,
}

pub struct Btn {
    pub r: Rect,
    pub cmd: Cmd,
}

#[derive(Default)]
pub struct Ui {
    pub btns: Vec<Btn>,
    next: Vec<Btn>,
    pub menu: bool,
    pub tab: u8,
    pub settings_subtab: u8,
    pub rebinding: Option<crate::settings::RebindAction>,
    pub scroll: f32,
    pub panel: Rect,
    pub mk_item: usize,
    pub mk_qty: u32,
    pub mk_price: u32,
}

impl Ui {
    pub fn new() -> Ui {
        Ui { mk_qty: 1, mk_price: 5, ..Default::default() }
    }
    pub fn begin(&mut self) {
        self.next.clear();
    }
    pub fn finish(&mut self) {
        std::mem::swap(&mut self.btns, &mut self.next);
    }
    pub fn btns_push_circle(&mut self, r: Rect, cmd: Cmd) {
        self.next.push(Btn { r, cmd });
    }
    pub fn hit(&self, p: Vec2) -> Option<Cmd> {
        self.btns.iter().rev().find(|b| b.r.contains(p)).map(|b| b.cmd)
    }

    /// Піксельна дерев'яна кнопка
    pub fn button(&mut self, font: &Font, r: Rect, label: &str, cmd: Cmd, enabled: bool, color: u32, fs: f32) {
        let col = if enabled { hx(color) } else { ha(0x444444, 0.7) };

        // Тінь під кнопкою (піксельна — зміщена на 2-3 пікселі)
        draw_rectangle(r.x + 2.0, r.y + 3.0, r.w, r.h, ha(0x000000, 0.4));

        // Основний фон кнопки
        draw_rectangle(r.x, r.y, r.w, r.h, col);

        // Дерев'яна текстура — тонкі горизонтальні лінії
        if r.h > 20.0 {
            for i in 0..(r.h as i32 / 8) {
                let ly = r.y + i as f32 * 8.0;
                if ly < r.y + r.h {
                    let alpha = if i % 3 == 0 { 0.04 } else { 0.02 };
                    draw_rectangle(r.x + 1.0, ly, r.w - 2.0, 1.0, ha(0xffffff, alpha));
                }
            }
        }

        // Верхній блік (3D ефект)
        draw_rectangle(r.x, r.y, r.w, r.h * 0.4, ha(0xffffff, if enabled { 0.1 } else { 0.04 }));
        // Нижня тінь
        draw_rectangle(r.x, r.y + r.h * 0.6, r.w, r.h * 0.4, ha(0x000000, if enabled { 0.18 } else { 0.1 }));

        // Зовнішня рамка — темна
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, ha(0x000000, 0.5));
        // Внутрішня рамка — світла
        draw_rectangle_lines(r.x + 2.0, r.y + 2.0, r.w - 4.0, r.h - 4.0, 1.0,
            ha(0xffffff, if enabled { 0.12 } else { 0.04 }));

        // Декоративні кутові пікселі (лише для великих кнопок)
        if r.w > 60.0 && r.h > 30.0 {
            let cs = 2.0;
            let cc = ha(0xffffff, if enabled { 0.18 } else { 0.06 });
            draw_rectangle(r.x + 5.0, r.y + 5.0, cs, cs, cc);
            draw_rectangle(r.x + r.w - 7.0, r.y + 5.0, cs, cs, cc);
            draw_rectangle(r.x + 5.0, r.y + r.h - 7.0, cs, cs, cc);
            draw_rectangle(r.x + r.w - 7.0, r.y + r.h - 7.0, cs, cs, cc);
        }

        // Текст з тінню
        let mut size = fs;
        while text_w(font, label, size) > r.w - 10.0 && size > 9.0 {
            size -= 1.0;
        }
        let tx = r.x + r.w / 2.0;
        let ty = r.y + r.h / 2.0 + size * 0.35;
        // Тінь тексту
        text_c(font, label, tx + 1.0, ty + 1.0, size, ha(0x000000, 0.55));
        // Основний текст
        text_c(font, label, tx, ty, size, if enabled { WHITE } else { ha(0xffffff, 0.4) });

        if enabled {
            self.next.push(Btn { r, cmd });
        }
    }
}

/// Піксельний бар стану (HP, голод, вода, енергія)
pub fn bar(font: &Font, x: f32, y: f32, w: f32, h: f32, v: f32, col: u32, label: &str, fs: f32) {
    let fill = (v / 100.0).clamp(0.0, 1.0);

    // Тінь під баром
    draw_rectangle(x + 2.0, y + 2.0, w, h, ha(0x000000, 0.4));

    // Темний фон
    draw_rectangle(x, y, w, h, Color::from_rgba(22, 20, 16, 200));

    // Дерев'яна текстура фону
    for i in 0..(h as i32 / 4) {
        let ly = y + i as f32 * 4.0;
        if ly < y + h {
            draw_rectangle(x, ly, w, 1.0, ha(0xffffff, 0.015));
        }
    }

    // Заповнення
    let fill_w = w * fill;
    if fill_w > 0.0 {
        draw_rectangle(x, y, fill_w, h, hx(col));
        // Верхній блік
        draw_rectangle(x, y, fill_w, h * 0.35, ha(0xffffff, 0.12));
        // Нижня тінь
        draw_rectangle(x, y + h * 0.7, fill_w, h * 0.3, ha(0x000000, 0.2));
        // Піксельний край заповнення
        if fill_w > 2.0 && fill < 0.98 {
            draw_rectangle(x + fill_w - 2.0, y, 2.0, h, ha(0xffffff, 0.1));
        }
    }

    // Рамка — подвійна піксельна
    draw_rectangle_lines(x, y, w, h, 2.0, ha(0x000000, 0.6));
    draw_rectangle_lines(x + 1.0, y + 1.0, w - 2.0, h - 2.0, 1.0, ha(0xffffff, 0.1));

    // Іконка стану (ліворуч)
    let icon = match col {
        0xe0933a => "H", // hunger
        0x4aa3df => "W", // water
        0xd9c23f => "E", // energy
        0xd9534f => "HP",
        _ => "",
    };
    if !icon.is_empty() {
        text(font, icon, x + 3.0, y + h * 0.75, fs * 0.7, ha(0xffffff, 0.6));
    }

    // Текст з тінню
    let display = format!("{} {}", label, v.round() as i32);
    text_c(font, &display, x + w / 2.0 + 1.0, y + h * 0.5 + fs * 0.34, fs * 0.85, ha(0x000000, 0.55));
    text_c(font, &display, x + w / 2.0, y + h * 0.5 + fs * 0.33, fs * 0.85, WHITE);
}
