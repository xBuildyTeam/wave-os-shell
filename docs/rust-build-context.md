# Rust & Tauri v2 Build Context — Wave OS Shell

This document provides comprehensive context for building and maintaining the
Wave OS Shell using Rust and Tauri v2. It supplements the API reference with
general Rust patterns, build system knowledge, and troubleshooting.

---

## 1. Rust Module System

### Module Declaration (main.rs)
```rust
mod shell;       // src/shell.rs
mod tray;        // src/tray.rs
mod hotkeys;     // src/hotkeys.rs
```

Every `.rs` file in `src-tauri/src/` must be declared in `main.rs` with `mod`.
If you add a new file, add `mod filename;` to main.rs or the build will fail.

### Cross-Module Calls
```rust
let shell_mode = crate::shell::get_shell_mode();
// Use `crate::` prefix to reference other modules
```

### Public vs Private
- `pub fn` — exported, can be called from other modules
- `fn` — private to the module
- `pub struct` — exported, can be used by other modules

---

## 2. Trait Imports (CRITICAL)

Rust traits must be imported to use their methods, even if the type is imported.
This is the #1 cause of build failures.

```rust
// WRONG — Manager trait not imported
use tauri::App;
app.get_webview_window("wave-os");  // E0599: method not found

// CORRECT — import both the type AND the trait
use tauri::{App, Manager};
app.get_webview_window("wave-os");  // works
```

### Traits Used in This Project
| Trait | Methods | Where Used |
|------|---------|------------|
| `tauri::Manager` | `get_webview_window()`, `default_window_icon()` | tray, hotkeys, window, main |
| `DialogExt` | `dialog()` | file_bridge |
| `GlobalShortcutExt` | `global_shortcut()` | hotkeys |

---

## 3. Closures and Borrowing

### Closure Arguments in Tauri v2
```rust
// on_shortcut: 3 args (app, shortcut, event)
app.global_shortcut().on_shortcut("ctrl+alt+w", move |app, _shortcut, _event| { ... })?;

// on_menu_event: 2 args (app, event)
.on_menu_event(|app, event| { ... })
```

The `app` in closures is `&AppHandle`, not `&App`. Both need `use tauri::Manager;`
to call `get_webview_window()`.

### `move` Keyword
Always use `move` for closures that capture external variables.

---

## 4. Error Handling

### Result Types
```rust
// Tauri commands return Result<T, String>
#[tauri::command]
pub fn my_command() -> Result<(), String> {
    something().map_err(|e| format!("Failed: {}", e))?;
    Ok(())
}

// Setup functions use Box<dyn std::error::Error>
pub fn my_setup(app: &App) -> Result<(), Box<dyn std::error::Error>> { ... }
```

---

## 5. #[cfg(windows)] and Cross-Platform

```rust
#[cfg(windows)]
use winreg::RegKey;

#[tauri::command]
pub fn my_command() -> Result<(), String> {
    #[cfg(windows)]
    {
        // Windows-specific code
        Ok(())
    }
    #[cfg(not(windows))]
    Err("Only available on Windows".to_string())
}
```

CI runs on `windows-latest`, so `#[cfg(windows)]` blocks are compiled.
Local dev on macOS/Linux needs `#[cfg(not(windows))]` fallbacks.

---

## 6. Cargo.toml Dependencies

```toml
tauri = { version = "2", features = ["tray-icon", "devtools"] }
tauri-plugin-global-shortcut = "2"

[target.'cfg(windows)'.dependencies]
winreg = "0.52"
windows = { version = "0.58", features = [...] }
```

All tauri-* crates must be version 2. Mixing v1 and v2 causes build failures.

---

## 7. serde for Command Parameters

```rust
#[derive(Serialize, Deserialize)]  // BOTH needed for command params
pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

#[derive(Serialize)]  // Only Serialize needed for return types
pub struct OllamaStatus {
    pub running: bool,
}
```

---

## 8. Common Build Errors

| Error | Meaning | Fix |
|-------|---------|-----|
| E0428 | Duplicate item | Remove #[tauri::command] from one of two same-named fns |
| E0599 | Method not found | Import the trait (e.g. `use tauri::Manager;`) |
| E0061 | Wrong arg count | Check Tauri v2 docs for exact arg counts |
| E0425 | Not found in scope | Use `Builder::new().build()` instead of `init()` |
| E0277 | Trait not impl | Check expected type |
| E0308 | Type mismatch | Add `.as_str()` or `&` as needed |

---

## 9. Checklist Before Pushing

- [ ] Every file with `get_webview_window()` has `use tauri::Manager;`
- [ ] No duplicate `#[tauri::command]` on same-named functions
- [ ] All icon files exist at referenced paths
- [ ] `index.html` exists in frontendDist directory
- [ ] `capabilities/default.json` has `remote.urls` for all domains
- [ ] CSP allows all required domains
- [ ] All tauri-* crates are version 2
- [ ] New modules declared in `main.rs` with `mod`
- [ ] Command params derive `Serialize + Deserialize`
- [ ] `#[cfg(windows)]` blocks have fallbacks
- [ ] CI uses `npx @tauri-apps/cli build`
- [ ] CI uses `swatinem/rust-cache@v2`