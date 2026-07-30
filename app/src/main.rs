//! CLI entry: serve | start | stop | status | restart

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;
use tokio::sync::mpsc;
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
        return Err("no resident manager running (start with `winserve serve`)".into());
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

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let mut args = env::args().skip(1);
    let cmd = args.next().unwrap_or_else(|| "start".into());
    let root = repo_root();
    let cfg_path = config_path(&root);

    if !cfg_path.exists() {
        eprintln!("error: missing config at {}", cfg_path.display());
        eprintln!("hint: copy config/default.yaml and set model.path");
        eprintln!("hint: override with WINSERVE_CONFIG=/path/to.yaml");
        return ExitCode::FAILURE;
    }

    match cmd.as_str() {
        "serve" => match run_serve(&root, &cfg_path).await {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("error: serve failed: {e}");
                ExitCode::FAILURE
            }
        },
        "start" => match run_start(&root, &cfg_path).await {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("error: start failed: {e}");
                eprintln!("hint: place pinned llama-server in bin/ (see bin/README.md)");
                eprintln!("hint: set model.path in config to an existing .gguf file");
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
            Err(e) => {
                // No resident: report config endpoint + Stopped (not an attach failure).
                match ServerManager::load_config(&root, &cfg_path) {
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
                }
            }
        },
        "print-config" => match Config::load(&cfg_path) {
            Ok(c) => {
                println!("{}", serde_yaml::to_string(&c).unwrap_or_default());
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("{e}");
                ExitCode::FAILURE
            }
        },
        "print-cmd" => match ServerManager::load_config(&root, &cfg_path) {
            Ok(mgr) => {
                let (bin, argv) = mgr.build_command();
                println!("{} {}", bin.display(), argv.join(" "));
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("{e}");
                ExitCode::FAILURE
            }
        },
        "stop" | "restart" => match attach(cmd.as_str()).await {
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
            eprintln!("usage: winserve <serve|start|stop|restart|status|print-config|print-cmd>");
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

async fn run_start(root: &std::path::Path, cfg_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut mgr = ServerManager::load_config(root, cfg_path)?;
    mgr.start().await?;
    println!("READY {}", mgr.openai_base());
    println!("Press Ctrl+C to stop");

    tokio::signal::ctrl_c().await?;
    mgr.stop().await?;
    Ok(())
}
