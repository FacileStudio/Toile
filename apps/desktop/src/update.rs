use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
use tauri_plugin_updater::UpdaterExt;

/// True when the build carries an updater block (a public key and the release endpoint). The
/// release workflow adds it only when the signing key exists, so an unsigned build never updates.
pub fn configured(app: &AppHandle) -> bool {
    app.config().plugins.0.contains_key("updater")
}

/// Registers the updater and checks GitHub's latest release once, in release builds that carry
/// an updater key. A newer version asks before it installs, then restarts Toile.
pub fn install(app: &AppHandle) {
    if cfg!(debug_assertions) || !configured(app) {
        return;
    }
    if let Err(err) = app.plugin(tauri_plugin_updater::Builder::new().build()) {
        eprintln!("toile: updater unavailable: {err}");
        return;
    }
    let app = app.clone();
    tauri::async_runtime::spawn(async move { check(&app).await });
}

fn ask(app: &AppHandle, version: &str) -> bool {
    app.dialog()
        .message(format!(
            "Toile {version} est disponible. L'installer maintenant ?"
        ))
        .title("Mise à jour")
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Installer".into(),
            "Plus tard".into(),
        ))
        .blocking_show()
}

async fn check(app: &AppHandle) {
    let update = match app.updater() {
        Ok(updater) => updater.check().await,
        Err(err) => Err(err),
    };
    let update = match update {
        Ok(Some(update)) => update,
        Ok(None) => return,
        Err(err) => return eprintln!("toile: update check failed: {err}"),
    };
    if !ask(app, &update.version) {
        return;
    }
    match update.download_and_install(|_, _| {}, || {}).await {
        Ok(()) => app.restart(),
        Err(err) => {
            eprintln!("toile: the update failed: {err}");
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
            }
        }
    }
}
