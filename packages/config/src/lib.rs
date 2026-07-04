//! Configuration: human-readable YAML, never raw llama.cpp flags.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use winserve_shared::{Result, WinServeError};

/// Root configuration document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub model: ModelConfig,
    pub performance: PerformanceConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub path: PathBuf,
}

/// High-level performance knobs. `auto` means hardware-derived defaults.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Context size, or `"auto"`.
    #[serde(default = "auto_str")]
    pub context: String,
    /// GPU layers, or `"auto"`.
    #[serde(default = "auto_str")]
    pub gpu_layers: String,
    /// Flash attention, or `"auto"`.
    #[serde(default = "auto_str")]
    pub flash_attention: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
}

fn auto_str() -> String {
    "auto".into()
}

fn default_log_level() -> String {
    "info".into()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".into(),
                port: 8080,
            },
            model: ModelConfig {
                path: PathBuf::from(r"D:\Models\model.gguf"),
            },
            performance: PerformanceConfig {
                context: auto_str(),
                gpu_layers: auto_str(),
                flash_attention: auto_str(),
            },
            logging: LoggingConfig {
                level: default_log_level(),
            },
        }
    }
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let text = std::fs::read_to_string(path.as_ref())?;
        serde_yaml::from_str(&text)
            .map_err(|e| WinServeError::Config(format!("invalid config: {e}")))
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let text = serde_yaml::to_string(self)
            .map_err(|e| WinServeError::Config(format!("serialize config: {e}")))?;
        std::fs::write(path.as_ref(), text)?;
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        if self.server.port == 0 {
            return Err(WinServeError::Config("server.port must be non-zero".into()));
        }
        if self.model.path.as_os_str().is_empty() {
            return Err(WinServeError::Config("model.path is required".into()));
        }
        Ok(())
    }
}
