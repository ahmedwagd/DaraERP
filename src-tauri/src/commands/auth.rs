// Role policy (for future use by non-auth commands):
//   ADMIN  → users, settings, full access
//   MANAGER → operational mutations (invoices, customers, products)
//   USER   → read-only
//
// Feature 001 (auth) commands are multi-role. No auth command requires ADMIN-only scope.
// guard_role() is available in crate::auth for use by future feature commands.

use serde::Serialize;
use tauri::State;

use crate::audit;
use crate::auth;
use crate::auth::secure_store;
use crate::auth::session::Session;
use crate::error::AppError;
use crate::models;
use crate::AppState;

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub name: Option<String>,
    pub email: String,
    pub role: String,
    pub language: String,
}

// ---------------------------------------------------------------------------
// Login
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn login(
    email: String,
    password: String,
    state: State<AppState>,
) -> Result<UserResponse, AppError> {
    if email.trim().is_empty() || password.trim().is_empty() {
        return Err(AppError::new(
            "VALIDATION_ERROR",
            "Email and password are required",
        ));
    }

    let db = state.db.lock().map_err(|e| {
        AppError::new("INTERNAL_ERROR", format!("Lock error: {e}"))
    })?;

    let user = match models::find_user_by_email(&db, email.trim()) {
        Ok(u) => u,
        Err(_) => {
            let _ = audit::audit_log(
                &db,
                "unknown",
                email.trim(),
                audit::actions::LOGIN_FAILED,
                "auth",
                "unknown",
                None,
            );
            return Err(AppError::new(
                "AUTH_INVALID_CREDENTIALS",
                "Invalid email or password",
            ));
        }
    };

    if !user.is_active {
        let _ = audit::audit_log(
            &db,
            &user.id,
            &user.email,
            audit::actions::LOGIN_FAILED,
            "auth",
            &user.id,
            None,
        );
        return Err(AppError::new(
            "AUTH_INVALID_CREDENTIALS",
            "Invalid email or password",
        ));
    }

    let valid = auth::verify_password(&password, &user.password_hash)?;
    if !valid {
        let _ = audit::audit_log(
            &db,
            &user.id,
            &user.email,
            audit::actions::LOGIN_FAILED,
            "auth",
            &user.id,
            None,
        );
        return Err(AppError::new(
            "AUTH_INVALID_CREDENTIALS",
            "Invalid email or password",
        ));
    }

    let (_jwt, refresh_token_raw, family_id, refresh_expiry) = auth::issue_tokens(
        &user.id,
        &user.role,
        &user.language,
        &state.jwt_secret,
    )?;

    let refresh_hash = auth::sha256_hex(&refresh_token_raw);

    let token_id = uuid::Uuid::new_v4().to_string();
    models::insert_refresh_token(
        &db,
        &token_id,
        &user.id,
        &family_id,
        &refresh_hash,
        &refresh_expiry.format("%Y-%m-%dT%H:%M:%S").to_string(),
    )?;

    secure_store::set_refresh_token(&refresh_token_raw)?;

    // Update in-memory session.
    let session = Session {
        user_id: user.id.clone(),
        role: user.role.clone(),
        language: user.language.clone(),
        family_id: family_id.clone(),
    };

    drop(db); // Release lock before mutating session.

    let mut sess = state.session.lock().map_err(|e| {
        AppError::new("INTERNAL_ERROR", format!("Lock error: {e}"))
    })?;
    *sess = Some(session);

    // Audit after session is set.
    let db2 = state.db.lock().map_err(|e| {
        AppError::new("INTERNAL_ERROR", format!("Lock error: {e}"))
    })?;
    let _ = audit::audit_log(
        &db2,
        &user.id,
        &user.email,
        audit::actions::LOGIN,
        "auth",
        &user.id,
        None,
    );

    Ok(UserResponse {
        id: user.id,
        name: user.name,
        email: user.email,
        role: user.role,
        language: user.language,
    })
}

