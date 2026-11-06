//! Логика игры: выживание, добыча, крафт, звери, рыбалка, лодка, мосты.
use std::collections::{HashMap, HashSet};

use macroquad::prelude::{rand, Vec2};
use shared::ClientMsg;

use crate::data::*;
use crate::save::Save;
use crate::world::*;
use crate::i18n::Texts;

pub const RABBITS: usize = 10;

pub const ANIMALS: [(MobKind, usize, f32, f32, f32, f32); 5] = [
    (MobKind::Rabbit, 10, 5.0, 30.0, 20.0, 15.0),  // 10 кроликів
    (MobKind::Wolf, 3, 8.0, 24.0, 8.0, 24.0),       // 3 вовки
    (MobKind::Deer, 5, 12.0, 20.0, 12.0, 20.0),     // 5 оленів
    (MobKind::Bear, 2, 5.0, 10.0, 5.0, 10.0),       // 2 ведмеді (рідкісні)
    (MobKind::Fox, 4, 15.0, 20.0, 15.0, 20.0),      // 4 лисиці
];

pub fn epoch() -> f64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0)
}
fn rnd() -> f32 {
    rand::gen_range(0.0f32, 1.0)
}

#[derive(Clone, Copy, PartialEq)]
pub enum MobKind {
    Rabbit,
    Wolf,
    Pet,
    Deer,      // Олень - мирний, дає більше м'яса
    Bear,      // Ведмідь - агресивний хижак
    Fox,       // Лисиця - дропає хутро
}

#[derive(Clone)]
pub struct Mob {
    pub kind: MobKind,
    pub x: f32,
    pub y: f32,
    pub hp: f32,
    pub dead: f64,
    pub cut: bool,
    pub calm: f64,
    pub trust: u8,
    pub hit: f64,
    pub vx: f32,
    pub vy: f32,
    pub t: f32,
    pub fl: bool,
    pub mv: bool,
    pub flash: f64,
    pub px: f32,
    pub py: f32,
}

impl Mob {
    pub fn new(kind: MobKind, x: f32, y: f32) -> Mob {
        let hp = match kind {
            MobKind::Rabbit => 2.0,
            MobKind::Wolf => 5.0,
            MobKind::Pet => 9.0,
            MobKind::Deer => 4.0,
            MobKind::Bear => 12.0,
            MobKind::Fox => 3.0,
        };
        Mob { kind, x, y, hp, dead: 0.0, cut: false, calm: 0.0, trust: 0, hit: 0.0, vx: 0.0, vy: 0.0, t: 0.0, fl: false, mv: false, flash: 0.0, px: x, py: y }
    }
}

/// Эффект удара: тип инструмента для анимации и цвет осколков.
pub struct Fx {
    pub x: f32,
    pub y: f32,
    pub t: f64,
    /// 0 нет, 1 топор, 2 кирка, 3 копьё, 4 лопата
    pub tool: u8,
    pub col: u32,
}

pub const FISHES: [(&str, u32, f32, f32, f32); 4] =
    [("Карась", 1, 1.0, 0.3, 1.0), ("Окунь", 1, 1.4, 0.4, 1.5), ("Форель", 2, 1.8, 1.0, 3.0), ("Щука", 3, 2.4, 2.0, 8.0)];

pub struct Fishing {
    /// 0 нет, 1 ждём поклёвку, 2 клюёт, 3 вываживание
    pub ph: u8,
    pub t: f64,
    pub bite: f64,
    pub armed: bool,
    pub p: f32,
    pub tn: f32,
    pub kind: usize,
    pub wx: f32,
    pub wy: f32,
}

pub struct Input {
    pub dir: Vec2,
    pub hold: bool,
}

pub struct Game {
    pub map: Map,
    pub inv: [u32; NI],
    pub tools: [i32; NT],
    pub gold: u32,
    pub x: f32,
    pub y: f32,
    pub face: u8,
    pub moving: bool,
    pub hunger: f32,
    pub thirst: f32,
    pub energy: f32,
    pub hp: f32,
    pub fires: Vec<(f32, f32, f64)>,
    pub furn: Vec<(f32, f32)>,
    pub bridges: HashSet<(i32, i32)>,
    pub in_boat: bool,
    pub houses: u32,
    pub made: u32,
    pub got: u32,
    pub caught: u32,
    pub biggest: f32,
    pub mobs: Vec<Mob>,
    pub has_pet: bool,
    pub fish: Fishing,
    pub dug: HashMap<(i32, i32), (u8, f64)>,
    pub msg: String,
    pub msg_until: f64,
    pub last_hit: f64,
    pub fx: Option<Fx>,
    /// Речі, що випали після смерті (x, y, інвентар, інструменти)
    pub death_drop: Option<(f32, f32, [u32; NI], [i32; NT])>,
    /// Для тестов: зафиксировать время суток.
    pub night_override: Option<f32>,
}

