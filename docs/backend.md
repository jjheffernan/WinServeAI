# Runtime (llama.cpp)

> **Readiness:** runtime-llama 3.6/5 (`mvp-ready`), runtime-process 3.8/5 (`mvp-ready`), server-manager 3.4/5 (`mvp-partial`) — details in [readiness/runtime-llama.md](./readiness/runtime-llama.md), [readiness/runtime-process.md](./readiness/runtime-process.md), [readiness/server-manager.md](./readiness/server-manager.md)

WinServeAI is a **process wrapper** around a pinned `llama-server` binary. There is **no** backend trait, plugin loader, or multi-provider layer.

```text
ServerManager
    │
    ├─ runtime/llama.rs     build argv (ONLY place with llama flags)
    └─ runtime/process.rs   spawn / monitor / stop
              │
              ▼
       bin/llama-server.exe   ← external system
              │
              ▼
       OpenAI API  http://host:port/v1
```

Clients talk to llama-server directly. WinServeAI starts the process, captures logs, waits until it is ready, and stops it cleanly.

## Boundary

| Our system | External system |
| --- | --- |
| YAML config (`config/default.yaml`) | `bin/llama-server.exe` |
| `app/src/runtime/llama.rs` → argv | GGUF model files (user paths) |
| `app/src/runtime/process.rs` → lifecycle | CUDA / runtime DLLs beside the binary |
| Unified logs under `logs/` | OpenAI-compatible routes on `/v1` |
| Readiness: `GET /v1/models` | |

Rules:

1. **Raw llama.cpp flags exist only in `app/src/runtime/llama.rs`.** Config and UI never store or invent flags.
2. **Only `ServerManager` owns the child process.** UI/CLI call `start` / `stop` / `status` — they never spawn or kill `llama-server` themselves.
3. **`bin/llama-server.exe` is not our code.** Pin a `b####` release and ship required DLLs next to the exe (Windows `PATH` is unreliable).

## Binary layout

```text
bin/
  llama-server.exe
  *.dll                 # CUDA / runtime deps, same directory
```

`runtime/llama.rs` resolves the binary as `{repo_or_install_root}/bin/llama-server.exe` (or `llama-server` on non-Windows). Spawn sets the working directory to the binary’s parent so adjacent DLLs load correctly.

Models are **not** shipped under `bin/`; the path comes from YAML (`model.path`).

## Spawn

Startup sequence (owned by `ServerManager`):

```text
load config → detect hardware → validate binary + model paths
  → build_command (llama.rs)
  → spawn (process.rs)
  → pipe stdout/stderr → logs/llama.log
  → poll GET /v1/models until 200 (timeout 120s)
  → Ready  |  Failed (stop child on failure)
```

Process details (`runtime/process.rs`):

- stdout and stderr are piped and forwarded line-by-line
- Windows: `CREATE_NEW_PROCESS_GROUP`, Job Object (`KILL_ON_JOB_CLOSE`), and `CTRL_BREAK` to the process group on stop
- `kill_on_drop(true)` as a cooperative safety net while the manager is exiting

Port already in use **fails start** (preflight bind check). Missing binary or model fails before spawn (`Failed`).

### Argv (`llama.rs` only)

Human YAML maps to flags here — nowhere else. Current mapping:

| Config | Argv |
| --- | --- |
| `server.host` / `server.port` | `--host` / `--port` |
| `model.path` | `--model` |
| `runtime.context` | `--ctx-size` |
| `gpu.auto` + `layers: auto` + GPU present | `--fit on` |
| `gpu.auto` + `layers: auto` + no GPU | `--n-gpu-layers 0` |
| explicit `gpu.layers` | `--n-gpu-layers {N}` |
| `runtime.flash_attention: true` | `-fa on` |

Prefer `--fit on` (or omit `-ngl`) for auto GPU allocation. Do not invent layer counts that disable fit. Full schema: [configuration.md](./configuration.md). Flag drift is pin-specific — see [research/02-llama-readiness.md](./research/02-llama-readiness.md).

### Readiness

Process alive ≠ ready. After spawn, the manager polls `GET {base}/v1/models`:

