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
| Implementation | 4/5 | Happy path: validate config/binary/model, port preflight **fails** start when busy, spawn via `ChildProcess`, wait readiness, set `Ready`; `stop` grace 8s; `restart` = stop+start; `get_status` marks `Crashed` on unexpected exit. `apply_config` blocked while Starting/Ready/Stopping. Resident ownership is outside this module (`serve` / tray + IPC). |
| Tests | 1/5 | `apply_config` unit coverage; lifecycle still mostly indirect via config/process/health units. |
| Docs | 4/5 | Operator path in `docs/backend.md` / `development.md` / `architecture.md`; state machine research in `docs/research/05-server-manager.md`. |
| Windows readiness | 4/5 | Inherits Job Object + CTRL_BREAK from `runtime/process`; port preflight blocks sleep/wake zombies holding the port. |

## Gaps

- No full lifecycle integration tests (missing binary/model, port busy, readiness timeout).
- Crash detection exists (`get_status` → `Crashed`) but no background exit watcher / restart policy.
- A2 operator smoke with real binary still open.

## Next actions (ordered)

1. Lifecycle tests (missing binary/model, port busy, readiness timeout).
2. Optional background wait task so `Crashed` is observed without a status poll.
3. Operator Windows proof: tray quit + CLI `winserve stop` leave no orphan.
