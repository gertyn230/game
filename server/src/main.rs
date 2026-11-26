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
use sqlx::{SqlitePool, Row};

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
    last_move: Instant,
    ip: String,
    tx: mpsc::UnboundedSender<String>,
    score: u32,
    big: f32,
    fish: u32,
    device: Option<DeviceInfo>,
    account_email: Option<String>,
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
    banned_ips: HashMap<String, String>, // IP -> Reason
    failed_auth_attempts: HashMap<String, (u32, Instant)>, // IP -> (count, last_attempt)
    kicked: HashMap<String, i64>, // Player name -> expires_at unix timestamp
    ticket_rate_limit: HashMap<String, (u32, Instant)>, // IP -> (count, first_attempt)
    promo_rate_limit: HashMap<String, (u32, Instant)>, // IP -> (count, first_attempt)
    auth_rate_limit: HashMap<String, (u32, Instant)>, // IP -> (count, first_attempt) for register/login
}

#[derive(Clone)]
struct App {
    world: Arc<Mutex<World>>,
    tx: broadcast::Sender<String>,
    admin_key: Option<String>,
    db: SqlitePool,
}

fn get_client_ip(headers: &HeaderMap) -> String {
    headers
        .get("x-real-ip")
        .or_else(|| headers.get("x-forwarded-for"))
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "127.0.0.1".to_string())
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
        .route("/admin", get(admin_page))
        .route("/admin.html", get(admin_page))
        .route("/api/players", get(api_players))
        .route("/api/admin/stats", get(api_admin_stats))
        .route("/api/admin/ban", post(api_ban))
        .route("/api/admin/unban", post(api_unban))
        .route("/api/admin/mute", post(api_mute))
        .route("/api/admin/unmute", post(api_unmute))
        .route("/api/admin/kick", post(api_kick))
        .route("/api/admin/broadcast", post(api_broadcast))
        .route("/api/admin/bans", get(api_list_bans))
        .route("/api/admin/logs", get(api_admin_logs))
        .route("/api/admin/tickets", get(api_tickets))
        .route("/api/admin/ticket/create", post(api_create_ticket))
        .route("/api/admin/ticket/update", post(api_update_ticket))
        .route("/api/admin/promos", get(api_get_promos))
        .route("/api/admin/promo/create", post(api_create_promo))
        .route("/api/admin/promo/toggle", post(api_toggle_promo))
        .route("/api/admin/promo/delete", post(api_delete_promo))
        .route("/api/updates", get(api_get_updates))
        .route("/api/admin/updates", get(api_admin_get_updates))
        .route("/api/admin/update/create", post(api_create_update))
        .route("/api/admin/update/edit", post(api_edit_update))
        .route("/api/admin/update/delete", post(api_delete_update))
        .route("/api/account/register", post(api_account_register))
        .route("/api/account/login", post(api_account_login))
        .route("/api/admin/accounts", get(api_admin_accounts))
        .route("/api/admin/account/reset-password", post(api_admin_reset_password))
        .route("/api/account/change-password", post(api_account_change_password))
        .route("/api/account/google", post(api_account_google))
        .route("/api/account/one-time-code", post(api_account_one_time_code))
        .route("/api/account/code-login", post(api_account_code_login))
        .route("/api/account/profile", get(api_account_profile))
        .route("/api/promo/redeem", post(api_redeem_promo))
        .route("/api/ticket/create", post(api_create_ticket))
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

async fn admin_page() -> Html<String> {
    if let Ok(content) = tokio::fs::read_to_string("/app/admin.html").await {
        return Html(content);
    }
    if let Ok(content) = tokio::fs::read_to_string("admin.html").await {
        return Html(content);
    }
    if let Ok(content) = tokio::fs::read_to_string("server/admin.html").await {
        return Html(content);
    }
    Html(include_str!("../admin.html").to_string())
}

async fn index(headers: HeaderMap, State(app): State<App>) -> Response {
    if let Some(host) = headers.get("host").and_then(|h| h.to_str().ok()) {
        if host.starts_with("admin.") {
            return admin_page().await.into_response();
        }
    }
    let w = app.world.lock().await;
    let mut names: Vec<String> = w.players.values().map(|c| esc(&c.state.name)).collect();
    names.sort();
    let rows: String = names.iter().map(|n| format!("<li>{n}</li>")).collect();
    page("Pixel Village: онлайн", &format!("<h1>Онлайн: {}</h1><ul>{rows}</ul>", names.len())).into_response()
}

async fn api_version() -> Json<serde_json::Value> {
    // Версія гри - змінюйте при кожному оновленні
    Json(serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "date": "2026-11-26",
        "changes": [
            "🔐 Обов'язкова авторизація перед грою (Email або Google)",
            "🌐 Повноцінний особистий кабінет гравця на сайті",
            "⚡ Швидкий вхід за одноразовим 6-значним кодом",
            "🛡️ Захист акаунтів від підбору паролів (Rate Limiting)",
            "🔑 Можливість самостійної зміни пароля в грі та на сайті",
            "🎁 Активація промокодів онлайн у кабінеті гравця",
            "📋 Підтримка та тікети безпосередньо з веб-кабінету",
            "🚀 Багатосторінковий портал: Кабінет, База знань, Оновлення"
        ],
        "download": "https://pixel-village.online/cabinet.html"
    }))
}

