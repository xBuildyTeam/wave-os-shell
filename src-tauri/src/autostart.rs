#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

/// Enable auto-start with the given tier
/// tier 1 = Registry Run key (user, no admin)
/// tier 2 = Task Scheduler (system, needs admin)
#[tauri::command]
pub fn enable_autostart(exe_path: String, tier: u32) -> Result<(), String> {
    match tier {
        1 => enable_autostart_user(exe_path),
        2 => enable_autostart_system(exe_path),
        _ => Err("Invalid tier. Use 1 (user) or 2 (system)".to_string()),
    }
}

/// Disable auto-start
#[tauri::command]
pub fn disable_autostart() -> Result<(), String> {
    #[cfg(windows)]
    {
        // Remove Run key
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(run_key) = hkcu.open_subkey_with_flags(
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run",
            KEY_SET_VALUE,
        ) {
            let _ = run_key.delete_value("WaveOS");
        }

        // Remove scheduled task
        let _ = std::process::Command::new("schtasks")
            .args(["/delete", "/tn", "WaveOS", "/f"])
            .output();
    }

    Ok(())
}

/// Check if auto-start is enabled
#[tauri::command]
pub fn get_autostart_status() -> bool {
    #[cfg(windows)]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(run_key) = hkcu.open_subkey(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run") {
            if run_key.get_value::<String, _>("WaveOS").is_ok() {
                return true;
            }
        }
        false
    }
    #[cfg(not(windows))]
    {
        false
    }
}

#[cfg(windows)]
fn enable_autostart_user(exe_path: String) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu
        .open_subkey_with_flags(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run", KEY_SET_VALUE)
        .map_err(|e| format!("Failed to open Run key: {}", e))?;

    run_key
        .set_value("WaveOS", &exe_path)
        .map_err(|e| format!("Failed to set Run value: {}", e))?;

    log::info!("Auto-start enabled (user tier)");
    Ok(())
}

#[cfg(windows)]
fn enable_autostart_system(exe_path: String) -> Result<(), String> {
    let output = std::process::Command::new("schtasks")
        .args([
            "/create", "/tn", "WaveOS", "/tr", &exe_path,
            "/sc", "onlogon", "/rl", "highest", "/f",
        ])
        .output()
        .map_err(|e| format!("Failed to create scheduled task: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "schtasks failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    log::info!("Auto-start enabled (system tier)");
    Ok(())
}

#[cfg(not(windows))]
fn enable_autostart_user(_exe_path: String) -> Result<(), String> {
    Err("Auto-start is only available on Windows".to_string())
}

#[cfg(not(windows))]
fn enable_autostart_system(_exe_path: String) -> Result<(), String> {
    Err("Auto-start is only available on Windows".to_string())
}
