# Module readiness

Last reviewed: 2026-07-31

| Module | Overall | Label | Top gap | Scorecard |
| --- | --- | --- | --- | --- |
| server-manager | 3.4/5 | mvp-partial | Operator Windows proof for orphan-free quit | [server-manager.md](./server-manager.md) |
| server-config | 3.6/5 | mvp-ready | No config migration / version field | [server-config.md](./server-config.md) |
| server-health | 3.8/5 | mvp-ready | Timeout does not surface last HTTP status | [server-health.md](./server-health.md) |
| server-logs | 3.8/5 | mvp-ready | Write failures swallowed; epoch timestamps only | [server-logs.md](./server-logs.md) |
| runtime-llama | 3.6/5 | mvp-ready | No binary version probe; A2 smoke open | [runtime-llama.md](./runtime-llama.md) |
| runtime-process | 3.8/5 | mvp-ready | No orphan-after-manager-kill integration test | [runtime-process.md](./runtime-process.md) |
| system | 3.8/5 | mvp-ready | NVML deferred; DXGI non-empty list is env gate | [system.md](./system.md) |
| api | 3.2/5 | mvp-partial | Helpers underused vs Config URL methods | [api.md](./api.md) |
| cli | 3.6/5 | mvp-partial | No live serve+attach automated smoke | [cli.md](./cli.md) |
| bin | 3.2/5 | mvp-partial | Fetch manual; no checksum / CI pin download | [bin.md](./bin.md) |
| installer | 3.4/5 | mvp-partial | Install→Ready operator proof open (ISCC compiles in CI) | [installer.md](./installer.md) |
| scripts-ops | 3.8/5 | mvp-partial | Full A2/orphan/tray-quit still operator+GGUF | [scripts-ops.md](./scripts-ops.md) |
| scripts-pr-loop | 3.0/5 | mvp-partial | Dry-run only; no formal tests | [scripts-pr-loop.md](./scripts-pr-loop.md) |
| docs | 4.0/5 | mvp-ready | No automated link checks beyond drift | [docs.md](./docs.md) |
| ui | 3.6/5 | mvp-partial | Windows quit proof open (tray icon/menu present) | [ui.md](./ui.md) |

## Summary

- **Overall project maturity:** **3.6/5** (`mvp-partial`) — average of 14 modules, excluding `ui`.
- **Phase alignment:** G/H resident IPC, E tray, F installer/first-run, and I1–I4 polish are merged on `dev`. Remaining gates: A2 Windows+GGUF smoke, install→Ready operator proof. Scorecards refreshed 2026-07-31 (coverage depth). See [audit/completion-audit.md](../audit/completion-audit.md).
