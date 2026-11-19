//! Тварини: приручення, бій, тік тварин

use std::collections::HashSet;
use crate::game::{Game, MobKind};
use crate::data::It;
use crate::i18n::Texts;
use crate::world::{Map, W, WATER};
use macroquad::prelude::rand;
use crate::particles::ParticleType;

fn rnd() -> f32 {
    rand::gen_range(0.0f32, 1.0)
}

/// Перевірка чи заблокована точка для руху тварин (межі карти, вода без мостів, будинки, дерева, скелі, руди)
fn is_blocked(map: &Map, bridges: &HashSet<(i32, i32)>, houses: u32, x: f32, y: f32, now: f64) -> bool {
    let w = W as f32;
    if x < 1.0 || y < 1.0 || x > w - 1.5 || y > w - 1.5 {
        return true;
    }
    let tx = x.floor() as i32;
    let ty = y.floor() as i32;
    let tt = map.terr_at(tx, ty);
    if tt == WATER && !bridges.contains(&(tx, ty)) {
        return true;
    }
    for i in 0..houses.min(24) as i32 {
        if tx == 14 + i % 12 && ty == 15 - i / 12 {
            return true;
        }
    }
    matches!(map.node_at(tx, ty), Some(n) if n.kind.blocks() && n.dead_until <= now)
}

/// Пошук кута руху з обходом перешкод (дерева, каміння, вода)
fn find_steered_velocity(
    map: &Map,
    bridges: &HashSet<(i32, i32)>,
    houses: u32,
    mx: f32,
    my: f32,
    target_ang: f32,
    speed: f32,
    now: f64,
) -> (f32, f32) {
    let check_dist = 0.45f32;
    const ANGLE_OFFSETS: [f32; 9] = [
        0.0,
        0.35,
        -0.35,
        0.75,
        -0.75,
        1.25,
        -1.25,
        1.9,
        -1.9,
    ];

    for &da in &ANGLE_OFFSETS {
        let ang = target_ang + da;
        let test_x = mx + ang.cos() * check_dist;
        let test_y = my + ang.sin() * check_dist;
        if !is_blocked(map, bridges, houses, test_x, test_y, now) {
            return (ang.cos() * speed, ang.sin() * speed);
        }
    }

    (0.0, 0.0)
}

