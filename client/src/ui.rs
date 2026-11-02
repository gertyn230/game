//! Мини-«интерфейс»: кнопки, которые видны и нажимаются и пальцем, и мышью.
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
    // Нові команди для налаштувань
    ToggleMusic,
    ToggleSound,
    MusicVolume(f32),
    SoundVolume(f32),
    ChangeLanguage,
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
    /// Нарисовать кнопку и зарегистрировать её для нажатия.
    pub fn button(&mut self, font: &Font, r: Rect, label: &str, cmd: Cmd, enabled: bool, color: u32, fs: f32) {
        let col = if enabled { hx(color) } else { ha(0x555555, 0.8) };
        draw_rectangle(r.x, r.y + 3.0, r.w, r.h, ha(0x000000, 0.35));
        draw_rectangle(r.x, r.y, r.w, r.h, col);
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, ha(0xffffff, if enabled { 0.5 } else { 0.2 }));
        let mut size = fs;
        while text_w(font, label, size) > r.w - 8.0 && size > 9.0 {
            size -= 1.0;
        }
        text_c(font, label, r.x + r.w / 2.0, r.y + r.h / 2.0 + size * 0.35, size, if enabled { WHITE } else { ha(0xffffff, 0.5) });
        if enabled {
            self.next.push(Btn { r, cmd });
        }
    }
}

pub fn bar(font: &Font, x: f32, y: f32, w: f32, h: f32, v: f32, col: u32, label: &str, fs: f32) {
    draw_rectangle(x, y, w, h, ha(0x000000, 0.55));
    draw_rectangle(x, y, w * (v / 100.0).clamp(0.0, 1.0), h, hx(col));
    draw_rectangle_lines(x, y, w, h, 1.5, ha(0xffffff, 0.35));
    text_c(font, &format!("{} {}", label, v.round() as i32), x + w / 2.0, y + h * 0.5 + fs * 0.33, fs * 0.9, WHITE);
}
