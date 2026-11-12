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

/// Адрес сервера. При сборке можно задать переменной окружения PV_SERVER.
const SERVER_URL: &str = match option_env!("PV_SERVER") {
    Some(u) if !u.is_empty() => u,
    _ => "ws://127.0.0.1:8080/ws",
};
const PLAYER_COLORS: [u32; 5] = [0xd94f4f, 0x4a6fa5, 0xc9a227, 0x8e5bb5, 0x2a9d8f];
const CLIENT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn window_conf() -> Conf {
    Conf {
        window_title: "Pixel Village".to_owned(),
        window_width: 1920,
        window_height: 1080,
        fullscreen: true,
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

fn cost_text(r: &Recipe) -> String {
    r.cost.iter().map(|(it, n)| format!("{} ×{}", IT_NAMES[*it as usize], n)).collect::<Vec<_>>().join(", ")
}

#[macroquad::main(window_conf)]
async fn main() {
    // Перевірка SERVER_URL у release режимі
    #[cfg(not(debug_assertions))]
    {
        if SERVER_URL.is_empty() || SERVER_URL.starts_with("ws://127.0.0.1") || SERVER_URL.starts_with("ws://localhost") {
            panic!("PV_SERVER must be set to a valid remote server for release builds");
        }
    }

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

    let net = Net::start(SERVER_URL.to_string(), name.clone());

    let mut g = Game::new();
    if let Some(s) = &saved {
        g.apply_save(s, get_time());
    }

    game_state.finish_loading();

    // Перевірити оновлення один раз при старті
    game_state.check_updates_once();

    // Первый запуск: спрашиваем имя экранной клавиатурой.
    let mut name_edit: Option<String> = if env_name.is_none() && saved.as_ref().map_or(true, |s| s.name.is_empty()) { Some(name.clone()) } else { None };
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

    loop {
        let dt = get_frame_time().min(0.05);
        let now = get_time();
        let texts = get_texts(&settings.language);
        let (sw, sh) = (screen_width(), screen_height());
        let base = sw.min(sh);
        let ts = (base / 11.0).floor().max(24.0);
        let u = ts / 16.0;
        let k = ts / 48.0;
        let (cx, cy) = (sw / 2.0, sh / 2.0);
        let fs = (base * 0.03).max(14.0);
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

        if is_quit_requested() {
            save::store(&snapshot(&g, now, &name, &posted));
            std::process::exit(0);
        }

        // ---------- сеть ----------
        while let Ok(ev) = net.in_rx.try_recv() {
            match ev {
                NetEvent::Connected => connected = true,
                NetEvent::Disconnected => {
                    if connected {
                        for (_, (item, qty)) in posted.drain() {
                            g.inv[item as usize] += qty;
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
                        g.say("Предложение выставлено", now);
                    }
                    ServerMsg::Sold { id, gold } => {
                        posted.remove(&id);
                        g.gold += gold;
                        g.say(&format!("Продано! +{gold} золота"), now);
                    }
                    ServerMsg::Withdrawn { offer } => {
                        posted.remove(&offer.id);
                        g.inv[(offer.item as usize).min(NI - 1)] += offer.qty;
                        g.say("Предложение снято", now);
                    }
                    ServerMsg::Bought { offer } => {
                        g.gold = g.gold.saturating_sub(offer.price);
                        g.inv[(offer.item as usize).min(NI - 1)] += offer.qty;
                        g.say(&format!("Куплено: {} ×{}", IT_NAMES[(offer.item as usize).min(NI - 1)], offer.qty), now);
                    }
                    ServerMsg::Err { text } => g.say(&text, now),
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
                        } else {
                            cmds.push(c);
                        }
                    } else if ui.menu {
                        if ui.panel.contains(pos) {
                            scroll_id = Some((id, pos.y));
                        }
                    } else if joy_id.is_none() && name_edit.is_none() {
                        joy_id = Some(id);
                        origin = pos;
                        cur = pos;
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
        let typing = name_edit.is_some();
        if let Some(buf) = name_edit.as_mut() {
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
        if !typing && is_key_pressed(KeyCode::M) || is_key_pressed(KeyCode::Tab) {
            cmds.push(if ui.menu { Cmd::Close } else { Cmd::Menu });
        }
        // ---------- ввод ----------
        let typing = name_edit.is_some() || ui.menu;
        for c in cmds {
            match c {
                Cmd::Eat => {
                    g.eat(now, &texts);
                    audio.play("eat");
                }
                Cmd::Drink => {
                    g.drink(now, &texts);
                    audio.play("drink");
                }
                Cmd::Dig => {
                    g.dig(now, &texts);
                    audio.play("hit");
                }
                Cmd::Tame => g.tame(now, &texts),
                Cmd::Boat => g.boat_toggle(now, &texts),
                Cmd::Bridge => g.bridge(now, &texts),
                Cmd::Menu => {
                    ui.menu = true;
                    ui.scroll = 0.0;
                }
                Cmd::Close => ui.menu = false,
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
                    g.craft(i, now);
                    audio.play("craft");
                }
                Cmd::Quick(j) => {
                    if let Some(&i) = g.quick_list(now).get(j) {
                        g.craft(i, now);
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
                        g.say("Рынок доступен только онлайн", now);
                    } else if g.inv[ui.mk_item] < ui.mk_qty {
                        g.say("Не хватает предметов", now);
                    } else {
                        g.inv[ui.mk_item] -= ui.mk_qty;
                        out.push(ClientMsg::Post { item: ui.mk_item as u8, qty: ui.mk_qty, price: ui.mk_price });
                    }
                }
                Cmd::MktBuy(id) => {
                    if let Some(o) = market.iter().find(|o| o.id == id) {
                        if g.gold < o.price {
                            g.say("Не хватает золота", now);
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
            }
        }

        let mut dir = Vec2::ZERO;
        if typing {
            // пока вводим имя, персонаж стоит
        } else if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            dir.x -= 1.0;
        }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            dir.x += 1.0;
        }
        if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
            dir.y -= 1.0;
        }
        if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
            dir.y += 1.0;
        }
        if dir != Vec2::ZERO {
            dir = dir.normalize();
        } else if joy_id.is_some() {
            let v = (cur - origin) / ring_r;
            if v.length() > 0.2 {
                dir = v.clamp_length_max(1.0);
            }
        }
        let hold = !typing && (is_key_down(KeyCode::Space) || is_key_down(KeyCode::E) || act_id.is_some());
        if typing {
            dir = Vec2::ZERO;
        }

        // ---------- логика ----------
        g.tick(dt, now, &Input { dir, hold }, &mut out, &texts);
        let (px, py) = (g.x, g.y);

        // ---------- сеть: отправка ----------
        send_t += dt;
        stats_t += dt;
        save_t += dt;
        if save_t > 8.0 {
            save_t = 0.0;
            save::store(&snapshot(&g, now, &name, &posted));
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
            labels.push(("💀 Твої речі".into(), sx, sy - 10.0 * u));
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
                        labels.push(("Друг".into(), sx, sy - 11.0 * u));
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
        for pass in 0..2 {
            if pass == 0 {
                continue;
            }
            for ty in (me_row + 1)..=y1 {
                for tx in x0..=x1 {
                    // узлы перед персонажем рисуются поверх него
                    if let Some(n) = g.map.node_at(tx, ty) {
                        let (sx, sy) = scr(tx, ty);
                        draw_node(n, sx, sy, u, now);
                    }
                }
            }
        }
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
        for (i, s) in [format!("Золото {}", g.gold), format!("Очки {}", g.score())].iter().enumerate() {
            let w = text_w(&font, s, fs * 0.9);
            draw_rectangle(sw - pad - w - 6.0, my0 + ms + fs * (0.5 + i as f32 * 1.25), w + 12.0, fs * 1.15, ha(0x000000, 0.55));
            text(&font, s, sw - pad - w, my0 + ms + fs * (1.35 + i as f32 * 1.25), fs * 0.9, WHITE);
        }
        // инвентарь
        {
            let max_w = sw * 0.46;
            let (mut x, mut y) = (pad, sh - pad - fs * 1.6);
            let mut chips: Vec<(u32, String)> = vec![];
            for i in 0..NI {
                if g.inv[i] > 0 {
                    chips.push((IT_COLOR[i], format!("{} {}", IT_NAMES[i], g.inv[i])));
                }
            }
            for t in 0..NT {
                if g.tools[t] > 0 {
                    let d = if TL_DUR[t] > 1 { format!("{} ({})", TL_NAMES[t], g.tools[t]) } else { TL_NAMES[t].to_string() };
                    chips.push((0xffffff, d));
                }
            }
            for (c, s) in chips.iter().rev() {
                let w = text_w(&font, s, fs * 0.85) + fs * 1.6;
                if x + w > pad + max_w {
                    x = pad;
                    y -= fs * 1.5;
                }
                draw_rectangle(x, y, w, fs * 1.3, ha(0x000000, 0.6));
                draw_rectangle(x + 3.0, y + 3.0, fs * 0.9, fs * 0.9, hx(*c));
                text(&font, s, x + fs * 1.2, y + fs * 0.95, fs * 0.85, WHITE);
                x += w + 4.0;
            }
        }
        // сообщение
        if now < g.msg_until {
            let w = text_w(&font, &g.msg, fs * 1.1) + fs * 2.0;
            draw_rectangle(cx - w / 2.0, pad + fs * 5.2, w, fs * 1.9, ha(0x000000, 0.7));
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
            text_c(&font, "Держи кнопку — тяни, отпусти — ослабь", cx, my - fs * 0.5, fs * 0.9, WHITE);
        }

        if let Some(buf) = &name_edit {
            draw_name_modal(&mut ui, &font, buf, kb_lang, sw, sh, fs, now);
        } else if !ui.menu {
            // быстрый крафт
            let ql = g.quick_list(now);
            let (qw, qh) = (fs * 7.5, fs * 2.5);
            let n = ql.len().min(5);
            let total = n as f32 * (qw + 6.0);
            if n > 0 {
                text_c(&font, texts.quick_craft, cx, sh - pad - qh - fs * 0.6 - 4.0, fs * 0.8, ha(0xffffff, 0.85));
            }
            for (j, &ri) in ql.iter().take(5).enumerate() {
                let r = Rect::new(cx - total / 2.0 + j as f32 * (qw + 6.0), sh - pad - qh, qw, qh);
                ui.button(&font, r, &format!("{} {}", j + 1, RECIPES[ri].name), Cmd::Quick(j), true, 0x2f7d36, fs * 0.95);
            }
            // кнопки действий справа
            let ab = base * 0.2;
            let ar = Rect::new(sw - ab - pad, sh - ab - pad - qh * 0.2, ab, ab);
            draw_circle(ar.x + ab / 2.0, ar.y + ab / 2.0, ab / 2.0, ha(if hold { 0x2f7d36 } else { 0x58b85f }, 0.9));
            draw_circle_lines(ar.x + ab / 2.0, ar.y + ab / 2.0, ab / 2.0, 3.0, ha(0x1f5a25, 0.9));
            text_c(&font, texts.action, ar.x + ab / 2.0, ar.y + ab / 2.0 + fs * 0.35, fs * 1.1, WHITE);
            ui.btns_push_circle(ar, Cmd::Act);
            let (bw2, bh2) = (ab * 0.62, fs * 2.3);
            for (i, (l, c)) in [
                (texts.menu, Cmd::Menu),
                (texts.dig, Cmd::Dig),
                (texts.drink, Cmd::Drink),
                (texts.eat, Cmd::Eat)
            ].iter().enumerate() {
                let r = Rect::new(sw - bw2 - pad, ar.y - (i as f32 + 1.0) * (bh2 + 6.0), bw2, bh2);
                ui.button(&font, r, l, *c, true, if *c == Cmd::Menu { 0x4a6fa5 } else { 0x6b5a3a }, fs);
            }
        } else {
            draw_menu(&mut ui, &font, &g, now, sw, sh, fs, &market, &posted, my_id, &board_score, &board_fish, &settings, texts);
        }
        // джойстик
        if joy_id.is_some() && !ui.menu {
            draw_circle_lines(origin.x, origin.y, ring_r, 3.0, ha(0xffffff, 0.55));
            let v = (cur - origin).clamp_length_max(ring_r);
            draw_circle(origin.x + v.x, origin.y + v.y, ring_r * 0.42, ha(0xffffff, 0.35));
        }
        ui.finish();

        // Показати вікно оновлення (якщо є)
        game_state.draw_update_window(&font);

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
) {
    let pad = fs * 0.6;
    let p = Rect::new(sw * 0.03, sh * 0.03, sw * 0.94, sh * 0.94);
    ui.panel = p;
    draw_rectangle(p.x, p.y, p.w, p.h, ha(0x1d1f1b, 0.95));
    draw_rectangle_lines(p.x, p.y, p.w, p.h, 3.0, hx(0x6b5a3a));
    // вкладки
    let tabs = [texts.craft, texts.market, texts.leaderboard, texts.settings];
    let tw = (p.w - pad * 3.0 - fs * 6.5 - 16.0) / 4.0;
    for (i, t) in tabs.iter().enumerate() {
        let r = Rect::new(p.x + pad + i as f32 * (tw + 4.0), p.y + pad, tw, fs * 2.4);
        ui.button(font, r, t, Cmd::Tab(i as u8), true, if ui.tab == i as u8 { 0x2f7d36 } else { 0x4a4f44 }, fs);
    }
    ui.button(font, Rect::new(p.x + p.w - pad - fs * 6.5, p.y + pad, fs * 6.5, fs * 2.4), texts.close, Cmd::Close, true, 0x8a3b2e, fs);
    let body = Rect::new(p.x + pad, p.y + pad * 2.0 + fs * 2.4, p.w - pad * 2.0, p.h - pad * 3.0 - fs * 2.4);
    let rh = fs * 3.0;
    match ui.tab {
        0 => {
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
                draw_rectangle(body.x, y, body.w, rh - 4.0, ha(if can { 0x2f4a30 } else { 0x2a2d27 }, 0.9));
                text(font, r.name, body.x + pad, y + fs * 1.15, fs * 1.05, if can { WHITE } else { ha(0xffffff, 0.55) });
                let mut extra = cost_text(r);
                if r.fire {
                    extra.push_str(" · у костра");
                }
                if r.furnace {
                    extra.push_str(" · у печи");
                }
                if matches!(r.out, Out::Tool(t) if g.tool(t)) {
                    extra = "уже есть".into();
                }
                text(font, &extra, body.x + pad, y + fs * 2.35, fs * 0.8, ha(0xffffff, if can { 0.8 } else { 0.45 }));
                ui.button(font, Rect::new(body.x + body.w - fs * 7.0, y + 4.0, fs * 6.6, rh - 12.0), "Сделать", Cmd::Craft(ri), can, 0x2f7d36, fs);
            }
        }
        1 => {
            // форма выставления
            let y = body.y;
            let bh = fs * 2.3;
            let sq = fs * 2.3;
            text(font, "Выставить на продажу:", body.x, y + fs * 1.0, fs * 0.9, ha(0xffffff, 0.85));
            let y2 = y + fs * 1.5;
            ui.button(font, Rect::new(body.x, y2, sq, bh), "<", Cmd::MktItem(-1), true, 0x4a4f44, fs);
            text_c(font, IT_NAMES[ui.mk_item], body.x + sq + fs * 4.5, y2 + bh * 0.65, fs, WHITE);
            ui.button(font, Rect::new(body.x + sq + fs * 9.0, y2, sq, bh), ">", Cmd::MktItem(1), true, 0x4a4f44, fs);
            let y3 = y2 + bh + 6.0;
            text(font, "Кол-во", body.x, y3 + bh * 0.65, fs * 0.9, WHITE);
            ui.button(font, Rect::new(body.x + fs * 4.0, y3, sq, bh), "-", Cmd::MktQty(-1), true, 0x4a4f44, fs);
            text_c(font, &format!("{}", ui.mk_qty), body.x + fs * 4.0 + sq + fs * 1.5, y3 + bh * 0.65, fs, WHITE);
            ui.button(font, Rect::new(body.x + fs * 4.0 + sq + fs * 3.0, y3, sq, bh), "+", Cmd::MktQty(1), true, 0x4a4f44, fs);
            let y4 = y3 + bh + 6.0;
            text(font, "Цена", body.x, y4 + bh * 0.65, fs * 0.9, WHITE);
            ui.button(font, Rect::new(body.x + fs * 4.0, y4, sq, bh), "-", Cmd::MktPrice(-1), true, 0x4a4f44, fs);
            text_c(font, &format!("{}", ui.mk_price), body.x + fs * 4.0 + sq + fs * 1.5, y4 + bh * 0.65, fs, WHITE);
            ui.button(font, Rect::new(body.x + fs * 4.0 + sq + fs * 3.0, y4, sq, bh), "+", Cmd::MktPrice(1), true, 0x4a4f44, fs);
            let can_post = g.inv[ui.mk_item] >= ui.mk_qty;
            ui.button(font, Rect::new(body.x + fs * 12.0, y3, fs * 8.0, bh * 2.0 + 6.0), "Выставить", Cmd::MktPost, can_post, 0x2f7d36, fs);
            text(font, &format!("Золото: {}   У тебя: {}", g.gold, g.inv[ui.mk_item]), body.x + fs * 22.0, y2 + bh * 0.65, fs * 0.9, hx(0xe0c060));
            // список предложений (прокручивается)
            let top = y4 + bh + fs * 1.2;
            text(font, "Предложения игроков:", body.x, top - fs * 0.3, fs * 0.9, ha(0xffffff, 0.85));
            let list = Rect::new(body.x, top, body.w, body.y + body.h - top);
            let rh2 = fs * 2.6;
            let content = market.len() as f32 * rh2;
            ui.scroll = ui.scroll.clamp(0.0, (content - list.h).max(0.0));
            if market.is_empty() {
                text(font, "Пока пусто (рынок работает онлайн)", list.x, list.y + fs * 1.2, fs * 0.9, ha(0xffffff, 0.6));
            }
            for (row, o) in market.iter().enumerate() {
                let yy = list.y + row as f32 * rh2 - ui.scroll;
                if yy < list.y || yy + rh2 > list.y + list.h {
                    continue;
                }
                let mine = Some(o.seller) == my_id || posted.contains_key(&o.id);
                draw_rectangle(list.x, yy, list.w, rh2 - 4.0, ha(0x2a2d27, 0.9));
                text(font, &format!("{}: {} ×{} за {} зол.", o.seller_name, IT_NAMES[(o.item as usize).min(NI - 1)], o.qty, o.price), list.x + pad, yy + rh2 * 0.62, fs * 0.9, WHITE);
                let can = mine || g.gold >= o.price;
                ui.button(font, Rect::new(list.x + list.w - fs * 6.4, yy + 3.0, fs * 6.0, rh2 - 10.0), if mine { "Снять" } else { "Купить" }, if mine { Cmd::MktWithdraw(o.id) } else { Cmd::MktBuy(o.id) }, can, if mine { 0x8a3b2e } else { 0x2f7d36 }, fs * 0.9);
            }
        }
        2 => {
            let mut y = body.y + fs;
            text(font, "Лучшие по очкам", body.x, y, fs * 1.1, hx(0xe0c060));
            for (i, (n, s)) in board_score.iter().enumerate() {
                y += fs * 1.4;
                text(font, &format!("{}. {} — {}", i + 1, n, s), body.x + pad, y, fs, WHITE);
            }
            y += fs * 2.4;
            text(font, "Лучшие рыбаки (самая крупная рыба)", body.x, y, fs * 1.1, hx(0x6aa0d0));
            for (i, (n, big, cnt)) in board_fish.iter().enumerate() {
                y += fs * 1.4;
                text(font, &format!("{}. {} — {:.2} кг ({} шт.)", i + 1, n, big, cnt), body.x + pad, y, fs, WHITE);
            }
            if board_score.is_empty() {
                text(font, "Рейтинг появится, когда сервер пришлёт данные", body.x, y + fs * 2.0, fs * 0.9, ha(0xffffff, 0.6));
            }
        }
        _ => {
            // Вкладка "Налаштування"
            let bh = fs * 2.8;
            let bw = body.w.min(fs * 22.0);
            let y_start = body.y + fs;

            // === МОВА ===
            text(font, texts.language, body.x + pad, y_start, fs * 1.1, WHITE);
            let lang_names = ["🇺🇦 Українська", "🇬🇧 English", "🇷🇺 Русский"];
            let current_lang = match settings.language {
                Language::Ukrainian => 0,
                Language::English => 1,
                Language::Russian => 2,
            };
            ui.button(
                font,
                Rect::new(body.x, y_start + fs * 1.5, bw, bh),
                lang_names[current_lang],
                Cmd::ChangeLanguage,
                true,
                0x4a6fa5,
                fs,
            );

            // === МУЗИКА ===
            let y_music = y_start + fs * 5.5;
            text(font, &format!("🎵 {}", texts.music), body.x + pad, y_music, fs * 1.1, WHITE);

            // Перемикач вкл/викл
            let toggle_w = fs * 8.0;
            ui.button(
                font,
                Rect::new(body.x, y_music + fs * 1.5, toggle_w, bh * 0.9),
                if settings.music_enabled { texts.yes } else { texts.no },
                Cmd::ToggleMusic,
                true,
                if settings.music_enabled { 0x2f7d36 } else { 0x8a3b2e },
                fs,
            );

            // Повзунок гучності
            if settings.music_enabled {
                let slider_y = y_music + fs * 1.5 + bh * 0.9 + fs * 0.8;
                let slider_w = bw * 0.7;
                draw_rectangle(body.x, slider_y, slider_w, fs * 1.5, ha(0x000000, 0.4));
                draw_rectangle(body.x, slider_y, slider_w * settings.music_volume, fs * 1.5, hx(0x2f7d36));
                draw_rectangle_lines(body.x, slider_y, slider_w, fs * 1.5, 2.0, hx(0x4a6fa5));
                text(font, &format!("{}: {}%", texts.volume, (settings.music_volume * 100.0) as i32), body.x + pad, slider_y + fs * 1.1, fs * 0.9, WHITE);

                // Кнопки +/-
                let btn_size = fs * 2.0;
                ui.button(font, Rect::new(body.x + slider_w + fs, slider_y, btn_size, fs * 1.5), "-", Cmd::MusicVolume(-0.1), true, 0x6b5a3a, fs);
                ui.button(font, Rect::new(body.x + slider_w + fs + btn_size + fs * 0.5, slider_y, btn_size, fs * 1.5), "+", Cmd::MusicVolume(0.1), true, 0x6b5a3a, fs);
            }

            // === ЗВУКИ ===
            let y_sound = if settings.music_enabled { y_music + fs * 7.0 } else { y_music + fs * 4.0 };
            text(font, &format!("🔊 {}", texts.sounds), body.x + pad, y_sound, fs * 1.1, WHITE);

            // Перемикач вкл/викл
            ui.button(
                font,
                Rect::new(body.x, y_sound + fs * 1.5, toggle_w, bh * 0.9),
                if settings.sounds_enabled { texts.yes } else { texts.no },
                Cmd::ToggleSound,
                true,
                if settings.sounds_enabled { 0x2f7d36 } else { 0x8a3b2e },
                fs,
            );

            // Повзунок гучності
            if settings.sounds_enabled {
                let slider_y = y_sound + fs * 1.5 + bh * 0.9 + fs * 0.8;
                let slider_w = bw * 0.7;
                draw_rectangle(body.x, slider_y, slider_w, fs * 1.5, ha(0x000000, 0.4));
                draw_rectangle(body.x, slider_y, slider_w * settings.sound_volume, fs * 1.5, hx(0x2f7d36));
                draw_rectangle_lines(body.x, slider_y, slider_w, fs * 1.5, 2.0, hx(0x4a6fa5));
                text(font, &format!("{}: {}%", texts.volume, (settings.sound_volume * 100.0) as i32), body.x + pad, slider_y + fs * 1.1, fs * 0.9, WHITE);

                // Кнопки +/-
                let btn_size = fs * 2.0;
                ui.button(font, Rect::new(body.x + slider_w + fs, slider_y, btn_size, fs * 1.5), "-", Cmd::SoundVolume(-0.1), true, 0x6b5a3a, fs);
                ui.button(font, Rect::new(body.x + slider_w + fs + btn_size + fs * 0.5, slider_y, btn_size, fs * 1.5), "+", Cmd::SoundVolume(0.1), true, 0x6b5a3a, fs);
            }

            // === ІНШІ ДІЇ ===
            let y_actions = if settings.music_enabled && settings.sounds_enabled {
                y_sound + fs * 7.0
            } else if settings.music_enabled || settings.sounds_enabled {
                y_sound + fs * 4.0
            } else {
                y_sound + fs * 3.5
            };

            let items: [(&str, Cmd); 4] = [
                ("🌉 Построить мост (2 доски)", Cmd::Bridge),
                ("🚣 Лодка: сесть / выйти", Cmd::Boat),
                ("🐺 Приручить волка (мясо)", Cmd::Tame),
                ("✏️ Сменить имя", Cmd::NameOpen),
            ];
            for (i, (l, c)) in items.iter().enumerate() {
                ui.button(
                    font,
                    Rect::new(body.x, y_actions + i as f32 * (bh + 8.0), bw, bh),
                    l,
                    *c,
                    true,
                    0x4a6fa5,
                    fs,
                );
            }
        }
    }
}

fn snapshot(g: &Game, now: f64, name: &str, posted: &HashMap<u32, (u8, u32)>) -> save::Save {
    let p: Vec<(u8, u32)> = posted.values().copied().collect();
    g.to_save(now, name, &p)
}

#[allow(clippy::too_many_arguments)]
fn draw_name_modal(ui: &mut Ui, font: &Font, buf: &str, lang: usize, sw: f32, sh: f32, fs: f32, now: f64) {
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
    text_c(font, "Как тебя зовут?", sw / 2.0, p.y + fs * 2.0, fs * 1.4, WHITE);
    text_c(font, "Имя увидят другие игроки. Можно печатать и с клавиатуры.", sw / 2.0, p.y + fs * 3.4, fs * 0.8, ha(0xffffff, 0.7));
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
    ui.button(font, Rect::new(x0, y, lw, kh), ["Язык UA", "Язык RU", "Язык EN"][lang], Cmd::Lang, true, 0x4a6fa5, fs);
    ui.button(font, Rect::new(x0 + lw + 8.0, y, sp, kh), "Пробел", Cmd::Space, true, 0x3d4238, fs);
    ui.button(font, Rect::new(x0 + lw + sp + 16.0, y, bw, kh), "Стереть", Cmd::Back, true, 0x8a3b2e, fs);
    ui.button(font, Rect::new(x0 + lw + sp + bw + 24.0, y, ow, kh), "Готово", Cmd::NameOk, !buf.trim().is_empty(), 0x2f7d36, fs);
}