// ---------------------------------------------------------------------------
// Refresh session
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn refresh_session(state: State<AppState>) -> Result<UserResponse, AppError> {
    let raw_token = secure_store::get_refresh_token()?;

    let mut db = state.db.lock().map_err(|e| {
        AppError::new("INTERNAL_ERROR", format!("Lock error: {e}"))
    })?;

    match auth::rotate_refresh_token(&mut db, &raw_token, &state.jwt_secret) {
        Ok((_new_jwt, new_refresh_raw, family_id, user_id, _new_expiry)) => {
            secure_store::set_refresh_token(&new_refresh_raw)?;

            let user = models::find_user_by_id(&db, &user_id)?;

            drop(db);

            let mut sess = state.session.lock().map_err(|e| {
                AppError::new("INTERNAL_ERROR", format!("Lock error: {e}"))
            })?;
            *sess = Some(Session {
                user_id: user.id.clone(),
                role: user.role.clone(),
                language: user.language.clone(),
                family_id,
            });

            Ok(UserResponse {
                id: user.id,
                name: user.name,
                email: user.email,
                role: user.role,
                language: user.language,
            })
        }
        Err(e) if e.code == "AUTH_INVALID_CREDENTIALS" || e.code == "AUTH_EXPIRED_TOKEN" => {
            drop(db);
            let _ = secure_store::clear_refresh_token();
            let mut sess = state.session.lock().map_err(|_| {
                AppError::new("INTERNAL_ERROR", "Session lock error")
            })?;
            *sess = None;
            Err(e)
        }
        Err(e) => Err(e),
    }
}

// ---------------------------------------------------------------------------
// Get current user
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_current_user(state: State<AppState>) -> Result<Option<UserResponse>, AppError> {
    let sess = state.session.lock().map_err(|e| {
        AppError::new("INTERNAL_ERROR", format!("Lock error: {e}"))
    })?;

    match sess.as_ref() {
        None => Ok(None),
        Some(session) => {
            let db = state.db.lock().map_err(|e| {
                AppError::new("INTERNAL_ERROR", format!("Lock error: {e}"))
            })?;

            match models::find_user_by_id(&db, &session.user_id) {
                Ok(user) => Ok(Some(UserResponse {
                    id: user.id,
                    name: user.name,
                    email: user.email,
                    role: user.role,
                    language: user.language,
                })),
                Err(_) => Ok(None),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Logout
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn logout(state: State<AppState>) -> Result<(), AppError> {
    // 1. Capture user identity and family_id from the current session.
    let (user_id, family_id) = {
        let sess = state.session.lock().map_err(|e| {
            AppError::new("INTERNAL_ERROR", format!("Lock error: {e}"))
        })?;
        match sess.as_ref() {
            None => return Ok(()),
            Some(s) => (s.user_id.clone(), Some(s.family_id.clone())),
        }
    };

    // 2. Clear in-memory session immediately so the UI can't show an active session.
    {
        let mut sess = state.session.lock().map_err(|e| {
            AppError::new("INTERNAL_ERROR", format!("Lock error: {e}"))
        })?;
        *sess = None;
    }

    // 3. Perform DB operations.
    let db = state.db.lock().map_err(|e| {
        AppError::new("INTERNAL_ERROR", format!("Lock error: {e}"))
    })?;

    let user_email = models::find_user_by_id(&db, &user_id)
        .map(|u| u.email)
        .unwrap_or_default();

    if let Some(ref fid) = family_id {
        let _ = auth::revoke_family_by_user(&db, fid);
    }

    let _ = audit::audit_log(
        &db,
        &user_id,
        &user_email,
        audit::actions::LOGOUT,
        "auth",
        &user_id,
        None,
    );

    drop(db);

    // 4. Clear keychain refresh token.
    let _ = secure_store::clear_refresh_token();

    Ok(())
}
