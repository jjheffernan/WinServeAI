//! CLI entry: start | stop | status | restart

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;
use winserve::server::config::Config;
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
        eprintln!("missing config at {}", cfg_path.display());
        eprintln!("copy config/default.yaml and set model.path");
        return ExitCode::FAILURE;
    }

    match cmd.as_str() {
        "start" => match run_start(&root, &cfg_path).await {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("start failed: {e}");
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
                "{cmd}: requires a long-running manager process (tray/service). \
                 For MVP, stop the `winserve start` process (Ctrl+C)."
            );
            ExitCode::FAILURE
        }
        other => {
            eprintln!("unknown command: {other}");
            eprintln!("usage: winserve <start|status|print-config|print-cmd>");
            ExitCode::FAILURE
        }
    }
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
