# Readiness: server-config

| Field | Value |
| --- | --- |
| Path | `app/src/server/config.rs` + `config/default.yaml` |
| Overall | **2.8 / 5** |
| Label | `mvp-partial` |
| Reviewed | 2026-07-04 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 4/5 | YAML is source of truth; no raw llama flags (`app/src/server/config.rs` module docs). Sections: `server`, `model`, `gpu`, `runtime`, `logging`. Matches `docs/architecture.md` and `docs/configuration.md`. |
| Implementation | 3/5 | `Config::load` / `save` / `validate` / `base_url` / `openai_v1_url` work. Defaults and serde defaults present. Validation only checks non-zero port and non-empty `model.path` — no host format, path existence, or layers parsing beyond passthrough. |
| Tests | 0/5 | No config unit tests. |
| Docs | 4/5 | `docs/configuration.md` mirrors `config/default.yaml` and documents localhost vs LAN; maps to `runtime/llama.rs` for flags. |
| Windows readiness | 3/5 | Default model path uses Windows style (`D:\models\model.gguf` in `config/default.yaml`); `PathBuf` works on Win10/11. |

## Gaps

- No automated tests for parse/validate/save round-trip.
- Validation does not check model file existence (manager does at start).
- No config migration / version field.

## Next actions (ordered)

1. Unit tests for load, defaults, validate failures, URL helpers.
2. Optionally validate `gpu.layers` is `"auto"` or integer string.
3. Document and implement config migration when schema changes.
