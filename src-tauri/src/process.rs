use std::process::Command;

/// Spawn a native process
#[tauri::command]
pub fn spawn_process(
    executable: String,
    args: Vec<String>,
    working_dir: Option<String>,
) -> Result<u32, String> {
    let mut cmd = Command::new(&executable);
    cmd.args(&args);
    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }
    let child = cmd.spawn().map_err(|e| format!("Failed to spawn process: {}", e))?;
    Ok(child.id())
}

/// Kill a process by PID
#[tauri::command]
pub fn kill_process(pid: u32) -> Result<(), String> {
    Command::new("taskkill")
        .args(["/f", "/pid", &pid.to_string()])
        .output()
        .map_err(|e| format!("Failed to kill process: {}", e))?;
    Ok(())
}

/// Check if a process is running by name
#[tauri::command]
pub fn is_process_running(name: String) -> bool {
    let output = Command::new("tasklist")
        .args(["/fi", &format!("imagename eq {}", name), "/fo", "csv", "/nh"])
        .output();
    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout.contains(&name)
        }
        Err(_) => false,
    }
}

/// Launch explorer.exe
#[tauri::command]
pub fn launch_explorer() -> Result<(), String> {
    Command::new("explorer.exe")
        .spawn()
        .map_err(|e| format!("Failed to launch explorer: {}", e))?;
    Ok(())
}

/// Launch Task Manager
#[tauri::command]
pub fn launch_task_manager() -> Result<(), String> {
    Command::new("taskmgr.exe")
        .spawn()
        .map_err(|e| format!("Failed to launch Task Manager: {}", e))?;
    Ok(())
}
