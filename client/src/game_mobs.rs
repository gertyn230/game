//! Тварини: приручення, бій, тік тварин

use crate::game::{Game, MobKind};
use crate::data::{It, Tl};
use crate::i18n::Texts;
use macroquad::prelude::rand;

fn rnd() -> f32 {
    rand::gen_range(0.0f32, 1.0)
}

impl Game {
    /// Приручити вовка
    pub fn tame(&mut self, now: f64, texts: &Texts) {
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
            self.say(texts.no_wolf_nearby, now);
            return;
        };
        if self.has_pet {
            self.say(texts.already_have_pet, now);
        } else if self.inv[It::Meat as usize] < 1 {
            self.say(texts.need_raw_meat, now);
        } else {
            self.inv[It::Meat as usize] -= 1;
            let m = &mut self.mobs[i];
            m.calm = now + 5.0;
            m.trust += 1;
            if m.trust >= 3 {
                m.kind = MobKind::Pet;
                m.hp = 9.0;
                self.has_pet = true;
                self.say(texts.wolf_is_friend, now);
            } else {
                let t = m.trust;
                self.say(&format!("{}: {t}/3", texts.wolf_trust), now);
            }
        }
    }

    /// Тік тварин - рух, атаки, смерть
    pub fn mobs_tick(&mut self, dt: f32, now: f64, texts: &Texts) {
        // Зберегти стан гравця щоб не було проблем з borrow checker
        let player_x = self.x;
        let player_y = self.y;
        let night_level = self.night();
        let fires_positions: Vec<(f32, f32)> = self.fires.iter().map(|f| (f.0, f.1)).collect();

        for m in &mut self.mobs {
            if m.dead > 0.0 {
                continue;
            }
            let dx = player_x - m.x;
            let dy = player_y - m.y;
            let d = (dx * dx + dy * dy).sqrt();

            match m.kind {
                MobKind::Rabbit => {
                    // Кролики тікають від гравця
                    if d < 4.0 {
                        let ang = dy.atan2(dx) + std::f32::consts::PI;
                        m.x += ang.cos() * 1.8 * dt;
                        m.y += ang.sin() * 1.8 * dt;
                    } else {
                        m.x += (rnd() - 0.5) * 0.4 * dt;
                        m.y += (rnd() - 0.5) * 0.4 * dt;
                    }
                }
                MobKind::Deer => {
                    // Олені тікають від гравця, але повільніше ніж кролики
                    if d < 5.0 {
                        let ang = dy.atan2(dx) + std::f32::consts::PI;
                        m.x += ang.cos() * 1.5 * dt;
                        m.y += ang.sin() * 1.5 * dt;
                    } else {
                        m.x += (rnd() - 0.5) * 0.3 * dt;
                        m.y += (rnd() - 0.5) * 0.3 * dt;
                    }
                }
                MobKind::Fox => {
                    // Лисиці швидкі і хитрі, тікають далеко
                    if d < 6.0 {
                        let ang = dy.atan2(dx) + std::f32::consts::PI;
                        m.x += ang.cos() * 2.2 * dt;
                        m.y += ang.sin() * 2.2 * dt;
                    } else {
                        m.x += (rnd() - 0.5) * 0.5 * dt;
                        m.y += (rnd() - 0.5) * 0.5 * dt;
                    }
                }
                MobKind::Wolf => {
                    // Вовки атакують вночі якщо нема вогнища поруч
                    if night_level > 0.2 && now > m.calm {
                        let near_fire = fires_positions.iter().any(|f| {
                            (f.0 - m.x).powi(2) + (f.1 - m.y).powi(2) < 64.0
                        });
                        if !near_fire && d < 8.0 {
                            let ang = dy.atan2(dx);
                            m.x += ang.cos() * 1.2 * dt;
                            m.y += ang.sin() * 1.2 * dt;
                        }
                    }
                }
                MobKind::Bear => {
                    // Ведмеді атакують завжди, якщо гравець близько
                    if now > m.calm && d < 6.0 {
                        let near_fire = fires_positions.iter().any(|f| {
                            (f.0 - m.x).powi(2) + (f.1 - m.y).powi(2) < 36.0
                        });
                        if !near_fire {
                            let ang = dy.atan2(dx);
                            m.x += ang.cos() * 1.0 * dt;
                            m.y += ang.sin() * 1.0 * dt;
                        }
                    } else {
                        m.x += (rnd() - 0.5) * 0.2 * dt;
                        m.y += (rnd() - 0.5) * 0.2 * dt;
                    }
                }
                MobKind::Pet => {
                    // Питомець слідує за гравцем
                    if d > 2.0 {
                        let ang = dy.atan2(dx);
                        m.x += ang.cos() * 1.5 * dt;
                        m.y += ang.sin() * 1.5 * dt;
                    }
                }
            }

            m.x = m.x.max(1.0).min(39.0);
            m.y = m.y.max(1.0).min(39.0);
        }

        // Атака вовків і ведмедів
        let mut bites = 0.0;
        for m in &self.mobs {
            if m.dead == 0.0 && now > m.calm {
                let dx = self.x - m.x;
                let dy = self.y - m.y;
                let dist = dx * dx + dy * dy;

                if m.kind == MobKind::Wolf && dist < 1.0 {
                    bites += 0.8 * dt;
                } else if m.kind == MobKind::Bear && dist < 1.5 {
                    bites += 2.0 * dt; // Ведмідь б'є сильніше
                }
            }
        }
        if bites > 0.0 {
            self.hp -= bites;
            self.say(texts.wolf_bites, now);
        }

        // Видалити мертвих тварин (після 120 секунд)
        let mut k = 0;
        let mut remove = vec![false; self.mobs.len()];
        for m in &self.mobs {
            if m.dead > 0.0 && now - m.dead > 120.0 {
                remove[k] = true;
            }
            k += 1;
        }
        let mut k = 0;
        self.mobs.retain(|_| {
            k += 1;
            !remove[k - 1]
        });
    }
}
