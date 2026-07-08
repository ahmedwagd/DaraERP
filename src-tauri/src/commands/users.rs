use serde::Serialize;
use tauri::State;

use crate::audit;
use crate::auth;
use crate::error::AppError;
use crate::models;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub role: String,
    pub language: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

fn to_response(user: models::User) -> UserResponse {
    UserResponse {
        id: user.id,
        email: user.email,
        name: user.name,
        role: user.role,
        language: user.language,
        is_active: user.is_active,
        created_at: user.created_at,
        updated_at: user.updated_at,
    }
}

/// Checks session exists and role is ADMIN. Returns acting admin user_id.
/// Session lock is released before returning.
fn require_admin(state: &State<AppState>) -> Result<String, AppError> {
    let sess = state.session.lock().map_err(|e| {
        AppError::new("INTERNAL_ERROR", format!("Lock error: {e}"))
    })?;
    let session = sess
        .as_ref()
        .ok_or_else(|| AppError::new("UNAUTHORIZED", "No active session"))?;
    let user_id = session.user_id.clone();
    auth::guard_role(&session.role, &["ADMIN"])?;
    drop(sess);
    Ok(user_id)
}

// ---------------------------------------------------------------------------
// create_user
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn create_user(
    email: String,
    name: String,
    password: String,
    role: String,
    language: String,
    state: State<AppState>,
) -> Result<UserResponse, AppError> {
    let acting_user_id = require_admin(&state)?;

    let email = email.trim().to_lowercase();
    let name = name.trim().to_string();

    if !email.contains('@') || !email.contains('.') {
        return Err(AppError::new("VALIDATION_ERROR", "Invalid email format"));
    }
    if name.is_empty() {
        return Err(AppError::new("VALIDATION_ERROR", "Name cannot be empty"));
    }
    if password.is_empty() {
        return Err(AppError::new("VALIDATION_ERROR", "Password cannot be empty"));
    }
    if !["ADMIN", "MANAGER", "USER"].contains(&role.as_str()) {
        return Err(AppError::new(
            "VALIDATION_ERROR",
            "Role must be one of: ADMIN, MANAGER, USER",
        ));
    }
    if !["en", "ar"].contains(&language.as_str()) {
        return Err(AppError::new(
            "VALIDATION_ERROR",
            "Language must be one of: en, ar",
        ));
    }

    let password_hash = auth::hash_password(&password)?;
    let id = uuid::Uuid::new_v4().to_string();

    let mut db = state.db.lock().map_err(|e| {
        AppError::new("INTERNAL_ERROR", format!("Lock error: {e}"))
    })?;

    let tx = db.transaction()?;

    let acting_user = models::find_user_by_id(&tx, &acting_user_id)?;
    let new_user = models::insert_user(&tx, &id, &email, &name, &password_hash, &role, &language)?;

    audit::audit_log(
        &tx,
        &acting_user.id,
        &acting_user.email,
        audit::actions::USER_CREATED,
        "user",
        &new_user.id,
        Some(&format!("{{ \"email\": \"{email}\", \"role\": \"{role}\" }}")),
    )?;

    tx.commit()?;

    Ok(to_response(new_user))
}

// ---------------------------------------------------------------------------
// list_users
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn list_users(
    role: Option<String>,
    is_active: Option<bool>,
    state: State<AppState>,
) -> Result<Vec<UserResponse>, AppError> {
    let _acting_user_id = require_admin(&state)?;

    if let Some(ref r) = role {
        if !["ADMIN", "MANAGER", "USER"].contains(&r.as_str()) {
            return Err(AppError::new(
                "VALIDATION_ERROR",
                "Invalid role filter; must be one of: ADMIN, MANAGER, USER",
            ));
        }
    }

    let db = state.db.lock().map_err(|e| {
        AppError::new("INTERNAL_ERROR", format!("Lock error: {e}"))
    })?;

    let users = models::list_users(&db, role.as_deref(), is_active)?;
    Ok(users.into_iter().map(to_response).collect())
}

// ---------------------------------------------------------------------------
// get_user
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_user(
    id: String,
    state: State<AppState>,
) -> Result<UserResponse, AppError> {
    let _acting_user_id = require_admin(&state)?;

    let db = state.db.lock().map_err(|e| {
        AppError::new("INTERNAL_ERROR", format!("Lock error: {e}"))
    })?;

    let user = models::find_user_by_id(&db, &id)?;
    Ok(to_response(user))
}

