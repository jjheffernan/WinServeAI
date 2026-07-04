# Research: logging.md

**Sources:** `app/src/server/logs.rs`, `app/src/server/manager.rs`, `app/src/runtime/process.rs`, `app/src/server/config.rs`, `config/default.yaml`  
**Architecture lock:** unified logs under `logging.dir` (default `logs/`); `ServerManager` owns writes; no separate logging package.

## Implementation (`LogSinks`)

`app/src/server/logs.rs` opens three append-only files in one directory:

| File | Method | Role |
| --- | --- | --- |
| `server.log` | `server(line)` | Manager lifecycle |
| `llama.log` | `llama(line)` | Child process stdio |
| `error.log` | `error(line)` | Failures / unexpected exit |

Behavior:

- `LogSinks::open(dir)` creates the directory (`create_dir_all`), then opens each file with `create(true).append(true)`.
- Each write is `writeln!` + `flush` under a `Mutex<File>`. Lock/write errors are swallowed (best-effort).
- No timestamps, levels, or rotation. Lines are plain text as passed by the caller.

Config: `logging.dir` (`LoggingSection`, default `"logs"` relative path). Manager opens sinks at `load_config` and again for the llama reader task on `start`.

## What is captured

### `server.log` (manager)

From `ServerManager`:

| Event | Example line |
| --- | --- |
| Config load | `config loaded; binary=…` |
| Hardware detect | `hardware: N gpu(s), M threads` |
| Port guard | `warning: port may already be in use` |
| Start command | `starting {program} {args…}` (full argv) |
| Spawn | `spawned pid=N` |
| Ready | `READY {openai_v1_url}` |
| Stop | `stopping (grace 8s)` then `stopped exit={code:?}` |

Exit codes appear on the **intentional stop** path only (`child.stop(8s)` → `stopped exit=…`). Stored in `last_exit` but not otherwise logged.

### `llama.log` (child stdio)

`ChildProcess::spawn` pipes stdout and stderr, reads line-by-line, and sends:

- `[stdout] {line}`
- `[stderr] {line}`

`start()` opens a second `LogSinks` and a tokio task that `recv`s the channel and calls `logs.llama(&line)`. Both streams land in one file, tagged by prefix.

### `error.log` (failures)

| Event | Line |
| --- | --- |
| Missing binary | `llama-server not found at …` |
| Missing model | `model not found: …` |
| Readiness timeout/fail | `readiness failed: {e}` |
| Unexpected child death | `llama-server exited unexpectedly` (`get_status` when `!is_running`) |

Crash path does **not** currently log an exit code (only the note above). `restart()` is stop+start with no dedicated “restart reason” line.

## Gaps vs old stub

Old `logging.md` claimed “Always capture: stdout, stderr, exit codes, restart reasons.”

| Claim | Actual |
| --- | --- |
| stdout / stderr | Yes → `llama.log` with `[stdout]` / `[stderr]` |
| exit codes | Only on graceful stop → `server.log` (`stopped exit=…`) |
| restart reasons | Not logged |

## Rotation (future)

No rotation, size limits, or retention. Files grow forever while append-open. Research checklist (`docs/research.md`) still has “Rotating logs” open. Prior-art notes NSSM-style rotation for Phase 5 service mode — not MVP. Doc should note rotation as a future improvement, not current behavior.

## Debugging tips (for user doc)

1. **Won’t start:** `error.log` (missing binary/model, readiness failed); `server.log` for command line and port warning.
2. **Model / CUDA / load errors:** `llama.log` (`[stderr]` often has llama-server diagnostics).
3. **Died after ready:** `error.log` unexpected-exit note; check `llama.log` for last lines before death.
4. **Reproduce argv:** `server.log` `starting …` line is the exact program + args.

## Doc outline for rewrite

1. Location / config (`logging.dir`)
2. Three-file table (accurate contents)
3. Capture pipeline (stdout/stderr tags, exit on stop)
4. Rotation: not implemented; future note
5. Debugging tips
6. Link `architecture.md` (unified logs rule)

No multi-backend, no `packages/logging` references (stale research/05 wording).
