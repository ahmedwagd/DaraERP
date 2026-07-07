use std::sync::Mutex;
use rusqlite::Connection;
use tauri::AppHandle;
use tauri::Manager;

use crate::error::AppError;

pub struct AppState {
    #[allow(dead_code)]
    pub db: Mutex<Connection>,
}

pub fn init_db(app_handle: &AppHandle) -> Result<Connection, AppError> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::new("IO_ERROR", e.to_string()))?;
    std::fs::create_dir_all(&app_data_dir)?;

    let db_path = app_data_dir.join("daraerp.db");
    let conn = Connection::open(&db_path)?;

    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    conn.execute_batch("PRAGMA journal_mode = WAL;")?;

    run_migrations(&conn)?;

    Ok(conn)
}

pub fn run_migrations(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version     TEXT PRIMARY KEY,
            applied_at  TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )?;

    let migrations: Vec<(&str, &str)> = vec![
        ("001_init", include_str!("../migrations/001_init.sql")),
    ];

    for (version, sql) in &migrations {
        let already_applied: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM schema_migrations WHERE version = ?1",
            [version],
            |row| row.get(0),
        )?;

        if already_applied {
            continue;
        }

        let tx = conn.unchecked_transaction()?;

        if let Err(e) = tx.execute_batch(sql) {
            let _ = tx.rollback();
            return Err(AppError::new(
                "DATABASE_ERROR",
                format!("Migration {version} failed: {e}"),
            ));
        }

        if let Err(e) = tx.execute(
            "INSERT INTO schema_migrations (version) VALUES (?1)",
            [version],
        ) {
            let _ = tx.rollback();
            return Err(AppError::new(
                "DATABASE_ERROR",
                format!("Failed to record migration {version}: {e}"),
            ));
        }

        tx.commit()?;
    }

    Ok(())
}
