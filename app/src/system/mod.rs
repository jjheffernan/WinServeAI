//! Minimal host probes: GPU, memory, network.

pub mod gpu;
pub mod memory;
pub mod network;

use gpu::GpuInfo;
use memory::MemoryInfo;

#[derive(Debug, Clone, Default)]
pub struct HardwareInfo {
    pub gpus: Vec<GpuInfo>,
    pub memory: MemoryInfo,
    pub cpu_threads: u32,
}

/// Detect what we need for auto GPU defaults. Stubs until DXGI/NVML land.
pub fn detect() -> HardwareInfo {
    HardwareInfo {
        gpus: gpu::detect_gpus(),
        memory: memory::detect(),
        cpu_threads: std::thread::available_parallelism()
            .map(|n| n.get() as u32)
            .unwrap_or(1),
    }
}
