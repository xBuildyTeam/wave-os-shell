use tauri_plugin_dialog::DialogExt;
use serde::Serialize;

#[derive(Serialize)]
pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

#[tauri::command]
pub async fn pick_files(app: tauri::AppHandle, filters: Option<Vec<FileFilter>>) -> Vec<String> {
    let mut builder = app.dialog().file();
    if let Some(ref filters) = filters {
        for f in filters {
            let ext_refs: Vec<&str> = f.extensions.iter().map(|s| s.as_str()).collect();
            builder = builder.add_filter(f.name.clone(), &ext_refs);
        }
    }
    builder.blocking_pick_files().unwrap_or_default()
        .into_iter().map(|p| p.to_string()).collect()
}

#[tauri::command]
pub async fn pick_folder(app: tauri::AppHandle) -> Option<String> {
    app.dialog().file().blocking_pick_folder().map(|p| p.to_string())
}

#[tauri::command]
pub async fn save_file(path: String, data: Vec<u8>) -> Result<(), String> {
    std::fs::write(&path, &data).map_err(|e| format!("Failed to save file: {}", e))
}

#[tauri::command]
pub async fn read_file(path: String) -> Result<Vec<u8>, String> {
    std::fs::read(&path).map_err(|e| format!("Failed to read file: {}", e))
}

#[tauri::command]
pub fn open_file(path: String) -> Result<(), String> {
    #[cfg(windows)]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", "", &path])
            .spawn().map_err(|e| format!("Failed to open file: {}", e))?;
    }
    #[cfg(not(windows))]
    {
        std::process::Command::new("xdg-open").arg(&path)
            .spawn().map_err(|e| format!("Failed to open file: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub fn show_in_explorer(path: String) -> Result<(), String> {
    #[cfg(windows)]
    {
        std::process::Command::new("explorer.exe")
            .args(["/select,", &path])
            .spawn().map_err(|e| format!("Failed to show in Explorer: {}", e))?;
    }
    Ok(())
}
