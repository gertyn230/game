//! Сервер Pixel Village: игроки, общий мир, рынок, рейтинги + страница «кто онлайн».
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use shared::{
    ClientMsg, DeviceInfo, Offer, PlayerState, ServerMsg, ITEM_COUNT, MAX_NAME_CHARS, NODE_COUNT_MAX, RESPAWN_SECS, WORLD_SIZE,
};
use tokio::sync::{broadcast, mpsc, Mutex};

const MAX_PLAYERS: usize = 200;
const MAX_OFFERS: usize = 300;

struct Conn {
    state: PlayerState,
    joined: Instant,
    tx: mpsc::UnboundedSender<String>,
    score: u32,
    big: f32,
    fish: u32,
    device: Option<DeviceInfo>,
}

#[derive(Default)]
struct World {
    players: HashMap<u32, Conn>,
    next_id: u32,
    dead: HashMap<u32, Instant>,
    offers: Vec<Offer>,
    next_offer: u32,
}

#[derive(Clone)]
struct App {
    world: Arc<Mutex<World>>,
    tx: broadcast::Sender<String>,
    admin_key: Option<String>,
}

fn enc(m: &ServerMsg) -> String {
    serde_json::to_string(m).unwrap_or_default()
}

#[tokio::main]
async fn main() {
    let (tx, _) = broadcast::channel::<String>(32);
    let app = App {
        world: Arc::new(Mutex::new(World::default())),
        tx,
        admin_key: std::env::var("ADMIN_KEY").ok().filter(|k| !k.is_empty()),
    };

    // Снимок мира ~15 раз/с, таблицы лидеров каждые ~5 с.
    {
        let app = app.clone();
        tokio::spawn(async move {
            let mut iv = tokio::time::interval(Duration::from_millis(66));
            let mut tick = 0u32;
            loop {
                iv.tick().await;
                tick += 1;
                if app.tx.receiver_count() == 0 {
                    continue;
                }
                let (players, board) = {
                    let mut w = app.world.lock().await;
                    let players: Vec<PlayerState> = w.players.values().map(|c| c.state.clone()).collect();
                    let board = if tick % 75 == 0 {
                        w.dead.retain(|_, t| t.elapsed().as_secs() < RESPAWN_SECS as u64);
                        let mut sc: Vec<(String, u32)> = w.players.values().map(|c| (c.state.name.clone(), c.score)).collect();
                        sc.sort_by(|a, b| b.1.cmp(&a.1));
                        sc.truncate(10);
                        let mut fi: Vec<(String, f32, u32)> =
                            w.players.values().filter(|c| c.big > 0.0).map(|c| (c.state.name.clone(), c.big, c.fish)).collect();
                        fi.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                        fi.truncate(5);
                        Some(ServerMsg::Board { score: sc, fishers: fi })
                    } else {
                        None
                    };
                    (players, board)
                };
                let _ = app.tx.send(enc(&ServerMsg::Snapshot { players }));
                if let Some(b) = board {
                    let _ = app.tx.send(enc(&b));
                }
            }
        });
    }

    let router = Router::new()
        .route("/", get(index))
        .route("/admin", get(admin))
        .route("/api/players", get(api_players))
        .route("/api/version", get(api_version))
        .route("/ws", get(ws_handler))
        .with_state(app);

    let port: u16 = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Pixel Village server: http://{addr}  (WebSocket: /ws)");
    let listener = tokio::net::TcpListener::bind(addr).await.expect("порт занят");
    axum::serve(listener, router).await.expect("сервер упал");
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&#39;")
}

fn page(title: &str, body: &str) -> Html<String> {
    Html(format!(
        "<!doctype html><html lang=\"ru\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
         <meta http-equiv=\"refresh\" content=\"3\"><title>{title}</title>\
         <style>body{{font:16px system-ui;background:#1d1f1b;color:#ece8dc;max-width:720px;margin:24px auto;padding:0 12px}}\
         table{{border-collapse:collapse;width:100%}}td,th{{border-bottom:1px solid #43473e;padding:6px;text-align:left}}</style>\
         </head><body>{body}</body></html>"
    ))
}

