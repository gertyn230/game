//! Покращене малювання тварин для Pixel Village
//! Більш деталізовані Pixel Art спрайти з плавнішими анімаціями

use macroquad::prelude::*;
use crate::game::{Mob, MobKind};
use crate::art::{hx, ha};

/// Допоміжна функція для малювання пікселя з флипом
#[inline]
fn px(sx: f32, sy: f32, u: f32, dx: i32, dy: i32, w: i32, h: i32, col: Color, flip: bool) {
    let x = if flip {
        sx - (dx as f32 + w as f32) * u
    } else {
        sx + dx as f32 * u
    };
    draw_rectangle(x, sy + dy as f32 * u, w as f32 * u, h as f32 * u, col);
}

/// Малює тварину з покращеною графікою
pub fn draw_modern_mob(m: &Mob, sx: f32, sy: f32, u: f32, now: f64) {
    // Якщо тварина мертва
    if m.dead > 0.0 {
        draw_dead_mob(m, sx, sy, u);
        return;
    }

    // Флеш при атаці
    let flash = if m.flash > 0.0 {
        Some(Color::new(1.0, 0.9, 0.9, 1.0))
    } else {
        None
    };

    // Флип (поворот)
    let fl = m.vx < 0.0;

    // Фаза ходи
    let moving = m.vx.abs() > 0.01 || m.vy.abs() > 0.01;
    let phase = if moving {
        (now * 6.0).sin() as f32
    } else {
        0.0
    };

    match m.kind {
        MobKind::Wolf => draw_wolf(sx, sy, u, now, moving, fl, phase, flash, false),
        MobKind::Pet => draw_wolf(sx, sy, u, now, moving, fl, phase, flash, true),
        MobKind::Deer => draw_deer(sx, sy, u, moving, fl, phase, flash),
        MobKind::Bear => draw_bear(sx, sy, u, moving, fl, phase, flash),
        MobKind::Rabbit => draw_rabbit(sx, sy, u, moving, fl, phase, flash),
        MobKind::Fox => draw_fox(sx, sy, u, now, moving, fl, phase, flash),
    }
}

/// Малює мертву тварину
fn draw_dead_mob(m: &Mob, sx: f32, sy: f32, u: f32) {
    let size = match m.kind {
        MobKind::Wolf | MobKind::Pet => 8.0,
        MobKind::Deer => 10.0,
        MobKind::Bear => 12.0,
        MobKind::Rabbit => 5.0,
        MobKind::Fox => 7.0,
    };

    // Тінь під тілом
    draw_rectangle(
        sx - size * u * 0.8,
        sy + 2.0 * u,
        size * u * 1.6,
        3.0 * u,
        ha(0, 0.4),
    );

    // Тіло (перевернуте)
    let color = match m.kind {
        MobKind::Wolf => 0x6a5a4a,
        MobKind::Pet => 0x8a7a6a,
        MobKind::Deer => 0x9a7a5a,
        MobKind::Bear => 0x5a4a3a,
        MobKind::Rabbit => 0xc9aa8a,
        MobKind::Fox => 0xd97a3a,
    };

    draw_rectangle(
        sx - size * u,
        sy - 2.0 * u,
        size * u * 2.0,
        4.0 * u,
        hx(color),
    );

    // Кістки (X очі)
    draw_line(
        sx - 3.0 * u,
        sy - 1.0 * u,
        sx - 1.0 * u,
        sy + 1.0 * u,
        1.0,
        Color::new(0.8, 0.8, 0.8, 0.7),
    );
    draw_line(
        sx - 1.0 * u,
        sy - 1.0 * u,
        sx - 3.0 * u,
        sy + 1.0 * u,
        1.0,
        Color::new(0.8, 0.8, 0.8, 0.7),
    );
}

