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
    Ban, ClientMsg, DeviceInfo, Mute, Offer, PlayerState, ServerMsg, ITEM_COUNT, MAX_NAME_CHARS, NODE_COUNT_MAX, RESPAWN_SECS, WORLD_SIZE,
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
    bans: Vec<Ban>,
    mutes: Vec<Mute>,
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
        .route("/api/admin/stats", get(api_admin_stats))
        .route("/api/admin/ban", get(api_ban))
        .route("/api/admin/unban", get(api_unban))
        .route("/api/admin/mute", get(api_mute))
        .route("/api/admin/unmute", get(api_unmute))
        .route("/api/admin/bans", get(api_list_bans))
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

    let os_rows: String = os_stats
        .iter()
        .map(|(os, count)| format!("<tr><td>{}</td><td>{}</td></tr>", esc(os), count))
        .collect();

    let offers: String = w
        .offers
        .iter()
        .map(|o| format!("<tr><td>{}</td><td>{}</td><td>{} ×{}</td><td>{}</td></tr>", o.id, esc(&o.seller_name), o.item, o.qty, o.price))
        .collect();

    // Статистика банів та мутів
    let now = chrono::Utc::now().timestamp();
    let active_bans: Vec<&Ban> = w.bans.iter()
        .filter(|b| b.expires_at.is_none() || b.expires_at.unwrap() > now)
        .collect();
    let active_mutes: Vec<&Mute> = w.mutes.iter()
        .filter(|m| m.expires_at.is_none() || m.expires_at.unwrap() > now)
        .collect();

    let ban_rows: String = active_bans.iter().map(|b| {
        let expires = match b.expires_at {
            Some(ts) => {
                let hours_left = (ts - now) / 3600;
                format!("{} год", hours_left)
            }
            None => "Постійно".to_string()
        };
        format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            esc(&b.player_name), esc(&b.reason), esc(&b.banned_by), expires
        )
    }).collect();

    let mute_rows: String = active_mutes.iter().map(|m| {
        let expires = match m.expires_at {
            Some(ts) => {
                let hours_left = (ts - now) / 3600;
                format!("{} год", hours_left)
            }
            None => "Постійно".to_string()
        };
        format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            esc(&m.player_name), esc(&m.reason), esc(&m.muted_by), expires
        )
    }).collect();

    let admin_key = q.get("key").unwrap();

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
         .action-btn{{padding:6px 12px;background:#238636;color:#fff;border:none;border-radius:6px;cursor:pointer;font-size:12px;margin:2px}}\
         .action-btn:hover{{background:#2ea043}}\
         .ban-btn{{background:#da3633}}\
         .ban-btn:hover{{background:#f85149}}\
         .mute-btn{{background:#ff922b}}\
         .mute-btn:hover{{background:#ffa94d}}\
         .form-section{{background:#151a21;padding:20px;border-radius:8px;margin:20px 0;border:1px solid #2d3640}}\
         input,select{{background:#0d1117;border:1px solid #30363d;color:#d9d7ce;padding:8px;border-radius:6px;margin:5px}}\
         .banned{{color:#f85149}}\
         .muted{{color:#ffa94d}}\
         </style>\
         </head><body><div class=\"container\">\
         <h1>🎮 Pixel Village — Адмінка</h1>\
         <div class=\"stats\">\
         <div class=\"stat-card\"><div class=\"stat-label\">Гравців онлайн</div><div class=\"stat-value online\">{}</div></div>\
         <div class=\"stat-card\"><div class=\"stat-label\">Активних оголошень</div><div class=\"stat-value\">{}</div></div>\
         <div class=\"stat-card\"><div class=\"stat-label\">Платформ</div><div class=\"stat-value\">{}</div></div>\
         <div class=\"stat-card\"><div class=\"stat-label banned\">Активних банів</div><div class=\"stat-value banned\">{}</div></div>\
         <div class=\"stat-card\"><div class=\"stat-label muted\">Активних мутів</div><div class=\"stat-value muted\">{}</div></div>\
         </div>\
         \
         <h2>🔨 Модерація</h2>\
         <div class=\"form-section\">\
         <h3>Забанити гравця</h3>\
         <form action=\"/api/admin/ban\" method=\"get\" target=\"_blank\">\
         <input type=\"hidden\" name=\"key\" value=\"{}\">\
         <input type=\"text\" name=\"player\" placeholder=\"Ім'я гравця\" required>\
         <input type=\"text\" name=\"reason\" placeholder=\"Причина\" value=\"Порушення правил\">\
         <input type=\"number\" name=\"hours\" placeholder=\"Годин (пусто = назавжди)\">\
         <input type=\"text\" name=\"admin\" placeholder=\"Ваше ім'я\" value=\"Адмін\">\
         <button type=\"submit\" class=\"action-btn ban-btn\">Забанити</button>\
         </form>\
         <h3 style=\"margin-top:20px\">Видати мут</h3>\
         <form action=\"/api/admin/mute\" method=\"get\" target=\"_blank\">\
         <input type=\"hidden\" name=\"key\" value=\"{}\">\
         <input type=\"text\" name=\"player\" placeholder=\"Ім'я гравця\" required>\
         <input type=\"text\" name=\"reason\" placeholder=\"Причина\" value=\"Спам/образи\">\
         <input type=\"number\" name=\"hours\" placeholder=\"Годин (пусто = назавжди)\">\
         <input type=\"text\" name=\"admin\" placeholder=\"Ваше ім'я\" value=\"Адмін\">\
         <button type=\"submit\" class=\"action-btn mute-btn\">Замутити</button>\
         </form>\
         </div>\
         \
         <h2>🚫 Активні бани ({})</h2>\
         <table><tr><th>Гравець</th><th>Причина</th><th>Забанив</th><th>Закінчується</th><th>Дії</th></tr>{}</table>\
         \
         <h2>🔇 Активні мути ({})</h2>\
         <table><tr><th>Гравець</th><th>Причина</th><th>Видав</th><th>Закінчується</th><th>Дії</th></tr>{}</table>\
         \
         <h2>👥 Гравці онлайн</h2>\
         <table><tr><th>ID</th><th>Ім'я</th><th>Позиція</th><th>Очки</th><th>Рекорд риби</th><th>В грі</th><th>Пристрій</th><th>Дії</th></tr>{}</table>\
         <h2>📊 Статистика по платформах</h2>\
         <table><tr><th>Платформа</th><th>Гравців</th></tr>{}</table>\
         <h2>🛒 Ринок</h2>\
         <table><tr><th>№</th><th>Продавець</th><th>Предмет</th><th>Ціна</th></tr>{}</table>\
         </div></body></html>",
        list.len(),
        w.offers.len(),
        os_stats.len(),
        active_bans.len(),
        active_mutes.len(),
        admin_key,
        admin_key,
        active_bans.len(),
        if ban_rows.is_empty() {
            "<tr><td colspan=\"5\" style=\"text-align:center;color:#7d8590\">Немає активних банів</td></tr>".to_string()
        } else {
            ban_rows.clone() + &active_bans.iter().map(|b| {
                format!("<td><a href=\"/api/admin/unban?key={}&player={}\" target=\"_blank\" class=\"action-btn\">Розбанити</a></td></tr>",
                    admin_key, urlencoding::encode(&b.player_name))
            }).collect::<String>()
        },
        active_mutes.len(),
        if mute_rows.is_empty() {
            "<tr><td colspan=\"5\" style=\"text-align:center;color:#7d8590\">Немає активних мутів</td></tr>".to_string()
        } else {
            mute_rows.clone() + &active_mutes.iter().map(|m| {
                format!("<td><a href=\"/api/admin/unmute?key={}&player={}\" target=\"_blank\" class=\"action-btn\">Зняти мут</a></td></tr>",
                    admin_key, urlencoding::encode(&m.player_name))
            }).collect::<String>()
        },
        list.iter().map(|c| {
            let device = if let Some(d) = &c.device {
                format!("{} {} ({})", esc(&d.os), esc(&d.version), esc(&d.arch))
            } else {
                "Невідомо".to_string()
            };
            format!(
                "<tr><td>{}</td><td>{}</td><td>{:.1}, {:.1}</td><td>{}</td><td>{:.2} кг</td><td>{} с</td><td>{}</td>\
                <td>\
                <a href=\"/api/admin/ban?key={}&player={}&reason=Порушення&hours=24&admin=Адмін\" target=\"_blank\" class=\"action-btn ban-btn\">Бан 24г</a>\
                <a href=\"/api/admin/mute?key={}&player={}&reason=Спам&hours=1&admin=Адмін\" target=\"_blank\" class=\"action-btn mute-btn\">Мут 1г</a>\
                </td></tr>",
                c.state.id,
                esc(&c.state.name),
                c.state.x,
                c.state.y,
                c.score,
                c.big,
                c.joined.elapsed().as_secs(),
                device,
                admin_key, urlencoding::encode(&c.state.name),
                admin_key, urlencoding::encode(&c.state.name)
            )
        }).collect::<String>(),
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

async fn api_ban(State(app): State<App>, Query(q): Query<HashMap<String, String>>) -> Response {
    match (&app.admin_key, q.get("key")) {
        (Some(k), Some(given)) if k == given => {}
        _ => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    }

    let player_name = match q.get("player") {
        Some(n) => n.clone(),
        None => return (StatusCode::BAD_REQUEST, "Missing player name").into_response(),
    };

    let reason = q.get("reason").cloned().unwrap_or_else(|| "Порушення правил".to_string());
    let duration_hours = q.get("hours").and_then(|h| h.parse::<i64>().ok());
    let banned_by = q.get("admin").cloned().unwrap_or_else(|| "Адміністратор".to_string());

    let now = chrono::Utc::now().timestamp();
    let expires_at = duration_hours.map(|h| now + h * 3600);

    let ban = Ban {
        player_name: player_name.clone(),
        reason: reason.clone(),
        banned_at: now,
        expires_at,
        banned_by: banned_by.clone(),
    };

    let mut w = app.world.lock().await;

    // Видалити старий бан якщо є
    w.bans.retain(|b| b.player_name != player_name);
    w.bans.push(ban.clone());

    // Відключити гравця якщо він онлайн
    let player_id = w.players.iter().find(|(_, c)| c.state.name == player_name).map(|(id, _)| *id);
    if let Some(pid) = player_id {
        if let Some(c) = w.players.get(&pid) {
            let msg = format!("Ви заблоковані: {}. Забанив: {}", reason, banned_by);
            reply(c, &ServerMsg::Err { text: msg });
        }
        w.players.remove(&pid);
    }

    Json(serde_json::json!({
        "success": true,
        "ban": ban,
        "message": format!("Гравець {} заблокований", player_name)
    })).into_response()
}

async fn api_unban(State(app): State<App>, Query(q): Query<HashMap<String, String>>) -> Response {
    match (&app.admin_key, q.get("key")) {
        (Some(k), Some(given)) if k == given => {}
        _ => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    }

    let player_name = match q.get("player") {
        Some(n) => n.clone(),
        None => return (StatusCode::BAD_REQUEST, "Missing player name").into_response(),
    };

    let mut w = app.world.lock().await;
    let before = w.bans.len();
    w.bans.retain(|b| b.player_name != player_name);
    let removed = before != w.bans.len();

    Json(serde_json::json!({
        "success": removed,
        "message": if removed {
            format!("Гравець {} розблокований", player_name)
        } else {
            format!("Гравець {} не знайдений у списку банів", player_name)
        }
    })).into_response()
}

async fn api_mute(State(app): State<App>, Query(q): Query<HashMap<String, String>>) -> Response {
    match (&app.admin_key, q.get("key")) {
        (Some(k), Some(given)) if k == given => {}
        _ => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    }

    let player_name = match q.get("player") {
        Some(n) => n.clone(),
        None => return (StatusCode::BAD_REQUEST, "Missing player name").into_response(),
    };

    let reason = q.get("reason").cloned().unwrap_or_else(|| "Порушення правил спілкування".to_string());
    let duration_hours = q.get("hours").and_then(|h| h.parse::<i64>().ok());
    let muted_by = q.get("admin").cloned().unwrap_or_else(|| "Адміністратор".to_string());

    let now = chrono::Utc::now().timestamp();
    let expires_at = duration_hours.map(|h| now + h * 3600);

    let mute = Mute {
        player_name: player_name.clone(),
        reason: reason.clone(),
        muted_at: now,
        expires_at,
        muted_by: muted_by.clone(),
    };

    let mut w = app.world.lock().await;

    // Видалити старий мут якщо є
    w.mutes.retain(|m| m.player_name != player_name);
    w.mutes.push(mute.clone());

    // Повідомити гравця якщо він онлайн
    if let Some((_, c)) = w.players.iter().find(|(_, c)| c.state.name == player_name) {
        let msg = format!("Вам видано мут: {}. Видав: {}", reason, muted_by);
        reply(c, &ServerMsg::Err { text: msg });
    }

    Json(serde_json::json!({
        "success": true,
        "mute": mute,
        "message": format!("Гравцю {} видано мут", player_name)
    })).into_response()
}

async fn api_unmute(State(app): State<App>, Query(q): Query<HashMap<String, String>>) -> Response {
    match (&app.admin_key, q.get("key")) {
        (Some(k), Some(given)) if k == given => {}
        _ => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    }

    let player_name = match q.get("player") {
        Some(n) => n.clone(),
        None => return (StatusCode::BAD_REQUEST, "Missing player name").into_response(),
    };

    let mut w = app.world.lock().await;
    let before = w.mutes.len();
    w.mutes.retain(|m| m.player_name != player_name);
    let removed = before != w.mutes.len();

    Json(serde_json::json!({
        "success": removed,
        "message": if removed {
            format!("З гравця {} знято мут", player_name)
        } else {
            format!("Гравець {} не знайдений у списку мутів", player_name)
        }
    })).into_response()
}

async fn api_list_bans(State(app): State<App>, Query(q): Query<HashMap<String, String>>) -> Response {
    match (&app.admin_key, q.get("key")) {
        (Some(k), Some(given)) if k == given => {}
        _ => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    }

    let w = app.world.lock().await;
    let now = chrono::Utc::now().timestamp();

    // Видалити прострочені бани та мути
    let active_bans: Vec<&Ban> = w.bans.iter()
        .filter(|b| b.expires_at.is_none() || b.expires_at.unwrap() > now)
        .collect();

    let active_mutes: Vec<&Mute> = w.mutes.iter()
        .filter(|m| m.expires_at.is_none() || m.expires_at.unwrap() > now)
        .collect();

    Json(serde_json::json!({
        "bans": active_bans,
        "mutes": active_mutes,
        "total_bans": active_bans.len(),
        "total_mutes": active_mutes.len(),
    })).into_response()
}

async fn api_players(State(app): State<App>) -> Json<serde_json::Value> {
    let w = app.world.lock().await;
    let names: Vec<&str> = w.players.values().map(|c| c.state.name.as_str()).collect();
    Json(serde_json::json!({ "count": names.len(), "players": names }))
}

async fn api_admin_stats(State(app): State<App>, Query(q): Query<HashMap<String, String>>) -> Response {
    match (&app.admin_key, q.get("key")) {
        (Some(k), Some(given)) if k == given => {}
        _ => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    }

    let w = app.world.lock().await;
    let mut os_stats: HashMap<String, usize> = HashMap::new();
    let mut version_stats: HashMap<String, usize> = HashMap::new();
    let mut arch_stats: HashMap<String, usize> = HashMap::new();

    let players: Vec<serde_json::Value> = w.players.values().map(|c| {
        if let Some(dev) = &c.device {
            *os_stats.entry(dev.os.clone()).or_insert(0) += 1;
            *version_stats.entry(dev.version.clone()).or_insert(0) += 1;
            *arch_stats.entry(dev.arch.clone()).or_insert(0) += 1;
        }

        serde_json::json!({
            "id": c.state.id,
            "name": c.state.name,
            "position": {"x": c.state.x, "y": c.state.y},
            "score": c.score,
            "big_fish": c.big,
            "fish_count": c.fish,
            "online_seconds": c.joined.elapsed().as_secs(),
            "device": c.device.as_ref().map(|d| serde_json::json!({
                "os": d.os,
                "version": d.version,
                "arch": d.arch,
            }))
        })
    }).collect();

    Json(serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "players": {
            "online": players.len(),
            "list": players,
        },
        "market": {
            "offers": w.offers.len(),
        },
        "stats": {
            "by_os": os_stats,
            "by_version": version_stats,
            "by_arch": arch_stats,
        }
    })).into_response()
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
                // Перевірка бану
                let now = chrono::Utc::now().timestamp();
                if let Some(ban) = w.bans.iter().find(|b| b.player_name == n) {
                    if ban.expires_at.is_none() || ban.expires_at.unwrap() > now {
                        let reason = format!("Ви заблоковані: {}. Забанив: {}", ban.reason, ban.banned_by);
                        if let Some(c) = w.players.get(&id) {
                            reply(c, &ServerMsg::Err { text: reason });
                        }
                        return;
                    }
                }

                // Перевірка муту
                let is_muted = w.mutes.iter().any(|m| {
                    m.player_name == n && (m.expires_at.is_none() || m.expires_at.unwrap() > now)
                });

                if let Some(c) = w.players.get_mut(&id) {
                    // Якщо гравець замучений, не дозволяємо змінювати ім'я
                    if !is_muted || c.state.name == format!("Игрок{id}") {
                        c.state.name = n;
                    } else {
                        reply(c, &ServerMsg::Err { text: "У вас мут, не можна змінювати ім'я".to_string() });
                    }
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
