# Module readiness

Last reviewed: 2026-07-04

| Module | Overall | Label | Top gap | Scorecard |
| --- | --- | --- | --- | --- |
| server-manager | 2.8/5 | mvp-partial | No tests; CLI stop/restart incomplete | [server-manager.md](./server-manager.md) |
| server-config | 2.8/5 | mvp-partial | No tests; thin validation | [server-config.md](./server-config.md) |
| server-health | 2.8/5 | mvp-partial | No tests; no `/health` fallback | [server-health.md](./server-health.md) |
| server-logs | 2.6/5 | mvp-partial | No timestamps/rotation; no tests | [server-logs.md](./server-logs.md) |
| runtime-llama | 2.8/5 | mvp-partial | No tests; `--fit` path unused (GPU stub) | [runtime-llama.md](./runtime-llama.md) |
| runtime-process | 2.4/5 | scaffold | CTRL_BREAK stub; no Job Object | [runtime-process.md](./runtime-process.md) |
| system | 1.6/5 | scaffold | GPU/memory probes are empty stubs | [system.md](./system.md) |
| api | 2.6/5 | mvp-partial | URL helpers only; no tests | [api.md](./api.md) |
| cli | 2.6/5 | mvp-partial | `stop`/`restart` not implemented | [cli.md](./cli.md) |
| bin | 1.4/5 | stub | No `llama-server.exe` committed | [bin.md](./bin.md) |
| installer | 1.4/5 | stub | README only; no Inno scripts | [installer.md](./installer.md) |
| scripts-ops | 2.8/5 | mvp-partial | Force-kill only (not graceful) | [scripts-ops.md](./scripts-ops.md) |
| scripts-pr-loop | 3.0/5 | mvp-partial | Dry-run only; no formal tests | [scripts-pr-loop.md](./scripts-pr-loop.md) |
| docs | 3.6/5 | mvp-ready | No automated doc/link checks | [docs.md](./docs.md) |
| ui | 0.4/5 | stub | Path missing; Phase 2 placeholder | [ui.md](./ui.md) |

## Summary

- **Overall project maturity:** **2.5/5** (`mvp-partial`) — average of 14 modules, excluding `ui` (Phase 2 only).
- **Phase alignment:** Phase 1 engine is partially in place (`ServerManager` start → readiness → stop happy path). Still needed for Phase 1: real Windows graceful stop (CTRL_BREAK / Job Object), non-stub hardware detection, automated tests, and a long-running manager so CLI `stop`/`restart` work. Phase 2 UI and Phase 3 installer remain stubs.
