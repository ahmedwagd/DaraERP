mod commands;
mod db;
mod error;
mod models;
mod auth;
mod audit;
mod notifications_engine;
mod pdf;
mod seed;



use std::sync::Mutex;

use auth::session::Session;
use tauri::Manager;

use crate::auth::secure_store;

/// Shared application state managed by Tauri.
pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub session: Mutex<Option<Session>>,
    pub jwt_secret: String,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
/// Runs the `DaraERP` Tauri application.
///
/// # Panics
///
/// Panics if database initialization fails or if the Tauri runtime fails to start.
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let conn = db::init_db(app.handle())
                .expect("Failed to initialize database");
            let jwt_secret = secure_store::load_jwt_secret()
                .expect("Failed to load JWT secret");

            app.manage(AppState {
                db: Mutex::new(conn),
                session: Mutex::new(None),
                jwt_secret,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::auth::login,
            commands::auth::refresh_session,
            commands::auth::get_current_user,
            commands::auth::logout,
            commands::users::create_user,
            commands::users::list_users,
            commands::users::get_user,
            commands::users::update_user,
            commands::users::set_user_active,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