impl Game {
    pub fn new() -> Game {
        let mut mobs = Vec::new();
        // Створюємо всіх тварин
        for (kind, count, x_min, x_range, y_min, y_range) in ANIMALS {
            for _ in 0..count {
                mobs.push(Mob::new(kind, x_min + rnd() * x_range, y_min + rnd() * y_range));
            }
        }
        Game {
            map: Map::new(),
            inv: [0; NI],
            tools: [0; NT],
            gold: 20,
            x: 20.5,
            y: 20.5,
            face: 0,
            moving: false,
            hunger: 100.0,
            thirst: 100.0,
            energy: 100.0,
            hp: 100.0,
            fires: vec![],
            furn: vec![],
            bridges: HashSet::new(),
            in_boat: false,
            houses: 0,
            made: 0,
            got: 0,
            caught: 0,
            biggest: 0.0,
            mobs,
            has_pet: false,
            fish: Fishing { ph: 0, t: 0.0, bite: 0.0, armed: true, p: 0.0, tn: 0.0, kind: 0, wx: 0.0, wy: 0.0 },
            dug: HashMap::new(),
            msg: String::new(),
            msg_until: 0.0,
            last_hit: 0.0,
            fx: None,
            death_drop: None,
            night_override: std::env::var("PV_NIGHT").ok().and_then(|v| v.parse().ok()),
        }
    }

    pub fn say(&mut self, t: &str, now: f64) {
        self.msg = t.to_string();
        self.msg_until = now + 2.5;
    }

    pub fn night(&self) -> f32 {
        if let Some(v) = self.night_override {
            return v;
        }
        let p = (epoch() % 240.0) / 240.0;
        ((0.5 - 0.5 * (6.2832 * p).cos() - 0.55).max(0.0) * 1.45) as f32
    }
    pub fn rain(&self) -> bool {
        ((epoch() * 1000.0 / 60000.0) as u64) % 5 == 3
    }
    pub fn score(&self) -> u32 {
        self.got + self.made * 20 + self.houses * 40
    }
    pub fn tool(&self, t: Tl) -> bool {
        self.tools[t as usize] > 0
    }
    pub fn held(&self) -> u8 {
        if self.tool(Tl::Spear) {
            1
        } else if self.tool(Tl::Axe) || self.tool(Tl::Axe2) || self.tool(Tl::Axe3) {
            2
        } else if self.tool(Tl::Pick) || self.tool(Tl::Pick2) || self.tool(Tl::Pick3) {
            3
        } else {
            0
        }
    }
    fn best_tool(&self, pick: bool) -> Option<(Tl, i32)> {
        let fam = if pick { [(Tl::Pick3, 3), (Tl::Pick2, 2), (Tl::Pick, 1)] } else { [(Tl::Axe3, 3), (Tl::Axe2, 2), (Tl::Axe, 1)] };
        fam.into_iter().find(|(t, _)| self.tool(*t))
    }

    pub fn terr_here(&self) -> u8 {
        self.map.terr_at(self.x.floor() as i32, self.y.floor() as i32)
    }

    pub fn blocked(&self, x: f32, y: f32, player: bool, now: f64) -> bool {
        let w = W as f32;
        if x < 0.3 || y < 0.3 || x > w - 0.3 || y > w - 0.3 {
            return true;
        }
        let (tx, ty) = (x.floor() as i32, y.floor() as i32);
        let tt = self.map.terr_at(tx, ty);
        if player && self.in_boat {
            if tt == GRASS || tt == SAND {
                return true;
            }
        } else if tt == WATER && !self.bridges.contains(&(tx, ty)) {
            return true;
        }
        matches!(self.map.node_at(tx, ty), Some(n) if n.kind.blocks() && n.dead_until <= now)
    }

    pub fn face_to(&mut self, tx: f32, ty: f32) {
        let (dx, dy) = (tx - self.x, ty - self.y);
        self.face = if dx.abs() > dy.abs() { if dx > 0.0 { 3 } else { 2 } } else if dy > 0.0 { 0 } else { 1 };
    }

    pub fn near_fire(&self, now: f64) -> bool {
        self.fires.iter().any(|f| f.2 > now && (f.0 - self.x).powi(2) + (f.1 - self.y).powi(2) < 6.0)
    }
    pub fn near_furnace(&self) -> bool {
        self.furn.iter().any(|f| (f.0 - self.x).powi(2) + (f.1 - self.y).powi(2) < 9.0)
    }
    pub fn near_water(&self) -> bool {
        let (x0, y0) = (self.x.floor() as i32, self.y.floor() as i32);
        (-1..=1).any(|a| (-1..=1).any(|b| self.map.terr_at(x0 + a, y0 + b) == WATER))
    }
    fn water_tile(&self, r: i32) -> Option<(f32, f32)> {
        let (x0, y0) = (self.x.floor() as i32, self.y.floor() as i32);
        let mut best = None;
        let mut bd = f32::MAX;
        for a in -r..=r {
            for b in -r..=r {
                let (x, y) = (x0 + a, y0 + b);
                if self.map.terr_at(x, y) != WATER {
                    continue;
                }
                let d = (x as f32 + 0.5 - self.x).powi(2) + (y as f32 + 0.5 - self.y).powi(2);
                if d < bd && d <= (r * r) as f32 {
                    bd = d;
                    best = Some((x as f32 + 0.5, y as f32 + 0.5));
                }
            }
        }
        best
    }

    // ---------- крафт ----------
    /// Рецепты, доступные сейчас, по важности для текущей ситуации.

    // ---------- действия ----------


