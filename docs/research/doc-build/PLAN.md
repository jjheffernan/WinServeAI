# Documentation build-out plan

**Worktree:** `research-06cce628` · **Branch:** `refactor/minimal-appliance`  
**Architecture lock:** appliance wrapper — `ServerManager` only, llama.cpp only, no backend traits.

## Already solid (do not rewrite from scratch)

| File | Notes |
| --- | --- |
| `architecture.md` | Canonical |
| `vision.md` | Canonical |
| `build-spec.md` | Agent prompt |
| `prior-art.md` | Research dump |
| `research/01–05-*.md` | Phase-0 deep dives |

## Stub / outdated targets

| Doc | Problem | Research output | Build output |
| --- | --- | --- | --- |
| `api.md` | References `packages/api` | `doc-build/api.research.md` | `api.md` |
| `backend.md` | Too thin; rename framing to runtime | `doc-build/backend.research.md` | `backend.md` |
| `configuration.md` | Thin; needs full schema + examples | `doc-build/configuration.research.md` | `configuration.md` |
| `contributing.md` | Mentions backend interfaces | `doc-build/contributing.research.md` | `contributing.md` |
| `development.md` | Thin; wrong package layout | `doc-build/development.research.md` | `development.md` |
| `installer.md` | Thin; not merged with research/04 | `doc-build/installer.research.md` | `installer.md` |
| `logging.md` | Thin | `doc-build/logging.research.md` | `logging.md` |
| `release-process.md` | Mentions workspace packages | `doc-build/release-process.research.md` | `release-process.md` |
| `research.md` | Checklist stale (`packages/config`) | `doc-build/research.research.md` | `research.md` |
| `roadmap.md` | Still says backend abstraction / plugins | `doc-build/roadmap.research.md` | `roadmap.md` |
| `adr/README.md` | Missing (only `.gitkeep`) | `doc-build/adr.research.md` | `adr/README.md` |

## Rules for all agents

1. Work **only** in worktree path (never main repo checkout).
2. Align with `architecture.md` / `AGENTS.md` — appliance, not platform.
3. Link to `research/01–05` and `prior-art.md` instead of duplicating.
4. No multi-backend, plugins, chat UI, or model marketplace content.
5. Prefer actionable scaffolding notes over marketing.

## Orchestrator deliverable

`docs/research/doc-build/STATUS.md` — per-file PASS/FAIL and gaps.
