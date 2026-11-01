//! Рисование: плитки, ресурсы, звери, персонаж (детальный), эффекты.
use macroquad::prelude::*;

use crate::game::{Fx, Mob, MobKind};
use crate::world::*;

pub fn hx(c: u32) -> Color {
    Color::from_rgba((c >> 16) as u8, (c >> 8) as u8, c as u8, 255)
}
pub fn ha(c: u32, a: f32) -> Color {
    Color::new(((c >> 16) & 255) as f32 / 255.0, ((c >> 8) & 255) as f32 / 255.0, (c & 255) as f32 / 255.0, a)
}
pub fn hash(x: i32, y: i32) -> u32 {
    (x.wrapping_mul(73856093) ^ y.wrapping_mul(19349663)) as u32
}

/// Прямоугольник в «пикселях спрайта» относительно точки (sx, sy).
pub fn rc(sx: f32, sy: f32, u: f32, ox: f32, oy: f32, w: f32, h: f32, c: Color) {
    draw_rectangle(sx + ox * u, sy + oy * u, w * u, h * u, c);
}

pub fn text(font: &Font, s: &str, x: f32, y: f32, size: f32, color: Color) {
    draw_text_ex(s, x, y, TextParams { font: Some(font), font_size: size as u16, color, ..Default::default() });
}
pub fn text_w(font: &Font, s: &str, size: f32) -> f32 {
    measure_text(s, Some(font), size as u16, 1.0).width
}
pub fn text_c(font: &Font, s: &str, cx: f32, y: f32, size: f32, color: Color) {
    text(font, s, cx - text_w(font, s, size) / 2.0, y, size, color);
}
pub fn label(font: &Font, s: &str, cx: f32, y: f32, size: f32) {
    let w = text_w(font, s, size);
    for (dx, dy) in [(-1.5, 0.0), (1.5, 0.0), (0.0, -1.5), (0.0, 1.5)] {
        text(font, s, cx - w / 2.0 + dx, y + dy, size, ha(0x000000, 0.85));
    }
    text(font, s, cx - w / 2.0, y, size, WHITE);
}

pub fn draw_tile(map: &Map, tx: i32, ty: i32, sx: f32, sy: f32, ts: f32, u: f32, now: f64, bridge: bool) {
    let h = hash(tx, ty);
    let t = map.terr_at(tx, ty);
    if t == WATER || t == FORD {
        let ford = t == FORD;
        let wv = (((now * 2.5) as i32) + tx + ty) & 1;
        draw_rectangle(sx, sy, ts + 1.0, ts + 1.0, hx(if ford { 0x7fb8c9 } else { 0x3f7fb5 }));
        rc(sx, sy, u, ((h >> 2) % 8) as f32 + wv as f32 * 2.0, ((h >> 6) % 12) as f32, 5.0, 1.0, hx(if ford { 0x9fd0de } else { 0x4f93c9 }));
        rc(sx, sy, u, 0.0, 14.0, 16.0, 2.0, hx(if ford { 0x6aa0b3 } else { 0x356f9e }));
        if ford {
            rc(sx, sy, u, 3.0, 5.0, 2.0, 2.0, hx(0xb8a88a));
            rc(sx, sy, u, 10.0, 9.0, 2.0, 2.0, hx(0xb8a88a));
        }
        if bridge {
            rc(sx, sy, u, 0.0, 0.0, 16.0, 16.0, hx(0x9a6b3a));
            for y in [3.0, 8.0, 13.0] {
                rc(sx, sy, u, 0.0, y, 16.0, 1.0, hx(0x7a5230));
            }
            rc(sx, sy, u, 0.0, 0.0, 2.0, 16.0, hx(0x5e3f20));
            rc(sx, sy, u, 14.0, 0.0, 2.0, 16.0, hx(0x5e3f20));
        }
        return;
    }
    if t == SAND {
        draw_rectangle(sx, sy, ts + 1.0, ts + 1.0, hx(if h % 2 == 0 { 0xe3d5a1 } else { 0xdccb94 }));
        if h % 7 == 0 {
            rc(sx, sy, u, ((h >> 3) % 12) as f32, ((h >> 5) % 12) as f32, 2.0, 1.0, hx(0xc9b982));
        }
        return;
    }
    let bm = biome(tx, ty);
    let (a, b) = match bm {
        2 => (0x8f9a74, 0x869068),
        1 => (0x5b9a50, 0x508c47),
        _ => (0x76b35c, 0x6aa653),
    };
    draw_rectangle(sx, sy, ts + 1.0, ts + 1.0, hx(if h % 3 != 0 { a } else { b }));
    if h % 5 == 0 {
        rc(sx, sy, u, ((h >> 3) % 12) as f32, ((h >> 5) % 12) as f32, 2.0, 3.0, hx(0x5f9a4a));
    }
    if bm == 0 && h % 11 == 0 {
        rc(sx, sy, u, ((h >> 4) % 12) as f32 + 1.0, ((h >> 6) % 12) as f32 + 1.0, 2.0, 2.0, hx(0xf7e96b));
    }
    if bm == 0 && h % 17 == 0 {
        rc(sx, sy, u, ((h >> 5) % 12) as f32 + 1.0, ((h >> 7) % 12) as f32 + 1.0, 2.0, 2.0, hx(0xf08fb0));
    }
}

