# Wave OS Desktop Shell — Tauri v2 Specification

**Version:** 1.0
**Date:** 2026-07-21
**Author:** xBuildy Ai
**Status:** Spec complete — ready for build

---

## Overview

A Tauri v2 desktop application that wraps the Wave OS web app (`app.oswave.io`) in a native Windows shell. The shell provides system-level access the browser can't — file system, process launching, system tray, global hotkeys, and optional shell replacement (replacing `explorer.exe` as the Windows shell).

Three modes of operation:
1. **App Mode** (default) — Wave icon on desktop, double-click opens Wave OS in a borderless fullscreen window. Explorer stays running underneath.
2. **Shell Mode** — Replaces `explorer.exe` as the Windows shell. On boot, Wave OS IS the desktop. No Start menu, no taskbar, just Wave OS.
3. **Kiosk Mode** — Shell mode + locked down. No Alt+Tab, no Ctrl+Alt+Del, no task manager. For dedicated Wave OS machines.

---

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Framework | Tauri v2 (Rust core + WebView2 frontend) |
| Frontend | Wave OS web app at `app.oswave.io` (loaded via WebView) |
| WebView | WebView2 (Chromium, pre-installed on Windows 10/11) |
| Shell Registry | Windows Registry (`HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\Shell`) |
| System Tray | `tray-icon` Tauri v2 plugin |
| Global Hotkeys | `global-shortcut` Tauri v2 plugin |
| Notifications | `notification` Tauri v2 plugin |
| File System | `fs` Tauri v2 plugin (scoped to user dirs) |
| Process Spawn | Rust `std::process::Command` |
| Auto-Start | Registry `Run` key + Task Scheduler fallback |
| Installer | MSI (WiX), same as uBase installer pattern |

---

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                 Wave OS Shell (Tauri v2)              │
│                                                       │
│  ┌──────────────────────────────────────────────────┐│
│  │              WebView2 (Chromium)                 ││
│  │                                                   ││
│  │    ┌─────────────────────────────────────┐        ││
│  │    │   Wave OS Web App (app.oswave.io)   │        ││
│  │    │   - Glassmorphism UI                │        ││
│  │    │   - All Wave OS apps (Surge, Story, │        ││
│  │    │     Files, Chat, Reef, etc.)       │        │
│  │    │   - Wave Assistant                 │        ││
│  │    └─────────────────────────────────────┘        ││
│  │                                                   ││
│  │    IPC Bridge (Tauri invoke API)                  ││
│  │    ↕ window.__TAURI__.invoke()                   ││
│  └──────────────────────────────────────────────────┘│
│                                                       │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────┐│
│  │ System Tray │  │ Shell Manager │  │ Process Mgr  ││
│  │ (tray-icon) │  │ (registry)   │  │ (Rust std)   ││
│  └─────────────┘  └──────────────┘  └──────────────┘│
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────┐│
│  │ Hotkey Mgr  │  │ Auto-Start    │  │ File Bridge  ││
│  │ (shortcut)  │  │ (registry)   │  │ (fs plugin) ││
│  └─────────────┘  └──────────────┘  └──────────────┘│
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────┐│
│  │ Ollama      │  │ Notif Center  │  │ Update Mgr   ││
│  │ Detector    │  │ (Win toast)   │  │ (updater)    ││
│  └─────────────┘  └──────────────┘  └──────────────┘│
└─────────────────────────────────────────────────────┘
```

---

## File Structure

```
wave-os-shell/
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── icons/
│   │   ├── icon.ico          # Wave logo icon
│   │   ├── icon-tray.ico     # System tray icon (16x16, 32x32)
│   │   └── tray-connected.ico
│   ├── capabilities/
│   │   └── default.json      # Tauri v2 permissions/capabilities
│   └── src/
│       ├── main.rs           # Entry point
│       ├── lib.rs            # Tauri app builder
│       ├── shell.rs          # Shell replacement logic (registry)
│       ├── tray.rs           # System tray menu + events
│       ├── hotkeys.rs        # Global hotkey registration
│       ├── process.rs        # Process spawning (Ollama, Harbor, Explorer)
│       ├── autostart.rs      # Auto-start registration
│       ├── ollama.rs         # Ollama detection (localhost:11434)
│       ├── file_bridge.rs    # File system bridge (drag-drop, save dialogs)
│       ├── kiosk.rs          # Kiosk mode lockdown
│       └── window.rs         # Window management (fullscreen, borderless)
├── src/
│   ├── shell-bridge.js       # Injected IPC bridge (window.__waveShell)
│   └── preload.js            # Pre-injected before Wave OS loads
├── installer/
│   ├── wave-os-shell.wxs     # WiX MSI definition
│   └── build-msi.ps1         # PowerShell build script
└── README.md
```

---

## Component Specs

### 1. Window Manager (`window.rs`)

Controls how the Tauri window appears on screen.

**App Mode (default):**
- Borderless window, maximized (not true fullscreen — preserves Alt+Tab)
- Size: 100% width, 100% height (with small margin for shadow)
- No title bar, no window controls (custom exit via tray or hotkey)
- Always on top toggle (optional)
- Minimize to tray on close (not exit)

**Shell Mode:**
- True borderless fullscreen
- No window controls at all
- Cannot be minimized (it IS the desktop)
- `Ctrl+Alt+W` → toggle Wave OS / Explorer (temporarily relaunches explorer.exe)

**Kiosk Mode:**
- True borderless fullscreen
- Disable Alt+Tab (via Windows API `SetWindowsHookEx`)
- Disable Ctrl+Alt+Del (via registry `DisableTaskMgr`)
- Disable Win key (via low-level keyboard hook)
- Exit only via PIN code in Settings

```rust
// src/window.rs — key functions

