# Readiness: server-manager

| Field | Value |
| --- | --- |
| Path | `app/src/server/manager.rs` |
| Overall | **3.4 / 5** |
| Label | `mvp-partial` |
| Reviewed | 2026-07-04 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 4/5 | Sole orchestration API (`load_config`, `detect_hardware`, `build_command`, `start`/`stop`/`restart`, `status`/`get_status`) in `app/src/server/manager.rs`; aligned with `docs/architecture.md`, `docs/research/05-server-manager.md`, ADR-0001. |
| Implementation | 4/5 | Happy path: validate config/binary/model, port preflight **fails** start when busy, spawn via `ChildProcess`, wait readiness, set `Ready`; `stop` grace 8s; `restart` = stop+start; `get_status` marks `Crashed` on unexpected exit. No auto-recovery from `Failed`/`Crashed`. No long-running owner for detached CLI `stop`/`restart`. |
| Tests | 1/5 | No manager lifecycle tests; coverage is indirect via config/process/health units. |
| Docs | 4/5 | Operator path in `docs/backend.md`; state machine research in `docs/research/05-server-manager.md`. |
| Windows readiness | 4/5 | Inherits Job Object + CTRL_BREAK from `runtime/process`; port preflight blocks sleep/wake zombies holding the port. |

## Gaps

- No unit/integration tests for lifecycle transitions.
- CLI cannot call `stop`/`restart` on a detached manager (`app/src/main.rs` rejects those commands).
- Crash detection exists (`get_status` → `Crashed`) but no background exit watcher / restart policy.
- A2 operator smoke with real binary still open.

## Next actions (ordered)

1. Long-running owner (tray/service or lockfile+IPC) so `stop`/`restart` work outside foreground `start`.
2. Lifecycle tests (missing binary/model, port busy, readiness timeout).
3. Optional background wait task so `Crashed` is observed without a status poll.
