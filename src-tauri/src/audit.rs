use rusqlite::Connection;

use crate::error::AppError;

/// Supported audit actions for the auth feature.
pub mod actions {
    pub const LOGIN: &str = "LOGIN";
    pub const LOGOUT: &str = "LOGOUT";
    pub const LOGIN_FAILED: &str = "LOGIN_FAILED";
    pub const TOKEN_THEFT_DETECTED: &str = "TOKEN_THEFT_DETECTED";
}

/// Writes an audit log entry.
///
/// # Arguments
/// * `conn` — SQLite connection
/// * `user_id` — actor's user ID
/// * `user_email` — actor's email (denormalised for durability)
/// * `action` — action code (use `actions::*` constants)
/// * `entity_type` — type of entity being acted upon
/// * `entity_id` — ID of the entity being acted upon
/// * `changes` — optional JSON describing the changes
pub fn audit_log(
    conn: &Connection,
    user_id: &str,
    user_email: &str,
    action: &str,
    entity_type: &str,
    entity_id: &str,
    changes: Option<&str>,
) -> Result<(), AppError> {
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO audit_logs (id, entity_type, entity_id, action, changes, user_id, user_email)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![id, entity_type, entity_id, action, changes, user_id, user_email],
    )?;
    Ok(())
}