// ---------------------------------------------------------------------------
// update_user
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn update_user(
    id: String,
    email: Option<String>,
    name: Option<String>,
    role: Option<String>,
    language: Option<String>,
    state: State<AppState>,
) -> Result<UserResponse, AppError> {
    let acting_user_id = require_admin(&state)?;

    if email.is_none() && name.is_none() && role.is_none() && language.is_none() {
        return Err(AppError::new(
            "VALIDATION_ERROR",
            "At least one field must be provided",
        ));
    }

    if let Some(ref v) = email {
        if !v.contains('@') || !v.contains('.') {
            return Err(AppError::new("VALIDATION_ERROR", "Invalid email format"));
        }
    }
    if let Some(ref v) = name {
        if v.trim().is_empty() {
            return Err(AppError::new("VALIDATION_ERROR", "Name cannot be empty"));
        }
    }
    if let Some(ref v) = role {
        if !["ADMIN", "MANAGER", "USER"].contains(&v.as_str()) {
            return Err(AppError::new(
                "VALIDATION_ERROR",
                "Role must be one of: ADMIN, MANAGER, USER",
            ));
        }
    }
    if let Some(ref v) = language {
        if !["en", "ar"].contains(&v.as_str()) {
            return Err(AppError::new(
                "VALIDATION_ERROR",
                "Language must be one of: en, ar",
            ));
        }
    }

    let mut db = state.db.lock().map_err(|e| {
        AppError::new("INTERNAL_ERROR", format!("Lock error: {e}"))
    })?;

    let target = models::find_user_by_id(&db, &id)?;

    if let Some(ref new_role) = role {
        if target.role == "ADMIN" && target.is_active && new_role != "ADMIN" {
            let active_admins = models::count_active_admins(&db)?;
            if active_admins <= 1 {
                return Err(AppError::new(
                    "BAD_REQUEST",
                    "Cannot demote the last active ADMIN",
                ));
            }
        }
    }

    let tx = db.transaction()?;

    let acting_user = models::find_user_by_id(&tx, &acting_user_id)?;
    let updated = models::update_user(
        &tx,
        &id,
        email.as_deref(),
        name.as_deref().map(|s| s.trim()),
        role.as_deref(),
        language.as_deref(),
    )?;

    let changes = {
        let mut parts: Vec<String> = Vec::new();
        if email.is_some() {
            parts.push("\"email\": \"<updated>\"".to_string());
        }
        if name.is_some() {
            parts.push("\"name\": \"<updated>\"".to_string());
        }
        if let Some(ref v) = role {
            parts.push(format!("\"role\": \"{v}\""));
        }
        if let Some(ref v) = language {
            parts.push(format!("\"language\": \"{v}\""));
        }
        format!("{{ {} }}", parts.join(", "))
    };

    audit::audit_log(
        &tx,
        &acting_user.id,
        &acting_user.email,
        audit::actions::USER_UPDATED,
        "user",
        &id,
        Some(&changes),
    )?;

    tx.commit()?;

    Ok(to_response(updated))
}

// ---------------------------------------------------------------------------
// set_user_active
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn set_user_active(
    id: String,
    is_active: bool,
    state: State<AppState>,
) -> Result<UserResponse, AppError> {
    let acting_user_id = require_admin(&state)?;

    if id == acting_user_id && !is_active {
        return Err(AppError::new(
            "BAD_REQUEST",
            "Admin cannot deactivate themselves",
        ));
    }

    let mut db = state.db.lock().map_err(|e| {
        AppError::new("INTERNAL_ERROR", format!("Lock error: {e}"))
    })?;

    let target = models::find_user_by_id(&db, &id)?;

    if !is_active && target.role == "ADMIN" && target.is_active {
        let active_admins = models::count_active_admins(&db)?;
        if active_admins <= 1 {
            return Err(AppError::new(
                "BAD_REQUEST",
                "Cannot deactivate the last active ADMIN",
            ));
        }
    }

    let tx = db.transaction()?;

    let acting_user = models::find_user_by_id(&tx, &acting_user_id)?;
    let updated = models::set_user_active(&tx, &id, is_active)?;

    if !is_active {
        models::revoke_all_refresh_tokens_for_user(&tx, &id)?;
        audit::audit_log(
            &tx,
            &acting_user.id,
            &acting_user.email,
            audit::actions::USER_DEACTIVATED,
            "user",
            &id,
            Some(&format!(
                "{{ \"target_id\": \"{id}\", \"target_role\": \"{}\" }}",
                target.role
            )),
        )?;
    } else {
        audit::audit_log(
            &tx,
            &acting_user.id,
            &acting_user.email,
            audit::actions::USER_ACTIVATED,
            "user",
            &id,
            Some(&format!(
                "{{ \"target_id\": \"{id}\", \"target_role\": \"{}\" }}",
                target.role
            )),
        )?;
    }

    tx.commit()?;

    Ok(to_response(updated))
}
