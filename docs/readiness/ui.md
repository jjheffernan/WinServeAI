# Readiness: ui

| Field | Value |
| --- | --- |
| Path | `apps/desktop/` (`winserve-tray`) |
| Overall | **3.0 / 5** |
| Label | `mvp-partial` |
| Reviewed | 2026-07-30 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 4/5 | Tauri commands wrap `ServerManager` only; settings blocked while Starting/Ready/Stopping per E-desktop. |
| Implementation | 3/5 | Status/start/stop + log tail + settings form → `apply_config` validate/write YAML. No path picker dialog / quit→stop / lockfile owner yet. |
| Tests | 2/5 | Manager `apply_config` write + Ready rejection tests; log `tail_file` test. No UI tests. |
| Docs | 3/5 | `apps/desktop/README.md` + TODO E1a–E1d. |
| Windows readiness | 1/5 | Compiles with Tauri host toolchain; not yet proven as installable tray on Win10/11. |

## Gaps

- Model path picker dialog (E2).
- Quit path → `stop()` + Job Object backstop (E1e).
- Tray should become resident G owner (lockfile + IPC) like `winserve serve`.

## Next actions (ordered)

1. E2 — native `.gguf` path picker (writes `model.path`).
2. E1e — quit → `stop()`; embed lockfile/IPC owner.
