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

pub async fn redeem_promocode(pool: &SqlitePool, code: &str) -> Result<Option<String>, sqlx::Error> {
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
            return Ok(None);
        }

        sqlx::query("UPDATE promocodes SET used = used + 1 WHERE code = ?")
            .bind(code)
            .execute(pool)
            .await?;

        Ok(Some(reward))
    } else {
        Ok(None)
    }
}


