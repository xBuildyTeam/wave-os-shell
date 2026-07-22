use tauri::{App, AppHandle, Manager};

/// Minimize to system tray
#[tauri::command]
pub fn minimize_to_tray(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("wave-os") {
        window
            .hide()
            .map_err(|e| format!("Failed to hide window: {}", e))?;
    }
    Ok(())
}

/// Toggle fullscreen
#[tauri::command]
pub fn toggle_fullscreen(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("wave-os") {
        let is_fullscreen = window.is_fullscreen().unwrap_or(false);
        window
            .set_fullscreen(!is_fullscreen)
            .map_err(|e| format!("Failed to toggle fullscreen: {}", e))?;
    }
    Ok(())
}

/// Get current window state
#[tauri::command]
pub fn get_window_state(app: AppHandle) -> String {
    if let Some(window) = app.get_webview_window("wave-os") {
        let is_fullscreen = window.is_fullscreen().unwrap_or(false);
        let is_maximized = window.is_maximized().unwrap_or(false);
        let is_visible = window.is_visible().unwrap_or(false);

        return serde_json::json!({
            "fullscreen": is_fullscreen,
            "maximized": is_maximized,
            "visible": is_visible,
        })
        .to_string();
    }
    "{}".to_string()
}

/// Set window to shell mode (true borderless fullscreen)
pub fn set_shell_mode(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(window) = app.get_webview_window("wave-os") {
        window.set_fullscreen(true)?;
        window.maximize()?;
        window.set_decorations(false)?;
        window.set_always_on_top(true)?;
        window.set_skip_taskbar(true)?;
    }
    Ok(())
}

/// Set window to kiosk mode (full lockdown)
pub fn set_kiosk_mode(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(window) = app.get_webview_window("wave-os") {
        window.set_fullscreen(true)?;
        window.maximize()?;
        window.set_decorations(false)?;
        window.set_always_on_top(true)?;
        window.set_skip_taskbar(true)?;
    }
    Ok(())
}