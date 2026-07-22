use tauri_plugin_dialog::DialogExt;
use serde::Serialize;

#[derive(Serialize)]
pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

/// Open native file picker → return selected file paths
#[tauri::command]
pub async fn pick_files(filters: Option<Vec<FileFilter>>) -> Vec<String> {
    let mut builder = tauri::AppHandle::current().dialog().file();

    if let Some(ref filters) = filters {
        let tauri_filters: Vec<tauri_plugin_dialog::FileFilter> = filters
            .iter()
            .map(|f| tauri_plugin_dialog::FileFilter {
                name: f.name.clone(),
                extensions: f.extensions.clone(),
            })
            .collect();

        let mut filter_refs: Vec<&tauri_plugin_dialog::FileFilter> = Vec::new();
        for f in &tauri_filters {
            filter_refs.push(f);
        }
        builder = builder.add_filter("Files", &filter_refs.iter().map(|f| (f.name.as_str(), f.extensions.as_slice())).collect::<Vec<_>>());
    }

    builder
        .blocking_pick_files()
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.to_string())
        .collect()
}

/// Open native folder picker → return selected path
#[tauri::command]
pub async fn pick_folder() -> Option<String> {
    tauri::AppHandle::current()
        .dialog()
        .file()
        .blocking_pick_folder()
        .map(|p| p.to_string())
}

/// Save file to disk
#[tauri::command]
pub async fn save_file(path: String, data: Vec<u8>) -> Result<(), String> {
    std::fs::write(&path, &data).map_err(|e| format!("Failed to save file: {}", e))
}

/// Read file from disk → return base64
#[tauri::command]
pub async fn read_file(path: String) -> Result<Vec<u8>, String> {
    std::fs::read(&path).map_err(|e| format!("Failed to read file: {}", e))
}

/// Open file with default system handler
#[tauri::command]
pub fn open_file(path: String) -> Result<(), String> {
    #[cfg(windows)]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", "", &path])
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }
    #[cfg(not(windows))]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }
    Ok(())
}

/// Show file in Explorer/Finder
#[tauri::command]
pub fn show_in_explorer(path: String) -> Result<(), String> {
    #[cfg(windows)]
    {
        std::process::Command::new("explorer.exe")
            .args(["/select,", &path])
            .spawn()
            .map_err(|e| format!("Failed to show in Explorer: {}", e))?;
    }
    Ok(())
}
