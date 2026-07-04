//! Backend abstraction.
//!
//! Concrete backends (`LlamaBackend`, future `OllamaBackend`, etc.) implement
//! [`Backend`]. Clients (desktop, tray, management API) only depend on this trait.

use async_trait::async_trait;
use winserve_shared::{HealthStatus, Metrics, Result, ServerState};

/// Backend-agnostic inference engine interface.
///
/// Never expose backend-specific flags or process details through this trait.
#[async_trait]
pub trait Backend: Send + Sync {
    /// Prepare the backend (resolve binaries, validate environment).
    async fn initialize(&mut self) -> Result<()>;

    /// Load the configured model (path comes from config, not the caller).
    async fn load_model(&mut self) -> Result<()>;

    /// Start serving the OpenAI-compatible endpoint.
    async fn start(&mut self) -> Result<()>;

    /// Gracefully stop the backend.
    async fn stop(&mut self) -> Result<()>;

    /// Current lifecycle state.
    fn status(&self) -> ServerState;

    /// Optional runtime metrics.
    fn metrics(&self) -> Metrics;

    /// Health probe (readiness / liveness).
    async fn health(&self) -> Result<HealthStatus>;

    /// Backend implementation version string.
    fn version(&self) -> &str;
}
