# Readiness: bin

| Field | Value |
| --- | --- |
| Path | `bin/` + `scripts/fetch-llama-pin.*` + `notices/` (pin notices) |
| Overall | **3.2 / 5** |
| Label | `mvp-partial` |
| Reviewed | 2026-07-30 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 4/5 | External boundary unchanged: operator/installer supplies pinned `llama-server` under `bin/`; `runtime::llama::default_binary` resolves it. Fetch scripts + notices keep git free of the binary. |
| Implementation | 3/5 | `scripts/fetch-llama-pin.ps1` / `.sh` pull pin **b9866** (CPU/CUDA/Vulkan assets) and sibling DLLs into `bin/`. Binary still not committed (by design). |
| Tests | 1/5 | Manual fetch validation; no checksum assert or CI job that downloads the pin. |
| Docs | 5/5 | `bin/README.md`, A2 smoke, `release-process.md`, `development.md`, `notices/THIRD_PARTY_NOTICES.md` + LICENSE/AUTHORS/CUDA. |
| Windows readiness | 3/5 | Primary path is Win x64 `.exe` + DLLs via PowerShell fetch; macOS/Linux helpers exist for cross-dev only. |

## Gaps

- No `llama-server.exe` in git (intentional); out-of-box still needs fetch or installer.
- Fetch is manually validated — no checksum / automated pin test in CI.
- A2 operator smoke with real GGUF still open.

## Next actions (ordered)

1. Optional CI: fetch pin on `windows-latest` and `print-cmd` / unit smoke (no inference).
2. Record shipped `b####` in release docs (I4).
3. Close A2 after Windows + GGUF proof.
