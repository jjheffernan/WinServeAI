# Readiness: ui

| Field | Value |
| --- | --- |
| Path | `apps/desktop/` (`winserve-tray`) |
| Overall | **3.2 / 5** |
| Label | `mvp-partial` |
| Reviewed | 2026-07-30 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 4/5 | Tauri commands wrap `ServerManager` only; tray is G owner (lockfile + IPC); quit → `stop()` with Job Object backstop. |
| Implementation | 4/5 | Status/start/stop/logs/settings/`.gguf` picker; `ExitRequested` / `CloseRequested` call `stop()`; lockfile + pipe listen for CLI attach. |
| Tests | 2/5 | Manager/log unit tests; no automated tray quit/IPC integration test. |
| Docs | 4/5 | `apps/desktop/README.md` + `docs/development.md` / `architecture.md` / `installer.md` cover tray + IPC. |
| Windows readiness | 2/5 | Compiles with Tauri host toolchain; Win10/11 tray + dialog + quit path still need operator proof. |

## Gaps

- System tray icon / menu (window-only shell today).
- Operator Windows proof for quit with no orphan `llama-server`.

## Next actions (ordered)

1. Optional tray icon menu (start/stop/status) for true minimize-to-tray UX.
2. Operator smoke: quit while Ready → no orphan llama-server; CLI `winserve stop` against tray owner.
