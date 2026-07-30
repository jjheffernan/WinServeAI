# Readiness: server-health

| Field | Value |
| --- | --- |
| Path | `app/src/server/health.rs` |
| Overall | **3.8 / 5** |
| Label | `mvp-ready` |
| Reviewed | 2026-07-30 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 4/5 | Readiness = HTTP success on `GET {base}/v1/models` (primary), with `GET {base}/health` fallback — matches `docs/architecture.md` and `docs/research/02-llama-readiness.md`. |
| Implementation | 4/5 | Real `reqwest` poll in `wait_until_ready` (`app/src/server/health.rs`): 2s request timeout, 250ms interval; each tick tries `/v1/models` then `/health`; overall deadline (manager uses 120s). |
| Tests | 4/5 | Timeout when nothing listens; mock TCP server for `/v1/models` 503→200; `/health` fallback when models returns 404. |
| Docs | 4/5 | `docs/api.md` documents primary + fallback readiness probes and READY semantics. |
| Windows readiness | 3/5 | HTTP client is platform-agnostic; works on Win10/11 dev. |

## Gaps

- Failures other than timeout are not distinguished beyond `Timeout` / `Http` (client build only).
- Last HTTP status/error is not surfaced on timeout for diagnostics.

## Next actions (ordered)

1. Surface last HTTP status/error in `HealthError` for diagnostics.
