//! Налаштування гри: мова, звуки, клавіші
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Language {
    Ukrainian,
    English,
    Russian,
}

impl Language {
    pub fn code(&self) -> &str {
        match self {
            Language::Ukrainian => "uk",
            Language::English => "en",
            Language::Russian => "ru",
        }
    }

    pub fn next(&self) -> Language {
        match self {
            Language::Ukrainian => Language::English,
            Language::English => Language::Russian,
            Language::Russian => Language::Ukrainian,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum ServerType {
    Server1,
    Server2,
}

impl Default for ServerType {
    fn default() -> Self {
        ServerType::Server1
    }
}

impl ServerType {
    pub fn url(&self) -> &str {
        "wss://api.pixel-village.online/ws"
    }

    pub fn next(&self) -> ServerType {
        ServerType::Server1
    }

    pub fn name(&self) -> &str {
        "Основний (pixel-village.online)"
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct KeyBindings {
    pub move_up: String,
    pub move_down: String,
    pub move_left: String,
    pub move_right: String,
    pub action: String,
    pub inventory: String,
    pub craft: String,
    pub eat: String,
    pub drink: String,
    pub dig: String,
    pub menu: String,
    pub quick1: String,
    pub quick2: String,
    pub quick3: String,
    pub quick4: String,
    pub quick5: String,
}

impl Default for KeyBindings {
    fn default() -> Self {
        KeyBindings {
            move_up: "W".to_string(),
            move_down: "S".to_string(),
            move_left: "A".to_string(),
            move_right: "D".to_string(),
            action: "Space".to_string(),
            inventory: "I".to_string(),
            craft: "C".to_string(),
            eat: "F".to_string(),
            drink: "R".to_string(),
            dig: "G".to_string(),
            menu: "M".to_string(),
            quick1: "1".to_string(),
            quick2: "2".to_string(),
            quick3: "3".to_string(),
            quick4: "4".to_string(),
            quick5: "5".to_string(),
        }
    }
}

pub fn parse_key(name: &str) -> Option<macroquad::prelude::KeyCode> {
    use macroquad::prelude::KeyCode::*;
    match name.trim().to_uppercase().as_str() {
        "A" => Some(A),
        "B" => Some(B),
        "C" => Some(C),
        "D" => Some(D),
        "E" => Some(E),
        "F" => Some(F),
        "G" => Some(G),
        "H" => Some(H),
        "I" => Some(I),
        "J" => Some(J),
        "K" => Some(K),
        "L" => Some(L),
        "M" => Some(M),
        "N" => Some(N),
        "O" => Some(O),
        "P" => Some(P),
        "Q" => Some(Q),
        "R" => Some(R),
        "S" => Some(S),
        "T" => Some(T),
        "U" => Some(U),
        "V" => Some(V),
        "W" => Some(W),
        "X" => Some(X),
        "Y" => Some(Y),
        "Z" => Some(Z),
        "0" => Some(Key0),
        "1" => Some(Key1),
        "2" => Some(Key2),
        "3" => Some(Key3),
        "4" => Some(Key4),
        "5" => Some(Key5),
        "6" => Some(Key6),
        "7" => Some(Key7),
        "8" => Some(Key8),
        "9" => Some(Key9),
        "SPACE" => Some(Space),
        "TAB" => Some(Tab),
        "ESCAPE" | "ESC" => Some(Escape),
        "ENTER" => Some(Enter),
        "LEFT" => Some(Left),
        "RIGHT" => Some(Right),
        "UP" => Some(Up),
        "DOWN" => Some(Down),
        "LSHIFT" | "SHIFT" => Some(LeftShift),
        "RSHIFT" => Some(RightShift),
        "LCTRL" | "CTRL" => Some(LeftControl),
        "RCTRL" => Some(RightControl),
        "LALT" | "ALT" => Some(LeftAlt),
        "RALT" => Some(RightAlt),
        "F1" => Some(F1),
        "F2" => Some(F2),
        "F3" => Some(F3),
        "F4" => Some(F4),
        "F5" => Some(F5),
        "F6" => Some(F6),
        "F7" => Some(F7),
        "F8" => Some(F8),
        "F9" => Some(F9),
        "F10" => Some(F10),
        "F11" => Some(F11),
        "F12" => Some(F12),
        "BACKSPACE" => Some(Backspace),
        "DELETE" | "DEL" => Some(Delete),
        _ => None,
    }
}

pub fn key_name(key: macroquad::prelude::KeyCode) -> &'static str {
    use macroquad::prelude::KeyCode::*;
    match key {
        A => "A",
        B => "B",
        C => "C",
        D => "D",
        E => "E",
        F => "F",
        G => "G",
        H => "H",
        I => "I",
        J => "J",
        K => "K",
        L => "L",
        M => "M",
        N => "N",
        O => "O",
        P => "P",
        Q => "Q",
        R => "R",
        S => "S",
        T => "T",
        U => "U",
        V => "V",
        W => "W",
        X => "X",
        Y => "Y",
        Z => "Z",
        Key0 => "0",
        Key1 => "1",
        Key2 => "2",
        Key3 => "3",
        Key4 => "4",
        Key5 => "5",
        Key6 => "6",
        Key7 => "7",
        Key8 => "8",
        Key9 => "9",
        Space => "Space",
        Tab => "Tab",
        Escape => "Esc",
        Enter => "Enter",
        Left => "Left",
        Right => "Right",
        Up => "Up",
        Down => "Down",
        LeftShift | RightShift => "Shift",
        LeftControl | RightControl => "Ctrl",
        LeftAlt | RightAlt => "Alt",
        F1 => "F1",
        F2 => "F2",
        F3 => "F3",
        F4 => "F4",
        F5 => "F5",
        F6 => "F6",
        F7 => "F7",
        F8 => "F8",
        F9 => "F9",
        F10 => "F10",
        F11 => "F11",
        F12 => "F12",
        Backspace => "Backspace",
        Delete => "Delete",
        _ => "Key",
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RebindAction {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Action,
    Inventory,
    Craft,
    Eat,
    Drink,
    Dig,
    Menu,
}

impl RebindAction {
    pub const ALL: [RebindAction; 11] = [
        RebindAction::Inventory,
        RebindAction::Craft,
        RebindAction::Action,
        RebindAction::Eat,
        RebindAction::Drink,
        RebindAction::Dig,
        RebindAction::Menu,
        RebindAction::MoveUp,
        RebindAction::MoveLeft,
        RebindAction::MoveDown,
        RebindAction::MoveRight,
    ];

    pub fn icon(&self) -> &'static str {
        match self {
            RebindAction::MoveUp => "⬆️",
            RebindAction::MoveDown => "⬇️",
            RebindAction::MoveLeft => "⬅️",
            RebindAction::MoveRight => "➡️",
            RebindAction::Action => "⚡",
            RebindAction::Inventory => "🎒",
            RebindAction::Craft => "🔨",
            RebindAction::Eat => "🍴",
            RebindAction::Drink => "🥤",
            RebindAction::Dig => "⛏️",
            RebindAction::Menu => "⚙️",
        }
    }
}

impl KeyBindings {
    pub fn get_key_for_action(&self, action: RebindAction) -> &str {
        match action {
            RebindAction::MoveUp => &self.move_up,
            RebindAction::MoveDown => &self.move_down,
            RebindAction::MoveLeft => &self.move_left,
            RebindAction::MoveRight => &self.move_right,
            RebindAction::Action => &self.action,
            RebindAction::Inventory => &self.inventory,
            RebindAction::Craft => &self.craft,
            RebindAction::Eat => &self.eat,
            RebindAction::Drink => &self.drink,
            RebindAction::Dig => &self.dig,
            RebindAction::Menu => &self.menu,
        }
    }

    pub fn set_key_for_action(&mut self, action: RebindAction, key: String) {
        match action {
            RebindAction::MoveUp => self.move_up = key,
            RebindAction::MoveDown => self.move_down = key,
            RebindAction::MoveLeft => self.move_left = key,
            RebindAction::MoveRight => self.move_right = key,
            RebindAction::Action => self.action = key,
            RebindAction::Inventory => self.inventory = key,
            RebindAction::Craft => self.craft = key,
            RebindAction::Eat => self.eat = key,
            RebindAction::Drink => self.drink = key,
            RebindAction::Dig => self.dig = key,
            RebindAction::Menu => self.menu = key,
        }
    }

    pub fn is_inventory_pressed(&self) -> bool {
        parse_key(&self.inventory).map_or(false, macroquad::prelude::is_key_pressed)
    }

    pub fn is_craft_pressed(&self) -> bool {
        parse_key(&self.craft).map_or(false, macroquad::prelude::is_key_pressed)
    }

    pub fn is_menu_pressed(&self) -> bool {
        parse_key(&self.menu).map_or(false, macroquad::prelude::is_key_pressed)
    }

    pub fn is_eat_pressed(&self) -> bool {
        parse_key(&self.eat).map_or(false, macroquad::prelude::is_key_pressed)
    }

    pub fn is_drink_pressed(&self) -> bool {
        parse_key(&self.drink).map_or(false, macroquad::prelude::is_key_pressed)
    }

    pub fn is_dig_pressed(&self) -> bool {
        parse_key(&self.dig).map_or(false, macroquad::prelude::is_key_pressed)
    }

    pub fn is_action_down(&self) -> bool {
        parse_key(&self.action).map_or(false, macroquad::prelude::is_key_down)
    }

    pub fn is_up_down(&self) -> bool {
        parse_key(&self.move_up).map_or(false, macroquad::prelude::is_key_down)
    }

    pub fn is_down_down(&self) -> bool {
        parse_key(&self.move_down).map_or(false, macroquad::prelude::is_key_down)
    }

    pub fn is_left_down(&self) -> bool {
        parse_key(&self.move_left).map_or(false, macroquad::prelude::is_key_down)
    }

    pub fn is_right_down(&self) -> bool {
        parse_key(&self.move_right).map_or(false, macroquad::prelude::is_key_down)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Settings {
    pub language: Language,
    pub music_volume: f32,
    pub sound_volume: f32,
    pub music_enabled: bool,
    pub sounds_enabled: bool,
    pub keybindings: KeyBindings,
    #[serde(default)]
    pub show_debug: bool,
    #[serde(default)]
    pub server_type: ServerType,
    #[serde(default)]
    pub mobile_mode: bool,
}

impl Settings {
    pub fn is_mobile(&self) -> bool {
        cfg!(any(target_os = "android", target_os = "ios")) || self.mobile_mode
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            language: Language::Ukrainian,
            music_volume: 0.5,
            sound_volume: 0.7,
            music_enabled: true,
            sounds_enabled: true,
            keybindings: KeyBindings::default(),
            show_debug: false,
            server_type: ServerType::Server1,
            mobile_mode: false,
        }
    }
}

fn settings_dir() -> PathBuf {
    if let Ok(d) = std::env::var("PV_SAVE_DIR") {
        return PathBuf::from(d);
    }
    #[cfg(target_os = "android")]
    {
        PathBuf::from("/data/data/com.gertyn.pixelvillage/files")
    }
    #[cfg(not(target_os = "android"))]
    {
        if let Ok(a) = std::env::var("APPDATA") {
            return PathBuf::from(a).join("PixelVillage");
        }
        if let Ok(h) = std::env::var("HOME") {
            return PathBuf::from(h).join(".pixel_village");
        }
        PathBuf::from(".")
    }
}

pub fn load_settings() -> Settings {
    let path = settings_dir().join("settings.json");
    if let Ok(s) = std::fs::read_to_string(&path) {
        if let Ok(mut settings) = serde_json::from_str::<Settings>(&s) {
            if settings.server_type != ServerType::Server1 {
                settings.server_type = ServerType::Server1;
                save_settings(&settings);
            }
            return settings;
        }
    }
    Settings::default()
}

pub fn save_settings(settings: &Settings) {
    let dir = settings_dir();
    let _ = std::fs::create_dir_all(&dir);
    if let Ok(json) = serde_json::to_string_pretty(settings) {
        let _ = std::fs::write(dir.join("settings.json"), json);
    }
}
