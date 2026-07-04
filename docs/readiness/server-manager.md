# Readiness: server-manager

| Field | Value |
| --- | --- |
| Path | `app/src/server/manager.rs` |
| Overall | **2.8 / 5** |
| Label | `mvp-partial` |
| Reviewed | 2026-07-04 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 4/5 | Matches appliance architecture: sole orchestration API (`load_config`, `detect_hardware`, `build_command`, `start`/`stop`/`restart`, `status`/`get_status`) in `app/src/server/manager.rs`; aligned with `docs/architecture.md`, `docs/research/05-server-manager.md`, ADR-0001. Not locked by contract tests. |
| Implementation | 3/5 | Happy path works: validate config/binary/model, spawn via `ChildProcess`, wait readiness, set `Ready`; `stop` grace 8s; `restart` = stop+start; `get_status` marks `Crashed` on unexpected exit. Port conflict is warning-only (`network::port_available`). No auto-recovery from `Failed`/`Crashed`. |
| Tests | 0/5 | No `#[test]` / `#[cfg(test)]` in crate. |
| Docs | 4/5 | Operator path in `docs/backend.md`; state machine research in `docs/research/05-server-manager.md`; architecture API list matches code. |
| Windows readiness | 3/5 | Runs on Windows via `runtime/process`, but inherits CTRL_BREAK stub and no Job Object — graceful stop is incomplete. |

## Gaps

- No unit or integration tests for lifecycle transitions.
- CLI cannot call `stop`/`restart` on a detached manager (`app/src/main.rs` rejects those commands).
- Crash detection exists (`get_status` → `Crashed`) but no restart policy.
- Port-in-use is logged, not blocked.

## Next actions (ordered)

1. Add lifecycle tests (start failure paths: missing binary, missing model, readiness timeout).
2. Wire a long-running owner (tray/service) so `stop`/`restart` work outside foreground `start`.
3. Harden Windows stop via `runtime-process` (CTRL_BREAK + Job Object), then re-score Windows readiness.
