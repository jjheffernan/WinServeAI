# Documentation index

WinServeAI docs for the **llama.cpp appliance** architecture (`ServerManager` only, no backend traits).

## Module readiness

| Doc | Purpose |
| --- | --- |
| [readiness/README.md](readiness/README.md) | Maturity at-a-glance: per-module overall score, label, top gap, and links to scorecards |

Scorecards live under [readiness/](readiness/) (Design, Implementation, Tests, Docs, Windows readiness). Primary operator guides also carry a short **Readiness** block at the top.

Prioritized work from scores: [TODO.md](TODO.md).

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

Phase-0 deep dives carry a path-note banner mapping historical `packages/*` paths to `app/src/…` (see [STATUS.md](research/doc-build/STATUS.md)).

## Doc-build pipeline

| Doc | Purpose | Status |
| --- | --- | --- |
| [research/doc-build/PLAN.md](research/doc-build/PLAN.md) | Build-out targets and agent rules | canonical |
| [research/doc-build/STATUS.md](research/doc-build/STATUS.md) | Per-file PASS/FAIL and gaps (orchestrator) | canonical |
| `research/doc-build/*.research.md` | Per-doc research briefs used for the rebuild | canonical |

## Decisions

| Doc | Purpose | Status |
| --- | --- | --- |
| [adr/README.md](adr/README.md) | Architecture Decision Record index | canonical |
| [adr/0001-appliance-architecture.md](adr/0001-appliance-architecture.md) | Accepted: single-crate llama.cpp appliance | canonical |

Link to phase-0 research and [prior-art.md](prior-art.md) instead of duplicating deep dives in operator guides.
