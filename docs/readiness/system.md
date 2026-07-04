# Readiness: system

| Field | Value |
| --- | --- |
| Path | `app/src/system/*` |
| Overall | **3.6 / 5** |
| Label | `mvp-ready` |
| Reviewed | 2026-07-04 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 4/5 | Modules `gpu`, `memory`, `network` and `HardwareInfo` / `detect()` in `app/src/system/mod.rs` match architecture and `docs/research/03-hardware-detection.md`. Inventory only — no layer calculator. |
| Implementation | 4/5 | Windows: DXGI adapter enum + dedicated VRAM (`gpu.rs` `win::detect_dxgi`); optional `nvidia-smi` for device totals / fallback when DXGI empty. RAM via `sysinfo` (`memory.rs`). `network::port_available` via `TcpListener::bind`. Soft miss: empty GPU list → CPU path, never fails `detect()`. NVML FFI deferred. |
| Tests | 3/5 | `port_available` bind/release; `memory::detect` reports RAM; `detect_gpus` does not panic. No DXGI-specific assertions (Windows-only path). |
| Docs | 3/5 | Research deep dive (`docs/research/03-hardware-detection.md`); operator coverage in configuration/backend GPU rules. |
| Windows readiness | 4/5 | DXGI + nvidia-smi path on Win10/11; CI compiles Windows code on `windows-latest`. |

## Gaps

- NVML FFI not implemented (nvidia-smi CLI only for device-wide totals).
- DXGI results not asserted in automated tests (no-panic only).
- Memory totals not yet consumed by argv policy (inventory only).

## Next actions (ordered)

1. Optional NVML when nvidia-smi is insufficient.
2. Assert non-empty GPU list on a known NVIDIA CI host (optional matrix).
3. Keep auto policy in `runtime/llama.rs` only — no layer math here.
