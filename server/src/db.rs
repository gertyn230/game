//! База даних для адмінки: логи, налаштування, історія дій
use sqlx::{SqlitePool, Row};
use serde::{Serialize, Deserialize};

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

    // Add multilang columns (ignore errors if they already exist)
    let _ = sqlx::query("ALTER TABLE game_updates ADD COLUMN title_en TEXT DEFAULT ''").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE game_updates ADD COLUMN description_en TEXT DEFAULT ''").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE game_updates ADD COLUMN title_ru TEXT DEFAULT ''").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE game_updates ADD COLUMN description_ru TEXT DEFAULT ''").execute(&pool).await;

    let upd_count: i64 = sqlx::query_scalar("SELECT count(*) FROM game_updates")
        .fetch_one(&pool)
        .await
        .unwrap_or(0);

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
