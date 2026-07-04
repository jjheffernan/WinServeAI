# Readiness: server-config

| Field | Value |
| --- | --- |
| Path | `app/src/server/config.rs` + `config/default.yaml` |
| Overall | **3.6 / 5** |
| Label | `mvp-ready` |
| Reviewed | 2026-07-04 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 4/5 | YAML is source of truth; no raw llama flags (`app/src/server/config.rs` module docs). Sections: `server`, `model`, `gpu`, `runtime`, `logging`. Matches `docs/architecture.md` and `docs/configuration.md`. |
| Implementation | 4/5 | `Config::load` / `save` / `validate` / `warnings` / `base_url` / `openai_v1_url`. Validation: non-zero port, non-empty host, non-empty `model.path`, `gpu.layers` is `"auto"` or integer, non-zero `runtime.context`. Soft warnings for missing model path and privileged ports. |
| Tests | 3/5 | Default validates; rejects zero port / bad layers; accepts numeric layers; YAML round-trip. |
| Docs | 4/5 | `docs/configuration.md` mirrors `config/default.yaml` and documents localhost vs LAN. |
| Windows readiness | 3/5 | Default model path uses Windows style (`D:\models\model.gguf` in `config/default.yaml`); `PathBuf` works on Win10/11. |

## Gaps

- Model file existence is a warning at load; hard-fail remains in manager at start.
- No config migration / version field.

## Next actions (ordered)

1. Document and implement config migration when schema changes.
2. Optional: surface `warnings()` in CLI `print-config` / start banner.