/// Малює вовка або собаку
#[allow(clippy::too_many_arguments)]
fn draw_wolf(
    sx: f32,
    sy: f32,
    u: f32,
    now: f64,
    moving: bool,
    fl: bool,
    phase: f32,
    flash: Option<Color>,
    is_pet: bool, // true = собака, false = вовк
) {
    // Кольори
    let fur = if is_pet { 0x8a7a6a } else { 0x6a5a4a }; // хутро
    let f2 = if is_pet { 0xbaa090 } else { 0x8a7a6a }; // світліше хутро
    let dk = if is_pet { 0x5a4a3a } else { 0x3a2a1a }; // темні частини

    // Анімація ходи
    let lf = if moving { (phase * 2.0) as i32 } else { 0 };
    let rf = if moving { (-phase * 2.0) as i32 } else { 0 };
    let wag = if is_pet { (now * 5.0).sin() * 1.5 } else { 0.0 };
    let tongue = if is_pet && !moving { ((now / 0.8) as i32 % 2) as f32 * 0.8 } else { 0.0 };

    // Тінь
    draw_rectangle(sx - 7.0 * u, sy + 4.5 * u, 14.0 * u, 2.5 * u, ha(0, 0.3));

    let q = |c: u32, dx: i32, dy: i32, w: i32, h: i32| {
        px(sx, sy, u, dx, dy, w, h, flash.unwrap_or(hx(c)), fl);
    };

    // Ноги
    q(dk, -6, 2 + lf, 2, 2);
    q(dk, -2, 2 + rf, 2, 2);
    q(dk, 1, 2 + rf, 2, 2);
    q(dk, 4, 2 + lf, 2, 2);

    // Хвіст (виляє якщо собака)
    q(fur, -10, -7 + wag as i32, 3, 2);
    q(dk, -10, -7 + wag as i32, 1, 2);

    // Тіло
    q(fur, -7, -5, 12, 7);
    q(f2, -6, -5, 10, 3);
    q(dk, -6, 1, 10, 1);

    // Груди
    q(f2, 3, -2, 3, 3);

    // Голова
    q(fur, 5, -7 + (tongue * 0.3) as i32, 6, 6);

    // Вуха
    q(dk, 5, -11, 2, 3);
    q(dk, 9, -11, 2, 3);
    q(f2, 5, -10, 1, 2);
    q(f2, 9, -10, 1, 2);

    // Очі
    q(if is_pet { 0x000000 } else { 0xffa500 }, 7, -5, 1, 1);
    q(if is_pet { 0x000000 } else { 0xffa500 }, 9, -5, 1, 1);

    // Ніс
    q(0x000000, 8, -3, 2, 1);

    // Язик (тільки собаки)
    if is_pet && tongue > 0.0 {
        q(0xff6b9d, 8, -2 + tongue as i32, 1, tongue as i32 + 1);
    }
}

/// Малює оленя
#[allow(clippy::too_many_arguments)]
fn draw_deer(
    sx: f32,
    sy: f32,
    u: f32,
    moving: bool,
    fl: bool,
    phase: f32,
    flash: Option<Color>,
) {
    // Кольори
    let body = 0x9a7a5a;
    let spot = 0xc9aa8a;
    let dk = 0x5a4a3a;

    // Анімація ходи
    let lf = if moving { (phase * 2.5) as i32 } else { 0 };
    let rf = if moving { (-phase * 2.5) as i32 } else { 0 };

    // Тінь
    draw_rectangle(sx - 8.0 * u, sy + 5.0 * u, 16.0 * u, 2.5 * u, ha(0, 0.3));

    let q = |c: u32, dx: i32, dy: i32, w: i32, h: i32| {
        px(sx, sy, u, dx, dy, w, h, flash.unwrap_or(hx(c)), fl);
    };

    // Ноги (довгі)
    q(dk, -7, 0 + lf, 2, 5);
    q(dk, -3, 0 + rf, 2, 5);
    q(dk, 2, 0 + rf, 2, 5);
    q(dk, 6, 0 + lf, 2, 5);

    // Тіло
    q(body, -8, -7, 14, 8);
    q(spot, -7, -6, 12, 4);

    // Плями
    q(spot, -5, -5, 2, 2);
    q(spot, -1, -6, 2, 2);
    q(spot, 3, -5, 2, 2);

    // Шия
    q(body, 5, -10, 3, 5);
    q(spot, 5, -9, 2, 3);

    // Голова
    q(body, 6, -13, 5, 5);
    q(spot, 6, -12, 3, 3);

    // Роги (розгалужені)
    q(dk, 7, -16, 1, 3);
    q(dk, 9, -16, 1, 3);
    q(dk, 6, -17, 2, 1);
    q(dk, 9, -17, 2, 1);

    // Вуха
    q(dk, 7, -14, 1, 2);
    q(dk, 10, -14, 1, 2);

    // Очі
    q(0x000000, 8, -11, 1, 1);
    q(0x000000, 10, -11, 1, 1);

    // Ніс
    q(0x000000, 9, -9, 2, 1);

    // Хвіст
    q(dk, -9, -4, 2, 3);
}

