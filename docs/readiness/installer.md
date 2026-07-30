# Readiness: installer

| Field | Value |
| --- | --- |
| Path | `installer/` + `scripts/stage-release.*` |
| Overall | **3.0 / 5** |
| Label | `mvp-partial` |
| Reviewed | 2026-07-30 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 4/5 | Goals and layout match `docs/specs/F-installer.md`; shortcuts → tray/manager; firewall opt-in for LAN. |
| Implementation | 3/5 | F0 `stage-release` + F1 `installer/inno/WinServeAI.iss` (Files/Icons/Tasks/UninstallRun). No compiled setup.exe in CI yet; F4/F5 open. |
| Tests | 0/5 | No ISCC compile or smoke install in CI. |
| Docs | 5/5 | Operator guide + F-installer + `installer/inno/README.md` build steps. |
| Windows readiness | 2/5 | ISS authored for Win x64 + admin; needs operator ISCC compile proof. |

## Gaps

- Notices payload polish / LICENSE copies at pin (F4).
- First-run model path guidance (F5).
- CI does not compile the installer.

## Next actions (ordered)

1. F4 — expand `notices/` with llama.cpp LICENSE/AUTHORS (+ CUDA note when shipping CUDA).
2. F5 — first-run model path guidance.
3. Optional: CI job to run ISCC when Windows + staged files available.
