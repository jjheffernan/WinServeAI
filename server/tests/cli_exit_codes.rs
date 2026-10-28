//! CLI exit-code coverage for host-safe paths (no resident, no GGUF, no tray).
//!
//! Runs the `winserve` binary. Does not start llama-server.

use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_winserve"))
}

fn unique_tmp(name: &str) -> std::path::PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("winserve-cli-{name}-{n}-{}", std::process::id()))
}

fn write_minimal_config() -> std::path::PathBuf {
    let path = unique_tmp("cfg").with_extension("yaml");
    std::fs::write(
        &path,
        r#"
server:
  port: 18080
  host: 127.0.0.1
model:
  path: ""
gpu:
  auto: true
  layers: auto
runtime:
  context: 2048
  flash_attention: true
logging:
  dir: logs
  max_bytes: 1048576
  max_age_secs: 3600
  keep: 1
"#,
    )
    .expect("write config");
    path
}

#[test]
fn missing_config_exits_failure() {
    let missing = unique_tmp("missing").with_extension("yaml");
    let out = bin()
        .env("WINSERVE_CONFIG", &missing)
        .arg("print-config")
        .output()
        .expect("spawn winserve");
    assert_eq!(out.status.code(), Some(1), "stderr={}", String::from_utf8_lossy(&out.stderr));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("missing config"), "stderr={err}");
}

#[test]
fn unknown_command_exits_failure() {
    let cfg = write_minimal_config();
    let out = bin()
        .env("WINSERVE_CONFIG", &cfg)
        .arg("not-a-real-command")
        .output()
        .expect("spawn winserve");
    let _ = std::fs::remove_file(&cfg);
    assert_eq!(out.status.code(), Some(1), "stderr={}", String::from_utf8_lossy(&out.stderr));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("unknown command"), "stderr={err}");
}

#[test]
fn print_config_exits_success() {
    let cfg = write_minimal_config();
    let out = bin()
        .env("WINSERVE_CONFIG", &cfg)
        .arg("print-config")
        .output()
        .expect("spawn winserve");
    let _ = std::fs::remove_file(&cfg);
    assert!(out.status.success(), "stderr={}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("server:") || stdout.contains("port:"), "stdout={stdout}");
}

#[test]
fn status_without_resident_exits_success_stopped() {
    let cfg = write_minimal_config();
    let out = bin()
        .env("WINSERVE_CONFIG", &cfg)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .arg("status")
        .output()
        .expect("spawn winserve");
    let _ = std::fs::remove_file(&cfg);
    assert!(
        out.status.success(),
        "status without resident should succeed with Stopped; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Stopped"), "stdout={stdout}");
}

#[test]
fn stop_without_resident_exits_failure() {
    let cfg = write_minimal_config();
    let out = bin()
        .env("WINSERVE_CONFIG", &cfg)
        .arg("stop")
        .output()
        .expect("spawn winserve");
    let _ = std::fs::remove_file(&cfg);
    assert_eq!(
        out.status.code(),
        Some(1),
        "stop with no resident must fail; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
}
