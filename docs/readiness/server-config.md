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
| Implementation | 4/5 | `Config::load` / `save` / `validate` / `warnings` / `base_url` / `openai_v1_url`. Validation: non-zero port, non-empty host, `gpu.layers` is `"auto"` or integer, non-zero `runtime.context`. Empty `model.path` allowed at load (first-run); soft warnings for empty/missing model path and privileged ports. |
| Tests | 3/5 | Default validates; rejects zero port / bad layers; accepts numeric layers; empty `model.path` warning; YAML round-trip. |
| Docs | 4/5 | `docs/configuration.md` + `docs/first-run.md` mirror `config/default.yaml` and localhost vs LAN. |
| Windows readiness | 3/5 | Shipped empty `model.path` + `FIRST_RUN.txt` / tray Browse; `PathBuf` works on Win10/11. Operator still supplies the GGUF. |

## Gaps

- Empty/missing model path is a warning at load; hard-fail remains in manager at start.
- No config migration / version field.

## Next actions (ordered)

1. Document and implement config migration when schema changes.
2. Optional: surface `warnings()` in CLI `print-config` / start banner.
