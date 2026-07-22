use serde::Serialize;

#[derive(Serialize)]
pub struct OllamaStatus {
    pub running: bool,
    pub models: Vec<String>,
    pub version: String,
}

/// Check if Ollama is running on localhost:11434
#[tauri::command]
pub async fn check_ollama() -> OllamaStatus {
    let url = "http://localhost:11434/api/tags";

    match reqwest::Client::new()
        .get(url)
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            let body: serde_json::Value = match resp.json().await {
                Ok(v) => v,
                Err(_) => return OllamaStatus {
                    running: true,
                    models: vec![],
                    version: String::new(),
                },
            };

            let models: Vec<String> = body["models"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|m| m["name"].as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            let version = body["version"]
                .as_str()
                .unwrap_or("")
                .to_string();

            OllamaStatus {
                running: true,
                models,
                version,
            }
        }
        _ => OllamaStatus {
            running: false,
            models: vec![],
            version: String::new(),
        },
    }
}

/// Start Ollama if installed but not running
#[tauri::command]
pub fn start_ollama() -> Result<(), String> {
    let paths = [
        "ollama",
        "C:\\Program Files\\Ollama\\ollama.exe",
        "C:\\Users\\Default\\AppData\\Local\\Programs\\Ollama\\ollama.exe",
    ];

    for path in &paths {
        if std::process::Command::new(path)
            .arg("serve")
            .spawn()
            .is_ok()
        {
            log::info!("Ollama started via {}", path);
            return Ok(());
        }
    }

    Err("Ollama not found. Install from ollama.com".to_string())
}
