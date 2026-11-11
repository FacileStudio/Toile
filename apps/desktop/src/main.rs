#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod login;
mod token;
mod update;

use tauri::Manager;

/// Starts the Toile window with the keychain token, loopback sign-in, the system browser and,
/// when the build carries an updater key, self-update.
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            token::token_get,
            token::token_set,
            token::token_clear,
            login::login_listen
        ])
        .setup(|app| {
            update::install(app.handle());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Toile failed to start");
}