    pub fn dugs(&mut self, tx: i32, ty: i32, now: f64) -> u8 {
        let e = self.dug.entry((tx, ty)).or_insert((0, now));
        if now - e.1 > 300.0 {
            *e = (0, now);
        }
        let n = e.0;
        if n < 3 {
            e.0 += 1;
        }
        n
    }
    pub fn use_tool(&mut self, t: Tl, now: f64) {
        let d = &mut self.tools[t as usize];
        *d -= 1;
        if *d <= 0 {
            *d = 0;
            self.say(&format!("{} сломан(а)", TL_NAMES[t as usize]), now);
        }
    }




    /// Основное действие (кнопка «Добыть»).
    fn act(&mut self, now: f64, out: &mut Vec<ClientMsg>, texts: &Texts) {
        if self.fish.ph != 0 || now - self.last_hit < 0.2 || self.energy < 5.0 {
            return;
        }
        // Підбір речей після смерті
        if let Some((dx, dy, dinv, dtools)) = self.death_drop {
            if (dx - self.x).abs() < 1.5 && (dy - self.y).abs() < 1.5 {
                for i in 0..NI {
                    self.inv[i] += dinv[i];
                }
                for i in 0..NT {
                    if dtools[i] > 0 {
                        self.tools[i] = dtools[i];
                    }
                }
                self.death_drop = None;
                self.say(texts.picked_up_items, now);
                return;
            }
        }
        // 1) звери и туши
        let mut m_best = None;
        let mut md = 2.6f32;
        for (i, m) in self.mobs.iter().enumerate() {
            if m.kind == MobKind::Pet {
                continue;
            }
            let d = (m.x - self.x).powi(2) + (m.y - self.y).powi(2) + if m.dead > 0.0 { 1.5 } else { 0.0 };
            if d < md {
                md = d;
                m_best = Some(i);
            }
        }
        if let Some(i) = m_best {
            let (mx, my, kind, dead, cut) = (self.mobs[i].x, self.mobs[i].y, self.mobs[i].kind, self.mobs[i].dead, self.mobs[i].cut);
            if dead > 0.0 {
                if cut {
                    return;
                }
                if !self.tool(Tl::Knife) {
                    self.say(texts.need_knife, now);
                    self.last_hit = now - 0.0;
                    return;
                }
                self.last_hit = now;
                self.energy -= 1.0;
                self.use_tool(Tl::Knife, now);
                let m = &mut self.mobs[i];
                m.cut = true;

                // Різні тварини дають різні ресурси
                let (meat, hide, decay_time) = match kind {
                    MobKind::Wolf => (3, 2, 55.0),
                    MobKind::Bear => (6, 3, 40.0),    // Ведмідь дає багато м'яса
                    MobKind::Deer => (5, 2, 60.0),    // Олень дає багато м'яса
                    MobKind::Fox => (1, 3, 70.0),     // Лисиця дає багато хутра
                    MobKind::Rabbit => (2, 1, 80.0),
                    MobKind::Pet => (0, 0, 80.0),     // Питомця не можна різати
                };

                m.dead = now - decay_time;
                self.inv[It::Meat as usize] += meat;
                self.inv[It::Hide as usize] += hide;
                self.got += 3;
                self.say(texts.corpse_butchered, now);
                return;
            }
            // Всі тварини крім кроликів можуть бути атаковані без списа
            if kind == MobKind::Rabbit && !self.tool(Tl::Spear) {
                self.say(texts.need_spear, now);
                self.last_hit = now;
                return;
            }
            // Для ведмедя та оленя потрібне спис для ефективного полювання
            if (kind == MobKind::Bear || kind == MobKind::Deer) && !self.tool(Tl::Spear) {
                self.say(texts.need_spear, now);
                self.last_hit = now;
                return;
            }
            self.face_to(mx, my);
            self.last_hit = now;
            self.energy -= 1.5;
            let sp = self.tool(Tl::Spear);
            let dmg = if sp { 2.0 } else { 1.0 };
            if sp {
                self.use_tool(Tl::Spear, now);
            }
            let m = &mut self.mobs[i];
            m.hp -= dmg;
            m.flash = now;
            self.fx = Some(Fx { x: mx, y: my, t: now, tool: if sp { 3 } else { 0 }, col: 0xc0392b });
            if self.mobs[i].hp <= 0.0 {
                self.mobs[i].dead = now;
                self.got += 1;
                self.say(texts.butcher_corpse, now);
            }
            return;
        }
        // 2) ресурсы
        let mut best = None;
        let mut bd = 2.25f32;
        for (i, n) in self.map.nodes.iter().enumerate() {
            if n.dead_until > now {
                continue;
            }
            let d = (n.x as f32 + 0.5 - self.x).powi(2) + (n.y as f32 + 0.5 - self.y).powi(2);
            if d < bd {
                bd = d;
                best = Some(i);
            }
        }
        let Some(i) = best else {
            // 3) вода: рыбалка или питьё
            if self.tool(Tl::Rod) && self.inv[It::Worm as usize] > 0 && self.fish.armed {
                if let Some((wx, wy)) = self.water_tile(4) {
                    self.cast(wx, wy, now, texts);
                    return;
                }
            }
            if self.near_water() {
                self.drink(now, texts);
            }
            return;
        };
        let (kind, nx, ny) = (self.map.nodes[i].kind, self.map.nodes[i].x, self.map.nodes[i].y);
        let mine = matches!(kind, Kind::Tree | Kind::Rock) || kind.is_ore();
        let bt = if mine { self.best_tool(kind != Kind::Tree) } else { None };
        let (tool, tier) = bt.map(|(t, r)| (Some(t), r)).unwrap_or((None, 0));
        if mine && (tool.is_none() || (kind == Kind::FeOre && tier < 2)) {
            let t = if kind == Kind::FeOre {
                "Железо добывают медной киркой или лучше"
            } else if kind == Kind::Tree {
                "Нужен топор: палка, камешки, нить"
            } else {
                "Нужна кирка: палка, камешки, нить"
            };
            self.say(t, now);
            self.last_hit = now;
            return;
        }
        self.face_to(nx as f32 + 0.5, ny as f32 + 0.5);
        self.last_hit = now;
        self.energy -= 1.5;
        let col = match kind {
            Kind::Tree => 0x2f7d32,
            Kind::Rock => 0x9a9aa4,
            Kind::Bush => 0xd94f4f,
            Kind::Stick => 0x8b5a2b,
            Kind::Pebble => 0xcfcfd8,
            Kind::Fiber => 0x9ad06a,
            Kind::Coal => 0x222222,
            Kind::CuOre => 0xc9803a,
            Kind::FeOre => 0xb5613a,
            Kind::BerryBush => 0xff3366,
            Kind::Mushroom => 0xcc6633,
            Kind::Herb => 0x66cc66,
            Kind::AgOre => 0xd0d0d8,
            Kind::AuOre => 0xffcc33,
        };
        let ft = match (mine, kind) {
            (true, Kind::Tree) => 1,
            (true, _) => 2,
            _ => 0,
        };
        self.fx = Some(Fx { x: nx as f32 + 0.5, y: ny as f32 + 0.5, t: now, tool: ft, col });
        if let Some(t) = tool {
            self.use_tool(t, now);
        }
        let n = &mut self.map.nodes[i];
        if n.hp == 0 {
            n.hp = Map::max_hp(n.kind);
        }
        n.hp -= 1;
        if n.hp > 0 {
            return;
        }
        n.dead_until = now + RESPAWN_SECS;
        let g: u32 = if matches!(kind, Kind::Tree | Kind::Rock) {
            1 + tier as u32
        } else if kind.is_ore() {
            1 + (tier as u32 + 1) / 2
        } else if kind == Kind::Bush && self.tool(Tl::Basket) {
            self.use_tool(Tl::Basket, now);
            2
        } else {
            1
        };
        let item = match kind {
            Kind::Tree => It::Wood,
            Kind::Rock => It::Stone,
            Kind::Bush => It::Food,
            Kind::Stick => It::Stick,
            Kind::Pebble => It::Pebble,
            Kind::Fiber => It::Fiber,
            Kind::Coal => It::Coal,
            Kind::CuOre => It::CuOre,
            Kind::FeOre => It::FeOre,
            Kind::BerryBush => It::Berry,
            Kind::Mushroom => It::Mushroom,
            Kind::Herb => It::Herb,
            Kind::AgOre => It::AgOre,
            Kind::AuOre => It::AuOre,
        };
        self.inv[item as usize] += g;
        self.got += g;
        out.push(ClientMsg::Dead { idx: (ny as u32) * W as u32 + nx as u32 });
    }

