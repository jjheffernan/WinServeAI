# Readiness: scripts-ops

| Field | Value |
| --- | --- |
| Path | `scripts/{start,stop,reset}.ps1`, `scripts/smoke-openai.ps1`, `scripts/smoke-check.sh` |
| Overall | **3.4 / 5** |
| Label | `mvp-partial` |
| Reviewed | 2026-07-30 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 4/5 | Thin PowerShell wrappers for local dev plus A2 operator smoke: poll `GET /v1/models`, optional chat, pin **b9866** preflight. Aligns with `docs/specs/A2-smoke.md` and `docs/development.md`. |
| Implementation | 4/5 | `start.ps1` / `stop.ps1` / `reset.ps1` for local Windows dev. `stop.ps1` prefers `winserve stop` when `%LOCALAPPDATA%\WinServeAI\manager.lock` owner is alive; `-Force` and missing-lock paths keep name-based kill. `smoke-openai.ps1` / `smoke-check.sh` unchanged. |
| Tests | 1/5 | Manual for PS1; `smoke-check.sh` is runnable preflight (not a formal script unit suite). |
| Docs | 4/5 | Documented in `docs/development.md` Scripts section and `docs/specs/A2-smoke.md`. |
| Windows readiness | 4/5 | Native PowerShell path; graceful IPC stop when resident serve is up; force remains fallback. |

## Gaps

- No check that build exists before `start.ps1` (falls through release → debug).
- No automated script unit tests; full A2 still needs operator Windows + GGUF.

## Next actions (ordered)

1. Fail clearly when neither release nor debug binary is present (`start.ps1`).
2. Close A2 after operator runs full Windows smoke (stop + orphan checks).
