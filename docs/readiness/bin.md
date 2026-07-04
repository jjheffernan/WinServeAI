# Readiness: bin

| Field | Value |
| --- | --- |
| Path | `bin/` |
| Overall | **1.4 / 5** |
| Label | `stub` |
| Reviewed | 2026-07-04 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 3/5 | External system boundary: place pinned `llama-server.exe` in `bin/` (`bin/README.md`); code resolves via `runtime::llama::default_binary`. Matches architecture. |
| Implementation | 0/5 | Directory contains only `bin/README.md`. No binary committed; no fetch/download script in-repo. |
| Tests | 0/5 | N/A — nothing to test without a binary. |
| Docs | 3/5 | `bin/README.md` plus references in `docs/backend.md`, `docs/development.md`, `docs/release-process.md` (pin policy). |
| Windows readiness | 1/5 | Path expects `llama-server.exe` on Windows, but nothing is shipped; operator must supply the binary manually. |

## Gaps

- No `llama-server.exe` in tree (by design for licensing/size, but blocks out-of-box run).
- No scripted pin/fetch for the documented release.
- No `THIRD_PARTY_NOTICES` file yet (mentioned in README).

## Next actions (ordered)

1. Document exact pin URL/version in release-process and a one-command fetch script.
2. Add `THIRD_PARTY_NOTICES` template for llama.cpp MIT.
3. Bundle binary only via installer (Phase 3), not git.
