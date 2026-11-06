//! Карта мира: вода, река, пляжи, биомы и ресурсы.
pub const W: usize = 80;
pub const RESPAWN_SECS: f64 = 480.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Tree,
    Rock,
    Bush,
    Stick,
    Pebble,
    Fiber,
    Coal,
    CuOre,
    FeOre,
    BerryBush,  // Ягідний кущ
    Mushroom,   // Гриби
    Herb,       // Лікарські трави
    AgOre,      // Срібна руда
    AuOre,      // Золота руда
}

impl Kind {
    pub fn blocks(self) -> bool {
        matches!(self, Kind::Tree | Kind::Rock | Kind::Coal | Kind::CuOre | Kind::FeOre | Kind::AgOre | Kind::AuOre)
    }
    pub fn is_ore(self) -> bool {
        matches!(self, Kind::Coal | Kind::CuOre | Kind::FeOre | Kind::AgOre | Kind::AuOre)
    }
}

pub struct Node {
    pub x: i32,
    pub y: i32,
    pub kind: Kind,
    pub hp: u8,
    pub dead_until: f64,
}

pub const GRASS: u8 = 0;
pub const WATER: u8 = 1;
pub const SAND: u8 = 2;
pub const FORD: u8 = 3;

pub struct Map {
    pub terr: Vec<u8>,
    pub nodes: Vec<Node>,
    idx: Vec<i32>,
}

struct Rng(u32);
impl Rng {
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 >> 8) as f32 / 16_777_216.0
    }
}

fn river_center(y: f32) -> f32 {
    18.0 + 10.0 * (y * 0.15).sin() + 4.0 * (y * 0.4).sin()
}

fn raw_water(x: i32, y: i32) -> bool {
    if x < 0 || y < 0 || x >= W as i32 || y >= W as i32 {
        return false;
    }
    let (xf, yf) = (x as f32, y as f32);
    // Річка ширша, озеро більше
    (xf + 0.5 - river_center(yf)).abs() < 2.0 || (xf - 62.0).powi(2) + (yf - 20.0).powi(2) < 25.0
}

/// 0 луга, 1 лес, 2 каменистое плато, 3 болото, 4 пустеля
pub fn biome(x: i32, y: i32) -> u8 {
    let (xf, yf) = (x as f32, y as f32);
    let n = (xf * 0.21 + 1.0).sin() + (yf * 0.19).cos() + ((xf + yf) * 0.11).sin();
    let n2 = (xf * 0.15).cos() + (yf * 0.17).sin() + ((xf - yf) * 0.13).cos();

    if n > 1.5 {
        4  // Пустеля (рідкісна)
    } else if n > 1.0 {
        2  // Каменисте плато
    } else if n < -1.5 && n2 < 0.0 {
        3  // Болото (поруч з водою)
    } else if n < -1.0 {
        1  // Ліс
    } else {
        0  // Луга
    }
}

