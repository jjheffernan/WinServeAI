# Runtime (llama.cpp)

> **Readiness:** runtime-llama 2.8/5 (`mvp-partial`), runtime-process 2.4/5 (`scaffold`), server-manager 2.8/5 (`mvp-partial`) — details in [readiness/runtime-llama.md](./readiness/runtime-llama.md), [readiness/runtime-process.md](./readiness/runtime-process.md), [readiness/server-manager.md](./readiness/server-manager.md)

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
- Windows: `CREATE_NEW_PROCESS_GROUP` (for a future cooperative stop signal)
- `kill_on_drop(true)` so a dropped manager handle does not leave an orphan **while the manager process is still exiting cooperatively**

Port already in use is logged as a warning; MVP does not auto-kill foreign listeners. Missing binary or model fails before spawn (`Failed`).

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

Today, cooperative console signaling on Windows is a stub: after grace, the child is force-killed. Unexpected exit while the manager still owns the child surfaces as `Crashed` on status poll.

Restart is `stop` then `start`.

## Windows process notes

**Intended guarantee:** when the manager is force-killed (Task Manager), `llama-server` must not orphan. That requires a Windows [Job Object](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects) with `KILL_ON_JOB_CLOSE` — spawn suspended, assign to the job, then resume; keep the job handle only in the manager (do not inherit it into the child). Prefer [`process-wrap`](https://docs.rs/process-wrap) over hand-rolled Win32.

**Intended stop path:** `CTRL_BREAK` to the process group → wait grace → terminate job / drop handle. Do not use `CTRL_C_EVENT` with a non-zero group ID (no process receives it).

**Not implemented yet:** Job Objects and real `CTRL_BREAK`. Current code relies on grace + hard kill and Tokio `kill_on_drop`. Details and open questions: [research/01-windows-process.md](./research/01-windows-process.md).

Job Objects do **not** fix a live-but-wedged `llama-server` after sleep/wake (port still held). Before spawn, treat port conflicts as errors or reclaim only PIDs we previously owned — never global `taskkill /IM llama-server.exe`.

## Pin policy

- Pin `llama-server` to a **`b####` release tag**, not `master`.
- Record tag + commit in release notes / vendor manifest when shipping.
- On each pin bump, re-check argv (`--fit`, `-fa`, `-ngl`) and readiness (`/v1/models` / `/health` bodies).

Auto-restart after crash is **off** for MVP.

## Out of scope

Multi-backend support, backend traits, plugins, chat UI, model downloads, and remote management. Abstractions for “many backends” only if a real second backend appears.

## See also

- [architecture.md](./architecture.md) — layering and ServerManager API
- [configuration.md](./configuration.md) — YAML schema
- [logging.md](./logging.md) — log files
- [api.md](./api.md) — OpenAI client surface
- [research/01-windows-process.md](./research/01-windows-process.md) — Job Objects, graceful stop
- [research/02-llama-readiness.md](./research/02-llama-readiness.md) — readiness and fit flags