async fn api_ban(State(app): State<App>, headers: HeaderMap, Json(req): Json<BanRequest>) -> Response {
    let client_ip = get_client_ip(&headers);
    if let Err(status) = check_admin_auth(&app, &headers, &client_ip).await {
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

    // Відключити гравця якщо він онлайн та заблокувати його IP
    let player_info = w.players.iter().find(|(_, c)| c.state.name == player_name).map(|(id, c)| (*id, c.ip.clone()));
    if let Some((pid, ip)) = player_info {
        if !ip.is_empty() && ip != "127.0.0.1" {
            w.banned_ips.insert(ip, reason.clone());
        }
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
    let client_ip = get_client_ip(&headers);
    if let Err(status) = check_admin_auth(&app, &headers, &client_ip).await {
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
    let client_ip = get_client_ip(&headers);
    if let Err(status) = check_admin_auth(&app, &headers, &client_ip).await {
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
    let client_ip = get_client_ip(&headers);
    if let Err(status) = check_admin_auth(&app, &headers, &client_ip).await {
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
    let client_ip = get_client_ip(&headers);
    if let Err(status) = check_admin_auth(&app, &headers, &client_ip).await {
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

// Public online players list - coordinates hidden to prevent wallhack/stalking
async fn api_players(State(app): State<App>) -> Json<serde_json::Value> {
    let w = app.world.lock().await;
    let players: Vec<serde_json::Value> = w.players.values().map(|c| {
        serde_json::json!({
            "id": c.state.id,
            "name": c.state.name,
            "face": c.state.face,
            "moving": c.state.moving,
            "status": "online"
        })
    }).collect();
    Json(serde_json::json!({ "count": players.len(), "players": players }))
}

async fn api_admin_stats(State(app): State<App>, headers: HeaderMap) -> Response {
    let client_ip = get_client_ip(&headers);
    if let Err(status) = check_admin_auth(&app, &headers, &client_ip).await {
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

async fn ws_handler(ws: WebSocketUpgrade, headers: HeaderMap, State(app): State<App>) -> impl IntoResponse {
    let client_ip = get_client_ip(&headers);
    ws.max_message_size(262144).on_upgrade(move |socket| client_loop(socket, app, client_ip))
}

fn clean_name(raw: &str) -> Option<String> {
    let s: String = raw.trim().chars().filter(|c| !c.is_control()).take(MAX_NAME_CHARS).collect();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

async fn client_loop(socket: WebSocket, app: App, client_ip: String) {
    let (mut sink, mut stream) = socket.split();

    // Check if IP is banned
    {
        let w = app.world.lock().await;
        if let Some(reason) = w.banned_ips.get(&client_ip) {
            let msg = format!("Ваша IP-адреса заблокована: {}", reason);
            let _ = sink.send(Message::Text(enc(&ServerMsg::Err { text: msg }))).await;
            return;
        }
    }

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
                last_move: Instant::now(),
                ip: client_ip.clone(),
                tx: dtx,
                score: 0,
                big: 0.0,
                fish: 0,
                device: None,
                account_email: None,
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
    let removed = w.players.remove(&id);
    let before = w.offers.len();
    w.offers.retain(|o| o.seller != id);
    if w.offers.len() != before {
        let _ = app.tx.send(enc(&ServerMsg::Market { offers: w.offers.clone() }));
    }

    if let Some(c) = removed {
        if let Some(email) = c.account_email {
            let pool = app.db.clone();
            tokio::spawn(async move {
                let _ = sqlx::query("UPDATE accounts SET last_x = ?, last_y = ?, score = ?, fish = ? WHERE email = ?")
                    .bind(c.state.x as f64)
                    .bind(c.state.y as f64)
                    .bind(c.score as i64)
                    .bind(c.fish as i64)
                    .bind(&email)
                    .execute(&pool)
                    .await;
            });
        }
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

                // Перевірка кіку (cooldown щоб уникнути миттєвого автоперепідключення)
                if let Some(&expires_at) = w.kicked.get(&n) {
                    if expires_at > now {
                        let remaining = expires_at - now;
                        let reason = format!("Вас було відключено адміністратором. Зачекайте ще {} с.", remaining);
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
                    let dt = c.last_move.elapsed().as_secs_f32().max(0.01);
                    c.last_move = Instant::now();
                    let target_x = x.clamp(0.0, WORLD_SIZE - 0.001);
                    let target_y = y.clamp(0.0, WORLD_SIZE - 0.001);
                    let dx = target_x - c.state.x;
                    let dy = target_y - c.state.y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    // Normal speed ~5-7 tiles/sec. Allow network lag bursts up to 14.0 * dt + 3.0 tiles
                    let max_allowed = 14.0 * dt + 3.0;
                    if dist <= max_allowed {
                        c.state.x = target_x;
                        c.state.y = target_y;
                    }
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
                // Check that player is close enough to harvest the node (<= 4.5 tiles)
                if let Some(c) = w.players.get(&id) {
                    let node_x = (idx % 40) as f32 + 0.5;
                    let node_y = (idx / 40) as f32 + 0.5;
                    let dx = c.state.x - node_x;
                    let dy = c.state.y - node_y;
                    if (dx * dx + dy * dy) > 20.25 {
                        return; // Too far from node
                    }
                }
                w.dead.insert(idx, Instant::now());
                let _ = app.tx.send(enc(&ServerMsg::Dead { idx, left: RESPAWN_SECS }));
            }
        }
        ClientMsg::Stats { score, big, fish } => {
            if let Some(c) = w.players.get_mut(&id) {
                c.score = score.min(100_000);
                c.big = if big.is_finite() { big.clamp(0.0, 50.0) } else { 0.0 };
                c.fish = fish.min(5_000);
            }
        }
        ClientMsg::Post { item, qty, price } => {
            if item >= ITEM_COUNT || qty == 0 || qty > 100 || price == 0 || price > 50_000 {
                return;
            }
            // Limit to max 5 concurrent market offers per player
            let player_offers_count = w.offers.iter().filter(|o| o.seller == id).count();
            if player_offers_count >= 5 {
                if let Some(c) = w.players.get(&id) {
                    reply(c, &ServerMsg::Err { text: "Максимум 5 активних пропозицій на ринку".into() });
                }
                return;
            }
            if w.offers.len() >= MAX_OFFERS {
                if let Some(c) = w.players.get(&id) {
                    reply(c, &ServerMsg::Err { text: "Ринок переповнений".into() });
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
        ClientMsg::Register { email, username, password, device_info } => {
            // Rate limit: max 5 register/login attempts per 2 minutes per IP
            let client_ip = w.players.get(&id).map(|c| c.ip.clone()).unwrap_or_default();
            let entry = w.auth_rate_limit.entry(client_ip).or_insert((0, Instant::now()));
            if entry.1.elapsed().as_secs() > 120 {
                entry.0 = 0;
                entry.1 = Instant::now();
            }
            entry.0 += 1;
            if entry.0 > 5 {
                if let Some(c) = w.players.get(&id) {
                    reply(c, &ServerMsg::AuthFailed { reason: "Забагато спроб. Зачекайте 2 хвилини".to_string() });
                }
                return;
            }
            drop(w);
            let res = db::register_account(&app.db, &email, &username, &password).await;
            let mut w = app.world.lock().await;
            match res {
                Ok(account) => {
                    if let Some(c) = w.players.get_mut(&id) {
                        c.account_email = Some(account.email.clone());
                        c.state.name = account.username.clone();
                        c.device = device_info;
                        reply(c, &ServerMsg::AuthSuccess { account });
                    }
                }
                Err(reason) => {
                    if let Some(c) = w.players.get(&id) {
                        reply(c, &ServerMsg::AuthFailed { reason });
                    }
                }
            }
        }
        ClientMsg::Login { email, password, device_info } => {
            // Rate limit: max 5 register/login attempts per 2 minutes per IP
            let client_ip = w.players.get(&id).map(|c| c.ip.clone()).unwrap_or_default();
            let entry = w.auth_rate_limit.entry(client_ip).or_insert((0, Instant::now()));
            if entry.1.elapsed().as_secs() > 120 {
                entry.0 = 0;
                entry.1 = Instant::now();
            }
            entry.0 += 1;
            if entry.0 > 5 {
                if let Some(c) = w.players.get(&id) {
                    reply(c, &ServerMsg::AuthFailed { reason: "Забагато спроб. Зачекайте 2 хвилини".to_string() });
                }
                return;
            }
            drop(w);
            let res = db::login_account(&app.db, &email, &password).await;
            let mut w = app.world.lock().await;
            match res {
                Ok(account) => {
                    if let Some(c) = w.players.get_mut(&id) {
                        c.account_email = Some(account.email.clone());
                        c.state.name = account.username.clone();
                        c.score = account.score;
                        c.fish = account.fish;
                        c.big = account.big_fish;
                        c.state.x = account.x;
                        c.state.y = account.y;
                        c.device = device_info;
                        reply(c, &ServerMsg::AuthSuccess { account });
                    }
                }
                Err(reason) => {
                    if let Some(c) = w.players.get(&id) {
                        reply(c, &ServerMsg::AuthFailed { reason });
                    }
                }
            }
        }
        ClientMsg::GoogleLogin { email, username, firebase_uid, device_info } => {
            drop(w);
            let res = db::google_login_account(&app.db, &email, &username, &firebase_uid).await;
            let mut w = app.world.lock().await;
            match res {
                Ok(account) => {
                    if let Some(c) = w.players.get_mut(&id) {
                        c.account_email = Some(account.email.clone());
                        c.state.name = account.username.clone();
                        c.score = account.score;
                        c.fish = account.fish;
                        c.big = account.big_fish;
                        c.state.x = account.x;
                        c.state.y = account.y;
                        c.device = device_info;
                        reply(c, &ServerMsg::AuthSuccess { account });
                    }
                }
                Err(reason) => {
                    if let Some(c) = w.players.get(&id) {
                        reply(c, &ServerMsg::AuthFailed { reason });
                    }
                }
            }
        }
        ClientMsg::CodeLogin { code, device_info } => {
            drop(w);
            let res = db::login_with_code(&app.db, &code).await;
            let mut w = app.world.lock().await;
            match res {
                Ok(account) => {
                    if let Some(c) = w.players.get_mut(&id) {
                        c.account_email = Some(account.email.clone());
                        c.state.name = account.username.clone();
                        c.score = account.score;
                        c.fish = account.fish;
                        c.big = account.big_fish;
                        c.state.x = account.x;
                        c.state.y = account.y;
                        c.device = device_info;
                        reply(c, &ServerMsg::AuthSuccess { account });
                    }
                }
                Err(reason) => {
                    if let Some(c) = w.players.get(&id) {
                        reply(c, &ServerMsg::AuthFailed { reason });
                    }
                }
            }
        }
        ClientMsg::SaveAccount { gold, score, fish, big_fish, inventory, tools, houses, x, y } => {
            let email_opt = w.players.get(&id).and_then(|c| c.account_email.clone());
            if let Some(email) = email_opt {
                if let Some(c) = w.players.get_mut(&id) {
                    c.score = score.min(100_000);
                    c.big = if big_fish.is_finite() { big_fish.clamp(0.0, 50.0) } else { 0.0 };
                    c.fish = fish.min(5_000);
                }
                let db_pool = app.db.clone();
                tokio::spawn(async move {
                    let _ = db::save_account_data(&db_pool, &email, gold, score, fish, big_fish, &inventory, &tools, houses, x, y).await;
                });
            }
        }
        ClientMsg::ChangePassword { old_password, new_password } => {
            let email_opt = w.players.get(&id).and_then(|c| c.account_email.clone());
            if let Some(email) = email_opt {
                drop(w);
                let res = db::change_account_password(&app.db, &email, &old_password, &new_password).await;
                let w = app.world.lock().await;
                match res {
                    Ok(()) => {
                        if let Some(c) = w.players.get(&id) {
                            reply(c, &ServerMsg::PasswordChanged);
                        }
                    }
                    Err(reason) => {
                        if let Some(c) = w.players.get(&id) {
                            reply(c, &ServerMsg::AuthFailed { reason });
                        }
                    }
                }
            } else {
                if let Some(c) = w.players.get(&id) {
                    reply(c, &ServerMsg::AuthFailed { reason: "Спочатку увійдіть в акаунт".to_string() });
                }
            }
        }
    }
}

// ============ Нові API endpoints для адмінки ============

async fn api_admin_logs(State(app): State<App>, headers: HeaderMap, Query(q): Query<HashMap<String, String>>) -> Response {
    let client_ip = get_client_ip(&headers);
    if let Err(status) = check_admin_auth(&app, &headers, &client_ip).await {
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
    #[allow(dead_code)]
    key: Option<String>,
    player: String,
    ticket_type: String,
    subject: String,
    message: String,
}

async fn api_create_ticket(State(app): State<App>, headers: HeaderMap, Json(req): Json<TicketCreateReq>) -> Response {
    let client_ip = get_client_ip(&headers);

    // Rate limit: max 3 tickets per 5 minutes per IP
    {
        let mut w = app.world.lock().await;
        let entry = w.ticket_rate_limit.entry(client_ip.clone()).or_insert((0, Instant::now()));
        if entry.1.elapsed().as_secs() > 300 {
            entry.0 = 0;
            entry.1 = Instant::now();
        }
        entry.0 += 1;
        if entry.0 > 3 {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                "Ви надіслали забагато тікетів. Будь ласка, зачекайте 5 хвилин."
            ).into_response();
        }
    }

    let player: String = req.player.trim().chars().take(32).collect();
    let ticket_type: String = req.ticket_type.trim().chars().take(32).collect();
    let subject: String = req.subject.trim().chars().take(100).collect();
    let message: String = req.message.trim().chars().take(1500).collect();

    if player.is_empty() || message.is_empty() {
        return (StatusCode::BAD_REQUEST, "Вкажіть нікнейм та текст повідомлення").into_response();
    }

    match db::create_ticket(&app.db, &player, &ticket_type, &subject, &message).await {
        Ok(id) => Json(serde_json::json!({
            "success": true,
            "id": id,
            "status": "created",
            "message": "Тікет успішно надіслано"
        })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Помилка: {}", e)).into_response()
    }
}

#[derive(serde::Deserialize)]
struct PromoRedeemReq {
    code: String,
    player: Option<String>,
}

async fn api_redeem_promo(State(app): State<App>, headers: HeaderMap, Json(req): Json<PromoRedeemReq>) -> Response {
    let client_ip = get_client_ip(&headers);

    // Rate limit: max 6 redemption attempts per minute per IP to prevent brute-force
    {
        let mut w = app.world.lock().await;
        let entry = w.promo_rate_limit.entry(client_ip.clone()).or_insert((0, Instant::now()));
        if entry.1.elapsed().as_secs() > 60 {
            entry.0 = 0;
            entry.1 = Instant::now();
        }
        entry.0 += 1;
        if entry.0 > 6 {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::json!({
                    "success": false,
                    "message": "Забагато спроб введення промокодів. Зачекайте 1 хвилину."
                }))
            ).into_response();
        }
    }

    let code: String = req.code.trim().to_uppercase().chars().take(32).collect();
    if code.is_empty() {
        return Json(serde_json::json!({ "success": false, "message": "Введіть промокод" })).into_response();
    }
    let player: String = req.player.as_deref().unwrap_or("Гравець").trim().chars().take(32).collect();
    if player.is_empty() {
        return Json(serde_json::json!({ "success": false, "message": "Введіть нікнейм гравця" })).into_response();
    }

    match db::redeem_promocode(&app.db, &code, &player).await {
        Ok(db::RedeemResult::Success(reward)) => {
            let _ = db::add_log(&app.db, &player, &format!("Активовано промокод {}: {}", code, reward)).await;

            // If player is currently online, send celebratory in-game announcement
            {
                let w = app.world.lock().await;
                if let Some((_, c)) = w.players.iter().find(|(_, c)| c.state.name == player) {
                    let msg = format!("🎁 Промокод активовано! Отримано: {}", reward);
                    reply(c, &ServerMsg::Err { text: msg });
                }
            }

            Json(serde_json::json!({
                "success": true,
                "reward": reward,
                "message": format!("Промокод активовано! Отримано: {}", reward)
            })).into_response()
        }
        Ok(db::RedeemResult::AlreadyRedeemed) => Json(serde_json::json!({
            "success": false,
            "message": "Ви вже активували цей промокод раніше!"
        })).into_response(),
        Ok(db::RedeemResult::InvalidOrExpired) => Json(serde_json::json!({
            "success": false,
            "message": "Промокод недійсний, вимкнений або вичерпано ліміт"
        })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Помилка БД: {}", e)).into_response()
    }
}

async fn api_tickets(State(app): State<App>, headers: HeaderMap, Query(q): Query<HashMap<String, String>>) -> Response {
    let client_ip = get_client_ip(&headers);
    if let Err(status) = check_admin_auth(&app, &headers, &client_ip).await {
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

async fn api_update_ticket(State(app): State<App>, headers: HeaderMap, Json(req): Json<TicketUpdateReq>) -> Response {
    let client_ip = get_client_ip(&headers);
    if let Err(status) = check_admin_auth(&app, &headers, &client_ip).await {
        // Also allow key in json body with constant-time comparison
        let admin_key = match &app.admin_key {
            Some(k) => k,
            None => return status.into_response(),
        };
        if !constant_time_compare(admin_key, &req.key) {
            return status.into_response();
        }
    }

    let _ = db::add_log(&app.db, "admin", &format!("Тікет #{} -> {}", req.id, req.status)).await;

    match db::update_ticket_status(&app.db, req.id, &req.status).await {
        Ok(_) => Json(serde_json::json!({"status": "updated"})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Помилка: {}", e)).into_response()
    }
}

async fn api_get_config(State(app): State<App>, headers: HeaderMap, Query(q): Query<HashMap<String, String>>) -> Response {
    let client_ip = get_client_ip(&headers);
    if let Err(status) = check_admin_auth(&app, &headers, &client_ip).await {
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

async fn api_set_config(State(app): State<App>, headers: HeaderMap, Json(req): Json<ConfigSetReq>) -> Response {
    let client_ip = get_client_ip(&headers);
    if let Err(status) = check_admin_auth(&app, &headers, &client_ip).await {
        let admin_key = match &app.admin_key {
            Some(k) => k,
            None => return status.into_response(),
        };
        if !constant_time_compare(admin_key, &req.key) {
            return status.into_response();
        }
    }

    let _ = db::add_log(&app.db, "admin", &format!("Змінено налаштування: {} = {}", req.config_key, req.value)).await;

    match db::set_config(&app.db, &req.config_key, &req.value).await {
        Ok(_) => Json(serde_json::json!({"status": "updated"})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Помилка: {}", e)).into_response()
    }
}

#[derive(serde::Deserialize)]
struct KickRequest {
    player: String,
    reason: Option<String>,
    admin: Option<String>,
}

async fn api_kick(State(app): State<App>, headers: HeaderMap, Json(req): Json<KickRequest>) -> Response {
    let client_ip = get_client_ip(&headers);
    if let Err(status) = check_admin_auth(&app, &headers, &client_ip).await {
        return status.into_response();
    }

    let player_name = req.player;
    let reason = req.reason.unwrap_or_else(|| "Відключено адміністратором".to_string());
    let admin = req.admin.unwrap_or_else(|| "Адміністратор".to_string());

    let mut w = app.world.lock().await;
    let now = chrono::Utc::now().timestamp();
    // 60 секунд кік-таймауту (щоб клієнт не міг миттєво автоперепідключитися)
    w.kicked.insert(player_name.clone(), now + 60);

    let player_id = w.players.iter().find(|(_, c)| c.state.name == player_name).map(|(id, _)| *id);

    if let Some(pid) = player_id {
        if let Some(c) = w.players.get(&pid) {
            let msg = format!("Вас відключено адміністратором: {}. Адмін: {}", reason, admin);
            reply(c, &ServerMsg::Err { text: msg });
        }
        w.players.remove(&pid);
        let _ = db::add_log(&app.db, &admin, &format!("Кік гравця {}: {}", player_name, reason)).await;
        Json(serde_json::json!({
            "success": true,
            "message": format!("Гравця {} відключено на 60 секунд", player_name)
        })).into_response()
    } else {
        let _ = db::add_log(&app.db, &admin, &format!("Кік гравця {} (таймаут 60с): {}", player_name, reason)).await;
        Json(serde_json::json!({
            "success": true,
            "message": format!("Гравця {} додано до списку відключених на 60 секунд", player_name)
        })).into_response()
    }
}

#[derive(serde::Deserialize)]
struct BroadcastRequest {
    message: String,
    admin: Option<String>,
}

async fn api_broadcast(State(app): State<App>, headers: HeaderMap, Json(req): Json<BroadcastRequest>) -> Response {
    let client_ip = get_client_ip(&headers);
    if let Err(status) = check_admin_auth(&app, &headers, &client_ip).await {
        return status.into_response();
    }

    let text = req.message.trim().to_string();
    if text.is_empty() {
        return (StatusCode::BAD_REQUEST, "Повідомлення не може бути порожнім").into_response();
    }
    let admin = req.admin.unwrap_or_else(|| "Адміністратор".to_string());

    let broadcast_msg = format!("📢 [Оголошення] {}", text);
    let msg_obj = ServerMsg::Err { text: broadcast_msg };
    let json_msg = enc(&msg_obj);

    // Send to broadcast channel
    let _ = app.tx.send(json_msg.clone());

    // Also send directly to each player
    {
        let w = app.world.lock().await;
        for c in w.players.values() {
            let _ = c.tx.send(json_msg.clone());
        }
    }

    let _ = db::add_log(&app.db, &admin, &format!("Оголошення: {}", text)).await;

    Json(serde_json::json!({
        "success": true,
        "message": "Оголошення надіслано всім гравцям"
    })).into_response()
}

async fn api_get_promos(State(app): State<App>, headers: HeaderMap) -> Response {
    let client_ip = get_client_ip(&headers);
    if let Err(status) = check_admin_auth(&app, &headers, &client_ip).await {
        return status.into_response();
    }

    match db::get_promocodes(&app.db).await {
        Ok(promos) => Json(promos).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Помилка БД: {}", e)).into_response()
    }
}

#[derive(serde::Deserialize)]
struct PromoCreateReq {
    code: String,
    reward: String,
    limit_uses: i64,
    expires: String,
}

async fn api_create_promo(State(app): State<App>, headers: HeaderMap, Json(req): Json<PromoCreateReq>) -> Response {
    let client_ip = get_client_ip(&headers);
    if let Err(status) = check_admin_auth(&app, &headers, &client_ip).await {
        return status.into_response();
    }

    match db::add_promocode(&app.db, &req.code, &req.reward, req.limit_uses, &req.expires).await {
        Ok(_) => {
            let _ = db::add_log(&app.db, "admin", &format!("Створено промокод: {}", req.code)).await;
            Json(serde_json::json!({ "success": true, "message": "Промокод створено" })).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Помилка БД: {}", e)).into_response()
    }
}

#[derive(serde::Deserialize)]
struct PromoActionReq {
    code: String,
}

async fn api_toggle_promo(State(app): State<App>, headers: HeaderMap, Json(req): Json<PromoActionReq>) -> Response {
    let client_ip = get_client_ip(&headers);
    if let Err(status) = check_admin_auth(&app, &headers, &client_ip).await {
        return status.into_response();
    }

    match db::toggle_promocode(&app.db, &req.code).await {
        Ok(new_status) => {
            let _ = db::add_log(&app.db, "admin", &format!("Статус промокоду {}: {}", req.code, if new_status == 1 { "увімкнено" } else { "вимкнено" })).await;
            Json(serde_json::json!({ "success": true, "active": new_status })).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Помилка БД: {}", e)).into_response()
    }
}

async fn api_delete_promo(State(app): State<App>, headers: HeaderMap, Json(req): Json<PromoActionReq>) -> Response {
    let client_ip = get_client_ip(&headers);
    if let Err(status) = check_admin_auth(&app, &headers, &client_ip).await {
        return status.into_response();
    }

    match db::delete_promocode(&app.db, &req.code).await {
        Ok(_) => {
            let _ = db::add_log(&app.db, "admin", &format!("Видалено промокод: {}", req.code)).await;
            Json(serde_json::json!({ "success": true })).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Помилка БД: {}", e)).into_response()
    }
}



async fn api_get_updates(State(app): State<App>) -> Response {
    match db::get_game_updates(&app.db).await {
        Ok(updates) => Json(updates).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Помилка БД: {}", e)).into_response(),
    }
}

async fn api_admin_get_updates(State(app): State<App>, headers: HeaderMap) -> Response {
    let client_ip = get_client_ip(&headers);
    if let Err(status) = check_admin_auth(&app, &headers, &client_ip).await {
        return status.into_response();
    }
    match db::get_game_updates(&app.db).await {
        Ok(updates) => Json(updates).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Помилка БД: {}", e)).into_response(),
    }
}

#[derive(serde::Deserialize)]
struct UpdateCreateReq {
    version: String,
    title: String,
    description: String,
    date: String,
    #[serde(default)]
    title_en: Option<String>,
    #[serde(default)]
    description_en: Option<String>,
    #[serde(default)]
    title_ru: Option<String>,
    #[serde(default)]
    description_ru: Option<String>,
}

async fn api_create_update(State(app): State<App>, headers: HeaderMap, Json(req): Json<UpdateCreateReq>) -> Response {
    let client_ip = get_client_ip(&headers);
    if let Err(status) = check_admin_auth(&app, &headers, &client_ip).await {
        return status.into_response();
    }

    let ver = req.version.trim();
    let title = req.title.trim();
    let desc = req.description.trim();
    let date = req.date.trim();
    let title_en = req.title_en.as_deref().unwrap_or("").trim();
    let desc_en = req.description_en.as_deref().unwrap_or("").trim();
    let title_ru = req.title_ru.as_deref().unwrap_or("").trim();
    let desc_ru = req.description_ru.as_deref().unwrap_or("").trim();

    if ver.is_empty() || title.is_empty() || desc.is_empty() {
        return (StatusCode::BAD_REQUEST, "Всі поля обов'язкові").into_response();
    }

    match db::create_game_update(&app.db, ver, title, desc, date, title_en, desc_en, title_ru, desc_ru).await {
        Ok(id) => {
            let _ = db::add_log(&app.db, "admin", &format!("Опубліковано оновлення: {} — {}", ver, title)).await;
            Json(serde_json::json!({ "success": true, "id": id, "message": "Оновлення опубліковано на сайті" })).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Помилка БД: {}", e)).into_response(),
    }
}

#[derive(serde::Deserialize)]
struct UpdateEditReq {
    id: i64,
    version: String,
    title: String,
    description: String,
    date: String,
    #[serde(default)]
    title_en: Option<String>,
    #[serde(default)]
    description_en: Option<String>,
    #[serde(default)]
    title_ru: Option<String>,
    #[serde(default)]
    description_ru: Option<String>,
}

async fn api_edit_update(State(app): State<App>, headers: HeaderMap, Json(req): Json<UpdateEditReq>) -> Response {
    let client_ip = get_client_ip(&headers);
    if let Err(status) = check_admin_auth(&app, &headers, &client_ip).await {
        return status.into_response();
    }

    let ver = req.version.trim();
    let title = req.title.trim();
    let desc = req.description.trim();
    let date = req.date.trim();
    let title_en = req.title_en.as_deref().unwrap_or("").trim();
    let desc_en = req.description_en.as_deref().unwrap_or("").trim();
    let title_ru = req.title_ru.as_deref().unwrap_or("").trim();
    let desc_ru = req.description_ru.as_deref().unwrap_or("").trim();

    if ver.is_empty() || title.is_empty() || desc.is_empty() {
        return (StatusCode::BAD_REQUEST, "Всі поля обов'язкові").into_response();
    }

    match db::update_game_update(&app.db, req.id, ver, title, desc, date, title_en, desc_en, title_ru, desc_ru).await {
        Ok(_) => {
            let _ = db::add_log(&app.db, "admin", &format!("Змінено оновлення #{}: {}", req.id, ver)).await;
            Json(serde_json::json!({ "success": true, "message": "Оновлення збережено" })).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Помилка БД: {}", e)).into_response(),
    }
}

#[derive(serde::Deserialize)]
struct UpdateDeleteReq {
    id: i64,
}

async fn api_delete_update(State(app): State<App>, headers: HeaderMap, Json(req): Json<UpdateDeleteReq>) -> Response {
    let client_ip = get_client_ip(&headers);
    if let Err(status) = check_admin_auth(&app, &headers, &client_ip).await {
        return status.into_response();
    }

    match db::delete_game_update(&app.db, req.id).await {
        Ok(_) => {
            let _ = db::add_log(&app.db, "admin", &format!("Видалено оновлення сайту #{}", req.id)).await;
            Json(serde_json::json!({ "success": true, "message": "Оновлення видалено" })).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Помилка БД: {}", e)).into_response(),
    }
}

// ============ API акаунтів (Email + Пароль) ============

#[derive(serde::Deserialize)]
struct ApiRegisterReq {
    email: String,
    username: String,
    password: String,
}

#[derive(serde::Deserialize)]
struct ApiLoginReq {
    email: String,
    password: String,
}

#[derive(serde::Deserialize)]
struct ApiResetPasswordReq {
    email: String,
    new_password: String,
}

async fn api_account_register(
    State(app): State<App>,
    headers: HeaderMap,
    Json(payload): Json<ApiRegisterReq>,
) -> Response {
    // Rate limit: max 5 register attempts per 2 minutes per IP
    let client_ip = get_client_ip(&headers);
    {
        let mut w = app.world.lock().await;
        let entry = w.auth_rate_limit.entry(client_ip).or_insert((0, Instant::now()));
        if entry.1.elapsed().as_secs() > 120 {
            entry.0 = 0;
            entry.1 = Instant::now();
        }
        entry.0 += 1;
        if entry.0 > 5 {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::json!({ "success": false, "error": "Забагато спроб. Зачекайте 2 хвилини" })),
            ).into_response();
        }
    }
    match db::register_account(&app.db, &payload.email, &payload.username, &payload.password).await {
        Ok(acc) => Json(serde_json::json!({ "success": true, "account": acc })).into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "error": err })),
        ).into_response(),
    }
}

async fn api_account_login(
    State(app): State<App>,
    headers: HeaderMap,
    Json(payload): Json<ApiLoginReq>,
) -> Response {
    // Rate limit: max 5 login attempts per 2 minutes per IP
    let client_ip = get_client_ip(&headers);
    {
        let mut w = app.world.lock().await;
        let entry = w.auth_rate_limit.entry(client_ip).or_insert((0, Instant::now()));
        if entry.1.elapsed().as_secs() > 120 {
            entry.0 = 0;
            entry.1 = Instant::now();
        }
        entry.0 += 1;
        if entry.0 > 5 {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::json!({ "success": false, "error": "Забагато спроб. Зачекайте 2 хвилини" })),
            ).into_response();
        }
    }
    match db::login_account(&app.db, &payload.email, &payload.password).await {
        Ok(acc) => Json(serde_json::json!({ "success": true, "account": acc })).into_response(),
        Err(err) => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "success": false, "error": err })),
        ).into_response(),
    }
}

async fn api_admin_accounts(
    State(app): State<App>,
    headers: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let client_ip = get_client_ip(&headers);
    if let Err(status) = check_admin_auth(&app, &headers, &client_ip).await {
        return status.into_response();
    }
    let limit = q.get("limit").and_then(|v| v.parse().ok()).unwrap_or(100);
    match db::admin_list_accounts(&app.db, limit).await {
        Ok(accounts) => Json(serde_json::json!({ "accounts": accounts })).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": err.to_string() })),
        ).into_response(),
    }
}

async fn api_admin_reset_password(
    State(app): State<App>,
    headers: HeaderMap,
    Json(payload): Json<ApiResetPasswordReq>,
) -> Response {
    let client_ip = get_client_ip(&headers);
    if let Err(status) = check_admin_auth(&app, &headers, &client_ip).await {
        return status.into_response();
    }
    if payload.new_password.len() < 6 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "error": "Пароль має містити щонайменше 6 символів" })),
        ).into_response();
    }
    match db::admin_reset_account_password(&app.db, &payload.email, &payload.new_password).await {
        Ok(true) => {
            let _ = db::add_log(&app.db, "admin", &format!("Скинуто пароль для акаунту {}", payload.email)).await;
            Json(serde_json::json!({ "success": true, "message": "Пароль успішно змінено" })).into_response()
        }
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "success": false, "error": "Акаунт з такою поштою не знайдено" })),
        ).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "success": false, "error": err.to_string() })),
        ).into_response(),
    }
}


