# Readiness: cli

| Field | Value |
| --- | --- |
| Path | `app/src/main.rs` |
| Overall | **3.6 / 5** |
| Label | `mvp-partial` |
| Reviewed | 2026-07-31 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 4/5 | CLI is a thin caller of `ServerManager` (`app/src/main.rs`); `serve` owns lockfile+IPC; `status|stop|restart` attach. Matches architecture (UI/CLI → manager only). |
| Implementation | 4/5 | `serve` resident owner; `start` one-shot foreground; attach path for `status|stop|restart`; `print-config` / `print-cmd` as before. |
| Tests | 3/5 | `app/tests/cli_exit_codes.rs`: missing config, unknown command, `print-config`, `status` without resident → Stopped/success, `stop` without resident → failure. No live attach/`serve` smoke yet. |
| Docs | 4/5 | `docs/development.md` documents serve / attach / tray and lockfile paths. |
| Windows readiness | 3/5 | Named pipe + `%LOCALAPPDATA%\WinServeAI\manager.lock`; PowerShell scripts invoke `winserve.exe`. |

## Gaps

- No automated CLI smoke for live `serve` + attach `status|stop`.
- Operator Windows proof that tray + CLI attach share one owner.

## Next actions (ordered)

1. Optional: live attach smoke with fake resident (still host-safe).
2. Close with operator Windows proof that tray + CLI share one owner.
