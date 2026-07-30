# Readiness: installer

| Field | Value |
| --- | --- |
| Path | `installer/` + `scripts/stage-release.*` |
| Overall | **2.2 / 5** |
| Label | `scaffold` |
| Reviewed | 2026-07-30 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 3/5 | Goals and install layout documented (`installer/README.md`, `docs/installer.md`, `docs/specs/F-installer.md`). |
| Implementation | 2/5 | F0 staging scripts `scripts/stage-release.ps1` / `.sh` produce `{app}` tree (exe, tray, bin, config, notices, logs). No `.iss` yet (F1). |
| Tests | 0/5 | No installer build or smoke install. |
| Docs | 5/5 | Operator guide + research + F-installer build contract; installer README documents stage-release. |
| Windows readiness | 1/5 | Staging script is Windows-first; Inno artifact still missing. |

## Gaps

- No Inno Setup script (F1).
- No firewall rule automation (F3).
- No first-run wizard (F5).

## Next actions (ordered)

1. Implement `installer/inno/WinServeAI.iss` consuming `installer/inno/files` (F1).
2. Shortcuts → tray/manager (F2); conditional firewall (F3); notices payload polish (F4).
3. First-run model path guidance (F5).
