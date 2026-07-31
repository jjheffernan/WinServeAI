//! Tauri shell for WinServeAI.
//!
//! All lifecycle goes through [`winserve::ServerManager`]. The webview must not
//! spawn `llama-server` or invent a second orchestrator.
//!
//! The tray process is the resident G owner: lockfile + named-pipe IPC so CLI
//! `status` / `stop` / `restart` can attach. Quit calls `stop()` first; Job
//! Object remains the orphan backstop.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, RunEvent, State, WindowEvent};
use tokio::sync::{mpsc, Mutex};
use winserve::ipc::lockfile::{self, LockGuard};
use winserve::ipc::pipe;
use winserve::server::manager::{ServerManager, Status};
use winserve::server::resident::{self, Command};

struct AppState {
    manager: Mutex<ServerManager>,
    /// Held for process lifetime; Drop removes the lockfile.
    _lock: LockGuard,
    stopping: AtomicBool,
}

async fn graceful_stop(state: &AppState) {
    if state.stopping.swap(true, Ordering::SeqCst) {
        return;
    }
    let mut mgr = state.manager.lock().await;
    let _ = mgr.stop().await;
}

fn show_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// Tray menu ids → actions (host-safe unit surface).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrayAction {
    Open,
    Start,
    Stop,
    Restart,
    Settings,
    Quit,
}

fn tray_action(id: &str) -> Option<TrayAction> {
    match id {
        "open" => Some(TrayAction::Open),
        "start" => Some(TrayAction::Start),
        "stop" => Some(TrayAction::Stop),
        "restart" => Some(TrayAction::Restart),
        "settings" => Some(TrayAction::Settings),
        "quit" => Some(TrayAction::Quit),
        _ => None,
    }
}

async fn tray_tooltip(state: &AppState) -> String {
    let mut mgr = state.manager.lock().await;
    let status = mgr.get_status().as_str();
    let endpoint = mgr.openai_base();
    format!("WinServeAI — {status}\n{endpoint}")
}

