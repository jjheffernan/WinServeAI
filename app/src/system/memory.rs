#[derive(Debug, Clone, Default)]
pub struct MemoryInfo {
    pub total_mb: u64,
}

pub fn detect() -> MemoryInfo {
    // TODO: sysinfo or Windows GlobalMemoryStatusEx.
    MemoryInfo { total_mb: 0 }
}