async fn index(State(app): State<App>) -> Html<String> {
    let w = app.world.lock().await;
    let mut names: Vec<String> = w.players.values().map(|c| esc(&c.state.name)).collect();
    names.sort();
    let rows: String = names.iter().map(|n| format!("<li>{n}</li>")).collect();
    page("Pixel Village: онлайн", &format!("<h1>Онлайн: {}</h1><ul>{rows}</ul>", names.len()))
}

async fn admin(State(app): State<App>, Query(q): Query<HashMap<String, String>>) -> Response {
    match (&app.admin_key, q.get("key")) {
        (Some(k), Some(given)) if k == given => {}
        _ => {
            return (StatusCode::UNAUTHORIZED, "Нужен ключ: /admin?key=ВАШ_КЛЮЧ (задаётся переменной ADMIN_KEY)")
                .into_response()
        }
    }
    let w = app.world.lock().await;
    let mut list: Vec<&Conn> = w.players.values().collect();
    list.sort_by_key(|c| c.state.id);

    // Статистика по платформам
    let mut os_stats: HashMap<String, usize> = HashMap::new();
    for conn in &list {
        if let Some(dev) = &conn.device {
            *os_stats.entry(dev.os.clone()).or_insert(0) += 1;
        }
    }

    let rows: String = list
        .iter()
        .map(|c| {
            let device = if let Some(d) = &c.device {
                format!("{} {} ({})", esc(&d.os), esc(&d.version), esc(&d.arch))
            } else {
                "Невідомо".to_string()
            };
            format!(
                "<tr><td>{}</td><td>{}</td><td>{:.1}, {:.1}</td><td>{}</td><td>{:.2} кг</td><td>{} с</td><td>{}</td></tr>",
                c.state.id,
                esc(&c.state.name),
                c.state.x,
                c.state.y,
                c.score,
                c.big,
                c.joined.elapsed().as_secs(),
                device
            )
        })
        .collect();

    let os_rows: String = os_stats
        .iter()
        .map(|(os, count)| format!("<tr><td>{}</td><td>{}</td></tr>", esc(os), count))
        .collect();

    let offers: String = w
        .offers
        .iter()
        .map(|o| format!("<tr><td>{}</td><td>{}</td><td>{} ×{}</td><td>{}</td></tr>", o.id, esc(&o.seller_name), o.item, o.qty, o.price))
        .collect();

    Html(format!(
        "<!doctype html><html lang=\"uk\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
         <meta http-equiv=\"refresh\" content=\"5\"><title>Pixel Village: Адмінка</title>\
         <style>\
         body{{font:14px system-ui;background:#0a0e14;color:#d9d7ce;margin:0;padding:20px}}\
         .container{{max-width:1400px;margin:0 auto}}\
         h1{{color:#ffb454;margin:0 0 20px 0;font-size:28px}}\
         h2{{color:#59c2ff;margin:30px 0 10px 0;font-size:20px}}\
         .stats{{display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:15px;margin:20px 0}}\
         .stat-card{{background:#151a21;padding:20px;border-radius:8px;border:1px solid #2d3640}}\
         .stat-value{{font-size:32px;font-weight:bold;color:#ffb454;margin:10px 0}}\
         .stat-label{{color:#7d8590;font-size:12px;text-transform:uppercase}}\
         table{{border-collapse:collapse;width:100%;background:#151a21;margin:10px 0;border-radius:8px;overflow:hidden}}\
         td,th{{border-bottom:1px solid #2d3640;padding:12px;text-align:left}}\
         th{{background:#1c2128;color:#59c2ff;font-weight:600}}\
         tr:hover{{background:#1c2128}}\
         .online{{color:#3fb950}}\
         .platform{{display:inline-block;padding:4px 8px;background:#2d3640;border-radius:4px;font-size:11px;color:#7d8590}}\
         </style>\
         </head><body><div class=\"container\">\
         <h1>🎮 Pixel Village — Адмінка</h1>\
         <div class=\"stats\">\
         <div class=\"stat-card\"><div class=\"stat-label\">Гравців онлайн</div><div class=\"stat-value online\">{}</div></div>\
         <div class=\"stat-card\"><div class=\"stat-label\">Активних оголошень</div><div class=\"stat-value\">{}</div></div>\
         <div class=\"stat-card\"><div class=\"stat-label\">Платформ</div><div class=\"stat-value\">{}</div></div>\
         </div>\
         <h2>👥 Гравці онлайн</h2>\
         <table><tr><th>ID</th><th>Ім'я</th><th>Позиція</th><th>Очки</th><th>Рекорд риби</th><th>В грі</th><th>Пристрій</th></tr>{}</table>\
         <h2>📊 Статистика по платформах</h2>\
         <table><tr><th>Платформа</th><th>Гравців</th></tr>{}</table>\
         <h2>🛒 Ринок</h2>\
         <table><tr><th>№</th><th>Продавець</th><th>Предмет</th><th>Ціна</th></tr>{}</table>\
         </div></body></html>",
        list.len(),
        w.offers.len(),
        os_stats.len(),
        rows,
        os_rows,
        offers
    ))
    .into_response()
}