/// Малює ведмедя
#[allow(clippy::too_many_arguments)]
fn draw_bear(
    sx: f32,
    sy: f32,
    u: f32,
    moving: bool,
    fl: bool,
    phase: f32,
    flash: Option<Color>,
) {
    // Кольори
    let fur = 0x5a4a3a;
    let f2 = 0x7a6a5a;
    let dk = 0x3a2a1a;

    // Анімація ходи
    let lf = if moving { (phase * 1.5) as i32 } else { 0 };
    let rf = if moving { (-phase * 1.5) as i32 } else { 0 };

    // Тінь (великий)
    draw_rectangle(sx - 10.0 * u, sy + 6.0 * u, 20.0 * u, 3.0 * u, ha(0, 0.35));

    let q = |c: u32, dx: i32, dy: i32, w: i32, h: i32| {
        px(sx, sy, u, dx, dy, w, h, flash.unwrap_or(hx(c)), fl);
    };

    // Ноги (товсті)
    q(dk, -8, 1 + lf, 3, 4);
    q(dk, -3, 1 + rf, 3, 4);
    q(dk, 2, 1 + rf, 3, 4);
    q(dk, 7, 1 + lf, 3, 4);

    // Кігті
    q(0xffffff, -8, 4 + lf, 1, 1);
    q(0xffffff, -3, 4 + rf, 1, 1);
    q(0xffffff, 2, 4 + rf, 1, 1);
    q(0xffffff, 7, 4 + lf, 1, 1);

    // Тіло (масивне)
    q(fur, -10, -8, 18, 10);
    q(f2, -9, -7, 16, 6);

    // Горб на спині
    q(fur, -4, -10, 6, 3);

    // Груди
    q(f2, 4, -4, 4, 5);

    // Голова (велика)
    q(fur, 7, -10, 8, 8);
    q(f2, 8, -9, 6, 5);

    // Вуха (круглі)
    q(dk, 7, -13, 2, 2);
    q(dk, 12, -13, 2, 2);
    q(f2, 7, -12, 1, 1);
    q(f2, 12, -12, 1, 1);

    // Очі (маленькі)
    q(0x000000, 9, -7, 1, 1);
    q(0x000000, 12, -7, 1, 1);

    // Морда
    q(f2, 10, -5, 3, 3);

    // Ніс (великий)
    q(0x000000, 11, -3, 2, 2);
}

