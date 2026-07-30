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
| Implementation | 4/5 | F0 stage-release + F1 ISS + F4 `notices/` (`THIRD_PARTY_NOTICES.md`, `llama.cpp-LICENSE.txt`, `llama.cpp-AUTHORS.txt`, `NVIDIA-CUDA-NOTICE.txt`) + `scripts/refresh-notices.sh`. F5 first-run open. |
| Tests | 0/5 | No ISCC compile or smoke install in CI. |
| Docs | 5/5 | Operator guide + F-installer + installer/inno README + bin/README refresh steps. |
| Windows readiness | 2/5 | ISS authored for Win x64 + admin; needs operator ISCC compile proof. |

## Gaps

- First-run model path guidance (F5).
- CI does not compile the installer.

## Next actions (ordered)

1. F5 — first-run model path guidance.
2. Optional: CI job to run ISCC when Windows + staged files available.
