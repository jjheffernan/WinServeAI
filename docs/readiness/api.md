# Readiness: api

| Field | Value |
| --- | --- |
| Path | `app/src/api/*` |
| Overall | **3.2 / 5** |
| Label | `mvp-partial` |
| Reviewed | 2026-07-31 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 3/5 | Passthrough-only by design: clients hit llama-server `/v1` directly (`app/src/api/mod.rs`, `openai.rs`). Matches architecture — no proxy, no chat stack. |
| Implementation | 3/5 | `chat_completions_url` and `models_url` format URLs from `Config` (`app/src/api/openai.rs`). Complete for intended scope; not an HTTP server or proxy. Manager uses `Config::openai_v1_url` more than these helpers. |
| Tests | 3/5 | Unit tests for URL helpers in `app/src/api/openai.rs` (sample host/port composition). |
| Docs | 4/5 | `docs/api.md` documents base URL, endpoints, readiness, and that helpers are not a server. |
| Windows readiness | 3/5 | Pure string formatting; works on Windows. |

## Gaps

- Helpers underused vs `Config::openai_v1_url` / `base_url`.
- No proxy (intentional; not a Phase 1 gap unless requirements change).

## Next actions (ordered)

1. Prefer helpers consistently from manager/CLI if kept.
2. Leave proxy out of MVP (per architecture).
