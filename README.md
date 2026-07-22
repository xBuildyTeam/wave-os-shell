# Wave OS Shell

Tauri v2 desktop wrapper for [Wave OS](https://app.oswave.io).

## Modes

| Mode | Description |
|------|-------------|
| **App** (default) | Wave icon on desktop. Double-click → Wave OS opens in borderless fullscreen window. Explorer runs underneath. |
| **Shell** | Replaces `explorer.exe` via registry. On boot, Wave OS IS the desktop. `Ctrl+Alt+W` for temporary Windows access. |
| **Kiosk** | Shell mode + full lockdown. No Alt+Tab, no Task Manager. PIN to exit. |

## Features

- 🌊 **System tray** — right-click for quick access, left-click to toggle visibility
- ⌨️ **Global hotkeys** — `Ctrl+Alt+W` (toggle Wave/Windows), `Ctrl+Alt+S` (settings)
- 📁 **Native file dialogs** — open/save/pick files from your hard drive
- 🤖 **Ollama detection** — auto-detect and start local Ollama instance
- 🔔 **Native notifications** — Windows toast notifications
- 🔄 **Auto-updates** — checks `oswave.io/updates/latest.json`
- 🚀 **Auto-start** — 3 tiers (Run key, Task Scheduler, shell replacement)
- ⚙️ **Process spawning** — launch native apps from Wave OS

## Build

### Prerequisites
- [Rust](https://rustup.rs/)
- [Tauri CLI v2](https://v2.tauri.app/): `cargo install tauri-cli --version "^2"`
- [WiX Toolset](https://wixtoolset.org/) (for MSI installer)
- Windows 10/11 with WebView2

### Development
```bash
cargo tauri dev
```

### Production Build
```bash
cargo tauri build
```

Output:
- `src-tauri/target/release/bundle/msi/Wave-OS-Shell_1.0.0_x64_en-US.msi`
- `src-tauri/target/release/bundle/nsis/Wave-OS-Shell_1.0.0_x64-setup.exe`

## Architecture

```
Wave OS Shell (Tauri v2)
├── WebView2 (loads app.oswave.io — same UI as web, pixel-perfect)
├── IPC Bridge (shell-bridge.js → window.__waveShell)
├── System Tray (tray-icon plugin)
├── Global Hotkeys (global-shortcut plugin)
├── Shell Manager (Windows Registry — Winlogon\Shell)
├── Process Manager (std::process::Command)
├── Ollama Detector (reqwest → localhost:11434)
├── File Bridge (fs + dialog plugins)
└── Auto-Start (Registry Run key + Task Scheduler)
```

## Detection in Wave OS

Wave OS can detect the native shell via `window.__waveShell`:

```javascript
if (window.__waveShell?.isNative) {
    // Enable Desktop Shell settings
    // Enable native file dialogs
    // Enable Ollama auto-detection
    // Enable process spawning
    // Show "Exit to Windows" in settings
}
```

## License

MIT
