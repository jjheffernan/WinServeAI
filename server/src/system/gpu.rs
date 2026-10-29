//! GPU name, VRAM, and live usage.
//!
//! Windows usage comes from the WDDM counters Task Manager uses
//! (`GPU Engine` utilization and `GPU Adapter Memory` dedicated bytes).
//! Those counters exist for NVIDIA, AMD, and Intel. DXGI `CurrentUsage` is
//! only this process, and llama-server is a child, so it is not the usage number.
//! `nvidia-smi` is the fallback when DXGI lists nothing.

use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct GpuInfo {
    pub name: String,
    pub vram_mb: u64,
    pub vram_used_mb: u64,
    /// Busiest engine, 0–100. None until a second sample exists.
    pub usage_pct: Option<u8>,
    /// Model file whose weights are on this card. None when the card is idle.
    pub hosted_model: Option<String>,
    pub(crate) key: Option<AdapterKey>,
}

/// Mark cards that hold llama-server's dedicated memory. The same model is set on every card that has it.
pub fn note_host(gpus: &mut [GpuInfo], pid: u32, model: &str) {
    #[cfg(windows)]
    {
        assign_hosted(gpus, &win::process_memory(), pid, model);
    }
    #[cfg(not(windows))]
    {
        let _ = (gpus, pid, model);
    }
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

/// Cross-platform fallback: nvidia-smi name, memory, and utilization.
fn detect_nvidia_smi() -> Vec<GpuInfo> {
    if let Some(gpus) = query_nvidia_smi("name,memory.total,memory.used,utilization.gpu") {
        if !gpus.is_empty() {
            return gpus;
        }
    }
    query_nvidia_smi("name,memory.total").unwrap_or_default()
}

fn query_nvidia_smi(fields: &str) -> Option<Vec<GpuInfo>> {
    let output = Command::new("nvidia-smi")
        .args([
            &format!("--query-gpu={fields}"),
            "--format=csv,noheader,nounits",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    Some(
        text.lines()
            .filter_map(|line| parse_nvidia_line(line.trim()))
            .collect(),
    )
}

fn parse_nvidia_line(line: &str) -> Option<GpuInfo> {
    if line.is_empty() {
        return None;
    }
    let parts: Vec<&str> = line.split(',').map(str::trim).collect();
    if parts.len() < 2 || parts[0].is_empty() {
        return None;
    }
    let usage_pct = parts.get(3).and_then(|raw| {
        let t = raw.trim().trim_end_matches('%');
        if t.eq_ignore_ascii_case("[N/A]") || t.eq_ignore_ascii_case("N/A") {
            return None;
        }
        let n: f64 = t.parse().ok()?;
        Some(n.clamp(0.0, 100.0).round() as u8)
    });
    Some(GpuInfo {
        name: parts[0].to_string(),
        vram_mb: parts[1].parse().unwrap_or(0),
        vram_used_mb: parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0),
        usage_pct,
        ..GpuInfo::default()
    })
}

#[cfg_attr(not(windows), allow(dead_code))]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct AdapterKey {
    high: i32,
    low: u32,
}

#[cfg_attr(not(windows), allow(dead_code))]
fn adapter_key(instance: &str) -> Option<AdapterKey> {
    let rest = instance.split("luid_0x").nth(1)?;
    let (high, rest) = rest.split_once("_0x")?;
    let low = rest
        .split(|c: char| !c.is_ascii_hexdigit())
        .next()
        .filter(|s| !s.is_empty())?;
    Some(AdapterKey {
        high: u32::from_str_radix(high, 16).ok()? as i32,
        low: u32::from_str_radix(low, 16).ok()?,
    })
}

#[cfg_attr(not(windows), allow(dead_code))]
fn engine_type(instance: &str) -> &str {
    instance
        .split("engtype_")
        .nth(1)
        .unwrap_or("")
        .split('_')
        .next()
        .unwrap_or("")
}

/// Sum each engine type across processes, then keep the busiest type. Capped at 100.
#[cfg_attr(not(windows), allow(dead_code))]
fn usage_from_engine_samples<'a>(
    samples: impl IntoIterator<Item = (&'a str, f64)>,
) -> HashMap<AdapterKey, u8> {
    let mut by_type: HashMap<(AdapterKey, &str), f64> = HashMap::new();
    for (instance, value) in samples {
        if !value.is_finite() || value < 0.0 {
            continue;
        }
        let Some(key) = adapter_key(instance) else {
            continue;
        };
        *by_type.entry((key, engine_type(instance))).or_insert(0.0) += value;
    }
    let mut busiest: HashMap<AdapterKey, f64> = HashMap::new();
    for ((key, _), sum) in by_type {
        let slot = busiest.entry(key).or_insert(0.0);
        if sum > *slot {
            *slot = sum;
        }
    }
    busiest
        .into_iter()
        .map(|(key, sum)| (key, sum.clamp(0.0, 100.0).round() as u8))
        .collect()
}

#[cfg_attr(not(windows), allow(dead_code))]
fn bytes_from_memory_samples<'a>(
    samples: impl IntoIterator<Item = (&'a str, f64)>,
) -> HashMap<AdapterKey, u64> {
    let mut totals: HashMap<AdapterKey, u64> = HashMap::new();
    for (instance, value) in samples {
        if !value.is_finite() || value < 0.0 {
            continue;
        }
        let Some(key) = adapter_key(instance) else {
            continue;
        };
        *totals.entry(key).or_insert(0) += value as u64;
    }
    totals
}

#[cfg_attr(not(windows), allow(dead_code))]
fn process_id(instance: &str) -> Option<u32> {
    let rest = instance.split("pid_").nth(1)?;
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

/// Dedicated bytes of one process on one adapter. Nodes on the same card are summed.
#[cfg_attr(not(windows), allow(dead_code))]
fn process_bytes_from_samples<'a>(
    samples: impl IntoIterator<Item = (&'a str, f64)>,
) -> HashMap<(u32, AdapterKey), u64> {
    let mut totals: HashMap<(u32, AdapterKey), u64> = HashMap::new();
    for (instance, value) in samples {
        if !value.is_finite() || value < 0.0 {
            continue;
        }
        let Some(pid) = process_id(instance) else {
            continue;
        };
        let Some(key) = adapter_key(instance) else {
            continue;
        };
        *totals.entry((pid, key)).or_insert(0) += value as u64;
    }
    totals
}

/// Ignore a few megabytes of driver noise. A loaded GGUF is much larger.
#[cfg_attr(not(windows), allow(dead_code))]
const HOST_MIN_BYTES: u64 = 32 * 1024 * 1024;

#[cfg_attr(not(windows), allow(dead_code))]
fn assign_hosted(
    gpus: &mut [GpuInfo],
    by_process: &HashMap<(u32, AdapterKey), u64>,
    pid: u32,
    model: &str,
) {
    let model = model.trim();
    for gpu in gpus.iter_mut() {
        gpu.hosted_model = None;
        if model.is_empty() || pid == 0 {
            continue;
        }
        let Some(key) = gpu.key else {
            continue;
        };
        if by_process.get(&(pid, key)).copied().unwrap_or(0) >= HOST_MIN_BYTES {
            gpu.hosted_model = Some(model.to_string());
        }
    }
}

#[cfg(windows)]
mod win {
    use super::{
        bytes_from_memory_samples, process_bytes_from_samples, usage_from_engine_samples,
        AdapterKey, GpuInfo,
    };
    use std::sync::Mutex;
    use windows::core::{Interface, PCWSTR, PWSTR};
    use windows::Win32::Graphics::Dxgi::{
        CreateDXGIFactory1, IDXGIAdapter3, IDXGIFactory1, DXGI_MEMORY_SEGMENT_GROUP_LOCAL,
    };
    use windows::Win32::System::Performance::{
        PdhAddEnglishCounterW, PdhCollectQueryData, PdhGetFormattedCounterArrayW, PdhOpenQueryW,
        PDH_FMT_COUNTERVALUE_ITEM_W, PDH_FMT_DOUBLE, PDH_NO_DATA,
    };

    struct Probe {
        query: isize,
        engine: isize,
        dedicated: isize,
        /// 0 when this Windows build has no per-process GPU memory counter.
        process: isize,
        /// Rate counters need two collects. The overview poll is the gap.
        primed: bool,
    }

    // ponytail: one process-wide query. First overview paint has no rate yet.
    static PROBE: Mutex<Option<Probe>> = Mutex::new(None);
    static PROBE_DEAD: Mutex<bool> = Mutex::new(false);
    static LAST_PROCESS: Mutex<std::collections::HashMap<(u32, AdapterKey), u64>> =
        Mutex::new(std::collections::HashMap::new());

    pub fn process_memory() -> std::collections::HashMap<(u32, AdapterKey), u64> {
        LAST_PROCESS
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }

    pub fn detect_dxgi() -> Vec<GpuInfo> {
        let mut found = enum_adapters();
        apply_usage(&mut found);
        found
    }

    fn enum_adapters() -> Vec<GpuInfo> {
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
                    let mut info =
                        windows::Win32::Graphics::Dxgi::DXGI_QUERY_VIDEO_MEMORY_INFO::default();
                    if adapter3
                        .QueryVideoMemoryInfo(0, DXGI_MEMORY_SEGMENT_GROUP_LOCAL, &mut info)
                        .is_ok()
                        && info.Budget > 0
                    {
                        vram_mb = info.Budget / (1024 * 1024);
                    }
                }
                let key = AdapterKey {
                    high: desc.AdapterLuid.HighPart,
                    low: desc.AdapterLuid.LowPart,
                };
                gpus.push(GpuInfo {
                    name,
                    vram_mb,
                    key: Some(key),
                    ..GpuInfo::default()
                });
            }
            gpus
        }
    }

    fn apply_usage(gpus: &mut [GpuInfo]) {
        let Some((usage, dedicated)) = sample() else {
            return;
        };
        for info in gpus.iter_mut() {
            let Some(key) = info.key else {
                continue;
            };
            if let Some(pct) = usage.get(&key) {
                info.usage_pct = Some(*pct);
            }
            if let Some(bytes) = dedicated.get(&key) {
                info.vram_used_mb = bytes / (1024 * 1024);
            }
        }
    }

    fn sample() -> Option<(
        std::collections::HashMap<AdapterKey, u8>,
        std::collections::HashMap<AdapterKey, u64>,
    )> {
        if PROBE_DEAD.lock().ok().as_deref() == Some(&true) {
            return None;
        }
        let mut slot = PROBE.lock().ok()?;
        if slot.is_none() {
            match open_probe() {
                Some(probe) => *slot = Some(probe),
                None => {
                    if let Ok(mut dead) = PROBE_DEAD.lock() {
                        *dead = true;
                    }
                    return None;
                }
            }
        }
        let probe = slot.as_mut()?;
        unsafe {
            let rc = PdhCollectQueryData(probe.query);
            if !probe.primed {
                probe.primed = true;
                return None;
            }
            if rc != 0 && rc != PDH_NO_DATA {
                return None;
            }
            if rc == PDH_NO_DATA {
                return None;
            }
            let engines = read_doubles(probe.engine);
            let memory = read_doubles(probe.dedicated);
            if probe.process != 0 {
                let rows = read_doubles(probe.process);
                if let Ok(mut last) = LAST_PROCESS.lock() {
                    *last = process_bytes_from_samples(rows.iter().map(|(n, v)| (n.as_str(), *v)));
                }
            }
            Some((
                usage_from_engine_samples(engines.iter().map(|(n, v)| (n.as_str(), *v))),
                bytes_from_memory_samples(memory.iter().map(|(n, v)| (n.as_str(), *v))),
            ))
        }
    }

    fn open_probe() -> Option<Probe> {
        unsafe {
            let mut query = 0isize;
            if PdhOpenQueryW(PCWSTR::null(), 0, &mut query) != 0 {
                return None;
            }
            let engine = add_counter(query, windows::core::w!(r"\GPU Engine(*)\Utilization Percentage"))?;
            let dedicated = add_counter(
                query,
                windows::core::w!(r"\GPU Adapter Memory(*)\Dedicated Usage"),
            )?;
            let process = add_counter(
                query,
                windows::core::w!(r"\GPU Process Memory(*)\Dedicated Usage"),
            )
            .unwrap_or(0);
            Some(Probe {
                query,
                engine,
                dedicated,
                process,
                primed: false,
            })
        }
    }

    unsafe fn add_counter(query: isize, path: PCWSTR) -> Option<isize> {
        let mut counter = 0isize;
        if PdhAddEnglishCounterW(query, path, 0, &mut counter) != 0 {
            return None;
        }
        Some(counter)
    }

    unsafe fn read_doubles(counter: isize) -> Vec<(String, f64)> {
        let mut size = 0u32;
        let mut count = 0u32;
        let _ = PdhGetFormattedCounterArrayW(counter, PDH_FMT_DOUBLE, &mut size, &mut count, None);
        if size == 0 {
            return Vec::new();
        }
        let item_size = std::mem::size_of::<PDH_FMT_COUNTERVALUE_ITEM_W>();
        let n = (size as usize / item_size).saturating_add(1);
        let mut items = vec![PDH_FMT_COUNTERVALUE_ITEM_W::default(); n];
        let mut bytes = (items.len() * item_size) as u32;
        if PdhGetFormattedCounterArrayW(
            counter,
            PDH_FMT_DOUBLE,
            &mut bytes,
            &mut count,
            Some(items.as_mut_ptr()),
        ) != 0
        {
            return Vec::new();
        }
        items
            .iter()
            .take(count as usize)
            .filter_map(|item| {
                let status = item.FmtValue.CStatus;
                if status != 0 && status != 1 {
                    return None;
                }
                let name = wide_to_string(item.szName)?;
                let value = item.FmtValue.Anonymous.doubleValue;
                if !value.is_finite() {
                    return None;
                }
                Some((name, value))
            })
            .collect()
    }

    unsafe fn wide_to_string(ptr: PWSTR) -> Option<String> {
        if ptr.is_null() {
            return None;
        }
        ptr.to_string().ok()
    }

    /// Optional NVML device-wide totals via `nvidia-smi` (avoids linking nvml.dll).
    pub fn nvml_device_totals() -> Option<Vec<u64>> {
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

    #[test]
    fn busiest_engine_is_per_type_not_a_sum_of_types() {
        let samples = [
            (
                "pid_1_luid_0x00000000_0x0000000A_phys_0_eng_0_engtype_3D",
                60.0,
            ),
            (
                "pid_2_luid_0x00000000_0x0000000A_phys_0_eng_3_engtype_3D",
                50.0,
            ),
            (
                "luid_0x00000000_0x0000000A_phys_0_eng_1_engtype_Compute",
                25.0,
            ),
            (
                "pid_9_luid_0x00000000_0x0000000B_phys_0_eng_0_engtype_3D",
                12.4,
            ),
        ];
        let map = usage_from_engine_samples(samples);
        assert_eq!(map[&AdapterKey { high: 0, low: 0xA }], 100);
        assert_eq!(map[&AdapterKey { high: 0, low: 0xB }], 12);
    }

    #[test]
    fn memory_counter_sums_nodes_for_one_adapter() {
        let samples = [
            ("luid_0x00000000_0x0000000A_phys_0", 1024.0 * 1024.0 * 100.0),
            ("luid_0x00000000_0x0000000A_phys_1", 1024.0 * 1024.0 * 50.0),
        ];
        let map = bytes_from_memory_samples(samples);
        assert_eq!(map[&AdapterKey { high: 0, low: 0xA }], 150 * 1024 * 1024);
    }

    #[test]
    fn nvidia_line_keeps_usage_for_the_fallback_path() {
        let gpu = parse_nvidia_line("NVIDIA GeForce RTX 4090, 24564, 1024, 37").unwrap();
        assert_eq!(gpu.vram_mb, 24564);
        assert_eq!(gpu.vram_used_mb, 1024);
        assert_eq!(gpu.usage_pct, Some(37));
        let idle = parse_nvidia_line("AMD does not use this, 8192, 0, [N/A]").unwrap();
        assert_eq!(idle.usage_pct, None);
    }

    #[test]
    fn model_is_marked_on_every_card_that_holds_it() {
        let mb = |n: f64| n * 1024.0 * 1024.0;
        let samples = [
            ("pid_4242_luid_0x00000000_0x0000000A_phys_0", mb(100.0)),
            ("pid_4242_luid_0x00000000_0x0000000A_phys_1", mb(40.0)),
            ("pid_4242_luid_0x00000000_0x0000000B_phys_0", mb(8.0)),
            ("pid_99_luid_0x00000000_0x0000000C_phys_0", mb(200.0)),
        ];
        let map = process_bytes_from_samples(samples);
        let mut gpus = vec![
            GpuInfo {
                name: "Arc A770".into(),
                key: Some(AdapterKey { high: 0, low: 0xA }),
                ..GpuInfo::default()
            },
            GpuInfo {
                name: "Radeon".into(),
                key: Some(AdapterKey { high: 0, low: 0xB }),
                ..GpuInfo::default()
            },
            GpuInfo {
                name: "RTX 5070".into(),
                key: Some(AdapterKey { high: 0, low: 0xC }),
                ..GpuInfo::default()
            },
        ];
        assign_hosted(&mut gpus, &map, 4242, "qwen3.gguf");
        assert_eq!(gpus[0].hosted_model.as_deref(), Some("qwen3.gguf"));
        assert_eq!(gpus[1].hosted_model, None);
        assert_eq!(gpus[2].hosted_model, None);
    }
}
