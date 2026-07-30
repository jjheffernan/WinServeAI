# Module readiness

Last reviewed: 2026-07-04

| Module | Overall | Label | Top gap | Scorecard |
| --- | --- | --- | --- | --- |
| server-manager | 3.4/5 | mvp-partial | Long-running manager for CLI stop/restart | [server-manager.md](./server-manager.md) |
| server-config | 3.6/5 | mvp-ready | No migration; model path only warns at load | [server-config.md](./server-config.md) |
| server-health | 3.8/5 | mvp-ready | Timeout does not surface last HTTP status | [server-health.md](./server-health.md) |
| server-logs | 3.6/5 | mvp-ready | Write failures swallowed; epoch timestamps only | [server-logs.md](./server-logs.md) |
| runtime-llama | 3.6/5 | mvp-ready | No binary version probe; A2 smoke open | [runtime-llama.md](./runtime-llama.md) |
| runtime-process | 3.8/5 | mvp-ready | Assign-after-spawn race; no orphan integration test | [runtime-process.md](./runtime-process.md) |
| system | 3.6/5 | mvp-ready | NVML FFI deferred; DXGI not asserted in CI | [system.md](./system.md) |
| api | 2.6/5 | mvp-partial | URL helpers only; no tests | [api.md](./api.md) |
| cli | 2.6/5 | mvp-partial | `stop`/`restart` not implemented | [cli.md](./cli.md) |
| bin | 2.2/5 | scaffold | No `llama-server.exe` committed; operator supplies **b9866** | [bin.md](./bin.md) |
| installer | 1.6/5 | scaffold | Spec only; no Inno scripts | [installer.md](./installer.md) |
| scripts-ops | 3.6/5 | mvp-ready | start.ps1 unclear if binary missing; A2 needs operator Windows proof | [scripts-ops.md](./scripts-ops.md) |
| scripts-pr-loop | 3.0/5 | mvp-partial | Dry-run only; no formal tests | [scripts-pr-loop.md](./scripts-pr-loop.md) |
| docs | 4.0/5 | mvp-ready | No automated link checks beyond drift | [docs.md](./docs.md) |
| ui | 2.2/5 | scaffold | No logs/settings/quit→stop yet | [ui.md](./ui.md) |

## Summary

- **Overall project maturity:** **3.2/5** (`mvp-partial`) — average of 14 modules, excluding `ui` (Phase 2 only).
- **Phase alignment:** Phase 1 engine code is in place (`ServerManager` start → readiness → stop; Job Object + CTRL_BREAK; DXGI/sysinfo; unit tests). Operator smoke scripts/spec ready (**A2** still open until Windows + GGUF proof). Phase 2/3 have build specs (`E-desktop`, `F-installer`); UI and Inno implementation still open.
