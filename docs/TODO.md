# Project TODO (from module readiness)

**Source:** [readiness/README.md](readiness/README.md) · **Project maturity:** **3.1/5** (`mvp-partial`)  
**Branch context:** appliance layout (`app/`, `bin/`, `config/`) · **Ordered plan:** [PLAN.md](PLAN.md)  
**Anti-drift:** [policies/doc-drift.md](policies/doc-drift.md) · `python3 scripts/check_doc_drift.py`

## Phase 1 — Remaining

| Priority | Module | Score | Action |
| --- | --- | --- | --- |
| P0 | operator | — | **A2** smoke on Windows: `start` → `GET /v1/models` with **b9866** + GGUF ([specs/A2-smoke.md](specs/A2-smoke.md)) |
| P1 | `server-manager` | 3.4 mvp-partial | Long-running manager so CLI `stop`/`restart` work (**E3**) |
| P1 | `cli` | 2.6 mvp-partial | Wire `stop`/`restart` once manager is resident (tray/service or lockfile+IPC) |
| P2 | `server-logs` | 2.8 mvp-partial | Rotation; unit tests for `ts=` lines |
| P2 | `server-health` | 3.4 mvp-partial | Optional `/health` fallback; 503→200 mock test |
| P2 | `bin` | 2.2 scaffold | Optional fetch script for **b9866**; `THIRD_PARTY_NOTICES` |
| P2 | `scripts-ops` | 3.2 mvp-partial | Prefer graceful stop over `Stop-Process -Force` when IPC exists |
| P2 | `api` | 2.6 mvp-partial | Unit tests for URL helpers |

## Phase 2 — Desktop

| Priority | Module | Score | Action |
| --- | --- | --- | --- |
| P3 | `ui` | 0.4 stub | Tauri (or chosen) shell per [specs/E-desktop.md](specs/E-desktop.md): start/stop/logs/status only via ServerManager |
| P3 | `cli` / manager | — | **E3** long-running owner for stop/restart |

## Phase 3 — Installer

| Priority | Module | Score | Action |
| --- | --- | --- | --- |
| P3 | `installer` | 1.6 scaffold | Inno Setup per [specs/F-installer.md](specs/F-installer.md): shortcut, firewall only for non-loopback, notices |

## Already stronger (Phase 1 code)

| Module | Score | Note |
| --- | --- | --- |
| `runtime-process` | 3.8 mvp-ready | Job Object + CTRL_BREAK; spawn/stop unit test |
| `system` | 3.6 mvp-ready | DXGI + nvidia-smi + sysinfo RAM |
| `runtime-llama` | 3.6 mvp-ready | `--fit` / `-ngl 0` argv tests |
| `server-config` | 3.6 mvp-ready | Validation + warnings + unit tests |
| `docs` | 4.0 mvp-ready | Scorecards + build specs (A2/E/F) + `check_doc_drift.py` |
| `scripts-pr-loop` | 3.0 mvp-partial | Dry-run works; wire real agent cmds when needed |

## How to refresh scores

Re-run the readiness review (see [readiness/PLAN.md](readiness/PLAN.md)) after material code changes; update scorecards and the **Readiness** blocks at the top of operator guides.

## See also

- [PLAN.md](./PLAN.md) — ordered implementation milestones
- [readiness/README.md](./readiness/README.md) — maturity dashboard
- [readiness/PLAN.md](readiness/PLAN.md) — scoring rubric
- [roadmap.md](./roadmap.md) — phased product plan
- [policies/doc-drift.md](./policies/doc-drift.md) — scorecards must match banners
- [policies/SOURCES.md](./policies/SOURCES.md) — canonical external URLs
