//! CLI entry: serve | start | stop | status | restart

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;
use tokio::sync::mpsc;
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
        "status" => match ServerManager::load_config(&root, &cfg_path) {
            Ok(mut mgr) => {
                println!("{:?}  {}", mgr.get_status(), mgr.openai_base());
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("{e}");
                ExitCode::FAILURE
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
        "stop" | "restart" => {
            eprintln!(
                "{cmd}: requires a resident manager process. Run `winserve serve`; \
                 CLI attach lands with the manager IPC. \
                 For now, stop the resident process (Ctrl+C)."
            );
            ExitCode::FAILURE
        }
        other => {
            eprintln!("unknown command: {other}");
            eprintln!("usage: winserve <serve|start|status|print-config|print-cmd>");
            ExitCode::FAILURE
        }
    }
}

/// Resident owner: holds the manager across backend stop/crash until Ctrl+C.
async fn run_serve(root: &std::path::Path, cfg_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut mgr = ServerManager::load_config(root, cfg_path)?;
    let (tx, mut rx) = mpsc::channel::<Command>(8);

    match mgr.start().await {
        Ok(()) => println!("READY {}", mgr.openai_base()),
        Err(e) => eprintln!("warning: start failed, staying resident: {e}"),
    }
    println!("serve: resident manager; Ctrl+C to stop");

    let signal = tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            let (cmd, ack) = Command::new(CommandKind::Stop);
            if tx.send(cmd).await.is_ok() {
                let _ = ack.await;
            }
        }
    });

    resident::run(&mut mgr, &mut rx).await;
    signal.abort();
    mgr.stop().await?;
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
