# Research: `contributing.md`

**Target:** `docs/contributing.md`  
**Branch lock:** appliance wrapper — `ServerManager` only, llama.cpp only, no backend traits.

## Problem with current doc

| Line / claim | Issue |
| --- | --- |
| “Prefer extending Server Manager / **Backend interfaces**” | Architecture forbids backend traits/plugins. There is no interface layer to extend. |
| MVP scope only via vision | Must also require the **Rules** in `architecture.md` (one backend, one orchestrator, flags only in `runtime/llama.rs`, YAML config, no downloads/chat/Docker/multi-provider). |
| Skills section | Mentions restore only; should point at `.agents/skills/` and key skills (`pr-worktree-review`, grill/ponytail/caveman). |
| No `pr_review_loop` | README / AGENTS.md document the loop; contributors and agents need a pointer. |
| Exclusions | Lists chat UI and model downloads; should also ban **plugins** / multi-backend (aligned with vision + architecture). |

## Canonical sources (do not invent)

| Source | Use for |
| --- | --- |
| `docs/architecture.md` § Rules | Non-negotiable contribution constraints |
| `docs/vision.md` | MVP in/out; product category |
| `AGENTS.md` | Agent layout, skills table, pr_review_loop dry-run |
| `docs/development.md` | Build/run (link only; do not duplicate) |
| `scripts/pr_review_loop/README.md` | Loop behavior; link, do not copy full config schema |
| `.agents/skills/pr-worktree-review/SKILL.md` | Agent contracts for the loop |

## Required rewrite outline

1. **Tone** — infrastructure / appliance; small surface area (keep).
2. **Before you code**
   - Read `vision.md` and `architecture.md`.
   - **Follow architecture Rules** (link or short restatement of the six rules).
   - Confirm MVP scope (or explicit Phase 5+ / later-only work).
   - Prefer extending **ServerManager** / `app/src/server/` and `app/src/runtime/` over special-casing UI/CLI. **No** “Backend interfaces.”
3. **Pull requests**
   - One concern per PR.
   - Update docs when behavior or architecture changes.
   - Tests at the appropriate layer (keep practical list; no package-monorepo language).
   - Explicit bans: chat UI, model downloads, plugins / multi-backend, backend-specific flags in config (flags only in `runtime/llama.rs`).
4. **Agent skills**
   - Canonical path: `.agents/skills/` (agent-agnostic).
   - Restore: `npm run skills:restore` (and optionally `skills:update`).
   - Do not install into per-agent skill folders.
   - Mention `pr-worktree-review` for the review loop.
5. **PR review loop**
   - Point to `python3 -m scripts.pr_review_loop` with example config dry-run.
   - Link `scripts/pr_review_loop/README.md` and skill.
6. **Code of conduct** — keep short.

## Out of scope for this doc

* Full development setup (→ `development.md`)
* Runtime/flag details (→ `backend.md` / architecture)
* Installer / release process

## Acceptance checklist

- [ ] No mention of “Backend interfaces” or backend traits
- [ ] Architecture rules required, not optional
- [ ] PR guidelines present
- [ ] Skills + restore documented
- [ ] `pr_review_loop` mentioned with dry-run command
- [ ] Explicit: no chat UI, model downloads, plugins