#[derive(serde::Deserialize)]
struct ApiChangePasswordReq {
    email: String,
    old_password: String,
    new_password: String,
}

async fn api_account_change_password(
    State(app): State<App>,
    headers: HeaderMap,
    Json(payload): Json<ApiChangePasswordReq>,
) -> Response {
    // Rate limit: max 5 attempts per 2 minutes per IP
    let client_ip = get_client_ip(&headers);
    {
        let mut w = app.world.lock().await;
        let entry = w.auth_rate_limit.entry(client_ip).or_insert((0, Instant::now()));
        if entry.1.elapsed().as_secs() > 120 {
            entry.0 = 0;
            entry.1 = Instant::now();
        }
        entry.0 += 1;
        if entry.0 > 5 {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::json!({ "success": false, "error": "Забагато спроб. Зачекайте 2 хвилини" })),
            ).into_response();
        }
    }
    match db::change_account_password(&app.db, &payload.email, &payload.old_password, &payload.new_password).await {
        Ok(()) => Json(serde_json::json!({ "success": true, "message": "Пароль успішно змінено" })).into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "error": err })),
        ).into_response(),
    }
}


#[derive(serde::Deserialize)]
struct ApiGoogleLoginReq {
    email: String,
    username: Option<String>,
    firebase_uid: String,
}

