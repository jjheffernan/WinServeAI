# Windows Process Lifecycle

> **Path note (appliance layout):** This note was written against an earlier `packages/*` monorepo. Map old paths to the current single crate:
> `packages/launcher` → `app/src/server/manager.rs` · `packages/process` → `app/src/runtime/process.rs` · `packages/llama` → `app/src/runtime/llama.rs` · `packages/api` / readiness → `app/src/server/health.rs` + `app/src/api/` · `packages/hardware` → `app/src/system/` · `packages/config` → `app/src/server/config.rs` · `packages/logging` → `app/src/server/logs.rs` · `packages/backend` / traits → **removed** (no backend trait).

Research for supervising `llama-server.exe` from the process runtime (`app/src/runtime/process.rs`, Phase 1 scaffolding). Complements [prior-art.md](../prior-art.md). Historical stub lived under `packages/process` — Tokio `Command` + `kill_on_drop(true)`, no Job Object, `graceful_stop` is hard-kill only.

**Goal:** one owner (Server Manager) that never orphans inference processes when the manager exits, crashes, or is force-killed; cooperative stop when possible; clear crash signals for optional restart.

---

## Job Objects (`KILL_ON_JOB_CLOSE`)

Windows [Job Objects](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects) group processes as a unit. Flag [`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_basic_limit_information) (0x2000): **closing the last handle to the job terminates every process in the job**, then destroys the job.

| Pattern | Detail |
| --- | --- |
| Create | `CreateJobObject` → `SetInformationJobObject(..., JobObjectExtendedLimitInformation, …)` with `LimitFlags = KILL_ON_JOB_CLOSE` |
| Assign race | Spawn **suspended** (`CREATE_SUSPENDED`), `AssignProcessToJobObject`, then resume — avoids child running before assignment ([SO](https://stackoverflow.com/questions/24012773/c-winapi-how-to-kill-child-processes-when-the-calling-parent-process-is-forcefully-terminated)) |
| Handle lifetime | Keep the job handle in the manager only; **do not inherit** it into the child (child holding a handle keeps the job alive after parent death) |
| Grandchildren | Default: children of job members join the job. Breakaway only if `BREAKAWAY_OK` / `SILENT_BREAKAWAY_OK` ([SO](https://stackoverflow.com/questions/33424492/windows-api-job-objects-dont-pass-on-to-grandchildren)) |
| Nested jobs | Parent may already be in a job (installer, sandbox). Classic limit: one process → one job unless nested jobs are available; check `IsProcessInJob` |

**Why it matters:** `taskkill /T` and Tokio `kill_on_drop` only help when the manager is still alive and cooperative. Job Objects are the OS-level guarantee when Task Manager kills the tray/manager.

Canonical writeups: [SO: auto-destroy children](https://stackoverflow.com/questions/53208/how-do-i-automatically-destroy-child-processes-in-windows), [SO: kill child when parent killed](https://stackoverflow.com/questions/3342941/kill-child-process-when-parent-process-is-killed), [Meziantou](https://www.meziantou.net/killing-all-child-processes-when-the-parent-exits-job-object.htm). Desktop precedents: [opencode JobObject](https://github.com/anomalyco/opencode/commit/ddd9c71cca1f30a8214174fc10975e2ff3bb4635), [CodeNomad](https://github.com/NeuralNomadsAI/CodeNomad/commit/1e58b9dd914e78728eabf40b5fcc645e885300f) (graceful stop, then drop job).

---

## `process-wrap` (preferred over hand-rolled Win32)

Crate: [watchexec/process-wrap](https://github.com/watchexec/process-wrap) · [docs.rs](https://docs.rs/process-wrap) · successor to `command-group`.

| Wrapper | Role |
| --- | --- |
| `JobObject` | Suspended spawn → assign → resume; job kill on drop / parent exit |
| `CreationFlags(...)` | Shim for flags (must use this, not `Command::creation_flags`, or `JobObject` cannot see them) |
| `KillOnDrop` | Tokio: replace `.kill_on_drop(true)` |

**Ordering:** `CreationFlags` **before** `JobObject`, or include `CREATE_SUSPENDED` in flags. `JobObject` always sets `CREATE_SUSPENDED` internally and resumes unless you asked to stay suspended.

```rust
// Tokio path (matches app/src/runtime/process.rs)
use process_wrap::tokio::*;
// CreationFlags(CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW)
//   .wrap(JobObject)
//   .wrap(KillOnDrop)
```

Community confirmation: [Rust users forum](https://users.rust-lang.org/t/killing-subprocesses-of-std-command/117905). Tauri note: [dormouse PR](https://github.com/diffplug/mouseterm/pull/41) — prefer explicit job ownership over relying on sidecar lifecycle.

**Phase 1 decision:** depend on `process-wrap` with `tokio1` + default Windows wrappers. Do not reimplement Job Objects in `winserve-process`.

---

## Graceful stop (console signals)

Two separate concerns ([good split](https://comcomponent.com/en/blog/2026/03/20/001-windows-app-safe-child-process-handling-job-object-exit-propagation-stdio-watchdog/)):

1. **Cooperative shutdown** — process group + console signal  
2. **Tree reaping** — Job Object (`TerminateJobObject` / drop handle)

### API facts

- Spawn with [`CREATE_NEW_PROCESS_GROUP`](https://learn.microsoft.com/en-us/windows/console/console-process-groups): child PID == process group ID.
- Send [`GenerateConsoleCtrlEvent`](https://learn.microsoft.com/en-us/windows/console/generateconsolectrlevent)`(CTRL_BREAK_EVENT, group_id)`.
- **Do not use `CTRL_C_EVENT` for a non-zero group ID** — API succeeds but **no process receives it** ([MSDN](https://learn.microsoft.com/en-us/windows/console/generateconsolectrlevent), [dotnet/docs#53173](https://github.com/dotnet/docs/issues/53173)).
- `CTRL_BREAK` cannot be disabled via `SetConsoleCtrlHandler` ignore-attribute; `CTRL_C` can.
- Caller must **share the child’s console**. GUI/tray parents often need `AttachConsole(pid)` + ignore own handler, or a tiny helper process that owns the console ([SO](https://stackoverflow.com/questions/813086/can-i-send-a-ctrl-c-sigint-to-an-application-on-windows), [SO](https://stackoverflow.com/questions/40762545/how-to-send-a-ctrlc-sigint-to-a-subprocess-in-windows)).

### Recommended stop sequence

```text
CTRL_BREAK to process group
  → wait(grace_timeout) for exit
  → if still alive: kill job / TerminateJobObject
  → drop job handle (KILL_ON_JOB_CLOSE safety net)
```

`CREATE_NO_WINDOW` is desirable for tray UX but may complicate console attach — **validate on pinned `llama-server` build** (open question in prior-art). Fallback that always works: timeout + job kill (hard stop, no flush).

Unix later: `SIGTERM` → wait → `SIGKILL` on process group via `process-wrap` `ProcessGroup`.

---

## Sleep / wake “zombie” processes

[llama.cpp discussion #20648](https://github.com/ggml-org/llama.cpp/discussions/20648) (Win11): after sleep/wake, the old `llama-server` can leave GPU, sit in system memory, and still hold the port while a **new** instance starts → duplicate processes, broken clients.

Community bat pattern:

1. Kill PID bound to configured port (`netstat` / equivalent)  
2. `taskkill /IM llama-server.exe /T` (aggressive name cleanup)  
3. Sleep ~5–7s for socket + GPU driver settle  
4. Refuse start if process still present  
5. Optional: Task Scheduler on Power-Troubleshooter Event ID 1 (resume)

**For WinServeAI:** Job Objects prevent orphans when the **manager** dies; they do **not** fix a live-but-wedged child after resume. Before every `spawn` (and optionally on resume notification):

- If configured port is in use by a PID we do not own → kill that PID (or fail with a clear error).  
- Prefer **port-based** cleanup over global `llama-server.exe` kill (user may run other instances).  
- Surface state `Failed` / `Crashed` if health never returns after wake; optional auto-restart is a policy choice (default off for MVP).

---

## Crash detection and restart policy

### Detection (process layer)

| Signal | Meaning |
| --- | --- |
| `try_wait` / `wait` → `Some(status)` | Process exited; record exit code |
| Exit code `1` (llama-server model load fail) | Permanent config/model error — **do not** restart blindly ([PR #9056](https://github.com/ggml-org/llama.cpp/pull/9056)) |
| Unexpected exit while `Ready` | Crash / OOM / driver — candidate for restart |
| Port listen but readiness never 200 | Stuck start — treat as failed start, not crashloop |

Liveness ≠ readiness: PID alive is not “serving”; poll **`GET /v1/models`** (WinServeAI primary readiness in `app/src/server/health.rs`; optional alternate `GET /health` when present — 503 loading / 200 ready) ([prior-art](../prior-art.md), [#20684](https://github.com/ggml-org/llama.cpp/issues/20684)).

### Policy patterns (steal, don’t invent)

OTP / supervisor classics ([Elixir Supervisor](https://elixir.hexdocs.pm/Supervisor.html), [ProcessKit-rs](https://github.com/ZelAnton/ProcessKit-rs/blob/main/docs/supervision.md), [opensandbox-supervisor](https://open-sandbox.ai/components/internal/supervisor/readme)):

| Knob | MVP default |
| --- | --- |
| Policy | `OnCrash` only (never restart clean stop / intentional stop) |
| Enabled | **off** (config flag); prior-art: optional single restart |
| `max_restarts` | 1–3 in a window, then surface `Crashed` and stop |
| Backoff | exponential + small jitter; cap (e.g. 1s → 30s) |
| `stable_after` | reset backoff if process stayed Ready ≥ N seconds |
| Burst / circuit breaker | if >N starts in M minutes → give up (crashloop) |
| Pre-start hook | port/PID cleanup (sleep-wake zombies) |

NSSM-style always-restart ([d4-ollama-win-service](https://github.com/internetics-net/d4-ollama-win-service)) is Phase 5 service mode, not tray MVP.

**Ownership:** process runtime (`app/src/runtime/process.rs`) reports exit events; Server Manager (`app/src/server/manager.rs`) owns restart policy and state machine (`Stopped → Starting → Ready | Failed → Stopping → Stopped` + `Crashed`).

---

## Recommendations for process runtime (`app/src/runtime/process.rs`)

1. **Add `process-wrap` (Tokio)** — `CreationFlags(CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW)` + `JobObject` + `KillOnDrop`. Remove bare `Command::kill_on_drop`.
2. **Keep the public surface small** — `ProcessSpec`, `spawn`, `graceful_stop(timeout)`, `kill`, `wait` / exit notification, `pid`, `is_running`. No health HTTP here.
3. **Stop path** — Windows: `CTRL_BREAK` to group → wait → job kill. Stub may start with job-kill-only if console attach is flaky; document the gap.
4. **Exit watcher** — background task or `wait()` future that yields `ProcessExit { code: Option<i32> }` so launcher can classify crash vs clean stop.
5. **Preflight helper (optional, same crate or launcher)** — `ensure_port_free(port)` / kill-by-port for owned port only; call before spawn and after sleep-wake if we subscribe to power events later.
6. **Tests** — tiny stub exe on Windows CI: spawn under job, kill parent handle, assert child gone; graceful_stop timeout path.
7. **Do not** — NSSM, Windows Service, global `taskkill /IM llama-server.exe`, auto-restart inside `process` (policy stays in launcher).

Gap vs current `lib.rs`: no job, no process group, `graceful_stop` == `kill`, no exit code surface, no port preflight.

---

## Concrete API sketch (Rust signatures only, no full impl)

```rust
use std::path::PathBuf;
use std::time::Duration;

pub struct ProcessSpec {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub working_dir: Option<PathBuf>,
    /// When set, refuse spawn (or kill occupant) if this TCP port is already bound.
    pub port: Option<u16>,
}

pub struct ProcessExit {
    pub code: Option<i32>,
}

pub struct ManagedProcess { /* child + job handle via process-wrap */ }

impl ManagedProcess {
    pub fn new(spec: ProcessSpec) -> Self;

    /// Spawn under Job Object (+ CREATE_NEW_PROCESS_GROUP on Windows).
    pub async fn spawn(&mut self) -> Result<()>;

    pub fn pid(&self) -> Option<u32>;
    pub fn is_running(&mut self) -> bool;

    /// CTRL_BREAK (Win) / SIGTERM (Unix), then hard-kill job after `grace`.
    pub async fn graceful_stop(&mut self, grace: Duration) -> Result<ProcessExit>;

    /// Immediate job/process kill.
    pub async fn kill(&mut self) -> Result<()>;

    /// Resolves when the child exits (crash or stop).
    pub async fn wait(&mut self) -> Result<ProcessExit>;
}

/// Optional: kill process listening on `port` if it is not `except_pid`.
#[cfg(windows)]
pub async fn reclaim_port(port: u16, except_pid: Option<u32>) -> Result<()>;
```

Launcher-owned (not in `process`):

```rust
pub struct RestartPolicy {
    pub enabled: bool,              // default false
    pub max_restarts: u32,          // e.g. 1
    pub window: Duration,           // burst window
    pub initial_backoff: Duration,
    pub max_backoff: Duration,
    pub stable_after: Duration,     // reset backoff after healthy uptime
}

// On ProcessExit while desired_state == Running:
//   if policy allows && !permanent_error(exit) { backoff; spawn } else { Crashed }
```

---

## Links

| Topic | URL |
| --- | --- |
| Job Objects | https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects |
| `KILL_ON_JOB_CLOSE` | https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_basic_limit_information |
| `GenerateConsoleCtrlEvent` | https://learn.microsoft.com/en-us/windows/console/generateconsolectrlevent |
| Console process groups | https://learn.microsoft.com/en-us/windows/console/console-process-groups |
| process-wrap | https://github.com/watchexec/process-wrap |
| process-wrap docs | https://docs.rs/process-wrap |
| Sleep/wake zombies | https://github.com/ggml-org/llama.cpp/discussions/20648 |
| `/health` semantics | https://github.com/ggml-org/llama.cpp/pull/9056 |
| `/health` under load | https://github.com/ggml-org/llama.cpp/issues/20684 |
| Job kill on parent death (SO) | https://stackoverflow.com/questions/3342941/kill-child-process-when-parent-process-is-killed |
| Auto-destroy children (SO) | https://stackoverflow.com/questions/53208/how-do-i-automatically-destroy-child-processes-in-windows |
| CTRL_BREAK vs CTRL_C (SO) | https://stackoverflow.com/questions/40762545/how-to-send-a-ctrlc-sigint-to-a-subprocess-in-windows |
| CTRL_C attach pattern (SO) | https://stackoverflow.com/questions/813086/can-i-send-a-ctrl-c-sigint-to-an-application-on-windows |
| CTRL_C group ID pitfall | https://github.com/dotnet/docs/issues/53173 |
| Job + signal split | https://comcomponent.com/en/blog/2026/03/20/001-windows-app-safe-child-process-handling-job-object-exit-propagation-stdio-watchdog/ |
| Internal prior art | [docs/prior-art.md](../prior-art.md) |
| Product process code (stub stop) | [`app/src/runtime/process.rs`](../../app/src/runtime/process.rs) |
| Operator runtime guide | [docs/backend.md](../backend.md) |
| Canonical external URLs | [docs/policies/SOURCES.md](../policies/SOURCES.md) |

---

## Open questions

1. **Does `llama-server` honor `CTRL_BREAK` on Windows** (flush / clean exit), or is timeout+job-kill the practical path? Validate on pinned release.
2. **`CREATE_NO_WINDOW` + `GenerateConsoleCtrlEvent`** — does attach-to-child-console work from a Tauri/tray parent, or do we need a console helper / skip graceful and only job-kill?
3. **Parent already in a job** — does `process-wrap` fail assign when manager is sandboxed/installed under a job? Need `IsProcessInJob` / nested-job handling?
4. **Port reclaim scope** — kill only PIDs we previously spawned (track PID file / in-memory), or any listener on configured port?
5. **Power resume** — subscribe to Windows resume events in Phase 2 tray, or only preflight-on-start?
6. **Permanent vs transient exits** — treat all non-zero as non-restartable until we map llama-server codes, or restart once on any unexpected exit?
