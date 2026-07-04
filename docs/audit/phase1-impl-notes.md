# Phase 1 implementation notes (leftovers pass)

**Branch:** `dev`  
**Date:** 2026-07-04  
**Scope:** PLAN milestones A–D code (except **D6** readiness refresh — completed in follow-up docs pass).

Readiness scores refreshed in D6 (overall **3.0/5**).

---

## Verification (this host)

| Check | Result |
| --- | --- |
| `cargo test -p winserve` | **14 passed**, 0 failed (non-Windows) |
| `cargo check -p winserve` | OK |
| `cargo clippy -p winserve --all-targets` | OK (no warnings) |
| `python3 scripts/check_doc_drift.py` | **doc drift OK** |
| `TODO` / `FIXME` in `app/src` | none |

---

## What landed (A–D, code)

### A — start path / pin

| Item | Location |
| --- | --- |
| **A1** Pin `llama-server` **b9866** | `bin/README.md` |
| **A3** Fail-hard if binary or model missing | `server/manager.rs` `start()` |

**A2** (operator smoke: `start` → `GET /v1/models`) still needs Windows + binary + GGUF; not automated here.

### B — Windows process ownership

| Item | Location |
| --- | --- |
| **B1** Job Object `KILL_ON_JOB_CLOSE` | `runtime/process.rs` `win::assign_to_kill_on_close_job` + `Drop` closes job |
| **B2** `CREATE_NEW_PROCESS_GROUP` + `CTRL_BREAK_EVENT` | `runtime/process.rs` spawn flags + `win::send_ctrl_break` |
| **B3** Port preflight fails start | `server/manager.rs` + `system/network.rs` |
| **B4** Exit watcher → `Status::Crashed` | `server/manager.rs` `get_status()` |

### C — Hardware defaults

| Item | Location |
| --- | --- |
| **C1** DXGI adapter + VRAM | `system/gpu.rs` `win::detect_dxgi` |
| **C2** Device totals via `nvidia-smi` (NVML FFI deferred) | `system/gpu.rs` `nvml_device_totals` / `detect_nvidia_smi` |
| **C3–C4** Auto: no GPU → `-ngl 0`; GPU → `--fit on` | `runtime/llama.rs` + unit tests |

### D — Tests & polish (except D6)

| Item | Location |
| --- | --- |
| **D1** Config load/validate, YAML roundtrip | `server/config.rs` tests |
| **D2** Health timeout when nothing listens | `server/health.rs` tests |
| **D3** Spawn/stop stub (`sleep` / `ping`) | `runtime/process.rs` tests |
| **D4** Log line timestamps (`ts=<epoch>`) | `server/logs.rs` |
| **D5** Stronger validation (port, host, layers, context; model path warnings) | `server/config.rs` |
| **D6** Readiness scorecards | done (see [readiness/README.md](../readiness/README.md)) |

Also: RAM via **sysinfo** (`system/memory.rs`).

---

## Windows-only code — needs `windows-latest` CI

Cross-platform unit tests pass on macOS/Linux, but the following paths are **`#[cfg(windows)]` only** and are **not compiled or exercised** on non-Windows runners:

| Module | Windows-only surface | What to validate on `windows-latest` |
| --- | --- | --- |
| `runtime/process.rs` | Job Object assign/close; `CREATE_NEW_PROCESS_GROUP`; `GenerateConsoleCtrlEvent(CTRL_BREAK)` | `spawn_and_stop_sleep_command` (uses `cmd /C ping`); stop returns; no orphan after manager kill (manual or integration) |
| `system/gpu.rs` | DXGI `CreateDXGIFactory1` / `EnumAdapters1` / optional `IDXGIAdapter3` budget | `detect_gpus_does_not_panic`; on NVIDIA hosts, non-empty list + VRAM when `nvidia-smi` present |
| `app/Cargo.toml` | `windows` crate features (JobObjects, Console, Dxgi, …) | `cargo check` / `cargo test -p winserve` must compile |

Cross-platform but Windows-relevant:

| Surface | Note |
| --- | --- |
| `runtime/llama.rs` `default_binary` | Resolves `bin/llama-server.exe` on Windows |
| `nvidia-smi` fallback | Used when DXGI empty (or for VRAM totals); requires driver tools on PATH |
| Full **A2** smoke | Needs `bin/llama-server.exe` (b9866) + GGUF; not in CI by default |

**Recommended CI job (next):** `windows-latest` → `cargo test -p winserve` + `cargo clippy -p winserve --all-targets`. Optional: matrix step that skips if no GPU / no binary.

---

## Leftovers pass actions

- No compile/test fixes required on this host.
- No obsolete `TODO` comments in `app/src` to remove.
- Clippy-clean; no unused-import cleanup needed.
- Doc drift refreshed in D6 readiness pass.

---

## Still open (not this pass)

1. **A2** — operator smoke on real Windows hardware.
2. Milestones **E** (desktop) / **F** (installer).
3. Optional: orphan-after-manager-kill integration test; suspended job assign.
