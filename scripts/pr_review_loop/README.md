# PR Worktree Review Loop

> **Readiness:** 3.0/5 (`mvp-partial`) — details in [../../docs/readiness/scripts-pr-loop.md](../../docs/readiness/scripts-pr-loop.md)

Automate feature development + PR review using git worktrees and a review/fix loop until the PR is clean (or `max_iterations` is hit).

## Layout

```text
repo/
  worktrees/
    wt-feature-a/     # git worktree for feature/a
    wt-feature-b/
  scripts/pr_review_loop/
  examples/pr_review_loop.json
```

## Constraints

* Never modify `base_branch`
* Only operate inside the assigned worktree
* Only push the corresponding feature branch
* Always re-run `git diff origin/<base>...HEAD` before each review
* Stop on `max_iterations` even if `FAIL`

## Quick start (plumbing only)

```bash
# From repo root
python3 -m scripts.pr_review_loop \
  --config examples/pr_review_loop.json \
  --dry-run \
  --no-push \
  -v
```

`--dry-run` uses stub agents (no external LLM). Output is JSON:

```json
[
  {
    "branch": "feature/example-a",
    "status": "PASS",
    "iterations": 1,
    "remaining_issues": []
  }
]
```

## Real agents

Point `coder_cmd`, `reviewer_cmd`, and `fixer_cmd` at any program that:

1. Reads a JSON payload on **stdin**
2. Performs work in `worktree` (env: `PR_LOOP_WORKTREE`, `PR_LOOP_BRANCH`, `PR_LOOP_ROLE`)
3. Writes result on **stdout**

### Reviewer stdout (required)

```json
{
  "issues": [
    {
      "file": "path/to/file",
      "line": 12,
      "issue": "what is wrong",
      "severity": "error|warning|info",
      "suggestion": "how to fix"
    }
  ],
  "status": "PASS"
}
```

`PASS` only when `issues` is empty.

Example hooks (no-op stubs) live in `hooks/`. Swap them for Cursor Agent CLI, a custom script, or any agent runner.

## Config

| Field | Meaning |
| --- | --- |
| `repo_path` | Repository root |
| `base_branch` | Protected base (default `dev` for day-to-day; use `main` only for release PRs) |
| `feature_branches` | Branches to process in parallel |
| `max_iterations` | Review/fix cycles (default `5`) |
| `task_prompt` | Instructions for the coder step |
| `coder_cmd` / `reviewer_cmd` / `fixer_cmd` | Shell commands |
| `dry_run` | Stub agents |
| `push` | `git push origin <feature_branch>` |

## Per-branch loop

1. **Coder** → `git add -A` → commit `agent change` → push
2. **Review** fresh diff vs `origin/<base>`
3. If `FAIL` → **Fixer** → commit `fix review issues` → push
4. Repeat 2–3 until `PASS` or `max_iterations`

Branches run **in parallel** (one thread per worktree).

## Agent skill

See `.agents/skills/pr-worktree-review/SKILL.md` for reviewer/fixer contracts.
