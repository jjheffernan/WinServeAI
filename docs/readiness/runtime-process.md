# Readiness: runtime-process

| Field | Value |
| --- | --- |
| Path | `app/src/runtime/process.rs` |
| Overall | **2.4 / 5** |
| Label | `scaffold` |
| Reviewed | 2026-07-04 |

## Dimensions

| Dimension | Score | Evidence |
| --- | --- | --- |
| Design | 4/5 | Spawn, stdout/stderr capture, grace stop, force kill — shape matches `docs/research/01-windows-process.md` and architecture shutdown sequence. Job Object / real CTRL_BREAK designed in research, not implemented. |
| Implementation | 2/5 | `ChildProcess::spawn` pipes stdout/stderr to log channel, sets `CREATE_NEW_PROCESS_GROUP` on Windows, `kill_on_drop(true)`. `stop` waits grace then `start_kill`. `send_ctrl_break` is an intentional no-op stub (`app/src/runtime/process.rs` lines 116–122). No Job Object. Non-Windows uses `start_kill` immediately as “graceful”. |
| Tests | 0/5 | No process tests. |
| Docs | 4/5 | `docs/backend.md` + `docs/research/01-windows-process.md` describe intended Windows behavior and current gaps. |
| Windows readiness | 2/5 | Process group flag set, but graceful console control is stubbed; stop relies on force kill after grace. No Job Object for orphan cleanup. |

## Gaps

- `send_ctrl_break` does not call `GenerateConsoleCtrlEvent`.
- No Windows Job Object to kill child trees.
- No tests for spawn/stop.

## Next actions (ordered)

1. Implement real CTRL_BREAK (or attach/console gymnastics) for graceful llama-server exit.
2. Assign child to a Job Object with `KILL_ON_JOB_CLOSE`.
3. Integration test: spawn a short-lived process, stop within grace.
