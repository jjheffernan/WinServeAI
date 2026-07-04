# Research: `backend.md` (runtime)

**Target:** rewrite `docs/backend.md` as the **runtime** doc (llama.cpp process boundary).  
**Sources:** [architecture.md](../../architecture.md), [01-windows-process.md](../01-windows-process.md), [02-llama-readiness.md](../02-llama-readiness.md), `app/src/runtime/`, `app/src/server/manager.rs`.

## Problem with current stub

`docs/backend.md` is a four-line stub. It correctly says there is **no** backend abstraction, but it does not document spawn, logs, stop, binary layout, or the Windows process guarantees we intend. Older research (`prior-art.md`, `05-server-manager.md`) still mentions `packages/backend` traits — **do not** carry that into the rewrite.

## Framing

Title may stay **Runtime (llama.cpp)** or **Backend runtime**. Body is about the **external process boundary**, not a pluggable backend layer.

```text
ServerManager  →  runtime/llama (argv) + runtime/process (spawn/stop)
                         │
                         ▼
                  bin/llama-server.exe   ← external system
                         │
                         ▼
                  OpenAI API  /v1
```

## Runtime boundary (our system vs external)

| Our system (`app/`) | External (`bin/`, models, DLLs) |
| --- | --- |
| YAML config | `llama-server.exe` |
| `runtime/llama.rs` → argv | GGUF model files |
| `runtime/process.rs` → spawn / stop | CUDA / runtime DLLs **beside** the binary |
| Logs under `logs/` | OpenAI routes on `/v1` |
| Readiness poll (`GET /v1/models`) | |

Rules:

1. **No backend trait, no plugins, no multi-provider layer.** One binary, one orchestrator.
2. **Raw llama.cpp flags only in `app/src/runtime/llama.rs`.** Config, UI, and `ServerManager` never invent `-ngl` / `--fit` strings.
3. **Process ownership only via `ServerManager` → `runtime/process`.** UI/CLI never spawn or kill `llama-server` directly.
4. **`bin/llama-server.exe` is not our code.** Pin a `b####` release; ship DLLs next to the exe (PATH is unreliable on Windows).

## `llama.rs` — flags only

Code truth (`app/src/runtime/llama.rs`):

| Responsibility | Detail |
| --- | --- |
| `default_binary(root)` | `{root}/bin/llama-server.exe` (Windows) / `llama-server` (else) |
| `build_command(config, hardware, binary)` | Only place that emits argv |

Current mapping (document as implemented; pin-specific drift lives in research/02):

| Config | Argv |
| --- | --- |
| `server.host` / `server.port` | `--host` / `--port` |
| `model.path` | `--model` |
| `runtime.context` | `--ctx-size` |
| `gpu.auto` + `layers: auto` + GPU present | `--fit on` |
| `gpu.auto` + `layers: auto` + no GPU | `--n-gpu-layers 0` |
| `layers: auto` without `gpu.auto` | `--n-gpu-layers 99` or `0` |
| explicit `layers` | `--n-gpu-layers {N}` |
| `runtime.flash_attention: true` | `-fa on` |

Anti-pattern (called out in research/02): inventing layer counts for “auto” in a way that disables `--fit`. Prefer omit `-ngl` / use `--fit on` when auto.

**Out of scope for `backend.md`:** full YAML schema (→ `configuration.md`), every upstream flag, OpenAI client examples (→ `api.md`).

## Process lifecycle

Owned by `ServerManager` (`app/src/server/manager.rs`), executed by `runtime/process.rs`.

### Start

```text
load_config → detect_hardware → validate paths
  → build_command (llama.rs)
  → spawn (process.rs): workdir = binary parent (DLL load path)
  → pipe stdout/stderr → logs/llama.log
  → wait GET /v1/models (120s) → Ready | Failed
```

Code notes:

