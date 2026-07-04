//! OpenAI compatibility utilities and health/readiness helpers.

use serde::{Deserialize, Serialize};
use winserve_shared::Result;

/// Endpoint base used by clients and readiness checks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    pub host: String,
    pub port: u16,
}

impl Endpoint {
    pub fn base_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }

    pub fn health_url(&self) -> String {
        format!("{}/health", self.base_url())
    }

    pub fn openai_v1_url(&self) -> String {
        format!("{}/v1", self.base_url())
    }
}

/// Probe whether the OpenAI-compatible server is ready.
///
/// Implementation lands in Phase 1 (HTTP client + readiness timeout).
pub async fn wait_until_ready(_endpoint: &Endpoint) -> Result<()> {
    // Stub: real implementation polls /health or /v1/models.
    Ok(())
}
