//! YAML config is the source of truth.
//!
//! No raw llama.cpp flags here. Mapping to argv lives only in `runtime::llama`.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid config: {0}")]
    Parse(String),
    #[error("validation: {0}")]
    Validate(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerSection,
    pub model: ModelSection,
    pub gpu: GpuSection,
    pub runtime: RuntimeSection,
    #[serde(default)]
    pub logging: LoggingSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSection {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSection {
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuSection {
    #[serde(default = "default_true")]
    pub auto: bool,
    /// `"auto"` or an integer layer count as a string.
    #[serde(default = "auto_str")]
    pub layers: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSection {
    #[serde(default = "default_context")]
    pub context: u32,
    #[serde(default = "default_true")]
    pub flash_attention: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingSection {
    #[serde(default = "default_log_dir")]
    pub dir: PathBuf,
}

fn default_host() -> String {
    "127.0.0.1".into()
}
fn default_port() -> u16 {
    8080
}
fn default_true() -> bool {
    true
}
fn auto_str() -> String {
    "auto".into()
}
fn default_context() -> u32 {
    32768
}
fn default_log_dir() -> PathBuf {
    PathBuf::from("logs")
}

impl Default for LoggingSection {
    fn default() -> Self {
        Self {
            dir: default_log_dir(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerSection {
                host: default_host(),
                port: default_port(),
            },
            model: ModelSection {
                path: PathBuf::from(r"D:\models\model.gguf"),
            },
            gpu: GpuSection {
                auto: true,
                layers: auto_str(),
            },
            runtime: RuntimeSection {
                context: default_context(),
                flash_attention: true,
            },
            logging: LoggingSection::default(),
        }
    }
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let text = std::fs::read_to_string(path)?;
        let cfg: Self = serde_yaml::from_str(&text)
            .map_err(|e| ConfigError::Parse(e.to_string()))?;
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), ConfigError> {
        self.validate()?;
        let text = serde_yaml::to_string(self)
            .map_err(|e| ConfigError::Parse(e.to_string()))?;
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, text)?;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.server.port == 0 {
            return Err(ConfigError::Validate("server.port must be non-zero".into()));
        }
        if self.model.path.as_os_str().is_empty() {
            return Err(ConfigError::Validate("model.path is required".into()));
        }
        Ok(())
    }

    pub fn base_url(&self) -> String {
        format!("http://{}:{}", self.server.host, self.server.port)
    }

    pub fn openai_v1_url(&self) -> String {
        format!("{}/v1", self.base_url())
    }
}
