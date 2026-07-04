# Readiness: scripts-ops

| Field | Value |
| --- | --- |
| Path | `scripts/{start,stop,reset}.ps1` |
| Overall | **2.8 / 5** |
| Label | `mvp-partial` |
| Reviewed | 2026-07-04 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 3/5 | Thin PowerShell wrappers for local dev: start winserve, force-stop processes, reset logs. Aligns with development workflow in `docs/development.md`. |
| Implementation | 3/5 | `start.ps1` sets `WINSERVE_CONFIG` and runs release then debug `winserve.exe`. `stop.ps1` force-stops processes named `winserve` and `llama-server`. `reset.ps1` calls stop and clears `logs/` files. Happy path for local Windows dev works; not graceful manager stop. |
| Tests | 1/5 | Manual only. |
| Docs | 4/5 | Documented in `docs/development.md` Scripts section with behavior notes. |
| Windows readiness | 3/5 | Native PowerShell; appropriate for Win10/11 dev. Force-kill is blunt but effective. |

## Gaps

- `stop.ps1` uses `Stop-Process -Force`, not `ServerManager::stop`.
- No check that build exists before start (falls through release → debug).
- No automated script tests.

## Next actions (ordered)

1. Prefer graceful stop if a manager IPC/PID file exists; keep force as fallback.
2. Fail clearly when neither release nor debug binary is present.
3. Optional: `scripts/check.sh` already gates cargo; leave PS1 as manual ops.
