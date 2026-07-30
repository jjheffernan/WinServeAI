# Readiness: ui

| Field | Value |
| --- | --- |
| Path | `apps/desktop/` (`winserve-tray`) |
| Overall | **2.6 / 5** |
| Label | `mvp-partial` |
| Reviewed | 2026-07-30 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 3/5 | Tauri 2 shell; commands wrap `ServerManager` only — matches `docs/specs/E-desktop.md` E1a–E1c. Canonical badge states; read-only log tail. |
| Implementation | 3/5 | Status/start/stop/restart UI + `manager_logs` tailing `server`/`llama`/`error` via `logs::tail_dir`. No settings / path picker / quit→stop yet. |
| Tests | 1/5 | `tail_file_returns_last_n_lines` in `server/logs.rs`; no UI tests. |
| Docs | 3/5 | `apps/desktop/README.md` + logging.md tail note + TODO E1a–E1c. |
| Windows readiness | 1/5 | Compiles with Tauri host toolchain; not yet proven as installable tray on Win10/11. |

## Gaps

- Settings / path picker (E1d, E2).
- Quit path → `stop()` + Job Object backstop (E1e).
- Tray should become resident G owner (lockfile + IPC) like `winserve serve`.

## Next actions (ordered)

1. E1d — settings YAML validate/write.
2. E1e — quit → `stop()`; consider embedding lockfile/IPC owner.
3. E2 — model path picker.
