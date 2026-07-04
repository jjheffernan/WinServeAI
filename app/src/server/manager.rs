//! ServerManager — the only orchestration API.
//!
//! ```text
//! load_config → detect_hardware → build_command → start → wait ready → READY
//! stop: signal → wait → force kill → flush logs
//! ```

use crate::runtime::llama::{build_command, default_binary};
use crate::runtime::process::{ChildProcess, ProcessError};
use crate::server::config::{Config, ConfigError};
use crate::server::health::{self, HealthError};
use crate::server::logs::{LogError, LogSinks};
use crate::system::{self, network, HardwareInfo};
use std::path::{Path, PathBuf};
use std::time::Duration;
use thiserror::Error;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Stopped,
    Starting,
    Ready,
    Stopping,
    Failed,
    Crashed,
}

#[derive(Debug, Error)]
pub enum ManagerError {
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Process(#[from] ProcessError),
    #[error(transparent)]
    Health(#[from] HealthError),
    #[error(transparent)]
    Log(#[from] LogError),
    #[error("{0}")]
    Other(String),
}

pub struct ServerManager {
    root: PathBuf,
    config: Config,
    hardware: HardwareInfo,
    binary: PathBuf,
    logs: LogSinks,
    child: Option<ChildProcess>,
    status: Status,
    last_exit: Option<i32>,
}

impl ServerManager {
    pub fn load_config(root: impl AsRef<Path>, config_path: impl AsRef<Path>) -> Result<Self, ManagerError> {
        let root = root.as_ref().to_path_buf();
        let config = Config::load(config_path)?;
        let hardware = system::detect();
        let binary = default_binary(&root);
        let logs = LogSinks::open(&config.logging.dir)?;
        logs.server(&format!("config loaded; binary={}", binary.display()));
        for w in config.warnings() {
            logs.server(&format!("warning: {w}"));
        }
        Ok(Self {
            root,
            config,
            hardware,
            binary,
            logs,
            child: None,
            status: Status::Stopped,
            last_exit: None,
        })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn hardware(&self) -> &HardwareInfo {
        &self.hardware
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn detect_hardware(&mut self) {
        self.hardware = system::detect();
        self.logs.server(&format!(
            "hardware: {} gpu(s), {} threads",
            self.hardware.gpus.len(),
            self.hardware.cpu_threads
        ));
    }

    pub fn build_command(&self) -> (PathBuf, Vec<String>) {
        build_command(&self.config, &self.hardware, &self.binary)
    }

    pub async fn start(&mut self) -> Result<(), ManagerError> {
        if self.child.is_some() {
            return Err(ProcessError::AlreadyRunning.into());
        }

        self.status = Status::Starting;
        self.detect_hardware();
        self.config.validate()?;

        if !self.binary.exists() {
            self.status = Status::Failed;
            let msg = format!("llama-server not found at {}", self.binary.display());
            self.logs.error(&msg);
            return Err(ManagerError::Other(msg));
        }

        if !self.config.model.path.exists() {
            self.status = Status::Failed;
            let msg = format!("model not found: {}", self.config.model.path.display());
            self.logs.error(&msg);
            return Err(ManagerError::Other(msg));
        }

        if !network::port_available(&self.config.server.host, self.config.server.port) {
            self.status = Status::Failed;
            let msg = format!(
                "port {}:{} is not available (in use or not bindable); stop the other process or change server.port",
                self.config.server.host, self.config.server.port
            );
            self.logs.error(&msg);
            return Err(ManagerError::Other(msg));
        }

        let (program, args) = self.build_command();
        self.logs.server(&format!("starting {} {}", program.display(), args.join(" ")));

        let (tx, mut rx) = mpsc::unbounded_channel::<String>();
        let logs = LogSinks::open(&self.config.logging.dir)?;
        tokio::spawn(async move {
            while let Some(line) = rx.recv().await {
                logs.llama(&line);
            }
        });

        let workdir = program.parent().map(|p| p.to_path_buf());
        let child = ChildProcess::spawn(program, args, workdir, tx).await?;
        self.logs.server(&format!("spawned pid={}", child.pid));
        self.child = Some(child);

        match health::wait_until_ready(&self.config.base_url(), Duration::from_secs(120)).await {
            Ok(()) => {
                self.status = Status::Ready;
                self.logs.server(&format!("READY {}", self.config.openai_v1_url()));
                Ok(())
            }
            Err(e) => {
                self.logs.error(&format!("readiness failed: {e}"));
                let _ = self.stop().await;
                self.status = Status::Failed;
                Err(e.into())
            }
        }
    }

    pub async fn stop(&mut self) -> Result<(), ManagerError> {
        self.status = Status::Stopping;
        if let Some(mut child) = self.child.take() {
            self.logs.server("stopping (grace 8s)");
            let code = child.stop(Duration::from_secs(8)).await?;
            self.last_exit = code;
            self.logs.server(&format!("stopped exit={code:?}"));
        }
        self.status = Status::Stopped;
        Ok(())
    }

    pub async fn restart(&mut self) -> Result<(), ManagerError> {
        let _ = self.stop().await;
        self.start().await
    }

    pub fn get_status(&mut self) -> Status {
        if let Some(child) = self.child.as_mut() {
            if !child.is_running() {
                self.status = Status::Crashed;
                self.logs.error("llama-server exited unexpectedly");
                self.child = None;
            }
        }
        self.status
    }

    pub fn openai_base(&self) -> String {
        self.config.openai_v1_url()
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}
