# Logging

WinServeAI writes three append-only streams under one directory. `ServerManager` owns all writes; there is no separate logging package.

Default directory is `logs/` (config: `logging.dir`). The manager creates the directory on open.

```yaml
logging:
  dir: logs
```

## Files

| File | Contents |
| --- | --- |
| `server.log` | Manager lifecycle: config load, hardware summary, start command + argv, pid, READY URL, stop, exit code on stop |
| `llama.log` | `llama-server` stdout and stderr (line-tagged) |
| `error.log` | Start failures, readiness failure, unexpected child exit |

Implementation: `app/src/server/logs.rs` (`LogSinks`). Each file is opened append-only; every line is flushed immediately. No log levels or timestamps.

## What is captured

### Manager lifecycle (`server.log`)

Typical lines:

- `config loaded; binary=…`
- `hardware: N gpu(s), M threads`
- `warning: port may already be in use` (best-effort port check)
- `starting {program} {args…}` — full command line
- `spawned pid=N`
- `READY http://host:port/v1`
- `stopping (grace 8s)` then `stopped exit=Some(code)` or `stopped exit=None`

### Child stdio (`llama.log`)

`app/src/runtime/process.rs` pipes both streams and forwards each line with a prefix:

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

Not implemented. Files grow for as long as the process appends to them. Size limits, rotation, and retention are future work (see open items in [research.md](research.md)).

## Debugging tips

1. **Server never becomes ready** — read `error.log` first (missing binary/model, readiness failure). Then `server.log` for the exact `starting …` argv and any port warning.
2. **Model or GPU load failure** — read `llama.log`, especially `[stderr]` lines from llama-server.
3. **Was ready, then died** — `error.log` for the unexpected-exit note; `llama.log` for the last output before death.
4. **Reproduce a bad start** — copy the `starting …` line from `server.log` and run that command manually beside `bin/`.

See [architecture.md](architecture.md) for the unified-logs rule and process ownership.
