//! GPU name + VRAM. DXGI/NVML implementation is Phase 1; stub is honest.

#[derive(Debug, Clone, Default)]
pub struct GpuInfo {
    pub name: String,
    pub vram_mb: u64,
}

/// Returns empty when no probe is available (CPU-only path).
pub fn detect_gpus() -> Vec<GpuInfo> {
    // TODO: DXGI adapter enum + optional NVML device totals.
    Vec::new()
}
