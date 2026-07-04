# Readiness: runtime-llama

| Field | Value |
| --- | --- |
| Path | `app/src/runtime/llama.rs` |
| Overall | **2.8 / 5** |
| Label | `mvp-partial` |
| Reviewed | 2026-07-04 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 4/5 | Sole owner of llama.cpp flags (`app/src/runtime/llama.rs`); `default_binary` + `build_command` from config + hardware. Matches architecture rule “raw flags only here”. |
| Implementation | 3/5 | Builds `--host`, `--port`, `--model`, `--ctx-size`, GPU layers / `--fit on`, `-fa on`. Auto+GPU path uses `--fit`; empty GPU list forces `--n-gpu-layers 0`. Because `system::gpu::detect_gpus` always returns `[]`, auto mode always takes the CPU path in practice. |
| Tests | 0/5 | No argv-building tests. |
| Docs | 4/5 | `docs/backend.md` documents flag ownership and mapping; research notes `--fit` in `docs/research/02-llama-readiness.md`. |
| Windows readiness | 3/5 | `default_binary` selects `llama-server.exe` under `cfg!(windows)`. |

## Gaps

- No tests for auto / explicit layers / flash-attention argv.
- `--fit` branch never exercised until GPU detection is real.
- No pin/version check of the binary.

## Next actions (ordered)

1. Unit tests for `build_command` with empty vs non-empty `HardwareInfo.gpus`.
2. Land real GPU detection (`system`) so auto/`--fit` works on Windows.
3. Optional binary version probe before spawn.
