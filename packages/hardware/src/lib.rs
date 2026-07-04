//! Hardware detection.
//!
//! Research targets (Phase 0/1): NVML, CUDA Runtime, DirectX, WMI.

use serde::{Deserialize, Serialize};
use winserve_shared::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub vram_mb: u64,
    pub cuda_capable: bool,
    pub cuda_compute_capability: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub name: String,
    pub physical_cores: u32,
    pub logical_cores: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_mb: u64,
    pub available_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub gpus: Vec<GpuInfo>,
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
}

/// Detect local hardware. Stub returns a minimal profile until platform probes land.
pub fn detect() -> Result<HardwareProfile> {
    Ok(HardwareProfile {
        gpus: vec![],
        cpu: CpuInfo {
            name: "unknown".into(),
            physical_cores: 0,
            logical_cores: 0,
        },
        memory: MemoryInfo {
            total_mb: 0,
            available_mb: 0,
        },
    })
}

/// Resolve `auto` performance settings from a hardware profile.
pub fn auto_gpu_layers(profile: &HardwareProfile) -> u32 {
    if profile.gpus.iter().any(|g| g.cuda_capable) {
        // Placeholder: real logic uses VRAM and model size.
        99
    } else {
        0
    }
}
