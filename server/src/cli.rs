//! Operator CLI helpers: doctor, debug, config, update, version.
//!
//! Keep these as thin probes over `ServerManager` / filesystem — no second orchestrator.

use crate::ipc::lockfile;
use crate::server::config::{is_loopback_host, is_wildcard_bind, Config};
use crate::system::network;
use crate::ServerManager;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

pub const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn usage() -> &'static str {
    "usage: winserve <command> [flags]

Lifecycle:
  serve                 Resident manager (lockfile + IPC); Ctrl+C to quit
  start                 Start backend (attach if resident, else foreground)
  stop                  Stop backend via resident IPC
  restart               Restart backend via resident IPC (no reinstall)
  status                Status + endpoint
  tui                   Terminal monitor

Inspect / fix (no reinstall):
  version | --version   App version + llama.cpp pin
  doctor [--fix]       Installation + runtime health checklist
  debug [--logs N] [--fix-lock] [--probe]
                        Snapshot, tail logs, clear stale lock, probe /v1/models
  config [--path|--validate|--cmd]
  config set <key> <value> [--apply]
                        Edit YAML (host/port/api_key/model.path/…).
                        --apply: stop if needed, save, start again
  update [--check]      Show upgrade path; --check hits GitHub latest tag
  print-config          Alias for config
  print-cmd             Alias for config --cmd

Env: WINSERVE_CONFIG=/path/to.yaml
"
}

pub fn print_version(root: &Path) {
    println!("winserve {CRATE_VERSION}");
    let pin_path = root.join("bin").join("VERSION");
    match fs::read_to_string(&pin_path) {
        Ok(pin) => println!("llama.cpp pin {}", pin.trim()),
        Err(_) => println!("llama.cpp pin (missing bin/VERSION)"),
    }
}

fn llama_bin(root: &Path) -> PathBuf {
    if cfg!(windows) {
        root.join("bin").join("llama-server.exe")
    } else {
        root.join("bin").join("llama-server")
    }
}

