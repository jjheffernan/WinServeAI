# Readiness: docs

| Field | Value |
| --- | --- |
| Path | `docs/` |
| Overall | **3.8 / 5** |
| Label | `mvp-ready` |
| Reviewed | 2026-07-04 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 4/5 | Appliance architecture locked (`docs/architecture.md`, `docs/adr/0001-appliance-architecture.md`); INDEX status legend; research deep dives 01–05; roadmap phases; readiness scorecards. |
| Implementation | 4/5 | Operator guides (`api`, `backend`, `configuration`, `development`, `logging`, `installer`) match current `app/src/**` paths. `docs/PLAN.md` / `docs/TODO.md` track implementation. |
| Tests | 3/5 | `scripts/check_doc_drift.py` enforces scorecard ↔ dashboard ↔ banner consistency and primary health probe claims (CI on `dev`/`main`). No full link checker. |
| Docs | 5/5 | Guides + research briefs + ADR + examples (config YAML, CLI commands, pr_review_loop). Strongest module in the tree. |
| Windows readiness | 3/5 | Primary OS guidance throughout (`.exe`, PowerShell, Inno, firewall); accurate for current code maturity. |

## Gaps

- No automated broken-link / path verification beyond drift checks.
- Some research banners still note historical `packages/*` paths.

## Next actions (ordered)

1. Keep readiness scorecards updated when modules change (`check_doc_drift.py`).
2. Optional CI: fail on links to missing files under `docs/` and `app/src/`.
3. Retire historical path notes once fully migrated.
