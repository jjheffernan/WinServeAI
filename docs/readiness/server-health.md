# Readiness: server-health

| Field | Value |
| --- | --- |
| Path | `app/src/server/health.rs` |
| Overall | **3.4 / 5** |
| Label | `mvp-partial` |
| Reviewed | 2026-07-04 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 4/5 | Readiness = HTTP success on `GET {base}/v1/models`, not process-alive alone — matches `docs/architecture.md` and `docs/research/02-llama-readiness.md`. |
| Implementation | 3/5 | Real `reqwest` poll in `wait_until_ready` (`app/src/server/health.rs`): 2s request timeout, 250ms interval, 503 treated as still loading, overall deadline (manager uses 120s). Other non-success statuses and errors are swallowed until timeout. No `/health` fallback. |
| Tests | 3/5 | `times_out_when_nothing_listens` asserts `HealthError::Timeout`. No 503→200 mock success path. |
| Docs | 4/5 | `docs/api.md` documents readiness probes and READY semantics. |
| Windows readiness | 3/5 | HTTP client is platform-agnostic; works on Win10/11 dev. |

## Gaps

- No alternate probe (`/health`) if `/v1/models` shape changes.
- Failures other than timeout are not distinguished beyond `Timeout` / `Http` (client build only).
- No success-path integration test (503 then 200).

## Next actions (ordered)

1. Integration test with a tiny local HTTP server returning 503 then 200.
2. Surface last HTTP status/error in `HealthError` for diagnostics.
3. Optional `/health` fallback per research notes.
