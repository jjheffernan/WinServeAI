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
    /// Sent to clients as `Authorization: Bearer`. Empty means localhost with no key.
    #[serde(default)]
    pub api_key: String,
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
    /// Rotate when the active file reaches this size (0 = disable size rotation).
    #[serde(default = "default_max_bytes")]
    pub max_bytes: u64,
    /// Rotate when the active file is older than this many seconds (0 = disable age rotation).
    #[serde(default = "default_max_age_secs")]
    pub max_age_secs: u64,
    /// How many rotated siblings to keep (`server.log.1` … `.N`).
    #[serde(default = "default_keep")]
    pub keep: u32,
}

pub fn is_loopback_host(host: &str) -> bool {
    matches!(
        host.trim().trim_matches(['[', ']']).to_ascii_lowercase().as_str(),
        "127.0.0.1" | "localhost" | "::1"
    )
}

/// Bind-all wildcards — listen everywhere, but never advertise these as client URLs.
pub fn is_wildcard_bind(host: &str) -> bool {
    matches!(
        host.trim().trim_matches(['[', ']']).to_ascii_lowercase().as_str(),
        "0.0.0.0" | "::" | "*"
    )
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
fn default_max_bytes() -> u64 {
    10 * 1024 * 1024
}
fn default_max_age_secs() -> u64 {
    7 * 24 * 60 * 60
}
fn default_keep() -> u32 {
    3
}

impl Default for LoggingSection {
    fn default() -> Self {
        Self {
            dir: default_log_dir(),
            max_bytes: default_max_bytes(),
            max_age_secs: default_max_age_secs(),
            keep: default_keep(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerSection {
                host: default_host(),
                port: default_port(),
                api_key: String::new(),
            },
            model: ModelSection {
                path: PathBuf::new(),
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
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        // Atomic replace so a power cut mid-write cannot leave a half YAML.
        let mut tmp = path.as_os_str().to_owned();
        tmp.push(".tmp");
        let tmp = PathBuf::from(tmp);
        std::fs::write(&tmp, &text)?;
        // Windows cannot rename over an existing file.
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        std::fs::rename(&tmp, path)?;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.server.port == 0 {
            return Err(ConfigError::Validate("server.port must be non-zero".into()));
        }
        // Empty model.path is allowed at load for first-run; `start` still requires a real file.
        if self.server.host.trim().is_empty() {
            return Err(ConfigError::Validate("server.host is required".into()));
        }
        if self.gpu.layers != "auto" && self.gpu.layers.parse::<u32>().is_err() {
            return Err(ConfigError::Validate(
                "gpu.layers must be \"auto\" or a non-negative integer".into(),
            ));
        }
        if self.runtime.context == 0 {
            return Err(ConfigError::Validate(
                "runtime.context must be non-zero".into(),
            ));
        }
        Ok(())
    }

    /// Soft checks for operator warnings (do not fail load).
    pub fn warnings(&self) -> Vec<String> {
        let mut w = Vec::new();
        if self.model.path.as_os_str().is_empty() {
            w.push(
                "model.path is empty — set a local .gguf before start (docs/first-run.md / Settings Browse)"
                    .into(),
            );
        } else if !self.model.path.exists() {
            w.push(format!(
                "model.path does not exist yet: {}",
                self.model.path.display()
            ));
        }
        if self.server.port < 1024 {
            w.push(format!(
                "server.port {} may require elevated privileges on some systems",
                self.server.port
            ));
        }
        if !is_loopback_host(&self.server.host) && self.server.api_key.trim().is_empty() {
            w.push(
                "server.api_key is empty while host is not loopback — set a key before other machines connect"
                    .into(),
            );
        }
        if is_wildcard_bind(&self.server.host) {
            w.push(
                "server.host is a wildcard bind — llama-server listens on all interfaces; client URLs use 127.0.0.1 (LAN peers need this PC's real IP, not a WSL/Hyper-V adapter)"
                    .into(),
            );
        }
        w
    }

    /// Host string for client URLs and readiness probes (never a wildcard).
    ///
    /// Bind address stays `server.host` (passed to llama-server). We do not
    /// auto-pick a LAN/WSL interface — operator owns that field across updates.
    pub fn client_host(&self) -> &str {
        if is_wildcard_bind(&self.server.host) {
            "127.0.0.1"
        } else {
            self.server.host.as_str()
        }
    }

    pub fn base_url(&self) -> String {
        format!("http://{}:{}", self.client_host(), self.server.port)
    }

    pub fn openai_v1_url(&self) -> String {
        format!("{}/v1", self.base_url())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_validates() {
        Config::default().validate().unwrap();
    }

    #[test]
    fn accepts_empty_model_path_for_first_run() {
        let mut c = Config::default();
        c.model.path = PathBuf::new();
        c.validate().unwrap();
        let w = c.warnings();
        assert!(w.iter().any(|s| s.contains("model.path is empty")));
    }

    #[test]
    fn rejects_zero_port() {
        let mut c = Config::default();
        c.server.port = 0;
        assert!(c.validate().is_err());
    }

    #[test]
    fn rejects_bad_gpu_layers() {
        let mut c = Config::default();
        c.gpu.layers = "lots".into();
        assert!(c.validate().is_err());
    }

    #[test]
    fn accepts_numeric_gpu_layers() {
        let mut c = Config::default();
        c.gpu.layers = "32".into();
        c.validate().unwrap();
    }

    #[test]
    fn roundtrip_yaml() {
        let c = Config::default();
        let text = serde_yaml::to_string(&c).unwrap();
        let parsed: Config = serde_yaml::from_str(&text).unwrap();
        assert_eq!(parsed.server.port, 8080);
        assert_eq!(parsed.gpu.layers, "auto");
        assert!(parsed.server.api_key.is_empty());
    }

    #[test]
    fn warns_when_lan_has_no_api_key() {
        let mut c = Config::default();
        c.server.host = "0.0.0.0".into();
        assert!(c
            .warnings()
            .iter()
            .any(|s| s.contains("server.api_key")));
        c.server.api_key = "secret".into();
        assert!(!c.warnings().iter().any(|s| s.contains("server.api_key")));
    }

    #[test]
    fn wildcard_bind_advertises_loopback_not_zero() {
        let mut c = Config::default();
        c.server.host = "0.0.0.0".into();
        c.server.api_key = "secret".into();
        assert_eq!(c.client_host(), "127.0.0.1");
        assert_eq!(c.base_url(), "http://127.0.0.1:8080");
        assert_eq!(c.openai_v1_url(), "http://127.0.0.1:8080/v1");
        // Bind field itself is unchanged (modular; never rewritten on save/load).
        assert_eq!(c.server.host, "0.0.0.0");
        assert!(c.warnings().iter().any(|s| s.contains("wildcard")));
    }

    #[test]
    fn save_is_atomic_replace() {
        let dir = std::env::temp_dir().join(format!(
            "winserve-cfg-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("default.yaml");
        let tmp = PathBuf::from(format!("{}.tmp", path.display()));
        let mut c = Config::default();
        c.server.port = 9090;
        c.save(&path).unwrap();
        assert!(path.is_file());
        assert!(!tmp.exists());
        c.server.port = 9091;
        c.save(&path).unwrap();
        let loaded = Config::load(&path).unwrap();
        assert_eq!(loaded.server.port, 9091);
        assert!(!tmp.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
