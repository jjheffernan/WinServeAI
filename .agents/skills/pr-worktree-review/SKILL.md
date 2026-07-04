---
name: pr-worktree-review
description: >
  PR worktree review loop contracts for coder, reviewer, and fixer agents.
  Use when running scripts/pr_review_loop, reviewing feature-branch diffs,
  fixing review issues in a worktree, or when the user mentions PR review loop,
  worktree review, or agent_coder/reviewer/fixer.
---

# PR Worktree Review

Orchestrator: `python3 -m scripts.pr_review_loop --config <json>`.

## Constraints (all roles)

* Never modify `base_branch` (usually `main`)
* Only edit files inside `PR_LOOP_WORKTREE`
* Only the orchestrator commits and pushes
* Feature branch only: `PR_LOOP_BRANCH`

## Coder (`PR_LOOP_ROLE=coder`)

Stdin JSON: `instructions`, `task_prompt`, `worktree`, `branch`.

Implement `task_prompt` with minimal diffs. Respect WinServeAI architecture:

```text
App → ServerManager → runtime (llama + process) → bin/llama-server.exe
```

No backend traits, plugins, or multi-provider layers.

Do not commit or push.

## Reviewer (`PR_LOOP_ROLE=reviewer`)

Stdin JSON: `instructions`, `diff`, `worktree`, `branch`.

Stdout **only** this JSON (no markdown fences):

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

* `PASS` only if `issues` is empty
* `FAIL` otherwise
* Prefer architecture, correctness, and regressions over style nits

## Fixer (`PR_LOOP_ROLE=fixer`)

Stdin JSON: `instructions`, `issues`, `worktree`, `branch`.

Fix listed issues only. Minimal diffs. Do not commit or push.
