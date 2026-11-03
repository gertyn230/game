//! Система звуків та музики
use macroquad::audio::{load_sound, play_sound, play_sound_once, Sound, PlaySoundParams};
use std::collections::HashMap;

pub struct AudioManager {
    pub sounds: HashMap<String, Sound>,
    pub music: Option<Sound>,
    pub music_volume: f32,
    pub sound_volume: f32,
    pub music_enabled: bool,
    pub sounds_enabled: bool,
}

impl AudioManager {
    pub fn new() -> Self {
        AudioManager {
            sounds: HashMap::new(),
            music: None,
            music_volume: 0.5,
            sound_volume: 0.7,
            music_enabled: true,
            sounds_enabled: true,
        }
    }

    /// Завантажити всі звуки (викликати при старті)
    pub async fn load_all(&mut self) {
        // UI звуки
        self.load_sound_if_exists("button_click", "assets/sounds/ui/button_click.ogg").await;
        self.load_sound_if_exists("button_hover", "assets/sounds/ui/button_hover.ogg").await;
        self.load_sound_if_exists("menu_open", "assets/sounds/ui/menu_open.ogg").await;
        self.load_sound_if_exists("menu_close", "assets/sounds/ui/menu_close.ogg").await;

        // Звуки гравця
        self.load_sound_if_exists("footstep_grass", "assets/sounds/player/footstep_grass.ogg").await;
        self.load_sound_if_exists("footstep_dirt", "assets/sounds/player/footstep_dirt.ogg").await;
        self.load_sound_if_exists("pickup_item", "assets/sounds/player/pickup_item.ogg").await;
        self.load_sound_if_exists("eat", "assets/sounds/player/eat.ogg").await;
        self.load_sound_if_exists("drink", "assets/sounds/player/drink.ogg").await;

        // Звуки дій
        self.load_sound_if_exists("dig_dirt", "assets/sounds/actions/dig_dirt.ogg").await;
        self.load_sound_if_exists("chop_tree", "assets/sounds/actions/chop_tree.ogg").await;
        self.load_sound_if_exists("mine_stone", "assets/sounds/actions/mine_stone.ogg").await;
        self.load_sound_if_exists("craft_item", "assets/sounds/actions/craft_item.ogg").await;

        // Звуки оточення
        self.load_sound_if_exists("wind", "assets/sounds/ambient/wind.ogg").await;
        self.load_sound_if_exists("birds", "assets/sounds/ambient/birds.ogg").await;
        self.load_sound_if_exists("night_crickets", "assets/sounds/ambient/night_crickets.ogg").await;

        // Фонова музика для меню
        self.load_music_if_exists("assets/music/menu_music.ogg").await;
    }

    async fn load_sound_if_exists(&mut self, name: &str, path: &str) {
        match load_sound(path).await {
            Ok(sound) => {
                self.sounds.insert(name.to_string(), sound);
            }
            Err(_) => {
                // Файл не знайдено - нічого страшного, просто не буде звуку
            }
        }
    }

    async fn load_music_if_exists(&mut self, path: &str) {
        match load_sound(path).await {
            Ok(sound) => {
                self.music = Some(sound);
            }
            Err(_) => {
                // Музика не знайдена
            }
        }
    }

    /// Програти звук
    pub fn play(&self, name: &str) {
        if !self.sounds_enabled {
            return;
        }
        if let Some(sound) = self.sounds.get(name) {
            play_sound_once(sound);
            macroquad::audio::set_sound_volume(sound, self.sound_volume);
        }
    }

    /// Запустити фонову музику (loop)
    pub fn play_music(&self) {
        if !self.music_enabled {
            return;
        }
        if let Some(ref music) = self.music {
            play_sound(
                music,
                PlaySoundParams {
                    looped: true,
                    volume: self.music_volume,
                },
            );
        }
    }

    /// Зупинити музику
    pub fn stop_music(&self) {
        if let Some(ref music) = self.music {
            macroquad::audio::stop_sound(music);
        }
    }

    /// Оновити налаштування
    pub fn update_settings(&mut self, music_enabled: bool, sounds_enabled: bool, music_vol: f32, sound_vol: f32) {
        let was_music_enabled = self.music_enabled;

        self.music_enabled = music_enabled;
        self.sounds_enabled = sounds_enabled;
        self.music_volume = music_vol;
        self.sound_volume = sound_vol;

        // Якщо музику вимкнули - зупинити
        if was_music_enabled && !music_enabled {
            self.stop_music();
        }
        // Якщо увімкнули - запустити
        else if !was_music_enabled && music_enabled {
            self.play_music();
        }
        // Якщо просто змінили гучність - оновити
        else if music_enabled {
            if let Some(ref music) = self.music {
                macroquad::audio::set_sound_volume(music, music_vol);
            }
        }
    }
}
