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

    let key = config.server.api_key.trim();
    if !key.is_empty() {
        args.push("--api-key".into());
        args.push(key.to_string());
    }

    // The server does not ship a page. Frontends live in clients/chat and call /v1.
    args.push("--no-ui".into());

    (binary.to_path_buf(), args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::config::Config;
    use crate::system::gpu::GpuInfo;
    use crate::system::memory::MemoryInfo;
    use crate::system::HardwareInfo;

    fn hw_cpu() -> HardwareInfo {
        HardwareInfo {
            gpus: vec![],
            memory: MemoryInfo::default(),
            cpu_threads: 4,
        }
    }

    fn hw_gpu() -> HardwareInfo {
        HardwareInfo {
            gpus: vec![GpuInfo {
                name: "Test GPU".into(),
                vram_mb: 8192,
                ..GpuInfo::default()
            }],
            memory: MemoryInfo::default(),
            cpu_threads: 8,
        }
    }

    #[test]
    fn auto_cpu_uses_zero_layers() {
        let cfg = Config::default();
        let (_bin, args) = build_command(&cfg, &hw_cpu(), Path::new("llama-server"));
        let joined = args.join(" ");
        assert!(joined.contains("--n-gpu-layers 0"), "{joined}");
        assert!(!joined.contains("--fit"), "{joined}");
        assert!(joined.contains("--no-ui"), "{joined}");
    }

    #[test]
    fn auto_gpu_uses_fit() {
        let cfg = Config::default();
        let (_bin, args) = build_command(&cfg, &hw_gpu(), Path::new("llama-server"));
        let joined = args.join(" ");
        assert!(joined.contains("--fit on"), "{joined}");
        assert!(!joined.contains("--n-gpu-layers"), "{joined}");
    }

    #[test]
    fn explicit_layers() {
        let mut cfg = Config::default();
        cfg.gpu.auto = false;
        cfg.gpu.layers = "12".into();
        let (_bin, args) = build_command(&cfg, &hw_gpu(), Path::new("llama-server"));
        let joined = args.join(" ");
        assert!(joined.contains("--n-gpu-layers 12"), "{joined}");
    }

    #[test]
    fn api_key_flag_only_when_set() {
        let mut cfg = Config::default();
        let (_bin, args) = build_command(&cfg, &hw_cpu(), Path::new("llama-server"));
        assert!(!args.iter().any(|a| a == "--api-key"));
        cfg.server.api_key = "  desk-key  ".into();
        let (_bin, args) = build_command(&cfg, &hw_cpu(), Path::new("llama-server"));
        assert!(args.windows(2).any(|w| w[0] == "--api-key" && w[1] == "desk-key"));
        assert!(!super::format_args_for_log(&args).contains("desk-key"));
    }
}

/// Argv for logs. The value after `--api-key` is replaced so server.log stays shareable.
pub fn format_args_for_log(args: &[String]) -> String {
    let mut out = Vec::with_capacity(args.len());
    let mut hide_next = false;
    for arg in args {
        if hide_next {
            out.push("***".into());
            hide_next = false;
            continue;
        }
        hide_next = arg == "--api-key";
        out.push(arg.clone());
    }
    out.join(" ")
}
