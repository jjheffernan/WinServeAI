# Readiness: server-logs

| Field | Value |
| --- | --- |
| Path | `app/src/server/logs.rs` |
| Overall | **2.6 / 5** |
| Label | `mvp-partial` |
| Reviewed | 2026-07-04 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 3/5 | Unified streams under one dir: `server.log`, `llama.log`, `error.log` via `LogSinks` — matches architecture layout. No levels/rotation design. |
| Implementation | 3/5 | `LogSinks::open` creates dir and append-only files; `server`/`llama`/`error` write + flush (`app/src/server/logs.rs`). Manager wires llama stdout/stderr lines into `llama.log`. No timestamps, levels, or rotation. |
| Tests | 0/5 | No log tests. |
| Docs | 4/5 | `docs/logging.md` lists files, typical lines, and capture requirements matching code. |
| Windows readiness | 3/5 | Std filesystem append works on Windows; paths from config (`logging.dir`). |

## Gaps

- No timestamps on lines (harder to correlate incidents).
- No rotation / size limits.
- Lock poison / write failures are silently ignored in `write_line`.

## Next actions (ordered)

1. Prefix lines with ISO timestamps.
2. Unit test open + write to a temp dir.
3. Simple size-based rotation when logs grow large.
