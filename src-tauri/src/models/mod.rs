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
    pub created_at: String,
    pub updated_at: String,
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
// User queries (Feature 001 + 002)
// ---------------------------------------------------------------------------

fn row_to_user(row: &rusqlite::Row) -> rusqlite::Result<User> {
    Ok(User {
        id: row.get(0)?,
        email: row.get(1)?,
        name: row.get(2)?,
        password_hash: row.get(3)?,
        role: row.get(4)?,
        language: row.get(5)?,
        is_active: row.get::<_, i32>(6)? != 0,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

pub fn find_user_by_email(conn: &Connection, email: &str) -> Result<User, AppError> {
    conn.query_row(
        "SELECT id, email, name, password_hash, role, language, is_active, created_at, updated_at
         FROM users WHERE email = ?1",
        [email],
        row_to_user,
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
        "SELECT id, email, name, password_hash, role, language, is_active, created_at, updated_at
         FROM users WHERE id = ?1",
        [id],
        row_to_user,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            AppError::new("NOT_FOUND", "User not found")
        }
        other => AppError::from(other),
    })
}

pub fn insert_user(
    conn: &Connection,
    id: &str,
    email: &str,
    name: &str,
    password_hash: &str,
    role: &str,
    language: &str,
) -> Result<User, AppError> {
    conn.execute(
        "INSERT INTO users (id, email, name, password_hash, role, language)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![id, email, name, password_hash, role, language],
    )
    .map_err(|e| match &e {
        rusqlite::Error::SqliteFailure(err, _)
            if err.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            AppError::new("CONFLICT", "A user with this email already exists")
        }
        _ => AppError::from(e),
    })?;
    find_user_by_id(conn, id)
}

pub fn list_users(
    conn: &Connection,
    role_filter: Option<&str>,
    active_filter: Option<bool>,
) -> Result<Vec<User>, AppError> {
    let mut sql = String::from(
        "SELECT id, email, name, password_hash, role, language, is_active, created_at, updated_at
         FROM users WHERE 1=1",
    );
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(role) = role_filter {
        sql.push_str(" AND role = ?");
        params.push(Box::new(role.to_string()));
    }
    if let Some(active) = active_filter {
        sql.push_str(" AND is_active = ?");
        params.push(Box::new(if active { 1i32 } else { 0i32 }));
    }
    sql.push_str(" ORDER BY created_at DESC");

    let mut stmt = conn.prepare(&sql)?;
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let rows = stmt.query_map(param_refs.as_slice(), row_to_user)?;

    let mut users = Vec::new();
    for row in rows {
        users.push(row?);
    }
    Ok(users)
}

pub fn update_user(
    conn: &Connection,
    id: &str,
    email: Option<&str>,
    name: Option<&str>,
    role: Option<&str>,
    language: Option<&str>,
) -> Result<User, AppError> {
    let mut sql = String::from("UPDATE users SET updated_at = datetime('now')");
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(v) = email {
        sql.push_str(", email = ?");
        params.push(Box::new(v.to_string()));
    }
    if let Some(v) = name {
        sql.push_str(", name = ?");
        params.push(Box::new(v.to_string()));
    }
    if let Some(v) = role {
        sql.push_str(", role = ?");
        params.push(Box::new(v.to_string()));
    }
    if let Some(v) = language {
        sql.push_str(", language = ?");
        params.push(Box::new(v.to_string()));
    }

    sql.push_str(" WHERE id = ?");
    params.push(Box::new(id.to_string()));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    conn.execute(&sql, param_refs.as_slice())
        .map_err(|e| match &e {
            rusqlite::Error::SqliteFailure(err, _)
                if err.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                AppError::new("CONFLICT", "A user with this email already exists")
            }
            _ => AppError::from(e),
        })?;
    find_user_by_id(conn, id)
}

pub fn set_user_active(conn: &Connection, id: &str, is_active: bool) -> Result<User, AppError> {
    let rows = conn.execute(
        "UPDATE users SET is_active = ?1, updated_at = datetime('now') WHERE id = ?2",
        rusqlite::params![if is_active { 1i32 } else { 0i32 }, id],
    )?;
    if rows == 0 {
        return Err(AppError::new("NOT_FOUND", "User not found"));
    }
    find_user_by_id(conn, id)
}

pub fn count_active_admins(conn: &Connection) -> Result<i64, AppError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM users WHERE role = 'ADMIN' AND is_active = 1",
        [],
        |row| row.get(0),
    )?;
    Ok(count)
}

pub fn revoke_all_refresh_tokens_for_user(conn: &Connection, user_id: &str) -> Result<(), AppError> {
    conn.execute(
        "UPDATE refresh_tokens SET is_revoked = 1 WHERE user_id = ?1",
        [user_id],
    )?;
    Ok(())
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
