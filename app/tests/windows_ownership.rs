//! Ownership proofs: single-instance lock, manager drop reaps backend, resident attach.
//!
//! Windows CI executes the `#[cfg(windows)]` cases (named pipe + Job Object).
//! Host-safe cases run everywhere via the fake llama-server helper.
//! Real GGUF A2 remains an operator gate (`tests/windows/a2-smoke.ps1`).

use std::net::TcpListener;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use winserve::ipc::lockfile;
use winserve::ipc::pipe::{listen, request};
use winserve::server::config::Config;
use winserve::server::manager::{ServerManager, Status};
use winserve::server::resident::{self, Command};
use tokio::sync::mpsc;

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn fixture_root(tag: &str) -> (PathBuf, PathBuf) {
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("winserve-own-{tag}-{n}"));
    std::fs::create_dir_all(root.join("bin")).unwrap();
    std::fs::create_dir_all(root.join("config")).unwrap();
    std::fs::create_dir_all(root.join("logs")).unwrap();

    let src = PathBuf::from(env!("CARGO_BIN_EXE_fake-llama-server"));
    let dst_name = if cfg!(windows) {
        "llama-server.exe"
    } else {
        "llama-server"
    };
    let dst = root.join("bin").join(dst_name);
    std::fs::copy(&src, &dst).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&dst).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&dst, perms).unwrap();
    }
    std::fs::write(root.join("bin").join("FAKE_BEHAVIOR"), "ready").unwrap();

    let gguf = root.join("model.gguf");
    std::fs::write(&gguf, b"fake-gguf").unwrap();
    let cfg_path = root.join("config").join("default.yaml");
    let mut cfg = Config::default();
    cfg.logging.dir = root.join("logs");
    cfg.model.path = gguf;
    cfg.server.host = "127.0.0.1".into();
    cfg.server.port = free_port();
    cfg.save(&cfg_path).unwrap();
    (root, cfg_path)
}

#[tokio::test]
async fn lockfile_second_acquire_fails_while_held() {
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("winserve-lock2-{n}.lock"));
    let _ = std::fs::remove_file(&path);
    let first = lockfile::acquire(&path, "pipe-a").expect("first owner");
    let err = lockfile::acquire(&path, "pipe-b").expect_err("second must fail");
    assert!(
        matches!(err, lockfile::LockError::AlreadyRunning { .. }),
        "{err}"
    );
    lockfile::release(&path).unwrap();
    let _ = first;
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn dropping_ready_manager_reaps_backend() {
    let (root, cfg_path) = fixture_root("drop");
    let port = {
        let mut mgr = ServerManager::load_config(&root, &cfg_path).unwrap();
        mgr.set_ready_timeout(Duration::from_secs(5));
        mgr.start().await.expect("start");
        assert_eq!(mgr.status(), Status::Ready);
        let port = mgr.config().server.port;
        // Drop without stop — Job Object / kill_on_drop must reap the child.
        drop(mgr);
        port
    };
    tokio::time::sleep(Duration::from_millis(400)).await;
    let url = format!("http://127.0.0.1:{port}/v1/models");
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(500))
        .build()
        .unwrap();
    let still_up = client.get(&url).send().await.is_ok();
    assert!(!still_up, "backend still answering after manager drop");
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn resident_attach_start_stop_roundtrip() {
    let (root, cfg_path) = fixture_root("attach");
    let mut mgr = ServerManager::load_config(&root, &cfg_path).unwrap();
    mgr.set_ready_timeout(Duration::from_secs(5));
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let pipe_name = format!("winserve-own-attach-{n}");
    let (tx, mut rx) = mpsc::channel::<Command>(8);
    let owner = tokio::spawn(async move {
        resident::run(&mut mgr, &mut rx).await;
    });
    let listen = tokio::spawn(listen(pipe_name.clone(), tx));

    let mut ready = None;
    for _ in 0..50 {
        tokio::time::sleep(Duration::from_millis(20)).await;
        if let Ok(r) = request(&pipe_name, "status").await {
            ready = Some(r);
            break;
        }
    }
    assert!(ready.expect("status").ok);

    let started = request(&pipe_name, "start").await.expect("start");
    assert!(started.ok, "{:?}", started.error);
    assert_eq!(started.status, "Ready");

    let stopped = request(&pipe_name, "stop").await.expect("stop");
    assert!(stopped.ok, "{:?}", stopped.error);
    assert_eq!(stopped.status, "Stopped");

    listen.abort();
    owner.abort();
    let _ = std::fs::remove_dir_all(root);
}

/// Windows-only: named-pipe transport is used by `listen`/`request` under cfg(windows).
#[cfg(windows)]
#[tokio::test]
async fn windows_named_pipe_status_attach() {
    let (root, cfg_path) = fixture_root("winpipe");
    let mut mgr = ServerManager::load_config(&root, &cfg_path).unwrap();
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let pipe_name = format!("winserve-winpipe-{n}");
    let (tx, mut rx) = mpsc::channel::<Command>(4);
    let owner = tokio::spawn(async move {
        resident::run(&mut mgr, &mut rx).await;
    });
    let listen = tokio::spawn(listen(pipe_name.clone(), tx));
    let mut resp = None;
    for _ in 0..80 {
        tokio::time::sleep(Duration::from_millis(25)).await;
        if let Ok(r) = request(&pipe_name, "endpoint").await {
            resp = Some(r);
            break;
        }
    }
    let resp = resp.expect("named pipe endpoint");
    assert!(resp.ok);
    assert!(resp.endpoint.contains("/v1"));
    listen.abort();
    owner.abort();
    let _ = std::fs::remove_dir_all(root);
}