impl Map {
    pub fn new() -> Map {
        let mut terr = vec![GRASS; W * W];
        for y in 0..W as i32 {
            for x in 0..W as i32 {
                let v = if raw_water(x, y) {
                    if y % 11 < 2 && (x as f32 + 0.5 - river_center(y as f32)).abs() < 1.6 {
                        FORD
                    } else {
                        WATER
                    }
                } else {
                    let mut near = false;
                    for a in -1..=1 {
                        for b in -1..=1 {
                            if raw_water(x + a, y + b) {
                                near = true;
                            }
                        }
                    }
                    if near {
                        SAND
                    } else {
                        GRASS
                    }
                };
                terr[y as usize * W + x as usize] = v;
            }
        }

        let mut rng = Rng(0x9E37_79B9);
        let mut nodes = Vec::new();
        let mut idx = vec![-1i32; W * W];
        for y in 0..W as i32 {
            for x in 0..W as i32 {
                let t = terr[y as usize * W + x as usize];
                if t == WATER || t == FORD || ((x - 40).abs() < 2 && (y - 40).abs() < 2) {
                    continue;
                }
                let r = rng.next();
                let kind = match biome(x, y) {
                    1 => match r {
                        // Ліс - багато дерев, ягід, грибів
                        r if r < 0.20 => Some(Kind::Tree),
                        r if r < 0.24 => Some(Kind::Bush),
                        r if r < 0.28 => Some(Kind::Stick),
                        r if r < 0.31 => Some(Kind::Fiber),
                        r if r < 0.35 => Some(Kind::BerryBush),
                        r if r < 0.37 => Some(Kind::Mushroom),
                        r if r < 0.39 => Some(Kind::Herb),
                        _ => None,
                    },
                    2 => match r {
                        // Каменисте плато - камені, руди
                        r if r < 0.18 => Some(Kind::Rock),
                        r if r < 0.22 => Some(Kind::Pebble),
                        r if r < 0.24 => Some(Kind::Stick),
                        r if r < 0.26 => Some(Kind::Herb),
                        _ => None,
                    },
                    3 => match r {
                        // Болото - багато трав, грибів, мало дерев
                        r if r < 0.10 => Some(Kind::Tree),
                        r if r < 0.15 => Some(Kind::Stick),
                        r if r < 0.20 => Some(Kind::Fiber),
                        r if r < 0.30 => Some(Kind::Mushroom),  // Багато грибів!
                        r if r < 0.38 => Some(Kind::Herb),      // Багато трав
                        r if r < 0.40 => Some(Kind::BerryBush),
                        _ => None,
                    },
                    4 => match r {
                        // Пустеля - камені, мало рослин
                        r if r < 0.12 => Some(Kind::Rock),
                        r if r < 0.16 => Some(Kind::Pebble),
                        r if r < 0.18 => Some(Kind::Stick),
                        r if r < 0.20 => Some(Kind::Herb),  // Рідкісні трави
                        _ => None,
                    },
                    _ => match r {
                        // Луга - збалансовано
                        r if r < 0.08 => Some(Kind::Tree),
                        r if r < 0.12 => Some(Kind::Rock),
                        r if r < 0.16 => Some(Kind::Bush),
                        r if r < 0.20 => Some(Kind::Stick),
                        r if r < 0.24 => Some(Kind::Pebble),
                        r if r < 0.28 => Some(Kind::Fiber),
                        r if r < 0.31 => Some(Kind::BerryBush),
                        r if r < 0.33 => Some(Kind::Mushroom),
                        r if r < 0.35 => Some(Kind::Herb),
                        _ => None,
                    },
                };
                if let Some(kind) = kind {
                    idx[y as usize * W + x as usize] = nodes.len() as i32;
                    nodes.push(Node { x, y, kind, hp: Map::max_hp(kind), dead_until: 0.0 });
                }
            }
        }
        // Руды: в основном на каменистом плато, редко на лугах.
        let mut rng = Rng(0x1357_9BDF);
        for y in 0..W as i32 {
            for x in 0..W as i32 {
                let i = y as usize * W + x as usize;
                if idx[i] >= 0 || terr[i] != GRASS || ((x - 40).abs() < 3 && (y - 40).abs() < 3) {
                    continue;
                }
                let (r, u) = (rng.next(), rng.next());
                let q = match biome(x, y) {
                    2 => 0.16,  // Каменисте плато - багато руд
                    4 => 0.12,  // Пустеля - теж багато руд
                    0 => 0.03,  // Луга - мало руд
                    _ => 0.01,  // Ліс та болото - дуже мало
                };
                if r < q {
                    let kind = if u < 0.4 {
                        Kind::Coal
                    } else if u < 0.65 {
                        Kind::CuOre
                    } else if u < 0.85 {
                        Kind::FeOre
                    } else if u < 0.95 {
                        Kind::AgOre  // Срібло рідкісніше
                    } else {
                        Kind::AuOre  // Золото найрідкісніше
                    };
                    idx[i] = nodes.len() as i32;
                    nodes.push(Node { x, y, kind, hp: 4, dead_until: 0.0 });
                }
            }
        }
        Map { terr, nodes, idx }
    }

    /// Применить «ресурс добыт» от сервера: индекс клетки и оставшиеся секунды.
    pub fn set_dead_cell(&mut self, cell: u32, until: f64) {
        if (cell as usize) < W * W {
            let i = self.idx[cell as usize];
            if i >= 0 {
                let n = &mut self.nodes[i as usize];
                n.dead_until = until;
                n.hp = 0;
            }
        }
    }

    pub fn max_hp(kind: Kind) -> u8 {
        match kind {
            Kind::Tree | Kind::Rock => 3,
            Kind::Coal | Kind::CuOre | Kind::FeOre | Kind::AgOre | Kind::AuOre => 4,
            Kind::BerryBush => 2,  // Ягідний кущ трохи міцніший
            _ => 1,
        }
    }

    pub fn terr_at(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x >= W as i32 || y >= W as i32 {
            GRASS
        } else {
            self.terr[y as usize * W + x as usize]
        }
    }

    pub fn node_at(&self, x: i32, y: i32) -> Option<&Node> {
        if x < 0 || y < 0 || x >= W as i32 || y >= W as i32 {
            return None;
        }
        let i = self.idx[y as usize * W + x as usize];
        if i < 0 {
            None
        } else {
            Some(&self.nodes[i as usize])
        }
    }
}
