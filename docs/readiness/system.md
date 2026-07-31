# Readiness: system

| Field | Value |
| --- | --- |
| Path | `app/src/system/*` |
| Overall | **3.8 / 5** |
| Label | `mvp-ready` |
| Reviewed | 2026-07-31 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 4/5 | Modules `gpu`, `memory`, `network` and `HardwareInfo` / `detect()` in `app/src/system/mod.rs` match architecture and `docs/research/03-hardware-detection.md`. Inventory only — no layer calculator. |
| Implementation | 4/5 | Windows: DXGI adapter enum + dedicated VRAM (`gpu.rs` `win::detect_dxgi`); optional `nvidia-smi` for device totals / fallback when DXGI empty. RAM via `sysinfo` (`memory.rs`). `network::port_available` via `TcpListener::bind`. Soft miss: empty GPU list → CPU path, never fails `detect()`. NVML FFI deferred. |
| Tests | 3/5 | `port_available` bind/release; `memory::detect` reports RAM; `detect_gpus` does not panic. DXGI adapter identity is an **environment gate** (known NVIDIA Windows host), not simulated in CI. |
| Docs | 4/5 | Research deep dive (`docs/research/03-hardware-detection.md`); operator coverage in configuration/backend GPU rules; DXGI known-hardware proof documented as env gate (this scorecard). |
| Windows readiness | 4/5 | DXGI + nvidia-smi path on Win10/11; CI compiles Windows code on `windows-latest`. |

## Gaps

- NVML FFI not implemented (nvidia-smi CLI only for device-wide totals).
- DXGI non-empty adapter list not asserted in automated CI (requires known GPU host).
- Memory totals not yet consumed by argv policy (inventory only).

## Environment gate (DXGI)

Do **not** invent fake DXGI certainty in unit tests. Proof that `detect_dxgi` returns a non-empty adapter list belongs on a known NVIDIA Windows host (operator or optional matrix), not on macOS/Linux CI sandboxes.

## Next actions (ordered)

1. Optional NVML when nvidia-smi is insufficient.
2. Optional matrix job on a known NVIDIA CI host asserting non-empty GPU list.
3. Keep auto policy in `runtime/llama.rs` only — no layer math here.
