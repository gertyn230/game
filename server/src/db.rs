//! База даних для адмінки: логи, налаштування, історія дій
use sqlx::{SqlitePool, Row};
use serde::{Serialize, Deserialize};
use sha2::{Digest, Sha256};
use rand::Rng;
use shared::AccountData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminLog {
    pub id: i64,
    pub admin: String,
    pub action: String,
    pub timestamp: i64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub key: String,
    pub value: String,
}

pub async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    // Use persistent file database by default for production
    std::fs::create_dir_all("data").ok();
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://./data/game.db?mode=rwc".to_string());

    let pool = SqlitePool::connect(&db_url).await?;

    // Створити таблиці
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS admin_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            admin TEXT NOT NULL,
            action TEXT NOT NULL,
            timestamp INTEGER NOT NULL
        )
        "#
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS server_config (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )
        "#
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tickets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            player TEXT NOT NULL,
            type TEXT NOT NULL,
            subject TEXT NOT NULL,
            message TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at INTEGER NOT NULL
        )
        "#
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS news (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            body TEXT NOT NULL,
            audience TEXT NOT NULL,
            type TEXT NOT NULL,
            send_at INTEGER NOT NULL,
            status TEXT NOT NULL
        )
        "#
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS promocodes (
            code TEXT PRIMARY KEY,
            reward TEXT NOT NULL,
            used INTEGER NOT NULL DEFAULT 0,
            limit_uses INTEGER NOT NULL,
            expires TEXT NOT NULL,
            active INTEGER NOT NULL DEFAULT 1
        )
        "#
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS promocode_redemptions (
            code TEXT NOT NULL,
            player TEXT NOT NULL,
            redeemed_at INTEGER NOT NULL,
            PRIMARY KEY (code, player)
        )
        "#
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS game_updates (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            version TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            date TEXT NOT NULL,
            created_at INTEGER NOT NULL
        )
        "#
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS accounts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            email TEXT UNIQUE NOT NULL COLLATE NOCASE,
            username TEXT NOT NULL,
            password_hash TEXT NOT NULL,
            firebase_uid TEXT,
            gold INTEGER NOT NULL DEFAULT 0,
            score INTEGER NOT NULL DEFAULT 0,
            fish INTEGER NOT NULL DEFAULT 0,
            big_fish REAL NOT NULL DEFAULT 0.0,
            inventory TEXT NOT NULL DEFAULT '[]',
            tools TEXT NOT NULL DEFAULT '[]',
            houses INTEGER NOT NULL DEFAULT 0,
            last_x REAL NOT NULL DEFAULT 20.5,
            last_y REAL NOT NULL DEFAULT 20.5,
            created_at INTEGER NOT NULL,
            last_login INTEGER NOT NULL
        )
        "#
    )
    .execute(&pool)
    .await?;

    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_accounts_email ON accounts(email)")
        .execute(&pool)
        .await;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS login_codes (
            code TEXT PRIMARY KEY,
            email TEXT NOT NULL,
            expires_at INTEGER NOT NULL
        )
        "#
    )
    .execute(&pool)
    .await?;

    // Add multilang columns (ignore errors if they already exist)
    let _ = sqlx::query("ALTER TABLE game_updates ADD COLUMN title_en TEXT DEFAULT ''").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE game_updates ADD COLUMN description_en TEXT DEFAULT ''").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE game_updates ADD COLUMN title_ru TEXT DEFAULT ''").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE game_updates ADD COLUMN description_ru TEXT DEFAULT ''").execute(&pool).await;

    let upd_count: i64 = sqlx::query_scalar("SELECT count(*) FROM game_updates")
        .fetch_one(&pool)
        .await
        .unwrap_or(0);

    let has_180: Option<i64> = sqlx::query_scalar("SELECT 1 FROM game_updates WHERE version = 'v1.8.0'")
        .fetch_optional(&pool)
        .await
        .unwrap_or(None);
    if has_180.is_none() {
        let _ = create_game_update(&pool, "v1.8.0", "Особистий кабінет, Google авторизація та обов'язковий вхід", "Додано багатосторінковий портал з особистим кабінетом, вхід через Google та 6-значний код, обов'язкову авторизацію перед грою та самостійну зміну пароля.", "26 Листопада 2026", "Player cabinet, Google Auth and mandatory login", "Added multi-page portal with player cabinet, Google login and one-time code, mandatory authorization before playing and password change.", "Личный кабинет, Google авторизация и обязательный вход", "Добавлен многостраничный портал с личным кабинетом, вход через Google и 6-значный код, обязательная авторизация перед игрой и смена пароля.").await;
    }

    if upd_count == 0 {
        let _ = create_game_update(&pool, "v1.7.5", "Онлайн мультиплеєр та оптимізація", "Підключено високошвидкісний європейський сервер pixel-village.online, додано підтримку промокодів та систему тікетів.", "21 Листопада 2026", "Online multiplayer and optimization", "Connected high-speed European server pixel-village.online, added promo code support and ticket system.", "Онлайн мультиплеер и оптимизация", "Подключен высокоскоростной европейский сервер pixel-village.online, добавлена поддержка промокодов и система тикетов.").await;
        let _ = create_game_update(&pool, "v1.7.3", "Новий інвентар та крафт", "Додано крафт на гарячі клавіші, налаштування керування та покращено штучний інтелект мобів.", "18 Листопада 2026", "New inventory and craft", "Added hotkey craft, control settings and improved mob AI.", "Новый инвентарь и крафт", "Добавлен крафт на горячие клавиши, настройки управления и улучшен искусственный интеллект мобов.").await;
        let _ = create_game_update(&pool, "v1.6.0", "Переробка меню налаштувань", "Повністю переписане меню з 4 категоріями: Мова, Аудіо, Керування, Графіка. Покращена структура коду.", "12 Листопада 2026", "Settings menu redesign", "Completely rewritten menu with 4 categories: Language, Audio, Controls, Graphics. Improved code structure.", "Переработка меню настроек", "Полностью переписанное меню с 4 категориями: Язык, Аудио, Управление, Графика. Улучшена структура кода.").await;
        let _ = create_game_update(&pool, "v1.5.0", "Система автооновлення", "Гра автоматично перевіряє наявність нових версій та завантажує оновлення прямо в клієнті.", "07 Листопада 2026", "Auto-update system", "The game automatically checks for updates and downloads them in client.", "Система автообновления", "Игра автоматически проверяет наличие новых версий и скачивает их прямо в клиенте.").await;
    }

    Ok(pool)
}

