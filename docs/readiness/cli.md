# Readiness: cli

| Field | Value |
| --- | --- |
| Path | `server/src/main.rs` |
| Overall | **3.6 / 5** |
| Label | `mvp-partial` |
| Reviewed | 2026-07-31 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 4/5 | CLI is a thin caller of `ServerManager` (`server/src/main.rs` + `cli.rs`); `serve` owns lockfile+IPC; attach for lifecycle; doctor/debug/config for runtime repair without reinstall. |
| Implementation | 4/5 | `version`, `doctor`, `debug`, `config`/`set --apply` (stop→reload→start), `update`, plus `serve`/`start`/`stop`/`restart`/`reload`/`status`/`tui`. |
| Tests | 3/5 | `server/tests/cli_exit_codes.rs`: version/help without config, doctor, config set, missing config, unknown command, status/stop without resident. No live attach/`serve` smoke yet. |
| Docs | 4/5 | `docs/development.md` CLI table; installer PATH task; ops-resilience cross-links. |
| Windows readiness | 4/5 | `winserve.exe` staged + Start Menu CLI; optional user PATH + App Paths; named pipe + `%LOCALAPPDATA%\WinServeAI\manager.lock`. |

## Gaps

- No automated CLI smoke for live `serve` + attach `status|stop`.
- Operator Windows proof that tray + CLI attach share one owner.

## Next actions (ordered)

1. Optional: live attach smoke with fake resident (still host-safe).
2. Close with operator Windows proof that tray + CLI share one owner.
