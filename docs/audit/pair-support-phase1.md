# Pair-support notes — Phase 1 (milestones A–D)

**Audience:** implementers on `dev`  
**Scope:** process Job Object + CTRL_BREAK, DXGI/NVML, tests, bin pin, log timestamps, config validation  
**Date:** 2026-07-04  
**Code reviewed:** `app/src` as of this write (stubs for job/CTRL_BREAK/DXGI)

No readiness score changes in this doc.

---

## 1. Windows Job Object + CTRL_BREAK

### Minimal correct sequence

**Spawn (reap on manager death):**

```text
CreateJobObject
  → SetInformationJobObject(JobObjectExtendedLimitInformation,
       LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE)   // 0x2000
  → spawn child CREATE_SUSPENDED | CREATE_NEW_PROCESS_GROUP [| CREATE_NO_WINDOW]
  → AssignProcessToJobObject(job, child)
  → ResumeThread
  → keep job handle in manager only (do not inherit into child)
```

Prefer **`process-wrap`** Tokio wrappers over hand-rolled Win32:

```text
CreationFlags(CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW)
  .wrap(JobObject)      // suspended assign + resume; KILL_ON_JOB_CLOSE
  .wrap(KillOnDrop)
```

Order: `CreationFlags` **before** `JobObject` (or include `CREATE_SUSPENDED` in flags). Do **not** call `Command::creation_flags` directly if using `JobObject` — the wrapper must see the flags.

**Stop (cooperative, then hard):**

```text
AttachConsole(child_pid) if parent has no shared console
  → SetConsoleCtrlHandler(NULL, TRUE)  // ignore CTRL_BREAK in parent
  → GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, process_group_id)
       // group_id == child PID when spawned with CREATE_NEW_PROCESS_GROUP
  → wait(grace)
  → if still alive: TerminateJobObject / child.kill
  → FreeConsole / restore handler
  → drop job handle (KILL_ON_JOB_CLOSE safety net)
```

### Citations

| Topic | URL |
| --- | --- |
| Job Objects | https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects |
| `KILL_ON_JOB_CLOSE` | https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_basic_limit_information |
| Suspended assign race | https://stackoverflow.com/questions/24012773/c-winapi-how-to-kill-child-processes-when-the-calling-parent-process-is-forcefully-terminated |
| Kill children on parent death | https://stackoverflow.com/questions/3342941/kill-child-process-when-parent-process-is-killed |
| Console process groups | https://learn.microsoft.com/en-us/windows/console/console-process-groups |
| `GenerateConsoleCtrlEvent` | https://learn.microsoft.com/en-us/windows/console/generateconsolectrlevent |
| CTRL_BREAK vs CTRL_C for groups | https://github.com/dotnet/docs/issues/53173 · https://stackoverflow.com/questions/40762545/how-to-send-a-ctrlc-sigint-to-a-subprocess-in-windows |
| AttachConsole pattern | https://stackoverflow.com/questions/813086/can-i-send-a-ctrl-c-sigint-to-an-application-on-windows |
| process-wrap | https://docs.rs/process-wrap · https://github.com/watchexec/process-wrap |

Full research: [`docs/research/01-windows-process.md`](../research/01-windows-process.md).

### Pitfalls (do not skip)

| Pitfall | Why it bites |
| --- | --- |
| **`CTRL_C_EVENT` + non-zero group ID** | API returns success; **no process receives it**. Must use **`CTRL_BREAK_EVENT`**. |
| **No `AttachConsole`** | GUI/tray/service parents often lack a shared console; `GenerateConsoleCtrlEvent` is a no-op without attach (or a tiny console helper). |
| **Job handle inherited by child** | Child holds a handle → job stays alive after parent death → **orphans survive**. Keep handle in manager only. |
| **Assign after child runs** | Race: child can fork/work before assignment. Spawn **suspended**, assign, then resume. |
| **`kill_on_drop` alone** | Only helps while manager is alive and cooperative. Task Manager kill of manager needs Job Object. |
| **`CREATE_NO_WINDOW` + attach** | May fail attach on some parents; fallback = grace timeout + job kill (always works, no flush). Validate on pinned `llama-server`. |
| **Parent already in a job** | Nested jobs / `IsProcessInJob` — installer/sandbox may block assign. |