pub fn draw_node(n: &Node, sx: f32, sy: f32, u: f32, now: f64) {
    let dead = n.dead_until > now;
    let sh = ha(0x000000, 0.18);
    match n.kind {
        Kind::Tree => {
            rc(sx, sy, u, 3.0, 13.0, 10.0, 2.0, sh);
            if dead {
                rc(sx, sy, u, 6.0, 10.0, 4.0, 4.0, hx(0x6b4423));
                rc(sx, sy, u, 6.0, 10.0, 4.0, 1.0, hx(0x9a6b3a));
            } else {
                rc(sx, sy, u, 7.0, 8.0, 3.0, 7.0, hx(0x6b4423));
                rc(sx, sy, u, 9.0, 8.0, 1.0, 7.0, hx(0x4d2f17));
                rc(sx, sy, u, 3.0, 7.0, 10.0, 3.0, hx(0x256427));
                rc(sx, sy, u, 2.0, 3.0, 12.0, 6.0, hx(0x2f7d32));
                rc(sx, sy, u, 4.0, 1.0, 8.0, 4.0, hx(0x3f9a43));
                rc(sx, sy, u, 5.0, 2.0, 3.0, 2.0, hx(0x62b865));
            }
        }
        Kind::Rock | Kind::Coal | Kind::CuOre | Kind::FeOre => {
            rc(sx, sy, u, 2.0, 13.0, 12.0, 2.0, sh);
            if dead {
                rc(sx, sy, u, 5.0, 12.0, 6.0, 2.0, hx(0x77777f));
            } else {
                rc(sx, sy, u, 2.0, 7.0, 12.0, 7.0, hx(0x6d6d76));
                rc(sx, sy, u, 3.0, 5.0, 9.0, 6.0, hx(0x8d8d96));
                rc(sx, sy, u, 4.0, 5.0, 4.0, 3.0, hx(0xb4b4be));
                rc(sx, sy, u, 2.0, 12.0, 12.0, 2.0, hx(0x55555c));
                if n.kind.is_ore() {
                    let (c1, c2) = match n.kind {
                        Kind::Coal => (0x1d1d22, 0x1d1d22),
                        Kind::CuOre => (0xd9823a, 0x4fb09a),
                        _ => (0xc4623a, 0x8a4a2a),
                    };
                    rc(sx, sy, u, 4.0, 6.0, 2.0, 2.0, hx(c1));
                    rc(sx, sy, u, 9.0, 5.0, 3.0, 2.0, hx(c1));
                    rc(sx, sy, u, 6.0, 10.0, 3.0, 2.0, hx(c1));
                    rc(sx, sy, u, 10.0, 10.0, 2.0, 2.0, hx(c2));
                }
            }
        }
        Kind::Bush => {
            rc(sx, sy, u, 2.0, 13.0, 12.0, 2.0, sh);
            if dead {
                rc(sx, sy, u, 5.0, 11.0, 6.0, 3.0, hx(0x3f8f3a));
            } else {
                rc(sx, sy, u, 2.0, 8.0, 12.0, 6.0, hx(0x2a6e2e));
                rc(sx, sy, u, 3.0, 5.0, 10.0, 6.0, hx(0x3f9a43));
                rc(sx, sy, u, 4.0, 5.0, 4.0, 2.0, hx(0x62b865));
                for (ox, oy) in [(5.0, 8.0), (10.0, 10.0), (8.0, 6.0)] {
                    rc(sx, sy, u, ox, oy, 2.0, 2.0, hx(0xd94f4f));
                }
            }
        }
        _ if dead => {}
        Kind::Stick => {
            rc(sx, sy, u, 3.0, 10.0, 8.0, 1.0, hx(0x8b5a2b));
            rc(sx, sy, u, 5.0, 8.0, 5.0, 1.0, hx(0x8b5a2b));
            rc(sx, sy, u, 3.0, 10.0, 3.0, 1.0, hx(0xb07a45));
        }
        Kind::Pebble => {
            rc(sx, sy, u, 4.0, 10.0, 4.0, 3.0, hx(0x9a9aa4));
            rc(sx, sy, u, 5.0, 10.0, 2.0, 1.0, hx(0xcfcfd8));
            rc(sx, sy, u, 9.0, 9.0, 3.0, 2.0, hx(0x9a9aa4));
        }
        Kind::Fiber => {
            rc(sx, sy, u, 5.0, 7.0, 1.0, 6.0, hx(0x9ad06a));
            rc(sx, sy, u, 7.0, 6.0, 1.0, 7.0, hx(0xb8e68a));
            rc(sx, sy, u, 9.0, 8.0, 1.0, 5.0, hx(0x9ad06a));
            rc(sx, sy, u, 6.0, 6.0, 1.0, 1.0, hx(0xe8d98a));
        }
    }
}