    // ---------- рыбалка ----------

    // ---------- сохранение ----------
    /// `posted` — предметы, выставленные на рынок: при сохранении возвращаются в инвентарь.
    pub fn to_save(&self, now: f64, name: &str, posted: &[(u8, u32)]) -> Save {
        let mut inv = self.inv.to_vec();
        for (it, q) in posted {
            if let Some(v) = inv.get_mut(*it as usize) {
                *v += q;
            }
        }
        Save {
            name: name.to_string(),
            inv,
            tools: self.tools.to_vec(),
            gold: self.gold,
            x: self.x,
            y: self.y,
            hunger: self.hunger,
            thirst: self.thirst,
            energy: self.energy,
            hp: self.hp,
            fires: self.fires.iter().filter(|f| f.2 > now).map(|f| (f.0, f.1, f.2 - now)).collect(),
            furn: self.furn.clone(),
            bridges: self.bridges.iter().copied().collect(),
            houses: self.houses,
            made: self.made,
            got: self.got,
            caught: self.caught,
            biggest: self.biggest,
            has_pet: self.has_pet,
            in_boat: self.in_boat,
        }
    }

    pub fn apply_save(&mut self, s: &Save, now: f64) {
        for (i, v) in s.inv.iter().enumerate().take(NI) {
            self.inv[i] = *v;
        }
        for (i, v) in s.tools.iter().enumerate().take(NT) {
            self.tools[i] = *v;
        }
        self.gold = s.gold;
        let ok = |v: f32| v.is_finite();
        if ok(s.x) && ok(s.y) {
            self.x = s.x.clamp(1.0, W as f32 - 1.0);
            self.y = s.y.clamp(1.0, W as f32 - 1.0);
        }
        self.hunger = if ok(s.hunger) { s.hunger.clamp(1.0, 100.0) } else { 100.0 };
        self.thirst = if ok(s.thirst) { s.thirst.clamp(1.0, 100.0) } else { 100.0 };
        self.energy = if ok(s.energy) { s.energy.clamp(0.0, 100.0) } else { 100.0 };
        self.hp = if ok(s.hp) { s.hp.clamp(10.0, 100.0) } else { 100.0 };
        self.fires = s.fires.iter().map(|f| (f.0, f.1, now + f.2)).collect();
        self.furn = s.furn.clone();
        self.bridges = s.bridges.iter().copied().collect();
        self.houses = s.houses;
        self.made = s.made;
        self.got = s.got;
        self.caught = s.caught;
        self.biggest = s.biggest;
        self.in_boat = s.in_boat && self.map.terr_at(self.x.floor() as i32, self.y.floor() as i32) == WATER;
        self.has_pet = s.has_pet;
        if self.has_pet {
            self.mobs.push(Mob::new(MobKind::Pet, self.x + 1.0, self.y));
        }
        if self.blocked(self.x, self.y, true, now) {
            self.x = 20.5;
            self.y = 20.5;
            self.in_boat = false;
        }
    }

