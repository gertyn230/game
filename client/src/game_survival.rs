//! Виживання: їжа, вода, голод, спрага

use crate::game::Game;
use crate::data::It;
use crate::i18n::Texts;

impl Game {
    /// Їсти їжу з інвентаря
    pub fn eat(&mut self, now: f64, texts: &Texts) {
        fn h(g: &mut Game, hunger: f32, en: f32) {
            g.hunger = (g.hunger + hunger).min(100.0);
            g.energy = (g.energy + en).min(100.0);
        }
        if self.inv[It::Roast as usize] > 0 {
            self.inv[It::Roast as usize] -= 1;
            h(self, 70.0, 25.0);
        } else if self.inv[It::RFish as usize] > 0 {
            self.inv[It::RFish as usize] -= 1;
            h(self, 65.0, 20.0);
        } else if self.inv[It::Meal as usize] > 0 {
            self.inv[It::Meal as usize] -= 1;
            h(self, 60.0, 15.0);
        } else if self.inv[It::Food as usize] > 0 {
            self.inv[It::Food as usize] -= 1;
            h(self, 20.0, 8.0);
        } else if self.inv[It::Meat as usize] > 0 {
            self.inv[It::Meat as usize] -= 1;
            h(self, 15.0, 5.0);
            self.hp -= 8.0;
            self.say(texts.raw_meat_bad, now);
        } else if self.inv[It::Fish as usize] > 0 {
            self.inv[It::Fish as usize] -= 1;
            h(self, 12.0, 5.0);
            self.hp -= 4.0;
            self.say(texts.raw_fish_bad, now);
        }
    }

    /// Пити воду
    pub fn drink(&mut self, now: f64, texts: &Texts) {
        if self.fish.ph != 0 {
            return;
        }
        if self.near_water() {
            self.thirst = (self.thirst + 15.0).min(100.0);
            self.last_hit = now;
            self.energy -= 0.3;
            self.say(texts.drinking_water, now);
        } else {
            self.say(texts.no_water_nearby, now);
        }
    }
}