pub fn draw_fire(sx: f32, sy: f32, u: f32, now: f64) {
    let h = 3.0 + ((now * 8.0) as i64 % 3) as f32;
    rc(sx, sy, u, -5.0, 1.0, 10.0, 2.0, hx(0x6b4423));
    rc(sx, sy, u, -2.0, -h, 4.0, h + 1.0, hx(0xf08a24));
    rc(sx, sy, u, -1.0, -h + 1.0, 2.0, h - 1.0, hx(0xffd54a));
}
pub fn draw_furnace(sx: f32, sy: f32, u: f32, now: f64) {
    rc(sx, sy, u, -6.0, 5.0, 12.0, 2.0, ha(0, 0.25));
    rc(sx, sy, u, -5.0, -4.0, 10.0, 9.0, hx(0x6e6e76));
    rc(sx, sy, u, -5.0, -4.0, 10.0, 2.0, hx(0x8d8d96));
    rc(sx, sy, u, -2.0, 0.0, 4.0, 4.0, hx(0x222222));
    rc(sx, sy, u, -1.0, 2.0 - ((now * 6.0) as i64 % 2) as f32, 2.0, 2.0, hx(0xf08a24));
}
pub fn draw_house(sx: f32, sy: f32, u: f32) {
    rc(sx, sy, u, 1.0, 14.0, 14.0, 2.0, ha(0, 0.18));
    rc(sx, sy, u, 2.0, 6.0, 12.0, 9.0, hx(0xc97b4a));
    rc(sx, sy, u, 2.0, 12.0, 12.0, 3.0, hx(0xa8623a));
    rc(sx, sy, u, 1.0, 2.0, 14.0, 5.0, hx(0x8a3b2e));
    rc(sx, sy, u, 1.0, 6.0, 14.0, 1.0, hx(0x6e2d23));
    rc(sx, sy, u, 3.0, 3.0, 10.0, 1.0, hx(0xb04a3a));
    rc(sx, sy, u, 7.0, 10.0, 3.0, 5.0, hx(0x3b2a1a));
    rc(sx, sy, u, 4.0, 8.0, 2.0, 2.0, hx(0xbfe3f5));
    rc(sx, sy, u, 11.0, 8.0, 2.0, 2.0, hx(0xbfe3f5));
}

