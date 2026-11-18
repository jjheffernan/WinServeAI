//! CLI entry: lifecycle + doctor/debug/config/update/version.

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;
use tokio::sync::mpsc;
use winserve::cli;
use winserve::ipc::lockfile::{self, LockGuard, LockInfo};
use winserve::ipc::pipe::{self, Response};
use winserve::server::config::Config;
use winserve::server::resident::{self, Command, CommandKind};
use winserve::ServerManager;

fn repo_root() -> PathBuf {
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn config_path(root: &std::path::Path) -> PathBuf {
    env::var_os("WINSERVE_CONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("config").join("default.yaml"))
}

/// Live resident owner from the lockfile, or `None` if missing/stale.
fn live_owner() -> Result<Option<LockInfo>, Box<dyn std::error::Error>> {
    let path = lockfile::default_path();
    let _ = lockfile::remove_if_stale(&path)?;
    match lockfile::read(&path)? {
        Some(info) if lockfile::pid_alive(info.pid) => Ok(Some(info)),
        Some(_) => {
            let _ = std::fs::remove_file(&path);
            Ok(None)
        }
        None => Ok(None),
    }
}

async fn attach(cmd: &str) -> Result<Response, Box<dyn std::error::Error>> {
    let Some(info) = live_owner()? else {
        return Err("no resident manager running (start with `winserve serve` or the tray)".into());
    };
    let resp = pipe::request(&info.pipe, cmd).await?;
    Ok(resp)
}

fn print_attach(resp: &Response) {
    if let Some(err) = &resp.error {
        eprintln!("error: {err}");
    }
    println!("{}  {}", resp.status, resp.endpoint);
}

fn flag(args: &[String], name: &str) -> bool {
    args.iter().any(|a| a == name)
}

fn flag_value(args: &[String], name: &str) -> Option<String> {
    let mut i = 0;
    while i < args.len() {
        if args[i] == name {
            return args.get(i + 1).cloned();
        }
        if let Some(rest) = args[i].strip_prefix(&format!("{name}=")) {
            return Some(rest.to_string());
        }
        i += 1;
    }
    None
}

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let argv: Vec<String> = env::args().skip(1).collect();
    if argv.is_empty()
        || argv.iter().any(|a| a == "-h" || a == "--help" || a == "help")
    {
        print!("{}", cli::usage());
        return ExitCode::SUCCESS;
    }
    if argv.iter().any(|a| a == "-V" || a == "--version")
        || argv.first().map(String::as_str) == Some("version")
    {
        cli::print_version(&repo_root());
        return ExitCode::SUCCESS;
    }

    let cmd = argv[0].as_str();
    let rest = &argv[1..];
    let root = repo_root();
    let cfg_path = config_path(&root);

    // Commands that work without a config file.
    match cmd {
        "update" => {
            return cli::run_update(&root, flag(rest, "--check")).await;
        }
        "doctor" => {
            if !cfg_path.exists() {
                eprintln!("error: missing config at {}", cfg_path.display());
                eprintln!("hint: copy config/default.yaml and set model.path");
                return ExitCode::FAILURE;
            }
            return cli::run_doctor(&root, &cfg_path, flag(rest, "--fix")).await;
        }
        "debug" => {
            if !cfg_path.exists() {
                eprintln!("error: missing config at {}", cfg_path.display());
                return ExitCode::FAILURE;
            }
            let logs = flag_value(rest, "--logs")
                .and_then(|s| s.parse().ok())
                .or_else(|| if flag(rest, "--logs") { Some(40) } else { None });
            return cli::run_debug(
                &root,
                &cfg_path,
                logs,
                flag(rest, "--fix-lock"),
                flag(rest, "--probe"),
            )
            .await;
        }
        _ => {}
    }

    if !cfg_path.exists() {
        eprintln!("error: missing config at {}", cfg_path.display());
        eprintln!("hint: copy config/default.yaml and set model.path");
        eprintln!("hint: override with WINSERVE_CONFIG=/path/to.yaml");
        eprintln!("hint: winserve version | update | help work without config");
        return ExitCode::FAILURE;
    }

    match cmd {
        "serve" => match run_serve(&root, &cfg_path).await {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                let msg = e.to_string();
                if msg.starts_with("already running") {
                    eprintln!("error: {msg}");
                    eprintln!("hint: use `winserve status|stop|restart` against the resident owner");
                } else {
                    eprintln!("error: serve failed: {msg}");
                }
                ExitCode::FAILURE
            }
        },
        "start" => match live_owner() {
            Ok(Some(_)) => match attach("start").await {
                Ok(resp) => {
                    print_attach(&resp);
                    if resp.ok {
                        ExitCode::SUCCESS
                    } else {
                        ExitCode::FAILURE
                    }
                }
                Err(e) => {
                    eprintln!("error: {e}");
                    ExitCode::FAILURE
                }
            },
            Ok(None) => match run_start(&root, &cfg_path).await {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("error: start failed: {e}");
                    eprintln!("hint: winserve doctor");
                    ExitCode::FAILURE
                }
            },
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::FAILURE
            }
        },
        "status" => match attach("status").await {
            Ok(resp) => {
                print_attach(&resp);
                if resp.ok {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::FAILURE
                }
            }
            Err(e) => match ServerManager::load_config(&root, &cfg_path) {
                Ok(mgr) => {
                    eprintln!("note: {e}");
                    println!("Stopped  {}", mgr.openai_base());
                    ExitCode::SUCCESS
                }
                Err(cfg_err) => {
                    eprintln!("error: {e}");
                    eprintln!("{cfg_err}");
                    ExitCode::FAILURE
                }
            },
        },
        "config" | "print-config" | "print-cmd" => {
            if cmd == "print-cmd" || flag(rest, "--cmd") {
                return cli::run_config_cmd(&root, &cfg_path);
            }
            if rest.first().map(String::as_str) == Some("set") {
                let key = match rest.get(1) {
                    Some(k) => k.as_str(),
                    None => {
                        eprintln!("usage: winserve config set <key> <value> [--apply]");
                        return ExitCode::FAILURE;
                    }
                };
                let value = match rest.get(2) {
                    Some(v) => v.as_str(),
                    None => {
                        eprintln!("usage: winserve config set <key> <value> [--apply]");
                        return ExitCode::FAILURE;
                    }
                };
                return cli::run_config_set(
                    &root,
                    &cfg_path,
                    key,
                    value,
                    flag(rest, "--apply"),
                )
                .await;
            }
            cli::run_config_show(&cfg_path, flag(rest, "--path"), flag(rest, "--validate"))
        }
        "tui" => match run_tui(&root, &cfg_path).await {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("error: tui failed: {e}");
                ExitCode::FAILURE
            }
        },
        "stop" | "restart" | "reload" => match attach(cmd).await {
            Ok(resp) => {
                print_attach(&resp);
                if resp.ok {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::FAILURE
                }
            }
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::FAILURE
            }
        },
        other => {
            eprintln!("unknown command: {other}");
            eprint!("{}", cli::usage());
            ExitCode::FAILURE
        }
    }
}