async fn api_account_google(
    State(app): State<App>,
    Json(payload): Json<ApiGoogleLoginReq>,
) -> Response {
    let username = payload.username.unwrap_or_default();
    match db::google_login_account(&app.db, &payload.email, &username, &payload.firebase_uid).await {
        Ok(account) => Json(serde_json::json!({ "success": true, "account": account })).into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "error": err })),
        ).into_response(),
    }
}

#[derive(serde::Deserialize)]
struct ApiOneTimeCodeReq {
    email: String,
}

async fn api_account_one_time_code(
    State(app): State<App>,
    Json(payload): Json<ApiOneTimeCodeReq>,
) -> Response {
    match db::generate_login_code(&app.db, &payload.email).await {
        Ok(code) => Json(serde_json::json!({ "success": true, "code": code, "expires_in": 900 })).into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "error": err })),
        ).into_response(),
    }
}

#[derive(serde::Deserialize)]
struct ApiCodeLoginReq {
    code: String,
}

async fn api_account_code_login(
    State(app): State<App>,
    Json(payload): Json<ApiCodeLoginReq>,
) -> Response {
    match db::login_with_code(&app.db, &payload.code).await {
        Ok(account) => Json(serde_json::json!({ "success": true, "account": account })).into_response(),
        Err(err) => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "success": false, "error": err })),
        ).into_response(),
    }
}