/// Зверь (sx, sy — центр на экране).
pub fn draw_mob(m: &Mob, sx: f32, sy: f32, u: f32, now: f64) {
    if m.dead > 0.0 {
        if !m.cut {
            rc(sx, sy, u, -5.0, 2.0, 10.0, 2.0, ha(0, 0.25));
            rc(sx, sy, u, -5.0, -1.0, 10.0, 4.0, if m.kind == MobKind::Wolf { hx(0x555a66) } else { hx(0xd9c7a6) });
            rc(sx, sy, u, -1.0, 0.0, 3.0, 2.0, hx(0xaa3333));
        }
        return;
    }
    let hit = now - m.flash < 0.14;
    let fl = m.fl;
    let q = |c: u32, dx: f32, dy: f32, w: f32, h: f32| {
        let col = if hit { WHITE } else { hx(c) };
        let x = if fl { sx - (dx + w) * u } else { sx + dx * u };
        draw_rectangle(x, sy + dy * u, w * u, h * u, col);
    };
    let ph = ((now / 0.12) as i64 % 2) as f32;
    if m.kind == MobKind::Rabbit {
        let hop = if m.mv { ((now * 12.5).sin().abs() * 3.0).round() as f32 } else { 0.0 };
        let ear = ((now / 0.5) as i64 % 2) as f32;
        draw_rectangle(sx - 4.0 * u, sy + 3.0 * u, 8.0 * u, 2.0 * u, ha(0, 0.25 - hop * 0.04));
        q(0xb8a583, -3.0, 1.0 - hop, 3.0, 2.0);
        q(0xcbb99a, -4.0, -3.0 - hop, 8.0, 5.0);
        q(0xefe3cc, -2.0, -hop, 5.0, 2.0);
        q(0xd9c7a6, 2.0, -5.0 - hop, 4.0, 4.0);
        q(0xd9c7a6, 3.0, -9.0 - hop + ear, 1.0, 4.0);
        q(0xe8a0a0, 4.0, -9.0 - hop, 1.0, 4.0);
        q(0x222222, 5.0, -4.0 - hop, 1.0, 1.0);
        q(0xe8a0a0, 6.0, -3.0 - hop, 1.0, 1.0);
        q(0xffffff, -5.0, -3.0 - hop, 2.0, 2.0);
        q(0xb8a583, 2.0, 1.0 - hop, 2.0, 2.0);
    } else {
        let pet = m.kind == MobKind::Pet;
        let (fur, f2, dk) = if pet { (0x9a7b55, 0xb89a72, 0x6b5238) } else { (0x5c616e, 0x7b8190, 0x3e424c) };
        let (lf, rf) = if m.mv { (ph, 1.0 - ph) } else { (0.0, 0.0) };
        let wag = if pet { ((now / 0.13) as i64 % 2) as f32 } else { 0.0 };
        let bob = if m.mv { 0.0 } else { ((now / 0.8) as i64 % 2) as f32 };
        draw_rectangle(sx - 6.0 * u, sy + 4.0 * u, 12.0 * u, 2.0 * u, ha(0, 0.28));
        q(dk, -5.0, 2.0, 2.0, 2.0 + lf);
        q(dk, -2.0, 2.0, 2.0, 2.0 + rf);
        q(dk, 1.0, 2.0, 2.0, 2.0 + rf);
        q(dk, 3.0, 2.0, 2.0, 2.0 + lf);
        q(fur, -9.0, -6.0 - wag, 3.0, 2.0);
        q(fur, -6.0, -4.0, 11.0, 6.0);
        q(f2, -5.0, -5.0, 8.0, 2.0);
        q(dk, -5.0, 1.0, 9.0, 1.0);
        q(fur, 4.0, -6.0 + bob, 5.0, 5.0);
        q(f2, 8.0, -3.0 + bob, 3.0, 2.0);
        q(0x111111, 10.0, -3.0 + bob, 1.0, 1.0);
        q(fur, 5.0, -8.0 + bob, 2.0, 2.0);
        q(dk, 7.0, -8.0 + bob, 1.0, 2.0);
        q(if pet { 0x222222 } else { 0xff4040 }, 7.0, -5.0 + bob, 1.0, 1.0);
        if pet {
            q(0xdd3333, 3.0, -2.0, 2.0, 2.0);
        }
    }
}

