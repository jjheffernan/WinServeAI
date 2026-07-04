# Readiness: bin

| Field | Value |
| --- | --- |
| Path | `bin/` |
| Overall | **2.2 / 5** |
| Label | `scaffold` |
| Reviewed | 2026-07-04 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 3/5 | External system boundary: place pinned `llama-server.exe` in `bin/` (`bin/README.md`); code resolves via `runtime::llama::default_binary`. Matches architecture. |
| Implementation | 1/5 | Directory contains only `bin/README.md`. Pin **b9866** documented with release URL and asset guidance. No binary committed; no fetch/download script in-repo. |
| Tests | 0/5 | N/A — nothing to test without a binary. |
| Docs | 5/5 | `bin/README.md` pins `b9866`; operator smoke path in `docs/specs/A2-smoke.md`, `scripts/smoke-openai.ps1`, `scripts/smoke-check.sh`; references in `docs/backend.md`, `docs/development.md`, `docs/release-process.md`. |
| Windows readiness | 2/5 | Path expects `llama-server.exe` on Windows; smoke scripts preflight presence and pin. Nothing is shipped; operator must supply the binary manually. |

## Gaps

- No `llama-server.exe` in tree (by design for licensing/size, but blocks out-of-box run).
- No scripted pin/fetch for the documented release.
- No `THIRD_PARTY_NOTICES` file yet (mentioned in README).

## Next actions (ordered)

1. One-command fetch script for **b9866** Windows asset.
2. Add `THIRD_PARTY_NOTICES` template for llama.cpp MIT.
3. Bundle binary only via installer (Phase 3), not git.
