# Readiness: ui

| Field | Value |
| --- | --- |
| Path | `apps/desktop/` (`winserve-tray`) |
| Overall | **1.6 / 5** |
| Label | `scaffold` |
| Reviewed | 2026-07-30 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 3/5 | Tauri 2 shell; commands wrap `ServerManager` only — matches `docs/specs/E-desktop.md` E1a. No chat/download/sidecar spawn. |
| Implementation | 2/5 | `apps/desktop/src-tauri` binary `winserve-tray`; invoke handlers for status/start/stop/restart/endpoint/config summary. Stub HTML/JS only (E1b+). No tray menu / quit→stop yet (E1e). |
| Tests | 0/5 | None. |
| Docs | 2/5 | `apps/desktop/README.md` + TODO E1a; primary architecture still Phase-2 light. |
| Windows readiness | 1/5 | Scaffold compiles with Tauri host toolchain; not yet proven as installable tray on Win10/11. |

## Gaps

- Canonical status UI / log viewer / settings / path picker (E1b–E2).
- Quit path → `stop()` + Job Object backstop (E1e).
- Tray should become resident G owner (lockfile + IPC) like `winserve serve`.

## Next actions (ordered)

1. E1b — canonical status badge + start/stop controls polish.
2. E1e — quit → `stop()`; consider embedding lockfile/IPC owner.
3. E1c / E1d / E2 — logs, settings, model path picker.
