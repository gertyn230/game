//! Общий протокол между клиентом и сервером.
use serde::{Deserialize, Serialize};

pub const WORLD_SIZE: f32 = 40.0;
pub const MAX_NAME_CHARS: usize = 16;
pub const RESPAWN_SECS: u32 = 480;
pub const NODE_COUNT_MAX: u32 = 1600;
pub const ITEM_COUNT: u8 = 19;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerState {
    pub id: u32,
    pub name: String,
    pub x: f32,
    pub y: f32,
    /// 0 вниз, 1 вверх, 2 влево, 3 вправо
    pub face: u8,
    pub moving: bool,
    pub color: u8,
    pub boat: bool,
    pub coat: bool,
    /// 0 ничего, 1 копьё, 2 топор, 3 кирка
    pub held: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Offer {
    pub id: u32,
    pub seller: u32,
    pub seller_name: String,
    pub item: u8,
    pub qty: u32,
    pub price: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "t")]
pub enum ClientMsg {
    Join { name: String },
    Move { x: f32, y: f32, face: u8, moving: bool, boat: bool, coat: bool, held: u8 },
    /// Ресурс на карте добыт (индекс клетки y*40+x).
    Dead { idx: u32 },
    Stats { score: u32, big: f32, fish: u32 },
    Post { item: u8, qty: u32, price: u32 },
    Buy { id: u32 },
    Withdraw { id: u32 },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "t")]
pub enum ServerMsg {
    Welcome { id: u32 },
    Snapshot { players: Vec<PlayerState> },
    Dead { idx: u32, left: u32 },
    World { dead: Vec<(u32, u32)> },
    Market { offers: Vec<Offer> },
    Board { score: Vec<(String, u32)>, fishers: Vec<(String, f32, u32)> },
    Posted { offer: Offer },
    Bought { offer: Offer },
    Sold { id: u32, gold: u32 },
    Withdrawn { offer: Offer },
    Err { text: String },
}