fn read_pin(root: &Path) -> Option<String> {
    fs::read_to_string(root.join("bin").join("VERSION"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn tail_file(path: &Path, max_lines: usize) -> Vec<String> {
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    text.lines()
        .rev()
        .take(max_lines)
        .map(str::to_string)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

fn check_line(ok: bool, label: &str, detail: &str) {
    let mark = if ok { "ok" } else { "FAIL" };
    println!("[{mark}] {label}: {detail}");
}

/// Installation + runtime checklist. Exit 0 when no FAIL lines.
pub async fn run_doctor(root: &Path, cfg_path: &Path, fix: bool) -> ExitCode {
    let mut failed = 0u32;
    println!("winserve doctor (root={})", root.display());
    println!("config: {}", cfg_path.display());

    // Config
    let cfg = match Config::load(cfg_path) {
        Ok(c) => {
            check_line(true, "config", "loads and validates");
            for w in c.warnings() {
                println!("[warn] config: {w}");
            }
            Some(c)
        }
        Err(e) => {
            check_line(false, "config", &e.to_string());
            failed += 1;
            None
        }
    };

    // Binary + pin
    let bin = llama_bin(root);
    if bin.is_file() {
        check_line(true, "llama-server", &bin.display().to_string());
    } else {
        check_line(
            false,
            "llama-server",
            &format!("missing {} (see bin/README.md / winserve update)", bin.display()),
        );
        failed += 1;
    }
    match read_pin(root) {
        Some(pin) => check_line(true, "pin", &pin),
        None => {
            check_line(false, "pin", "bin/VERSION missing");
            failed += 1;
        }
    }

    // Model
    if let Some(ref c) = cfg {
        let p = &c.model.path;
        if p.as_os_str().is_empty() {
            check_line(false, "model.path", "empty — set a local .gguf (config set model.path …)");
            failed += 1;
        } else if p.is_file() {
            check_line(true, "model.path", &p.display().to_string());
        } else {
            check_line(false, "model.path", &format!("not found: {}", p.display()));
            failed += 1;
        }

        // Bind / key
        if is_wildcard_bind(&c.server.host) {
            check_line(
                true,
                "bind",
                &format!(
                    "{} (wildcard; clients use {})",
                    c.server.host,
                    c.client_host()
                ),
            );
        } else {
            check_line(true, "bind", &format!("{}:{}", c.server.host, c.server.port));
        }
        if !is_loopback_host(&c.server.host) && !is_wildcard_bind(&c.server.host) {
            // Specific interface IP — remind about WSL/DHCP drift
            println!(
                "[warn] bind: specific interface IP — if peers cannot connect after reboot/update, prefer 0.0.0.0 + api_key (see docs/ops-resilience.md)"
            );
        }
        if !is_loopback_host(&c.server.host) && c.server.api_key.trim().is_empty() {
            check_line(false, "api_key", "empty while bind is not loopback");
            failed += 1;
        } else if c.server.api_key.trim().is_empty() {
            check_line(true, "api_key", "empty (ok on loopback)");
        } else {
            check_line(true, "api_key", "set");
        }

        // Port
        let free = network::port_available(&c.server.host, c.server.port);
        let lock = lockfile::default_path();
        let owner = match lockfile::read(&lock) {
            Ok(Some(info)) if lockfile::pid_alive(info.pid) => Some(info),
            Ok(Some(_)) => {
                check_line(false, "lockfile", &format!("stale at {}", lock.display()));
                if fix {
                    match lockfile::remove_if_stale(&lock) {
                        Ok(true) => {
                            println!("[fix] lockfile: removed stale lock");
                        }
                        Ok(false) => {}
                        Err(e) => println!("[FAIL] lockfile fix: {e}"),
                    }
                } else {
                    println!("[hint] winserve debug --fix-lock");
                    failed += 1;
                }
                None
            }
            Ok(None) => None,
            Err(e) => {
                check_line(false, "lockfile", &e.to_string());
                failed += 1;
                None
            }
        };
        if let Some(info) = &owner {
            check_line(
                true,
                "resident",
                &format!("pid={} pipe={}", info.pid, info.pipe),
            );
            if free {
                println!("[warn] port: appears free while resident is up (probe may still work)");
            } else {
                check_line(
                    true,
                    "port",
                    &format!("{}:{} in use (expected while serving)", c.server.host, c.server.port),
                );
            }
        } else {
            check_line(true, "resident", "none");
            if free {
                check_line(
                    true,
                    "port",
                    &format!("{}:{} available", c.server.host, c.server.port),
                );
            } else {
                check_line(
                    false,
                    "port",
                    &format!(
                        "{}:{} busy (stop the other listener or change server.port)",
                        c.server.host, c.server.port
                    ),
                );
                failed += 1;
            }
        }

        // Live API probe when something is listening
        if !free || owner.is_some() {
            let url = format!("{}/models", c.openai_v1_url());
            match reqwest::Client::new()
                .get(&url)
                .timeout(Duration::from_secs(3))
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    check_line(true, "GET /v1/models", &format!("{} -> {}", url, resp.status()));
                }
                Ok(resp) => {
                    check_line(
                        false,
                        "GET /v1/models",
                        &format!("{} -> {}", url, resp.status()),
                    );
                    failed += 1;
                }
                Err(e) => {
                    if owner.is_some() {
                        check_line(false, "GET /v1/models", &e.to_string());
                        failed += 1;
                    } else {
                        println!("[warn] GET /v1/models: {e}");
                    }
                }
            }
        }

        // Logs
        let log_dir = if c.logging.dir.is_absolute() {
            c.logging.dir.clone()
        } else {
            root.join(&c.logging.dir)
        };
        if log_dir.is_dir() {
            check_line(true, "logs", &log_dir.display().to_string());
            let err_log = log_dir.join("error.log");
            if err_log.is_file() {
                let lines = tail_file(&err_log, 5);
                if !lines.is_empty() {
                    println!("--- error.log (last {}) ---", lines.len());
                    for line in lines {
                        println!("{line}");
                    }
                }
            }
        } else {
            println!("[warn] logs: {} missing (created on start)", log_dir.display());
        }
    }

    if failed == 0 {
        println!("doctor: all checks passed");
        ExitCode::SUCCESS
    } else {
        println!("doctor: {failed} check(s) failed — fix above, then retry (no reinstall needed)");
        println!("hint: winserve debug --logs 40");
        println!("hint: winserve config set model.path <file.gguf> --apply");
        ExitCode::FAILURE
    }
}

pub async fn run_debug(
    root: &Path,
    cfg_path: &Path,
    logs: Option<usize>,
    fix_lock: bool,
    probe: bool,
) -> ExitCode {
    println!("winserve debug {CRATE_VERSION}");
    println!("root:   {}", root.display());
    println!("config: {}", cfg_path.display());

    let lock = lockfile::default_path();
    match lockfile::read(&lock) {
        Ok(Some(info)) if lockfile::pid_alive(info.pid) => {
            println!("lock:   {} (live pid={} pipe={})", lock.display(), info.pid, info.pipe);
        }
        Ok(Some(info)) => {
            println!(
                "lock:   {} (STALE pid={} pipe={})",
                lock.display(),
                info.pid,
                info.pipe
            );
            if fix_lock {
                match lockfile::remove_if_stale(&lock) {
                    Ok(true) => println!("lock:   removed stale"),
                    Ok(false) => println!("lock:   still present"),
                    Err(e) => {
                        eprintln!("error: {e}");
                        return ExitCode::FAILURE;
                    }
                }
            } else {
                println!("hint:   winserve debug --fix-lock");
            }
        }
        Ok(None) => println!("lock:   none"),
        Err(e) => println!("lock:   error {e}"),
    }

    let cfg = match Config::load(cfg_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("config: {e}");
            return ExitCode::FAILURE;
        }
    };
    println!("bind:   {}:{}", cfg.server.host, cfg.server.port);
    println!("client: {}", cfg.openai_v1_url());
    println!("model:  {}", cfg.model.path.display());
    println!("pin:    {}", read_pin(root).unwrap_or_else(|| "(missing)".into()));
    println!("binary: {}", llama_bin(root).display());

    if let Ok(mut mgr) = ServerManager::load_config(root, cfg_path) {
        mgr.detect_hardware();
        let (bin, argv) = mgr.build_command();
        println!("argv:   {} {}", bin.display(), argv.join(" "));
        let hw = mgr.hardware();
        println!(
            "hw:     {} gpu(s), {} threads",
            hw.gpus.len(),
            hw.cpu_threads
        );
        for (i, g) in hw.gpus.iter().enumerate() {
            println!("  gpu[{i}] {}  vram_mb={}", g.name, g.vram_mb);
        }
    }

    if probe {
        let url = format!("{}/models", cfg.openai_v1_url());
        match reqwest::Client::new()
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
        {
            Ok(resp) => println!("probe:  GET {url} -> {}", resp.status()),
            Err(e) => println!("probe:  GET {url} failed: {e}"),
        }
    }

    if let Some(n) = logs {
        let log_dir = if cfg.logging.dir.is_absolute() {
            cfg.logging.dir.clone()
        } else {
            root.join(&cfg.logging.dir)
        };
        for name in ["error.log", "server.log", "llama.log"] {
            let path = log_dir.join(name);
            let lines = tail_file(&path, n);
            println!("--- {} (last {}) ---", path.display(), lines.len());
            if lines.is_empty() {
                println!("(empty or missing)");
            } else {
                for line in lines {
                    println!("{line}");
                }
            }
        }
    }

    ExitCode::SUCCESS
}

pub fn run_config_show(cfg_path: &Path, path_only: bool, validate: bool) -> ExitCode {
    if path_only {
        println!("{}", cfg_path.display());
        return ExitCode::SUCCESS;
    }
    match Config::load(cfg_path) {
        Ok(c) => {
            if validate {
                println!("ok: {}", cfg_path.display());
                for w in c.warnings() {
                    println!("warning: {w}");
                }
            } else {
                println!("{}", serde_yaml::to_string(&c).unwrap_or_default());
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

pub fn run_config_cmd(root: &Path, cfg_path: &Path) -> ExitCode {
    match ServerManager::load_config(root, cfg_path) {
        Ok(mut mgr) => {
            mgr.detect_hardware();
            let (bin, argv) = mgr.build_command();
            println!("{} {}", bin.display(), argv.join(" "));
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn set_field(cfg: &mut Config, key: &str, value: &str) -> Result<(), String> {
    match key {
        "host" | "server.host" => cfg.server.host = value.to_string(),
        "port" | "server.port" => {
            cfg.server.port = value
                .parse()
                .map_err(|_| format!("port must be u16, got {value:?}"))?
        }
        "api_key" | "server.api_key" => cfg.server.api_key = value.to_string(),
        "model.path" | "model" => cfg.model.path = PathBuf::from(value),
        "gpu.layers" | "layers" => cfg.gpu.layers = value.to_string(),
        "gpu.auto" => {
            cfg.gpu.auto = match value {
                "true" | "1" | "yes" => true,
                "false" | "0" | "no" => false,
                _ => return Err(format!("gpu.auto must be true/false, got {value:?}")),
            }
        }
        "runtime.context" | "context" => {
            cfg.runtime.context = value
                .parse()
                .map_err(|_| format!("context must be u32, got {value:?}"))?
        }
        "runtime.flash_attention" | "flash_attention" => {
            cfg.runtime.flash_attention = match value {
                "true" | "1" | "yes" => true,
                "false" | "0" | "no" => false,
                _ => {
                    return Err(format!(
                        "flash_attention must be true/false, got {value:?}"
                    ))
                }
            }
        }
        other => {
            return Err(format!(
                "unknown key {other:?} (host|port|api_key|model.path|gpu.layers|gpu.auto|runtime.context|runtime.flash_attention)"
            ))
        }
    }
    cfg.validate().map_err(|e| e.to_string())
}

/// Edit config on disk. With `apply`, stop → save → start when a resident is up.
pub async fn run_config_set(
    _root: &Path,
    cfg_path: &Path,
    key: &str,
    value: &str,
    apply: bool,
) -> ExitCode {
    let mut cfg = match Config::load(cfg_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(e) = set_field(&mut cfg, key, value) {
        eprintln!("error: {e}");
        return ExitCode::FAILURE;
    }

    if !apply {
        if let Err(e) = cfg.save(cfg_path) {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
        println!("saved {} = {:?} -> {}", key, value, cfg_path.display());
        println!("hint: if the backend is Ready, run `winserve restart` (or config set … --apply)");
        return ExitCode::SUCCESS;
    }

    // --apply: prefer resident restart path without reinstall
    let owner_alive = matches!(
        lockfile::read(&lockfile::default_path()),
        Ok(Some(info)) if lockfile::pid_alive(info.pid)
    );

    if owner_alive {
        // stop → write YAML → reload in-memory → start (no reinstall, tray stays up)
        if let Err(e) = ipc("stop").await {
            eprintln!("error: stop before apply failed: {e}");
            return ExitCode::FAILURE;
        }
        if let Err(e) = cfg.save(cfg_path) {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
        println!("saved {} = {:?}", key, value);
        if let Err(e) = ipc("reload").await {
            eprintln!("error: reload failed: {e}");
            eprintln!("hint: YAML is on disk; quit tray/serve once, then start again");
            return ExitCode::FAILURE;
        }
        match ipc("start").await {
            Ok(()) => {
                println!("applied: reloaded config and started");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("error: start after apply failed: {e}");
                eprintln!("hint: winserve doctor");
                ExitCode::FAILURE
            }
        }
    } else {
        if let Err(e) = cfg.save(cfg_path) {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
        println!("saved {} = {:?} -> {}", key, value, cfg_path.display());
        println!("hint: winserve start   # or open the tray");
        ExitCode::SUCCESS
    }
}

async fn ipc(cmd: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = lockfile::default_path();
    let _ = lockfile::remove_if_stale(&path)?;
    let info = match lockfile::read(&path)? {
        Some(info) if lockfile::pid_alive(info.pid) => info,
        _ => return Err("no resident manager running".into()),
    };
    let resp = crate::ipc::pipe::request(&info.pipe, cmd).await?;
    if let Some(err) = resp.error {
        return Err(err.into());
    }
    Ok(())
}

pub async fn run_update(root: &Path, check: bool) -> ExitCode {
    print_version(root);
    let pin = read_pin(root).unwrap_or_else(|| "(unknown)".into());
    println!();
    println!("Upgrade (no need to wipe config):");
    println!("  1. Download the latest WinServeAI-*-setup.exe from");
    println!("     https://github.com/jjheffernan/WinServeAI/releases");
    println!("  2. Run the installer — config\\default.yaml is kept (onlyifdoesntexist).");
    println!("  3. winserve doctor");
    println!();
    println!("Pin only (llama-server):");
    println!("  .\\scripts\\fetch-llama-pin.ps1 -Pin {pin} -Variant vulkan");
    println!("  Current pin file: {}", root.join("bin").join("VERSION").display());

    if check {
        println!();
        println!("Checking GitHub latest release…");
        match reqwest::Client::new()
            .get("https://api.github.com/repos/jjheffernan/WinServeAI/releases/latest")
            .header("User-Agent", format!("winserve/{CRATE_VERSION}"))
            .header("Accept", "application/vnd.github+json")
            .timeout(Duration::from_secs(10))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                match resp.json::<serde_json::Value>().await {
                    Ok(v) => {
                        let tag = v
                            .get("tag_name")
                            .and_then(|t| t.as_str())
                            .unwrap_or("(none)");
                        println!("latest release tag: {tag}");
                        println!("this build:         v{CRATE_VERSION}");
                        if tag.trim_start_matches('v') == CRATE_VERSION {
                            println!("update: you are on the latest tagged version");
                        } else {
                            println!("update: a newer (or different) tag is published — install that setup");
                        }
                    }
                    Err(e) => eprintln!("error: parse release json: {e}"),
                }
            }
            Ok(resp) => eprintln!("error: GitHub API {}", resp.status()),
            Err(e) => eprintln!("error: {e}"),
        }
    }
    ExitCode::SUCCESS
}
