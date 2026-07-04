//! System RAM via sysinfo.

use sysinfo::System;

#[derive(Debug, Clone, Default)]
pub struct MemoryInfo {
    pub total_mb: u64,
    pub available_mb: u64,
}

pub fn detect() -> MemoryInfo {
    let mut sys = System::new();
    sys.refresh_memory();
    MemoryInfo {
        total_mb: sys.total_memory() / (1024 * 1024),
        available_mb: sys.available_memory() / (1024 * 1024),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_reports_some_ram() {
        let m = detect();
        // CI/dev machines always have some RAM.
        assert!(m.total_mb > 0);
    }
}
