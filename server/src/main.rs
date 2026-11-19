//! Сервер Pixel Village: игроки, общий мир, рынок, рейтинги + страница «кто онлайн».
mod db;

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
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use shared::{
    Ban, ClientMsg, DeviceInfo, Mute, Offer, PlayerState, ServerMsg, ITEM_COUNT, MAX_NAME_CHARS, NODE_COUNT_MAX, RESPAWN_SECS, WORLD_SIZE,
};
use tokio::sync::{broadcast, mpsc, Mutex};
use sqlx::SqlitePool;

const MAX_PLAYERS: usize = 200;
const MAX_OFFERS: usize = 300;

// Admin API request structures
#[derive(serde::Deserialize)]
struct BanRequest {
    player: String,
    reason: Option<String>,
    hours: Option<i64>,
    admin: Option<String>,
}

#[derive(serde::Deserialize)]
struct UnbanRequest {
    player: String,
}

#[derive(serde::Deserialize)]
struct MuteRequest {
    player: String,
    reason: Option<String>,
    hours: Option<i64>,
    admin: Option<String>,
}

#[derive(serde::Deserialize)]
struct UnmuteRequest {
    player: String,
}

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
    failed_auth_attempts: HashMap<String, (u32, Instant)>, // IP -> (count, last_attempt)
}

#[derive(Clone)]
struct App {
    world: Arc<Mutex<World>>,
    tx: broadcast::Sender<String>,
    admin_key: Option<String>,
    db: SqlitePool,
}

fn enc(m: &ServerMsg) -> String {
    serde_json::to_string(m).unwrap_or_default()
}

// Constant-time comparison for admin key
// TODO #103: Use constant-time comparison for all admin key checks
fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut result = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        result |= x ^ y;
    }
    result == 0
}

