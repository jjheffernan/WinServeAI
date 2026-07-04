//! ONLY place that knows llama.cpp flags.

use crate::server::config::Config;
use crate::system::HardwareInfo;
use std::path::{Path, PathBuf};

/// Resolve path to `llama-server` / `llama-server.exe`.
pub fn default_binary(repo_root: &Path) -> PathBuf {
    let name = if cfg!(windows) {
        "llama-server.exe"
    } else {
        "llama-server"
    };
    repo_root.join("bin").join(name)
}

/// Build argv from config + hardware. No UI/config layer should call this with flags.
pub fn build_command(config: &Config, hardware: &HardwareInfo, binary: &Path) -> (PathBuf, Vec<String>) {
    let mut args = vec![
        "--host".into(),
        config.server.host.clone(),
        "--port".into(),
        config.server.port.to_string(),
        "--model".into(),
        config.model.path.display().to_string(),
        "--ctx-size".into(),
        config.runtime.context.to_string(),
    ];

    // GPU layers: auto → omit -ngl and use --fit when supported; else explicit.
    if config.gpu.auto && config.gpu.layers == "auto" {
        // Prefer llama-server auto-fit when GPU present; CPU-only → 0 layers.
        if hardware.gpus.is_empty() {
            args.push("--n-gpu-layers".into());
            args.push("0".into());
        } else {
            // --fit on when auto (llama.cpp recent builds). Fallback: high layer count.
            args.push("--fit".into());
            args.push("on".into());
        }
    } else if config.gpu.layers == "auto" {
        args.push("--n-gpu-layers".into());
        args.push(if hardware.gpus.is_empty() { "0" } else { "99" }.into());
    } else {
        args.push("--n-gpu-layers".into());
        args.push(config.gpu.layers.clone());
    }

    if config.runtime.flash_attention {
        args.push("-fa".into());
        args.push("on".into());
    }

    (binary.to_path_buf(), args)
}
