//! ServerManager lifecycle against a controllable `fake-llama-server` binary.
//!
//! No backend trait: the fake is installed as `{root}/bin/llama-server[.exe]`.

use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use winserve::server::config::Config;
use winserve::server::manager::{ServerManager, Status};

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
    let root = std::env::temp_dir().join(format!("winserve-life-{tag}-{n}"));
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
    std::fs::copy(&src, &dst).unwrap_or_else(|e| {
        panic!("copy {} → {}: {e}", src.display(), dst.display())
    });
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&dst).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&dst, perms).unwrap();
    }

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

fn write_behavior(root: &Path, behavior: &str) {
    std::fs::write(root.join("bin").join("FAKE_BEHAVIOR"), behavior).unwrap();
}

#[tokio::test]
async fn start_ready_stop_and_restart() {
    let (root, cfg_path) = fixture_root("ready");
    write_behavior(&root, "ready");
    let mut mgr = ServerManager::load_config(&root, &cfg_path).unwrap();
    mgr.set_ready_timeout(Duration::from_secs(5));

    mgr.start().await.expect("start ready");
    assert_eq!(mgr.status(), Status::Ready);

    mgr.stop().await.expect("stop");
    assert_eq!(mgr.status(), Status::Stopped);

    mgr.restart().await.expect("restart");
    assert_eq!(mgr.status(), Status::Ready);
    mgr.stop().await.expect("final stop");
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn start_times_out_when_never_ready() {
    let (root, cfg_path) = fixture_root("never");
    write_behavior(&root, "never_ready");
    let mut mgr = ServerManager::load_config(&root, &cfg_path).unwrap();
    mgr.set_ready_timeout(Duration::from_millis(600));

    let err = mgr.start().await.unwrap_err().to_string();
    assert!(
        err.to_lowercase().contains("timeout") || err.contains("not ready"),
        "got {err}"
    );
    assert_eq!(mgr.status(), Status::Failed);
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn crash_detected_after_ready() {
    let (root, cfg_path) = fixture_root("crash");
    write_behavior(&root, "exit_after_ready");
    let mut mgr = ServerManager::load_config(&root, &cfg_path).unwrap();
    mgr.set_ready_timeout(Duration::from_secs(5));

    mgr.start().await.expect("start before crash");
    assert_eq!(mgr.status(), Status::Ready);

    let mut crashed = false;
    for _ in 0..40 {
        tokio::time::sleep(Duration::from_millis(50)).await;
        if mgr.get_status() == Status::Crashed {
            crashed = true;
            break;
        }
    }
    assert!(crashed, "expected Crashed after fake backend exit");
    let _ = std::fs::remove_dir_all(root);
}