async fn refresh_tray_tooltip(app: &AppHandle, state: &AppState) {
    let tip = tray_tooltip(state).await;
    if let Some(tray) = app.tray_by_id("winserve") {
        let _ = tray.set_tooltip(Some(&tip));
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ManagerSnapshot {
    status: String,
    endpoint: String,
    error: Option<String>,
}

fn snapshot(manager: &mut ServerManager, error: Option<String>) -> ManagerSnapshot {
    ManagerSnapshot {
        status: manager.get_status().as_str().to_string(),
        endpoint: manager.openai_base(),
        error,
    }
}

fn find_root() -> PathBuf {
    // Prefer cwd (repo / install root), then walk up from the executable.
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if cwd.join("config").join("default.yaml").is_file() {
        return cwd;
    }
    if let Ok(exe) = std::env::current_exe() {
        for dir in exe.ancestors().skip(1).take(6) {
            if dir.join("config").join("default.yaml").is_file() {
                return dir.to_path_buf();
            }
        }
    }
    cwd
}

fn config_path(root: &Path) -> PathBuf {
    std::env::var_os("WINSERVE_CONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("config").join("default.yaml"))
}

#[tauri::command]
async fn manager_status(state: State<'_, Arc<AppState>>) -> Result<ManagerSnapshot, String> {
    let mut mgr = state.manager.lock().await;
    Ok(snapshot(&mut mgr, None))
}

#[tauri::command]
async fn manager_start(state: State<'_, Arc<AppState>>) -> Result<ManagerSnapshot, String> {
    let mut mgr = state.manager.lock().await;
    let err = mgr.start().await.err().map(|e| e.to_string());
    Ok(snapshot(&mut mgr, err))
}

#[tauri::command]
async fn manager_stop(state: State<'_, Arc<AppState>>) -> Result<ManagerSnapshot, String> {
    let mut mgr = state.manager.lock().await;
    let err = mgr.stop().await.err().map(|e| e.to_string());
    Ok(snapshot(&mut mgr, err))
}

#[tauri::command]
async fn manager_restart(state: State<'_, Arc<AppState>>) -> Result<ManagerSnapshot, String> {
    let mut mgr = state.manager.lock().await;
    let err = mgr.restart().await.err().map(|e| e.to_string());
    Ok(snapshot(&mut mgr, err))
}

#[tauri::command]
async fn manager_endpoint(state: State<'_, Arc<AppState>>) -> Result<String, String> {
    let mgr = state.manager.lock().await;
    Ok(mgr.openai_base())
}

#[tauri::command]
async fn manager_config_summary(state: State<'_, Arc<AppState>>) -> Result<SettingsDto, String> {
    let mgr = state.manager.lock().await;
    Ok(settings_dto(&mgr))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SettingsDto {
    host: String,
    port: u16,
    model_path: String,
    gpu_auto: bool,
    gpu_layers: String,
    context: u32,
    flash_attention: bool,
    logging_dir: String,
    config_path: String,
    editable: bool,
    status: String,
}

fn settings_dto(mgr: &ServerManager) -> SettingsDto {
    let cfg = mgr.config();
    let status = mgr.status();
    let editable = matches!(
        status,
        Status::Stopped | Status::Failed | Status::Crashed
    );
    SettingsDto {
        host: cfg.server.host.clone(),
        port: cfg.server.port,
        model_path: cfg.model.path.display().to_string(),
        gpu_auto: cfg.gpu.auto,
        gpu_layers: cfg.gpu.layers.clone(),
        context: cfg.runtime.context,
        flash_attention: cfg.runtime.flash_attention,
        logging_dir: cfg.logging.dir.display().to_string(),
        config_path: mgr.config_path().display().to_string(),
        editable,
        status: status.as_str().to_string(),
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SettingsPatch {
    host: String,
    port: u16,
    model_path: String,
    gpu_auto: bool,
    gpu_layers: String,
    context: u32,
    flash_attention: bool,
}

#[tauri::command]
async fn manager_apply_settings(
    state: State<'_, Arc<AppState>>,
    patch: SettingsPatch,
) -> Result<SettingsDto, String> {
    let mut mgr = state.manager.lock().await;
    let mut next = mgr.config().clone();
    next.server.host = patch.host;
    next.server.port = patch.port;
    next.model.path = PathBuf::from(patch.model_path);
    next.gpu.auto = patch.gpu_auto;
    next.gpu.layers = patch.gpu_layers;
    next.runtime.context = patch.context;
    next.runtime.flash_attention = patch.flash_attention;
    mgr.apply_config(next).map_err(|e| e.to_string())?;
    Ok(settings_dto(&mgr))
}

/// Native file dialog for a local `.gguf` only (no download). Returns the path string;
/// does not write YAML — caller saves via settings when editable.
#[tauri::command]
fn pick_model_path(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let picked = app
        .dialog()
        .file()
        .add_filter("GGUF models", &["gguf"])
        .blocking_pick_file();

    let Some(file) = picked else {
        return Ok(None);
    };
    let path = file.into_path().map_err(|e| e.to_string())?;
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .eq_ignore_ascii_case("gguf");
    if !ext {
        return Err("model path must be a .gguf file".into());
    }
    if !path.is_file() {
        return Err(format!("model path is not a file: {}", path.display()));
    }
    Ok(Some(path.display().to_string()))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct LogTailDto {
    server: Vec<String>,
    llama: Vec<String>,
    error: Vec<String>,
    dir: String,
}

/// Viewer-only: tail `server.log` / `llama.log` / `error.log` under the config logging dir.
#[tauri::command]
async fn manager_logs(
    state: State<'_, Arc<AppState>>,
    max_lines: Option<usize>,
) -> Result<LogTailDto, String> {
    let mgr = state.manager.lock().await;
    let rel = &mgr.config().logging.dir;
    let dir = if rel.is_absolute() {
        rel.clone()
    } else {
        mgr.root().join(rel)
    };
    let n = max_lines.unwrap_or(200).clamp(1, 2000);
    let tail = winserve::server::logs::tail_dir(&dir, n).map_err(|e| e.to_string())?;
    Ok(LogTailDto {
        server: tail.server,
        llama: tail.llama,
        error: tail.error,
        dir: dir.display().to_string(),
    })
}

#[allow(dead_code)]
fn _status_exhaustiveness(s: Status) -> &'static str {
    // Keep tray aligned with manager status vocabulary.
    s.as_str()
}

fn spawn_ipc(state: Arc<AppState>, pipe_name: String) {
    tauri::async_runtime::spawn(async move {
        let (tx, mut rx) = mpsc::channel::<Command>(8);
        let listen_name = pipe_name.clone();
        tokio::spawn(async move {
            if let Err(e) = pipe::listen(listen_name, tx).await {
                eprintln!("winserve-tray ipc listen ended: {e}");
            }
        });
        while let Some(Command { kind, reply }) = rx.recv().await {
            let mut mgr = state.manager.lock().await;
            let _ = reply.send(resident::handle(&mut mgr, kind).await);
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let root = find_root();
    let cfg = config_path(&root);
    let (lock, info) = LockGuard::acquire_default(lockfile::DEFAULT_PIPE_NAME).unwrap_or_else(|e| {
        panic!("winserve-tray: cannot acquire manager lock (is another owner running?): {e}")
    });
    eprintln!(
        "winserve-tray: lock {} (pid={}, pipe={})",
        lock.path().display(),
        info.pid,
        info.pipe
    );
    eprintln!("winserve-tray: ipc {}", pipe::endpoint_for(&info.pipe));

    let manager = ServerManager::load_config(&root, &cfg).unwrap_or_else(|e| {
        panic!(
            "winserve-tray: failed to load config {} under {}: {e}",
            cfg.display(),
            root.display()
        )
    });
    let state = Arc::new(AppState {
        manager: Mutex::new(manager),
        _lock: lock,
        stopping: AtomicBool::new(false),
    });
    spawn_ipc(state.clone(), info.pipe);

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            manager_status,
            manager_start,
            manager_stop,
            manager_restart,
            manager_endpoint,
            manager_config_summary,
            manager_apply_settings,
            pick_model_path,
            manager_logs,
        ])
        .setup(|app| {
            let open_i = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
            let start_i = MenuItem::with_id(app, "start", "Start", true, None::<&str>)?;
            let stop_i = MenuItem::with_id(app, "stop", "Stop", true, None::<&str>)?;
            let restart_i = MenuItem::with_id(app, "restart", "Restart", true, None::<&str>)?;
            let settings_i = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let sep1 = PredefinedMenuItem::separator(app)?;
            let sep2 = PredefinedMenuItem::separator(app)?;
            let menu = Menu::with_items(
                app,
                &[
                    &open_i,
                    &sep1,
                    &start_i,
                    &stop_i,
                    &restart_i,
                    &sep2,
                    &settings_i,
                    &quit_i,
                ],
            )?;

            let icon = app
                .default_window_icon()
                .cloned()
                .expect("winserve-tray: default window icon required for tray");

            let _tray = TrayIconBuilder::with_id("winserve")
                .icon(icon)
                .tooltip("WinServeAI")
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| {
                    let Some(action) = tray_action(event.id.as_ref()) else {
                        return;
                    };
                    let state = app.state::<Arc<AppState>>().inner().clone();
                    match action {
                        TrayAction::Open | TrayAction::Settings => show_main(app),
                        TrayAction::Quit => {
                            let app = app.clone();
                            tauri::async_runtime::spawn(async move {
                                graceful_stop(&state).await;
                                app.exit(0);
                            });
                        }
                        TrayAction::Start => {
                            let app = app.clone();
                            tauri::async_runtime::spawn(async move {
                                {
                                    let mut mgr = state.manager.lock().await;
                                    let _ = mgr.start().await;
                                }
                                refresh_tray_tooltip(&app, &state).await;
                            });
                        }
                        TrayAction::Stop => {
                            let app = app.clone();
                            tauri::async_runtime::spawn(async move {
                                {
                                    let mut mgr = state.manager.lock().await;
                                    let _ = mgr.stop().await;
                                }
                                refresh_tray_tooltip(&app, &state).await;
                            });
                        }
                        TrayAction::Restart => {
                            let app = app.clone();
                            tauri::async_runtime::spawn(async move {
                                {
                                    let mut mgr = state.manager.lock().await;
                                    let _ = mgr.restart().await;
                                }
                                refresh_tray_tooltip(&app, &state).await;
                            });
                        }
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    // Left click opens the menu; double-click focuses the window.
                    if let TrayIconEvent::DoubleClick {
                        button: MouseButton::Left,
                        ..
                    } = event
                    {
                        show_main(tray.app_handle());
                    }
                })
                .build(app)?;

            let state = app.state::<Arc<AppState>>().inner().clone();
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                refresh_tray_tooltip(&app_handle, &state).await;
            });
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building winserve-tray")
        .run(|app, event| match event {
            RunEvent::ExitRequested { api, .. } => {
                let state = app.state::<Arc<AppState>>();
                if !state.stopping.load(Ordering::SeqCst) {
                    api.prevent_exit();
                    let app = app.clone();
                    let state = Arc::clone(&state);
                    tauri::async_runtime::spawn(async move {
                        graceful_stop(&state).await;
                        app.exit(0);
                    });
                }
            }
            RunEvent::WindowEvent {
                event: WindowEvent::CloseRequested { api, .. },
                ..
            } => {
                // Close → hide to tray. Quit from the tray menu stops the manager.
                api.prevent_close();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
            _ => {}
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tray_menu_ids_map_to_actions() {
        assert_eq!(tray_action("open"), Some(TrayAction::Open));
        assert_eq!(tray_action("start"), Some(TrayAction::Start));
        assert_eq!(tray_action("stop"), Some(TrayAction::Stop));
        assert_eq!(tray_action("restart"), Some(TrayAction::Restart));
        assert_eq!(tray_action("settings"), Some(TrayAction::Settings));
        assert_eq!(tray_action("quit"), Some(TrayAction::Quit));
        assert_eq!(tray_action("unknown"), None);
    }
}
