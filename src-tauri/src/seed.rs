use argon2::PasswordHasher;
use rusqlite::Connection;

use crate::error::AppError;

/// Seeds a default admin account when running in debug mode.
///
/// Only runs when:
///   1. `cfg(debug_assertions)` is true (i.e. debug build).
///   2. The `users` table is empty.
///
/// This is a temporary convenience until Feature 002 formalises user creation.
/// No default passwords, hard-coded credentials, or dev admin paths are active
/// in production builds.
#[cfg(debug_assertions)]
pub fn maybe_seed_admin(conn: &Connection) -> Result<(), AppError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM users",
        [],
        |row| row.get(0),
    )?;

    if count > 0 {
        return Ok(());
    }

    let id = uuid::Uuid::new_v4().to_string();
    let email = "admin@daraerp.local";
    let name = "Admin";
    let role = "ADMIN";
    let language = "en";

    let salt = argon2::password_hash::SaltString::generate(&mut rand::rngs::OsRng);
    let hash = argon2::Argon2::default()
        .hash_password(b"admin123", &salt)
        .map_err(|e| AppError::new("INTERNAL_ERROR", format!("Failed to hash password: {e}")))?
        .to_string();

    conn.execute(
        "INSERT INTO users (id, email, name, password_hash, role, language) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![id, email, name, hash, role, language],
    )?;

    println!("[seed] Created default admin: admin@daraerp.local (DEBUG ONLY)");

    Ok(())
}

/// No-op in release builds.
#[cfg(not(debug_assertions))]
pub fn maybe_seed_admin(_conn: &Connection) -> Result<(), AppError> {
    Ok(())
}