pub async fn add_log(pool: &SqlitePool, admin: &str, action: &str) -> Result<(), sqlx::Error> {
    let timestamp = chrono::Utc::now().timestamp();
    sqlx::query(
        "INSERT INTO admin_logs (admin, action, timestamp) VALUES (?, ?, ?)"
    )
    .bind(admin)
    .bind(action)
    .bind(timestamp)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_logs(pool: &SqlitePool, limit: i32) -> Result<Vec<AdminLog>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, admin, action, timestamp FROM admin_logs ORDER BY id DESC LIMIT ?"
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows.iter().map(|row| AdminLog {
        id: row.get("id"),
        admin: row.get("admin"),
        action: row.get("action"),
        timestamp: row.get("timestamp"),
    }).collect())
}

pub async fn set_config(pool: &SqlitePool, key: &str, value: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT OR REPLACE INTO server_config (key, value) VALUES (?, ?)"
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_config(pool: &SqlitePool, key: &str) -> Result<Option<String>, sqlx::Error> {
    let row = sqlx::query(
        "SELECT value FROM server_config WHERE key = ?"
    )
    .bind(key)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| r.get("value")))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticket {
    pub id: i64,
    pub player: String,
    pub ticket_type: String,
    pub subject: String,
    pub message: String,
    pub status: String,
    pub created_at: i64,
}

