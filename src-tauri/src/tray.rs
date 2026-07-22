use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, CheckMenuItem},
    tray::TrayIconBuilder,
    App, Manager, WebviewWindow,
};

pub fn create_tray(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let open_item = MenuItem::with_id(app, "open", "Open Wave OS", true, None::<&str>)?;
    let windows_item = MenuItem::with_id(app, "windows", "Windows Mode", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let shell_app = CheckMenuItem::with_id(app, "shell_app", "App Mode (default)", true, true, None::<&str>)?;
    let shell_replace = CheckMenuItem::with_id(app, "shell_replace", "Shell Replacement", true, false, None::<&str>)?;
    let sep3 = PredefinedMenuItem::separator(app)?;
    let exit_item = MenuItem::with_id(app, "exit", "Exit to Windows", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[
        &open_item, &windows_item, &sep1, &settings_item, &sep2,
        &shell_app, &shell_replace, &sep3, &exit_item,
    ])?;

    let _tray = TrayIconBuilder::with_id("wave-tray")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("Wave OS")
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "open" => {
                    if let Some(window) = app.get_webview_window("wave-os") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "windows" => { let _ = std::process::Command::new("explorer.exe").spawn(); }
                "settings" => {
                    if let Some(window) = app.get_webview_window("wave-os") {
                        let _ = window.show();
                        let _ = window.set_focus();
                        let _ = window.eval("window.location.hash = '#/settings';");
                    }
                }
                "exit" => {
                    let shell_mode = crate::shell::get_shell_mode();
                    if shell_mode != "app" {
                        let _ = std::process::Command::new("explorer.exe").spawn();
                    }
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app)?;
    Ok(())
}
