// Wave OS Shell Bridge — Injected into WebView before Wave OS loads
// Exposes native APIs to the web app via window.__waveShell

window.__waveShell = {
    // ===== Identity =====
    isNative: true,
    platform: 'windows',
    version: '1.0.0',
    shellMode: 'app',

    // ===== Process Management =====
    spawnProcess: (exe, args, cwd) => window.__TAURI__.core.invoke('spawn_process', {
        executable: exe, args: args || [], workingDir: cwd
    }),
    killProcess: (pid) => window.__TAURI__.core.invoke('kill_process', { pid }),
    isProcessRunning: (name) => window.__TAURI__.core.invoke('is_process_running', { name }),
    launchExplorer: () => window.__TAURI__.core.invoke('launch_explorer'),
    launchTaskManager: () => window.__TAURI__.core.invoke('launch_task_manager'),

    // ===== Shell Management =====
    getShellMode: () => window.__TAURI__.core.invoke('get_shell_mode'),
    enableShellMode: (exePath) => window.__TAURI__.core.invoke('enable_shell_mode', { exePath }),
    disableShellMode: () => window.__TAURI__.core.invoke('disable_shell_mode'),
    isShell: () => window.__TAURI__.core.invoke('is_shell_mode'),
    killExplorer: () => window.__TAURI__.core.invoke('kill_explorer'),

    // ===== Ollama =====
    checkOllama: () => window.__TAURI__.core.invoke('check_ollama'),
    startOllama: () => window.__TAURI__.core.invoke('start_ollama'),

    // ===== File System =====
    pickFiles: (filters) => window.__TAURI__.core.invoke('pick_files', { filters }),
    pickFolder: () => window.__TAURI__.core.invoke('pick_folder'),
    saveFile: (path, data) => window.__TAURI__.core.invoke('save_file', { path, data }),
    readFile: (path) => window.__TAURI__.core.invoke('read_file', { path }),
    openFile: (path) => window.__TAURI__.core.invoke('open_file', { path }),
    showInExplorer: (path) => window.__TAURI__.core.invoke('show_in_explorer', { path }),

    // ===== Auto-Start =====
    enableAutoStart: (tier) => window.__TAURI__.core.invoke('enable_autostart', { tier: tier || 1 }),
    disableAutoStart: () => window.__TAURI__.core.invoke('disable_autostart'),
    getAutoStartStatus: () => window.__TAURI__.core.invoke('get_autostart_status'),

    // ===== Window Management =====
    minimizeToTray: () => window.__TAURI__.core.invoke('minimize_to_tray'),
    toggleFullscreen: () => window.__TAURI__.core.invoke('toggle_fullscreen'),
    getWindowState: () => window.__TAURI__.core.invoke('get_window_state'),

    // ===== Notifications =====
    notify: (title, body) => {
        if (window.__TAURI__?.notification) {
            window.__TAURI__.notification.sendNotification({
                title: title || 'Wave OS',
                body: body || ''
            });
        }
    },

    // ===== Updates =====
    checkForUpdates: async () => {
        if (window.__TAURI__?.updater) {
            try {
                const update = await window.__TAURI__.updater.check();
                if (update?.available) {
                    return { available: true, version: update.version, notes: update.body };
                }
                return { available: false };
            } catch (e) {
                return { available: false, error: e.message };
            }
        }
        return { available: false, error: 'Updater not available' };
    },

    installUpdate: async () => {
        if (window.__TAURI__?.updater) {
            try {
                await window.__TAURI__.updater.downloadAndInstall();
                return { success: true };
            } catch (e) {
                return { success: false, error: e.message };
            }
        }
        return { success: false, error: 'Updater not available' };
    },

    // ===== Init =====
    init: async function() {
        try {
            this.shellMode = await this.getShellMode();
            console.log('[Wave Shell] Initialized, mode:', this.shellMode);
        } catch (e) {
            console.warn('[Wave Shell] Failed to get shell mode:', e);
        }
        window.dispatchEvent(new CustomEvent('wave-shell-ready', { detail: { mode: this.shellMode } }));
    }
};

// Auto-initialize
window.__waveShell.init();

// Prevent context menu in production
if (!window.__TAURI__?.internal?.isDev) {
    window.addEventListener('contextmenu', (e) => {
        // Allow context menu in input/textarea fields
        if (e.target.tagName !== 'INPUT' && e.target.tagName !== 'TEXTAREA') {
            e.preventDefault();
        }
    });
}

// Handle window close button → minimize to tray instead of exit
window.addEventListener('beforeunload', (e) => {
    // In shell mode, prevent unload entirely
    if (window.__waveShell.shellMode !== 'app') {
        e.preventDefault();
        e.returnValue = '';
    }
});

console.log('[Wave Shell] Bridge injected');