/// Частицы и взмах после удара.
pub fn draw_fx_particles(fx: &Fx, sx: f32, sy: f32, u: f32, now: f64) {
    let k = ((now - fx.t) / 0.26) as f32;
    if !(0.0..1.0).contains(&k) {
        return;
    }
    for i in 0..5 {
        let x = sx + ((i as f32 - 2.0) * 4.0 * k) * u;
        let y = sy - 6.0 * k * u + ((i % 2) as f32 * 3.0 * k * k) * u;
        draw_rectangle(x, y, 2.0 * u, 2.0 * u, hx(fx.col));
    }
}

pub struct Look {
    pub body: Color,
    pub face: u8,
    pub moving: bool,
    pub coat: bool,
    pub held: u8,
    pub boat: bool,
    /// (инструмент: 1 топор, 2 кирка, 3 копьё, 4 лопата; прогресс 0..1)
    pub swing: Option<(u8, f32)>,
}

fn half_ellipse(cx: f32, cy: f32, rx: f32, ry: f32, col: Color) {
    let n = 18;
    let mut prev = vec2(cx - rx, cy);
    for i in 1..=n {
        let a = std::f32::consts::PI + std::f32::consts::PI * i as f32 / n as f32;
        let p = vec2(cx + rx * a.cos(), cy + ry * a.sin());
        draw_triangle(vec2(cx, cy), prev, p, col);
        prev = p;
    }
}