- `CREATE_NEW_PROCESS_GROUP` on Windows (for future `CTRL_BREAK`).
- `kill_on_drop(true)` today — **not** a Job Object.
- Port preflight is a warning only (`port_available`), not auto-kill.
- Missing binary or model → `Failed` before spawn.
- Readiness timeout or probe failure → stop child, `Failed`.

### Logs

| File | Source |
| --- | --- |
| `logs/server.log` | Manager lifecycle (start, pid, READY, stop, exit) |
| `logs/llama.log` | Child stdout/stderr lines (`[stdout]` / `[stderr]` prefixes) |
| `logs/error.log` | Failures (missing binary/model, readiness fail, unexpected exit) |

Process module only forwards lines on a channel; `LogSinks` writes files. Detail stays thin here; point at [logging.md](../../logging.md).

### Stop

```text
Stopping → grace (8s) → force kill → Stopped
```

Code today (`process.rs`):

- Windows: `send_ctrl_break` is a **stub** (no-op); grace then `start_kill`.
- Non-Windows: `start_kill` immediately, then wait/force.
- Exit code recorded on manager (`last_exit`).

Unexpected exit while owned → `Crashed` on `get_status()`.

### Binary layout

```text
bin/
  llama-server.exe
  *.dll          # CUDA / runtime deps beside the exe
```

Spawn `current_dir` = `bin/` parent of the program so relative DLL resolution works. Models live under user paths from YAML, not under `bin/`.

## Job Objects (from `research/01-windows-process.md`)

**Not implemented yet.** Document as the intended Windows guarantee, not current code.

| Topic | Decision / note |
| --- | --- |
| Why | `kill_on_drop` / `taskkill` only help while the manager is alive. Job Objects reap children when Task Manager kills the tray/manager. |
| Flag | `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` — closing last job handle terminates all members |
| Assign race | Spawn **suspended** → `AssignProcessToJobObject` → resume |
| Handle | Keep job handle in manager only; **do not inherit** into child |
| Preferred crate | [`process-wrap`](https://docs.rs/process-wrap) (`JobObject` + `CreationFlags` + `KillOnDrop`) — do not hand-roll Win32 |
| Graceful stop | `CTRL_BREAK` to process group → wait grace → `TerminateJobObject` / drop handle |
| `CTRL_C` pitfall | Do **not** use `CTRL_C_EVENT` with non-zero group ID (no process receives it) |
| Sleep/wake | Jobs do **not** fix wedged live children after resume; port/PID preflight before spawn ([#20648](https://github.com/ggml-org/llama.cpp/discussions/20648)) |

Open questions (link, don’t expand): does pinned `llama-server` honor `CTRL_BREAK`? Does `CREATE_NO_WINDOW` break console attach from a tray parent?

## Explicit non-goals (keep out of `backend.md`)

- Backend traits, `packages/backend`, plugins, multi-backend roadmaps
- Chat UI, model downloads, HuggingFace, agents, RAG, MCP
- Full ServerManager state-machine essay (→ architecture / research/05)
- Full `/health` vs `/v1/models` debate — product uses **`GET /v1/models`** (code + architecture)
- Installer / licensing (→ installer docs)

## Outline for rewrite

1. Positioning: runtime boundary, no abstraction layer
2. Layout: `runtime/llama.rs`, `runtime/process.rs`, `bin/llama-server.exe`
3. Spawn sequence + readiness
4. Logs
5. Stop sequence
6. Windows notes (Job Objects intended; current gaps)
7. Pin / DLL policy (short)
8. Links to architecture, configuration, logging, research/01–02

## Code vs research gaps to mention briefly

| Area | Today | Intended |
| --- | --- | --- |
| Job Object | none | `process-wrap` + `KILL_ON_JOB_CLOSE` |
| Graceful stop | grace + hard kill | `CTRL_BREAK` then job kill |
| Port reclaim | warning only | optional kill of **owned** PID only |
| Auto-restart | off | stays off for MVP |