/// Resident owner: holds the manager across backend stop/crash until Ctrl+C.
async fn run_serve(root: &std::path::Path, cfg_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let (lock, info) = LockGuard::acquire_default(lockfile::DEFAULT_PIPE_NAME)?;
    println!(
        "serve: lock {} (pid={}, pipe={})",
        lock.path().display(),
        info.pid,
        info.pipe
    );

    let mut mgr = ServerManager::load_config(root, cfg_path)?;
    let (tx, mut rx) = mpsc::channel::<Command>(8);
    let ipc = tokio::spawn(pipe::listen(info.pipe.clone(), tx.clone()));
    println!("serve: ipc {}", pipe::endpoint_for(&info.pipe));

    match mgr.start().await {
        Ok(()) => println!("READY {}", mgr.openai_base()),
        Err(e) => eprintln!("warning: start failed, staying resident: {e}"),
    }
    println!("serve: resident manager; Ctrl+C to stop");

    let signal_tx = tx.clone();
    let mut signal = tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            let (cmd, ack) = Command::new(CommandKind::Stop);
            if signal_tx.send(cmd).await.is_ok() {
                let _ = ack.await;
            }
        }
    });
    drop(tx);

    tokio::select! {
        _ = resident::run(&mut mgr, &mut rx) => {}
        _ = &mut signal => {}
    }
    ipc.abort();
    signal.abort();
    let _ = mgr.stop().await;
    drop(lock);
    Ok(())
}

/// Terminal monitor. Owns the manager when nothing else does; otherwise attaches.
async fn run_tui(root: &std::path::Path, cfg_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let cfg = Config::load(cfg_path)?;
    let info = winserve::tui::ScreenInfo {
        model: cfg.model.path.display().to_string(),
        context: cfg.runtime.context,
        gpu_layers: cfg.gpu.layers.clone(),
        log_dir: if cfg.logging.dir.is_absolute() {
            cfg.logging.dir.clone()
        } else {
            root.join(&cfg.logging.dir)
        },
        owns: false,
    };

    if let Some(owner) = live_owner()? {
        return lift_tui(winserve::tui::run(None, Some(owner.pipe), info).await);
    }

    let (lock, lock_info) = match LockGuard::acquire_default(lockfile::DEFAULT_PIPE_NAME) {
        Ok(pair) => pair,
        Err(winserve::ipc::lockfile::LockError::AlreadyRunning { pipe, .. }) => {
            return lift_tui(winserve::tui::run(None, Some(pipe), info).await);
        }
        Err(e) => return Err(e.into()),
    };

    let mut mgr = ServerManager::load_config(root, cfg_path)?;
    let (tx, mut rx) = mpsc::channel::<Command>(8);
    let ipc = tokio::spawn(pipe::listen(lock_info.pipe.clone(), tx.clone()));
    let resident = tokio::spawn(async move {
        resident::run(&mut mgr, &mut rx).await;
    });
    let mut owned = info;
    owned.owns = true;
    let result = winserve::tui::run(Some(tx.clone()), None, owned).await;
    let (cmd, ack) = Command::new(CommandKind::Stop);
    if tx.send(cmd).await.is_ok() {
        let _ = ack.await;
    }
    drop(tx);
    let _ = resident.await;
    ipc.abort();
    drop(lock);
    lift_tui(result)
}

fn lift_tui(result: Result<(), String>) -> Result<(), Box<dyn std::error::Error>> {
    result.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e).into())
}

async fn run_start(root: &std::path::Path, cfg_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut mgr = ServerManager::load_config(root, cfg_path)?;
    mgr.start().await?;
    println!("READY {}", mgr.openai_base());
    println!("Press Ctrl+C to stop");

    tokio::signal::ctrl_c().await?;
    mgr.stop().await?;
    Ok(())
}
