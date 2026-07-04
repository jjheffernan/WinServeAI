# Contributing

Thanks for helping build WinServeAI. This is infrastructure software — prefer stability, clear boundaries, and small surface area.

## Before You Code

1. Read [`docs/vision.md`](vision.md) and [`docs/architecture.md`](architecture.md).
2. **Follow the architecture Rules** (non-negotiable):
   - One backend: llama.cpp only. No backend traits, plugins, or multi-provider layers.
   - One orchestrator: `ServerManager`. UI/CLI only call it.
   - Raw llama flags only in `app/src/runtime/llama.rs`.
   - YAML config is the source of truth (`config/default.yaml`).
   - Unified logs under `logs/`.
   - No model downloads, chat UI, Docker, or multi-provider support.
3. Confirm the change fits MVP scope (see vision), or is explicitly deferred work.
4. Prefer extending `ServerManager` (`app/src/server/`) and runtime (`app/src/runtime/`) over special-casing the UI or CLI.

Dev setup and commands: [`development.md`](development.md).

## Branches

| Branch | Purpose |
| --- | --- |
| **`dev`** | Default working branch — experimental / preview. Open PRs here. |
| **`main`** | Releases only — stable tags and release artifacts. |

```bash
git checkout dev
git pull
git checkout -b feature/your-change
# ... work, then PR into dev
```

Promote to release with a PR from `dev` → `main` (see [release-process.md](release-process.md)). Do not commit day-to-day work on `main`.

## Pull Requests

* Target **`dev`**, not `main` (except release promotions)
* One concern per PR
* Update docs when behavior or architecture changes
* Add tests at the appropriate layer (unit → runtime/process → integration → hardware → installer)
* Do **not** introduce:
  * Chat UI
  * Model download ecosystems
  * Plugins or multi-backend abstractions
  * Backend-specific flags in config (flags belong only in `runtime/llama.rs`)

## Agent Skills

Project skills live in `.agents/skills/` and work with any Agent Skills–compatible tool. Keep a single agent-agnostic copy there — do not install skills into per-agent directories (`.claude/skills`, `.cursor/skills`, etc.).

| Skill | Use when |
| --- | --- |
| `pr-worktree-review` | PR worktree review loop (coder / reviewer / fixer) |
| `grill-me` / `grilling` | Stress-test a plan before building |
| `ponytail` | Prefer the laziest correct solution (YAGNI) |
| `caveman` | Ultra-compressed communication |

```bash
npm run skills:restore
npm run skills:update
```

## PR review loop

For automated feature work + review in git worktrees, use `scripts/pr_review_loop`. Plumbing-only dry run:

```bash
python3 -m scripts.pr_review_loop --config examples/pr_review_loop.json --dry-run --no-push
```

Details: [`scripts/pr_review_loop/README.md`](../scripts/pr_review_loop/README.md). Agent contracts: `.agents/skills/pr-worktree-review/SKILL.md`.

## Code of Conduct

Be respectful. Assume good intent. Optimize for long-term maintainability.
