pub mod secure_store;
pub mod session;

use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::RngCore;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::audit;
use crate::error::AppError;
use crate::models;

// ---------------------------------------------------------------------------
// JWT claims
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub lang: String,
    pub exp: usize,
}

// ---------------------------------------------------------------------------
// Password hashing
// ---------------------------------------------------------------------------

/// Hashes a plaintext password using Argon2id.
#[allow(dead_code)]
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut rand::rngs::OsRng);
    let hash = argon2::Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::new("INTERNAL_ERROR", format!("hashing failed: {e}")))?
        .to_string();
    Ok(hash)
}

/// Verifies a plaintext password against an Argon2id hash.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::new("INTERNAL_ERROR", format!("invalid hash format: {e}")))?;
    Ok(argon2::Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

// ---------------------------------------------------------------------------
// Token issuance
// ---------------------------------------------------------------------------

/// Issues a JWT access token and an opaque refresh token.
///
/// Returns `(jwt, refresh_token_raw, family_id, refresh_expiry)`.
pub fn issue_tokens(
    user_id: &str,
    role: &str,
    language: &str,
    jwt_secret: &str,
) -> Result<(String, String, String, chrono::DateTime<Utc>), AppError> {
    let now = Utc::now();
    let exp = (now + Duration::minutes(15)).timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        lang: language.to_string(),
        exp,
    };

    let jwt = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::new("INTERNAL_ERROR", format!("JWT encoding failed: {e}")))?;

    let family_id = uuid::Uuid::new_v4().to_string();

    let mut refresh_bytes = vec![0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut refresh_bytes);
    let refresh_token_raw = base64_url(&refresh_bytes);

    let refresh_expiry = now + Duration::days(30);

    Ok((jwt, refresh_token_raw, family_id, refresh_expiry))
}

// ---------------------------------------------------------------------------
// JWT verification
// ---------------------------------------------------------------------------

/// Validates a JWT and returns its claims.
#[allow(dead_code)]
pub fn verify_jwt(token: &str, jwt_secret: &str) -> Result<Claims, AppError> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
            AppError::new("AUTH_EXPIRED_TOKEN", "Session expired")
        }
        _ => AppError::new("AUTH_INVALID_CREDENTIALS", "Invalid token"),
    })?;

    Ok(token_data.claims)
}

// ---------------------------------------------------------------------------
// Refresh token rotation & theft detection
// ---------------------------------------------------------------------------

/// Rotates a refresh token.
///
/// 1. Hashes the presented raw token (SHA-256) and looks it up.
/// 2. Not found → `AUTH_INVALID_CREDENTIALS`.
/// 3. Found + expired → `AUTH_EXPIRED_TOKEN`.
/// 4. Found + already revoked → **theft detected**: revoke whole family, write audit,
///    return `AUTH_INVALID_CREDENTIALS`. Keychain cleanup is handled by the caller.
/// 5. Found + active + not expired → revoke old token, issue new token in same family.
///
/// All write operations run inside a single transaction.
///
/// Returns `(new_jwt, new_refresh_raw, family_id, user_id, new_refresh_expiry)`.
pub fn rotate_refresh_token(
    conn: &mut Connection,
    raw_token: &str,
    jwt_secret: &str,
) -> Result<(String, String, String, String, chrono::DateTime<Utc>), AppError> {
    let hash = sha256_hex(raw_token);

    let tx = conn.transaction()?;

    let stored = models::find_refresh_token_by_hash(&tx, &hash)?;

    let now = Utc::now();
    let expires_at = chrono::NaiveDateTime::parse_from_str(
        &stored.expires_at,
        "%Y-%m-%dT%H:%M:%S",
    )
    .map_err(|_| AppError::new("INTERNAL_ERROR", "Invalid expiry format in DB"))?
        .and_local_timezone(Utc)
        .single()
        .ok_or_else(|| AppError::new("INTERNAL_ERROR", "Invalid expiry time"))?;

    if now > expires_at {
        return Err(AppError::new(
            "AUTH_EXPIRED_TOKEN",
            "Refresh token has expired",
        ));
    }

    if stored.is_revoked {
        models::revoke_family(&tx, &stored.family_id)?;

        let user = models::find_user_by_id(&tx, &stored.user_id)?;
        audit::audit_log(
            &tx,
            &user.id,
            &user.email,
            audit::actions::TOKEN_THEFT_DETECTED,
            "auth",
            &user.id,
            Some(&format!(
                "{{ \"family_id\": \"{}\", \"token_id\": \"{}\" }}",
                stored.family_id, stored.id
            )),
        )?;

        tx.commit()?;
        return Err(AppError::new(
            "AUTH_INVALID_CREDENTIALS",
            "Token theft detected — session revoked",
        ));
    }

    models::revoke_refresh_token(&tx, &stored.id)?;

    let user = models::find_user_by_id(&tx, &stored.user_id)?;

    let (new_jwt, new_refresh_raw, _family_id, new_expiry) =
        issue_tokens(&user.id, &user.role, &user.language, jwt_secret)?;

    let new_hash = sha256_hex(&new_refresh_raw);
    let new_id = uuid::Uuid::new_v4().to_string();
    models::insert_refresh_token(
        &tx,
        &new_id,
        &user.id,
        &stored.family_id,
        &new_hash,
        &new_expiry.format("%Y-%m-%dT%H:%M:%S").to_string(),
    )?;

    tx.commit()?;

    Ok((new_jwt, new_refresh_raw, stored.family_id, user.id, new_expiry))
}

/// Revokes every token in a family.
pub fn revoke_family_by_user(conn: &Connection, family_id: &str) -> Result<(), AppError> {
    models::revoke_family(conn, family_id)
}

// ---------------------------------------------------------------------------
// Role guard
// ---------------------------------------------------------------------------

/// Returns `PERMISSION_DENIED` if `user_role` is not in `allowed_roles`.
#[allow(dead_code)]
pub fn guard_role(user_role: &str, allowed_roles: &[&str]) -> Result<(), AppError> {
    if allowed_roles.contains(&user_role) {
        Ok(())
    } else {
        Err(AppError::new(
            "PERMISSION_DENIED",
            "You do not have permission to perform this action",
        ))
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

pub(crate) fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

fn base64_url(bytes: &[u8]) -> String {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    URL_SAFE_NO_PAD.encode(bytes)
}