pub fn create_app_window(app: &mut App) -> Result<()> {
    let window = WebviewWindowBuilder::new(app, "wave-os", WebviewUrl::App(WAVE_OS_URL.into()))
        .title("Wave OS")
        .fullscreen(false)
        .maximized(true)
        .decorations(false)
        .always_on_top(false)
        .skip_taskbar(false)
        .visible(true)
        .inner_size(1920.0, 1080.0)
        .build()?;
    
    Ok(())
}

pub fn create_shell_window(app: &mut App) -> Result<()> {
    let window = WebviewWindowBuilder::new(app, "wave-os", WebviewUrl::App(WAVE_OS_URL.into()))
        .title("Wave OS")
        .fullscreen(true)
        .maximized(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(true)
        .build()?;
    
    Ok(())
}
```

### 2. Shell Manager (`shell.rs`)

Manages replacing and restoring `explorer.exe` as the Windows shell.

```rust
// src/shell.rs

const SHELL_KEY: &str = "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Winlogon";
const SHELL_VALUE: &str = "Shell";

/// Check current shell mode
pub fn get_shell_mode() -> ShellMode {
    let shell = read_registry(HKEY_LOCAL_MACHINE, SHELL_KEY, SHELL_VALUE);
    match shell.as_str() {
        s if s.contains("wave-os-shell") => ShellMode::Shell,
        _ => ShellMode::App,
    }
}

/// Replace explorer.exe with Wave OS Shell
/// Requires admin privileges
pub fn enable_shell_mode(exe_path: &str) -> Result<()> {
    // 1. Write registry key
    write_registry(
        HKEY_LOCAL_MACHINE,
        SHELL_KEY,
        SHELL_VALUE,
        &format!("\"{}\" --shell", exe_path)
    )?;
    
    // 2. Kill explorer.exe (it will relaunch as Wave OS on next boot)
    // DON'T kill immediately — prompt user to reboot
    
    // 3. Log the change for rollback
    log_shell_change("explorer.exe", &format!("\"{}\" --shell", exe_path));
    
    Ok(())
}

/// Restore explorer.exe as shell
pub fn disable_shell_mode() -> Result<()> {
    write_registry(
        HKEY_LOCAL_MACHINE,
        SHELL_KEY,
        SHELL_VALUE,
        "explorer.exe"
    )?;
    
    // Prompt user to reboot
    Ok(())
}

/// Temporarily launch explorer.exe (for accessing Windows without reboot)
pub fn launch_explorer() -> Result<()> {
    Command::new("explorer.exe").spawn()?;
    Ok(())
}

/// Temporarily kill explorer.exe (return to Wave OS only)
pub fn kill_explorer() -> Result<()> {
    Command::new("taskkill")
        .args(["/f", "/im", "explorer.exe"])
        .spawn()?;
    Ok(())
}

pub enum ShellMode {
    App,       // Explorer is the shell, Wave OS is just an app
    Shell,     // Wave OS IS the shell
    Kiosk,     // Wave OS is shell + locked down
}
```

**Registry change flow:**
1. User toggles "Replace Windows Shell" in Wave OS Settings
2. Tauri requests elevation (UAC prompt)
3. Registry key written: `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\Shell` = `"C:\...\wave-os-shell.exe" --shell`
4. User prompted: "Changes take effect on next login. Reboot now?"
5. On next boot, Windows launches Wave OS instead of Explorer

### 3. System Tray (`tray.rs`)

System tray icon with context menu for quick access.

```
┌──────────────────────────┐
│  🌊 Wave OS              │  ← tray icon (right-click menu)
│  ─────────────────────  │
│  ▸ Open Wave OS          │  ← brings window to front
│  ▸ Windows Mode          │  ← temporarily launch explorer.exe
│  ─────────────────────  │
│  ▸ Settings              │  ← opens Wave OS settings tab
│  ▸ Check for Updates     │
│  ─────────────────────  │
│  ▸ Shell Mode            │
│    ○ App (default)       │  ← radio: explorer is shell
│    ○ Shell Replacement   │  ← radio: Wave OS is shell
│    ○ Kiosk Lockdown      │  ← radio: Wave OS + locked
│  ─────────────────────  │
│  ✕ Exit to Windows       │  ← closes Wave OS, launches explorer
└──────────────────────────┘
```

**Tray states:**
- 🌊 Teal icon — Wave OS running, connected
- 🌊 Dimmed icon — Wave OS running, no internet
- 🌊 Red icon — Error (WebView2 missing, etc.)
- 🌊 Spinning — Starting up / loading

**Left-click behavior:**
- If window hidden → show and focus
- If window visible → minimize to tray
- If shell mode → no-op (window is always visible)

```rust
// src/tray.rs

pub fn create_tray(app: &App) -> Result<()> {
    let tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("Wave OS")
        .menu(&build_tray_menu(app)?)
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "open" => show_window(app),
                "windows" => launch_explorer(),
                "settings" => open_wave_settings(app),
                "shell_app" => set_shell_mode(app, ShellMode::App),
                "shell_replace" => set_shell_mode(app, ShellMode::Shell),
                "shell_kiosk" => set_shell_mode(app, ShellMode::Kiosk),
                "exit" => exit_to_windows(app),
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, .. } = event {
                toggle_window_visibility(tray.app_handle());
            }
        })
        .build(app)?;
    
    Ok(())
}
```

### 4. Global Hotkeys (`hotkeys.rs`)

| Hotkey | Action | Mode |
|--------|--------|------|
| `Ctrl+Alt+W` | Toggle Wave OS / Explorer | App + Shell |
| `Ctrl+Alt+S` | Open Wave OS Settings | App + Shell |
| `Ctrl+Alt+T` | Open Terminal (if available) | App + Shell |
| `Win+Esc` | Exit to Windows (shell mode) | Shell only |
| `Ctrl+Alt+L` | Lock kiosk (kiosk mode) | Kiosk only |

```rust
// src/hotkeys.rs

