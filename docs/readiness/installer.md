# Readiness: installer

| Field | Value |
| --- | --- |
| Path | `installer/` + `scripts/stage-release.*` + `notices/` |
| Overall | **3.4 / 5** |
| Label | `mvp-partial` |
| Reviewed | 2026-07-30 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 4/5 | Goals and layout match `docs/specs/F-installer.md`; shortcuts → tray/manager; firewall opt-in; notices rollup + LICENSE/AUTHORS/CUDA. |
| Implementation | 4/5 | F0–F5: stage-release (incl. `FIRST_RUN.txt`) + ISS (`InfoAfterFile`) + notices + empty `model.path` + tray first-run banner. |
| Tests | 0/5 | No ISCC compile or smoke install in CI. |
| Docs | 5/5 | Operator guide + F-installer + first-run + installer/inno README + bin/README refresh steps. |
| Windows readiness | 2/5 | ISS authored for Win x64 + admin; needs operator ISCC compile proof. |

## Gaps

- CI does not compile the installer.
- Operator Windows install→Ready smoke still open (A2).

## Next actions (ordered)

1. Optional: CI job to run ISCC when Windows + staged files available.
2. A2 Windows smoke with user GGUF.
