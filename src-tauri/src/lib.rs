mod db;
mod error;
mod models;
mod commands;
mod auth;
mod audit;
mod notifications_engine;
mod pdf;
mod seed;

use db::AppState;
use tauri::Manager;

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
            app.manage(AppState {
                db: std::sync::Mutex::new(conn),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::greet,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
