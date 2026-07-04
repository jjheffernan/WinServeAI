# Module readiness review plan

**Branch:** `dev` (work) · `main` (releases)  
**Goal:** Score each owned module for completeness and code maturity so status is visible at a glance.

## Scoring rubric (0–5 each dimension)

| Dimension | 0 | 1–2 | 3 | 4 | 5 |
| --- | --- | --- | --- | --- | --- |
| **Design** | Missing / wrong shape | Sketch only | Matches architecture | Documented + aligned | Locked by ADR / tests of contract |
| **Implementation** | Absent | Stub / TODO-heavy | Happy path works | Edge cases handled | Production-hardened |
| **Tests** | None | Manual only | Smoke / few unit | Unit + integration | CI matrix coverage |
| **Docs** | None | Stale / wrong paths | Operator guide exists | Guide matches code | Guide + research + examples |
| **Windows readiness** | N/A or broken | Partial / Unix-only assumptions | Works on Win10/11 dev | Graceful stop, logs, ports | Installer + Job Object + firewall |

**Overall score** = average of applicable dimensions, reported as `X.X / 5` and a label:

| Overall | Label |
| --- | --- |
| 0.0–1.4 | `stub` |
| 1.5–2.4 | `scaffold` |
| 2.5–3.4 | `mvp-partial` |
| 3.5–4.4 | `mvp-ready` |
| 4.5–5.0 | `production` |

## Modules in scope

| ID | Path | Primary docs |
| --- | --- | --- |
| `server-manager` | `app/src/server/manager.rs` | `backend.md`, `research/05` |
| `server-config` | `app/src/server/config.rs` + `config/default.yaml` | `configuration.md` |
| `server-health` | `app/src/server/health.rs` | `api.md`, `research/02` |
| `server-logs` | `app/src/server/logs.rs` | `logging.md` |
| `runtime-llama` | `app/src/runtime/llama.rs` | `backend.md` |
| `runtime-process` | `app/src/runtime/process.rs` | `backend.md`, `research/01` |
| `system` | `app/src/system/*` | `research/03` |
| `api` | `app/src/api/*` | `api.md` |
| `cli` | `app/src/main.rs` | `development.md` |
| `bin` | `bin/` | `backend.md`, `release-process.md` |
| `installer` | `installer/` | `installer.md`, `research/04` |
| `scripts-ops` | `scripts/{start,stop,reset}.ps1`, `smoke-openai.ps1`, `smoke-check.sh` | `development.md`, `specs/A2-smoke.md` |
| `scripts-pr-loop` | `scripts/pr_review_loop/` | `scripts/pr_review_loop/README.md` |
| `docs` | `docs/` | `INDEX.md` |
| `ui` | `app/ui/` | (Phase 2) |

## Deliverables

1. `docs/readiness/README.md` — dashboard table (module, overall, label, top gap)
2. `docs/readiness/<id>.md` — per-module scorecard (dimensions, evidence, gaps, next actions)
3. Pipe into module docs: add a short **Readiness** block at the top of each primary doc linking to the scorecard and showing `overall / label`
4. Update `docs/INDEX.md` with a Readiness section
5. Update `docs/roadmap.md` only if scores imply phase adjustments (optional, one paragraph)

## Rules

- Score **code as it exists**, not aspirational design
- Cite file paths and line-level behaviors as evidence
- No application code changes
- Appliance architecture: no backend traits, one ServerManager
