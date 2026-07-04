# Readiness: cli

| Field | Value |
| --- | --- |
| Path | `app/src/main.rs` |
| Overall | **2.6 / 5** |
| Label | `mvp-partial` |
| Reviewed | 2026-07-04 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 3/5 | CLI is a thin caller of `ServerManager` (`app/src/main.rs`); matches architecture (UI/CLI → manager only). No IPC for multi-command control. |
| Implementation | 2/5 | `start` loads config, starts manager, prints READY, blocks on Ctrl+C then `stop`. `status` / `print-config` / `print-cmd` work on a fresh manager (status is usually `Stopped` — no attach to running process). `stop` and `restart` print an error and exit failure (lines 78–84). |
| Tests | 1/5 | Manual only (documented in `docs/development.md`); no CLI tests. |
| Docs | 4/5 | `docs/development.md` documents commands and explicitly notes stop/restart MVP limits. |
| Windows readiness | 3/5 | `tokio::signal::ctrl_c` works on Windows for foreground stop; PowerShell scripts invoke `winserve.exe`. |

## Gaps

- `stop` / `restart` not implemented (need long-running manager process).
- `status` does not observe an already-running `winserve start`.
- No automated CLI smoke tests.

## Next actions (ordered)

1. Document/implement a single-instance or PID file so status/stop can attach.
2. Implement `stop`/`restart` against that owner, or keep CLI start-only and rely on scripts.
3. Smoke-test script: `print-cmd` then `start` with a mock binary.
