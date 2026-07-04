# Readiness: installer

| Field | Value |
| --- | --- |
| Path | `installer/` |
| Overall | **1.4 / 5** |
| Label | `stub` |
| Reviewed | 2026-07-04 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 3/5 | Goals and install layout documented (`installer/README.md`, `docs/installer.md`, `docs/research/04-installer-licensing.md`): Inno Setup preferred, firewall only for non-loopback, ship notices. |
| Implementation | 0/5 | `installer/` contains only `README.md`. No `.iss` / NSIS / WiX scripts. |
| Tests | 0/5 | No installer build or smoke install. |
| Docs | 4/5 | Operator guide + research on licensing, CUDA DLLs, firewall, tool choice. |
| Windows readiness | 0/5 | No Windows installer artifact or script exists. |

## Gaps

- No Inno Setup script.
- No packaging of `winserve.exe` + `bin/llama-server.exe`.
- No firewall rule automation.
- No first-run wizard (Phase 3 scope).

## Next actions (ordered)

1. Add minimal `installer/winserve.iss` installing release binary + config + notices.
2. Bundle pinned `llama-server.exe` and private DLLs.
3. Conditional firewall rule when bind is non-loopback.
