use tauri::{App, Manager};

pub fn register_hotkeys(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    // Ctrl+Alt+W — Toggle Wave OS / Explorer
    app.global_shortcut().on_shortcut("ctrl+alt+w", move |app, _shortcut, _event| {
        if let Some(window) = app.get_webview_window("wave-os") {
            if window.is_visible().unwrap_or(false) {
                let _ = window.hide();
                let _ = std::process::Command::new("explorer.exe").spawn();
            } else {
                let _ = window.show();
                let _ = window.set_focus();
                let _ = std::process::Command::new("taskkill")
                    .args(["/f", "/im", "explorer.exe"])
                    .spawn();
            }
        }
    })?;

    // Ctrl+Alt+S — Open Settings
    app.global_shortcut().on_shortcut("ctrl+alt+s", move |app, _shortcut, _event| {
        if let Some(window) = app.get_webview_window("wave-os") {
            let _ = window.show();
            let _ = window.set_focus();
            let _ = window.eval("window.location.hash = '#/settings';");
        }
    })?;

    log::info!("Global hotkeys registered: Ctrl+Alt+W, Ctrl+Alt+S");
    Ok(())
}
