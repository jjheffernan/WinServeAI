# Readiness: server-logs

| Field | Value |
| --- | --- |
| Path | `app/src/server/logs.rs` |
| Overall | **3.8 / 5** |
| Label | `mvp-ready` |
| Reviewed | 2026-07-30 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 4/5 | Unified streams under one dir: `server.log`, `llama.log`, `error.log` via `LogSinks` — matches architecture layout. Size/age rotation + retention via `logging.max_bytes` / `max_age_secs` / `keep`. No log levels. |
| Implementation | 4/5 | `LogSinks::open` creates dir and append-only rotating files; `server`/`llama`/`error` write + flush. Lines prefixed with `ts=<unix-epoch-seconds>`. Manager wires llama stdout/stderr into `llama.log`. Rotate closes handle before rename (Windows-safe). |
| Tests | 3/5 | Temp-dir tests: timestamp prefix, size rotation → `*.1`, age rotation on open. |
| Docs | 4/5 | `docs/logging.md` + `docs/configuration.md` list files, rotation knobs, and capture requirements matching code. |
| Windows readiness | 4/5 | Std filesystem append + close-before-rename rotation works on Windows; paths from config (`logging.dir`). |

## Gaps

- Lock poison / write failures are silently ignored in `write_line`.
- No ISO-8601 timestamps (epoch `ts=` only).

## Next actions (ordered)

1. Optional ISO-8601 timestamps if operators need human-readable times.
2. Surface write failures instead of swallowing them.
