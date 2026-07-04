//! `LlamaBackend` — translates high-level config into llama-server process args.
//!
//! Callers never see llama.cpp flags; only [`winserve_backend::Backend`].

use async_trait::async_trait;
use std::path::PathBuf;
use winserve_backend::Backend;
use winserve_config::Config;
use winserve_hardware::{auto_gpu_layers, HardwareProfile};
use winserve_process::{ManagedProcess, ProcessSpec};
use winserve_shared::{HealthStatus, Metrics, Result, ServerState, WinServeError};

pub struct LlamaBackend {
    config: Config,
    hardware: HardwareProfile,
    llama_server: PathBuf,
    process: Option<ManagedProcess>,
    state: ServerState,
}

impl LlamaBackend {
    pub fn new(config: Config, hardware: HardwareProfile, llama_server: PathBuf) -> Self {
        Self {
            config,
            hardware,
            llama_server,
            process: None,
            state: ServerState::Stopped,
        }
    }

    /// Build launch args from config. Never expose this to the UI layer.
    fn build_args(&self) -> Vec<String> {
        let gpu_layers = if self.config.performance.gpu_layers == "auto" {
            auto_gpu_layers(&self.hardware).to_string()
        } else {
            self.config.performance.gpu_layers.clone()
        };

        let mut args = vec![
            "--host".into(),
            self.config.server.host.clone(),
            "--port".into(),
            self.config.server.port.to_string(),
            "--model".into(),
            self.config.model.path.display().to_string(),
            "--n-gpu-layers".into(),
            gpu_layers,
        ];

        if self.config.performance.context != "auto" {
            args.push("--ctx-size".into());
            args.push(self.config.performance.context.clone());
        }

        args
    }
}

#[async_trait]
impl Backend for LlamaBackend {
    async fn initialize(&mut self) -> Result<()> {
        if !self.llama_server.exists() {
            return Err(WinServeError::Backend(format!(
                "llama-server not found at {}",
                self.llama_server.display()
            )));
        }
        self.config.validate()?;
        Ok(())
    }

    async fn load_model(&mut self) -> Result<()> {
        // Model path is validated at start; dedicated preload may land later.
        if !self.config.model.path.exists() {
            return Err(WinServeError::Backend(format!(
                "model not found: {}",
                self.config.model.path.display()
            )));
        }
        Ok(())
    }

    async fn start(&mut self) -> Result<()> {
        self.state = ServerState::Starting;
        let spec = ProcessSpec {
            program: self.llama_server.clone(),
            args: self.build_args(),
            working_dir: self.llama_server.parent().map(|p| p.to_path_buf()),
        };
        let mut process = ManagedProcess::new(spec);
        process.spawn().await?;
        self.process = Some(process);
        self.state = ServerState::Running;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.state = ServerState::Stopping;
        if let Some(mut process) = self.process.take() {
            process.graceful_stop().await?;
        }
        self.state = ServerState::Stopped;
        Ok(())
    }

    fn status(&self) -> ServerState {
        self.state
    }

    fn metrics(&self) -> Metrics {
        Metrics::default()
    }

    async fn health(&self) -> Result<HealthStatus> {
        Ok(HealthStatus {
            healthy: self.state == ServerState::Running,
            state: self.state,
            message: None,
        })
    }

    fn version(&self) -> &str {
        "llama.cpp (bundled)"
    }
}
