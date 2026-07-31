# Readiness: installer

| Field | Value |
| --- | --- |
| Path | `installer/` + `scripts/stage-release.*` + `notices/` |
| Overall | **3.4 / 5** |
| Label | `mvp-partial` |
| Reviewed | 2026-07-31 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 4/5 | Goals and layout match `docs/specs/F-installer.md`; shortcuts → tray/manager; firewall opt-in; notices rollup + LICENSE/AUTHORS/CUDA. |
| Implementation | 4/5 | F0–F5: stage-release (incl. `FIRST_RUN.txt`) + ISS (`InfoAfterFile`) + notices + empty `model.path` + tray first-run banner. |
| Tests | 3/5 | CI: stage-release layout asserts + `ISCC.exe` compile on `windows-latest`; no clean-machine install→Ready automation. |
| Docs | 5/5 | Operator guide + F-installer + first-run + installer/inno README + bin/README refresh steps. |
| Windows readiness | 3/5 | ISS compiles in CI; install→Ready operator smoke still open. |

## Gaps

- Operator Windows install→Ready smoke still open (A2).

## Next actions (ordered)

1. A2 Windows smoke with user GGUF + clean-machine install→Ready.
