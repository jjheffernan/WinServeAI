//! Resident attach over the local IPC transport (named pipe on Windows, TCP elsewhere).
//!
//! Proves CLI-shaped attach: listen → resident `ServerManager` loop → client request.

use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use winserve::ipc::pipe::{listen, request};
use winserve::server::config::Config;
use winserve::server::manager::ServerManager;
use winserve::server::resident::{self, Command};

#[tokio::test]
async fn resident_attach_status_and_endpoint() {
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("winserve-attach-{n}"));
    std::fs::create_dir_all(root.join("config")).unwrap();
    std::fs::create_dir_all(root.join("logs")).unwrap();
    let path = root.join("config").join("default.yaml");
    let mut cfg = Config::default();
    cfg.logging.dir = root.join("logs");
    cfg.model.path = root.join("missing.gguf");
    cfg.server.port = 18080;
    cfg.save(&path).unwrap();

    let mut mgr = ServerManager::load_config(&root, &path).unwrap();
    let pipe_name = format!("winserve-attach-{n}");
    let (tx, mut rx) = mpsc::channel::<Command>(8);

    let owner = tokio::spawn(async move {
        resident::run(&mut mgr, &mut rx).await;
    });
    let listen = tokio::spawn(listen(pipe_name.clone(), tx));

    let mut status = None;
    for _ in 0..50 {
        tokio::time::sleep(Duration::from_millis(20)).await;
        if let Ok(r) = request(&pipe_name, "status").await {
            status = Some(r);
            break;
        }
    }
    let status = status.expect("status attach");
    assert!(status.ok);
    assert_eq!(status.status, "Stopped");
    assert!(status.endpoint.ends_with("/v1"));

    let endpoint = request(&pipe_name, "endpoint").await.expect("endpoint");
    assert!(endpoint.ok);
    assert!(endpoint.endpoint.contains("18080"));

    let start = request(&pipe_name, "start").await.expect("start");
    assert!(!start.ok);
    assert_eq!(start.status, "Failed");
    assert!(
        start
            .error
            .as_deref()
            .unwrap_or("")
            .contains("llama-server not found"),
        "{:?}",
        start.error
    );

    listen.abort();
    owner.abort();
    let _ = std::fs::remove_dir_all(root);
}
