use serde::Serialize;
use tauri::App;

#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

#[derive(Serialize, Clone)]
pub enum ShellMode {
    App,
    Shell,
    Kiosk,
}

impl std::fmt::Display for ShellMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShellMode::App => write!(f, "app"),
            ShellMode::Shell => write!(f, "shell"),
            ShellMode::Kiosk => write!(f, "kiosk"),
        }
    }
}

#[cfg(windows)]
const SHELL_KEY: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon";
#[cfg(windows)]
const SHELL_VALUE: &str = "Shell";

/// Get the current shell mode from the registry
#[tauri::command]
pub fn get_shell_mode() -> String {
    #[cfg(windows)]
    {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        if let Ok(subkey) = hklm.open_subkey(SHELL_KEY) {
            if let Ok(shell) = subkey.get_value::<String, _>(SHELL_VALUE) {
                if shell.contains("wave-os-shell") {
                    if shell.contains("--kiosk") {
                        return "kiosk".to_string();
                    }
                    return "shell".to_string();
                }
            }
        }
    }
    "app".to_string()
}

/// Check if running as shell
#[tauri::command]
pub fn is_shell_mode() -> bool {
    get_shell_mode() != "app"
}

/// Enable shell replacement mode
#[tauri::command]
pub fn enable_shell_mode(exe_path: String) -> Result<(), String> {
    #[cfg(windows)]
    {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let subkey = hklm
            .open_subkey_with_flags(SHELL_KEY, KEY_SET_VALUE)
            .map_err(|e| format!("Failed to open registry: {}. Run as administrator.", e))?;

        let current_shell = subkey
            .get_value::<String, _>(SHELL_VALUE)
            .unwrap_or_else(|_| "explorer.exe".to_string());

        // Log current value for rollback
        log::info!("Backing up current shell: {}", current_shell);

        let new_value = format!("\"{}\" --shell", exe_path);
        subkey
            .set_value(SHELL_VALUE, &new_value)
            .map_err(|e| format!("Failed to set registry value: {}", e))?;

        log::info!("Shell mode enabled. Changes take effect on next login.");
        Ok(())
    }

    #[cfg(not(windows))]
    Err("Shell mode is only available on Windows".to_string())
}

/// Disable shell replacement mode — restore explorer.exe
#[tauri::command]
pub fn disable_shell_mode() -> Result<(), String> {
    #[cfg(windows)]
    {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let subkey = hklm
            .open_subkey_with_flags(SHELL_KEY, KEY_SET_VALUE)
            .map_err(|e| format!("Failed to open registry: {}. Run as administrator.", e))?;

        subkey
            .set_value(SHELL_VALUE, &"explorer.exe".to_string())
            .map_err(|e| format!("Failed to restore explorer.exe: {}", e))?;

        log::info!("Shell mode disabled. Explorer will be restored on next login.");
        Ok(())
    }

    #[cfg(not(windows))]
    Err("Shell mode is only available on Windows".to_string())
}

/// Temporarily launch explorer.exe
#[tauri::command]
pub fn launch_explorer() -> Result<(), String> {
    std::process::Command::new("explorer.exe")
        .spawn()
        .map_err(|e| format!("Failed to launch explorer: {}", e))?;
    Ok(())
}

/// Kill explorer.exe (return to Wave OS only)
#[tauri::command]
pub fn kill_explorer() -> Result<(), String> {
    std::process::Command::new("taskkill")
        .args(["/f", "/im", "explorer.exe"])
        .spawn()
        .map_err(|e| format!("Failed to kill explorer: {}", e))?;
    Ok(())
}
