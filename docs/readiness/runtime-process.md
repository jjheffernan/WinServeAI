# Readiness: runtime-process

| Field | Value |
| --- | --- |
| Path | `app/src/runtime/process.rs` |
| Overall | **3.8 / 5** |
| Label | `mvp-ready` |
| Reviewed | 2026-07-04 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 4/5 | Spawn, stdout/stderr capture, grace stop, force kill — matches `docs/research/01-windows-process.md` and architecture shutdown sequence. Job Object + CTRL_BREAK are implemented (hand-rolled Win32, not `process-wrap`). |
| Implementation | 4/5 | `ChildProcess::spawn` pipes stdio, sets `CREATE_NEW_PROCESS_GROUP \| CREATE_SUSPENDED`, assigns Job Object (`KILL_ON_JOB_CLOSE`) then resumes the primary thread, `kill_on_drop(true)`. `stop` sends real `CTRL_BREAK` via `AttachConsole` + `GenerateConsoleCtrlEvent`, waits grace, then `start_kill`. `Drop` closes the job handle. |
| Tests | 3/5 | `spawn_and_stop_sleep_command` (`sleep` / `cmd /C ping`) asserts spawn + stop. No dedicated orphan-after-manager-kill integration test. |
| Docs | 4/5 | `docs/backend.md` + `docs/research/01-windows-process.md` describe Windows behavior. |
| Windows readiness | 4/5 | Job Object + CTRL_BREAK path compiles and is exercised on `windows-latest` CI unit tests. Graceful stop + orphan reaping on manager death are in code; not yet proven with real `llama-server` (A2). |

## Gaps

- No integration test that force-killing the manager reaps `llama-server`.
- `CREATE_NO_WINDOW` / tray-parent attach edge cases unvalidated with pinned binary.

## Next actions (ordered)

1. Operator smoke (A2) with real `llama-server` — confirm CTRL_BREAK flush and job reaping.
2. Integration test: drop job handle → child gone.
