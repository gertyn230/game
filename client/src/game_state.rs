//! Обгортка для управління станами гри з меню
use crate::menu::{Menu, MenuAction, MenuState};
use crate::settings::{Settings, save_settings};
use crate::i18n::get_texts;
use macroquad::prelude::*;

pub struct GameStateManager {
    pub menu: Menu,
    pub in_game: bool,
    pub should_quit: bool,
}

impl GameStateManager {
    pub fn new(has_save: bool) -> Self {
        let mut menu = Menu::new();
        menu.has_save = has_save;
        GameStateManager {
            menu,
            in_game: false,
            should_quit: false,
        }
    }

    pub fn update(&mut self, font: &Font, settings: &mut Settings) -> bool {
        let texts = get_texts(&settings.language);

        match self.menu.state {
            MenuState::Loading => {
                self.menu.draw_loading(font, texts);
                false
            }
            MenuState::MainMenu => {
                if let Some(action) = self.menu.draw_main_menu(font, texts) {
                    match action {
                        MenuAction::Play | MenuAction::Continue => {
                            self.menu.state = MenuState::InGame;
                            self.in_game = true;
                            return true;
                        }
                        MenuAction::NewGame => {
                            self.menu.state = MenuState::InGame;
                            self.in_game = true;
                            return true;
                        }
                        MenuAction::OpenSettings => {
                            self.menu.state = MenuState::Settings;
                        }
                        MenuAction::Quit => {
                            self.should_quit = true;
                        }
                        _ => {}
                    }
                }
                false
            }
            MenuState::Settings => {
                if let Some(action) = self.menu.draw_settings_menu(font, texts, settings) {
                    if action == MenuAction::CloseSettings {
                        self.menu.state = MenuState::MainMenu;
                        save_settings(settings);
                    }
                }
                false
            }
            MenuState::InGame => true,
        }
    }

    pub fn show_menu(&mut self) {
        self.menu.state = MenuState::MainMenu;
        self.in_game = false;
    }

    pub fn update_loading(&mut self, progress: f32, message: &str) {
        self.menu.update_loading(progress, message);
    }

    pub fn finish_loading(&mut self) {
        self.menu.finish_loading();
    }
}
