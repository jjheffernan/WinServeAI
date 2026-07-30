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
| Design | 4/5 | Tauri commands wrap `ServerManager` only; settings gated; `.gguf` path picker only (no download). |
| Implementation | 4/5 | Status/start/stop + logs + settings + `pick_model_path` (tauri-plugin-dialog, `.gguf` filter). No quit→stop / lockfile owner yet (E1e). |
| Tests | 2/5 | Manager `apply_config` tests; log tail test. No UI/dialog tests. |
| Docs | 3/5 | `apps/desktop/README.md` + TODO through E2. |
| Windows readiness | 1/5 | Compiles with Tauri host toolchain; dialog not yet proven on Win10/11 install. |

## Gaps

- Quit path → `stop()` + Job Object backstop (E1e).
- Tray should become resident G owner (lockfile + IPC) like `winserve serve`.

## Next actions (ordered)

1. E1e — quit → `stop()`; embed lockfile/IPC owner.
