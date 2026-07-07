use rusqlite::Connection;
use serde::Serialize;

use crate::error::AppError;

// ---------------------------------------------------------------------------
// Row types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub password_hash: String,
    pub role: String,
    pub language: String,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct RefreshToken {
    pub id: String,
    pub token_hash: String,
    pub family_id: String,
    pub user_id: String,
    pub expires_at: String,
    pub is_revoked: bool,
}

// ---------------------------------------------------------------------------
// User queries
// ---------------------------------------------------------------------------

pub fn find_user_by_email(conn: &Connection, email: &str) -> Result<User, AppError> {
    conn.query_row(
        "SELECT id, email, name, password_hash, role, language, is_active
         FROM users WHERE email = ?1",
        [email],
        |row| {
            Ok(User {
                id: row.get(0)?,
                email: row.get(1)?,
                name: row.get(2)?,
                password_hash: row.get(3)?,
                role: row.get(4)?,
                language: row.get(5)?,
                is_active: row.get::<_, i32>(6)? != 0,
            })
        },
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            AppError::new("NOT_FOUND", "User not found")
        }
        other => AppError::from(other),
    })
}

pub fn find_user_by_id(conn: &Connection, id: &str) -> Result<User, AppError> {
    conn.query_row(
        "SELECT id, email, name, password_hash, role, language, is_active
         FROM users WHERE id = ?1",
        [id],
        |row| {
            Ok(User {
                id: row.get(0)?,
                email: row.get(1)?,
                name: row.get(2)?,
                password_hash: row.get(3)?,
                role: row.get(4)?,
                language: row.get(5)?,
                is_active: row.get::<_, i32>(6)? != 0,
            })
        },
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            AppError::new("NOT_FOUND", "User not found")
        }
        other => AppError::from(other),
    })
}

// ---------------------------------------------------------------------------
// Refresh token queries
// ---------------------------------------------------------------------------

pub fn insert_refresh_token(
    conn: &Connection,
    id: &str,
    user_id: &str,
    family_id: &str,
    token_hash: &str,
    expires_at: &str,
) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO refresh_tokens (id, user_id, family_id, token_hash, expires_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, user_id, family_id, token_hash, expires_at],
    )?;
    Ok(())
}

pub fn find_refresh_token_by_hash(
    conn: &Connection,
    token_hash: &str,
) -> Result<RefreshToken, AppError> {
    conn.query_row(
        "SELECT id, token_hash, family_id, user_id, expires_at, is_revoked
         FROM refresh_tokens WHERE token_hash = ?1",
        [token_hash],
        |row| {
            Ok(RefreshToken {
                id: row.get(0)?,
                token_hash: row.get(1)?,
                family_id: row.get(2)?,
                user_id: row.get(3)?,
                expires_at: row.get(4)?,
                is_revoked: row.get::<_, i32>(5)? != 0,
            })
        },
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            AppError::new("NOT_FOUND", "Refresh token not found")
        }
        other => AppError::from(other),
    })
}

pub fn revoke_refresh_token(conn: &Connection, id: &str) -> Result<(), AppError> {
    conn.execute(
        "UPDATE refresh_tokens SET is_revoked = 1 WHERE id = ?1",
        [id],
    )?;
    Ok(())
}

pub fn revoke_family(conn: &Connection, family_id: &str) -> Result<(), AppError> {
    conn.execute(
        "UPDATE refresh_tokens SET is_revoked = 1 WHERE family_id = ?1",
        [family_id],
    )?;
    Ok(())
}

#[allow(dead_code)]
pub fn get_active_family_id(conn: &Connection, user_id: &str) -> Result<Option<String>, AppError> {
    let result: Result<String, rusqlite::Error> = conn.query_row(
        "SELECT family_id FROM refresh_tokens
         WHERE user_id = ?1 AND is_revoked = 0
         ORDER BY created_at DESC LIMIT 1",
        [user_id],
        |row| row.get(0),
    );
    match result {
        Ok(family_id) => Ok(Some(family_id)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::from(e)),
    }
}
