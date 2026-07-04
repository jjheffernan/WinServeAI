# Project TODO (from module readiness)

**Source:** [readiness/README.md](readiness/README.md) · **Project maturity:** **2.5/5** (`mvp-partial`)  
**Branch context:** appliance layout (`app/`, `bin/`, `config/`) · **Ordered plan:** [PLAN.md](PLAN.md)  
**Anti-drift:** [policies/doc-drift.md](policies/doc-drift.md) · `python3 scripts/check_doc_drift.py`

## Phase 1 — Core engine (priority)

| Priority | Module | Score | Action |
| --- | --- | --- | --- |
| P0 | `runtime-process` | 2.4 scaffold | Implement Windows `CTRL_BREAK` / Job Object; stop relying on force-kill only |
| P0 | `system` | 1.6 scaffold | DXGI (VRAM) + optional NVML; stop returning empty GPU list |
| P0 | `bin` | 1.4 stub | Pin and document `bin/llama-server.exe` for local/dev (not necessarily commit binary) |
| P1 | `server-manager` | 2.8 mvp-partial | Long-running manager so `stop`/`restart` work; exit watcher → `Crashed` |
| P1 | `cli` | 2.6 mvp-partial | Wire `stop`/`restart` once manager is resident (tray/service or lockfile+IPC) |
| P1 | `runtime-llama` | 2.8 mvp-partial | Exercise `--fit` path once GPU detection works; pin flag set to release |
| P1 | `server-logs` | 2.6 mvp-partial | Timestamps on lines; plan rotation |
| P1 | tests | 0 across modules | Unit tests for config/argv; process stub exe; health timeout behavior |
| P2 | `server-health` | 2.8 mvp-partial | Optional `/health` fallback; generous timeouts under load |
| P2 | `server-config` | 2.8 mvp-partial | Stronger validation (model exists warning, port range) |
| P2 | `scripts-ops` | 2.8 mvp-partial | Prefer graceful stop over `Stop-Process -Force` |

## Phase 2 — Desktop

| Priority | Module | Score | Action |
| --- | --- | --- | --- |
| P3 | `ui` | 0.4 stub | Tauri (or chosen) shell: start/stop/logs/status only via ServerManager |

## Phase 3 — Installer

| Priority | Module | Score | Action |
| --- | --- | --- | --- |
| P3 | `installer` | 1.4 stub | Inno Setup script, shortcut, firewall only for non-loopback, notices |

## Already stronger

| Module | Score | Note |
| --- | --- | --- |
| `docs` | 3.6 mvp-ready | Operator guides + research; keep scores updated |
| `scripts-pr-loop` | 3.0 mvp-partial | Dry-run works; wire real agent cmds when needed |
| `api` | 2.6 mvp-partial | Passthrough URLs sufficient for MVP |

## How to refresh scores

Re-run the readiness review (see [readiness/PLAN.md](readiness/PLAN.md)) after material code changes; update scorecards and the **Readiness** blocks at the top of operator guides.

## See also

- [PLAN.md](./PLAN.md) — ordered implementation milestones
- [readiness/README.md](./readiness/README.md) — maturity dashboard
- [readiness/PLAN.md](./readiness/PLAN.md) — scoring rubric
- [roadmap.md](./roadmap.md) — phased product plan
- [policies/doc-drift.md](./policies/doc-drift.md) — scorecards must match banners
- [policies/SOURCES.md](./policies/SOURCES.md) — canonical external URLs
