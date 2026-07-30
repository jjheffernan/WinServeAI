# Logging

> **Readiness:** 3.6/5 (`mvp-ready`) — details in [readiness/server-logs.md](./readiness/server-logs.md)

WinServeAI writes three append-only streams under one directory. `ServerManager` owns all writes; there is no separate logging package.

Default directory is `logs/` (config: `logging.dir`). The manager creates the directory on open.

```yaml
logging:
  dir: logs
  max_bytes: 10485760   # 10 MiB; 0 disables size rotation
  max_age_secs: 604800  # 7 days; 0 disables age rotation
  keep: 3               # retain server.log.1 … .N
```

## Files

| File | Contents |
| --- | --- |
| `server.log` | Manager lifecycle: config load, hardware summary, start command + argv, pid, READY URL, stop, exit code on stop |
| `llama.log` | `llama-server` stdout and stderr (line-tagged) |
| `error.log` | Start failures, readiness failure, unexpected child exit |

Implementation: [`app/src/server/logs.rs`](../app/src/server/logs.rs) (`LogSinks`). Each file is opened append-only; every line is prefixed with `ts=<unix-epoch-seconds>` and flushed immediately. No log levels.

## What is captured

### Manager lifecycle (`server.log`)

Typical lines:

- `config loaded; binary=…`
- `hardware: N gpu(s), M threads`
- port-in-use failures (start aborts when the configured port is not bindable)
- `starting {program} {args…}` — full command line
- `spawned pid=N`
- `READY http://host:port/v1`
- `stopping (grace 8s)` then `stopped exit=Some(code)` or `stopped exit=None`

### Child stdio (`llama.log`)

[`app/src/runtime/process.rs`](../app/src/runtime/process.rs) pipes both streams and forwards each line with a prefix:

```text
[stdout] …
[stderr] …
```

Both streams share one file. Model load errors, CUDA messages, and llama-server chatter usually appear here (often under `[stderr]`).

### Failures (`error.log`)

- Missing `llama-server` binary or model path
- Readiness never reached (`readiness failed: …`)
- Child died while managed (`llama-server exited unexpectedly` from `get_status`)

Exit codes are written on **intentional stop** (`stopped exit=…` in `server.log`). Unexpected exit currently logs a note only, not the code. `restart()` is stop then start; there is no separate “restart reason” line.

## Rotation

Before each write (and once on open), `LogSinks` rotates an active file when:

- `max_bytes > 0` and the file size is at least that many bytes, or
- `max_age_secs > 0` and the file’s mtime is at least that old.

Rotation renames `name` → `name.1` → `name.2` … up to `keep`, deleting the oldest. The active file is closed before rename (Windows-safe) and reopened empty.

**UI / tray:** `logs::tail_file` / `logs::tail_dir` return the last N lines for the desktop log viewer (`manager_logs`). Viewer only — no writes from the webview.
## Debugging tips

1. **Server never becomes ready** — read `error.log` first (missing binary/model, readiness failure). Then `server.log` for the exact `starting …` argv and any port warning.
2. **Model or GPU load failure** — read `llama.log`, especially `[stderr]` lines from llama-server.
3. **Was ready, then died** — `error.log` for the unexpected-exit note; `llama.log` for the last output before death.
4. **Reproduce a bad start** — copy the `starting …` line from `server.log` and run that command manually beside `bin/`.

See [architecture.md](architecture.md) for the unified-logs rule and process ownership.

## Sources / See also

### Internal

- [architecture.md](./architecture.md) — unified logs under `logs/`
- [backend.md](./backend.md) — spawn pipes stdout/stderr
- [configuration.md](./configuration.md) — `logging.dir`
- [api.md](./api.md) — readiness failures surface in `error.log`
- [research/01-windows-process.md](./research/01-windows-process.md) — exit codes and crash signals
- [research/05-server-manager.md](./research/05-server-manager.md) — lifecycle ownership
- [`app/src/server/logs.rs`](../app/src/server/logs.rs)
- [`app/src/runtime/process.rs`](../app/src/runtime/process.rs)

### Upstream

- [llama.cpp server README](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md) — child process chatter often on stderr

Canonical external URL index: [policies/SOURCES.md](./policies/SOURCES.md).
