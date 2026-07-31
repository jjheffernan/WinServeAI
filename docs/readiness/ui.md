# Readiness: ui

| Field | Value |
| --- | --- |
| Path | `apps/desktop/` (`winserve-tray`) |
| Overall | **3.6 / 5** |
| Label | `mvp-partial` |
| Reviewed | 2026-07-30 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 4/5 | Tauri commands wrap `ServerManager` only; tray is G owner (lockfile + IPC); quit → `stop()` with Job Object backstop. |
| Implementation | 4/5 | Status/start/stop/logs/settings/`.gguf` picker; system-tray icon/menu (Open/Start/Stop/Restart/Settings/Quit); close hides to tray; Quit/`ExitRequested` call `stop()`; lockfile + pipe listen for CLI attach. |
| Tests | 2/5 | Tray menu-id unit test; manager/log unit tests; no automated tray quit/IPC operator proof. |
| Docs | 4/5 | `apps/desktop/README.md` + `docs/development.md` / `architecture.md` / `installer.md` cover tray + IPC. |
| Windows readiness | 3/5 | Tray icon/menu implemented; Win10/11 dialog + quit-with-no-orphan still need operator proof. |

## Gaps

- Operator Windows proof for quit with no orphan `llama-server`.

## Next actions (ordered)

1. Operator smoke: quit while Ready → no orphan llama-server; CLI `winserve stop` against tray owner.
2. Optional: tray-owned attach integration test on Windows CI.