// Check admin authorization from headers
// TODO #65: Extract real client IP from ConnectInfo<SocketAddr> instead of hardcoding 127.0.0.1
async fn check_admin_auth(app: &App, headers: &HeaderMap, client_ip: &str) -> Result<(), StatusCode> {
    // Check rate limiting
    {
        let w = app.world.lock().await;
        if let Some((count, last)) = w.failed_auth_attempts.get(client_ip) {
            if *count >= 5 && last.elapsed().as_secs() < 300 {
                return Err(StatusCode::TOO_MANY_REQUESTS);
            }
        }
    }

    let admin_key = match &app.admin_key {
        Some(key) => key,
        None => return Err(StatusCode::SERVICE_UNAVAILABLE), // Admin not configured
    };

    // Extract key from X-Admin-Key or Authorization header
    let provided_key = headers
        .get("X-Admin-Key")
        .or_else(|| headers.get("Authorization"))
        .and_then(|h| h.to_str().ok())
        .map(|s| s.trim_start_matches("Bearer ").trim());

    let provided_key = match provided_key {
        Some(key) => key,
        None => {
            record_failed_auth(app, client_ip).await;
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    if !constant_time_compare(admin_key, provided_key) {
        record_failed_auth(app, client_ip).await;
        // Add delay on failed attempt to slow down brute force
        tokio::time::sleep(Duration::from_millis(500)).await;
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Clear failed attempts on successful auth
    let mut w = app.world.lock().await;
    w.failed_auth_attempts.remove(client_ip);

    Ok(())
}

async fn record_failed_auth(app: &App, client_ip: &str) {
    let mut w = app.world.lock().await;
    let entry = w.failed_auth_attempts.entry(client_ip.to_string()).or_insert((0, Instant::now()));
    entry.0 += 1;
    entry.1 = Instant::now();

    // Cleanup old entries (>1 hour)
    w.failed_auth_attempts.retain(|_, (_, time)| time.elapsed().as_secs() < 3600);
}

#[tokio::main]
async fn main() {
    // Ініціалізувати базу даних
    let db = db::init_db().await.expect("Не вдалось підключитись до БД");

    let (tx, _) = broadcast::channel::<String>(256);
    let app = App {
        world: Arc::new(Mutex::new(World::default())),
        tx,
        admin_key: std::env::var("ADMIN_KEY").ok().filter(|k| !k.is_empty()),
        db,
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
                    let board = if tick.is_multiple_of(75) {
                        let now = chrono::Utc::now().timestamp();
                        w.bans.retain(|b| b.expires_at.is_none() || b.expires_at.unwrap() > now);
                        w.mutes.retain(|m| m.expires_at.is_none() || m.expires_at.unwrap() > now);
                        // TODO #100: Broadcast ServerMsg::Respawn when nodes are pruned from w.dead
                        w.dead.retain(|_, t| t.elapsed().as_secs() < RESPAWN_SECS as u64);
                        let mut sc: Vec<(String, u32)> = w.players.values().map(|c| (c.state.name.clone(), c.score)).collect();
                        sc.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
                        sc.truncate(10);
                        let mut fi: Vec<(String, f32, u32)> =
                            w.players.values().filter(|c| c.big > 0.0).map(|c| (c.state.name.clone(), c.big, c.fish)).collect();
                        fi.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal).then_with(|| a.0.cmp(&b.0)));
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
        .route("/api/players", get(api_players))
        .route("/api/admin/stats", get(api_admin_stats))
        .route("/api/admin/ban", post(api_ban))
        .route("/api/admin/unban", post(api_unban))
        .route("/api/admin/mute", post(api_mute))
        .route("/api/admin/unmute", post(api_unmute))
        .route("/api/admin/bans", get(api_list_bans))
        .route("/api/admin/logs", get(api_admin_logs))
        .route("/api/admin/tickets", get(api_tickets))
        .route("/api/admin/ticket/create", post(api_create_ticket))
        .route("/api/admin/ticket/update", post(api_update_ticket))
        .route("/api/admin/config", get(api_get_config))
        .route("/api/admin/config/set", post(api_set_config))
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

async fn api_version() -> Json<serde_json::Value> {
    // Версія гри - змінюйте при кожному оновленні
    Json(serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "date": "2026-11-17",
        "changes": [
            "✅ Виправлено відображення координат гравців в Admin Dashboard",
            "✅ Виправлено Android touch controls (simulate_mouse_with_touch)",
            "✅ Налаштовано production сервер (wss://api.pixel-village.online/ws)",
            "✅ Додано Discord Rich Presence",
            "✅ Виправлено баги компіляції та warnings",
            "🎮 Multiplayer працює стабільно",
            "🔧 Admin панель з real-time статистикою",
            "🚀 Автоматичне оновлення через GitHub Releases"
        ],
        "download": "https://github.com/gertyn230/game/releases/latest"
    }))
}

async fn api_ban(State(app): State<App>, headers: HeaderMap, Json(req): Json<BanRequest>) -> Response {
    if let Err(status) = check_admin_auth(&app, &headers, "127.0.0.1").await {
        return status.into_response();
    }

    let player_name = req.player;
    let reason = req.reason.unwrap_or_else(|| "Порушення правил".to_string());
    let duration_hours = req.hours;
    let banned_by = req.admin.unwrap_or_else(|| "Адміністратор".to_string());

    let now = chrono::Utc::now().timestamp();
    let expires_at = duration_hours.map(|h| {
        let h = h.min(87600);
        now + h.saturating_mul(3600)
    });

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

async fn api_unban(State(app): State<App>, headers: HeaderMap, Json(req): Json<UnbanRequest>) -> Response {
    if let Err(status) = check_admin_auth(&app, &headers, "127.0.0.1").await {
        return status.into_response();
    }

    let player_name = req.player;

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

async fn api_mute(State(app): State<App>, headers: HeaderMap, Json(req): Json<MuteRequest>) -> Response {
    if let Err(status) = check_admin_auth(&app, &headers, "127.0.0.1").await {
        return status.into_response();
    }

    let player_name = req.player;
    let reason = req.reason.unwrap_or_else(|| "Порушення правил спілкування".to_string());
    let duration_hours = req.hours;
    let muted_by = req.admin.unwrap_or_else(|| "Адміністратор".to_string());

    let now = chrono::Utc::now().timestamp();
    let expires_at = duration_hours.map(|h| {
        let h = h.min(87600);
        now + h.saturating_mul(3600)
    });

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

async fn api_unmute(State(app): State<App>, headers: HeaderMap, Json(req): Json<UnmuteRequest>) -> Response {
    if let Err(status) = check_admin_auth(&app, &headers, "127.0.0.1").await {
        return status.into_response();
    }

    let player_name = req.player;

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

async fn api_list_bans(State(app): State<App>, headers: HeaderMap) -> Response {
    if let Err(status) = check_admin_auth(&app, &headers, "127.0.0.1").await {
        return status.into_response();
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

// TODO #104: Restrict /api/players to authenticated admins or remove coordinates
async fn api_players(State(app): State<App>) -> Json<serde_json::Value> {
    let w = app.world.lock().await;
    let players: Vec<serde_json::Value> = w.players.values().map(|c| {
        serde_json::json!({
            "id": c.state.id,
            "name": c.state.name,
            "position": {"x": c.state.x, "y": c.state.y},
            "face": c.state.face,
            "moving": c.state.moving,
            "status": "online"
        })
    }).collect();
    Json(serde_json::json!({ "count": players.len(), "players": players }))
}

async fn api_admin_stats(State(app): State<App>, headers: HeaderMap) -> Response {
    if let Err(status) = check_admin_auth(&app, &headers, "127.0.0.1").await {
        return status.into_response();
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

    // Активні бани та мути
    let now = chrono::Utc::now().timestamp();
    let active_bans: Vec<serde_json::Value> = w.bans.iter()
        .filter(|b| b.expires_at.is_none() || b.expires_at.unwrap() > now)
        .map(|b| serde_json::json!({
            "player_name": b.player_name,
            "reason": b.reason,
            "banned_by": b.banned_by,
            "banned_at": b.banned_at,
            "expires_at": b.expires_at,
        }))
        .collect();

    let active_mutes: Vec<serde_json::Value> = w.mutes.iter()
        .filter(|m| m.expires_at.is_none() || m.expires_at.unwrap() > now)
        .map(|m| serde_json::json!({
            "player_name": m.player_name,
            "reason": m.reason,
            "muted_by": m.muted_by,
            "muted_at": m.muted_at,
            "expires_at": m.expires_at,
        }))
        .collect();

    // Торгові пропозиції
    let offers: Vec<serde_json::Value> = w.offers.iter()
        .map(|o| serde_json::json!({
            "id": o.id,
            "seller_name": o.seller_name,
            "item": o.item,
            "qty": o.qty,
            "price": o.price,
        }))
        .collect();

    Json(serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "players": {
            "online": players.len(),
            "list": players,
        },
        "market": {
            "offers_count": w.offers.len(),
            "offers": offers,
        },
        "moderation": {
            "active_bans": active_bans.len(),
            "bans": active_bans,
            "active_mutes": active_mutes.len(),
            "mutes": active_mutes,
        },
        "stats": {
            "by_os": os_stats,
            "by_version": version_stats,
            "by_arch": arch_stats,
        }
    })).into_response()
}

async fn ws_handler(ws: WebSocketUpgrade, State(app): State<App>) -> impl IntoResponse {
    ws.max_message_size(262144).on_upgrade(move |socket| client_loop(socket, app))
}

fn clean_name(raw: &str) -> Option<String> {
    let s: String = raw.trim().chars().filter(|c| !c.is_control()).take(MAX_NAME_CHARS).collect();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

async fn client_loop(socket: WebSocket, app: App) {
    // TODO #69: Add ping/pong heartbeat and idle timeout to detect dead connections
    let (mut sink, mut stream) = socket.split();
    let mut rx = app.tx.subscribe();
    // TODO #101: Use bounded channel to prevent memory DoS
    let (dtx, mut drx) = mpsc::unbounded_channel::<String>();

    let (id, welcome_pack) = {
        let mut w = app.world.lock().await;
        if w.players.len() >= MAX_PLAYERS {
            return;
        }
        w.next_id += 1;
        let id = w.next_id;
        // TODO #13: Don't insert player until Join is received and ban check passes. Reject all messages from unjoined sessions.
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
                        w.players.remove(&id);
                        return;
                    }
                }

                // Перевірка муту
                // TODO #71: Check mute by player identity (IP/session), not by requested name
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
                    c.state.x = x.clamp(0.0, WORLD_SIZE - 0.001);
                    c.state.y = y.clamp(0.0, WORLD_SIZE - 0.001);
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
                if w.dead.contains_key(&idx) {
                    return;
                }
                w.dead.insert(idx, Instant::now());
                let _ = app.tx.send(enc(&ServerMsg::Dead { idx, left: RESPAWN_SECS }));
            }
        }
        ClientMsg::Stats { score, big, fish } => {
            // TODO #73: Compute scores server-side instead of accepting ClientMsg::Stats
            if let Some(c) = w.players.get_mut(&id) {
                c.score = score;
                c.big = if big.is_finite() { big.clamp(0.0, 50.0) } else { 0.0 };
                c.fish = fish;
            }
        }
        ClientMsg::Post { item, qty, price } => {
            // TODO #16: Server should maintain player inventories for secure market transactions
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
            // TODO #68: Persist offline sales so sellers receive gold on reconnect
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

// ============ Нові API endpoints для адмінки ============

async fn api_admin_logs(State(app): State<App>, headers: HeaderMap, Query(q): Query<HashMap<String, String>>) -> Response {
    if let Err(status) = check_admin_auth(&app, &headers, "127.0.0.1").await {
        return status.into_response();
    }

    let limit = q.get("limit").and_then(|s| s.parse().ok()).unwrap_or(50);

    match db::get_logs(&app.db, limit).await {
        Ok(logs) => Json(logs).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Помилка БД: {}", e)).into_response()
    }
}

#[derive(serde::Deserialize)]
struct TicketCreateReq {
    key: String,
    player: String,
    ticket_type: String,
    subject: String,
    message: String,
}

// TODO #74: Ticket creation should use player session auth, not admin key
async fn api_create_ticket(State(app): State<App>, Json(req): Json<TicketCreateReq>) -> Response {
    // Перевірка ключа (для тестів можна створювати без ключа)
    if let Some(k) = &app.admin_key {
        if &req.key != k {
            return (StatusCode::UNAUTHORIZED, "Потрібен ключ").into_response();
        }
    }

    match db::create_ticket(&app.db, &req.player, &req.ticket_type, &req.subject, &req.message).await {
        Ok(id) => Json(serde_json::json!({"id": id, "status": "created"})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Помилка: {}", e)).into_response()
    }
}

async fn api_tickets(State(app): State<App>, headers: HeaderMap, Query(q): Query<HashMap<String, String>>) -> Response {
    if let Err(status) = check_admin_auth(&app, &headers, "127.0.0.1").await {
        return status.into_response();
    }

    let status = q.get("status").map(|s| s.as_str());

    match db::get_tickets(&app.db, status).await {
        Ok(tickets) => Json(tickets).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Помилка: {}", e)).into_response()
    }
}

#[derive(serde::Deserialize)]
struct TicketUpdateReq {
    key: String,
    id: i64,
    status: String,
}

async fn api_update_ticket(State(app): State<App>, Json(req): Json<TicketUpdateReq>) -> Response {
    match (&app.admin_key, Some(&req.key)) {
        (Some(k), Some(given)) if k == given => {}
        _ => return (StatusCode::UNAUTHORIZED, "Потрібен ключ").into_response()
    }

    // Додати лог
    let _ = db::add_log(&app.db, "admin", &format!("Тікет #{} -> {}", req.id, req.status)).await;

    match db::update_ticket_status(&app.db, req.id, &req.status).await {
        Ok(_) => Json(serde_json::json!({"status": "updated"})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Помилка: {}", e)).into_response()
    }
}

async fn api_get_config(State(app): State<App>, headers: HeaderMap, Query(q): Query<HashMap<String, String>>) -> Response {
    if let Err(status) = check_admin_auth(&app, &headers, "127.0.0.1").await {
        return status.into_response();
    }

    let config_key = match q.get("config_key") {
        Some(k) => k,
        None => return (StatusCode::BAD_REQUEST, "Потрібен параметр config_key").into_response()
    };

    match db::get_config(&app.db, config_key).await {
        Ok(Some(value)) => Json(serde_json::json!({"key": config_key, "value": value})).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Налаштування не знайдено").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Помилка: {}", e)).into_response()
    }
}

#[derive(serde::Deserialize)]
struct ConfigSetReq {
    key: String,
    config_key: String,
    value: String,
}

async fn api_set_config(State(app): State<App>, Json(req): Json<ConfigSetReq>) -> Response {
    match (&app.admin_key, Some(&req.key)) {
        (Some(k), Some(given)) if k == given => {}
        _ => return (StatusCode::UNAUTHORIZED, "Потрібен ключ").into_response()
    }

    // Додати лог
    let _ = db::add_log(&app.db, "admin", &format!("Змінено налаштування: {} = {}", req.config_key, req.value)).await;

    match db::set_config(&app.db, &req.config_key, &req.value).await {
        Ok(_) => Json(serde_json::json!({"status": "updated"})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Помилка: {}", e)).into_response()
    }
}