pub fn register_hotkeys(app: &App, mode: ShellMode) -> Result<()> {
    // Always available
    GlobalShortcut::builder()
        .with_shortcut("Ctrl+Alt+W")
        .with_handler(|app| toggle_wave_explorer(app))
        .build(app)?;
    
    GlobalShortcut::builder()
        .with_shortcut("Ctrl+Alt+S")
        .with_handler(|app| open_settings(app))
        .build(app)?;
    
    // Shell mode only
    if mode == ShellMode::Shell || mode == ShellMode::Kiosk {
        GlobalShortcut::builder()
            .with_shortcut("Win+Esc")
            .with_handler(|app| exit_to_windows(app))
            .build(app)?;
    }
    
    // Kiosk mode only
    if mode == ShellMode::Kiosk {
        GlobalShortcut::builder()
            .with_shortcut("Ctrl+Alt+L")
            .with_handler(|app| lock_kiosk(app))
            .build(app)?;
    }
    
    Ok(())
}
```

### 5. Process Manager (`process.rs`)

Launches and manages native processes from Wave OS.

```rust
// src/process.rs

/// Spawn a native process (called from JS via IPC)
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
    let child = cmd.spawn().map_err(|e| e.to_string())?;
    Ok(child.id())
}

/// Kill a process by PID
#[tauri::command]
pub fn kill_process(pid: u32) -> Result<(), String> {
    Command::new("taskkill")
        .args(["/f", "/pid", &pid.to_string()])
        .output()
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Check if a process is running
#[tauri::command]
pub fn is_process_running(name: String) -> bool {
    let output = Command::new("tasklist")
        .args(["/fi", &format!("imagename eq {}", name), "/fo", "csv"])
        .output();
    match output {
        Ok(o) => String::from_utf8_lossy(&o.stdout).contains(&name),
        Err(_) => false,
    }
}

/// Launch explorer.exe (temporary Windows access)
#[tauri::command]
pub fn launch_explorer() -> Result<(), String> {
    Command::new("explorer.exe").spawn().map_err(|e| e.to_string())?;
    Ok(())
}

/// Launch Windows Task Manager
#[tauri::command]
pub fn launch_task_manager() -> Result<(), String> {
    Command::new("taskmgr.exe").spawn().map_err(|e| e.to_string())?;
    Ok(())
}
```

### 6. Ollama Detector (`ollama.rs`)

Detects local Ollama instance and bridges it to Wave OS.

```rust
// src/ollama.rs

/// Check if Ollama is running on localhost:11434
#[tauri::command]
pub async fn check_ollama() -> OllamaStatus {
    let url = "http://localhost:11434/api/tags";
    
    match reqwest::get(url).await {
        Ok(resp) if resp.status().is_success() => {
            let body = resp.json::<serde_json::Value>().await.ok();
            let models = body
                .and_then(|v| v["models"].as_array().cloned())
                .unwrap_or_default();
            
            OllamaStatus {
                running: true,
                models: models.iter()
                    .filter_map(|m| m["name"].as_str().map(String::from))
                    .collect(),
                version: body
                    .and_then(|v| v["version"].as_str().map(String::from))
                    .unwrap_or_default(),
            }
        }
        _ => OllamaStatus {
            running: false,
            models: vec![],
            version: String::new(),
        }
    }
}

/// Start Ollama if installed but not running
#[tauri::command]
pub fn start_ollama() -> Result<(), String> {
    // Try common install paths
    let paths = [
        "ollama",  // PATH
        "C:\\Users\\%USERNAME%\\AppData\\Local\\Programs\\Ollama\\ollama.exe",
        "C:\\Program Files\\Ollama\\ollama.exe",
    ];
    
    for path in &paths {
        if let Ok(child) = Command::new(path).arg("serve").spawn() {
            return Ok(());
        }
    }
    
    Err("Ollama not found. Install from ollama.com".to_string())
}

pub struct OllamaStatus {
    running: bool,
    models: Vec<String>,
    version: String,
}
```

### 7. Auto-Start (`autostart.rs`)

Three tiers of auto-start, from gentle to aggressive.

```rust
// src/autostart.rs

/// Tier 1: Registry Run key (starts on login, no admin needed)
pub fn enable_autostart_user(exe_path: &str) -> Result<()> {
    let key = "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run";
    write_registry(HKEY_CURRENT_USER, key, "WaveOS", exe_path)?;
    Ok(())
}

/// Tier 2: Task Scheduler (starts on boot, before login)
pub fn enable_autostart_system(exe_path: &str) -> Result<()> {
    Command::new("schtasks")
        .args([
            "/create", "/tn", "WaveOS", "/tr", exe_path,
            "/sc", "onlogon", "/rl", "highest", "/f"
        ])
        .output()
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Tier 3: Shell replacement (starts instead of explorer.exe)
/// Handled by shell.rs — registry Winlogon\Shell key

/// Disable auto-start
pub fn disable_autostart() -> Result<()> {
    // Remove Run key
    delete_registry(HKEY_CURRENT_USER, 
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run", "WaveOS")?;
    // Remove scheduled task
    Command::new("schtasks")
        .args(["/delete", "/tn", "WaveOS", "/f"])
        .output()?;
    // Restore shell
    disable_shell_mode()?;
    Ok(())
}
```

### 8. File Bridge (`file_bridge.rs`)

Bridges Wave OS file operations to the native file system.

```rust
// src/file_bridge.rs

/// Open native file picker → return selected file path(s)
#[tauri::command]
pub async fn pick_files(filters: Option<Vec<FileFilter>>) -> Vec<String> {
    let mut builder = FileDialogBuilder::new();
    if let Some(filters) = filters {
        for f in filters {
            builder.add_filter(f.name, &f.extensions);
        }
    }
    builder.pick_files().unwrap_or_default()
}

/// Open native folder picker → return selected path
#[tauri::command]
pub async fn pick_folder() -> Option<String> {
    FileDialogBuilder::new().pick_folder()
}

/// Save file to disk (bytes from Wave OS)
#[tauri::command]
pub async fn save_file(path: String, data: Vec<u8>) -> Result<(), String> {
    std::fs::write(&path, &data).map_err(|e| e.to_string())
}

/// Read file from disk → return bytes to Wave OS
#[tauri::command]
pub async fn read_file(path: String) -> Result<Vec<u8>, String> {
    std::fs::read(&path).map_err(|e| e.to_string())
}

/// Open file with default system handler
#[tauri::command]
pub fn open_file(path: String) -> Result<(), String> {
    Command::new("cmd")
        .args(["/c", "start", "", &path])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Show file in Explorer
#[tauri::command]
pub fn show_in_explorer(path: String) -> Result<(), String> {
    Command::new("explorer.exe")
        .args(["/select,", &path])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}
```

### 9. Kiosk Mode (`kiosk.rs`)

Full lockdown for dedicated Wave OS machines.

```rust
// src/kiosk.rs

/// Enable kiosk mode lockdown
pub fn enable_kiosk() -> Result<()> {
    // Disable Task Manager
    write_registry(
        HKEY_CURRENT_USER,
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Policies\\System",
        "DisableTaskMgr",
        &1u32
    )?;
    
    // Disable registry editing
    write_registry(
        HKEY_CURRENT_USER,
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Policies\\System",
        "DisableRegistryTools",
        &1u32
    )?;
    
    // Disable CMD
    write_registry(
        HKEY_CURRENT_USER,
        "SOFTWARE\\Policies\\Microsoft\\Windows\\System",
        "DisableCMD",
        &1u32
    )?;
    
    // Install low-level keyboard hook to block Win key, Ctrl+Esc, Alt+Tab
    install_keyboard_hook()?;
    
    Ok(())
}

/// Disable kiosk mode (requires PIN)
pub fn disable_kiosk(pin: String) -> Result<(), String> {
    if !verify_pin(&pin) {
        return Err("Invalid PIN".to_string());
    }
    
    // Reverse all registry changes
    delete_registry(HKEY_CURRENT_USER,
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Policies\\System",
        "DisableTaskMgr")?;
    delete_registry(HKEY_CURRENT_USER,
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Policies\\System",
        "DisableRegistryTools")?;
    delete_registry(HKEY_CURRENT_USER,
        "SOFTWARE\\Policies\\Microsoft\\Windows\\System",
        "DisableCMD")?;
    
    uninstall_keyboard_hook()?;
    
    Ok(())
}

/// Low-level keyboard hook to intercept system keys
fn install_keyboard_hook() -> Result<()> {
    // Uses SetWindowsHookExW with WH_KEYBOARD_LL
    // Blocks: VK_LWIN, VK_RWIN, VK_ESCAPE (with Ctrl)
    // Allows: Ctrl+Alt+L (lock kiosk), Ctrl+Alt+Del (can't be blocked easily)
    Ok(())
}
```

### 10. IPC Bridge (`shell-bridge.js`)

Injected into the WebView before Wave OS loads. Exposes native APIs to the web app.

```javascript
// src/shell-bridge.js — injected via Tauri preload

window.__waveShell = {
    // Process management
    spawnProcess: (exe, args, cwd) => window.__TAURI__.invoke('spawn_process', { 
        executable: exe, args: args || [], workingDir: cwd 
    }),
    killProcess: (pid) => window.__TAURI__.invoke('kill_process', { pid }),
    isProcessRunning: (name) => window.__TAURI__.invoke('is_process_running', { name }),
    
    // File system
    pickFiles: (filters) => window.__TAURI__.invoke('pick_files', { filters }),
    pickFolder: () => window.__TAURI__.invoke('pick_folder'),
    saveFile: (path, data) => window.__TAURI__.invoke('save_file', { path, data }),
    readFile: (path) => window.__TAURI__.invoke('read_file', { path }),
    openFile: (path) => window.__TAURI__.invoke('open_file', { path }),
    showInExplorer: (path) => window.__TAURI__.invoke('show_in_explorer', { path }),
    
    // Shell management
    getShellMode: () => window.__TAURI__.invoke('get_shell_mode'),
    enableShellMode: () => window.__TAURI__.invoke('enable_shell_mode'),
    disableShellMode: () => window.__TAURI__.invoke('disable_shell_mode'),
    launchExplorer: () => window.__TAURI__.invoke('launch_explorer'),
    killExplorer: () => window.__TAURI__.invoke('kill_explorer'),
    
    // Ollama
    checkOllama: () => window.__TAURI__.invoke('check_ollama'),
    startOllama: () => window.__TAURI__.invoke('start_ollama'),
    
    // Kiosk
    enableKiosk: () => window.__TAURI__.invoke('enable_kiosk'),
    disableKiosk: (pin) => window.__TAURI__.invoke('disable_kiosk', { pin }),
    
    // Window
    minimizeToTray: () => window.__TAURI__.invoke('minimize_to_tray'),
    toggleFullscreen: () => window.__TAURI__.invoke('toggle_fullscreen'),
    
    // System info
    getPlatform: () => window.__TAURI__.invoke('get_platform'),
    getVersion: () => window.__TAURI__.invoke('get_version'),
    
    // Notifications
    notify: (title, body) => window.__TAURI__.invoke('send_notification', { title, body }),
    
    // Auto-start
    enableAutoStart: (tier) => window.__TAURI__.invoke('enable_autostart', { tier }),
    disableAutoStart: () => window.__TAURI__.invoke('disable_autostart'),
    
    // Check if running as shell
    isShell: () => window.__TAURI__.invoke('is_shell_mode'),
    
    // App info
    isNative: true,
    platform: 'windows',
    version: '1.0.0'
};

// Emit ready event
window.dispatchEvent(new CustomEvent('wave-shell-ready'));
```

### 11. Wave OS Settings Integration

New "Desktop Shell" section in Wave OS Settings (existing Settings app).

```jsx
{/* Desktop Shell Settings — only shown when window.__waveShell exists */}
{window.__waveShell?.isNative && (
  <SettingsSection title="Desktop Shell" icon="🌊">
    {/* Mode selector */}
    <SettingRow label="Shell Mode">
      <RadioGroup value={shellMode} onChange={setShellMode}>
        <Radio value="app" label="App Mode (recommended)" 
               description="Wave OS runs as a desktop app. Windows stays normal." />
        <Radio value="shell" label="Shell Replacement" 
               description="Wave OS replaces the Windows desktop on next boot." />
        <Radio value="kiosk" label="Kiosk Lockdown" 
               description="Full lockdown. No Alt+Tab, no Task Manager. Requires PIN to exit." />
      </RadioGroup>
    </SettingRow>
    
    {/* Auto-start */}
    <SettingRow label="Auto-Start">
      <Toggle checked={autoStart} onChange={setAutoStart} />
      <p className="text-muted">
        Start Wave OS automatically when you log in
      </p>
    </SettingRow>
    
    {/* System tray */}
    <SettingRow label="System Tray">
      <Toggle checked={trayEnabled} onChange={setTrayEnabled} />
      <p className="text-muted">
        Show Wave OS icon in system tray for quick access
      </p>
    </SettingRow>
    
    {/* Global hotkeys */}
    <SettingRow label="Global Hotkeys">
      <div className="space-y-2">
        <div className="flex justify-between">
          <span>Toggle Wave/Windows</span>
          <Kbd>Ctrl+Alt+W</Kbd>
        </div>
        <div className="flex justify-between">
          <span>Open Settings</span>
          <Kbd>Ctrl+Alt+S</Kbd>
        </div>
        <div className="flex justify-between">
          <span>Exit to Windows</span>
          <Kbd>Win+Esc</Kbd>
        </div>
      </div>
    </SettingRow>
    
    {/* Ollama status */}
    <SettingRow label="Local AI (Ollama)">
      <div className="flex items-center gap-3">
        <StatusDot running={ollamaRunning} />
        <span>{ollamaRunning ? `Running (${ollamaModels.length} models)` : 'Not detected'}</span>
        {!ollamaRunning && (
          <button onClick={startOllama} className="btn-secondary">
            Start Ollama
          </button>
        )}
      </div>
    </SettingRow>
    
    {/* File system access */}
    <SettingRow label="File System">
      <Toggle checked={fsAccess} onChange={setFsAccess} />
      <p className="text-muted">
        Allow Wave OS to read/write files on your hard drive directly
      </p>
    </SettingRow>
    
    {/* Shell mode warning */}
    {shellMode === 'shell' && (
      <div className="warning-banner">
        ⚠️ Shell mode replaces your Windows desktop. To undo this, 
        use Ctrl+Alt+W to temporarily access Windows, or change 
        the setting back here. A reboot is required for changes 
        to take effect.
      </div>
    )}
    
    {/* Kiosk warning */}
    {shellMode === 'kiosk' && (
      <div className="danger-banner">
        🔒 Kiosk mode locks down the system completely. Set a PIN 
        to exit. Without the PIN, the only way out is a system 
        restore. Make sure you remember it.
      </div>
    )}
  </SettingsSection>
)}
```

---

## tauri.conf.json

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Wave OS Shell",
  "version": "1.0.0",
  "identifier": "com.oswave.shell",
  "build": {
    "frontendDist": "../src",
    "devUrl": "http://localhost:1420"
  },
  "app": {
    "windows": [
      {
        "label": "wave-os",
        "url": "https://app.oswave.io",
        "title": "Wave OS",
        "width": 1920,
        "height": 1080,
        "maximized": true,
        "decorations": false,
        "resizable": true,
        "fullscreen": false
      }
    ],
    "security": {
      "csp": "default-src 'self' https://app.oswave.io https://*.base44.app https://*.oswave.io https://media.base44.com http://localhost:11434 https://*.thetaedgecloud.com https://*.ipfs.io data: blob:; style-src 'self' 'unsafe-inline' https://app.oswave.io; script-src 'self' 'unsafe-inline' 'unsafe-eval' https://app.oswave.io; img-src 'self' data: blob: https:; connect-src 'self' https: http://localhost:11434 wss:;"
    },
    "trayIcon": {
      "id": "wave-tray",
      "iconPath": "icons/icon-tray.ico",
      "tooltip": "Wave OS"
    }
  },
  "bundle": {
    "active": true,
    "targets": ["msi", "nsis"],
    "icon": ["icons/icon.ico"],
    "windows": {
      "webviewInstallMode": {
        "type": "downloadBootstrapper"
      }
    }
  },
  "plugins": {
    "global-shortcut": {},
    "notification": {},
    "fs": {
      "scope": ["$HOME/**", "$DOCUMENT/**", "$DOWNLOAD/**", "$DESKTOP/**"]
    },
    "updater": {
      "pubkey": "",
      "endpoints": ["https://oswave.io/updates/latest.json"]
    }
  }
}
```

---

## Cargo.toml Dependencies

```toml
[package]
name = "wave-os-shell"
version = "1.0.0"
edition = "2021"

[dependencies]
tauri = { version = "2", features = ["tray-icon", "devtools"] }
tauri-plugin-global-shortcut = "2"
tauri-plugin-notification = "2"
tauri-plugin-fs = "2"
tauri-plugin-dialog = "2"
tauri-plugin-updater = "2"
tauri-plugin-shell = "2"
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
winreg = "0.52"
windows = { version = "0.58", features = [
    "Win32_Foundation",
    "Win32_UI_WindowsAndMessaging",
    "Win32_System_Registry",
    "Win32_Security",
    "Win32_System_Threading"
] }
log = "0.4"
env_logger = "0.11"

[build-dependencies]
tauri-build = { version = "2", features = [] }
```

---

## Build & Deploy

### Prerequisites
- Rust toolchain (rustup)
- Tauri CLI v2 (`cargo install tauri-cli --version "^2"`)
- WiX Toolset (for MSI)
- Windows SDK
- WebView2 (pre-installed on Win 10/11, or bundled bootstrapper)

### Build commands
```bash
# Development
cargo tauri dev

# Production build (creates MSI + NSIS)
cargo tauri build

# Output: src-tauri/target/release/bundle/
#   msi/Wave-OS-Shell_1.0.0_x64_en-US.msi
#   nsis/Wave-OS-Shell_1.0.0_x64-setup.exe
```

### GitHub Actions CI
```yaml
name: Build Wave OS Shell
on:
  push:
    tags: ['v*']
jobs:
  build:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install tauri-cli --version "^2"
      - run: cargo tauri build
      - uses: softprops/action-gh-release@v1
        with:
          files: |
            src-tauri/target/release/bundle/msi/*.msi
            src-tauri/target/release/bundle/nsis/*.exe
```

### Distribution
- GitHub Releases (MSI + NSIS installer)
- Also published to `oswave.io/download`
- Auto-update via Tauri updater plugin (checks `oswave.io/updates/latest.json`)
- WebView2 bootstrapper bundled in MSI for older Windows

---

## Security Considerations

1. **Shell replacement requires admin** — UAC prompt for registry changes
2. **Kiosk PIN** — SHA-256 hashed, stored in app data, 4-8 digits
3. **CSP** — Locked to `app.oswave.io` and trusted domains only
4. **File system scope** — Limited to user directories (HOME, Documents, Downloads, Desktop)
5. **No arbitrary code execution** — `spawnProcess` only accepts whitelisted executables in production
6. **Registry changes logged** — Rollback log stored in `%APPDATA%/wave-os-shell/shell-log.json`
7. **Updater** — Signature-verified via Tauri updater pubkey

---

## Detection in Wave OS Web App

Wave OS can detect if it's running inside the native shell via `window.__waveShell`. This enables native-only features:

```javascript
// In Wave OS web app
const isNativeShell = !!window.__waveShell;
const shellMode = window.__waveShell?.getShellMode?.() || 'web';

// Native-only features
if (isNativeShell) {
    // Enable Desktop Shell settings section
    // Enable native file dialogs (pickFiles, saveFile)
    // Enable Ollama auto-detection via Rust (not just browser fetch)
    // Enable process spawning (launch games, apps)
    // Enable system notifications (native, not browser)
    // Show "Exit to Windows" button in Settings
}
```

---

## Migration Path

### Phase 1: App Mode (v1.0)
- Tauri v2 wrapper around `app.oswave.io`
- System tray, auto-start, global hotkeys
- Ollama detection, native file dialogs
- MSI installer
- **No shell replacement** — just a better Wave OS experience

### Phase 2: Shell Mode (v1.1)
- Shell replacement via registry key
- "Exit to Windows" flow
- Shell mode settings UI in Wave OS
- `Ctrl+Alt+W` toggle

### Phase 3: Kiosk Mode (v1.2)
- Full lockdown (disable TaskMgr, CMD, registry, Win key)
- PIN-based exit
- Keyboard hooks
- For dedicated Wave OS machines

### Phase 4: Native Features (v2.0)
- Local file manager (bypass browser limitations)
- Native context menus
- Desktop shortcuts to Wave OS apps
- Wallpaper integration (Wave OS wallpaper = Windows wallpaper)
- Start menu replacement (Wave OS app launcher = Start menu)
- Taskbar integration (show running apps in Windows taskbar)
- Multi-monitor support (different Wave OS instances per monitor)

---

## Repo
- GitHub: `xBuildyTeam/wave-os-shell`
- License: MIT
- CI: GitHub Actions (Windows runner)
- Releases: GitHub Releases + `oswave.io/download`
