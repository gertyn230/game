//! Дії гравця: копання, мости, човен

use crate::game::Game;
use crate::data::{It, Tl};
use crate::i18n::Texts;
use crate::world::{WATER, FORD, GRASS, SAND, biome};
use crate::game::Fx;
use macroquad::prelude::rand;

fn rnd() -> f32 {
    rand::gen_range(0.0f32, 1.0)
}

impl Game {
    /// Копати землю лопатою для пошуку черв'яків
    pub fn dig(&mut self, now: f64, texts: &Texts) {
        if self.fish.ph != 0 || now - self.last_hit < 0.5 {
            return;
        }
        if !self.tool(Tl::Shovel) {
            self.say(texts.need_shovel, now);
            return;
        }
        let (tx, ty) = (self.x.floor() as i32, self.y.floor() as i32);
        let t = self.map.terr_at(tx, ty);
        if t == WATER || t == FORD {
            self.say(texts.cant_dig_here, now);
            return;
        }
        let bm = biome(tx, ty);
        if bm == 2 {
            self.say(texts.plateau_too_hard, now);
            return;
        }
        let e = self.dugs(tx, ty, now);
        if e >= 3 {
            self.say(texts.already_dug, now);
            return;
        }
        self.last_hit = now;
        self.energy -= 2.0;
        self.use_tool(Tl::Shovel, now);
        self.fx = Some(Fx { x: self.x, y: self.y + 0.4, t: now, tool: 4, col: 0x6b4a2b });
        let ch = (if bm == 1 { 0.7 } else if t == SAND { 0.25 } else { 0.55 }) + if self.rain() { 0.2 } else { 0.0 };
        if rnd() < ch {
            self.inv[It::Worm as usize] += 1;
            self.say(texts.found_worm, now);
        } else {
            self.say(texts.nothing_found, now);
        }
    }

    /// Побудувати міст через воду
    pub fn bridge(&mut self, now: f64, texts: &Texts) {
        if self.in_boat {
            self.say(texts.cant_build_from_boat, now);
            return;
        }
        let d = [(0, 1), (0, -1), (-1, 0), (1, 0)][self.face as usize];
        let c = (self.x.floor() as i32 + d.0, self.y.floor() as i32 + d.1);
        if self.map.terr_at(c.0, c.1) != WATER || self.bridges.contains(&c) {
            self.say(texts.face_water, now);
        } else if self.inv[It::Plank as usize] < 2 {
            self.say(texts.need_planks, now);
        } else {
            self.inv[It::Plank as usize] -= 2;
            self.bridges.insert(c);
            self.say(texts.bridge_built, now);
        }
    }

    /// Увійти/вийти з човна
    pub fn boat_toggle(&mut self, now: f64, texts: &Texts) {
        if !self.tool(Tl::Boat) {
            self.say(texts.need_boat, now);
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
            None => self.say(if self.in_boat { texts.need_free_shore } else { texts.approach_water }, now),
            Some((x, y)) => {
                self.in_boat = !self.in_boat;
                self.x = x as f32 + 0.5;
                self.y = y as f32 + 0.5;
                self.say(if self.in_boat { texts.in_boat } else { texts.on_shore }, now);
            }
        }
    }
}
