//! Рибалка

use crate::game::{Game, FISHES};
use crate::data::{It, Tl};
use crate::i18n::Texts;
use macroquad::prelude::rand;

fn rnd() -> f32 {
    rand::gen_range(0.0f32, 1.0)
}

impl Game {
    /// Закинути вудку
    pub fn cast(&mut self, wx: f32, wy: f32, now: f64, texts: &Texts) {
        self.inv[It::Worm as usize] -= 1;
        self.use_tool(Tl::Rod, now);
        self.fish.ph = 1;
        self.fish.armed = false;
        self.fish.wx = wx;
        self.fish.wy = wy;
        self.fish.bite = now + 2.5 + rnd() as f64 * 5.5;
        self.face_to(wx, wy);
        self.last_hit = now;
        self.say(texts.cast_rod, now);
    }

    /// Тік рибалки - обробка всіх фаз
    pub fn fishing_tick(&mut self, dt: f32, now: f64, hold: bool, texts: &Texts) {
        if !hold {
            self.fish.armed = true;
        }
        if self.fish.ph == 0 {
            return;
        }
        if self.moving {
            self.fish.ph = 0;
            self.say(texts.moved_rod_retracted, now);
            return;
        }
        match self.fish.ph {
            1 => {
                // Фаза 1: чекаємо поклювання
                if hold && self.fish.armed {
                    self.fish.armed = false;
                    self.fish.ph = 0;
                    self.say(texts.retracted_rod, now);
                } else if now > self.fish.bite {
                    self.fish.ph = 2;
                    self.fish.t = now + 1.3;
                    self.say(texts.fish_biting, now);
                }
            }
            2 => {
                // Фаза 2: клює, треба натиснути
                if hold && self.fish.armed {
                    self.fish.armed = false;
                    self.fish.ph = 3;
                    self.fish.p = 30.0;
                    self.fish.tn = 40.0;
                    let r = rnd() - if self.night() > 0.3 { 0.1 } else { 0.0 };
                    self.fish.kind = if r < 0.45 { 0 } else if r < 0.75 { 1 } else if r < 0.93 { 2 } else { 3 };
                    self.say(texts.pull_keep_green, now);
                } else if now > self.fish.t {
                    self.fish.ph = 0;
                    self.say(texts.fish_escaped, now);
                }
            }
            _ => {
                // Фаза 3: вивіджуємо рибу
                let f = FISHES[self.fish.kind];
                let pw = f.2;
                let pull = (now as f32 / 0.23 + pw).sin() * pw * 7.0;
                self.fish.tn += (if hold { 26.0 + pw * 8.0 } else { -32.0 } + pull) * dt;
                self.fish.tn = self.fish.tn.max(0.0);
                if self.fish.tn >= 100.0 {
                    self.fish.ph = 0;
                    self.say(texts.line_broke, now);
                } else {
                    let good = self.fish.tn > 30.0 && self.fish.tn < 75.0;
                    self.fish.p += (if good { 16.0 - pw * 3.0 } else { -9.0 }) * dt;
                    if self.fish.p >= 100.0 {
                        // Спіймали!
                        self.fish.ph = 0;
                        let wt = ((f.3 + rnd() * (f.4 - f.3)) * 100.0).round() / 100.0;
                        self.inv[It::Fish as usize] += f.1;
                        self.got += f.1 * 3;
                        self.caught += 1;
                        if wt > self.biggest {
                            self.biggest = wt;
                        }
                        self.say(&format!("{}: {} {} кг", texts.caught_fish, f.0, wt), now);
                    } else if self.fish.p <= 0.0 {
                        self.fish.ph = 0;
                        self.say(texts.fish_got_away, now);
                    }
                }
            }
        }
    }
}
