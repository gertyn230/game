#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod art;
mod audio;
mod data;
mod game;
mod game_state;
mod i18n;
mod menu;
mod settings_menu;
mod net;
mod save;
mod settings;
mod ui;
mod world;
mod updater;
mod discord_rpc;
mod particles;
mod translate_messages;

// Ігрові модулі (розширення для game)
mod game_survival;
mod game_actions;
mod game_fishing;
mod game_mobs;
mod game_crafting;

use std::collections::HashMap;

use macroquad::prelude::*;
use shared::{ClientMsg, DeviceInfo, Offer, PlayerState, ServerMsg};

use art::*;
use audio::*;
use data::*;
use game::*;
use game_state::*;
use i18n::*;
use net::{Net, NetEvent};
use settings::*;
use ui::*;
use world::*;

#[derive(Default, Clone)]
pub struct AccountUiState {
    pub active: Option<shared::AccountData>,
    pub mode: u8, // 0 = login, 1 = register, 2 = change password, 3 = code
    pub active_field: u8, // 0 = email, 1 = password, 2 = username, 3 = new_password, 4 = code
    pub email: String,
    pub password: String,
    pub username: String,
    pub new_password: String,
    pub code: String,
    pub status: Option<String>,
}

const PLAYER_COLORS: [u32; 5] = [0xd94f4f, 0x4a6fa5, 0xc9a227, 0x8e5bb5, 0x2a9d8f];
const CLIENT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn window_conf() -> Conf {
    Conf {
        window_title: "Pixel Village".to_owned(),
        window_width: 1280,
        window_height: 720,
        fullscreen: false,
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Ph {
    Start,
    Move,
    End,
}

fn pointers() -> Vec<(u64, Ph, Vec2)> {
    let mut v = Vec::new();
    for t in touches() {
        let ph = match t.phase {
            TouchPhase::Started => Ph::Start,
            TouchPhase::Ended | TouchPhase::Cancelled => Ph::End,
            _ => Ph::Move,
        };
        v.push((t.id, ph, t.position));
    }
    let (mx, my) = mouse_position();
    let mp = vec2(mx, my);
    if is_mouse_button_pressed(MouseButton::Left) {
        v.push((u64::MAX, Ph::Start, mp));
    } else if is_mouse_button_released(MouseButton::Left) {
        v.push((u64::MAX, Ph::End, mp));
    } else if is_mouse_button_down(MouseButton::Left) {
        v.push((u64::MAX, Ph::Move, mp));
    }
    v
}

struct Other {
    st: PlayerState,
    dx: f32,
    dy: f32,
}

struct FloatingText {
    text: String,
    x: f32,
    y: f32,
    t: f64,
    color: Color,
}

fn spawn_hit_feedback(
    outcome: HitOutcome,
    floating_texts: &mut Vec<FloatingText>,
    now: f64,
    texts: &Texts,
) {
    match outcome {
        HitOutcome::HitNode { destroyed, item, count, x, y, .. } => {
            if destroyed && count > 0 {
                let item_name = texts.item_names[item as usize];
                floating_texts.push(FloatingText {
                    text: format!("+{} {}", count, item_name),
                    x,
                    y: y - 0.2,
                    t: now,
                    color: Color::from_rgba(100, 255, 120, 255),
                });
            }
        }
        HitOutcome::Butchered { x, y, .. } => {
            floating_texts.push(FloatingText {
                text: format!("🍖 {}", texts.corpse_butchered),
                x,
                y: y - 0.5,
                t: now,
                color: Color::from_rgba(255, 200, 100, 255),
            });
        }
        _ => {}
    }
}

fn cost_text(r: &Recipe, texts: &Texts) -> String {
    r.cost.iter().map(|(it, n)| format!("{} ×{}", texts.item_names[*it as usize], n)).collect::<Vec<_>>().join(", ")
}

#[macroquad::main(window_conf)]
async fn main() {
    // Видалити старі backup файли з попередніх оновлень
    updater::Updater::cleanup_old_version();

    simulate_mouse_with_touch(false);
    rand::srand(macroquad::miniquad::date::now() as u64);
    let font = load_ttf_font_from_bytes(include_bytes!("../assets/DejaVuSans.ttf")).expect("шрифт");

    // Завантажуємо налаштування
    let mut settings = load_settings();
    let saved = save::load();

    // Ініціалізуємо систему меню з версією
    let mut game_state = GameStateManager::new(saved.is_some(), CLIENT_VERSION);

    // Завантажуємо фонове зображення меню ПЕРЕД показом
    game_state.menu.load_background().await;

    // Чекаємо один кадр щоб текстура точно завантажилась
    next_frame().await;

    // Показуємо екран завантаження
    for i in 0..20 {
        let progress = (i as f32 / 20.0) * 0.3;
        let texts = get_texts(&settings.language);
        game_state.update_loading(progress, texts.loading_assets);
        game_state.update(&font, &mut settings);
        next_frame().await;
    }

    // Ініціалізуємо аудіо систему
    let mut audio = AudioManager::new();
    audio.load_all().await;
    audio.update_settings(settings.music_enabled, settings.sounds_enabled, settings.music_volume, settings.sound_volume);

    // Ініціалізуємо Discord Rich Presence
    let mut discord_rpc = discord_rpc::DiscordRpc::new();

    for i in 20..40 {
        let progress = 0.3 + ((i - 20) as f32 / 20.0) * 0.4;
        let texts = get_texts(&settings.language);
        game_state.update_loading(progress, texts.loading_world);
        game_state.update(&font, &mut settings);
        next_frame().await;
    }

    if settings.music_enabled {
        audio.play_music();
    }

    let env_name = std::env::var("PV_NAME").ok();
    let mut name = env_name
        .clone()
        .or_else(|| saved.as_ref().map(|s| s.name.clone()).filter(|n| !n.is_empty()))
        .unwrap_or_else(|| format!("Гравець{}", rand::gen_range(100, 999)));

    // Завершуємо завантаження
    for i in 40..50 {
        let progress = 0.7 + ((i - 40) as f32 / 10.0) * 0.3;
        let texts = get_texts(&settings.language);
        game_state.update_loading(progress, texts.connecting);
        game_state.update(&font, &mut settings);
        next_frame().await;
    }

    let net = Net::start(settings.server_type.url().to_string(), name.clone());

    let mut g = Game::new();
    if let Some(s) = &saved {
        g.apply_save(s, get_time());
    }

    game_state.finish_loading();

    // Перевірити оновлення один раз при старті
    game_state.check_updates_once();

    // Первый запуск: спрашиваем имя экранной клавиатурой.
    let mut name_edit: Option<String> = if env_name.is_none() && saved.as_ref().is_none_or(|s| s.name.is_empty()) { Some(name.clone()) } else { None };
    let mut kb_lang = 0usize;
    let mut save_t = 0.0f32;
    prevent_quit();
    let minimap = build_minimap(&g.map);
    let mut ui = Ui::new();
    let my_color = hx(PLAYER_COLORS[(name.chars().count() * 7 + name.len()) % 5]);

    let mut my_id: Option<u32> = None;
    let mut connected = false;
    let mut version_mismatch: Option<String> = None;
    let mut others: HashMap<u32, Other> = HashMap::new();
    let mut online = 0usize;
    let mut market: Vec<Offer> = vec![];
    let mut posted: HashMap<u32, (u8, u32)> = HashMap::new();
    let mut board_score: Vec<(String, u32)> = vec![];
    let mut board_fish: Vec<(String, f32, u32)> = vec![];
    let mut send_t = 0.0f32;
    let mut stats_t = 0.0f32;
    let mut last_sent = (0.0f32, 0.0f32, 0u8, false, false, false, 0u8);
    let mut joy_id: Option<u64> = None;
    let mut origin = Vec2::ZERO;
    let mut cur = Vec2::ZERO;
    let mut act_id: Option<u64> = None;
    let mut scroll_id: Option<(u64, f32)> = None;
    let mut floating_texts: Vec<FloatingText> = Vec::new();
    let mut acc_ui = AccountUiState::default();
    if let Some(creds) = save::load_account_creds() {
        acc_ui.email = creds.email;
        acc_ui.password = creds.password;
        acc_ui.username = creds.username;
    }

    loop {
        let dt = get_frame_time().min(0.05);
        let now = get_time();

        // Перевірка закриття вікна (працює завжди, незалежно від стану гри)
        if is_quit_requested() {
            if acc_ui.active.is_some() {
                net.send(ClientMsg::SaveAccount {
                    gold: g.gold,
                    score: g.score(),
                    fish: g.caught,
                    big_fish: g.biggest,
                    inventory: g.inv.to_vec(),
                    tools: g.tools.to_vec(),
                    houses: g.houses,
                    x: g.x,
                    y: g.y,
                });
            }
            save::store(&snapshot(&g, now, &name, &posted));
            break;
        }

        let is_mobile = settings.is_mobile();
        let texts = get_texts(&settings.language);
        let (sw, sh) = (screen_width(), screen_height());
        let base = sw.min(sh);
        let ts = (base / 11.0).floor().max(24.0);
        let u = ts / 16.0;
        let k = ts / 48.0;
        let (cx, cy) = (sw / 2.0, sh / 2.0);
        let fs = (base * 0.03).max(14.0);
        let pad = fs * 0.5;
        let ring_r = base * 0.09;
        let mut out: Vec<ClientMsg> = vec![];
        let mut cmds: Vec<Cmd> = vec![];

        // Перевірка стану меню
        if !game_state.in_game {
            game_state.update(&font, &mut settings);
            if game_state.should_quit {
                save::store(&snapshot(&g, now, &name, &posted));
                break;
            }
            next_frame().await;
            continue;
        }

        // Обов'язкова авторизація перед грою
        if acc_ui.active.is_none() {
            ui.menu = true;
            ui.tab = 5;
        }

        // ---------- сеть ----------
        while let Ok(ev) = net.in_rx.try_recv() {
            match ev {
                NetEvent::Connected => {
                    connected = true;
                    if !acc_ui.email.is_empty() && !acc_ui.password.is_empty() {
                        net.send(ClientMsg::Login {
                            email: acc_ui.email.clone(),
                            password: acc_ui.password.clone(),
                            device_info: Some(DeviceInfo::detect()),
                        });
                    }
                }
                NetEvent::Disconnected => {
                    if connected {
                        // TODO #2: Market items should be server-authoritative, don't refund on disconnect
                        for (_, (item, qty)) in posted.drain() {
                            if let Some(slot) = g.inv.get_mut(item as usize) {
                                *slot += qty;
                            }
                        }
                    }
                    connected = false;
                    my_id = None;
                    others.clear();
                    market.clear();
                    online = 0;
                }
                NetEvent::Msg(m) => match m {
                    ServerMsg::Welcome { id, version } => {
                        my_id = Some(id);
                        if version != CLIENT_VERSION {
                            version_mismatch = Some(version);
                        }
                    }
                    ServerMsg::Snapshot { players } => {
                        online = players.len();
                        // Update Discord RPC with online count
                        discord_rpc.update(
                            &format!("Online: {}", online),
                            "Playing Pixel Village",
                            Some(online)
                        );
                        let mut seen = Vec::new();
                        for p in players {
                            if Some(p.id) == my_id {
                                continue;
                            }
                            seen.push(p.id);
                            match others.get_mut(&p.id) {
                                Some(o) => o.st = p,
                                None => {
                                    others.insert(p.id, Other { dx: p.x, dy: p.y, st: p });
                                }
                            }
                        }
                        others.retain(|id, _| seen.contains(id));
                    }
                    ServerMsg::Dead { idx, left } => g.map.set_dead_cell(idx, now + left as f64),
                    ServerMsg::World { dead } => {
                        for (idx, left) in dead {
                            g.map.set_dead_cell(idx, now + left as f64);
                        }
                    }
                    ServerMsg::Market { offers } => market = offers,
                    ServerMsg::Board { score, fishers } => {
                        board_score = score;
                        board_fish = fishers;
                    }
                    ServerMsg::Posted { offer } => {
                        posted.insert(offer.id, (offer.item, offer.qty));
                        g.say(texts.offer_posted, now);
                    }
                    ServerMsg::Sold { id, gold } => {
                        posted.remove(&id);
                        // TODO #29: Gold should be server-authoritative
                        g.gold += gold;
                        g.say(&format!("{} +{gold} {}", texts.offer_sold, texts.gold_label), now);
                    }
                    ServerMsg::Withdrawn { offer } => {
                        posted.remove(&offer.id);
                        g.inv[(offer.item as usize).min(NI - 1)] += offer.qty;
                        g.say(texts.offer_withdrawn, now);
                    }
                    ServerMsg::Bought { offer } => {
                        // TODO #29: Gold should be server-authoritative
                        g.gold = g.gold.saturating_sub(offer.price);
                        g.inv[(offer.item as usize).min(NI - 1)] += offer.qty;
                        g.say(&format!("{}: {} ×{}", texts.bought, texts.item_names[(offer.item as usize).min(NI - 1)], offer.qty), now);
                    }
                    ServerMsg::Err { text } => g.say(translate_messages::translate_message(&text, texts), now),
                    ServerMsg::AuthSuccess { account } => {
                        let is_first = acc_ui.active.is_none();
                        acc_ui.active = Some(account.clone());
                        acc_ui.username = account.username.clone();
                        acc_ui.status = Some(format!("Успішний вхід як {}", account.username));
                        name = account.username.clone();
                        g.gold = account.gold;
                        g.caught = account.fish;
                        g.biggest = account.big_fish;
                        g.houses = account.houses;
                        for (i, v) in account.inventory.iter().enumerate().take(NI) {
                            g.inv[i] = *v;
                        }
                        for (i, v) in account.tools.iter().enumerate().take(NT) {
                            g.tools[i] = *v;
                        }
                        if is_first && account.x > 0.0 && account.y > 0.0 {
                            g.x = account.x;
                            g.y = account.y;
                        }
                        save::store_account_creds(&save::AccountCredentials {
                            email: account.email.clone(),
                            password: acc_ui.password.clone(),
                            username: account.username.clone(),
                        });
                        save::store(&snapshot(&g, now, &name, &posted));
                        g.say(&format!("Вхід: {}", account.username), now);
                    }
                    ServerMsg::AuthFailed { reason } => {
                        acc_ui.status = Some(format!("Помилка: {}", reason));
                        g.say(&format!("Помилка акаунту: {}", reason), now);
                    }
                    ServerMsg::PasswordChanged => {
                        acc_ui.status = Some("Пароль успішно змінено!".to_string());
                        // Update saved credentials with the new password
                        if acc_ui.active.is_some() && !acc_ui.new_password.is_empty() {
                            acc_ui.password = acc_ui.new_password.clone();
                            save::store_account_creds(&save::AccountCredentials {
                                email: acc_ui.email.clone(),
                                password: acc_ui.password.clone(),
                                username: acc_ui.username.clone(),
                            });
                        }
                        acc_ui.new_password.clear();
                        acc_ui.mode = 0;
                        g.say("Пароль успішно змінено!", now);
                    }
                },
            }
        }

        // ---------- ввод ----------
        for (id, ph, pos) in pointers() {
            match ph {
                Ph::Start => {
                    if let Some(c) = ui.hit(pos) {
                        if c == Cmd::Act {
                            act_id = Some(id);
                            if is_mobile {
                                let outcome = g.act_target(now, &mut out, texts, None);
                                spawn_hit_feedback(outcome, &mut floating_texts, now, texts);
                                if outcome != HitOutcome::None {
                                    audio.play("hit");
                                }
                            }
                        } else {
                            cmds.push(c);
                        }
                    } else if ui.menu {
                        if ui.panel.contains(pos) {
                            scroll_id = Some((id, pos.y));
                        }
                    } else if is_mobile && joy_id.is_none() && name_edit.is_none() && pos.x < sw * 0.45 {
                        // Джойстик для пересування на лівій стороні мобільного екрану
                        joy_id = Some(id);
                        origin = pos;
                        cur = pos;
                    } else if !ui.menu && name_edit.is_none() {
                        let wx = g.x + (pos.x - cx) / ts;
                        let wy = g.y + (pos.y - cy) / ts;
                        g.face_to(wx, wy);

                        // Удар по ресурсу або тварині
                        if let Some(target) = g.find_nearby_target(now, Some((wx, wy))) {
                            if target.dist_sq <= 3.8 {
                                let outcome = g.act_target(now, &mut out, texts, Some((target.x, target.y)));
                                spawn_hit_feedback(outcome, &mut floating_texts, now, texts);
                                if outcome != HitOutcome::None {
                                    audio.play("hit");
                                }
                            }
                        }
                    }
                }
                Ph::Move => {
                    if joy_id == Some(id) {
                        cur = pos;
                    }
                    if let Some((sid, ly)) = scroll_id {
                        if sid == id {
                            ui.scroll -= pos.y - ly;
                            scroll_id = Some((sid, pos.y));
                        }
                    }
                }
                Ph::End => {
                    if joy_id == Some(id) {
                        joy_id = None;
                    }
                    if act_id == Some(id) {
                        act_id = None;
                    }
                    if matches!(scroll_id, Some((sid, _)) if sid == id) {
                        scroll_id = None;
                    }
                }
            }
        }
        if ui.menu {
            ui.scroll -= mouse_wheel().1 * 40.0;
        }
        let typing = (ui.menu && ui.tab == 5) || name_edit.is_some();
        if ui.menu && ui.tab == 5 {
            if acc_ui.active.is_none() {
                // Автоматично обираємо коректне активне поле для поточного режиму
                match acc_ui.mode {
                    0 => if acc_ui.active_field != 0 && acc_ui.active_field != 1 { acc_ui.active_field = 0; },
                    1 => if acc_ui.active_field != 0 && acc_ui.active_field != 1 && acc_ui.active_field != 2 { acc_ui.active_field = 2; },
                    3 => acc_ui.active_field = 4,
                    _ => {}
                }
                while let Some(c) = get_char_pressed() {
                    if !c.is_control() {
                        match acc_ui.active_field {
                            0 => if acc_ui.email.chars().count() < 64 { acc_ui.email.push(c); },
                            1 => if acc_ui.password.chars().count() < 32 { acc_ui.password.push(c); },
                            2 => if acc_ui.username.chars().count() < 20 { acc_ui.username.push(c); },
                            4 => if acc_ui.code.chars().count() < 8 && (c.is_ascii_digit() || c.is_alphabetic()) {
                                for uc in c.to_uppercase() {
                                    acc_ui.code.push(uc);
                                }
                            },
                            _ => {}
                        }
                    }
                }
                if is_key_pressed(KeyCode::Backspace) {
                    match acc_ui.active_field {
                        0 => { acc_ui.email.pop(); },
                        1 => { acc_ui.password.pop(); },
                        2 => { acc_ui.username.pop(); },
                        4 => { acc_ui.code.pop(); },
                        _ => {}
                    }
                }
                if is_key_pressed(KeyCode::Tab) {
                    if acc_ui.mode == 0 {
                        acc_ui.active_field = if acc_ui.active_field == 0 { 1 } else { 0 };
                    } else if acc_ui.mode == 1 {
                        acc_ui.active_field = match acc_ui.active_field {
                            2 => 0,
                            0 => 1,
                            _ => 2,
                        };
                    } else if acc_ui.mode == 3 {
                        acc_ui.active_field = 4;
                    }
                }
                if is_key_pressed(KeyCode::Enter) {
                    if acc_ui.mode == 3 {
                        cmds.push(Cmd::AccountCodeSubmit);
                    } else {
                        cmds.push(Cmd::AccountSubmit);
                    }
                }
            } else if acc_ui.mode == 2 {
                // Change password mode: 1 = old password, 3 = new password
                if acc_ui.active_field != 1 && acc_ui.active_field != 3 {
                    acc_ui.active_field = 1;
                }
                while let Some(c) = get_char_pressed() {
                    if !c.is_control() {
                        match acc_ui.active_field {
                            1 => if acc_ui.password.chars().count() < 32 { acc_ui.password.push(c); },
                            3 => if acc_ui.new_password.chars().count() < 32 { acc_ui.new_password.push(c); },
                            _ => {}
                        }
                    }
                }
                if is_key_pressed(KeyCode::Backspace) {
                    match acc_ui.active_field {
                        1 => { acc_ui.password.pop(); },
                        3 => { acc_ui.new_password.pop(); },
                        _ => {}
                    }
                }
                if is_key_pressed(KeyCode::Tab) {
                    acc_ui.active_field = if acc_ui.active_field == 1 { 3 } else { 1 };
                }
                if is_key_pressed(KeyCode::Enter) {
                    cmds.push(Cmd::AccountChangePassword);
                }
            }
        } else if let Some(buf) = name_edit.as_mut() {
            while let Some(c) = get_char_pressed() {
                if !c.is_control() && buf.chars().count() < 16 {
                    buf.push(c);
                }
            }
            if is_key_pressed(KeyCode::Backspace) {
                buf.pop();
            }
            if is_key_pressed(KeyCode::Enter) {
                cmds.push(Cmd::NameOk);
            }
        } else {
            while get_char_pressed().is_some() {}
        }
        for (key, c) in [
            (KeyCode::F, Cmd::Eat),
            (KeyCode::R, Cmd::Drink),
            (KeyCode::G, Cmd::Dig),
            (KeyCode::T, Cmd::Tame),
            (KeyCode::B, Cmd::Boat),
            (KeyCode::Y, Cmd::Bridge),
            (KeyCode::Key1, Cmd::Quick(0)),
            (KeyCode::Key2, Cmd::Quick(1)),
            (KeyCode::Key3, Cmd::Quick(2)),
            (KeyCode::Key4, Cmd::Quick(3)),
            (KeyCode::Key5, Cmd::Quick(4)),
        ] {
            if !typing && is_key_pressed(key) {
                cmds.push(c);
            }
        }
        // Обробка зміни клавіші в режимі ребінду
        if let Some(action) = ui.rebinding {
            if let Some(key) = get_last_key_pressed() {
                if key == KeyCode::Escape {
                    ui.rebinding = None;
                } else {
                    let k_name = key_name(key);
                    settings.keybindings.set_key_for_action(action, k_name.to_string());
                    save_settings(&settings);
                    ui.rebinding = None;
                    audio.play("craft");
                }
            }
        }

        if !typing && settings.keybindings.is_inventory_pressed() {
            cmds.push(Cmd::Inventory);
        }
        if !typing && settings.keybindings.is_craft_pressed() {
            cmds.push(Cmd::CraftMenu);
        }
        if !typing && settings.keybindings.is_eat_pressed() {
            cmds.push(Cmd::Eat);
        }
        if !typing && settings.keybindings.is_drink_pressed() {
            cmds.push(Cmd::Drink);
        }
        if !typing && settings.keybindings.is_dig_pressed() {
            cmds.push(Cmd::Dig);
        }
        if !typing && settings.keybindings.is_menu_pressed() {
            cmds.push(if ui.menu { Cmd::Close } else { Cmd::Menu });
        }
        if !typing && is_key_pressed(KeyCode::Escape) {
            cmds.push(if ui.menu { Cmd::Close } else { Cmd::Settings });
        }
        // ---------- ввод ----------
        let typing = name_edit.is_some() || (ui.menu && ui.rebinding.is_none());
        for c in cmds {
            match c {
                Cmd::Eat => {
                    g.eat(now, texts);
                    audio.play("eat");
                }
                Cmd::Drink => {
                    g.drink(now, texts);
                    audio.play("drink");
                }
                Cmd::Dig => {
                    g.dig(now, texts);
                    audio.play("hit");
                }
                Cmd::Tame => g.tame(now, texts),
                Cmd::Boat => g.boat_toggle(now, texts),
                // TODO #30: Bridges/fires/furnaces should sync via server
                Cmd::Bridge => g.bridge(now, texts),
                Cmd::Menu => {
                    ui.menu = !ui.menu;
                    ui.scroll = 0.0;
                }
                Cmd::Inventory => {
                    if ui.menu && ui.tab == 0 {
                        ui.menu = false;
                    } else {
                        ui.menu = true;
                        ui.tab = 0; // Відкрити вкладку "Інвентар"
                        ui.scroll = 0.0;
                    }
                }
                Cmd::CraftMenu => {
                    if ui.menu && ui.tab == 1 {
                        ui.menu = false;
                    } else {
                        ui.menu = true;
                        ui.tab = 1; // Відкрити вкладку "Крафт"
                        ui.scroll = 0.0;
                    }
                }
                Cmd::Settings => {
                    ui.menu = true;
                    ui.tab = 4; // Відкрити вкладку "Налаштування"
                    ui.settings_subtab = 0;
                    ui.rebinding = None;
                    ui.scroll = 0.0;
                }
                Cmd::SettingsSubtab(s) => {
                    ui.settings_subtab = s;
                    ui.rebinding = None;
                }
                Cmd::Rebind(action) => {
                    if ui.rebinding == Some(action) {
                        ui.rebinding = None;
                    } else {
                        ui.rebinding = Some(action);
                    }
                }
                Cmd::ResetKeybinds => {
                    settings.keybindings = Default::default();
                    save_settings(&settings);
                    ui.rebinding = None;
                    audio.play("craft");
                }
                Cmd::Close => {
                    if acc_ui.active.is_none() {
                        acc_ui.status = Some("⚠️ Для гри потрібна обов'язкова авторизація!".to_string());
                        ui.menu = true;
                        ui.tab = 5;
                    } else {
                        ui.menu = false;
                        ui.rebinding = None;
                    }
                }
                Cmd::Quit => {
                    // Зберігаємо гру перед виходом
                    save::store(&snapshot(&g, now, &name, &posted));
                    save_settings(&settings);
                    // Виходимо з гри
                    std::process::exit(0);
                }
                Cmd::ChangeLanguage => {
                    settings.language = settings.language.next();
                    save_settings(&settings);
                }
                Cmd::ToggleMusic => {
                    settings.music_enabled = !settings.music_enabled;
                    audio.update_settings(settings.music_enabled, settings.sounds_enabled, settings.music_volume, settings.sound_volume);
                    save_settings(&settings);
                }
                Cmd::ToggleSound => {
                    settings.sounds_enabled = !settings.sounds_enabled;
                    audio.update_settings(settings.music_enabled, settings.sounds_enabled, settings.music_volume, settings.sound_volume);
                    save_settings(&settings);
                }
                Cmd::MusicVolume(v) => {
                    settings.music_volume = (settings.music_volume + v).clamp(0.0, 1.0);
                    audio.update_settings(settings.music_enabled, settings.sounds_enabled, settings.music_volume, settings.sound_volume);
                    save_settings(&settings);
                }
                Cmd::SoundVolume(v) => {
                    settings.sound_volume = (settings.sound_volume + v).clamp(0.0, 1.0);
                    audio.update_settings(settings.music_enabled, settings.sounds_enabled, settings.music_volume, settings.sound_volume);
                    save_settings(&settings);
                }
                Cmd::Tab(t) => {
                    ui.tab = t;
                    ui.scroll = 0.0;
                }
                Cmd::Craft(i) => {
                    g.craft(i, now, texts);
                    audio.play("craft");
                }
                Cmd::Quick(j) => {
                    if let Some(&i) = g.quick_list(now).get(j) {
                        g.craft(i, now, texts);
                        audio.play("craft");
                    }
                }
                Cmd::MktItem(d) => ui.mk_item = ((ui.mk_item as i32 + d).rem_euclid(NI as i32)) as usize,
                Cmd::MktQty(d) => ui.mk_qty = ((ui.mk_qty as i32 + d).clamp(1, 99)) as u32,
                Cmd::MktPrice(d) => {
                    let step = (ui.mk_price / 10).max(1) as i32;
                    ui.mk_price = ((ui.mk_price as i32 + d * step).clamp(1, 9999)) as u32;
                }
                Cmd::MktPost => {
                    if !connected || my_id.is_none() {
                        g.say(texts.market_online_only, now);
                    } else if g.inv[ui.mk_item] < ui.mk_qty {
                        g.say(texts.not_enough_items, now);
                    } else {
                        // TODO #27: Track in-flight posts, refund on ServerMsg::Err
                        g.inv[ui.mk_item] -= ui.mk_qty;
                        out.push(ClientMsg::Post { item: ui.mk_item as u8, qty: ui.mk_qty, price: ui.mk_price });
                    }
                }
                Cmd::MktBuy(id) => {
                    if let Some(o) = market.iter().find(|o| o.id == id) {
                        if g.gold < o.price {
                            g.say(texts.not_enough_gold, now);
                        } else {
                            out.push(ClientMsg::Buy { id });
                        }
                    }
                }
                Cmd::MktWithdraw(id) => out.push(ClientMsg::Withdraw { id }),
                Cmd::NameOpen => {
                    name_edit = Some(name.clone());
                    ui.menu = false;
                }
                Cmd::Key(ch) => {
                    if let Some(b) = name_edit.as_mut() {
                        if b.chars().count() < 16 {
                            b.push(if b.is_empty() { ch.to_uppercase().next().unwrap_or(ch) } else { ch });
                        }
                    }
                }
                Cmd::Space => {
                    if let Some(b) = name_edit.as_mut() {
                        if !b.is_empty() && !b.ends_with(' ') && b.chars().count() < 16 {
                            b.push(' ');
                        }
                    }
                }
                Cmd::Back => {
                    if let Some(b) = name_edit.as_mut() {
                        b.pop();
                    }
                }
                Cmd::Lang => kb_lang = (kb_lang + 1) % 3,
                Cmd::NameOk => {
                    if let Some(b) = name_edit.take() {
                        let n = b.trim().to_string();
                        if n.is_empty() {
                            name_edit = Some(b);
                        } else {
                            name = n.clone();
                            net.send(ClientMsg::Join {
                                name: n,
                                device_info: Some(DeviceInfo::detect()),
                            });
                            save::store(&snapshot(&g, now, &name, &posted));
                        }
                    }
                }
                Cmd::Act => {}
                Cmd::AccountField(f) => {
                    acc_ui.active_field = f;
                }
                Cmd::AccountSwitchModeTab(m) => {
                    acc_ui.mode = m;
                    acc_ui.active_field = match m {
                        0 => 0,
                        1 => 2,
                        3 => 4,
                        _ => 0,
                    };
                    acc_ui.status = None;
                }
                Cmd::AccountGoogleLogin => {
                    #[cfg(target_os = "windows")]
                    {
                        let _ = std::process::Command::new("rundll32")
                            .args(["url.dll,FileProtocolHandler", "https://pixel-village.online/cabinet.html"])
                            .spawn();
                    }
                    #[cfg(not(target_os = "windows"))]
                    {
                        let _ = std::process::Command::new("xdg-open")
                            .arg("https://pixel-village.online/cabinet.html")
                            .spawn();
                    }
                    acc_ui.mode = 3;
                    acc_ui.active_field = 4;
                    acc_ui.status = Some("Відкрито кабінет у браузері. Увійдіть через Google та введіть 6-значний код:".to_string());
                }
                Cmd::AccountCodeSubmit => {
                    let clean_code = acc_ui.code.trim();
                    if clean_code.is_empty() {
                        acc_ui.status = Some("Помилка: Введіть 6-значний код з кабінету".to_string());
                    } else {
                        acc_ui.status = Some("Перевірка коду...".to_string());
                        net.send(ClientMsg::CodeLogin {
                            code: clean_code.to_string(),
                            device_info: Some(DeviceInfo::detect()),
                        });
                    }
                }
                Cmd::AccountSubmit => {
                    if acc_ui.mode == 0 {
                        // Login
                        if acc_ui.email.trim().is_empty() || acc_ui.password.is_empty() {
                            acc_ui.status = Some("Помилка: Заповніть пошту та пароль".to_string());
                        } else {
                            acc_ui.status = Some("Авторизація...".to_string());
                            net.send(ClientMsg::Login {
                                email: acc_ui.email.trim().to_string(),
                                password: acc_ui.password.clone(),
                                device_info: Some(DeviceInfo::detect()),
                            });
                        }
                    } else {
                        // Register
                        if acc_ui.email.trim().is_empty() || acc_ui.password.is_empty() || acc_ui.username.trim().is_empty() {
                            acc_ui.status = Some("Помилка: Заповніть всі поля".to_string());
                        } else if acc_ui.password.len() < 6 {
                            acc_ui.status = Some("Помилка: Пароль має містити щонайменше 6 символів".to_string());
                        } else {
                            acc_ui.status = Some("Реєстрація акаунту...".to_string());
                            net.send(ClientMsg::Register {
                                email: acc_ui.email.trim().to_string(),
                                username: acc_ui.username.trim().to_string(),
                                password: acc_ui.password.clone(),
                                device_info: Some(DeviceInfo::detect()),
                            });
                        }
                    }
                }
                Cmd::AccountLogout => {
                    acc_ui.active = None;
                    acc_ui.password.clear();
                    acc_ui.new_password.clear();
                    acc_ui.mode = 0;
                    save::clear_account_creds();
                    acc_ui.status = Some("Ви вийшли з акаунту".to_string());
                    g.say("Ви вийшли з акаунту", now);
                }
                Cmd::AccountChangePassword => {
                    if acc_ui.mode == 2 {
                        if acc_ui.password.is_empty() || acc_ui.new_password.is_empty() {
                            acc_ui.status = Some("Помилка: Заповніть старий і новий пароль".to_string());
                        } else if acc_ui.new_password.len() < 6 {
                            acc_ui.status = Some("Помилка: Новий пароль має містити щонайменше 6 символів".to_string());
                        } else {
                            acc_ui.status = Some("Зміна пароля...".to_string());
                            net.send(ClientMsg::ChangePassword {
                                old_password: acc_ui.password.clone(),
                                new_password: acc_ui.new_password.clone(),
                            });
                        }
                    } else {
                        acc_ui.mode = 2;
                        acc_ui.active_field = 1;
                        acc_ui.new_password.clear();
                        acc_ui.status = None;
                    }
                }
                Cmd::AccountSaveCloud => {
                    if acc_ui.active.is_some() {
                        net.send(ClientMsg::SaveAccount {
                            gold: g.gold,
                            score: g.score(),
                            fish: g.caught,
                            big_fish: g.biggest,
                            inventory: g.inv.to_vec(),
                            tools: g.tools.to_vec(),
                            houses: g.houses,
                            x: g.x,
                            y: g.y,
                        });
                        acc_ui.status = Some("Прогрес надіслано в хмару!".to_string());
                        g.say("Прогрес збережено в хмару!", now);
                    }
                }
            }
        }

        let mut dir = Vec2::ZERO;
        if typing {
            // пока вводим имя або відкрите меню, персонаж стоїть
        } else {
            if is_key_down(KeyCode::Left) || settings.keybindings.is_left_down() {
                dir.x -= 1.0;
            }
            if is_key_down(KeyCode::Right) || settings.keybindings.is_right_down() {
                dir.x += 1.0;
            }
            if is_key_down(KeyCode::Up) || settings.keybindings.is_up_down() {
                dir.y -= 1.0;
            }
            if is_key_down(KeyCode::Down) || settings.keybindings.is_down_down() {
                dir.y += 1.0;
            }
        }
        if dir != Vec2::ZERO {
            dir = dir.normalize();
        } else if is_mobile && joy_id.is_some() {
            let v = (cur - origin) / ring_r;
            if v.length() > 0.2 {
                dir = v.clamp_length_max(1.0);
            }
        }
        let (mx, my) = mouse_position();
        let in_minimap_area = mx >= sw - base * 0.2 - pad - 10.0 && my <= pad + base * 0.2 + fs * 6.0;
        let mouse_act = !is_mobile && !ui.menu && name_edit.is_none() && !in_minimap_area && is_mouse_button_down(MouseButton::Left);
        let hold = !typing && (settings.keybindings.is_action_down() || is_key_down(KeyCode::Space) || mouse_act || act_id.is_some());
        if typing {
            dir = Vec2::ZERO;
        }

        // ---------- логика ----------
        g.tick(dt, now, &Input { dir, hold }, &mut out, texts);
        g.particles.update(dt);
        let (px, py) = (g.x, g.y);

        // ---------- сеть: отправка ----------
        send_t += dt;
        stats_t += dt;
        save_t += dt;
        if save_t > 8.0 {
            save_t = 0.0;
            save::store(&snapshot(&g, now, &name, &posted));
            if acc_ui.active.is_some() {
                net.send(ClientMsg::SaveAccount {
                    gold: g.gold,
                    score: g.score(),
                    fish: g.caught,
                    big_fish: g.biggest,
                    inventory: g.inv.to_vec(),
                    tools: g.tools.to_vec(),
                    houses: g.houses,
                    x: g.x,
                    y: g.y,
                });
            }
        }
        let held = g.held();
        let cs = (px, py, g.face, g.moving, g.in_boat, g.tool(Tl::Coat), held);
        if send_t > 0.066 && (cs != last_sent || send_t > 1.0) {
            out.push(ClientMsg::Move { x: px, y: py, face: g.face, moving: g.moving, boat: g.in_boat, coat: g.tool(Tl::Coat), held });
            last_sent = cs;
            send_t = 0.0;
        }
        if stats_t > 5.0 {
            stats_t = 0.0;
            // TODO #28: Stats should be computed server-side
            out.push(ClientMsg::Stats { score: g.score(), big: g.biggest, fish: g.caught });
        }
        for m in out {
            net.send(m);
        }
        for o in others.values_mut() {
            let kk = (dt * 12.0).min(1.0);
            o.dx += (o.st.x - o.dx) * kk;
            o.dy += (o.st.y - o.dy) * kk;
        }

        // ---------- рисование мира ----------
        clear_background(hx(0x2b3a2b));
        let x0 = (px - cx / ts).floor() as i32 - 1;
        let x1 = (px + cx / ts).ceil() as i32 + 1;
        let y0 = (py - cy / ts).floor() as i32 - 1;
        let y1 = (py + cy / ts).ceil() as i32 + 1;
        let scr = |tx: i32, ty: i32| (((tx as f32 - px) * ts + cx).floor(), ((ty as f32 - py) * ts + cy).floor());
        let wp = |x: f32, y: f32| (cx + (x - px) * ts, cy + (y - py) * ts);
        for ty in y0..=y1 {
            for tx in x0..=x1 {
                let (sx, sy) = scr(tx, ty);
                if tx < 0 || ty < 0 || tx >= W as i32 || ty >= W as i32 {
                    draw_rectangle(sx, sy, ts + 1.0, ts + 1.0, hx(0x2b3a2b));
                } else {
                    draw_tile(&g.map, tx, ty, sx, sy, ts, u, now, g.bridges.contains(&(tx, ty)));
                }
            }
        }
        for i in 0..g.houses.min(24) as i32 {
            let (sx, sy) = scr(14 + i % 12, 15 - i / 12);
            draw_house(sx, sy, u);
        }
        for f in &g.furn {
            let (sx, sy) = wp(f.0, f.1);
            draw_furnace(sx, sy, u, now);
        }
        for f in &g.fires {
            let (sx, sy) = wp(f.0, f.1);
            draw_fire(sx, sy, u, now);
        }
        let me_row = py.floor() as i32;
        let mut labels: Vec<(String, f32, f32)> = vec![];
        // Малюємо "могилу" з речами після смерті
        if let Some((dx, dy, _, _)) = g.death_drop {
            let (sx, sy) = wp(dx, dy);
            // Мерехтливий рюкзак з речами
            let pulse = ((now * 2.0).sin() * 0.3 + 0.7) as f32;
            draw_rectangle(sx - 6.0 * u, sy - 6.0 * u, 12.0 * u, 12.0 * u, Color::new(0.6, 0.4, 0.2, pulse));
            draw_rectangle_lines(sx - 6.0 * u, sy - 6.0 * u, 12.0 * u, 12.0 * u, 1.5, Color::new(1.0, 0.8, 0.3, pulse));
            labels.push((texts.pick_up_items.into(), sx, sy - 10.0 * u));
        }
        let swing = g.fx.as_ref().and_then(|f| {
            let p = ((now - f.t) / 0.26) as f32;
            (f.tool > 0 && (0.0..1.0).contains(&p)).then_some((f.tool, p))
        });
        for pass in 0..2 {
            for ty in y0..=y1 {
                if (pass == 0) != (ty <= me_row) {
                    continue;
                }
                for tx in x0..=x1 {
                    if let Some(n) = g.map.node_at(tx, ty) {
                        let (sx, sy) = scr(tx, ty);
                        draw_node(n, sx, sy, u, now);
                    }
                }
            }
            if pass == 0 {
                for m in &g.mobs {
                    let (sx, sy) = wp(m.x, m.y);
                    draw_mob(m, sx, sy, u, now);
                    if m.kind == game::MobKind::Pet && m.dead == 0.0 {
                        labels.push((texts.pet_friend.into(), sx, sy - 11.0 * u));
                    }
                }
                for o in others.values() {
                    let (ox, oy) = wp(o.dx, o.dy);
                    let lk = Look {
                        body: hx(PLAYER_COLORS[(o.st.color as usize) % 5]),
                        face: o.st.face,
                        moving: o.st.moving,
                        coat: o.st.coat,
                        held: o.st.held,
                        boat: o.st.boat,
                        swing: None,
                    };
                    let (lx, ly) = person(ox, oy, k, now, &lk);
                    labels.push((o.st.name.clone(), lx, ly));
                }
                let lk = Look { body: my_color, face: g.face, moving: g.moving, coat: g.tool(Tl::Coat), held, boat: g.in_boat, swing };
                let (lx, ly) = person(cx, cy, k, now, &lk);
                labels.push((name.clone(), lx, ly));
            }
        }
        if let Some(f) = &g.fx {
            let (sx, sy) = wp(f.x, f.y);
            draw_fx_particles(f, sx, sy, u, now);
        }

        // Малюємо частинки з правильною конвертацією координат
        g.particles.draw(|wx, wy| wp(wx, wy), u);

        // Малювання плаваючих текстів (отримані ресурси)
        floating_texts.retain(|ft| (now - ft.t) < 1.3);
        for ft in &floating_texts {
            let age = (now - ft.t) as f32;
            let alpha = (1.0 - age / 1.3).clamp(0.0, 1.0);
            let lift = age * 0.8;
            let (sx, sy) = wp(ft.x, ft.y - lift);
            let text_size = fs * 0.85;
            let mut col = ft.color;
            col.a = alpha;
            let shadow = Color::new(0.0, 0.0, 0.0, alpha * 0.75);
            text_c(&font, &ft.text, sx + 1.0, sy + 1.0, text_size, shadow);
            text_c(&font, &ft.text, sx, sy, text_size, col);
        }

        // Підсвітка та міцність об'єкта біля гравця
        if let Some(target) = g.find_nearby_target(now, None) {
            let (sx, sy) = wp(target.x, target.y);
            let base_r = 16.0 * u;
            draw_circle_lines(sx, sy, base_r, 1.5, ha(0x38bdf8, 0.55));

            if is_mobile {
                match target.kind {
                    TargetKind::Node(kind, _, hp, max_hp) => {
                        let pips_str = (0..max_hp).map(|i| if i < hp { "●" } else { "○" }).collect::<Vec<_>>().join(" ");
                        let node_name = match kind {
                            Kind::Tree => "🌲 Дерево",
                            Kind::Rock => "🪨 Камінь",
                            Kind::Bush => "🌿 Кущ",
                            Kind::BerryBush => "🍓 Ягоди",
                            Kind::CuOre => "⛏️ Мідь",
                            Kind::FeOre => "⛏️ Залізо",
                            Kind::AuOre => "⛏️ Золото",
                            _ => "Ресурс",
                        };
                        let badge = format!("{} [{}]", node_name, pips_str);
                        let bw = text_w(&font, &badge, fs * 0.75) + 12.0;
                        draw_rectangle(sx - bw / 2.0, sy - base_r - fs * 0.85, bw, fs * 1.1, ha(0x0a0f18, 0.85));
                        text_c(&font, &badge, sx, sy - base_r - fs * 0.1, fs * 0.75, WHITE);
                    }
                    TargetKind::Mob(kind, _, hp, dead) => {
                        let mob_name = match kind {
                            MobKind::Wolf => "🐺 Вовк",
                            MobKind::Bear => "🐻 Ведмідь",
                            MobKind::Deer => "🦌 Олень",
                            MobKind::Fox => "🦊 Лисиця",
                            MobKind::Rabbit => "🐰 Кролик",
                            MobKind::Pet => "🐶 Пес",
                        };
                        let badge = if dead { format!("🍖 Туша {}", mob_name) } else { format!("{} (HP: {:.0})", mob_name, hp.max(0.0)) };
                        let bw = text_w(&font, &badge, fs * 0.75) + 12.0;
                        draw_rectangle(sx - bw / 2.0, sy - base_r - fs * 0.85, bw, fs * 1.1, ha(0x0a0f18, 0.85));
                        text_c(&font, &badge, sx, sy - base_r - fs * 0.1, fs * 0.75, WHITE);
                    }
                    _ => {}
                }
            }
        }

        // удочка, леска, поплавок
        if g.fish.ph > 0 {
            let (bx, by) = wp(g.fish.wx, g.fish.wy);
            let dip = if g.fish.ph == 2 { ((now * 16.0).sin().abs() * 5.0) as f32 * k } else { (now * 12.0).sin() as f32 * 1.5 * k };
            let hx0 = cx + (if g.face == 2 { -14.0 } else if g.face == 3 { 14.0 } else { 12.0 }) * k;
            let hy0 = cy - 4.0 * k;
            let (tipx, tipy) = (hx0 + (bx - hx0) * 0.25, hy0 - 34.0 * k);
            draw_line(hx0, hy0 + 4.0 * k, tipx, tipy, 2.5 * k, hx(0x6b4a2b));
            let sag = if g.fish.ph == 3 { 0.0 } else { 30.0 * k };
            let (mx, my) = ((tipx + bx) / 2.0, tipy.min(by) + sag);
            let mut prev = (tipx, tipy);
            for i in 1..=12 {
                let t = i as f32 / 12.0;
                let p = ((1.0 - t).powi(2) * tipx + 2.0 * (1.0 - t) * t * mx + t * t * bx, (1.0 - t).powi(2) * tipy + 2.0 * (1.0 - t) * t * my + t * t * (by + dip));
                draw_line(prev.0, prev.1, p.0, p.1, 1.0_f32.max(k), ha(0xffffff, 0.85));
                prev = p;
            }
            draw_circle(bx, by + dip, 4.0 * k, WHITE);
            draw_circle(bx, by + dip - 2.0 * k, 3.0 * k, hx(0xee3333));
            if g.fish.ph == 2 {
                draw_circle_lines(bx, by, (12.0 + ((now * 12.0) % 8.0) as f32) * k, 2.0, WHITE);
            }
        }
        // TODO #77: Redundant second foreground-node rendering loop for pass==1 removed (already drawn in pass 1 above)
        // ночь, свет костров, дождь
        let al = g.night();
        if al > 0.0 {
            draw_rectangle(0.0, 0.0, sw, sh, ha(0x0a0f28, al.min(0.65)));
            if al > 0.1 {
                for f in &g.fires {
                    let (fx, fy) = wp(f.0, f.1);
                    for (r, a) in [(44.0, 0.10), (30.0, 0.12), (16.0, 0.16)] {
                        draw_circle(fx, fy, r * u, ha(0xffaa46, a));
                    }
                }
            }
        }
        if g.rain() {
            for _ in 0..60 {
                let (rx, ry) = (rand::gen_range(0.0, sw), rand::gen_range(0.0, sh));
                draw_line(rx, ry, rx - 2.0, ry + 9.0 * u, 1.5, ha(0xbed2ff, 0.55));
            }
        }
        for (s, x, y) in &labels {
            label(&font, s, *x, *y, ts * 0.3);
        }

        // ---------- интерфейс ----------
        ui.begin();
        let pad = fs * 0.5;
        // полоски состояния
        let (bw, bh) = (fs * 8.5, fs * 1.25);
        for (i, (v, c, l)) in [
            (g.hunger, 0xe0933a, texts.hunger),
            (g.thirst, 0x4aa3df, texts.thirst),
            (g.energy, 0xd9c23f, texts.energy),
            (g.hp, 0xd9534f, texts.health)
        ].iter().enumerate() {
            bar(&font, pad, pad + i as f32 * (bh + 4.0), bw, bh, *v, *c, l, fs);
        }
        let mut ly = pad + 4.0 * (bh + 4.0) + fs * 0.9;
        let (status, scol) = if connected { (texts.online, 0x6fbf73) } else { ("❌", 0xd9534f) };
        draw_circle(pad + fs * 0.4, ly - fs * 0.3, fs * 0.28, hx(scol));
        text(&font, &format!("{}: {} ({})", texts.online, online.max(usize::from(connected)), status), pad + fs * 1.0, ly, fs * 0.9, WHITE);
        for o in others.values().take(5) {
            ly += fs * 1.1;
            text(&font, &format!("• {}", o.st.name), pad + fs * 0.6, ly, fs * 0.85, ha(0xffffff, 0.9));
        }
        // мини-карта + очки
        let ms = base * 0.2;
        let (mx0, my0) = (sw - ms - pad, pad);
        draw_rectangle(mx0 - 3.0, my0 - 3.0, ms + 6.0, ms + 6.0, hx(0x4a3220));
        draw_texture_ex(&minimap, mx0, my0, WHITE, DrawTextureParams { dest_size: Some(vec2(ms, ms)), ..Default::default() });
        let mp = |x: f32, y: f32| (mx0 + x / W as f32 * ms, my0 + y / W as f32 * ms);
        for f in &g.fires {
            let (a, b) = mp(f.0, f.1);
            draw_rectangle(a - 1.5, b - 1.5, 3.0, 3.0, hx(0xff9020));
        }
        for o in others.values() {
            let (a, b) = mp(o.dx, o.dy);
            draw_rectangle(a - 2.0, b - 2.0, 4.0, 4.0, hx(PLAYER_COLORS[(o.st.color as usize) % 5]));
        }
        let (a, b) = mp(px, py);
        draw_rectangle(a - 3.5, b - 3.5, 7.0, 7.0, BLACK);
        draw_rectangle(a - 2.5, b - 2.5, 5.0, 5.0, my_color);

        // Кнопка відключення від сервера (під міні-картою)
        let disconnect_btn_width = ms;
        let disconnect_btn_height = fs * 1.8;
        let disconnect_btn_x = mx0;
        let disconnect_btn_y = my0 + ms + fs * 3.5;

        let (mx, my) = mouse_position();
        let disconnect_hovered = connected && mx >= disconnect_btn_x && mx <= disconnect_btn_x + disconnect_btn_width
            && my >= disconnect_btn_y && my <= disconnect_btn_y + disconnect_btn_height;

        if connected {
            let disconnect_bg = if disconnect_hovered {
                ha(0xd94f4f, 0.85)
            } else {
                ha(0x8a3a3a, 0.7)
            };

            draw_rectangle(disconnect_btn_x, disconnect_btn_y, disconnect_btn_width, disconnect_btn_height, disconnect_bg);
            draw_rectangle_lines(disconnect_btn_x, disconnect_btn_y, disconnect_btn_width, disconnect_btn_height, 2.0, hx(0xff6060));

            let disconnect_text = texts.disconnect;
            let disconnect_text_w = text_w(&font, disconnect_text, fs * 0.75);
            text(&font, disconnect_text, disconnect_btn_x + (disconnect_btn_width - disconnect_text_w) / 2.0,
                disconnect_btn_y + disconnect_btn_height * 0.7, fs * 0.75, WHITE);

            let disconnect_clicked = (disconnect_hovered && is_mouse_button_pressed(MouseButton::Left))
                || touches().iter().any(|t| (t.phase == TouchPhase::Started || t.phase == TouchPhase::Ended)
                    && t.position.x >= disconnect_btn_x && t.position.x <= disconnect_btn_x + disconnect_btn_width
                    && t.position.y >= disconnect_btn_y && t.position.y <= disconnect_btn_y + disconnect_btn_height);

            if disconnect_clicked {
                // Відкриваємо меню налаштувань
                // TODO #76: Should actually disconnect from server
                ui.menu = true;
                ui.tab = 4;
            }
        }

        for (i, s) in [format!("{} {}", texts.gold, g.gold), format!("{} {}", texts.score, g.score())].iter().enumerate() {
            let w = text_w(&font, s, fs * 0.9);
            draw_rectangle(sw - pad - w - 6.0, my0 + ms + fs * (0.5 + i as f32 * 1.25), w + 12.0, fs * 1.15, ha(0x000000, 0.55));
            text(&font, s, sw - pad - w, my0 + ms + fs * (1.35 + i as f32 * 1.25), fs * 0.9, WHITE);
        }

        // сообщение
        if now < g.msg_until {
            let w = text_w(&font, &g.msg, fs * 1.1) + fs * 2.0;
            let msg_x = cx - w / 2.0;
            let msg_y = pad + fs * 5.2;
            // Тінь
            draw_rectangle(msg_x + 2.0, msg_y + 2.0, w, fs * 1.9, ha(0x000000, 0.5));
            // Фон з заокругленням
            draw_rectangle(msg_x, msg_y, w, fs * 1.9, ha(0x1a1c18, 0.85));
            // Верхня кольорова лінія
            draw_rectangle(msg_x, msg_y, w, 3.0, ha(0x58b85f, 0.8));
            // Рамка
            draw_rectangle_lines(msg_x, msg_y, w, fs * 1.9, 1.0, ha(0xffffff, 0.2));
            label(&font, &g.msg, cx, pad + fs * 6.5, fs * 1.1);
        }

        // Попередження про оновлення від сервера
        if version_mismatch.is_some() {
            // Запустити перевірку оновлень один раз
            game_state.check_updates_once();
            version_mismatch = None; // не показувати більше
        }
        // шкала вываживания
        if g.fish.ph == 3 {
            let (mw, mh) = (base * 0.6, fs * 1.0);
            let (mx, my) = (cx - mw / 2.0, sh - base * 0.30);
            draw_rectangle(mx - 8.0, my - fs * 2.0, mw + 16.0, mh * 2.0 + fs * 2.4, ha(0x000000, 0.65));
            draw_rectangle(mx, my, mw, mh, hx(0x5a2a2a));
            draw_rectangle(mx + mw * 0.3, my, mw * 0.45, mh, hx(0x3c8a3c));
            draw_rectangle(mx + mw * (g.fish.tn / 100.0).clamp(0.0, 1.0) - 3.0, my - 5.0, 6.0, mh + 10.0, WHITE);
            draw_rectangle(mx, my + mh + 8.0, mw, mh * 0.6, hx(0x333333));
            draw_rectangle(mx, my + mh + 8.0, mw * (g.fish.p / 100.0).clamp(0.0, 1.0), mh * 0.6, hx(0x4aa3df));
            text_c(&font, texts.fish_minigame_hint, cx, my - fs * 0.5, fs * 0.9, WHITE);
        }

        if let Some(buf) = &name_edit {
            draw_name_modal(&mut ui, &font, buf, kb_lang, sw, sh, fs, now, texts);
        } else if !ui.menu {
            if is_mobile {
                // Мобільна кругла кнопка дії
                let ab = base * 0.22;
                let ar = Rect::new(sw - ab - pad, sh - ab - pad - fs * 0.5, ab, ab);
                draw_circle(ar.x + ab / 2.0 + 2.0, ar.y + ab / 2.0 + 2.0, ab / 2.0, ha(0x000000, 0.4));
                draw_circle(ar.x + ab / 2.0, ar.y + ab / 2.0, ab / 2.0, ha(if hold { 0x2f7d36 } else { 0x58b85f }, 0.9));
                draw_circle(ar.x + ab / 2.0 - ab * 0.08, ar.y + ab / 2.0 - ab * 0.08, ab * 0.35, ha(0xffffff, 0.12));
                draw_circle_lines(ar.x + ab / 2.0, ar.y + ab / 2.0, ab / 2.0, 2.5, ha(0x1f5a25, 0.9));
                draw_circle_lines(ar.x + ab / 2.0, ar.y + ab / 2.0, ab / 2.0 - 2.0, 1.0, ha(0xffffff, 0.2));
                text_c(&font, texts.action, ar.x + ab / 2.0, ar.y + ab / 2.0 + fs * 0.35, fs * 1.0, WHITE);
                ui.btns_push_circle(ar, Cmd::Act);

                // Компактні плаваючі мобільні пігулки дій (вертикально праворуч)
                let pill_w = ab * 0.75;
                let pill_h = fs * 2.0;
                let mobile_pills = [
                    ("🎒", texts.inventory, Cmd::Inventory, 0x7c4f24),
                    ("🔨", texts.craft, Cmd::CraftMenu, 0x2e6b3e),
                    ("🍴", texts.eat, Cmd::Eat, 0x6b5a3a),
                    ("🥤", texts.drink, Cmd::Drink, 0x3a5a6b),
                    ("⛏️", texts.dig, Cmd::Dig, 0x5a5040),
                    ("⚙️", texts.menu, Cmd::Menu, 0x4a6fa5),
                ];
                for (i, (icon, label, cmd, col)) in mobile_pills.iter().enumerate() {
                    let bx = sw - pad - pill_w;
                    let by = ar.y - (i as f32 + 1.0) * (pill_h + 5.0);
                    let r = Rect::new(bx, by, pill_w, pill_h);
                    ui.button(&font, r, &format!("{icon} {label}"), *cmd, true, *col, fs * 0.78);
                }
            } else {
                // ПК, Linux, macOS: чистий екран без зайвих наекранних кнопок дій!
                // Стильна смуга гарячих клавіш по центру внизу екрану:
                let hint_str = texts.controls_desktop_hint;
                let hint_fs = fs * 0.8;
                let bar_w = text_w(&font, hint_str, hint_fs) + fs * 2.2;
                let bar_h = fs * 1.7;
                let bar_x = (sw - bar_w) / 2.0;
                let bar_y = sh - pad - bar_h;
                draw_rectangle(bar_x, bar_y, bar_w, bar_h, ha(0x0d111a, 0.85));
                draw_rectangle_lines(bar_x, bar_y, bar_w, bar_h, 1.2, ha(0x38bdf8, 0.45));
                text_c(&font, hint_str, sw / 2.0, bar_y + bar_h * 0.65, hint_fs, Color::from_rgba(226, 232, 240, 240));
            }
        } else {
            draw_menu(&mut ui, &font, &g, now, sw, sh, fs, &market, &posted, my_id, &board_score, &board_fish, &settings, texts, &acc_ui);
        }
        // джойстик (лише для мобільних)
        if is_mobile && joy_id.is_some() && !ui.menu {
            draw_circle_lines(origin.x, origin.y, ring_r, 3.0, ha(0xffffff, 0.55));
            let v = (cur - origin).clamp_length_max(ring_r);
            draw_circle(origin.x + v.x, origin.y + v.y, ring_r * 0.42, ha(0xffffff, 0.35));
        }
        ui.finish();

        // Debug Info панель (F3 для показу/приховування)
        if is_key_pressed(KeyCode::F3) {
            settings.show_debug = !settings.show_debug;
            save_settings(&settings);
        }

        if settings.show_debug {
            let debug_fs = fs * 0.7;
            let debug_x = pad;
            let debug_y = pad + fs * 2.0;
            let fps = get_fps();
            let debug_lines = vec![
                format!("FPS: {}", fps),
                format!("{}: {}", texts.debug_players_online, online),
                format!("{}: ({:.1}, {:.1})", texts.debug_position, g.x, g.y),
                format!("{}: {:.0}%", texts.debug_time_of_day, g.night() * 100.0),
                format!("{}: {}", texts.debug_version, CLIENT_VERSION),
                if connected { texts.debug_connected.to_string() } else { texts.debug_disconnected.to_string() },
            ];

            // Напівпрозорий фон
            let box_w = 280.0;
            let box_h = debug_lines.len() as f32 * debug_fs * 1.5 + pad;
            draw_rectangle(debug_x, debug_y - pad, box_w, box_h, ha(0x000000, 0.7));
            draw_rectangle_lines(debug_x, debug_y - pad, box_w, box_h, 2.0, ha(0x4a6fa5, 0.8));

            // Виводимо debug інформацію
            for (i, line) in debug_lines.iter().enumerate() {
                let y = debug_y + i as f32 * debug_fs * 1.5;
                text(&font, line, debug_x + pad * 0.5, y + debug_fs, debug_fs, ha(0xffffff, 0.9));
            }
        }

        // Показати вікно оновлення (якщо є)
        game_state.draw_update_window(&font, texts);

        next_frame().await;
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_menu(
    ui: &mut Ui,
    font: &Font,
    g: &Game,
    now: f64,
    sw: f32,
    sh: f32,
    fs: f32,
    market: &[Offer],
    posted: &HashMap<u32, (u8, u32)>,
    my_id: Option<u32>,
    board_score: &[(String, u32)],
    board_fish: &[(String, f32, u32)],
    settings: &Settings,
    texts: &Texts,
    acc_ui: &AccountUiState,
) {
    let pad = fs * 0.6;
    let p = Rect::new(sw * 0.03, sh * 0.03, sw * 0.94, sh * 0.94);
    ui.panel = p;
    // Затемнення фону за панеллю
    draw_rectangle(0.0, 0.0, sw, sh, ha(0x000000, 0.4));
    // Тінь панелі
    draw_rectangle(p.x + 4.0, p.y + 4.0, p.w, p.h, ha(0x000000, 0.5));
    // Фон панелі з градієнтом
    draw_rectangle(p.x, p.y, p.w, p.h, ha(0x1a1c18, 0.97));
    // Верхня світла смуга
    draw_rectangle(p.x, p.y, p.w, 4.0, ha(0x8b7a55, 0.6));
    // Рамка
    draw_rectangle_lines(p.x, p.y, p.w, p.h, 3.0, hx(0x8b7a55));
    draw_rectangle_lines(p.x + 2.0, p.y + 2.0, p.w - 4.0, p.h - 4.0, 1.0, ha(0xffffff, 0.08));
    // вкладки (6 вкладок: Інвентар, Крафт, Ринок, Таблиця, Налаштування, Акаунт)
    let account_label = match settings.language {
        Language::Ukrainian => "Акаунт",
        Language::English => "Account",
        Language::Russian => "Аккаунт",
    };
    let tabs = [texts.inventory, texts.craft, texts.market, texts.leaderboard, texts.settings, account_label];
    let tw = (p.w - pad * 3.0 - fs * 6.5 - 24.0) / 6.0;
    for (i, t) in tabs.iter().enumerate() {
        let r = Rect::new(p.x + pad + i as f32 * (tw + 4.0), p.y + pad, tw, fs * 2.4);
        ui.button(font, r, t, Cmd::Tab(i as u8), true, if ui.tab == i as u8 { 0x2f7d36 } else { 0x4a4f44 }, fs);
    }
    ui.button(font, Rect::new(p.x + p.w - pad - fs * 6.5, p.y + pad, fs * 6.5, fs * 2.4), texts.close, Cmd::Close, true, 0x8a3b2e, fs);
    let body = Rect::new(p.x + pad, p.y + pad * 2.0 + fs * 2.4, p.w - pad * 2.0, p.h - pad * 3.0 - fs * 2.4);
    let rh = fs * 3.2;
    match ui.tab {
        0 => {
            // === ПІКСЕЛЬНИЙ ІНВЕНТАР ===
            let tools_w = (body.w * 0.36).max(fs * 13.0).min(body.w * 0.45);
            let inv_w = body.w - tools_w - pad;

            // Ліва панель - Рюкзак
            let inv_panel = Rect::new(body.x, body.y, inv_w, body.h);
            draw_rectangle(inv_panel.x, inv_panel.y, inv_panel.w, inv_panel.h, ha(0x1e1b18, 0.9));
            draw_rectangle_lines(inv_panel.x, inv_panel.y, inv_panel.w, inv_panel.h, 2.0, hx(0x4a3a28));

            // Заголовок рюкзака
            draw_rectangle(inv_panel.x, inv_panel.y, inv_panel.w, fs * 2.2, ha(0x2d241c, 0.95));
            draw_rectangle(inv_panel.x, inv_panel.y + fs * 2.2 - 2.0, inv_panel.w, 2.0, hx(0x6b533b));
            text(font, &format!("🎒 {}", texts.inventory), inv_panel.x + pad, inv_panel.y + fs * 1.5, fs * 1.05, hx(0xf5d996));

            // Сітка слотів предметів
            let slot_cols = ((inv_panel.w - pad * 2.0) / (fs * 7.5)).floor().max(2.0) as usize;
            let slot_w = (inv_panel.w - pad * 2.0 - (slot_cols - 1) as f32 * pad) / slot_cols as f32;
            let slot_h = fs * 3.2;

            let mut active_items: Vec<(usize, u32)> = Vec::new();
            for (i, &count) in g.inv.iter().enumerate() {
                if count > 0 {
                    active_items.push((i, count));
                }
            }

            let grid_y = inv_panel.y + fs * 2.6;
            let grid_h = inv_panel.h - fs * 2.8;
            let total_rows = (active_items.len() + slot_cols - 1) / slot_cols;
            let content_h = total_rows as f32 * (slot_h + pad);
            ui.scroll = ui.scroll.clamp(0.0, (content_h - grid_h).max(0.0));

            if active_items.is_empty() {
                text_c(font, texts.empty_inv_label, inv_panel.x + inv_panel.w / 2.0, inv_panel.y + inv_panel.h / 2.0, fs * 1.0, ha(0xffffff, 0.5));
            } else {
                for (idx, &(i, count)) in active_items.iter().enumerate() {
                    let col = idx % slot_cols;
                    let row = idx / slot_cols;
                    let sx = inv_panel.x + pad + col as f32 * (slot_w + pad);
                    let sy = grid_y + row as f32 * (slot_h + pad) - ui.scroll;

                    if sy + slot_h < grid_y || sy > grid_y + grid_h {
                        continue;
                    }

                    // Піксельний слот 3D-рамка
                    draw_rectangle(sx, sy, slot_w, slot_h, ha(0x141210, 0.95));
                    draw_line(sx, sy, sx + slot_w, sy, 2.0, ha(0x0a0807, 0.9));
                    draw_line(sx, sy, sx, sy + slot_h, 2.0, ha(0x0a0807, 0.9));
                    draw_line(sx, sy + slot_h, sx + slot_w, sy + slot_h, 2.0, ha(0x383025, 0.8));
                    draw_line(sx + slot_w, sy, sx + slot_w, sy + slot_h, 2.0, ha(0x383025, 0.8));

                    // Кольорова іконка предмета
                    let icon_size = fs * 1.8;
                    let icon_x = sx + pad * 0.7;
                    let icon_y = sy + (slot_h - icon_size) / 2.0;
                    draw_rectangle(icon_x, icon_y, icon_size, icon_size, hx(IT_COLOR[i]));
                    draw_rectangle_lines(icon_x, icon_y, icon_size, icon_size, 1.5, ha(0x000000, 0.6));
                    draw_rectangle(icon_x + 1.0, icon_y + 1.0, icon_size * 0.4, icon_size * 0.4, ha(0xffffff, 0.25));

                    // Назва предмета
                    let name_x = icon_x + icon_size + pad * 0.6;
                    let it_name = texts.item_names[i];
                    text(font, it_name, name_x, sy + fs * 1.3, fs * 0.85, WHITE);

                    // Кількість (значок у кутку)
                    let count_str = format!("×{}", count);
                    let count_w = text_w(font, &count_str, fs * 0.8);
                    let badge_x = sx + slot_w - count_w - pad * 0.8;
                    let badge_y = sy + slot_h - fs * 0.5;
                    draw_rectangle(badge_x - 3.0, badge_y - fs * 0.75, count_w + 6.0, fs * 0.95, ha(0x000000, 0.75));
                    text(font, &count_str, badge_x, badge_y, fs * 0.8, hx(0xf5d996));
                }
            }

            // Права панель - Інструменти та Статистика
            let tools_panel = Rect::new(body.x + inv_w + pad, body.y, tools_w, body.h);
            draw_rectangle(tools_panel.x, tools_panel.y, tools_panel.w, tools_panel.h, ha(0x1a1c18, 0.9));
            draw_rectangle_lines(tools_panel.x, tools_panel.y, tools_panel.w, tools_panel.h, 2.0, hx(0x4a3a28));

            // Заголовок інструментів
            draw_rectangle(tools_panel.x, tools_panel.y, tools_panel.w, fs * 2.2, ha(0x242820, 0.95));
            draw_rectangle(tools_panel.x, tools_panel.y + fs * 2.2 - 2.0, tools_panel.w, 2.0, hx(0x4a6040));
            text(font, &format!("🛠️ {}", texts.tools_label), tools_panel.x + pad, tools_panel.y + fs * 1.5, fs * 1.05, hx(0x96d9f5));

            // Список інструментів
            let mut ty = tools_panel.y + fs * 2.8;
            let mut tool_count = 0;
            for (t, &dur) in g.tools.iter().enumerate() {
                if dur > 0 {
                    tool_count += 1;
                    let max_dur = TL_DUR[t];
                    let card_h = fs * 2.4;

                    draw_rectangle(tools_panel.x + pad, ty, tools_panel.w - pad * 2.0, card_h, ha(0x222620, 0.85));
                    draw_rectangle_lines(tools_panel.x + pad, ty, tools_panel.w - pad * 2.0, card_h, 1.0, ha(0xffffff, 0.15));

                    // Назва інструмента
                    text(font, texts.tool_names[t], tools_panel.x + pad * 1.6, ty + fs * 1.1, fs * 0.88, WHITE);

                    // Смужка міцності
                    let bar_w = tools_panel.w - pad * 3.2;
                    let bar_h = fs * 0.55;
                    let bar_y = ty + fs * 1.45;
                    let ratio = (dur as f32 / max_dur as f32).clamp(0.0, 1.0);

                    draw_rectangle(tools_panel.x + pad * 1.6, bar_y, bar_w, bar_h, ha(0x000000, 0.6));
                    let dur_col = if ratio > 0.5 { 0x58b85f } else if ratio > 0.25 { 0xd9c23f } else { 0xd9534f };
                    draw_rectangle(tools_panel.x + pad * 1.6, bar_y, bar_w * ratio, bar_h, hx(dur_col));
                    draw_rectangle_lines(tools_panel.x + pad * 1.6, bar_y, bar_w, bar_h, 1.0, ha(0xffffff, 0.2));

                    let dur_text = format!("{}: {}/{}", texts.durability_label, dur, max_dur);
                    let dtw = text_w(font, &dur_text, fs * 0.7);
                    text(font, &dur_text, tools_panel.x + tools_panel.w - pad * 1.6 - dtw, ty + fs * 1.1, fs * 0.75, hx(0xd0d0d0));

                    ty += card_h + pad * 0.6;
                }
            }

            if tool_count == 0 {
                text(font, "—", tools_panel.x + pad * 2.0, ty + fs * 1.0, fs * 0.9, ha(0xffffff, 0.4));
            }

            // Статистика гравця внизу правої панелі
            let stat_y = tools_panel.y + tools_panel.h - fs * 7.0;
            draw_rectangle(tools_panel.x + pad, stat_y, tools_panel.w - pad * 2.0, fs * 6.4, ha(0x141613, 0.9));
            draw_rectangle_lines(tools_panel.x + pad, stat_y, tools_panel.w - pad * 2.0, fs * 6.4, 1.5, hx(0x4a3a28));

            text(font, &format!("🪙 {}: {}", texts.gold_label, g.gold), tools_panel.x + pad * 1.8, stat_y + fs * 1.4, fs * 0.9, hx(0xe0c060));
            text(font, &format!("🏆 {}: {}", texts.score, g.score()), tools_panel.x + pad * 1.8, stat_y + fs * 2.8, fs * 0.9, hx(0x6aa0d0));
            text(font, &format!("🏠 {}: {}", texts.houses_label, g.houses), tools_panel.x + pad * 1.8, stat_y + fs * 4.2, fs * 0.9, hx(0x7bc676));
            let pet_status = if g.has_pet { texts.yes } else { texts.no };
            text(font, &format!("🐺 {}: {}", texts.tame_wolf, pet_status), tools_panel.x + pad * 1.8, stat_y + fs * 5.6, fs * 0.9, WHITE);
        }
        1 => {
            // === КРАФТ / ВЕРСТАТ ===
            let ql = g.quick_list(now);
            let mut order: Vec<usize> = ql.clone();
            for i in 0..RECIPES.len() {
                if !order.contains(&i) {
                    order.push(i);
                }
            }
            let content = order.len() as f32 * rh;
            ui.scroll = ui.scroll.clamp(0.0, (content - body.h).max(0.0));
            for (row, &ri) in order.iter().enumerate() {
                let y = body.y + row as f32 * rh - ui.scroll;
                if y < body.y || y + rh > body.y + body.h {
                    continue;
                }
                let r = &RECIPES[ri];
                let can = g.can_craft(r, now);
                // Фон рядка з чергуванням кольорів та піксельною рамкою
                let row_bg = if can {
                    ha(if row % 2 == 0 { 0x243e26 } else { 0x1f3621 }, 0.92)
                } else {
                    ha(if row % 2 == 0 { 0x232521 } else { 0x1c1e1a }, 0.88)
                };
                draw_rectangle(body.x, y, body.w, rh - 4.0, row_bg);
                draw_rectangle_lines(body.x, y, body.w, rh - 4.0, 1.0, ha(0xffffff, if can { 0.15 } else { 0.05 }));

                // Ліва кольорова смужка
                let status_color = if can { 0x58b85f } else { 0x4a4f44 };
                draw_rectangle(body.x, y, 4.0, rh - 4.0, hx(status_color));

                // Іконка результату крафту
                let icon_size = fs * 1.8;
                let icon_x = body.x + pad + 4.0;
                let icon_y = y + (rh - 4.0 - icon_size) / 2.0;
                match r.out {
                    Out::Item(it, _) => {
                        draw_rectangle(icon_x, icon_y, icon_size, icon_size, hx(IT_COLOR[it as usize]));
                        draw_rectangle_lines(icon_x, icon_y, icon_size, icon_size, 1.5, ha(0x000000, 0.6));
                        draw_rectangle(icon_x + 1.0, icon_y + 1.0, icon_size * 0.35, icon_size * 0.35, ha(0xffffff, 0.25));
                    }
                    Out::Tool(_) => {
                        draw_rectangle(icon_x, icon_y, icon_size, icon_size, hx(0x4a6fa5));
                        draw_rectangle_lines(icon_x, icon_y, icon_size, icon_size, 1.5, ha(0x000000, 0.6));
                        text(font, "⚔", icon_x + icon_size * 0.2, icon_y + icon_size * 0.75, fs * 0.9, WHITE);
                    }
                    Out::Fire => {
                        draw_rectangle(icon_x, icon_y, icon_size, icon_size, hx(0xd9823a));
                        draw_rectangle_lines(icon_x, icon_y, icon_size, icon_size, 1.5, ha(0x000000, 0.6));
                        text(font, "🔥", icon_x + icon_size * 0.15, icon_y + icon_size * 0.75, fs * 0.9, WHITE);
                    }
                    Out::Furnace => {
                        draw_rectangle(icon_x, icon_y, icon_size, icon_size, hx(0x7a5a3a));
                        draw_rectangle_lines(icon_x, icon_y, icon_size, icon_size, 1.5, ha(0x000000, 0.6));
                        text(font, "🧱", icon_x + icon_size * 0.15, icon_y + icon_size * 0.75, fs * 0.9, WHITE);
                    }
                    Out::House => {
                        draw_rectangle(icon_x, icon_y, icon_size, icon_size, hx(0x8a5a2e));
                        draw_rectangle_lines(icon_x, icon_y, icon_size, icon_size, 1.5, ha(0x000000, 0.6));
                        text(font, "🏠", icon_x + icon_size * 0.15, icon_y + icon_size * 0.75, fs * 0.9, WHITE);
                    }
                }

                // Назва рецепта
                let text_start_x = icon_x + icon_size + pad * 0.8;
                text(font, texts.recipe_names[ri], text_start_x, y + fs * 1.2, fs * 1.05, if can { WHITE } else { ha(0xffffff, 0.5) });

                // Вартість та умови
                let mut extra = cost_text(r, texts);
                if r.fire {
                    extra.push_str(&format!(" · {}", texts.at_campfire));
                }
                if r.furnace {
                    extra.push_str(&format!(" · {}", texts.at_furnace));
                }
                if matches!(r.out, Out::Tool(t) if g.tool(t)) {
                    extra = texts.already_have.into();
                }
                text(font, &extra, text_start_x, y + fs * 2.45, fs * 0.82, ha(0xffffff, if can { 0.85 } else { 0.45 }));

                // Кнопка створити
                let btn_w = fs * 7.0;
                let btn_h = rh - 12.0;
                ui.button(font, Rect::new(body.x + body.w - btn_w - pad * 0.5, y + 4.0, btn_w, btn_h), texts.make, Cmd::Craft(ri), can, 0x2f7d36, fs);
            }
        }
        2 => {
            // форма виставлення на продаж
            let y = body.y;
            let bh = fs * 2.3;
            let sq = fs * 2.3;
            text(font, &format!("{}:", texts.sell_item), body.x, y + fs * 1.0, fs * 0.9, ha(0xffffff, 0.85));
            let y2 = y + fs * 1.5;
            ui.button(font, Rect::new(body.x, y2, sq, bh), "<", Cmd::MktItem(-1), true, 0x4a4f44, fs);
            text_c(font, texts.item_names[ui.mk_item], body.x + sq + fs * 4.5, y2 + bh * 0.65, fs, WHITE);
            ui.button(font, Rect::new(body.x + sq + fs * 9.0, y2, sq, bh), ">", Cmd::MktItem(1), true, 0x4a4f44, fs);
            let y3 = y2 + bh + 6.0;
            text(font, texts.quantity, body.x, y3 + bh * 0.65, fs * 0.9, WHITE);
            ui.button(font, Rect::new(body.x + fs * 4.0, y3, sq, bh), "-", Cmd::MktQty(-1), true, 0x4a4f44, fs);
            text_c(font, &format!("{}", ui.mk_qty), body.x + fs * 4.0 + sq + fs * 1.5, y3 + bh * 0.65, fs, WHITE);
            ui.button(font, Rect::new(body.x + fs * 4.0 + sq + fs * 3.0, y3, sq, bh), "+", Cmd::MktQty(1), true, 0x4a4f44, fs);
            let y4 = y3 + bh + 6.0;
            text(font, texts.price, body.x, y4 + bh * 0.65, fs * 0.9, WHITE);
            ui.button(font, Rect::new(body.x + fs * 4.0, y4, sq, bh), "-", Cmd::MktPrice(-1), true, 0x4a4f44, fs);
            text_c(font, &format!("{}", ui.mk_price), body.x + fs * 4.0 + sq + fs * 1.5, y4 + bh * 0.65, fs, WHITE);
            ui.button(font, Rect::new(body.x + fs * 4.0 + sq + fs * 3.0, y4, sq, bh), "+", Cmd::MktPrice(1), true, 0x4a4f44, fs);
            let can_post = g.inv[ui.mk_item] >= ui.mk_qty;
            ui.button(font, Rect::new(body.x + fs * 12.0, y3, fs * 8.0, bh * 2.0 + 6.0), texts.post_offer, Cmd::MktPost, can_post, 0x2f7d36, fs);
            text(font, &format!("{}: {}   {}: {}", texts.gold_label, g.gold, texts.you_have, g.inv[ui.mk_item]), body.x + fs * 22.0, y2 + bh * 0.65, fs * 0.9, hx(0xe0c060));
            // список предложений (прокручивается)
            let top = y4 + bh + fs * 1.2;
            text(font, &format!("{}:", texts.player_offers), body.x, top - fs * 0.3, fs * 0.9, ha(0xffffff, 0.85));
            let list = Rect::new(body.x, top, body.w, body.y + body.h - top);
            let rh2 = fs * 2.6;
            let content = market.len() as f32 * rh2;
            ui.scroll = ui.scroll.clamp(0.0, (content - list.h).max(0.0));
            if market.is_empty() {
                text(font, texts.market_empty, list.x, list.y + fs * 1.2, fs * 0.9, ha(0xffffff, 0.6));
            }
            for (row, o) in market.iter().enumerate() {
                let yy = list.y + row as f32 * rh2 - ui.scroll;
                if yy < list.y || yy + rh2 > list.y + list.h {
                    continue;
                }
                let mine = Some(o.seller) == my_id || posted.contains_key(&o.id);
                draw_rectangle(list.x, yy, list.w, rh2 - 4.0, ha(0x2a2d27, 0.9));
                text(font, &format!("{}: {} ×{} {} {} {}", o.seller_name, texts.item_names[(o.item as usize).min(NI - 1)], o.qty, texts.for_gold, o.price, texts.gold_label), list.x + pad, yy + rh2 * 0.62, fs * 0.9, WHITE);
                let can = mine || g.gold >= o.price;
                ui.button(font, Rect::new(list.x + list.w - fs * 6.4, yy + 3.0, fs * 6.0, rh2 - 10.0), if mine { texts.withdraw } else { texts.buy }, if mine { Cmd::MktWithdraw(o.id) } else { Cmd::MktBuy(o.id) }, can, if mine { 0x8a3b2e } else { 0x2f7d36 }, fs * 0.9);
            }
        }
        3 => {
            let mut y = body.y + fs;
            text(font, texts.best_score, body.x, y, fs * 1.1, hx(0xe0c060));
            for (i, (n, s)) in board_score.iter().enumerate() {
                y += fs * 1.4;
                text(font, &format!("{}. {} — {}", i + 1, n, s), body.x + pad, y, fs, WHITE);
            }
            y += fs * 2.4;
            text(font, texts.best_fishers, body.x, y, fs * 1.1, hx(0x6aa0d0));
            for (i, (n, big, cnt)) in board_fish.iter().enumerate() {
                y += fs * 1.4;
                text(font, &format!("{}. {} — {:.2} {} ({} {})", i + 1, n, big, texts.kg_unit, cnt, texts.pcs_unit), body.x + pad, y, fs, WHITE);
            }
            if board_score.is_empty() {
                text(font, texts.leaderboard_pending, body.x, y + fs * 2.0, fs * 0.9, ha(0xffffff, 0.6));
            }
        }
        4 => {
            // === СУЧАСНІ НАЛАШТУВАННЯ (MODERN SETTINGS) ===
            let subtab_h = fs * 2.2;
            let subtabs = [
                ("🌐", texts.general_tab),
                ("🎵", texts.audio_tab),
                ("⌨️", texts.controls_tab),
            ];
            let subtab_w = (body.w - pad * 2.0 - 16.0) / 3.0;
            for (i, (icon, label)) in subtabs.iter().enumerate() {
                let rx = body.x + pad + i as f32 * (subtab_w + 8.0);
                let r = Rect::new(rx, body.y + pad, subtab_w, subtab_h);
                let active = ui.settings_subtab == i as u8;
                ui.button(font, r, &format!("{} {}", icon, label), Cmd::SettingsSubtab(i as u8), true, if active { 0x0284c7 } else { 0x1e293b }, fs * 0.9);
            }

            let sbody_y = body.y + pad + subtab_h + fs * 0.8;

            match ui.settings_subtab {
                0 => {
                    // === ЗАГАЛЬНІ: МОВА ТА СЕРВЕРНІ ДІЇ ===
                    let bh = fs * 2.8;
                    let bw = body.w.min(fs * 22.0);
                    text(font, texts.language, body.x + pad, sbody_y + fs * 0.9, fs * 1.1, WHITE);
                    let lang_names = ["🇺🇦 Українська", "🇬🇧 English", "🇷🇺 Русский"];
                    let current_lang = match settings.language {
                        Language::Ukrainian => 0,
                        Language::English => 1,
                        Language::Russian => 2,
                    };
                    ui.button(font, Rect::new(body.x + pad, sbody_y + fs * 1.5, bw, bh), lang_names[current_lang], Cmd::ChangeLanguage, true, 0x0284c7, fs);

                    let y_actions = sbody_y + fs * 5.2;
                    text(font, texts.actions, body.x + pad, y_actions, fs * 1.05, Color::from_rgba(148, 163, 184, 255));
                    let items: [(&str, Cmd); 4] = [
                        (texts.build_bridge, Cmd::Bridge),
                        (texts.boat_toggle, Cmd::Boat),
                        (texts.tame_wolf, Cmd::Tame),
                        (texts.change_name, Cmd::NameOpen),
                    ];
                    for (i, (l, c)) in items.iter().enumerate() {
                        ui.button(
                            font,
                            Rect::new(body.x + pad, y_actions + fs * 0.8 + i as f32 * (bh + 8.0), bw, bh),
                            l,
                            *c,
                            true,
                            0x334155,
                            fs,
                        );
                    }
                }
                1 => {
                    // === ЗВУК ТА МУЗИКА ===
                    let bh = fs * 2.6;
                    let bw = body.w.min(fs * 22.0);

                    // Музика
                    text(font, &format!("🎵 {}", texts.music), body.x + pad, sbody_y + fs * 0.9, fs * 1.1, WHITE);
                    let toggle_w = fs * 8.0;
                    ui.button(
                        font,
                        Rect::new(body.x + pad, sbody_y + fs * 1.4, toggle_w, bh * 0.9),
                        if settings.music_enabled { texts.yes } else { texts.no },
                        Cmd::ToggleMusic,
                        true,
                        if settings.music_enabled { 0x10b981 } else { 0x64748b },
                        fs,
                    );

                    if settings.music_enabled {
                        let slider_y = sbody_y + fs * 1.4 + bh * 0.9 + fs * 0.8;
                        let slider_w = bw * 0.7;
                        draw_rectangle(body.x + pad, slider_y, slider_w, fs * 1.5, ha(0x0f172a, 0.9));
                        draw_rectangle(body.x + pad, slider_y, slider_w * settings.music_volume, fs * 1.5, hx(0x0ea5e9));
                        draw_rectangle_lines(body.x + pad, slider_y, slider_w, fs * 1.5, 1.5, hx(0x38bdf8));
                        text(font, &format!("{}: {}%", texts.volume, (settings.music_volume * 100.0) as i32), body.x + pad * 1.5, slider_y + fs * 1.1, fs * 0.85, WHITE);

                        let btn_size = fs * 2.0;
                        ui.button(font, Rect::new(body.x + pad + slider_w + fs, slider_y, btn_size, fs * 1.5), "-", Cmd::MusicVolume(-0.1), true, 0x334155, fs);
                        ui.button(font, Rect::new(body.x + pad + slider_w + fs + btn_size + fs * 0.5, slider_y, btn_size, fs * 1.5), "+", Cmd::MusicVolume(0.1), true, 0x334155, fs);
                    }

                    // Звуки
                    let y_sound = if settings.music_enabled { sbody_y + fs * 8.0 } else { sbody_y + fs * 5.0 };
                    text(font, &format!("🔊 {}", texts.sounds), body.x + pad, y_sound, fs * 1.1, WHITE);
                    ui.button(
                        font,
                        Rect::new(body.x + pad, y_sound + fs * 0.5, toggle_w, bh * 0.9),
                        if settings.sounds_enabled { texts.yes } else { texts.no },
                        Cmd::ToggleSound,
                        true,
                        if settings.sounds_enabled { 0x10b981 } else { 0x64748b },
                        fs,
                    );

                    if settings.sounds_enabled {
                        let slider_y = y_sound + fs * 0.5 + bh * 0.9 + fs * 0.8;
                        let slider_w = bw * 0.7;
                        draw_rectangle(body.x + pad, slider_y, slider_w, fs * 1.5, ha(0x0f172a, 0.9));
                        draw_rectangle(body.x + pad, slider_y, slider_w * settings.sound_volume, fs * 1.5, hx(0x0ea5e9));
                        draw_rectangle_lines(body.x + pad, slider_y, slider_w, fs * 1.5, 1.5, hx(0x38bdf8));
                        text(font, &format!("{}: {}%", texts.volume, (settings.sound_volume * 100.0) as i32), body.x + pad * 1.5, slider_y + fs * 1.1, fs * 0.85, WHITE);

                        let btn_size = fs * 2.0;
                        ui.button(font, Rect::new(body.x + pad + slider_w + fs, slider_y, btn_size, fs * 1.5), "-", Cmd::SoundVolume(-0.1), true, 0x334155, fs);
                        ui.button(font, Rect::new(body.x + pad + slider_w + fs + btn_size + fs * 0.5, slider_y, btn_size, fs * 1.5), "+", Cmd::SoundVolume(0.1), true, 0x334155, fs);
                    }
                }
                _ => {
                    // === КЕРУВАННЯ ТА БІНДИ ===
                    text(font, texts.rebind_hint, body.x + pad, sbody_y + fs * 0.6, fs * 0.85, Color::from_rgba(148, 163, 184, 255));

                    let actions = RebindAction::ALL;
                    let cols = 2;
                    let item_w = (body.w - pad * 3.0) / cols as f32;
                    let item_h = fs * 2.4;
                    let start_y = sbody_y + fs * 1.2;

                    for (idx, &action) in actions.iter().enumerate() {
                        let col = idx % cols;
                        let row = idx / cols;
                        let bx = body.x + pad + col as f32 * (item_w + pad);
                        let by = start_y + row as f32 * (item_h + 6.0);

                        let is_rebinding_this = ui.rebinding == Some(action);
                        let current_key = settings.keybindings.get_key_for_action(action);

                        let action_name = match action {
                            RebindAction::Inventory => texts.inventory,
                            RebindAction::Craft => texts.craft,
                            RebindAction::Action => texts.action,
                            RebindAction::Eat => texts.eat,
                            RebindAction::Drink => texts.drink,
                            RebindAction::Dig => texts.dig,
                            RebindAction::Menu => texts.menu,
                            RebindAction::MoveUp => texts.move_up,
                            RebindAction::MoveDown => texts.move_down,
                            RebindAction::MoveLeft => texts.move_left,
                            RebindAction::MoveRight => texts.move_right,
                        };

                        let btn_label = if is_rebinding_this {
                            format!("{} {} [ ... ]", action.icon(), action_name)
                        } else {
                            format!("{} {} [ {} ]", action.icon(), action_name, current_key)
                        };

                        let col_hex = if is_rebinding_this { 0x0284c7 } else { 0x1e293b };
                        ui.button(font, Rect::new(bx, by, item_w, item_h), &btn_label, Cmd::Rebind(action), true, col_hex, fs * 0.85);
                    }

                    // Кнопка скидання біндів
                    let reset_btn_w = fs * 11.0;
                    let reset_btn_h = fs * 2.0;
                    let reset_y = start_y + (actions.len() as f32 / cols as f32).ceil() * (item_h + 6.0) + fs * 0.4;
                    ui.button(font, Rect::new(body.x + (body.w - reset_btn_w) / 2.0, reset_y, reset_btn_w, reset_btn_h), &format!("🔄 {}", texts.reset_defaults), Cmd::ResetKeybinds, true, 0x475569, fs * 0.85);
                }
            }
        }
        5 => {
            // === ВКЛАДКА АКАУНТУ (ACCOUNT TAB) ===
            let mut y = body.y + pad;
            if let Some(acc) = &acc_ui.active {
                // Гравець авторизований
                let card_w = (body.w - pad * 2.0).min(fs * 28.0);
                let card_x = body.x + (body.w - card_w) / 2.0;
                let card_h = (body.h - pad * 2.0).min(fs * 22.0);
                draw_rectangle(card_x, y, card_w, card_h, ha(0x181a17, 0.95));
                draw_rectangle_lines(card_x, y, card_w, card_h, 2.0, hx(0x3e5235));

                // Шапка акаунту
                draw_rectangle(card_x, y, card_w, fs * 2.6, ha(0x233120, 0.9));
                text(font, "ОБЛІКОВИЙ ЗАПИС ГРАВЦЯ", card_x + pad, y + fs * 1.7, fs * 1.1, hx(0x73d16b));
                y += fs * 3.4;

                text(font, &format!("Нікнейм: {}", acc.username), card_x + pad, y, fs * 1.05, WHITE);
                y += fs * 1.6;
                text(font, &format!("Електронна пошта: {}", acc.email), card_x + pad, y, fs * 0.9, hx(0xa1b2c4));
                y += fs * 1.8;

                // Статус синхронізації
                draw_rectangle(card_x + pad, y, card_w - pad * 2.0, fs * 2.0, ha(0x122416, 0.85));
                draw_rectangle_lines(card_x + pad, y, card_w - pad * 2.0, fs * 2.0, 1.0, hx(0x27ae60));
                text(font, "Синхронізація активна (хмарний сервер)", card_x + pad * 1.5, y + fs * 1.35, fs * 0.8, hx(0x2ecc71));
                y += fs * 2.8;

                // Статистика
                text(font, "Збережений хмарний прогрес:", card_x + pad, y, fs * 0.95, hx(0xf1c40f));
                y += fs * 1.5;
                text(font, &format!("Золото: {}   |   Очки: {}", g.gold, g.score()), card_x + pad * 1.5, y, fs * 0.85, WHITE);
                y += fs * 1.4;
                text(font, &format!("Риба: {} (макс. {:.2} кг)   |   Будинки: {}", g.caught, g.biggest, g.houses), card_x + pad * 1.5, y, fs * 0.85, WHITE);
                y += fs * 2.2;

                if let Some(msg) = &acc_ui.status {
                    text(font, msg, card_x + pad, y, fs * 0.85, hx(0x5dade2));
                    y += fs * 1.8;
                }

                // Кнопки дій
                let btn_w = (card_w - pad * 4.0) / 3.0;
                let btn_h = fs * 2.2;
                ui.button(font, Rect::new(card_x + pad, y, btn_w, btn_h), "У хмару", Cmd::AccountSaveCloud, true, 0x27ae60, fs * 0.85);
                let cp_label = if acc_ui.mode == 2 { "Скасувати" } else { "Зміна пароля" };
                ui.button(font, Rect::new(card_x + pad * 2.0 + btn_w, y, btn_w, btn_h), cp_label, Cmd::AccountChangePassword, true, 0x2980b9, fs * 0.85);
                ui.button(font, Rect::new(card_x + pad * 3.0 + btn_w * 2.0, y, btn_w, btn_h), "Вийти", Cmd::AccountLogout, true, 0xc0392b, fs * 0.85);

                if acc_ui.mode == 2 {
                    y += btn_h + pad * 1.2;
                    let field_w = card_w - pad * 2.0;
                    let field_h = fs * 2.0;

                    // Старий пароль
                    let old_active = acc_ui.active_field == 1;
                    let cur_old = if old_active && (now * 2.5) as i64 % 2 == 0 { "|" } else { "" };
                    let old_rect = Rect::new(card_x + pad, y, field_w, field_h);
                    ui.hit_area(old_rect, Cmd::AccountField(1));
                    draw_rectangle(old_rect.x, old_rect.y, old_rect.w, old_rect.h, hx(if old_active { 0x18281d } else { 0x121411 }));
                    draw_rectangle_lines(old_rect.x, old_rect.y, old_rect.w, old_rect.h, if old_active { 2.0 } else { 1.5 }, hx(if old_active { 0x2ecc71 } else { 0x3e4a39 }));
                    if acc_ui.password.is_empty() && !old_active {
                        text(font, "Поточний пароль...", old_rect.x + pad * 1.2, y + fs * 1.35, fs * 0.85, ha(0xffffff, 0.4));
                    } else {
                        let old_masked = "*".repeat(acc_ui.password.len());
                        text(font, &format!("{}{cur_old}", old_masked), old_rect.x + pad * 1.2, y + fs * 1.35, fs * 0.85, WHITE);
                    }
                    y += field_h + pad * 0.8;

                    // Новий пароль
                    let new_active = acc_ui.active_field == 3;
                    let cur_new = if new_active && (now * 2.5) as i64 % 2 == 0 { "|" } else { "" };
                    let new_rect = Rect::new(card_x + pad, y, field_w, field_h);
                    ui.hit_area(new_rect, Cmd::AccountField(3));
                    draw_rectangle(new_rect.x, new_rect.y, new_rect.w, new_rect.h, hx(if new_active { 0x18281d } else { 0x121411 }));
                    draw_rectangle_lines(new_rect.x, new_rect.y, new_rect.w, new_rect.h, if new_active { 2.0 } else { 1.5 }, hx(if new_active { 0x2ecc71 } else { 0x3e4a39 }));
                    if acc_ui.new_password.is_empty() && !new_active {
                        text(font, "Новий пароль (мін. 6 симв.)...", new_rect.x + pad * 1.2, y + fs * 1.35, fs * 0.85, ha(0xffffff, 0.4));
                    } else {
                        let new_masked = "*".repeat(acc_ui.new_password.len());
                        text(font, &format!("{}{cur_new}", new_masked), new_rect.x + pad * 1.2, y + fs * 1.35, fs * 0.85, WHITE);
                    }
                    y += field_h + pad * 0.8;

                    ui.button(font, Rect::new(card_x + pad, y, field_w, fs * 2.2), "Зберегти новий пароль", Cmd::AccountChangePassword, true, 0x27ae60, fs * 0.9);
                }
            } else {
                // Гравець НЕ авторизований: Форма Входу / Реєстрації / Коду з сайту
                let card_w = (body.w - pad * 2.0).min(fs * 28.0);
                let card_x = body.x + (body.w - card_w) / 2.0;
                let card_h = (body.h - pad * 2.0).min(fs * 27.0);
                draw_rectangle(card_x, y, card_w, card_h, ha(0x181a17, 0.95));
                draw_rectangle_lines(card_x, y, card_w, card_h, 2.0, hx(0x4a3a28));

                let field_w = card_w - pad * 2.0;

                // 1. Верхня кнопка швидкого відкриття кабінету / Google в браузері
                ui.button(font, Rect::new(card_x + pad, y + pad, field_w, fs * 2.1), "Увійти через Google (через браузер)", Cmd::AccountGoogleLogin, true, 0x2563eb, fs * 0.85);
                y += pad + fs * 2.1 + pad * 0.6;

                // 2. Три акуратні вкладки: [ Вхід (Email) ] [ Реєстрація ] [ Код з сайту ]
                let tab_gap = pad * 0.5;
                let tab_w = (field_w - tab_gap * 2.0) / 3.0;
                let tab_h = fs * 2.0;
                ui.button(font, Rect::new(card_x + pad, y, tab_w, tab_h), "Вхід (Email)", Cmd::AccountSwitchModeTab(0), true, if acc_ui.mode == 0 { 0x27ae60 } else { 0x2c3328 }, fs * 0.78);
                ui.button(font, Rect::new(card_x + pad + tab_w + tab_gap, y, tab_w, tab_h), "Реєстрація", Cmd::AccountSwitchModeTab(1), true, if acc_ui.mode == 1 { 0x27ae60 } else { 0x2c3328 }, fs * 0.78);
                ui.button(font, Rect::new(card_x + pad + (tab_w + tab_gap) * 2.0, y, tab_w, tab_h), "Код з сайту", Cmd::AccountSwitchModeTab(3), true, if acc_ui.mode == 3 { 0x27ae60 } else { 0x2c3328 }, fs * 0.78);
                y += tab_h + pad * 0.8;

                let field_h = fs * 2.0;

                if acc_ui.mode == 3 {
                    // === ВКЛАДКА: ВХІД ЗА КОДОМ З САЙТУ ===
                    text(font, "Введіть 6-значний код з особистого кабінету:", card_x + pad, y + fs * 0.7, fs * 0.75, ha(0xffffff, 0.85));
                    y += fs * 1.0;

                    let code_active = acc_ui.active_field == 4;
                    let cur4 = if code_active && (now * 2.5) as i64 % 2 == 0 { "|" } else { "" };
                    let code_rect = Rect::new(card_x + pad, y, field_w, field_h);
                    ui.hit_area(code_rect, Cmd::AccountField(4));
                    draw_rectangle(code_rect.x, code_rect.y, code_rect.w, code_rect.h, hx(if code_active { 0x18281d } else { 0x121411 }));
                    draw_rectangle_lines(code_rect.x, code_rect.y, code_rect.w, code_rect.h, if code_active { 2.0 } else { 1.5 }, hx(if code_active { 0x2ecc71 } else { 0x3e4a39 }));
                    if acc_ui.code.is_empty() && !code_active {
                        text(font, "Введіть 6-значний код...", code_rect.x + pad * 1.2, y + fs * 1.35, fs * 0.85, ha(0xffffff, 0.4));
                    } else {
                        text(font, &format!("{}{cur4}", acc_ui.code), code_rect.x + pad * 1.2, y + fs * 1.35, fs * 0.95, if code_active { hx(0x73d16b) } else { WHITE });
                    }
                    y += field_h + pad * 0.8;

                    ui.button(font, Rect::new(card_x + pad, y, field_w, fs * 2.2), "Увійти за кодом", Cmd::AccountCodeSubmit, true, 0x27ae60, fs * 0.9);
                    y += fs * 2.2 + pad * 0.8;

                } else {
                    // === ВКЛАДКА: ВХІД / РЕЄСТРАЦІЯ ЗА EMAIL ===
                    if acc_ui.mode == 1 {
                        text(font, "Ігровий нікнейм:", card_x + pad, y + fs * 0.7, fs * 0.75, ha(0xffffff, 0.8));
                        y += fs * 0.9;
                        let user_active = acc_ui.active_field == 2;
                        let cur2 = if user_active && (now * 2.5) as i64 % 2 == 0 { "|" } else { "" };
                        let user_rect = Rect::new(card_x + pad, y, field_w, field_h);
                        ui.hit_area(user_rect, Cmd::AccountField(2));
                        draw_rectangle(user_rect.x, user_rect.y, user_rect.w, user_rect.h, hx(if user_active { 0x18281d } else { 0x121411 }));
                        draw_rectangle_lines(user_rect.x, user_rect.y, user_rect.w, user_rect.h, if user_active { 2.0 } else { 1.5 }, hx(if user_active { 0x2ecc71 } else { 0x3e4a39 }));
                        if acc_ui.username.is_empty() && !user_active {
                            text(font, "Нікнейм (2-20 символів)...", user_rect.x + pad * 1.2, y + fs * 1.35, fs * 0.85, ha(0xffffff, 0.4));
                        } else {
                            text(font, &format!("{}{cur2}", acc_ui.username), user_rect.x + pad * 1.2, y + fs * 1.35, fs * 0.85, WHITE);
                        }
                        y += field_h + pad * 0.6;
                    }

                    // Електронна пошта
                    text(font, "Електронна пошта:", card_x + pad, y + fs * 0.7, fs * 0.75, ha(0xffffff, 0.8));
                    y += fs * 0.9;
                    let email_active = acc_ui.active_field == 0;
                    let cur0 = if email_active && (now * 2.5) as i64 % 2 == 0 { "|" } else { "" };
                    let email_rect = Rect::new(card_x + pad, y, field_w, field_h);
                    ui.hit_area(email_rect, Cmd::AccountField(0));
                    draw_rectangle(email_rect.x, email_rect.y, email_rect.w, email_rect.h, hx(if email_active { 0x18281d } else { 0x121411 }));
                    draw_rectangle_lines(email_rect.x, email_rect.y, email_rect.w, email_rect.h, if email_active { 2.0 } else { 1.5 }, hx(if email_active { 0x2ecc71 } else { 0x3e4a39 }));
                    if acc_ui.email.is_empty() && !email_active {
                        text(font, "name@example.com", email_rect.x + pad * 1.2, y + fs * 1.35, fs * 0.85, ha(0xffffff, 0.4));
                    } else {
                        text(font, &format!("{}{cur0}", acc_ui.email), email_rect.x + pad * 1.2, y + fs * 1.35, fs * 0.85, WHITE);
                    }
                    y += field_h + pad * 0.6;

                    // Пароль
                    text(font, "Пароль:", card_x + pad, y + fs * 0.7, fs * 0.75, ha(0xffffff, 0.8));
                    y += fs * 0.9;
                    let pass_active = acc_ui.active_field == 1;
                    let cur1 = if pass_active && (now * 2.5) as i64 % 2 == 0 { "|" } else { "" };
                    let pass_rect = Rect::new(card_x + pad, y, field_w, field_h);
                    ui.hit_area(pass_rect, Cmd::AccountField(1));
                    draw_rectangle(pass_rect.x, pass_rect.y, pass_rect.w, pass_rect.h, hx(if pass_active { 0x18281d } else { 0x121411 }));
                    draw_rectangle_lines(pass_rect.x, pass_rect.y, pass_rect.w, pass_rect.h, if pass_active { 2.0 } else { 1.5 }, hx(if pass_active { 0x2ecc71 } else { 0x3e4a39 }));
                    if acc_ui.password.is_empty() && !pass_active {
                        text(font, "Мінімум 6 символів...", pass_rect.x + pad * 1.2, y + fs * 1.35, fs * 0.85, ha(0xffffff, 0.4));
                    } else {
                        let masked = "*".repeat(acc_ui.password.len());
                        text(font, &format!("{}{cur1}", masked), pass_rect.x + pad * 1.2, y + fs * 1.35, fs * 0.85, WHITE);
                    }
                    y += field_h + pad * 0.8;

                    // Кнопка відправки форми
                    let submit_label = if acc_ui.mode == 0 { "Увійти в гру" } else { "Створити акаунт і грати" };
                    ui.button(font, Rect::new(card_x + pad, y, field_w, fs * 2.2), submit_label, Cmd::AccountSubmit, true, 0x27ae60, fs * 0.9);
                    y += fs * 2.2 + pad * 0.8;
                }

                // Статус / Повідомлення про помилку (з чітким відступом, жодних накладань!)
                if let Some(msg) = &acc_ui.status {
                    let col = if msg.starts_with("Помилка") { 0xe74c3c } else { 0x2ecc71 };
                    text(font, msg, card_x + pad, y + fs * 0.7, fs * 0.8, hx(col));
                    y += fs * 1.5;
                }

                // Підказка внизу (без зламаних емодзі)
                text(font, "Збереження золота, предметів та будинку на сервері.", card_x + pad, y + fs * 0.7, fs * 0.7, ha(0xffffff, 0.5));
                y += fs * 0.9;
                text(font, "Грайте з будь-якого пристрою під своїм акаунтом!", card_x + pad, y + fs * 0.7, fs * 0.7, ha(0xffffff, 0.4));
            }
        }
        _ => {}
    }

    // Кнопка "Вийти з гри" внизу меню
    let exit_btn_width = fs * 10.0;
    let exit_btn_height = fs * 2.0;
    let exit_btn_x = p.x + (p.w - exit_btn_width) / 2.0;
    let exit_btn_y = p.y + p.h - exit_btn_height - pad;

    ui.button(
        font,
        Rect::new(exit_btn_x, exit_btn_y, exit_btn_width, exit_btn_height),
        texts.quit,
        Cmd::Quit,
        true,
        0xd94f4f,
        fs,
    );
}

fn snapshot(g: &Game, now: f64, name: &str, posted: &HashMap<u32, (u8, u32)>) -> save::Save {
    let p: Vec<(u8, u32)> = posted.values().copied().collect();
    g.to_save(now, name, &p)
}

#[allow(clippy::too_many_arguments)]
fn draw_name_modal(ui: &mut Ui, font: &Font, buf: &str, lang: usize, sw: f32, sh: f32, fs: f32, now: f64, texts: &Texts) {
    const UA: [&str; 4] = ["1234567890", "йцукенгшщзхї", "фівапролджєґ", "ячсмитьбю'-"];
    const RU: [&str; 4] = ["1234567890", "йцукенгшщзхъ", "фывапролджэё", "ячсмитьбю.-"];
    const EN: [&str; 4] = ["1234567890", "qwertyuiop", "asdfghjkl_", "zxcvbnm.-"];
    let rows: &[&str; 4] = match lang {
        0 => &UA,
        1 => &RU,
        _ => &EN,
    };
    draw_rectangle(0.0, 0.0, sw, sh, ha(0x000000, 0.8));
    let p = Rect::new(sw * 0.03, sh * 0.03, sw * 0.94, sh * 0.94);
    draw_rectangle(p.x, p.y, p.w, p.h, ha(0x1d1f1b, 0.97));
    draw_rectangle_lines(p.x, p.y, p.w, p.h, 3.0, hx(0x6b5a3a));
    text_c(font, texts.what_is_your_name, sw / 2.0, p.y + fs * 2.0, fs * 1.4, WHITE);
    text_c(font, texts.name_modal_desc, sw / 2.0, p.y + fs * 3.4, fs * 0.8, ha(0xffffff, 0.7));
    let f = Rect::new(p.x + fs * 2.0, p.y + fs * 4.2, p.w - fs * 4.0, fs * 2.8);
    draw_rectangle(f.x, f.y, f.w, f.h, ha(0x000000, 0.6));
    draw_rectangle_lines(f.x, f.y, f.w, f.h, 2.0, hx(0x58b85f));
    let cursor = if (now * 2.0) as i64 % 2 == 0 { "|" } else { " " };
    text_c(font, &format!("{buf}{cursor}"), sw / 2.0, f.y + f.h * 0.68, fs * 1.4, WHITE);
    let kw = ((p.w - fs * 2.0) / 12.5).min(fs * 3.4);
    let kh = fs * 2.7;
    let top = f.y + f.h + fs * 1.2;
    for (ri, row) in rows.iter().enumerate() {
        let chars: Vec<char> = row.chars().collect();
        let total = chars.len() as f32 * (kw + 4.0);
        for (ci, ch) in chars.iter().enumerate() {
            let r = Rect::new(sw / 2.0 - total / 2.0 + ci as f32 * (kw + 4.0), top + ri as f32 * (kh + 5.0), kw, kh);
            let up = if buf.is_empty() { ch.to_uppercase().next().unwrap_or(*ch) } else { *ch };
            ui.button(font, r, &up.to_string(), Cmd::Key(*ch), true, 0x3d4238, fs * 1.2);
        }
    }
    let y = top + 4.0 * (kh + 5.0) + fs * 0.4;
    let (lw, bw, ow) = (fs * 5.0, fs * 5.5, fs * 7.0);
    let sp = p.w - fs * 2.0 - lw - bw - ow - 24.0;
    let x0 = p.x + fs;
    ui.button(font, Rect::new(x0, y, lw, kh), ["UA", "RU", "EN"][lang], Cmd::Lang, true, 0x4a6fa5, fs);
    ui.button(font, Rect::new(x0 + lw + 8.0, y, sp, kh), texts.space, Cmd::Space, true, 0x3d4238, fs);
    ui.button(font, Rect::new(x0 + lw + sp + 16.0, y, bw, kh), texts.delete_key, Cmd::Back, true, 0x8a3b2e, fs);
    ui.button(font, Rect::new(x0 + lw + sp + bw + 24.0, y, ow, kh), texts.done, Cmd::NameOk, !buf.trim().is_empty(), 0x2f7d36, fs);
}