pub async fn create_ticket(pool: &SqlitePool, player: &str, ticket_type: &str, subject: &str, message: &str) -> Result<i64, sqlx::Error> {
    let timestamp = chrono::Utc::now().timestamp();
    let result = sqlx::query(
        "INSERT INTO tickets (player, type, subject, message, status, created_at) VALUES (?, ?, ?, ?, 'нова', ?)"
    )
    .bind(player)
    .bind(ticket_type)
    .bind(subject)
    .bind(message)
    .bind(timestamp)
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}

pub async fn get_tickets(pool: &SqlitePool, status_filter: Option<&str>) -> Result<Vec<Ticket>, sqlx::Error> {
    let rows = if let Some(status) = status_filter {
        sqlx::query("SELECT * FROM tickets WHERE status = ? ORDER BY id DESC")
            .bind(status)
            .fetch_all(pool)
            .await?
    } else {
        sqlx::query("SELECT * FROM tickets ORDER BY id DESC")
            .fetch_all(pool)
            .await?
    };

    Ok(rows.iter().map(|row| Ticket {
        id: row.get("id"),
        player: row.get("player"),
        ticket_type: row.get("type"),
        subject: row.get("subject"),
        message: row.get("message"),
        status: row.get("status"),
        created_at: row.get("created_at"),
    }).collect())
}

