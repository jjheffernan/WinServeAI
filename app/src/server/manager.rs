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

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Stopped => "Stopped",
            Status::Starting => "Starting",
            Status::Ready => "Ready",
            Status::Stopping => "Stopping",
            Status::Failed => "Failed",
            Status::Crashed => "Crashed",
        }
    }
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
    config_path: PathBuf,
    config: Config,
    hardware: HardwareInfo,
    binary: PathBuf,
    logs: LogSinks,
    child: Option<ChildProcess>,
    status: Status,
    last_exit: Option<i32>,
    /// How long `start` waits for HTTP readiness (default 120s).
    ready_timeout: Duration,
}

impl ServerManager {
    pub fn load_config(root: impl AsRef<Path>, config_path: impl AsRef<Path>) -> Result<Self, ManagerError> {
        let root = root.as_ref().to_path_buf();
        let config_path = config_path.as_ref().to_path_buf();
        let config = Config::load(&config_path)?;
        let hardware = system::detect();
        let binary = default_binary(&root);
        let logs = LogSinks::open(&config.logging)?;
        logs.server(&format!("config loaded; binary={}", binary.display()));
        for w in config.warnings() {
            logs.server(&format!("warning: {w}"));
        }
        Ok(Self {
            root,
            config_path,
            config,
            hardware,
            binary,
            logs,
            child: None,
            status: Status::Stopped,
            last_exit: None,
            ready_timeout: Duration::from_secs(120),
        })
    }

    /// Override readiness wait (tests / controllable fake backends).
    pub fn set_ready_timeout(&mut self, timeout: Duration) {
        self.ready_timeout = timeout;
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    pub fn hardware(&self) -> &HardwareInfo {
        &self.hardware
    }

    pub fn status(&self) -> Status {
        self.status
    }

    /// Validate + write YAML, then replace in-memory config.
    ///
    /// Rejected while Starting / Ready / Stopping — stop first (E1d).
    pub fn apply_config(&mut self, next: Config) -> Result<(), ManagerError> {
        let _ = self.get_status();
        match self.status {
            Status::Starting | Status::Ready | Status::Stopping => {
                return Err(ManagerError::Other(
                    "stop the server before editing settings (Starting/Ready/Stopping)".into(),
                ));
            }
            Status::Stopped | Status::Failed | Status::Crashed => {}
        }
        next.validate()?;
        next.save(&self.config_path)?;
        self.config = next;
        self.logs.server(&format!(
            "config saved {}; endpoint will be {}",
            self.config_path.display(),
            self.config.openai_v1_url()
        ));
        for w in self.config.warnings() {
            self.logs.server(&format!("warning: {w}"));
        }
        Ok(())
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

        if self.config.model.path.as_os_str().is_empty() {
            self.status = Status::Failed;
            let msg = "model.path is empty — set a local .gguf (docs/first-run.md / Settings Browse)"
                .to_string();
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
        let logs = LogSinks::open(&self.config.logging)?;
        tokio::spawn(async move {
            while let Some(line) = rx.recv().await {
                logs.llama(&line);
            }
        });

        let workdir = program.parent().map(|p| p.to_path_buf());
        let child = ChildProcess::spawn(program, args, workdir, tx).await?;
        self.logs.server(&format!("spawned pid={}", child.pid));
        self.child = Some(child);

        match health::wait_until_ready(&self.config.base_url(), self.ready_timeout).await {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::config::Config;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_cfg() -> (PathBuf, PathBuf, Config) {
        let root = std::env::temp_dir().join(format!(
            "winserve-mgr-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("config")).unwrap();
        std::fs::create_dir_all(root.join("logs")).unwrap();
        let path = root.join("config").join("default.yaml");
        let mut cfg = Config::default();
        cfg.logging.dir = root.join("logs");
        cfg.model.path = root.join("missing.gguf");
        cfg.save(&path).unwrap();
        (root, path, cfg)
    }

    #[test]
    fn apply_config_writes_yaml_when_stopped() {
        let (root, path, mut cfg) = temp_cfg();
        let mut mgr = ServerManager::load_config(&root, &path).unwrap();
        cfg.server.port = 9090;
        cfg.server.host = "127.0.0.1".into();
        mgr.apply_config(cfg).unwrap();
        let reloaded = Config::load(&path).unwrap();
        assert_eq!(reloaded.server.port, 9090);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn apply_config_rejects_while_ready() {
        let (root, path, cfg) = temp_cfg();
        let mut mgr = ServerManager::load_config(&root, &path).unwrap();
        mgr.status = Status::Ready;
        let err = mgr.apply_config(cfg).unwrap_err().to_string();
        assert!(err.contains("stop the server"), "got {err}");
        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn start_fails_when_binary_missing() {
        let (root, path, _) = temp_cfg();
        let mut mgr = ServerManager::load_config(&root, &path).unwrap();
        let err = mgr.start().await.unwrap_err().to_string();
        assert!(err.contains("llama-server not found"), "got {err}");
        assert_eq!(mgr.status(), Status::Failed);
        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn start_fails_when_model_path_empty() {
        let (root, path, mut cfg) = temp_cfg();
        std::fs::create_dir_all(root.join("bin")).unwrap();
        let bin = root.join("bin").join(if cfg!(windows) {
            "llama-server.exe"
        } else {
            "llama-server"
        });
        std::fs::write(&bin, b"stub").unwrap();
        cfg.model.path = PathBuf::new();
        cfg.save(&path).unwrap();
        let mut mgr = ServerManager::load_config(&root, &path).unwrap();
        let err = mgr.start().await.unwrap_err().to_string();
        assert!(err.contains("model.path is empty"), "got {err}");
        assert_eq!(mgr.status(), Status::Failed);
        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn start_fails_when_model_file_missing() {
        let (root, path, _) = temp_cfg();
        std::fs::create_dir_all(root.join("bin")).unwrap();
        let bin = root.join("bin").join(if cfg!(windows) {
            "llama-server.exe"
        } else {
            "llama-server"
        });
        std::fs::write(&bin, b"stub").unwrap();
        let mut mgr = ServerManager::load_config(&root, &path).unwrap();
        let err = mgr.start().await.unwrap_err().to_string();
        assert!(err.contains("model not found"), "got {err}");
        assert_eq!(mgr.status(), Status::Failed);
        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn start_fails_when_port_busy() {
        let (root, path, mut cfg) = temp_cfg();
        std::fs::create_dir_all(root.join("bin")).unwrap();
        let bin = root.join("bin").join(if cfg!(windows) {
            "llama-server.exe"
        } else {
            "llama-server"
        });
        std::fs::write(&bin, b"stub").unwrap();
        let gguf = root.join("model.gguf");
        std::fs::write(&gguf, b"fake").unwrap();
        cfg.model.path = gguf;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        cfg.server.host = "127.0.0.1".into();
        cfg.server.port = port;
        cfg.save(&path).unwrap();
        let mut mgr = ServerManager::load_config(&root, &path).unwrap();
        let err = mgr.start().await.unwrap_err().to_string();
        assert!(err.contains("not available"), "got {err}");
        assert_eq!(mgr.status(), Status::Failed);
        drop(listener);
        let _ = std::fs::remove_dir_all(root);
    }
}
