# Readiness: api

| Field | Value |
| --- | --- |
| Path | `server/src/api/*` |
| Overall | **3.2 / 5** |
| Label | `mvp-partial` |
| Reviewed | 2026-07-31 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 3/5 | Passthrough-only by design: clients hit llama-server completion routes directly (`server/src/api/mod.rs`, `openai.rs`). Matches architecture — no proxy, no chat stack. |
| Implementation | 3/5 | URL helpers for models, chat, responses, legacy completions, Anthropic messages (`server/src/api/openai.rs`). Complete for intended scope; not an HTTP server or proxy. Manager uses `Config::openai_v1_url` / `base_url` more than these helpers. |
| Tests | 3/5 | Unit tests for URL helpers in `server/src/api/openai.rs` (four completion routes + models; no `/v1/v1`). |
| Docs | 4/5 | `docs/api.md` documents both bases, all common completion methods, OpenClaw `api` map, readiness, and that helpers are not a server. |
| Windows readiness | 3/5 | Pure string formatting; works on Windows. Smoke: `scripts/smoke-openai.ps1` hits chat, responses, messages, completions. |

## Gaps

- Helpers underused vs `Config::openai_v1_url` / `base_url`.
- No proxy (intentional; not a Phase 1 gap unless requirements change).

## Next actions (ordered)

1. Prefer helpers consistently from manager/CLI if kept.
2. Leave proxy out of MVP (per architecture).
