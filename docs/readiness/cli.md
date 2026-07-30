# Readiness: cli

| Field | Value |
| --- | --- |
| Path | `app/src/main.rs` |
| Overall | **3.2 / 5** |
| Label | `mvp-partial` |
| Reviewed | 2026-07-30 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 4/5 | CLI is a thin caller of `ServerManager` (`app/src/main.rs`); `serve` owns lockfile+IPC; `status|stop|restart` attach. Matches architecture (UI/CLI → manager only). |
| Implementation | 4/5 | `serve` resident owner; `start` one-shot foreground; attach path for `status|stop|restart`; `print-config` / `print-cmd` as before. |
| Tests | 1/5 | Manual only (documented in `docs/development.md`); no CLI tests. |
| Docs | 4/5 | `docs/development.md` documents serve / attach / tray and lockfile paths. |
| Windows readiness | 3/5 | Named pipe + `%LOCALAPPDATA%\WinServeAI\manager.lock`; PowerShell scripts invoke `winserve.exe`. |

## Gaps

- No automated CLI smoke tests for attach vs one-shot `start`.
- Operator Windows proof that tray + CLI attach share one owner.

## Next actions (ordered)

1. Smoke-test script: `serve` + attach `status|stop`, and one-shot `start` Ctrl+C.
2. Optional: document recovery when lockfile is stale (already reclaimed on read).
