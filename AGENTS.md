# Agent Instructions

WinServeAI is a **Windows-native local AI server** monorepo. Treat it as infrastructure software: small surface area, stable defaults, backend-agnostic core.

## Architecture (non-negotiable)

```text
UI → Server Manager (packages/launcher) → Backend trait → concrete backend (llama)
```

* Never let UI code depend on llama.cpp or backend-specific flags
* All process ownership goes through `winserve-launcher` (Server Manager)
* Config is human-readable YAML only (`packages/config`)

Read `docs/architecture.md` and `docs/vision.md` before large changes.

## Skills (agent-agnostic)

Canonical skills live in **`.agents/skills/`** (Agent Skills standard). They are not Cursor-only.

| Skill | Use when |
| --- | --- |
| `grill-me` / `grilling` | Stress-test a plan or design before building |
| `ponytail` | Prefer the laziest correct solution (YAGNI) |
| `caveman` | Ultra-compressed communication (token efficiency) |
| `design-an-interface` | Explore module/API shapes |
| `handoff` | Compact context for another agent |
| `ubiquitous-language` | Harden domain terminology |

Restore / update:

```bash
npm run skills:restore
npm run skills:update
```

Do **not** install skills into per-agent directories (`.claude/skills`, `.windsurf/skills`, etc.). Keep a single agent-agnostic copy under `.agents/skills/`.

## Scope discipline

MVP excludes chat UI, model downloads, HuggingFace, agents, RAG, MCP, Docker, auth, metrics dashboards, and remote management. Push those to Phase 5+ unless explicitly requested.

## Language / stack

* Core: Rust Cargo workspace
* Desktop: Tauri (Phase 2)
* Installer: Inno Setup preferred starting point (Phase 3)
* Primary OS: Windows
