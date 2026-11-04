//! Крафт та рецепти

use crate::game::{Game, MobKind};
use crate::data::{RECIPES, Recipe, Out, It, Tl, TL_DUR};

impl Game {
    /// Створити предмет за рецептом
    pub fn craft(&mut self, i: usize, now: f64) {
        let Some(r) = RECIPES.get(i) else { return };
        if !self.can_craft(r, now) {
            return;
        }
        for (it, n) in r.cost {
            self.inv[*it as usize] -= n;
        }
        match r.out {
            Out::Item(it, n) => self.inv[it as usize] += n,
            Out::Tool(t) => {
                self.tools[t as usize] = TL_DUR[t as usize];
                self.made += 1;
            }
            Out::Fire => self.fires.push((self.x, self.y, now + 240.0)),
            Out::Furnace => self.furn.push((self.x, self.y)),
            Out::House => self.houses += 1,
        }
        self.say(&format!("Готово: {}", r.name), now);
    }

    /// Список доступних рецептів, відсортованих за важливістю
    pub fn quick_list(&self, now: f64) -> Vec<usize> {
        let nt = self.night() > 0.2;
        let wolf = self.mobs.iter().any(|m| m.kind == MobKind::Wolf && m.dead == 0.0 && ((m.x - self.x).powi(2) + (m.y - self.y).powi(2)).sqrt() < 12.0);
        let mut v: Vec<(i32, usize)> = RECIPES
            .iter()
            .enumerate()
            .filter(|(_, r)| self.can_craft(r, now))
            .map(|(i, r)| {
                let mut p = i as i32;
                match r.out {
                    Out::Fire if nt || wolf => p -= 100,
                    Out::Item(It::Roast | It::RFish | It::Meal, _) if self.hunger < 55.0 => p -= 60,
                    _ => {}
                }
                (p, i)
            })
            .collect();
        v.sort_by_key(|x| x.0);
        v.into_iter().map(|x| x.1).collect()
    }

    /// Чи можна зараз створити цей рецепт
    pub fn can_craft(&self, r: &Recipe, now: f64) -> bool {
        if r.fire && !self.fires.iter().any(|f| f.2 > now && (f.0 - self.x).powi(2) + (f.1 - self.y).powi(2) < 4.0) {
            return false;
        }
        if r.furnace && !self.furn.iter().any(|f| (f.0 - self.x).powi(2) + (f.1 - self.y).powi(2) < 4.0) {
            return false;
        }
        for (it, n) in r.cost {
            if self.inv[*it as usize] < *n {
                return false;
            }
        }
        true
    }
}