/// Спокійна швидкість блукання
fn wander_speed(kind: MobKind) -> f32 {
    match kind {
        MobKind::Rabbit => 0.9,
        MobKind::Deer => 0.75,
        MobKind::Fox => 1.0,
        MobKind::Wolf => 0.7,
        MobKind::Bear => 0.55,
        MobKind::Pet => 0.8,
    }
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

    /// Тік тварин - плавний рух, обхід перешкод, колізії, атаки, смерть
    pub fn mobs_tick(&mut self, dt: f32, now: f64, texts: &Texts) {
        let dt = dt.min(0.1);
        let player_x = self.x;
        let player_y = self.y;
        let night_level = self.night();
        let map = &self.map;
        let bridges = &self.bridges;
        let houses = self.houses;

        let fires_positions: Vec<(f32, f32)> = self.fires.iter().filter(|f| f.2 > now).map(|f| (f.0, f.1)).collect();
        let wolf_targets: Vec<(usize, f32, f32)> = self.mobs.iter().enumerate()
            .filter(|(_, m)| m.kind == MobKind::Wolf && m.dead == 0.0)
            .map(|(idx, m)| (idx, m.x, m.y))
            .collect();
        let mut pet_hits: Vec<(usize, f32)> = Vec::new();

        for m in &mut self.mobs {
            if m.dead > 0.0 {
                m.vx = 0.0;
                m.vy = 0.0;
                m.mv = false;
                continue;
            }

            let dx = player_x - m.x;
            let dy = player_y - m.y;
            let d = (dx * dx + dy * dy).sqrt();

            let mut is_active = false;

            match m.kind {
                MobKind::Rabbit => {
                    // Кролики тікають від гравця, плавно оминаючи дерева та каміння
                    if d < 4.5 {
                        let flee_ang = dy.atan2(dx) + std::f32::consts::PI;
                        let (vx, vy) = find_steered_velocity(map, bridges, houses, m.x, m.y, flee_ang, 2.2, now);
                        m.vx = vx;
                        m.vy = vy;
                        m.t = 0.6;
                        is_active = true;
                    }
                }
                MobKind::Deer => {
                    // Олені полохливі, тікають при наближенні гравця
                    if d < 5.5 {
                        let flee_ang = dy.atan2(dx) + std::f32::consts::PI;
                        let (vx, vy) = find_steered_velocity(map, bridges, houses, m.x, m.y, flee_ang, 1.8, now);
                        m.vx = vx;
                        m.vy = vy;
                        m.t = 0.6;
                        is_active = true;
                    }
                }
                MobKind::Fox => {
                    // Лисиці дуже швидкі і тримаються подалі від гравця
                    if d < 6.0 {
                        let flee_ang = dy.atan2(dx) + std::f32::consts::PI;
                        let (vx, vy) = find_steered_velocity(map, bridges, houses, m.x, m.y, flee_ang, 2.4, now);
                        m.vx = vx;
                        m.vy = vy;
                        m.t = 0.6;
                        is_active = true;
                    }
                }
                MobKind::Wolf => {
                    // Вовки атакують вночі якщо нема вогнища поруч
                    let near_fire = fires_positions.iter().any(|f| {
                        (f.0 - m.x).powi(2) + (f.1 - m.y).powi(2) < 64.0
                    });
                    if night_level > 0.2 && now > m.calm && !near_fire && d < 8.0 {
                        if d > 0.75 {
                            let chase_ang = dy.atan2(dx);
                            let (vx, vy) = find_steered_velocity(map, bridges, houses, m.x, m.y, chase_ang, 1.5, now);
                            m.vx = vx;
                            m.vy = vy;
                        } else {
                            m.vx = 0.0;
                            m.vy = 0.0;
                        }
                        m.t = 0.5;
                        is_active = true;
                    }
                }
                MobKind::Bear => {
                    // Ведмеді атакують вдень і вночі, якщо гравець близько і нема вогню
                    let near_fire = fires_positions.iter().any(|f| {
                        (f.0 - m.x).powi(2) + (f.1 - m.y).powi(2) < 36.0
                    });
                    if now > m.calm && d < 6.0 && !near_fire {
                        if d > 0.9 {
                            let chase_ang = dy.atan2(dx);
                            let (vx, vy) = find_steered_velocity(map, bridges, houses, m.x, m.y, chase_ang, 1.2, now);
                            m.vx = vx;
                            m.vy = vy;
                        } else {
                            m.vx = 0.0;
                            m.vy = 0.0;
                        }
                        m.t = 0.5;
                        is_active = true;
                    }
                }
                MobKind::Pet => {
                    is_active = true;
                    if d > 14.0 {
                        // Якщо гравець занадто далеко - питомець наздоганяє його
                        m.x = (player_x - 0.8).clamp(1.0, (W - 2) as f32);
                        m.y = player_y.clamp(1.0, (W - 2) as f32);
                        m.vx = 0.0;
                        m.vy = 0.0;
                    } else {
                        // Шукаємо найближчого ворожого вовка
                        let mut target_wolf = None;
                        let mut min_dist_sq = 25.0;
                        for &(w_idx, wx, wy) in &wolf_targets {
                            let dist_sq = (wx - m.x).powi(2) + (wy - m.y).powi(2);
                            if dist_sq < min_dist_sq {
                                min_dist_sq = dist_sq;
                                target_wolf = Some((w_idx, wx, wy));
                            }
                        }

                        if let Some((target_idx, target_x, target_y)) = target_wolf {
                            let ed = ((target_x - m.x).powi(2) + (target_y - m.y).powi(2)).sqrt().max(0.001);
                            if ed > 0.75 {
                                let chase_ang = (target_y - m.y).atan2(target_x - m.x);
                                let (vx, vy) = find_steered_velocity(map, bridges, houses, m.x, m.y, chase_ang, 3.0, now);
                                m.vx = vx;
                                m.vy = vy;
                            } else {
                                m.vx = 0.0;
                                m.vy = 0.0;
                            }

                            if ed < 1.0 && now - m.hit > 0.9 {
                                m.hit = now;
                                pet_hits.push((target_idx, 2.0));
                            }
                        } else if d > 2.0 {
                            let follow_ang = dy.atan2(dx);
                            let (vx, vy) = find_steered_velocity(map, bridges, houses, m.x, m.y, follow_ang, 2.2, now);
                            m.vx = vx;
                            m.vy = vy;
                        } else {
                            m.vx = 0.0;
                            m.vy = 0.0;
                        }
                    }
                }
            }

            // Спокійне блукання / відпочинок коли тварина спокійна (без тремтіння на місці)
            if !is_active {
                m.t -= dt;
                if m.t <= 0.0 {
                    m.t = 1.8 + rnd() * 2.5; // Зміна поведінки кожні 1.8 - 4.3 секунди
                    if rnd() < 0.45 {
                        // Вибираємо напрямок ходьби
                        let a = rnd() * std::f32::consts::TAU;
                        let sp = wander_speed(m.kind);
                        let (vx, vy) = find_steered_velocity(map, bridges, houses, m.x, m.y, a, sp, now);
                        m.vx = vx;
                        m.vy = vy;
                    } else {
                        // Стоїть, щипає травку або оглядається
                        m.vx = 0.0;
                        m.vy = 0.0;
                        if rnd() < 0.35 {
                            m.fl = !m.fl;
                        }
                    }
                } else if m.vx.abs() > 0.01 || m.vy.abs() > 0.01 {
                    // Перевіряємо чи попереду немає раптової перешкоди
                    let check_x = m.x + m.vx.signum() * 0.35;
                    let check_y = m.y + m.vy.signum() * 0.35;
                    if is_blocked(map, bridges, houses, check_x, check_y, now) {
                        let cur_ang = m.vy.atan2(m.vx);
                        let sp = (m.vx * m.vx + m.vy * m.vy).sqrt();
                        let (svx, svy) = find_steered_velocity(map, bridges, houses, m.x, m.y, cur_ang, sp, now);
                        if svx.hypot(svy) > 0.01 {
                            m.vx = svx;
                            m.vy = svy;
                        } else {
                            m.vx = 0.0;
                            m.vy = 0.0;
                            m.t = 1.0 + rnd() * 1.5;
                        }
                    }
                }
            }

            // Застосування руху з поосьовою колізією (ковзання вздовж дерев, каменів, води)
            let old_x = m.x;
            let old_y = m.y;
            m.px = old_x;
            m.py = old_y;

            if m.vx.abs() > 0.001 || m.vy.abs() > 0.001 {
                let nx = (m.x + m.vx * dt).clamp(1.0, (W - 2) as f32);
                let ny = (m.y + m.vy * dt).clamp(1.0, (W - 2) as f32);

                let check_x = nx + m.vx.signum() * 0.22;
                let check_y = ny + m.vy.signum() * 0.22;

                let was_blocked = is_blocked(map, bridges, houses, m.x, m.y, now);

                let mut moved_x = false;
                if !is_blocked(map, bridges, houses, check_x, m.y, now)
                    || (!is_blocked(map, bridges, houses, nx, m.y, now) && was_blocked)
                {
                    m.x = nx;
                    moved_x = true;
                }

                let mut moved_y = false;
                if !is_blocked(map, bridges, houses, m.x, check_y, now)
                    || (!is_blocked(map, bridges, houses, m.x, ny, now) && was_blocked)
                {
                    m.y = ny;
                    moved_y = true;
                }

                if !moved_x && !moved_y {
                    m.vx = 0.0;
                    m.vy = 0.0;
                    m.t = 0.5;
                }
            }

            // Оновлення прапорців анімації та напрямку погляду
            let moved_dist = (m.x - old_x).hypot(m.y - old_y);
            m.mv = moved_dist > 0.0005;
            if m.vx < -0.05 {
                m.fl = true;
            } else if m.vx > 0.05 {
                m.fl = false;
            }
        }

        // Застосувати шкоду від питомця ворожим вовкам
        for (target_idx, dmg) in pet_hits {
            if let Some(target) = self.mobs.get_mut(target_idx) {
                target.hp -= dmg;
                if target.hp <= 0.0 {
                    target.dead = now;
                }
            }
        }

        // Атака вовків і ведмедів
        // Bug #51: wolves only attack at night, bears/wolves don't attack near fire
        let mut wolf_bites = 0.0;
        let mut bear_attacks = 0.0;
        for m in &self.mobs {
            if m.dead == 0.0 && now > m.calm {
                let dx = self.x - m.x;
                let dy = self.y - m.y;
                let dist = dx * dx + dy * dy;
                let near_fire = fires_positions.iter().any(|f| {
                    (f.0 - m.x).powi(2) + (f.1 - m.y).powi(2) < 64.0
                });

                if m.kind == MobKind::Wolf && night_level > 0.2 && !near_fire && dist < 1.0 {
                    wolf_bites += 0.8 * dt;
                } else if m.kind == MobKind::Bear && !near_fire && dist < 1.5 {
                    bear_attacks += 2.0 * dt;
                }
            }
        }

        if wolf_bites > 0.0 {
            self.hp -= wolf_bites;
            self.say(texts.wolf_bites, now);
            // Ефект крові при атаці
            self.particles.emit_burst(player_x, player_y, ParticleType::Blood);
        }

        if bear_attacks > 0.0 {
            self.hp -= bear_attacks;
            self.say(texts.bear_attacks, now);
            // Більше крові при атаці ведмедя
            self.particles.emit_burst(player_x, player_y, ParticleType::Blood);
            self.particles.emit_burst(player_x, player_y, ParticleType::Blood);
        }

        // Видалити мертвих тварин (після 120 секунд)
        // Bug #52: reset has_pet when pet dies/removed
        let mut pet_removed = false;
        let mut remove = vec![false; self.mobs.len()];
        for (k, m) in self.mobs.iter().enumerate() {
            if m.dead > 0.0 && now - m.dead > 120.0 {
                remove[k] = true;
                if m.kind == MobKind::Pet { pet_removed = true; }
            }
        }
        if pet_removed { self.has_pet = false; }
        let mut k = 0;
        self.mobs.retain(|_| {
            k += 1;
            !remove[k - 1]
        });
    }
}