async fn api_account_profile(
    State(app): State<App>,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let email = match q.get("email") {
        Some(e) if !e.trim().is_empty() => e.trim().to_lowercase(),
        _ => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Потрібен email" }))).into_response(),
    };

    let row = sqlx::query(
        r#"
        SELECT id, email, username, gold, score, fish, big_fish,
               inventory, tools, houses, last_x, last_y, created_at, last_login
        FROM accounts WHERE email = ?
        "#
    )
    .bind(&email)
    .fetch_optional(&app.db)
    .await;

    match row {
        Ok(Some(r)) => {
            let inv_str: String = r.get("inventory");
            let tools_str: String = r.get("tools");
            let inventory: Vec<u32> = serde_json::from_str(&inv_str).unwrap_or_default();
            let tools: Vec<i32> = serde_json::from_str(&tools_str).unwrap_or_default();

            // Check if player is currently online
            let is_online = {
                let w = app.world.lock().await;
                w.players.values().any(|c| c.account_email.as_deref() == Some(&email))
            };

            Json(serde_json::json!({
                "success": true,
                "profile": {
                    "id": r.get::<i64, _>("id"),
                    "email": r.get::<String, _>("email"),
                    "username": r.get::<String, _>("username"),
                    "gold": r.get::<i64, _>("gold"),
                    "score": r.get::<i64, _>("score"),
                    "fish": r.get::<i64, _>("fish"),
                    "big_fish": r.get::<f64, _>("big_fish"),
                    "houses": r.get::<i64, _>("houses"),
                    "inventory": inventory,
                    "tools": tools,
                    "x": r.get::<f64, _>("last_x"),
                    "y": r.get::<f64, _>("last_y"),
                    "created_at": r.get::<i64, _>("created_at"),
                    "last_login": r.get::<i64, _>("last_login"),
                    "is_online": is_online
                }
            })).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Акаунт не знайдено" }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}