- **200** → `Ready`
- **503** → still loading (keep waiting)
- connect errors / other statuses → retry until deadline
- timeout or child exit during load → stop child, `Failed`

OpenAI compatibility is provided by llama-server itself. WinServeAI only ensures the process is up and answering.

## Logs

| File | Contents |
| --- | --- |
| `logs/server.log` | Manager lifecycle (start command, pid, READY, stop, exit code) |
| `logs/llama.log` | llama-server stdout/stderr |
| `logs/error.log` | Failures (missing paths, readiness timeout, unexpected exit) |

Always capture stdout, stderr, and exit codes. See [logging.md](./logging.md).

## Stop

```text
Stopping → grace (8s) → force kill → Stopped
```

`ServerManager::stop` calls `ChildProcess::stop` with an 8-second grace period, records the exit code, then returns to `Stopped`. `stop` is the only supported shutdown path for UI/CLI.

On Windows, stop sends `CTRL_BREAK` to the process group, waits grace (8s), then force-kills if still alive. Unexpected exit while the manager still owns the child surfaces as `Crashed` on status poll.

Restart is `stop` then `start`.

## Windows process notes

**Orphan prevention:** children are assigned to a Windows [Job Object](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects) with `KILL_ON_JOB_CLOSE`. Closing the job handle (manager exit / `Drop`) terminates remaining children. Keep the job handle only in the manager (do not inherit it into the child). Implementation is hand-rolled Win32 in `runtime/process.rs` (not `process-wrap`).

**Stop path:** `AttachConsole` + `GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, pid)` → wait grace → `start_kill` / drop job handle. Do not use `CTRL_C_EVENT` with a non-zero group ID (no process receives it). Details: [research/01-windows-process.md](./research/01-windows-process.md).

Job Objects do **not** fix a live-but-wedged `llama-server` after sleep/wake (port still held). Start **fails** if the configured port is not bindable — never global `taskkill /IM llama-server.exe`.

## Pin policy

- Pin `llama-server` to a **`b####` release tag**, not `master`.
- Record tag + commit in release notes / vendor manifest when shipping.
- On each pin bump, re-check argv (`--fit`, `-fa`, `-ngl`) and readiness (`/v1/models` / `/health` bodies).

Auto-restart after crash is **off** for MVP.

## Out of scope

Multi-backend support, backend traits, plugins, chat UI, model downloads, and remote management. Abstractions for “many backends” only if a real second backend appears.

## Sources / See also

### Internal

- [architecture.md](./architecture.md) — layering and ServerManager API
- [configuration.md](./configuration.md) — YAML schema
- [logging.md](./logging.md) — log files
- [api.md](./api.md) — OpenAI client surface
- [research/01-windows-process.md](./research/01-windows-process.md) — Job Objects, graceful stop
- [research/02-llama-readiness.md](./research/02-llama-readiness.md) — readiness and fit flags
- [research/05-server-manager.md](./research/05-server-manager.md) — state machine
- [`app/src/runtime/llama.rs`](../app/src/runtime/llama.rs) — only place with llama flags
- [`app/src/runtime/process.rs`](../app/src/runtime/process.rs) — spawn / stop
- [`app/src/server/health.rs`](../app/src/server/health.rs) — `GET /v1/models` readiness

### Upstream

- [llama.cpp `tools/server` README](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md) — flags, `/v1`, optional `/health`
- [Discussion #18049 — `--fit`](https://github.com/ggml-org/llama.cpp/discussions/18049) — auto VRAM; `-ngl` disables fit for layers
- [Discussion #20648](https://github.com/ggml-org/llama.cpp/discussions/20648) — Win11 sleep/wake port zombies
- [Job Objects](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects) — `KILL_ON_JOB_CLOSE`
- [`GenerateConsoleCtrlEvent`](https://learn.microsoft.com/en-us/windows/console/generateconsolectrlevent) — `CTRL_BREAK` to process group
- [process-wrap](https://docs.rs/process-wrap) — preferred Job Object wrapper ([crates.io](https://crates.io/crates/process-wrap))
- [llama.cpp releases (`b####`)](https://github.com/ggml-org/llama.cpp/releases)

Canonical external URL index: [policies/SOURCES.md](./policies/SOURCES.md).