/// Малює кролика
#[allow(clippy::too_many_arguments)]
fn draw_rabbit(
    sx: f32,
    sy: f32,
    u: f32,
    moving: bool,
    fl: bool,
    phase: f32,
    flash: Option<Color>,
) {
    // Кольори
    let fur = 0xc9aa8a;
    let f2 = 0xffffff;
    let dk = 0x8a7a6a;

    // Анімація ходи (стрибки)
    let jump = if moving { (phase.abs() * 1.5) as i32 } else { 0 };

    // Тінь
    draw_rectangle(sx - 4.0 * u, sy + 3.0 * u, 8.0 * u, 2.0 * u, ha(0, 0.3));

    let q = |c: u32, dx: i32, dy: i32, w: i32, h: i32| {
        px(sx, sy, u, dx, dy, w, h, flash.unwrap_or(hx(c)), fl);
    };

    // Задні ноги (великі)
    q(dk, -4, 1 - jump, 3, 3);
    q(dk, 1, 1 - jump, 3, 3);

    // Передні лапки (маленькі)
    q(dk, -2, 0 - jump, 2, 2);
    q(dk, 2, 0 - jump, 2, 2);

    // Тіло (овальне)
    q(fur, -5, -5 - jump, 8, 6);
    q(f2, -4, -4 - jump, 6, 4); // біле пузо

    // Хвіст (пухнастий)
    q(f2, -6, -3 - jump, 2, 2);

    // Голова (кругла)
    q(fur, 2, -7 - jump, 5, 5);
    q(f2, 3, -6 - jump, 3, 3); // біла морда

    // Вуха (довгі)
    q(fur, 3, -12 - jump, 2, 5);
    q(fur, 5, -12 - jump, 2, 5);
    q(0xffb0b0, 3, -11 - jump, 1, 4); // рожеві всередині
    q(0xffb0b0, 5, -11 - jump, 1, 4);

    // Очі (великі)
    q(0x000000, 4, -5 - jump, 1, 1);
    q(0x000000, 6, -5 - jump, 1, 1);

    // Ніс (рожевий трикутник)
    q(0xffb0b0, 5, -3 - jump, 1, 1);
}

/// Малює лисицю
#[allow(clippy::too_many_arguments)]
fn draw_fox(
    sx: f32,
    sy: f32,
    u: f32,
    _now: f64,
    moving: bool,
    fl: bool,
    phase: f32,
    flash: Option<Color>,
) {
    // Кольори
    let fur = 0xd97a3a; // помаранчевий
    let f2 = 0xffffff; // біле
    let dk = 0x5a3a1a; // темний

    // Анімація ходи
    let lf = if moving { (phase * 2.0) as i32 } else { 0 };
    let rf = if moving { (-phase * 2.0) as i32 } else { 0 };

    // Тінь
    draw_rectangle(sx - 7.0 * u, sy + 4.0 * u, 14.0 * u, 2.5 * u, ha(0, 0.3));

    let q = |c: u32, dx: i32, dy: i32, w: i32, h: i32| {
        px(sx, sy, u, dx, dy, w, h, flash.unwrap_or(hx(c)), fl);
    };

    // Ноги
    q(dk, -6, 2 + lf, 2, 3);
    q(dk, -2, 2 + rf, 2, 3);
    q(dk, 1, 2 + rf, 2, 3);
    q(dk, 4, 2 + lf, 2, 3);

    // Хвіст (пухнастий з білим кінчиком)
    q(fur, -11, -6, 4, 3);
    q(fur, -10, -7, 3, 2);
    q(f2, -11, -7, 2, 2); // білий кінчик

    // Тіло
    q(fur, -7, -5, 11, 7);
    q(f2, -5, -4, 8, 3); // біле пузо

    // Груди
    q(f2, 3, -2, 3, 3);

    // Голова (гостра морда)
    q(fur, 5, -7, 6, 6);
    q(f2, 6, -5, 4, 3); // біла морда

    // Вуха (гострі трикутні)
    q(fur, 5, -10, 2, 3);
    q(fur, 9, -10, 2, 3);
    q(f2, 6, -9, 1, 2);
    q(f2, 9, -9, 1, 2);

    // Очі (хитрі)
    q(0x000000, 7, -5, 1, 1);
    q(0x000000, 9, -5, 1, 1);

    // Ніс (чорний)
    q(0x000000, 8, -3, 2, 1);
}

