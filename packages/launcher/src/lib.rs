//! Server Manager — the single owner of the inference process.
//!
//! ```text
//! Desktop UI / Tray / CLI / Management API
//!              │
//!              ▼
//!        Server Manager  (this crate)
//!              │
//!   ┌──────────┼──────────┬─────────────┐
//!   ▼          ▼          ▼             ▼
//! Config   Hardware   Logging      Backend
//!                                      │
//!                                      ▼
//!                               llama-server.exe
//! ```
//!
//! UI clients only: edit config, display logs/state, start, stop.

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use winserve_api::Endpoint;
use winserve_backend::Backend;
use winserve_config::Config;
use winserve_hardware::{detect, HardwareProfile};
use winserve_llama::LlamaBackend;
use winserve_shared::{HealthStatus, Metrics, Result, ServerState};

/// Orchestrates configuration, hardware, logging, and the active backend.
pub struct ServerManager {
    config: Config,
    hardware: HardwareProfile,
    backend: Arc<Mutex<Box<dyn Backend>>>,
}

impl ServerManager {
    /// Create a manager with the default llama.cpp backend.
    pub fn with_llama(config: Config, llama_server: PathBuf) -> Result<Self> {
        let hardware = detect()?;
        let backend: Box<dyn Backend> =
            Box::new(LlamaBackend::new(config.clone(), hardware.clone(), llama_server));
        Ok(Self {
            config,
            hardware,
            backend: Arc::new(Mutex::new(backend)),
        })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn update_config(&mut self, config: Config) -> Result<()> {
        config.validate()?;
        self.config = config;
        Ok(())
    }

    pub fn hardware(&self) -> &HardwareProfile {
        &self.hardware
    }

    pub fn endpoint(&self) -> Endpoint {
        Endpoint {
            host: self.config.server.host.clone(),
            port: self.config.server.port,
        }
    }

    pub async fn start(&self) -> Result<()> {
        let mut backend = self.backend.lock().await;
        backend.initialize().await?;
        backend.load_model().await?;
        backend.start().await?;
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut backend = self.backend.lock().await;
        backend.stop().await
    }

    pub async fn status(&self) -> ServerState {
        let backend = self.backend.lock().await;
        backend.status()
    }

    pub async fn health(&self) -> Result<HealthStatus> {
        let backend = self.backend.lock().await;
        backend.health().await
    }

    pub async fn metrics(&self) -> Metrics {
        let backend = self.backend.lock().await;
        backend.metrics()
    }

    pub async fn version(&self) -> String {
        let backend = self.backend.lock().await;
        backend.version().to_string()
    }
}
