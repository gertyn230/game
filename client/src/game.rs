//! Логика игры: выживание, добыча, крафт, звери, рыбалка, лодка, мосты.
use std::collections::{HashMap, HashSet};

use macroquad::prelude::{rand, Vec2};
use shared::ClientMsg;

use crate::data::*;
use crate::save::Save;
use crate::world::*;

pub const RABBITS: usize = 10;

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
    /// Для тестов: зафиксировать время суток.
    pub night_override: Option<f32>,
}

impl Game {
    pub fn new() -> Game {
        let mut mobs = Vec::new();
        for _ in 0..RABBITS {
            mobs.push(Mob::new(MobKind::Rabbit, 5.0 + rnd() * 30.0, 20.0 + rnd() * 15.0));
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

    fn face_to(&mut self, tx: f32, ty: f32) {
        let (dx, dy) = (tx - self.x, ty - self.y);
        self.face = if dx.abs() > dy.abs() { if dx > 0.0 { 3 } else { 2 } } else if dy > 0.0 { 0 } else { 1 };
    }

    pub fn near_fire(&self, now: f64) -> bool {
        self.fires.iter().any(|f| f.2 > now && (f.0 - self.x).powi(2) + (f.1 - self.y).powi(2) < 6.0)
    }
    pub fn near_furnace(&self) -> bool {
        self.furn.iter().any(|f| (f.0 - self.x).powi(2) + (f.1 - self.y).powi(2) < 9.0)
    }
    fn near_water(&self) -> bool {
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
    pub fn can_craft(&self, r: &Recipe, now: f64) -> bool {
        r.cost.iter().all(|(it, n)| self.inv[*it as usize] >= *n)
            && !matches!(r.out, Out::Tool(t) if self.tool(t))
            && !(r.fire && !self.near_fire(now))
            && !(r.furnace && !self.near_furnace())
    }
    pub fn craft(&mut self, i: usize, now: f64) {
        let Some(r) = RECIPES.get(i) else { return };
        if !self.can_craft(r, now) {
            return;
        }
        for (it, n) in r.cost {
            self.inv[*it as usize] -= n;
        }
        match r.out {
            Out::Item(it, n) => self.inv[it as usize] += n,
            Out::Tool(t) => {
                self.tools[t as usize] = TL_DUR[t as usize];
                self.made += 1;
            }
            Out::Fire => self.fires.push((self.x, self.y, now + 240.0)),
            Out::Furnace => self.furn.push((self.x, self.y)),
            Out::House => self.houses += 1,
        }
        self.say(&format!("Готово: {}", r.name), now);
    }
    /// Рецепты, доступные сейчас, по важности для текущей ситуации.
    pub fn quick_list(&self, now: f64) -> Vec<usize> {
        let nt = self.night() > 0.2;
        let wolf = self.mobs.iter().any(|m| m.kind == MobKind::Wolf && m.dead == 0.0 && ((m.x - self.x).powi(2) + (m.y - self.y).powi(2)).sqrt() < 12.0);
        let mut v: Vec<(i32, usize)> = RECIPES
            .iter()
            .enumerate()
            .filter(|(_, r)| self.can_craft(r, now))
            .map(|(i, r)| {
                let mut p = i as i32;
                match r.out {
                    Out::Fire if nt || wolf => p -= 100,
                    Out::Item(It::Roast | It::RFish | It::Meal, _) if self.hunger < 55.0 => p -= 60,
                    Out::Tool(Tl::Spear) if wolf => p -= 80,
                    Out::Tool(Tl::Coat | Tl::Mail) if self.hp < 60.0 => p -= 40,
                    _ => {}
                }
                (p, i)
            })
            .collect();
        v.sort();
        v.into_iter().map(|x| x.1).collect()
    }

    // ---------- действия ----------
    pub fn eat(&mut self, now: f64) {
        let h = |g: &mut Game, v: f32| g.hunger = (g.hunger + v).min(100.0);
        if self.inv[It::Roast as usize] > 0 {
            self.inv[It::Roast as usize] -= 1;
            h(self, 70.0);
        } else if self.inv[It::RFish as usize] > 0 {
            self.inv[It::RFish as usize] -= 1;
            h(self, 65.0);
        } else if self.inv[It::Meal as usize] > 0 {
            self.inv[It::Meal as usize] -= 1;
            h(self, 60.0);
        } else if self.inv[It::Food as usize] > 0 {
            self.inv[It::Food as usize] -= 1;
            h(self, 20.0);
        } else if self.inv[It::Meat as usize] > 0 {
            self.inv[It::Meat as usize] -= 1;
            h(self, 15.0);
            self.hp -= 8.0;
            self.say("Сырое мясо вредно — пожарь у костра", now);
        } else if self.inv[It::Fish as usize] > 0 {
            self.inv[It::Fish as usize] -= 1;
            h(self, 12.0);
            self.hp -= 4.0;
            self.say("Сырая рыба — лучше пожарь", now);
        }
    }

    pub fn drink(&mut self, now: f64) {
        if self.fish.ph != 0 {
            return;
        }
        if self.near_water() {
            self.thirst = (self.thirst + 15.0).min(100.0);
            self.last_hit = now;
            self.energy -= 0.3;
            self.say("Ты пьёшь воду", now);
        } else {
            self.say("Воды рядом нет", now);
        }
    }

    pub fn dig(&mut self, now: f64) {
        if self.fish.ph != 0 || now - self.last_hit < 0.5 {
            return;
        }
        if !self.tool(Tl::Shovel) {
            self.say("Нужна лопата: палки, камешки, нить", now);
            return;
        }
        let (tx, ty) = (self.x.floor() as i32, self.y.floor() as i32);
        let t = self.map.terr_at(tx, ty);
        if t == WATER || t == FORD {
            self.say("Тут нельзя копать", now);
            return;
        }
        let bm = biome(tx, ty);
        if bm == 2 {
            self.say("Плато слишком твёрдое", now);
            return;
        }
        let e = self.dugs(tx, ty, now);
        if e >= 3 {
            self.say("Здесь уже перекопано", now);
            return;
        }
        self.last_hit = now;
        self.energy -= 2.0;
        self.use_tool(Tl::Shovel, now);
        self.fx = Some(Fx { x: self.x, y: self.y + 0.4, t: now, tool: 4, col: 0x6b4a2b });
        let ch = (if bm == 1 { 0.7 } else if t == SAND { 0.25 } else { 0.55 }) + if self.rain() { 0.2 } else { 0.0 };
        if rnd() < ch {
            self.inv[It::Worm as usize] += 1;
            self.say("Нашёл червя", now);
        } else {
            self.say("Пусто, попробуй в другом месте", now);
        }
    }
    fn dugs(&mut self, tx: i32, ty: i32, now: f64) -> u8 {
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
    fn use_tool(&mut self, t: Tl, now: f64) {
        let d = &mut self.tools[t as usize];
        *d -= 1;
        if *d <= 0 {
            *d = 0;
            self.say(&format!("{} сломан(а)", TL_NAMES[t as usize]), now);
        }
    }

    pub fn bridge(&mut self, now: f64) {
        if self.in_boat {
            self.say("Из лодки мост не построить", now);
            return;
        }
        let d = [(0, 1), (0, -1), (-1, 0), (1, 0)][self.face as usize];
        let c = (self.x.floor() as i32 + d.0, self.y.floor() as i32 + d.1);
        if self.map.terr_at(c.0, c.1) != WATER || self.bridges.contains(&c) {
            self.say("Встань лицом к воде", now);
        } else if self.inv[It::Plank as usize] < 2 {
            self.say("Нужно 2 доски", now);
        } else {
            self.inv[It::Plank as usize] -= 2;
            self.bridges.insert(c);
            self.say("Мост построен", now);
        }
    }

    pub fn boat_toggle(&mut self, now: f64) {
        if !self.tool(Tl::Boat) {
            self.say("Нужна лодка: 10 досок и 4 нити", now);
            return;
        }
        let (x0, y0) = (self.x.floor() as i32, self.y.floor() as i32);
        let mut hit = None;
        'o: for a in -1..=1 {
            for b in -1..=1 {
                let (x, y) = (x0 + a, y0 + b);
                let t = self.map.terr_at(x, y);
                let ok = if self.in_boat {
                    (t == GRASS || t == SAND) && self.map.node_at(x, y).is_none()
                } else {
                    t == WATER && !self.bridges.contains(&(x, y))
                };
                if ok {
                    hit = Some((x, y));
                    break 'o;
                }
            }
        }
        match hit {
            None => self.say(if self.in_boat { "Нужен свободный берег" } else { "Подойди к воде" }, now),
            Some((x, y)) => {
                self.in_boat = !self.in_boat;
                self.x = x as f32 + 0.5;
                self.y = y as f32 + 0.5;
                self.say(if self.in_boat { "Ты в лодке" } else { "Ты на берегу" }, now);
            }
        }
    }

    pub fn tame(&mut self, now: f64) {
        let mut best = None;
        let mut bd = 12.25;
        for (i, m) in self.mobs.iter().enumerate() {
            if m.kind != MobKind::Wolf || m.dead > 0.0 {
                continue;
            }
            let d = (m.x - self.x).powi(2) + (m.y - self.y).powi(2);
            if d < bd {
                bd = d;
                best = Some(i);
            }
        }
        let Some(i) = best else {
            self.say("Рядом нет волка", now);
            return;
        };
        if self.has_pet {
            self.say("У тебя уже есть питомец", now);
        } else if self.inv[It::Meat as usize] < 1 {
            self.say("Нужно сырое мясо", now);
        } else {
            self.inv[It::Meat as usize] -= 1;
            let m = &mut self.mobs[i];
            m.calm = now + 5.0;
            m.trust += 1;
            if m.trust >= 3 {
                m.kind = MobKind::Pet;
                m.hp = 9.0;
                self.has_pet = true;
                self.say("Волк стал твоим другом!", now);
            } else {
                let t = m.trust;
                self.say(&format!("Волк доверяет: {t}/3"), now);
            }
        }
    }

    /// Основное действие (кнопка «Добыть»).
    fn act(&mut self, now: f64, out: &mut Vec<ClientMsg>) {
        if self.fish.ph != 0 || now - self.last_hit < 0.3 || self.energy < 5.0 {
            return;
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
                    self.say("Нужен нож: палка, камешек, нить", now);
                    self.last_hit = now - 0.0;
                    return;
                }
                self.last_hit = now;
                self.energy -= 1.0;
                self.use_tool(Tl::Knife, now);
                let wolf = kind == MobKind::Wolf;
                let m = &mut self.mobs[i];
                m.cut = true;
                m.dead = now - if wolf { 55.0 } else { 80.0 };
                self.inv[It::Meat as usize] += if wolf { 3 } else { 2 };
                self.inv[It::Hide as usize] += if wolf { 2 } else { 1 };
                self.got += 3;
                self.say("Туша разделана: мясо и шкура", now);
                return;
            }
            if kind == MobKind::Rabbit && !self.tool(Tl::Spear) {
                self.say("Кролика поймает только копьё", now);
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
                self.say("Разделай тушу: нужен каменный нож", now);
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
                    self.cast(wx, wy, now);
                    return;
                }
            }
            if self.near_water() {
                self.drink(now);
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
        };
        self.inv[item as usize] += g;
        self.got += g;
        out.push(ClientMsg::Dead { idx: (ny as u32) * W as u32 + nx as u32 });
    }

    // ---------- рыбалка ----------
    fn cast(&mut self, wx: f32, wy: f32, now: f64) {
        self.inv[It::Worm as usize] -= 1;
        self.use_tool(Tl::Rod, now);
        self.fish.ph = 1;
        self.fish.armed = false;
        self.fish.wx = wx;
        self.fish.wy = wy;
        self.fish.bite = now + 2.5 + rnd() as f64 * 5.5;
        self.face_to(wx, wy);
        self.last_hit = now;
        self.say("Закинул удочку, жди поклёвку", now);
    }
    fn fishing_tick(&mut self, dt: f32, now: f64, hold: bool) {
        if !hold {
            self.fish.armed = true;
        }
        if self.fish.ph == 0 {
            return;
        }
        if self.moving {
            self.fish.ph = 0;
            self.say("Ты отошёл — удочка смотана", now);
            return;
        }
        match self.fish.ph {
            1 => {
                if hold && self.fish.armed {
                    self.fish.armed = false;
                    self.fish.ph = 0;
                    self.say("Ты смотал удочку", now);
                } else if now > self.fish.bite {
                    self.fish.ph = 2;
                    self.fish.t = now + 1.3;
                    self.say("Клюёт! Жми «Добыть»!", now);
                }
            }
            2 => {
                if hold && self.fish.armed {
                    self.fish.armed = false;
                    self.fish.ph = 3;
                    self.fish.p = 30.0;
                    self.fish.tn = 40.0;
                    let r = rnd() - if self.night() > 0.3 { 0.1 } else { 0.0 };
                    self.fish.kind = if r < 0.45 { 0 } else if r < 0.75 { 1 } else if r < 0.93 { 2 } else { 3 };
                    self.say("Тяни! Держи полоску в зелёной зоне", now);
                } else if now > self.fish.t {
                    self.fish.ph = 0;
                    self.say("Рыба сорвалась", now);
                }
            }
            _ => {
                let f = FISHES[self.fish.kind];
                let pw = f.2;
                let pull = (now as f32 / 0.23 + pw).sin() * pw * 7.0;
                self.fish.tn += (if hold { 26.0 + pw * 8.0 } else { -32.0 } + pull) * dt;
                self.fish.tn = self.fish.tn.max(0.0);
                if self.fish.tn >= 100.0 {
                    self.fish.ph = 0;
                    self.say("Леска порвалась!", now);
                } else {
                    let good = self.fish.tn > 30.0 && self.fish.tn < 75.0;
                    self.fish.p += (if good { 16.0 - pw * 3.0 } else { -9.0 }) * dt;
                    if self.fish.p >= 100.0 {
                        self.fish.ph = 0;
                        let wt = ((f.3 + rnd() * (f.4 - f.3)) * 100.0).round() / 100.0;
                        self.inv[It::Fish as usize] += f.1;
                        self.got += f.1 * 3;
                        self.caught += 1;
                        if wt > self.biggest {
                            self.biggest = wt;
                        }
                        self.say(&format!("Поймана: {} {} кг", f.0, wt), now);
                    } else if self.fish.p <= 0.0 {
                        self.fish.ph = 0;
                        self.say("Рыба ушла", now);
                    }
                }
            }
        }
    }

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
    fn die(&mut self, now: f64) {
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
        self.say("Ты погиб и потерял вещи", now);
    }

    // ---------- главный тик ----------
    pub fn tick(&mut self, dt: f32, now: f64, inp: &Input, out: &mut Vec<ClientMsg>) {
        // выживание
        self.hunger = (self.hunger - 0.35 * dt).max(0.0);
        self.thirst = (self.thirst - 0.28 * dt).max(0.0);
        let rain = self.rain();
        let de = if self.moving { -(if rain { 1.0 } else { 0.5 }) } else { 1.5 };
        self.energy = (self.energy + de * dt).clamp(0.0, 100.0);
        if self.hunger <= 0.0 || self.thirst <= 0.0 {
            self.hp -= dt;
        } else if self.hunger > 50.0 && self.thirst > 40.0 && self.hp < 100.0 {
            self.hp = (self.hp + 0.3 * dt).min(100.0);
        }
        if self.hp <= 0.0 {
            self.die(now);
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
        self.fishing_tick(dt, now, inp.hold);
        if inp.hold {
            self.act(now, out);
        }
        self.fires.retain(|f| f.2 > now);
        self.mobs_tick(dt, now);
    }

    fn mobs_tick(&mut self, dt: f32, now: f64) {
        let nt = self.night();
        let wolves = self.mobs.iter().filter(|m| m.kind == MobKind::Wolf && m.dead == 0.0).count();
        if nt > 0.3 && wolves < 3 && rnd() < dt * 0.15 {
            let a = rnd() * 6.283;
            let (x, y) = (self.x + a.cos() * 8.0, self.y + a.sin() * 8.0);
            if !self.blocked(x, y, false, now) {
                self.mobs.push(Mob::new(MobKind::Wolf, x, y));
            }
        }
        let fire_near = |g: &Game, x: f32, y: f32, r: f32| g.fires.iter().any(|f| (f.0 - x).powi(2) + (f.1 - y).powi(2) < r * r);
        let mut remove = vec![false; self.mobs.len()];
        let mut pet_hits: Vec<usize> = vec![];
        let (px, py) = (self.x, self.y);
        let mut bites = 0.0f32;
        for i in 0..self.mobs.len() {
            let mut m = self.mobs[i].clone();
            if m.kind == MobKind::Wolf {
                if m.dead > 0.0 {
                    if now - m.dead > 60.0 {
                        remove[i] = true;
                    }
                    continue;
                }
                if nt < 0.1 {
                    remove[i] = true;
                    continue;
                }
            } else if m.dead > 0.0 {
                if now - m.dead > 90.0 {
                    m.dead = 0.0;
                    m.cut = false;
                    m.hp = 2.0;
                    m.x = 5.0 + rnd() * 30.0;
                    m.y = 5.0 + rnd() * 30.0;
                    m.px = m.x;
                    m.py = m.y;
                    self.mobs[i] = m;
                }
                continue;
            }
            let (mut sx, mut sy) = (0.0f32, 0.0f32);
            let (dx, dy) = (px - m.x, py - m.y);
            let d = (dx * dx + dy * dy).sqrt().max(0.001);
            match m.kind {
                MobKind::Pet => {
                    if d > 14.0 {
                        m.x = px;
                        m.y = py;
                        self.mobs[i] = m;
                        continue;
                    }
                    let mut tg = None;
                    let mut td = 25.0;
                    for (j, w) in self.mobs.iter().enumerate() {
                        if w.kind != MobKind::Wolf || w.dead > 0.0 {
                            continue;
                        }
                        let e = (w.x - m.x).powi(2) + (w.y - m.y).powi(2);
                        if e < td {
                            td = e;
                            tg = Some(j);
                        }
                    }
                    if let Some(j) = tg {
                        let (ex, ey) = (self.mobs[j].x - m.x, self.mobs[j].y - m.y);
                        let ed = (ex * ex + ey * ey).sqrt().max(0.001);
                        sx = ex / ed * 3.0;
                        sy = ey / ed * 3.0;
                        if ed < 1.0 && now - m.hit > 0.9 {
                            m.hit = now;
                            pet_hits.push(j);
                        }
                    } else if d > 2.0 {
                        sx = dx / d * 3.0;
                        sy = dy / d * 3.0;
                    }
                }
                MobKind::Wolf => {
                    let nf = fire_near(self, m.x, m.y, 3.5);
                    if m.calm > now {
                    } else if nf {
                        sx = -dx / d * 2.0;
                        sy = -dy / d * 2.0;
                    } else if d < 14.0 {
                        sx = dx / d * 2.4;
                        sy = dy / d * 2.4;
                    }
                    if d < 0.9 && !nf && m.calm <= now && now - m.hit > 1.0 {
                        m.hit = now;
                        bites += if self.tool(Tl::Mail) { 2.0 } else if self.tool(Tl::Coat) { 3.0 } else { 6.0 };
                    }
                }
                MobKind::Rabbit => {
                    if self.blocked(m.x, m.y, false, now) {
                        m.x = 5.0 + rnd() * 30.0;
                        m.y = 20.0 + rnd() * 15.0;
                        self.mobs[i] = m;
                        continue;
                    }
                    if d < 3.0 {
                        sx = -dx / d * 3.0;
                        sy = -dy / d * 3.0;
                    } else {
                        m.t -= dt;
                        if m.t <= 0.0 {
                            m.t = 1.0 + rnd() * 2.0;
                            let a = rnd() * 6.283;
                            let s = if rnd() < 0.4 { 0.0 } else { 0.8 };
                            m.vx = a.cos() * s;
                            m.vy = a.sin() * s;
                        }
                        sx = m.vx;
                        sy = m.vy;
                    }
                }
            }
            let (nx, ny) = (m.x + sx * dt, m.y + sy * dt);
            if !self.blocked(nx, m.y, false, now) {
                m.x = nx;
            }
            if !self.blocked(m.x, ny, false, now) {
                m.y = ny;
            }
            // сохраняем только то, что мы меняли (метки видимости/вспышки остаются)
            let keep_flash = self.mobs[i].flash;
            let keep_hp = self.mobs[i].hp;
            m.flash = keep_flash;
            m.hp = keep_hp;
            m.fl = if (m.x - m.px) < -0.002 { true } else if (m.x - m.px) > 0.002 { false } else { m.fl };
            m.mv = (m.x - m.px).abs() + (m.y - m.py).abs() > 0.002;
            m.px = m.x;
            m.py = m.y;
            self.mobs[i] = m;
        }
        for j in pet_hits {
            let w = &mut self.mobs[j];
            w.hp -= 2.0;
            w.flash = now;
            if w.hp <= 0.0 {
                w.dead = now;
            }
        }
        if bites > 0.0 {
            self.hp -= bites;
            self.say("Волк кусает!", now);
        }
        let mut k = 0;
        self.mobs.retain(|_| {
            k += 1;
            !remove[k - 1]
        });
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
            g.tick(0.016, *t, &Input { dir: Vec2::ZERO, hold: true }, out);
            g.tick(0.016, *t + 0.1, &Input { dir: Vec2::ZERO, hold: false }, out);
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
        assert_eq!(g.mobs.len(), RABBITS);
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
        g.eat(now);
        assert_eq!(g.hunger, 90.0);
    }

    #[test]
    fn survival_and_death() {
        let mut g = Game::new();
        g.mobs.clear();
        let mut out = vec![];
        let h0 = g.hunger;
        g.tick(1.0, 1.0, &Input { dir: Vec2::ZERO, hold: false }, &mut out);
        assert!(g.hunger < h0 && g.thirst < 100.0);
        g.inv[It::Wood as usize] = 5;
        g.hp = 0.5;
        g.hunger = 0.0;
        g.tick(1.0, 2.0, &Input { dir: Vec2::ZERO, hold: false }, &mut out);
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
        g.drink(1.0);
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
        g.boat_toggle(1.0);
        assert!(!g.in_boat, "без лодки не поплывёшь");
        g.tools[Tl::Boat as usize] = 1;
        g.boat_toggle(1.0);
        assert!(g.in_boat);
        assert_eq!(g.map.terr_at(g.x.floor() as i32, g.y.floor() as i32), WATER);
        assert!(g.blocked(x - 1.0, y, true, 1.0), "в лодке на сушу нельзя");
        g.boat_toggle(2.0);
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
        g.dig(1.0);
        assert!(g.msg.contains("лопата"));
        g.tools[Tl::Shovel as usize] = 30;
        let mut t = 2.0;
        for _ in 0..3 {
            g.dig(t);
            t += 1.0;
        }
        g.dig(t);
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
            g.tick(0.016, t, &idle, &mut out);
            g.tick(0.016, t + 0.4, &press, &mut out);
            assert_eq!(g.fish.ph, 1, "удочка заброшена");
            assert_eq!(g.inv[It::Worm as usize], 0, "червь потрачен");
            t += 0.4;
            // ждём поклёвку
            g.tick(0.016, t + 0.5, &idle, &mut out);
            t += 9.0;
            g.tick(0.016, t, &idle, &mut out);
            assert_eq!(g.fish.ph, 2, "рыба клюнула");
            g.tick(0.016, t + 0.2, &press, &mut out);
            assert_eq!(g.fish.ph, 3, "вываживание началось");
            let mut tt = t + 0.2;
            for _ in 0..4000 {
                tt += 0.02;
                let hold = g.fish.tn < 55.0;
                g.tick(0.02, tt, &Input { dir: Vec2::ZERO, hold }, &mut out);
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
            g.tame(10.0 + i as f64);
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
            g.tick(0.05, t, &Input { dir: Vec2::ZERO, hold: false }, &mut out);
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