---

## 2. DXGI VRAM (minimal)

**Goal:** adapter list + dedicated VRAM + process budget. Inventory only — **no** layer calculator.

```text
CreateDXGIFactory1 / CreateDXGIFactory2
  → for i in 0..:
       EnumAdapters1(i) → IDXGIAdapter1
       GetDesc1 → Description, DedicatedVideoMemory, VendorId
       QueryInterface → IDXGIAdapter3
       QueryVideoMemoryInfo(NodeIndex=0, DXGI_MEMORY_SEGMENT_GROUP_LOCAL)
         → Budget, CurrentUsage
```

### Notes

- **Primary API:** [`IDXGIAdapter3::QueryVideoMemoryInfo`](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_4/nf-dxgi1_4-idxgiadapter3-queryvideomemoryinfo) with `DXGI_MEMORY_SEGMENT_GROUP_LOCAL`.
- **Inventory totals:** `DXGI_ADAPTER_DESC::DedicatedVideoMemory` / `Description` (all vendors).
- **Budget vs free:** DXGI `Budget` / `CurrentUsage` are **this process**. Device-wide free on NVIDIA = optional **NVML** (`nvmlDeviceGetMemoryInfo`), not DXGI.
- **WDDM:** NVML **per-process** memory is N/A — do not use for auto config.
- **Soft miss:** no DXGI / no GPU → empty `gpus` → CPU path (`-ngl 0`). Never fail `detect()`.
- **Skip Microsoft Basic Render** (or treat as non-GPU) so auto does not think a software adapter is CUDA.
- Sample: [SO: DirectX get VRAM](https://stackoverflow.com/questions/35527315/directx-get-vram-used-by-game). Research: [`docs/research/03-hardware-detection.md`](../research/03-hardware-detection.md).

**Auto policy (feeds `runtime/llama.rs`):** any real GPU + `gpu.auto` + `layers: auto` → `--fit on` (omit `-ngl`). No GPU → `--n-gpu-layers 0`. Never invent `-ngl 99` for auto.

---

## 3. Test patterns (no `llama-server` binary)

Unit-test these on any host (macOS CI included):

| Target | What to assert |
| --- | --- |
| **`Config::load` / `validate`** | Valid YAML → Ok; `port == 0` → err; empty `model.path` → err; bad YAML → Parse; optional: non-numeric `gpu.layers` when not `"auto"`. |
| **`build_command` argv snapshots** | Table-driven: `(config, hardware) → args`. Cases: auto+empty GPUs → `-ngl 0`; auto+GPU → `--fit on` (no `-ngl`); explicit layers `"12"` → `-ngl 12`; `flash_attention` → `-fa on`; host/port/model/ctx present. |
| **`port_available`** | Bind a `TcpListener` on `127.0.0.1:0`, read port, assert `port_available` is false for that port; drop listener, assert true (or use a known-free high port). |
| **`default_binary`** | Path ends with `bin/llama-server.exe` on Windows cfg / `bin/llama-server` otherwise. |
| **URL helpers** | `base_url` / `openai_v1_url` format only. |

**Health without binary:** mock HTTP server returning 503 then 200 on `/v1/models`; assert `wait_until_ready` succeeds / times out.

**Process (Windows CI only):** tiny stub `.exe` that sleeps / exits on CTRL_BREAK — spawn under job, drop parent job handle, assert child gone; grace-timeout → kill path. Do **not** require real `llama-server` for unit tests.

---

## 4. Bug checklist (common mistakes)

- [ ] **Spawn ≠ Ready** — set `Starting` on spawn; `Ready` only after `GET /v1/models` 200 (or documented `/health` 200). PID alive is not serving.
- [ ] **`-ngl 99` kills `--fit`** — never pass both; auto path must omit `-ngl` when using `--fit on`.
- [ ] **Force-kill only** — grace wait with no-op signal is still hard-kill; implement real `CTRL_BREAK` or document job-kill-only fallback.
- [ ] **Job handle in child** — breaks `KILL_ON_JOB_CLOSE` orphan guarantee.
- [ ] **Port in use = soft warn only** — sleep/wake zombies hold the port ([llama.cpp #20648](https://github.com/ggml-org/llama.cpp/discussions/20648)); preflight should fail or reclaim **owned** port, not only log.
- [ ] **No exit watcher** — child dies while `Ready` must become `Crashed` without waiting for next CLI poll.
- [ ] **Empty GPU list forever** — stub `detect_gpus() → []` forces CPU path even on gaming PCs.
- [ ] **Restart on model-load exit** — llama-server exit `1` on bad model is permanent; do not blind-restart.
- [ ] **Global `taskkill /IM llama-server.exe`** — kills user-owned instances; prefer port/PID we own.
- [ ] **Raw flags outside `runtime/llama.rs`** — config/UI must not invent argv.

---

## 5. Current code review (caveman-style)

One line: location · problem · fix.

- `runtime/process.rs:L116-122`: 🔴 `send_ctrl_break` is no-op — implement `AttachConsole` + `GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, pid)` or accept job-kill-only and drop the grace pretence.
- `runtime/process.rs:L35-50`: 🔴 no Job Object — add `process-wrap` `JobObject` + `KillOnDrop`; keep job handle out of child.
- `runtime/process.rs:L45-50`: 🟡 `creation_flags(CREATE_NEW_PROCESS_GROUP)` alone — must go through `process-wrap` `CreationFlags` when Job Object lands, or flags are invisible to the wrapper.
- `system/gpu.rs:L10-13`: 🔴 `detect_gpus` always `[]` — DXGI enum or auto GPU never leaves CPU `-ngl 0`.
- `runtime/llama.rs:L41-43`: 🟡 `auto=false` + `layers=auto` uses `-ngl 99` — prefer `--fit on` or `0`/`explicit` only; `99` disables fit and surprises operators.
- `server/manager.rs:L122-125`: 🟡 port busy only warns — fail start or reclaim listener PID on configured port before spawn.
- `server/manager.rs:L175-184`: 🟡 `Crashed` only on `get_status` poll — add wait/exit task so unexpected death is observed without a status call.
- `server/manager.rs:L143-155`: ✅ readiness gate correct — `Ready` only after health; keep this when wiring job/stop.
- `server/logs.rs:L55-59`: 🔵 no timestamps — prefix ISO-8601 (or local) on every line for forensics.
- `server/config.rs:L135-142`: 🔵 weak validate — port range, warn if model path missing at load (start already hard-fails on missing file).
- tests: 🔴 zero automated tests — add config + argv + `port_available` first (no binary needed).

**Looks good:** single `ServerManager`; flags only in `llama.rs`; binary/model missing → `Failed` with clear messages; health polls `/v1/models` and treats 503 as loading.

---

## Implementer quick map

| Milestone | Primary files |
| --- | --- |
| B1–B2 Job + CTRL_BREAK | `app/src/runtime/process.rs` |
| B3 port preflight | `app/src/system/network.rs`, `manager.rs` |
| B4 exit → Crashed | `manager.rs` (+ process `wait`) |
| C1–C2 DXGI/NVML | `app/src/system/gpu.rs` |
| C3–C4 auto `--fit` | `app/src/runtime/llama.rs` + hardware |
| D1–D2 tests | `config.rs`, `llama.rs`, `health.rs`, `network.rs` |
| D4 timestamps | `app/src/server/logs.rs` |
| D5 validation | `app/src/server/config.rs` |
| A1 bin pin | `bin/README.md` (binary not committed) |

Deeper background: [`docs/research/01-windows-process.md`](../research/01-windows-process.md), [`docs/research/03-hardware-detection.md`](../research/03-hardware-detection.md), [`docs/PLAN.md`](../PLAN.md).
