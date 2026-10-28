//! GPU name + VRAM. Windows: DXGI; optional NVML device totals; nvidia-smi fallback.

use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct GpuInfo {
    pub name: String,
    pub vram_mb: u64,
}

/// Detect GPUs. Never fails — empty means CPU-only path.
pub fn detect_gpus() -> Vec<GpuInfo> {
    #[cfg(windows)]
    {
        let dxgi = win::detect_dxgi();
        if !dxgi.is_empty() {
            return merge_nvml_totals(dxgi);
        }
    }
    detect_nvidia_smi()
}

#[cfg(windows)]
fn merge_nvml_totals(mut gpus: Vec<GpuInfo>) -> Vec<GpuInfo> {
    if let Some(totals) = win::nvml_device_totals() {
        for (gpu, total) in gpus.iter_mut().zip(totals.iter()) {
            if gpu.vram_mb == 0 {
                gpu.vram_mb = *total;
            }
        }
    }
    gpus
}

/// Cross-platform fallback: `nvidia-smi --query-gpu=name,memory.total --format=csv,noheader,nounits`
fn detect_nvidia_smi() -> Vec<GpuInfo> {
    let output = Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,memory.total",
            "--format=csv,noheader,nounits",
        ])
        .output();
    let Ok(output) = output else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let mut gpus = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.splitn(2, ',');
        let name = parts.next().unwrap_or("NVIDIA GPU").trim().to_string();
        let vram_mb = parts
            .next()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);
        gpus.push(GpuInfo { name, vram_mb });
    }
    gpus
}

#[cfg(windows)]
mod win {
    use super::GpuInfo;
    use windows::core::Interface;
    use windows::Win32::Graphics::Dxgi::{
        CreateDXGIFactory1, IDXGIAdapter3, IDXGIFactory1, DXGI_MEMORY_SEGMENT_GROUP_LOCAL,
    };

    pub fn detect_dxgi() -> Vec<GpuInfo> {
        unsafe {
            let factory: IDXGIFactory1 = match CreateDXGIFactory1() {
                Ok(f) => f,
                Err(_) => return Vec::new(),
            };
            let mut gpus = Vec::new();
            let mut i = 0u32;
            loop {
                let adapter = match factory.EnumAdapters1(i) {
                    Ok(a) => a,
                    Err(_) => break,
                };
                i += 1;
                let desc = match adapter.GetDesc1() {
                    Ok(d) => d,
                    Err(_) => continue,
                };
                // Skip software adapters (WARP). DXGI_ADAPTER_FLAG_SOFTWARE = 2
                // windows 0.58: Flags is already u32 (not a newtype with .0).
                if (desc.Flags & 2) != 0 {
                    continue;
                }
                let name = String::from_utf16_lossy(
                    &desc.Description[..desc
                        .Description
                        .iter()
                        .position(|&c| c == 0)
                        .unwrap_or(desc.Description.len())],
                );
                if name.contains("Microsoft Basic Render") {
                    continue;
                }
                let mut vram_mb = (desc.DedicatedVideoMemory as u64) / (1024 * 1024);
                if let Ok(adapter3) = adapter.cast::<IDXGIAdapter3>() {
                    let mut info = windows::Win32::Graphics::Dxgi::DXGI_QUERY_VIDEO_MEMORY_INFO::default();
                    if adapter3
                        .QueryVideoMemoryInfo(0, DXGI_MEMORY_SEGMENT_GROUP_LOCAL, &mut info)
                        .is_ok()
                        && info.Budget > 0
                    {
                        vram_mb = info.Budget / (1024 * 1024);
                    }
                }
                gpus.push(GpuInfo { name, vram_mb });
            }
            gpus
        }
    }

    /// Optional NVML device-wide totals via `nvidia-smi` (avoids linking nvml.dll).
    pub fn nvml_device_totals() -> Option<Vec<u64>> {
        // Prefer nvidia-smi for totals; dedicated NVML FFI can land later.
        let output = std::process::Command::new("nvidia-smi")
            .args(["--query-gpu=memory.total", "--format=csv,noheader,nounits"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let text = String::from_utf8_lossy(&output.stdout);
        let totals: Vec<u64> = text
            .lines()
            .filter_map(|l| l.trim().parse().ok())
            .collect();
        if totals.is_empty() {
            None
        } else {
            Some(totals)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_gpus_does_not_panic() {
        let _ = detect_gpus();
    }
}
