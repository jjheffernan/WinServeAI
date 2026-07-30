# Documentation index

WinServeAI docs for the **llama.cpp appliance** architecture (`ServerManager` only, no backend traits).

## Module readiness

| Doc | Purpose |
| --- | --- |
| [readiness/README.md](readiness/README.md) | Maturity at-a-glance: per-module overall score, label, top gap, and links to scorecards |

Scorecards live under [readiness/](readiness/) (Design, Implementation, Tests, Docs, Windows readiness). Primary operator guides also carry a short **Readiness** block at the top.

Prioritized work from scores: [TODO.md](TODO.md). Ordered milestones: [PLAN.md](PLAN.md).

Doc drift policy (readiness scores must match banners/dashboard): [policies/doc-drift.md](policies/doc-drift.md) · `python3 scripts/check_doc_drift.py`.

Canonical external URLs (llama.cpp, DXGI, Job Objects, Inno, NVIDIA): [policies/SOURCES.md](policies/SOURCES.md).

Apply-worktree audit (caveman/ponytail review + subagent scores): [audit/apply-review.md](audit/apply-review.md).

Status legend:

| Status | Meaning |
| --- | --- |
| **canonical** | Architecture lock or finished appliance-aligned guide |
| **building** | Incomplete or actively changing |
| **stub** | Thin or outdated; needs build-out |

## Product & architecture

| Doc | Purpose | Status |
| --- | --- | --- |
| [architecture.md](architecture.md) | Runtime boundary, `ServerManager` API, layout, non-negotiable rules | canonical |
| [vision.md](vision.md) | Product category, MVP in/out, principles | canonical |
| [build-spec.md](build-spec.md) | Drop-in agent prompt for the minimal appliance | canonical |
| [roadmap.md](roadmap.md) | Phased plan and version milestones (appliance path) | canonical |
| [release-process.md](release-process.md) | Branches (`dev` = work, `main` = releases), channels, pin policy | canonical |

**Branches:** `dev` is the default working branch (experimental / preview). `main` is releases only.

## Operator & developer guides

| Doc | Purpose | Status |
| --- | --- | --- |
| [api.md](api.md) | OpenAI-compatible client surface and readiness probes | canonical |
| [backend.md](backend.md) | Runtime layer (`runtime/llama` + `runtime/process`); not a multi-backend guide | canonical |
| [configuration.md](configuration.md) | YAML schema, examples, localhost vs LAN | canonical |
| [development.md](development.md) | Prerequisites, build/run, scripts, single-crate layout | canonical |
| [contributing.md](contributing.md) | Contribution rules aligned with appliance architecture | canonical |
| [logging.md](logging.md) | Unified `logs/` streams and capture requirements | canonical |
| [installer.md](installer.md) | Windows packaging goals, tool choice, firewall | canonical |
| [release-process.md](release-process.md) | Channels, llama pin policy, stable checklist | canonical |

## Research

| Doc | Purpose | Status |
| --- | --- | --- |
| [research.md](research.md) | Phase-0 research checklist | canonical |
| [prior-art.md](prior-art.md) | External products, links, scaffolding recommendations | canonical |
| [research/01-windows-process.md](research/01-windows-process.md) | Job Objects, graceful stop, crash/restart | canonical |
| [research/02-llama-readiness.md](research/02-llama-readiness.md) | Health probes, `--fit`, pin policy, port conflicts | canonical |
| [research/03-hardware-detection.md](research/03-hardware-detection.md) | DXGI / NVML / CUDA inventory | canonical |
| [research/04-installer-licensing.md](research/04-installer-licensing.md) | Inno Setup, firewall, CUDA DLLs, MIT notices | canonical |
| [research/05-server-manager.md](research/05-server-manager.md) | State machine and public manager API | canonical |
| [research/06-competitor-architecture.md](research/06-competitor-architecture.md) | Ownership families + diagrams (pinned briefs) | building |
| [research/competitors/PLAN.md](research/competitors/PLAN.md) | Competitor research contract + confidence vocabulary | building |
| [research/competitors/STATUS.md](research/competitors/STATUS.md) | Brief roll-up PASS/GAP + residual uncertainties | building |
| [comparison.md](comparison.md) | WinServeAI vs competitor families; adopt/reject/defer | building |
| [audit/completion-audit.md](audit/completion-audit.md) | MVP implemented vs Windows-proven; test gaps | building |
| [audit/competitor-accuracy.md](audit/competitor-accuracy.md) | Independent accuracy gate for competitor reports | building |

Phase-0 deep dives carry a path-note banner mapping historical `packages/*` paths to `app/src/…` (see [STATUS.md](research/doc-build/STATUS.md)). Competitor briefs: `research/competitors/*.research.md`.

## Doc-build pipeline

| Doc | Purpose | Status |
| --- | --- | --- |
| [research/doc-build/PLAN.md](research/doc-build/PLAN.md) | Build-out targets and agent rules | canonical |
| [research/doc-build/STATUS.md](research/doc-build/STATUS.md) | Per-file PASS/FAIL and gaps (orchestrator) | canonical |
| `research/doc-build/*.research.md` | Per-doc research briefs used for the rebuild | canonical |

## Build specs (implementation contracts)

| Doc | Purpose | Status |
| --- | --- | --- |
| [specs/A2-smoke.md](specs/A2-smoke.md) | Operator smoke checklist; automated vs manual; scripts | building |
| [specs/E-desktop.md](specs/E-desktop.md) | Phase 2 desktop: UI surfaces, path picker, E3 manager options | building |
| [specs/F-installer.md](specs/F-installer.md) | Phase 3 Inno installer: layout, notices, firewall, shortcuts | building |

Scripts: `scripts/smoke-check.sh` (preflight, no inference), `scripts/smoke-openai.ps1` (Windows `/v1` poll + optional chat).

## Decisions

| Doc | Purpose | Status |
| --- | --- | --- |
| [adr/README.md](adr/README.md) | Architecture Decision Record index | canonical |
| [adr/0001-appliance-architecture.md](adr/0001-appliance-architecture.md) | Accepted: single-crate llama.cpp appliance | canonical |

Link to phase-0 research and [prior-art.md](prior-art.md) instead of duplicating deep dives in operator guides.
