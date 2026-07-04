# Readiness: scripts-pr-loop

| Field | Value |
| --- | --- |
| Path | `scripts/pr_review_loop/` |
| Overall | **3.0 / 5** |
| Label | `mvp-partial` |
| Reviewed | 2026-07-04 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 4/5 | Worktree-scoped coder/review/fix loop with protected `base_branch`, max iterations, parallel branches (`scripts/pr_review_loop/loop.py`, `models.py`, `gitops.py`, `worktrees.py`). Constraints documented in README. |
| Implementation | 3/5 | CLI (`__main__.py`) supports `--dry-run`, `--no-push`, config JSON. Dry-run uses stub agents (`agents.py` dry-run backend). Real agents via stdin/stdout commands. Happy-path plumbing works. |
| Tests | 2/5 | No formal pytest suite; `--dry-run` exercises end-to-end plumbing (documented as plumbing-only quick start). |
| Docs | 4/5 | `scripts/pr_review_loop/README.md` covers layout, config fields, agent contracts, example hooks. |
| Windows readiness | 2/5 | Pure Python/git; runnable on Windows if Python and git worktrees available, but primary examples are bash-oriented and not Windows-specific. |

## Gaps

- No automated unit tests for gitops/worktrees.
- Real agent integration is operator-supplied (hooks are echo stubs).
- Windows path/quoting not specially validated.

## Next actions (ordered)

1. Add lightweight unit tests for config load and dry-run result JSON shape.
2. Document Windows PowerShell invocation of `python -m scripts.pr_review_loop`.
3. Optional CI job running `--dry-run --no-push` against a fixture config.