    // ---------- смерть ----------
    fn die(&mut self, now: f64, texts: &Texts) {
        // Зберігаємо речі на місці смерті
        self.death_drop = Some((self.x, self.y, self.inv, self.tools));
        self.inv = [0; NI];
        self.tools = [0; NT];
        self.hunger = 60.0;
        self.thirst = 60.0;
        self.energy = 60.0;
        self.hp = 60.0;
        self.x = 20.5;
        self.y = 20.5;
        self.in_boat = false;
        self.fish.ph = 0;
        self.say(texts.you_died_message, now);
    }

    // ---------- главний тік ----------
    pub fn tick(&mut self, dt: f32, now: f64, inp: &Input, out: &mut Vec<ClientMsg>, texts: &Texts) {
        // виживання
        self.hunger = (self.hunger - 0.35 * dt).max(0.0);
        self.thirst = (self.thirst - 0.28 * dt).max(0.0);
        let rain = self.rain();
        // Енергія: менша витрата при русі, більше відновлення
        let de = if self.moving { -(if rain { 0.6 } else { 0.25 }) } else { 2.5 };
        self.energy = (self.energy + de * dt).clamp(0.0, 100.0);
        if self.hunger <= 0.0 || self.thirst <= 0.0 {
            self.hp -= dt;
        } else if self.hunger > 50.0 && self.thirst > 40.0 && self.hp < 100.0 {
            self.hp = (self.hp + 0.3 * dt).min(100.0);
        }
        if self.hp <= 0.0 {
            self.die(now, texts);
        }
        // движение
        let dir = inp.dir;
        self.moving = dir != Vec2::ZERO;
        if self.moving {
            if dir.x.abs() > dir.y.abs() {
                self.face = if dir.x > 0.0 { 3 } else { 2 };
            } else {
                self.face = if dir.y > 0.0 { 0 } else { 1 };
            }
            let sp = (if self.energy < 10.0 { 2.0 } else { 3.5 }) * if self.terr_here() == FORD { 0.6 } else { 1.0 };
            let nx = self.x + dir.x * sp * dt;
            let ny = self.y + dir.y * sp * dt;
            if !self.blocked(nx, self.y, true, now) {
                self.x = nx;
            }
            if !self.blocked(self.x, ny, true, now) {
                self.y = ny;
            }
        }
        self.fishing_tick(dt, now, inp.hold, texts);
        if inp.hold {
            self.act(now, out, texts);
        }
        self.fires.retain(|f| f.2 > now);
        self.mobs_tick(dt, now, texts);
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    fn idx(name: &str) -> usize {
        RECIPES.iter().position(|r| r.name == name).unwrap_or_else(|| panic!("нет рецепта {name}"))
    }

    /// Игра, где живым остался один ресурс нужного вида, а игрок стоит рядом.
    fn setup(kind: Kind) -> (Game, usize) {
        let mut g = Game::new();
        g.mobs.clear();
        let i = g.map.nodes.iter().position(|n| n.kind == kind).expect("нет такого ресурса на карте");
        for (j, n) in g.map.nodes.iter_mut().enumerate() {
            if j != i {
                n.dead_until = 1e9;
            }
        }
        g.x = g.map.nodes[i].x as f32 + 0.5;
        g.y = g.map.nodes[i].y as f32 + 1.3;
        (g, i)
    }
    fn hit(g: &mut Game, t: &mut f64, times: usize, out: &mut Vec<ClientMsg>) {
        for _ in 0..times {
            *t += 0.4;
            g.energy = 100.0;
            g.tick(0.016, *t, &Input { dir: Vec2::ZERO, hold: true }, out, &crate::i18n::get_texts(&crate::settings::Language::UA));
            g.tick(0.016, *t + 0.1, &Input { dir: Vec2::ZERO, hold: false }, out, &crate::i18n::get_texts(&crate::settings::Language::UA));
        }
    }
    fn near_water_land(g: &Game) -> (f32, f32) {
        for y in 1..W as i32 - 1 {
            for x in 1..W as i32 - 1 {
                let t = g.map.terr_at(x, y);
                if (t == GRASS || t == SAND) && g.map.node_at(x, y).is_none() && g.map.terr_at(x + 1, y) == WATER {
                    return (x as f32 + 0.5, y as f32 + 0.5);
                }
            }
        }
        panic!("нет берега");
    }

    #[test]
    fn save_roundtrip_keeps_progress() {
        let mut g = Game::new();
        g.mobs.clear();
        g.inv[It::Wood as usize] = 7;
        g.inv[It::Fish as usize] = 2;
        g.tools[Tl::Pick2 as usize] = 55;
        g.gold = 99;
        g.hunger = 42.0;
        g.houses = 3;
        g.fires.push((21.0, 21.0, 100.0));
        g.bridges.insert((5, 6));
        g.has_pet = true;
        let sv = g.to_save(10.0, "Тест", &[(It::Stone as u8, 4)]);
        assert_eq!(sv.inv[It::Stone as usize], 4, "выставленное на рынок возвращается в сохранение");
        let json = serde_json::to_string(&sv).unwrap();
        let back: Save = serde_json::from_str(&json).unwrap();
        let mut g2 = Game::new();
        g2.apply_save(&back, 500.0);
        assert_eq!(g2.inv[It::Wood as usize], 7);
        assert_eq!(g2.tools[Tl::Pick2 as usize], 55);
        assert_eq!(g2.gold, 99);
        assert_eq!(g2.houses, 3);
        assert!((g2.hunger - 42.0).abs() < 0.01);
        assert!(g2.near_fire(500.0) || (g2.x - 21.0).abs() > 2.0, "костёр сохранил оставшееся время");
        assert!((g2.fires[0].2 - 590.0).abs() < 0.01, "осталось 90 секунд горения");
        assert!(g2.bridges.contains(&(5, 6)));
        assert!(g2.mobs.iter().any(|m| m.kind == MobKind::Pet));
    }

    #[test]
    fn map_has_everything() {
        let g = Game::new();
        for k in [Kind::Tree, Kind::Rock, Kind::Bush, Kind::Stick, Kind::Pebble, Kind::Fiber, Kind::Coal, Kind::CuOre, Kind::FeOre] {
            assert!(g.map.nodes.iter().any(|n| n.kind == k), "на карте нет ресурса");
        }
        assert!(g.map.terr.iter().any(|&t| t == WATER));
        assert!(g.map.terr.iter().any(|&t| t == FORD));
        assert!(g.map.terr.iter().any(|&t| t == SAND));
        assert_eq!(g.mobs.len(), ANIMALS.iter().map(|a| a.1).sum::<usize>());
    }

    #[test]
    fn tree_needs_axe_then_gives_wood_and_reports_depletion() {
        let (mut g, _) = setup(Kind::Tree);
        let (mut t, mut out) = (10.0, vec![]);
        hit(&mut g, &mut t, 5, &mut out);
        assert_eq!(g.inv[It::Wood as usize], 0, "без топора дерево рубить нельзя");
        assert!(g.msg.contains("топор"));
        g.tools[Tl::Axe as usize] = TL_DUR[Tl::Axe as usize];
        hit(&mut g, &mut t, 3, &mut out);
        assert_eq!(g.inv[It::Wood as usize], 2);
        assert_eq!(g.tools[Tl::Axe as usize], 40 - 3, "топор должен изнашиваться");
        assert!(out.iter().any(|m| matches!(m, ClientMsg::Dead { .. })), "сервер должен узнать о срубленном дереве");
    }

    #[test]
    fn iron_needs_copper_pick() {
        let (mut g, i) = setup(Kind::Rock);
        g.map.nodes[i].kind = Kind::FeOre;
        g.map.nodes[i].hp = 4;
        let (mut t, mut out) = (10.0, vec![]);
        g.tools[Tl::Pick as usize] = 40;
        hit(&mut g, &mut t, 6, &mut out);
        assert_eq!(g.inv[It::FeOre as usize], 0);
        assert!(g.msg.contains("медной"));
        g.tools[Tl::Pick2 as usize] = 80;
        hit(&mut g, &mut t, 4, &mut out);
        assert!(g.inv[It::FeOre as usize] >= 2, "медная кирка берёт железо");
        assert_eq!(g.tools[Tl::Pick as usize], 40, "хуже кирка не должна тратиться");
    }

    #[test]
    fn bush_by_hand_and_pickups() {
        let (mut g, _) = setup(Kind::Bush);
        let (mut t, mut out) = (10.0, vec![]);
        hit(&mut g, &mut t, 1, &mut out);
        assert_eq!(g.inv[It::Food as usize], 1);
        let (mut g, _) = setup(Kind::Stick);
        hit(&mut g, &mut t, 1, &mut out);
        assert_eq!(g.inv[It::Stick as usize], 1);
    }

    #[test]
    fn crafting_chain_to_iron_tools() {
        let mut g = Game::new();
        g.mobs.clear();
        let now = 5.0;
        g.inv[It::Stick as usize] = 20;
        g.inv[It::Pebble as usize] = 20;
        g.inv[It::Fiber as usize] = 30;
        g.inv[It::Stone as usize] = 10;
        g.inv[It::CuOre as usize] = 6;
        g.inv[It::FeOre as usize] = 8;
        g.inv[It::Coal as usize] = 8;
        g.craft(idx("Каменная кирка"), now);
        assert!(g.tool(Tl::Pick));
        let n = g.inv[It::Pebble as usize];
        g.craft(idx("Каменная кирка"), now);
        assert_eq!(g.inv[It::Pebble as usize], n, "второй такой же инструмент не делается");
        // плавка без печи нельзя
        g.craft(idx("Выплавить медь"), now);
        assert_eq!(g.inv[It::Cu as usize], 0);
        g.craft(idx("Печь"), now);
        assert!(g.near_furnace());
        for _ in 0..3 {
            g.craft(idx("Выплавить медь"), now);
        }
        assert_eq!(g.inv[It::Cu as usize], 3);
        g.craft(idx("Медный топор"), now);
        assert!(g.tool(Tl::Axe2));
        for _ in 0..4 {
            g.craft(idx("Выплавить железо"), now);
        }
        assert_eq!(g.inv[It::Fe as usize], 4);
    }

    #[test]
    fn fire_makes_roast_and_quick_list_prefers_it() {
        let mut g = Game::new();
        g.mobs.clear();
        let now = 5.0;
        g.inv[It::Wood as usize] = 2;
        g.inv[It::Stick as usize] = 3;
        g.inv[It::Meat as usize] = 1;
        g.craft(idx("Жаркое из мяса"), now);
        assert_eq!(g.inv[It::Roast as usize], 0, "без костра не жарится");
        g.craft(idx("Костёр"), now);
        assert!(g.near_fire(now));
        assert!(!g.near_fire(now + 241.0), "костёр сгорает");
        g.craft(idx("Жаркое из мяса"), now);
        assert_eq!(g.inv[It::Roast as usize], 1);
        g.hunger = 20.0;
        g.eat(now, &crate::i18n::get_texts(&crate::settings::Language::UA));
        assert_eq!(g.hunger, 90.0);
    }

    #[test]
    fn survival_and_death() {
        let mut g = Game::new();
        g.mobs.clear();
        let mut out = vec![];
        let h0 = g.hunger;
        g.tick(1.0, 1.0, &Input { dir: Vec2::ZERO, hold: false }, &mut out, &crate::i18n::get_texts(&crate::settings::Language::UA));
        assert!(g.hunger < h0 && g.thirst < 100.0);
        g.inv[It::Wood as usize] = 5;
        g.hp = 0.5;
        g.hunger = 0.0;
        g.tick(1.0, 2.0, &Input { dir: Vec2::ZERO, hold: false }, &mut out, &crate::i18n::get_texts(&crate::settings::Language::UA));
        assert_eq!(g.inv[It::Wood as usize], 0, "после смерти вещи теряются");
        assert_eq!(g.hp, 60.0);
    }

    #[test]
    fn drinking_and_water_blocks() {
        let mut g = Game::new();
        g.mobs.clear();
        let (x, y) = near_water_land(&g);
        g.x = x;
        g.y = y;
        g.thirst = 10.0;
        g.drink(1.0, &crate::i18n::get_texts(&crate::settings::Language::UA));
        assert!(g.thirst > 20.0);
        assert!(g.blocked(x + 1.0, y, true, 1.0), "в воду пешком нельзя");
        g.bridges.insert(((x + 1.0).floor() as i32, y.floor() as i32));
        assert!(!g.blocked(x + 1.0, y, true, 1.0), "по мосту можно");
    }

    #[test]
    fn boat_swims_and_lands() {
        let mut g = Game::new();
        g.mobs.clear();
        let (x, y) = near_water_land(&g);
        g.x = x;
        g.y = y;
        g.boat_toggle(1.0, &crate::i18n::get_texts(&crate::settings::Language::UA));
        assert!(!g.in_boat, "без лодки не поплывёшь");
        g.tools[Tl::Boat as usize] = 1;
        g.boat_toggle(1.0, &crate::i18n::get_texts(&crate::settings::Language::UA));
        assert!(g.in_boat);
        assert_eq!(g.map.terr_at(g.x.floor() as i32, g.y.floor() as i32), WATER);
        assert!(g.blocked(x - 1.0, y, true, 1.0), "в лодке на сушу нельзя");
        g.boat_toggle(2.0, &crate::i18n::get_texts(&crate::settings::Language::UA));
        assert!(!g.in_boat);
    }

    #[test]
    fn digging_worms_with_shovel() {
        let mut g = Game::new();
        g.mobs.clear();
        let spot = (0..W as i32)
            .flat_map(|y| (0..W as i32).map(move |x| (x, y)))
            .find(|&(x, y)| g.map.terr_at(x, y) == GRASS && biome(x, y) != 2)
            .unwrap();
        g.x = spot.0 as f32 + 0.5;
        g.y = spot.1 as f32 + 0.5;
        g.dig(1.0, &crate::i18n::get_texts(&crate::settings::Language::UA));
        assert!(g.msg.contains("лопата"));
        g.tools[Tl::Shovel as usize] = 30;
        let mut t = 2.0;
        for _ in 0..3 {
            g.dig(t, &crate::i18n::get_texts(&crate::settings::Language::UA));
            t += 1.0;
        }
        g.dig(t, &crate::i18n::get_texts(&crate::settings::Language::UA));
        assert!(g.msg.contains("перекопано"), "одну точку можно копать 3 раза");
        assert_eq!(g.tools[Tl::Shovel as usize], 27);
    }

    #[test]
    fn fishing_full_cycle() {
        let mut catches = 0;
        for attempt in 0..40 {
            let mut g = Game::new();
            g.mobs.clear();
            for n in g.map.nodes.iter_mut() {
                n.dead_until = 1e9;
            }
            let (x, y) = near_water_land(&g);
            g.x = x;
            g.y = y;
            g.tools[Tl::Rod as usize] = 25;
            g.inv[It::Worm as usize] = 1;
            let mut out = vec![];
            let mut t = 10.0 + attempt as f64;
            let idle = Input { dir: Vec2::ZERO, hold: false };
            let press = Input { dir: Vec2::ZERO, hold: true };
            g.tick(0.016, t, &idle, &mut out, &crate::i18n::get_texts(&crate::settings::Language::UA));
            g.tick(0.016, t + 0.4, &press, &mut out, &crate::i18n::get_texts(&crate::settings::Language::UA));
            assert_eq!(g.fish.ph, 1, "удочка заброшена");
            assert_eq!(g.inv[It::Worm as usize], 0, "червь потрачен");
            t += 0.4;
            // ждём поклёвку
            g.tick(0.016, t + 0.5, &idle, &mut out, &crate::i18n::get_texts(&crate::settings::Language::UA));
            t += 9.0;
            g.tick(0.016, t, &idle, &mut out, &crate::i18n::get_texts(&crate::settings::Language::UA));
            assert_eq!(g.fish.ph, 2, "рыба клюнула");
            g.tick(0.016, t + 0.2, &press, &mut out, &crate::i18n::get_texts(&crate::settings::Language::UA));
            assert_eq!(g.fish.ph, 3, "вываживание началось");
            let mut tt = t + 0.2;
            for _ in 0..4000 {
                tt += 0.02;
                let hold = g.fish.tn < 55.0;
                g.tick(0.02, tt, &Input { dir: Vec2::ZERO, hold }, &mut out, &crate::i18n::get_texts(&crate::settings::Language::UA));
                if g.fish.ph == 0 {
                    break;
                }
            }
            assert_eq!(g.fish.ph, 0);
            if g.inv[It::Fish as usize] > 0 {
                catches += 1;
                assert!(g.biggest > 0.0 && g.caught == 1);
            }
        }
        assert!(catches > 10, "умелый игрок должен часто вытаскивать рыбу, поймано {catches}/40");
    }

    #[test]
    fn taming_wolf_and_pet_fights() {
        let mut g = Game::new();
        g.mobs.clear();
        g.mobs.push(Mob::new(MobKind::Wolf, g.x + 1.0, g.y));
        g.inv[It::Meat as usize] = 3;
        for i in 0..3 {
            g.tame(10.0 + i as f64, &crate::i18n::get_texts(&crate::settings::Language::UA));
        }
        assert!(g.has_pet && g.mobs[0].kind == MobKind::Pet);
        // питомец нападает на волка (ночь зафиксирована так, чтобы волки не исчезали и не появлялись новые)
        g.night_override = Some(0.2);
        g.mobs.push(Mob::new(MobKind::Wolf, g.x + 2.0, g.y));
        let mut out = vec![];
        let mut t = 20.0;
        for _ in 0..400 {
            t += 0.05;
            g.hp = 100.0;
            g.tick(0.05, t, &Input { dir: Vec2::ZERO, hold: false }, &mut out, &crate::i18n::get_texts(&crate::settings::Language::UA));
        }
        let wolf = g.mobs.iter().find(|m| m.kind == MobKind::Wolf);
        assert!(wolf.map_or(true, |w| w.dead > 0.0), "питомец должен победить волка");
        assert!(g.mobs.iter().any(|m| m.kind == MobKind::Pet), "питомец жив");
    }

    #[test]
    fn rabbit_needs_spear_and_knife_for_carcass() {
        let mut g = Game::new();
        g.mobs.clear();
        for n in g.map.nodes.iter_mut() {
            n.dead_until = 1e9;
        }
        g.mobs.push(Mob::new(MobKind::Rabbit, g.x + 0.8, g.y));
        let mut out = vec![];
        let mut t = 10.0;
        hit(&mut g, &mut t, 2, &mut out);
        assert!(g.msg.contains("копьё"));
        g.tools[Tl::Spear as usize] = 20;
        for _ in 0..3 {
            g.mobs[0].x = g.x + 0.8;
            g.mobs[0].y = g.y;
            hit(&mut g, &mut t, 1, &mut out);
        }
        assert!(g.mobs[0].dead > 0.0, "кролик убит");
        assert_eq!(g.inv[It::Meat as usize], 0, "мясо только после разделки");
        hit(&mut g, &mut t, 1, &mut out);
        assert!(g.msg.contains("нож"));
        g.tools[Tl::Knife as usize] = 15;
        hit(&mut g, &mut t, 1, &mut out);
        assert_eq!(g.inv[It::Meat as usize], 2);
        assert_eq!(g.inv[It::Hide as usize], 1);
    }
}