pub async fn update_ticket_status(pool: &SqlitePool, id: i64, status: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE tickets SET status = ? WHERE id = ?")
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromoCode {
    pub code: String,
    pub reward: String,
    pub used: i64,
    pub limit_uses: i64,
    pub expires: String,
    pub active: i64,
}

pub async fn get_promocodes(pool: &SqlitePool) -> Result<Vec<PromoCode>, sqlx::Error> {
    let rows = sqlx::query("SELECT code, reward, used, limit_uses, expires, active FROM promocodes ORDER BY code ASC")
        .fetch_all(pool)
        .await?;

    Ok(rows.iter().map(|row| PromoCode {
        code: row.get("code"),
        reward: row.get("reward"),
        used: row.get("used"),
        limit_uses: row.get("limit_uses"),
        expires: row.get("expires"),
        active: row.get("active"),
    }).collect())
}

pub async fn add_promocode(pool: &SqlitePool, code: &str, reward: &str, limit_uses: i64, expires: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT OR REPLACE INTO promocodes (code, reward, used, limit_uses, expires, active) VALUES (?, ?, 0, ?, ?, 1)"
    )
    .bind(code)
    .bind(reward)
    .bind(limit_uses)
    .bind(expires)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn toggle_promocode(pool: &SqlitePool, code: &str) -> Result<i64, sqlx::Error> {
    let row = sqlx::query("SELECT active FROM promocodes WHERE code = ?")
        .bind(code)
        .fetch_optional(pool)
        .await?;

    if let Some(r) = row {
        let act: i64 = r.get("active");
        let new_act = if act == 1 { 0 } else { 1 };
        sqlx::query("UPDATE promocodes SET active = ? WHERE code = ?")
            .bind(new_act)
            .bind(code)
            .execute(pool)
            .await?;
        Ok(new_act)
    } else {
        Ok(0)
    }
}

pub async fn delete_promocode(pool: &SqlitePool, code: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM promocodes WHERE code = ?")
        .bind(code)
        .execute(pool)
        .await?;
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
pub enum RedeemResult {
    Success(String),
    AlreadyRedeemed,
    InvalidOrExpired,
}

pub async fn redeem_promocode(pool: &SqlitePool, code: &str, player: &str) -> Result<RedeemResult, sqlx::Error> {
    let row = sqlx::query("SELECT reward, used, limit_uses, active FROM promocodes WHERE code = ?")
        .bind(code)
        .fetch_optional(pool)
        .await?;

    if let Some(r) = row {
        let active: i64 = r.get("active");
        let used: i64 = r.get("used");
        let limit_uses: i64 = r.get("limit_uses");
        let reward: String = r.get("reward");

        if active == 0 || used >= limit_uses {
            return Ok(RedeemResult::InvalidOrExpired);
        }

        let clean_player = player.trim();
        if !clean_player.is_empty() {
            let already: Option<i64> = sqlx::query_scalar("SELECT 1 FROM promocode_redemptions WHERE code = ? AND player = ?")
                .bind(code)
                .bind(clean_player)
                .fetch_optional(pool)
                .await?;
            if already.is_some() {
                return Ok(RedeemResult::AlreadyRedeemed);
            }
        }

        sqlx::query("UPDATE promocodes SET used = used + 1 WHERE code = ?")
            .bind(code)
            .execute(pool)
            .await?;

        if !clean_player.is_empty() {
            let now = chrono::Utc::now().timestamp();
            let _ = sqlx::query("INSERT INTO promocode_redemptions (code, player, redeemed_at) VALUES (?, ?, ?)")
                .bind(code)
                .bind(clean_player)
                .bind(now)
                .execute(pool)
                .await;
        }

        Ok(RedeemResult::Success(reward))
    } else {
        Ok(RedeemResult::InvalidOrExpired)
    }
}




#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameUpdate {
    pub id: i64,
    pub version: String,
    pub title: String,
    pub description: String,
    pub date: String,
    #[serde(default)]
    pub title_en: String,
    #[serde(default)]
    pub description_en: String,
    #[serde(default)]
    pub title_ru: String,
    #[serde(default)]
    pub description_ru: String,
    pub created_at: i64,
}

pub async fn get_game_updates(pool: &SqlitePool) -> Result<Vec<GameUpdate>, sqlx::Error> {
    let rows = sqlx::query("SELECT id, version, title, description, date, COALESCE(title_en, '') as title_en, COALESCE(description_en, '') as description_en, COALESCE(title_ru, '') as title_ru, COALESCE(description_ru, '') as description_ru, created_at FROM game_updates ORDER BY id DESC")
        .fetch_all(pool)
        .await?;

    Ok(rows.iter().map(|row| GameUpdate {
        id: row.get("id"),
        version: row.get("version"),
        title: row.get("title"),
        description: row.get("description"),
        date: row.get("date"),
        title_en: row.get("title_en"),
        description_en: row.get("description_en"),
        title_ru: row.get("title_ru"),
        description_ru: row.get("description_ru"),
        created_at: row.get("created_at"),
    }).collect())
}

pub async fn create_game_update(
    pool: &SqlitePool,
    version: &str,
    title: &str,
    description: &str,
    date: &str,
    title_en: &str,
    description_en: &str,
    title_ru: &str,
    description_ru: &str,
) -> Result<i64, sqlx::Error> {
    let timestamp = chrono::Utc::now().timestamp();
    let result = sqlx::query(
        "INSERT INTO game_updates (version, title, description, date, title_en, description_en, title_ru, description_ru, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(version)
    .bind(title)
    .bind(description)
    .bind(date)
    .bind(title_en)
    .bind(description_en)
    .bind(title_ru)
    .bind(description_ru)
    .bind(timestamp)
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}

pub async fn update_game_update(
    pool: &SqlitePool,
    id: i64,
    version: &str,
    title: &str,
    description: &str,
    date: &str,
    title_en: &str,
    description_en: &str,
    title_ru: &str,
    description_ru: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE game_updates SET version = ?, title = ?, description = ?, date = ?, title_en = ?, description_en = ?, title_ru = ?, description_ru = ? WHERE id = ?"
    )
    .bind(version)
    .bind(title)
    .bind(description)
    .bind(date)
    .bind(title_en)
    .bind(description_en)
    .bind(title_ru)
    .bind(description_ru)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_game_update(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM game_updates WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// ---------------------------------------------------------
// Система акаунтів (Email + Пароль)
// ---------------------------------------------------------

pub fn hash_password(password: &str) -> String {
    let mut salt_bytes = [0u8; 16];
    rand::thread_rng().fill(&mut salt_bytes);
    let salt = hex::encode(salt_bytes);
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(password.as_bytes());
    let hash = hex::encode(hasher.finalize());
    format!("{}:{}", salt, hash)
}

pub fn verify_password(password: &str, stored: &str) -> bool {
    let parts: Vec<&str> = stored.split(':').collect();
    if parts.len() != 2 {
        return false;
    }
    let salt = parts[0];
    let expected_hash = parts[1];
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(password.as_bytes());
    let hash = hex::encode(hasher.finalize());
    hash == expected_hash
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountAdminView {
    pub id: i64,
    pub email: String,
    pub username: String,
    pub gold: i64,
    pub score: i64,
    pub fish: i64,
    pub houses: i64,
    pub created_at: i64,
    pub last_login: i64,
}

pub async fn register_account(
    pool: &SqlitePool,
    email: &str,
    username: &str,
    password: &str,
) -> Result<AccountData, String> {
    let email = email.trim().to_lowercase();
    let username = username.trim().to_string();

    if email.len() < 5 || email.len() > 100 || !email.contains('@') || !email.contains('.') {
        return Err("Введіть коректну адресу електронної пошти (наприклад: name@gmail.com)".to_string());
    }

    if username.len() < 2 || username.len() > 24 {
        return Err("Нікнейм має містити від 2 до 24 символів".to_string());
    }

    if password.len() < 6 {
        return Err("Пароль має містити щонайменше 6 символів".to_string());
    }

    if password.len() > 128 {
        return Err("Пароль занадто довгий".to_string());
    }

    // Перевірка чи пошта вже зайнята
    let existing: Option<i64> = sqlx::query_scalar("SELECT id FROM accounts WHERE email = ?")
        .bind(&email)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Помилка бази даних: {}", e))?;

    if existing.is_some() {
        return Err("Акаунт з такою поштою вже існує. Будь ласка, увійдіть".to_string());
    }

    let password_hash = hash_password(password);
    let now = chrono::Utc::now().timestamp();

    let res = sqlx::query(
        r#"
        INSERT INTO accounts (
            email, username, password_hash, gold, score, fish, big_fish,
            inventory, tools, houses, last_x, last_y, created_at, last_login
        ) VALUES (?, ?, ?, 0, 0, 0, 0.0, '[]', '[]', 0, 20.5, 20.5, ?, ?)
        "#
    )
    .bind(&email)
    .bind(&username)
    .bind(&password_hash)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| format!("Помилка збереження акаунту: {}", e))?;

    if res.rows_affected() == 0 {
        return Err("Не вдалося створити акаунт".to_string());
    }

    Ok(AccountData {
        email,
        username,
        gold: 0,
        score: 0,
        fish: 0,
        big_fish: 0.0,
        inventory: Vec::new(),
        tools: Vec::new(),
        houses: 0,
        x: 20.5,
        y: 20.5,
    })
}

pub async fn login_account(
    pool: &SqlitePool,
    email: &str,
    password: &str,
) -> Result<AccountData, String> {
    let email = email.trim().to_lowercase();

    let row = sqlx::query(
        r#"
        SELECT email, username, password_hash, gold, score, fish, big_fish,
               inventory, tools, houses, last_x, last_y
        FROM accounts WHERE email = ?
        "#
    )
    .bind(&email)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Помилка бази даних: {}", e))?;

    let row = match row {
        Some(r) => r,
        None => return Err("Акаунт з такою поштою не знайдено".to_string()),
    };

    let stored_hash: String = row.get("password_hash");
    if !verify_password(password, &stored_hash) {
        return Err("Невірний пароль".to_string());
    }

    let now = chrono::Utc::now().timestamp();
    let _ = sqlx::query("UPDATE accounts SET last_login = ? WHERE email = ?")
        .bind(now)
        .bind(&email)
        .execute(pool)
        .await;

    let username: String = row.get("username");
    let gold: i64 = row.get("gold");
    let score: i64 = row.get("score");
    let fish: i64 = row.get("fish");
    let big_fish: f64 = row.get("big_fish");
    let inv_str: String = row.get("inventory");
    let tools_str: String = row.get("tools");
    let houses: i64 = row.get("houses");
    let x: f64 = row.get("last_x");
    let y: f64 = row.get("last_y");

    let inventory: Vec<u32> = serde_json::from_str(&inv_str).unwrap_or_default();
    let tools: Vec<i32> = serde_json::from_str(&tools_str).unwrap_or_default();

    Ok(AccountData {
        email,
        username,
        gold: gold.max(0) as u32,
        score: score.max(0) as u32,
        fish: fish.max(0) as u32,
        big_fish: big_fish as f32,
        inventory,
        tools,
        houses: houses.max(0) as u32,
        x: x as f32,
        y: y as f32,
    })
}

pub async fn save_account_data(
    pool: &SqlitePool,
    email: &str,
    gold: u32,
    score: u32,
    fish: u32,
    big_fish: f32,
    inventory: &[u32],
    tools: &[i32],
    houses: u32,
    x: f32,
    y: f32,
) -> Result<(), sqlx::Error> {
    let email = email.trim().to_lowercase();
    let inv_json = serde_json::to_string(inventory).unwrap_or_else(|_| "[]".to_string());
    let tools_json = serde_json::to_string(tools).unwrap_or_else(|_| "[]".to_string());

    sqlx::query(
        r#"
        UPDATE accounts SET
            gold = ?,
            score = ?,
            fish = ?,
            big_fish = ?,
            inventory = ?,
            tools = ?,
            houses = ?,
            last_x = ?,
            last_y = ?
        WHERE email = ?
        "#
    )
    .bind(gold as i64)
    .bind(score as i64)
    .bind(fish as i64)
    .bind(big_fish as f64)
    .bind(inv_json)
    .bind(tools_json)
    .bind(houses as i64)
    .bind(x as f64)
    .bind(y as f64)
    .bind(&email)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn admin_list_accounts(
    pool: &SqlitePool,
    limit: i64,
) -> Result<Vec<AccountAdminView>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, email, username, gold, score, fish, houses, created_at, last_login
        FROM accounts
        ORDER BY last_login DESC
        LIMIT ?
        "#
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .iter()
        .map(|row| AccountAdminView {
            id: row.get("id"),
            email: row.get("email"),
            username: row.get("username"),
            gold: row.get("gold"),
            score: row.get("score"),
            fish: row.get("fish"),
            houses: row.get("houses"),
            created_at: row.get("created_at"),
            last_login: row.get("last_login"),
        })
        .collect())
}

pub async fn admin_reset_account_password(
    pool: &SqlitePool,
    email: &str,
    new_password: &str,
) -> Result<bool, sqlx::Error> {
    let email = email.trim().to_lowercase();
    let new_hash = hash_password(new_password);

    let res = sqlx::query("UPDATE accounts SET password_hash = ? WHERE email = ?")
        .bind(new_hash)
        .bind(&email)
        .execute(pool)
        .await?;

    Ok(res.rows_affected() > 0)
}

pub async fn change_account_password(
    pool: &SqlitePool,
    email: &str,
    old_password: &str,
    new_password: &str,
) -> Result<(), String> {
    let email = email.trim().to_lowercase();

    if new_password.len() < 6 {
        return Err("Новий пароль має містити щонайменше 6 символів".to_string());
    }
    if new_password.len() > 128 {
        return Err("Пароль занадто довгий".to_string());
    }

    let row = sqlx::query("SELECT password_hash FROM accounts WHERE email = ?")
        .bind(&email)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Помилка бази даних: {}", e))?;

    let row = match row {
        Some(r) => r,
        None => return Err("Акаунт не знайдено".to_string()),
    };

    let stored_hash: String = row.get("password_hash");
    if !verify_password(old_password, &stored_hash) {
        return Err("Невірний поточний пароль".to_string());
    }

    let new_hash = hash_password(new_password);
    sqlx::query("UPDATE accounts SET password_hash = ? WHERE email = ?")
        .bind(&new_hash)
        .bind(&email)
        .execute(pool)
        .await
        .map_err(|e| format!("Помилка збереження: {}", e))?;

    Ok(())
}


pub async fn google_login_account(
    pool: &SqlitePool,
    email: &str,
    username: &str,
    firebase_uid: &str,
) -> Result<AccountData, String> {
    let email = email.trim().to_lowercase();
    let mut username = username.trim().to_string();

    if email.is_empty() || !email.contains('@') {
        return Err("Некоректний email від Google".to_string());
    }

    if username.is_empty() {
        username = email.split('@').next().unwrap_or("Player").to_string();
    }
    if username.chars().count() > 20 {
        username = username.chars().take(20).collect();
    }
    if username.chars().count() < 2 {
        let prefix: String = email.chars().take_while(|&c| c != '@').take(8).collect();
        username = if prefix.len() >= 2 { format!("User_{}", prefix) } else { "Player".to_string() };
    }

    let existing = sqlx::query(
        r#"
        SELECT email, username, gold, score, fish, big_fish,
               inventory, tools, houses, last_x, last_y
        FROM accounts WHERE email = ?
        "#
    )
    .bind(&email)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Помилка бази даних: {}", e))?;

    let now = chrono::Utc::now().timestamp();

    if let Some(row) = existing {
        // Update firebase_uid and last_login
        let _ = sqlx::query("UPDATE accounts SET firebase_uid = ?, last_login = ? WHERE email = ?")
            .bind(firebase_uid)
            .bind(now)
            .bind(&email)
            .execute(pool)
            .await;

        let uname: String = row.get("username");
        let gold: i64 = row.get("gold");
        let score: i64 = row.get("score");
        let fish: i64 = row.get("fish");
        let big_fish: f64 = row.get("big_fish");
        let inv_str: String = row.get("inventory");
        let tools_str: String = row.get("tools");
        let houses: i64 = row.get("houses");
        let x: f64 = row.get("last_x");
        let y: f64 = row.get("last_y");

        let inventory: Vec<u32> = serde_json::from_str(&inv_str).unwrap_or_default();
        let tools: Vec<i32> = serde_json::from_str(&tools_str).unwrap_or_default();

        Ok(AccountData {
            email,
            username: uname,
            gold: gold.max(0) as u32,
            score: score.max(0) as u32,
            fish: fish.max(0) as u32,
            big_fish: big_fish as f32,
            inventory,
            tools,
            houses: houses.max(0) as u32,
            x: x as f32,
            y: y as f32,
        })
    } else {
        // Create new account via Google
        let mut random_pwd = [0u8; 16];
        rand::thread_rng().fill(&mut random_pwd);
        let random_pwd_str = hex::encode(random_pwd);
        let password_hash = hash_password(&random_pwd_str);

        sqlx::query(
            r#"
            INSERT INTO accounts (
                email, username, password_hash, firebase_uid, gold, score, fish, big_fish,
                inventory, tools, houses, last_x, last_y, created_at, last_login
            ) VALUES (?, ?, ?, ?, 0, 0, 0, 0.0, '[]', '[]', 0, 20.5, 20.5, ?, ?)
            "#
        )
        .bind(&email)
        .bind(&username)
        .bind(&password_hash)
        .bind(firebase_uid)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await
        .map_err(|e| format!("Помилка створення акаунту через Google: {}", e))?;

        Ok(AccountData {
            email,
            username,
            gold: 0,
            score: 0,
            fish: 0,
            big_fish: 0.0,
            inventory: Vec::new(),
            tools: Vec::new(),
            houses: 0,
            x: 20.5,
            y: 20.5,
        })
    }
}

pub async fn generate_login_code(pool: &SqlitePool, email: &str) -> Result<String, String> {
    let email = email.trim().to_lowercase();
    let code: u32 = rand::thread_rng().gen_range(100_000..999_999);
    let code_str = code.to_string();
    let expires_at = chrono::Utc::now().timestamp() + 900; // 15 mins

    sqlx::query("INSERT OR REPLACE INTO login_codes (code, email, expires_at) VALUES (?, ?, ?)")
        .bind(&code_str)
        .bind(&email)
        .bind(expires_at)
        .execute(pool)
        .await
        .map_err(|e| format!("Помилка збереження коду: {}", e))?;

    Ok(code_str)
}

pub async fn login_with_code(pool: &SqlitePool, code: &str) -> Result<AccountData, String> {
    let code = code.trim();
    let now = chrono::Utc::now().timestamp();

    let row = sqlx::query("SELECT email, expires_at FROM login_codes WHERE code = ?")
        .bind(code)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Помилка бази даних: {}", e))?;

    let row = match row {
        Some(r) => r,
        None => return Err("Невірний або застарілий код входу".to_string()),
    };

    let expires_at: i64 = row.get("expires_at");
    if expires_at < now {
        let _ = sqlx::query("DELETE FROM login_codes WHERE code = ?").bind(code).execute(pool).await;
        return Err("Термін дії коду вичерпано. Згенеруйте новий в кабінеті".to_string());
    }

    let email: String = row.get("email");
    let _ = sqlx::query("DELETE FROM login_codes WHERE code = ?").bind(code).execute(pool).await;

    let acc_row = sqlx::query(
        r#"
        SELECT email, username, gold, score, fish, big_fish,
               inventory, tools, houses, last_x, last_y
        FROM accounts WHERE email = ?
        "#
    )
    .bind(&email)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Помилка: {}", e))?;

    let acc_row = match acc_row {
        Some(r) => r,
        None => {
            // Якщо запис акаунту ще не був створений (наприклад, збій під час Google авторизації),
            // створюємо його автоматично зараз, щоб гравець гарантовано міг увійти!
            let prefix: String = email.chars().take_while(|&c| c != '@').take(12).collect();
            let default_username = if prefix.len() >= 2 { prefix } else { "Player".to_string() };
            let _ = sqlx::query(
                r#"
                INSERT INTO accounts (email, username, password_hash, gold, score, fish, big_fish, inventory, tools, houses, last_x, last_y, created_at, last_login)
                VALUES (?, ?, '', 0, 0, 0, 0.0, '[]', '[]', 0, 0.0, 0.0, ?, ?)
                "#
            )
            .bind(&email)
            .bind(&default_username)
            .bind(now)
            .bind(now)
            .execute(pool)
            .await;

            return Ok(AccountData {
                email,
                username: default_username,
                gold: 0,
                score: 0,
                fish: 0,
                big_fish: 0.0,
                inventory: vec![],
                tools: vec![],
                houses: 0,
                x: 0.0,
                y: 0.0,
            });
        }
    };

    let username: String = acc_row.get("username");
    let gold: i64 = acc_row.get("gold");
    let score: i64 = acc_row.get("score");
    let fish: i64 = acc_row.get("fish");
    let big_fish: f64 = acc_row.get("big_fish");
    let inv_str: String = acc_row.get("inventory");
    let tools_str: String = acc_row.get("tools");
    let houses: i64 = acc_row.get("houses");
    let x: f64 = acc_row.get("last_x");
    let y: f64 = acc_row.get("last_y");

    let inventory: Vec<u32> = serde_json::from_str(&inv_str).unwrap_or_default();
    let tools: Vec<i32> = serde_json::from_str(&tools_str).unwrap_or_default();

    Ok(AccountData {
        email,
        username,
        gold: gold.max(0) as u32,
        score: score.max(0) as u32,
        fish: fish.max(0) as u32,
        big_fish: big_fish as f32,
        inventory,
        tools,
        houses: houses.max(0) as u32,
        x: x as f32,
        y: y as f32,
    })
}
