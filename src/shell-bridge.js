// Wave OS Shell Bridge v2 — Tauri v2 compatible
window.__waveShell = {
    isNative: typeof window !== 'undefined' && typeof window.__TAURI__ !== 'undefined',
    platform: 'windows',
    version: '2.0.0',
    shellMode: 'app',
    _init: () => {
        const isDev = window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1';
        if (!isDev) {
            window.addEventListener('contextmenu', (e) => {
                if (e.target.tagName !== 'INPUT' && e.target.tagName !== 'TEXTAREA') e.preventDefault();
            });
        }
        console.log('[WaveShell] Bridge initialized. Native:', window.__waveShell.isNative);
    },
    isTauri: () => typeof window.__TAURI__ !== 'undefined',
    spawnProcess: (exe, args, cwd) => window.__TAURI__.core.invoke('spawn_process', { executable: exe, args: args || [], workingDir: cwd }),
    killProcess: (pid) => window.__TAURI__.core.invoke('kill_process', { pid }),
    isProcessRunning: (name) => window.__TAURI__.core.invoke('is_process_running', { name }),
    launchExplorer: () => window.__TAURI__.core.invoke('launch_explorer'),
    launchTaskManager: () => window.__TAURI__.core.invoke('launch_task_manager'),
    getShellMode: () => window.__TAURI__.core.invoke('get_shell_mode'),
    enableShellMode: (exePath) => window.__TAURI__.core.invoke('enable_shell_mode', { exePath }),
    disableShellMode: () => window.__TAURI__.core.invoke('disable_shell_mode'),
    isShell: () => window.__TAURI__.core.invoke('is_shell_mode'),
    killExplorer: () => window.__TAURI__.core.invoke('kill_explorer'),
    checkOllama: () => window.__TAURI__.core.invoke('check_ollama'),
    startOllama: () => window.__TAURI__.core.invoke('start_ollama'),
    pickFiles: (filters) => window.__TAURI__.core.invoke('pick_files', { filters }),
    pickFolder: () => window.__TAURI__.core.invoke('pick_folder'),
    saveFile: (path, data) => window.__TAURI__.core.invoke('save_file', { path, data }),
    readFile: (path) => window.__TAURI__.core.invoke('read_file', { path }),
    openFile: (path) => window.__TAURI__.core.invoke('open_file', { path }),
    showInExplorer: (path) => window.__TAURI__.core.invoke('show_in_explorer', { path }),
    enableAutoStart: (tier) => window.__TAURI__.core.invoke('enable_autostart', { tier: tier || 1 }),
    disableAutoStart: () => window.__TAURI__.core.invoke('disable_autostart'),
    getAutoStartStatus: () => window.__TAURI__.core.invoke('get_autostart_status'),
    minimizeToTray: () => window.__TAURI__.core.invoke('minimize_to_tray'),
    toggleFullscreen: () => window.__TAURI__.core.invoke('toggle_fullscreen'),
    getWindowState: () => window.__TAURI__.core.invoke('get_window_state'),
    notify: (title, body) => { if (window.__TAURI__?.notification) window.__TAURI__.notification.sendNotification({ title, body }); },
    toggleDevTools: () => { if (window.__TAURI__?.core) window.__TAURI__.core.invoke('plugin:window|toggle_devtools'); }
};
window.__waveShell._init();
