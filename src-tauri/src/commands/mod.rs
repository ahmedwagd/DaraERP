pub mod auth;
pub mod users;

use crate::error::AppError;

#[tauri::command]
pub fn greet(name: &str) -> Result<String, AppError> {
    if name.trim().is_empty() {
        return Err(AppError::new(
            "VALIDATION_ERROR",
            "Name cannot be empty",
        ));
    }
    Ok(format!("Hello, {name}! Welcome to DaraERP."))
}