/// Детальный персонаж. (x, y) — центр корпуса на экране, k = размер клетки / 48.
/// Возвращает точку над головой для имени.
pub fn person(x: f32, y: f32, k: f32, t: f64, lk: &Look) -> (f32, f32) {
    let ts = t as f32;
    let m = lk.moving;
    let s = if m { (ts * 1000.0 / 110.0).sin() } else { 0.0 };
    let bob = if m { -(ts * 1000.0 / 110.0).cos().abs() * 1.6 } else { (ts * 1000.0 / 600.0).sin() * 0.8 };
    let f = lk.face;
    let sd = f > 1;
    let sg: f32 = if f == 3 { 1.0 } else { -1.0 };
    let (fr, bk) = (f == 0, f == 1);
    let hw = if sd { 6.0 } else { 9.0 };
    let tt = bob; // смещение корпуса/головы
    let body = if lk.coat { hx(0x8a6a4a) } else { lk.body };
    let (sk, sk2, hair, pants, boot) = (hx(0xefc39b), hx(0xd19a72), hx(0x4a2f18), hx(0x34436a), hx(0x2a1f17));

    let l = |x1: f32, y1: f32, x2: f32, y2: f32, w: f32, c: Color| {
        draw_line(x + x1 * k, y + y1 * k, x + x2 * k, y + y2 * k, w * k, c);
        draw_circle(x + x1 * k, y + y1 * k, w * k / 2.0, c);
        draw_circle(x + x2 * k, y + y2 * k, w * k / 2.0, c);
    };
    let e = |cx: f32, cy: f32, rx: f32, ry: f32, c: Color| draw_ellipse(x + cx * k, y + cy * k, rx * k, ry * k, 0.0, c);
    let rr = |rx: f32, ry: f32, w: f32, h: f32, r: f32, c: Color| {
        draw_rectangle(x + (rx + r) * k, y + ry * k, (w - 2.0 * r) * k, h * k, c);
        draw_rectangle(x + rx * k, y + (ry + r) * k, w * k, (h - 2.0 * r) * k, c);
        for (cx, cy) in [(rx + r, ry + r), (rx + w - r, ry + r), (rx + r, ry + h - r), (rx + w - r, ry + h - r)] {
            draw_circle(x + cx * k, y + cy * k, r * k, c);
        }
    };

    e(0.0, 16.0, 13.0, 4.0, ha(0, 0.28));
    // ноги
    if !sd {
        let la = if s > 0.0 { s * 3.0 } else { 0.0 };
        let ra = if s < 0.0 { -s * 3.0 } else { 0.0 };
        l(-4.0, 3.0, -4.0, 12.0 - la, 6.0, pants);
        l(-4.0, 13.0 - la, -4.0, 14.0 - la, 7.0, boot);
        l(4.0, 3.0, 4.0, 12.0 - ra, 6.0, pants);
        l(4.0, 13.0 - ra, 4.0, 14.0 - ra, 7.0, boot);
    } else {
        let d = s * 7.0 * sg;
        let lf = if s > 0.0 { s * 2.0 } else { 0.0 };
        l(0.0, 3.0, -d, 12.0, 5.0, hx(0x27324f));
        l(-d + sg, 13.0, -d + sg * 3.0, 14.0, 6.0, boot);
        l(0.0, 3.0, d, 12.0 - lf, 6.0, pants);
        l(d + sg, 13.0 - lf, d + sg * 3.0, 14.0 - lf, 7.0, boot);
    }
    if lk.boat {
        e(0.0, 11.0, 19.0, 7.0, hx(0x6b4423));
        e(0.0, 9.0, 17.0, 5.0, hx(0x8a5c30));
        l(-17.0, 9.0, 17.0, 9.0, 2.0, hx(0x4d2f17));
    }
    let hand = if sd { (-sg * s * 6.0, tt + 1.0) } else { (hw + 1.5, tt + 1.0 + s * 3.0) };
    if sd {
        rr(if sg > 0.0 { -hw - 5.0 } else { hw - 1.0 }, tt - 10.0, 6.0, 13.0, 3.0, hx(0x6b4a2b));
        l(0.0, tt - 9.0, sg * s * 6.0, tt + 1.0, 4.5, body);
        l(0.0, tt - 9.0, sg * s * 6.0, tt + 1.0, 4.5, ha(0, 0.3));
    }
    // корпус
    rr(-hw, tt - 12.0, hw * 2.0, 16.0, 4.0, body);
    draw_rectangle(x - hw * k, y + (tt - 12.0) * k, hw * 0.6 * k, 16.0 * k, ha(0xffffff, 0.14));
    draw_rectangle(x + hw * 0.4 * k, y + (tt - 12.0) * k, hw * 0.6 * k, 16.0 * k, ha(0, 0.2));
    l(-hw, tt + 1.0, hw, tt + 1.0, 2.5, hx(0x4a3018));
    l(-2.0, tt - 6.0, -3.0, tt, 1.0, ha(0, 0.18));
    l(3.0, tt - 7.0, 3.0, tt, 1.0, ha(0, 0.14));
    if lk.coat {
        l(-hw, tt + 3.0, hw, tt + 3.0, 2.5, hx(0xe6e0d0));
    }
    if bk {
        rr(-6.0, tt - 10.0, 12.0, 12.0, 3.0, hx(0x6b4a2b));
        l(-6.0, tt - 9.0, 6.0, tt - 9.0, 1.5, hx(0x8a6338));
        l(-4.0, tt - 12.0, -4.0, tt - 3.0, 1.5, hx(0x4a3018));
        l(4.0, tt - 12.0, 4.0, tt - 3.0, 1.5, hx(0x4a3018));
    }
    // руки
    if !sd {
        let a = s * 3.0;
        l(-hw - 1.0, tt - 9.0, -hw - 1.5, tt + 1.0 - a, 5.0, body);
        e(-hw - 1.5, tt + 2.0 - a, 2.4, 2.4, sk);
        l(hw + 1.0, tt - 9.0, hand.0, hand.1, 5.0, body);
        e(hand.0, hand.1 + 1.5, 2.4, 2.4, sk);
    } else {
        l(0.0, tt - 9.0, hand.0, hand.1, 5.0, body);
        e(hand.0, hand.1 + 1.5, 2.4, 2.4, sk);
    }
    // голова
    l(0.0, tt - 13.0, 0.0, tt - 10.0, 4.5, sk2);
    if lk.coat {
        e(0.0, tt - 12.0, hw + 1.0, 2.4, hx(0xe6e0d0));
    }
    e(0.0, tt - 20.0, 6.8, 7.4, sk);
    e(0.0, tt - 14.8, 4.8, 1.6, ha(0, 0.08));
    if fr {
        e(-7.0, tt - 19.0, 1.3, 2.2, sk2);
        e(7.0, tt - 19.0, 1.3, 2.2, sk2);
    } else if sd {
        e(-sg * 0.5, tt - 18.5, 1.6, 2.4, sk2);
    }
    if bk {
        e(0.0, tt - 20.0, 7.1, 7.7, hair);
    } else {
        half_ellipse(x + 0.0, y + (tt - 21.0) * k, 7.2 * k, 7.4 * k, hair);
        if fr {
            e(-5.6, tt - 18.5, 1.6, 3.2, hair);
            e(5.6, tt - 18.5, 1.6, 3.2, hair);
        } else {
            e(-sg * 3.8, tt - 18.5, 3.6, 5.6, hair);
        }
    }
    if fr {
        e(-2.7, tt - 19.0, 1.15, 1.3, hx(0x222222));
        e(2.7, tt - 19.0, 1.15, 1.3, hx(0x222222));
        l(-4.3, tt - 21.3, -1.4, tt - 21.6, 1.0, hair);
        l(1.4, tt - 21.6, 4.3, tt - 21.3, 1.0, hair);
        l(0.0, tt - 18.0, -0.4, tt - 16.0, 1.0, sk2);
        l(-1.7, tt - 14.2, 1.7, tt - 14.2, 1.0, hx(0xa8644a));
    } else if sd {
        e(sg * 3.2, tt - 19.0, 1.1, 1.3, hx(0x222222));
        l(sg * 3.5, tt - 21.2, sg * 6.0, tt - 21.0, 1.0, hair);
        e(sg * 7.0, tt - 17.0, 1.5, 1.5, sk);
        l(sg * 3.5, tt - 14.2, sg * 5.5, tt - 14.2, 1.0, hx(0xa8644a));
    }
    // инструмент в руке / взмах
    let kind = lk.swing.map(|s| s.0).unwrap_or(match lk.held {
        1 => 3,
        2 => 1,
        3 => 2,
        _ => 0,
    });
    if kind > 0 {
        let ang = match lk.swing {
            Some((_, p)) => (-1.1 + p * 2.1) * if f == 2 { -1.0 } else { 1.0 },
            None => {
                if sd {
                    sg * 0.15
                } else {
                    0.1
                }
            }
        };
        let (ca, sa) = (ang.cos(), ang.sin());
        let tr = |px: f32, py: f32| vec2(x + (hand.0 + px * ca - py * sa) * k, y + (hand.1 + 1.0 + px * sa + py * ca) * k);
        let line = |a: Vec2, b: Vec2, w: f32, c: Color| {
            draw_line(a.x, a.y, b.x, b.y, w * k, c);
        };
        let quad = |x0: f32, y0: f32, w: f32, h: f32, c: Color| {
            let (a, b, cc, d) = (tr(x0, y0), tr(x0 + w, y0), tr(x0 + w, y0 + h), tr(x0, y0 + h));
            draw_triangle(a, b, cc, c);
            draw_triangle(a, cc, d, c);
        };
        match kind {
            3 => {
                line(tr(0.0, 8.0), tr(0.0, -26.0), 2.0, hx(0x8b5a2b));
                draw_triangle(tr(-2.5, -26.0), tr(0.0, -34.0), tr(2.5, -26.0), hx(0xd9d9e0));
            }
            _ => {
                line(tr(0.0, 4.0), tr(0.0, -16.0), 2.2, hx(0x8b5a2b));
                match kind {
                    1 => quad(-1.0, -17.0, 7.0, 6.0, hx(0xb8b8c0)),
                    4 => quad(-3.0, -22.0, 6.0, 7.0, hx(0x9a9aa4)),
                    _ => line(tr(-6.0, -16.0), tr(6.0, -16.0), 3.0, hx(0x8d8d96)),
                }
            }
        }
    }
    (x, y + (tt - 31.0) * k)
}

/// Картинка для мини-карты.
pub fn build_minimap(map: &Map) -> Texture2D {
    let mut img = Image::gen_image_color(W as u16, W as u16, BLACK);
    for y in 0..W as i32 {
        for x in 0..W as i32 {
            let c = match map.terr_at(x, y) {
                WATER => 0x3f7fb5,
                FORD => 0x7fb8c9,
                SAND => 0xe3d5a1,
                _ => match biome(x, y) {
                    2 => 0x8f9a74,
                    1 => 0x4f8a45,
                    _ => 0x76b35c,
                },
            };
            img.set_pixel(x as u32, y as u32, hx(c));
        }
    }
    let t = Texture2D::from_image(&img);
    t.set_filter(FilterMode::Nearest);
    t
}
