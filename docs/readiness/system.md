# Readiness: system

| Field | Value |
| --- | --- |
| Path | `app/src/system/*` |
| Overall | **1.6 / 5** |
| Label | `scaffold` |
| Reviewed | 2026-07-04 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 3/5 | Modules `gpu`, `memory`, `network` and `HardwareInfo` / `detect()` in `app/src/system/mod.rs` match architecture layout and `docs/research/03-hardware-detection.md` intent. |
| Implementation | 1/5 | `gpu::detect_gpus` returns `Vec::new()` with TODO for DXGI/NVML (`app/src/system/gpu.rs`). `memory::detect` returns `total_mb: 0` (`memory.rs`). Only real probes: `cpu_threads` via `available_parallelism`, and `network::port_available` via `TcpListener::bind`. |
| Tests | 0/5 | No system tests. |
| Docs | 3/5 | Research deep dive exists (`docs/research/03-hardware-detection.md`); no dedicated operator guide (covered lightly in configuration/backend). |
| Windows readiness | 1/5 | Windows-specific DXGI/NVML not implemented; auto GPU defaults always see zero GPUs. |

## Gaps

- Empty GPU list forces CPU-only argv (`--n-gpu-layers 0`) in auto mode.
- Memory totals unused and always zero.
- No tests for `port_available`.

## Next actions (ordered)

1. Implement DXGI adapter enumeration (name + VRAM when available).
2. Implement `GlobalMemoryStatusEx` or `sysinfo` for RAM.
3. Unit-test `port_available` bind/release behavior.
