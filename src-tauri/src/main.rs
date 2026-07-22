#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused)]

mod shell;
mod tray;
mod hotkeys;
mod process;
mod autostart;
mod ollama;
mod file_bridge;
mod window;

use tauri::Manager;

fn main() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            tray::create_tray(app)?;
            hotkeys::register_hotkeys(app)?;

            let args: Vec<String> = std::env::args().collect();
            if args.iter().any(|a| a == "--shell") {
                log::info!("Starting in Shell Mode");
                window::set_shell_mode(app)?;
            } else if args.iter().any(|a| a == "--kiosk") {
                log::info!("Starting in Kiosk Mode");
                window::set_kiosk_mode(app)?;
            } else {
                log::info!("Starting in App Mode");
            }

            if let Some(window) = app.get_webview_window("wave-os") {
                window.eval(include_str!("../../src/shell-bridge.js"))?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            shell::get_shell_mode,
            shell::enable_shell_mode,
            shell::disable_shell_mode,
            shell::launch_explorer,
            shell::kill_explorer,
            shell::is_shell_mode,
            process::spawn_process,
            process::kill_process,
            process::is_process_running,
            process::launch_task_manager,
            autostart::enable_autostart,
            autostart::disable_autostart,
            autostart::get_autostart_status,
            ollama::check_ollama,
            ollama::start_ollama,
            file_bridge::pick_files,
            file_bridge::pick_folder,
            file_bridge::save_file,
            file_bridge::read_file,
            file_bridge::open_file,
            file_bridge::show_in_explorer,
            window::minimize_to_tray,
            window::toggle_fullscreen,
            window::get_window_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Wave OS Shell");
}