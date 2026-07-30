//! Tauri shell for WinServeAI.
//!
//! All lifecycle goes through [`winserve::ServerManager`]. The webview must not
//! spawn `llama-server` or invent a second orchestrator.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::Serialize;
use tauri::State;
use tokio::sync::Mutex;
use winserve::server::manager::{ServerManager, Status};
use winserve::Config;

struct AppState {
    manager: Mutex<ServerManager>,
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
async fn manager_config_summary(state: State<'_, Arc<AppState>>) -> Result<ConfigSummary, String> {
    let mgr = state.manager.lock().await;
    let cfg: &Config = mgr.config();
    Ok(ConfigSummary {
        host: cfg.server.host.clone(),
        port: cfg.server.port,
        model_path: cfg.model.path.display().to_string(),
        logging_dir: cfg.logging.dir.display().to_string(),
    })
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ConfigSummary {
    host: String,
    port: u16,
    model_path: String,
    logging_dir: String,
}

#[allow(dead_code)]
fn _status_exhaustiveness(s: Status) -> &'static str {
    // Keep tray aligned with manager status vocabulary.
    s.as_str()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let root = find_root();
    let cfg = config_path(&root);
    let manager = ServerManager::load_config(&root, &cfg).unwrap_or_else(|e| {
        panic!(
            "winserve-tray: failed to load config {} under {}: {e}",
            cfg.display(),
            root.display()
        )
    });
    let state = Arc::new(AppState {
        manager: Mutex::new(manager),
    });

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            manager_status,
            manager_start,
            manager_stop,
            manager_restart,
            manager_endpoint,
            manager_config_summary,
        ])
        .run(tauri::generate_context!())
        .expect("error while running winserve-tray");
}
