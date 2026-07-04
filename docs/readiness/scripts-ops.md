# Readiness: scripts-ops

| Field | Value |
| --- | --- |
| Path | `scripts/{start,stop,reset}.ps1`, `scripts/smoke-openai.ps1`, `scripts/smoke-check.sh` |
| Overall | **3.2 / 5** |
| Label | `mvp-partial` |
| Reviewed | 2026-07-04 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 4/5 | Thin PowerShell wrappers for local dev plus A2 operator smoke: poll `GET /v1/models`, optional chat, pin **b9866** preflight. Aligns with `docs/specs/A2-smoke.md` and `docs/development.md`. |
| Implementation | 4/5 | `start.ps1` / `stop.ps1` / `reset.ps1` for local Windows dev. `smoke-openai.ps1` reads host/port from YAML, polls readiness (≥120s), uses model id from `/v1/models`, port-busy check on `-Start`. `smoke-check.sh` runs tests + `print-cmd` without GGUF. |
| Tests | 1/5 | Manual for PS1; `smoke-check.sh` is runnable preflight (not a formal script unit suite). |
| Docs | 4/5 | Documented in `docs/development.md` Scripts section and `docs/specs/A2-smoke.md`. |
| Windows readiness | 3/5 | Native PowerShell smoke path; force-kill `stop.ps1` remains blunt (not A2 pass criteria). |

## Gaps

- `stop.ps1` uses `Stop-Process -Force`, not `ServerManager::stop`.
- No check that build exists before `start.ps1` (falls through release → debug).
- No automated script unit tests; full A2 still needs operator Windows + GGUF.

## Next actions (ordered)

1. Prefer graceful stop if a manager IPC/PID file exists; keep force as fallback.
2. Fail clearly when neither release nor debug binary is present (`start.ps1`).
3. Close A2 after operator runs full Windows smoke (stop + orphan checks).
