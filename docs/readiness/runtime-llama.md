# Readiness: runtime-llama

| Field | Value |
| --- | --- |
| Path | `app/src/runtime/llama.rs` |
| Overall | **3.6 / 5** |
| Label | `mvp-ready` |
| Reviewed | 2026-07-04 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 4/5 | Sole owner of llama.cpp flags (`app/src/runtime/llama.rs`); `default_binary` + `build_command` from config + hardware. Matches architecture rule “raw flags only here”. |
| Implementation | 4/5 | Builds `--host`, `--port`, `--model`, `--ctx-size`, GPU layers / `--fit on`, `-fa on`. Auto+GPU path uses `--fit on` (omit `-ngl`); empty GPU list forces `--n-gpu-layers 0`. Explicit layers and flash-attention supported. |
| Tests | 3/5 | Argv snapshots: auto+empty GPUs → `-ngl 0`; auto+GPU → `--fit on`; explicit layers / flash-attention cases. |
| Docs | 4/5 | `docs/backend.md` documents flag ownership and mapping; research notes `--fit` in `docs/research/02-llama-readiness.md`. |
| Windows readiness | 3/5 | `default_binary` selects `llama-server.exe` under `cfg!(windows)`. Pin **b9866** documented in `bin/README.md`. |

## Gaps

- No binary version probe before spawn.
- A2 operator smoke with real `llama-server` still open.
- `auto: false` + `layers: auto` still uses `-ngl 99` when GPUs present (documented; prefer explicit layers).

## Next actions (ordered)

1. A2 smoke: `start` → `GET /v1/models` with pinned binary.
2. Optional binary version probe before spawn.
3. Revisit `-ngl 99` when `auto: false` and `layers: auto`.
