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
        // Звуки дій
        self.load_sound_if_exists("hit", "assets/sounds/hit.ogg").await;
        self.load_sound_if_exists("craft", "assets/sounds/craft.ogg").await;
        self.load_sound_if_exists("eat", "assets/sounds/eat.ogg").await;
        self.load_sound_if_exists("drink", "assets/sounds/drink.ogg").await;
        self.load_sound_if_exists("pickup", "assets/sounds/pickup.ogg").await;
        self.load_sound_if_exists("step", "assets/sounds/step.ogg").await;
        self.load_sound_if_exists("death", "assets/sounds/death.ogg").await;

        // Фонова музика
        self.load_music_if_exists("assets/music/background.ogg").await;
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
