# Readiness: docs

| Field | Value |
| --- | --- |
| Path | `docs/` |
| Overall | **3.6 / 5** |
| Label | `mvp-ready` |
| Reviewed | 2026-07-04 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 4/5 | Appliance architecture locked (`docs/architecture.md`, `docs/adr/0001-appliance-architecture.md`); INDEX status legend; research deep dives 01–05; roadmap phases. |
| Implementation | 4/5 | Operator guides (`api`, `backend`, `configuration`, `development`, `logging`, `installer`) match current `app/src/**` paths and known CLI gaps. Doc-build STATUS tracks rebuild quality. |
| Tests | 2/5 | Manual review via `docs/research/doc-build/STATUS.md`; no automated link or path checks. |
| Docs | 5/5 | Guides + research briefs + ADR + examples (config YAML, CLI commands, pr_review_loop). Strongest module in the tree. |
| Windows readiness | 3/5 | Primary OS guidance throughout (`.exe`, PowerShell, Inno, firewall); accurate for current code maturity. |

## Gaps

- No automated broken-link / path verification.
- Readiness dashboard (this review) was missing before this pass.
- Some research banners still note historical `packages/*` paths.

## Next actions (ordered)

1. Keep readiness scorecards updated when modules change.
2. Optional CI: fail on links to missing files under `docs/` and `app/src/`.
3. Retire historical path notes once fully migrated.
