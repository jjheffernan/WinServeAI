# Readiness: ui

| Field | Value |
| --- | --- |
| Path | `apps/desktop/` (`winserve-tray`) |
| Overall | **2.2 / 5** |
| Label | `scaffold` |
| Reviewed | 2026-07-30 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 3/5 | Tauri 2 shell; commands wrap `ServerManager` only — matches `docs/specs/E-desktop.md` E1a/E1b. Canonical badge states only (no “Running”). |
| Implementation | 3/5 | `winserve-tray` invoke handlers for status/start/stop/restart/endpoint/config summary. Status badge + start/stop/restart/copy URL with transitional disables and optimistic Starting/Stopping. No tray menu / quit→stop yet (E1e). |
| Tests | 0/5 | None. |
| Docs | 2/5 | `apps/desktop/README.md` + TODO E1a/E1b. |
| Windows readiness | 1/5 | Scaffold compiles with Tauri host toolchain; not yet proven as installable tray on Win10/11. |

## Gaps

- Log viewer / settings / path picker (E1c, E1d, E2).
- Quit path → `stop()` + Job Object backstop (E1e).
- Tray should become resident G owner (lockfile + IPC) like `winserve serve`.

## Next actions (ordered)

1. E1c — log viewer from `logs/`.
2. E1e — quit → `stop()`; consider embedding lockfile/IPC owner.
3. E1d / E2 — settings + model path picker.
