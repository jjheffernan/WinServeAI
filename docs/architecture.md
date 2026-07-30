# Architecture

> **Module readiness dashboard:** [readiness/README.md](./readiness/README.md)

WinServeAI is a **Windows-native llama.cpp appliance wrapper**, not a multi-backend platform.

```text
CLI (winserve) ──┐
                 │  attach: status | stop | restart
winserve-tray ───┼──► ServerManager     (app/server)  ← only orchestration API
  (or serve)     │         │
                 │         ▼
                 │    runtime/llama     builds argv (ONLY place with llama flags)
                 │    runtime/process   spawn / monitor / kill (+ Job Object)
                 │         │
                 │         ▼
                 │    bin/llama-server.exe   ← external system
                 │         │
                 │         ▼
                 │    OpenAI API  http://host:port/v1
                 │
                 └── lockfile + named pipe (resident owner only)
```

**Resident owner** is either `winserve serve` or `winserve-tray`. It holds
`%LOCALAPPDATA%\WinServeAI\manager.lock` and listens on pipe `winserve-manager`
(`\\.\pipe\winserve-manager` on Windows). A second owner refuses with “already
running”. Foreground `winserve start` is one-shot (no lock/IPC).

## Explicit runtime boundary

| Our system | External system |
| --- | --- |
| config (YAML) | `llama-server.exe` |
| process control + Job Object | model GGUF files |
| lockfile + named-pipe IPC | CUDA/runtime DLLs beside the binary |
| logs | |
| readiness (`/v1/models`, `/health` fallback) | |

## ServerManager API

```text
load_config()
detect_hardware()
build_command()
start()
stop()
restart()
status() / get_status()
apply_config()   # tray/settings; blocked while Starting/Ready/Stopping
```

Startup: read config → detect GPU → build command → spawn → wait `/v1/models`
(then `/health` if needed) → READY.

Shutdown: signal → wait grace → force kill → flush logs. Tray quit calls
`stop()`; Job Object `KILL_ON_JOB_CLOSE` is the orphan backstop.

UI (tray) and CLI only call `ServerManager` — in-process in the owner, or via
IPC attach for `status` / `stop` / `restart`.

## Layout

```text
app/                 # crate winserve
  src/server/        # manager, config, health, logs, resident
  src/ipc/           # lockfile + pipe
  src/runtime/       # llama + process (external boundary)
  src/api/           # OpenAI URL helpers (passthrough)
  src/system/        # gpu, memory, network
apps/desktop/        # Tauri 2 → winserve-tray
bin/                 # llama-server.exe
config/              # default.yaml (+ FIRST_RUN.txt for install tree)
logs/                # server.log, llama.log, error.log
models/              # user models (not shipped)
installer/inno/      # Inno Setup
scripts/
docs/
```

## Rules (do not mess up)

1. **One backend:** llama.cpp only. No backend trait, no plugins.
2. **One orchestrator:** `ServerManager`. UI/CLI only call it (in-process or IPC).
3. **Raw flags only in `runtime/llama.rs`.**
4. **Config is YAML source of truth.**
5. **Unified logs** under `logs/`.
6. **No model downloads, chat UI, Docker, or multi-provider support.**
7. **One resident owner** — lockfile + pipe; never two managers fighting over the Job Object.

Abstractions for “many backends” come later *only if needed*.

## Sources / See also

### Internal

- [adr/0001-appliance-architecture.md](./adr/0001-appliance-architecture.md) — accepted decision
- [vision.md](./vision.md) — product category
- [backend.md](./backend.md) — runtime boundary
- [development.md](./development.md) — CLI serve / tray / IPC paths
- [first-run.md](./first-run.md) — empty `model.path`
- [api.md](./api.md) — OpenAI surface and readiness
- [research/05-server-manager.md](./research/05-server-manager.md) — state machine
- [research/01-windows-process.md](./research/01-windows-process.md) — Job Objects
- [research/02-llama-readiness.md](./research/02-llama-readiness.md) — `GET /v1/models`
- [specs/E-desktop.md](./specs/E-desktop.md) — tray shell
- [prior-art.md](./prior-art.md) — external analogues
- [comparison.md](./comparison.md) — WinServeAI vs competitor architecture families
- [research/06-competitor-architecture.md](./research/06-competitor-architecture.md) — ownership-family diagrams
- [audit/completion-audit.md](./audit/completion-audit.md) — implemented vs Windows-proven
- [`app/src/server/health.rs`](../app/src/server/health.rs) — readiness probe
- [`app/src/runtime/llama.rs`](../app/src/runtime/llama.rs) — argv only here
- [`app/src/ipc/`](../app/src/ipc/) — lockfile + pipe

### Upstream

- [llama.cpp `tools/server`](https://github.com/ggml-org/llama.cpp/tree/master/tools/server) — external OpenAI `/v1` server
- [Job Objects](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects) — orphan prevention (`KILL_ON_JOB_CLOSE` in `runtime/process.rs`)
- [Named Pipes](https://learn.microsoft.com/en-us/windows/win32/ipc/named-pipes) — localhost same-user IPC

Canonical external URL index: [policies/SOURCES.md](./policies/SOURCES.md).
