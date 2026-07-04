//! Shared types and error definitions used across WinServeAI packages.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// High-level server lifecycle state exposed to clients (desktop, CLI, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerState {
    Stopped,
    Starting,
    Running,
    Stopping,
    Unhealthy,
    Crashed,
}

/// Backend identity. UI and clients never depend on a concrete backend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackendKind {
    LlamaCpp,
    // Future: Ollama, Vllm, TensorRt
}

/// Snapshot of backend health for status surfaces.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub healthy: bool,
    pub state: ServerState,
    pub message: Option<String>,
}

/// Lightweight metrics placeholder (Phase 5 expands this).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Metrics {
    pub uptime_secs: u64,
}

#[derive(Debug, Error)]
pub enum WinServeError {
    #[error("configuration error: {0}")]
    Config(String),

    #[error("process error: {0}")]
    Process(String),

    #[error("backend error: {0}")]
    Backend(String),

    #[error("hardware error: {0}")]
    Hardware(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, WinServeError>;
