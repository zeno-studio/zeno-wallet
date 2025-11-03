use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{OptionalExtension, params};
use rusqlite_migration::{M, Migrations};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;
use tauri::{AppHandle, Manager};

type DbPool = Pool<SqliteConnectionManager>;
static DB_POOL: OnceLock<Arc<DbPool>> = OnceLock::new();

#[derive(Serialize, Deserialize)]
pub struct Account {
    pub id: i64,
    pub name: String,
    pub address: String,
}

fn get_db_path(app: &AppHandle) -> PathBuf {
    app.path()
        .app_config_dir()
        .expect("Failed to get app config dir")
        .join("wallet.db")
}

fn init_db_pool(app: &AppHandle) -> DbPool {
    let db_path = get_db_path(app);
    let config_dir = db_path.parent().expect("Invalid db path");
    std::fs::create_dir_all(config_dir).expect("Failed to create config dir");
    let manager = SqliteConnectionManager::file(&db_path);
    let pool = Pool::new(manager).expect("Failed to create SQLite pool");
    let mut conn = pool.get().unwrap();

    // 完整性检查
    let result: String = conn
        .query_row("PRAGMA integrity_check;", [], |row| row.get(0))
        .unwrap_or_else(|_| "corrupt".to_string());
    if result != "ok" {
        panic!("❌ Database integrity check failed: {result}");
    }

    // 执行迁移
    let migrations = Migrations::new(vec![
        M::up("CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);"),
        M::up(
            r#"
            CREATE TABLE IF NOT EXISTS accounts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                address TEXT UNIQUE NOT NULL
            );
            "#,
        ),
        // 未来版本可继续追加 M::up("ALTER TABLE ...")
    ]);

    migrations.to_latest(&mut conn).expect("Migration failed");

    pool
}

pub fn init_db(app: &AppHandle) {
    let pool = Arc::new(init_db_pool(app));
    DB_POOL.set(pool).expect("Database already initialized");
}

fn conn() -> PooledConnection<SqliteConnectionManager> {
    DB_POOL
        .get()
        .expect("Database not initialized. Call db::init_db() first.")
        .get()
        .expect("DB connection error")
}

pub fn insert_account(name: &str, address: &str) -> rusqlite::Result<()> {
    let c = conn();
    c.execute(
        "INSERT OR IGNORE INTO accounts (name, address) VALUES (?1, ?2)",
        params![name, address],
    )?;
    Ok(())
}

pub fn list_accounts() -> rusqlite::Result<Vec<Account>> {
    let c = conn();
    let mut stmt = c.prepare("SELECT id, name, address FROM accounts ORDER BY id DESC")?;
    let rows = stmt.query_map([], |row| {
        Ok(Account {
            id: row.get(0)?,
            name: row.get(1)?,
            address: row.get(2)?,
        })
    })?;
    Ok(rows.filter_map(Result::ok).collect())
}

pub fn set_setting(key: &str, value: &str) -> rusqlite::Result<()> {
    let c = conn();
    c.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

pub fn get_setting(key: &str) -> rusqlite::Result<Option<String>> {
    let c = conn();
    c.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        params![key],
        |r| r.get(0),
    )
    .optional()
}