async fn api_version() -> Json<serde_json::Value> {
    // Версія гри - змінюйте при кожному оновленні
    Json(serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "date": "2026-11-07",
        "changes": [
            "Автоматичне оновлення через GitHub Releases",
            "Карта збільшена до 80x80 (+300%)",
            "Додано 2 нові біоми: болото та пустеля",
            "3 нові тварини: олень, ведмідь, лисиця",
            "Нові ресурси: ягоди, гриби, лікарські трави",
            "Нові руди: срібло та золото",
            "Нові рецепти: ягідний пирог, грибний суп, целебний отвар",
            "+13 нових тварин на карті",
            "+7 нових предметів"
        ],
        "download": "https://github.com/gertyn230/game/releases/latest"
    }))
}

async fn api_players(State(app): State<App>) -> Json<serde_json::Value> {
    let w = app.world.lock().await;
    let names: Vec<&str> = w.players.values().map(|c| c.state.name.as_str()).collect();
    Json(serde_json::json!({ "count": names.len(), "players": names }))
}

async fn ws_handler(ws: WebSocketUpgrade, State(app): State<App>) -> impl IntoResponse {
    ws.max_message_size(4096).on_upgrade(move |socket| client_loop(socket, app))
}

fn clean_name(raw: &str) -> Option<String> {
    let s: String = raw.chars().filter(|c| !c.is_control()).take(MAX_NAME_CHARS).collect();
    let s = s.trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

async fn client_loop(socket: WebSocket, app: App) {
    let (mut sink, mut stream) = socket.split();
    let mut rx = app.tx.subscribe();
    let (dtx, mut drx) = mpsc::unbounded_channel::<String>();

    let (id, welcome_pack) = {
        let mut w = app.world.lock().await;
        if w.players.len() >= MAX_PLAYERS {
            return;
        }
        w.next_id += 1;
        let id = w.next_id;
        w.players.insert(
            id,
            Conn {
                state: PlayerState {
                    id,
                    name: format!("Игрок{id}"),
                    x: 20.5,
                    y: 20.5,
                    face: 0,
                    moving: false,
                    color: (id % 5) as u8,
                    boat: false,
                    coat: false,
                    held: 0,
                },
                joined: Instant::now(),
                tx: dtx,
                score: 0,
                big: 0.0,
                fish: 0,
                device: None,
            },
        );
        let dead: Vec<(u32, u32)> = w
            .dead
            .iter()
            .filter_map(|(i, t)| {
                let e = t.elapsed().as_secs() as u32;
                (e < RESPAWN_SECS).then_some((*i, RESPAWN_SECS - e))
            })
            .collect();
        (id, vec![enc(&ServerMsg::Welcome { id, version: env!("CARGO_PKG_VERSION").to_string() }), enc(&ServerMsg::World { dead }), enc(&ServerMsg::Market { offers: w.offers.clone() })])
    };

    let mut ok = true;
    for m in welcome_pack {
        if sink.send(Message::Text(m)).await.is_err() {
            ok = false;
            break;
        }
    }
    if ok {
        loop {
            tokio::select! {
                msg = stream.next() => match msg {
                    Some(Ok(Message::Text(t))) => handle_msg(&app, id, &t).await,
                    Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                    _ => {}
                },
                snap = rx.recv() => match snap {
                    Ok(s) => { if sink.send(Message::Text(s)).await.is_err() { break; } }
                    Err(broadcast::error::RecvError::Lagged(_)) => {}
                    Err(_) => break,
                },
                direct = drx.recv() => match direct {
                    Some(s) => { if sink.send(Message::Text(s)).await.is_err() { break; } }
                    None => break,
                },
            }
        }
    }

    // Игрок вышел: его предложения снимаются (клиент вернёт предметы себе сам).
    let mut w = app.world.lock().await;
    w.players.remove(&id);
    let before = w.offers.len();
    w.offers.retain(|o| o.seller != id);
    if w.offers.len() != before {
        let _ = app.tx.send(enc(&ServerMsg::Market { offers: w.offers.clone() }));
    }
}

fn reply(conn: &Conn, m: &ServerMsg) {
    let _ = conn.tx.send(enc(m));
}

async fn handle_msg(app: &App, id: u32, text: &str) {
    let Ok(msg) = serde_json::from_str::<ClientMsg>(text) else { return };
    let mut w = app.world.lock().await;
    if !w.players.contains_key(&id) {
        return;
    }
    match msg {
        ClientMsg::Join { name, device_info } => {
            if let Some(n) = clean_name(&name) {
                if let Some(c) = w.players.get_mut(&id) {
                    c.state.name = n;
                    c.device = device_info;
                }
            }
        }
        ClientMsg::Move { x, y, face, moving, boat, coat, held } => {
            if x.is_finite() && y.is_finite() {
                if let Some(c) = w.players.get_mut(&id) {
                    c.state.x = x.clamp(0.0, WORLD_SIZE);
                    c.state.y = y.clamp(0.0, WORLD_SIZE);
                    c.state.face = face.min(3);
                    c.state.moving = moving;
                    c.state.boat = boat;
                    c.state.coat = coat;
                    c.state.held = held.min(3);
                }
            }
        }
        ClientMsg::Dead { idx } => {
            if idx < NODE_COUNT_MAX {
                w.dead.insert(idx, Instant::now());
                let _ = app.tx.send(enc(&ServerMsg::Dead { idx, left: RESPAWN_SECS }));
            }
        }
        ClientMsg::Stats { score, big, fish } => {
            if let Some(c) = w.players.get_mut(&id) {
                c.score = score;
                c.big = if big.is_finite() { big.clamp(0.0, 50.0) } else { 0.0 };
                c.fish = fish;
            }
        }
        ClientMsg::Post { item, qty, price } => {
            if item >= ITEM_COUNT || qty == 0 || qty > 999 || price == 0 || price > 100_000 {
                return;
            }
            if w.offers.len() >= MAX_OFFERS {
                if let Some(c) = w.players.get(&id) {
                    reply(c, &ServerMsg::Err { text: "Рынок переполнен".into() });
                }
                return;
            }
            w.next_offer += 1;
            let name = w.players.get(&id).map(|c| c.state.name.clone()).unwrap_or_default();
            let offer = Offer { id: w.next_offer, seller: id, seller_name: name, item, qty, price };
            w.offers.push(offer.clone());
            if let Some(c) = w.players.get(&id) {
                reply(c, &ServerMsg::Posted { offer });
            }
            let _ = app.tx.send(enc(&ServerMsg::Market { offers: w.offers.clone() }));
        }
        ClientMsg::Buy { id: oid } => {
            let Some(pos) = w.offers.iter().position(|o| o.id == oid) else {
                if let Some(c) = w.players.get(&id) {
                    reply(c, &ServerMsg::Err { text: "Уже продано".into() });
                }
                return;
            };
            if w.offers[pos].seller == id {
                return;
            }
            let offer = w.offers.remove(pos);
            if let Some(c) = w.players.get(&id) {
                reply(c, &ServerMsg::Bought { offer: offer.clone() });
            }
            if let Some(s) = w.players.get(&offer.seller) {
                reply(s, &ServerMsg::Sold { id: offer.id, gold: offer.price });
            }
            let _ = app.tx.send(enc(&ServerMsg::Market { offers: w.offers.clone() }));
        }
        ClientMsg::Withdraw { id: oid } => {
            if let Some(pos) = w.offers.iter().position(|o| o.id == oid && o.seller == id) {
                let offer = w.offers.remove(pos);
                if let Some(c) = w.players.get(&id) {
                    reply(c, &ServerMsg::Withdrawn { offer });
                }
                let _ = app.tx.send(enc(&ServerMsg::Market { offers: w.offers.clone() }));
            }
        }
    }
}
