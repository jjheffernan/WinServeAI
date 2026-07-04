# Readiness: ui

| Field | Value |
| --- | --- |
| Path | `app/ui/` |
| Overall | **0.4 / 5** |
| Label | `stub` |
| Reviewed | 2026-07-04 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 1/5 | Mentioned in `docs/architecture.md` layout as “optional later (empty in MVP)” and Phase 2 in `docs/roadmap.md`. No UI design artifacts. |
| Implementation | 0/5 | `app/ui/` does not exist in the tree (no placeholder files). |
| Tests | 0/5 | None. |
| Docs | 1/5 | Roadmap Phase 2 lists start/stop/status, log viewer, settings, model path picker — aspirational only. |
| Windows readiness | 0/5 | No Tauri/desktop shell. |

## Gaps

- Entire Phase 2 surface missing.
- No dependency on ServerManager from a UI layer yet (correct for Phase 1).

## Next actions (ordered)

1. Defer until Phase 1 engine is `mvp-ready` (graceful stop, hardware, tests).
2. Add empty `app/ui/` placeholder only if packaging requires it.
3. Desktop must call `ServerManager` only — no llama flags in UI.
