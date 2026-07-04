# Research: `docs/research.md` rewrite

**Target:** Phase 0 checklist aligned with appliance architecture.  
**Sources:** [architecture.md](../../architecture.md), [vision.md](../../vision.md), [prior-art.md](../../prior-art.md), [research/01–05](../).

## Problem with current checklist

| Issue | Detail |
| --- | --- |
| Stale layout | Mentions `packages/config`; monorepo is single `app/` crate |
| Platform framing | “Backend Management” implies multi-backend; product is llama.cpp appliance only |
| Unchecked research | Deep dives in `research/01–05` and `prior-art.md` already answer most Phase 0 questions |
| No index | Checklist does not link the notes that close each item |

## Appliance framing (lock)

```text
App/CLI → ServerManager → runtime (llama + process) → bin/llama-server.exe → /v1
```

Config lives at `config/default.yaml`; load/validate in `app/src/server/`. No `packages/*` workspace.

## Coverage map (mark done when note exists)

| Checklist area | Covered by | Status |
| --- | --- | --- |
| Installer (Inno preferred; NSIS/WiX/MSIX deferred) | [04](../04-installer-licensing.md), [prior-art](../../prior-art.md) | **done** |
| Hardware (DXGI, NVML, CUDA driver, sysinfo; WMI deferred) | [03](../03-hardware-detection.md), prior-art | **done** |
| Process lifecycle (spawn, stdio, Job Object, stop, crash, restart policy) | [01](../01-windows-process.md), prior-art | **done** |
| Server Manager state machine | [05](../05-server-manager.md), prior-art | **done** |
| Config (YAML, field→argv, no raw flags) | [02](../02-llama-readiness.md), architecture, configuration.md | **done** (YAML + mapping) |
| Config migration | — | **open** (Phase 4 stability; no note yet) |
| Schema validation | — | **open** (implement in `app/src/server/` when coding; no dedicated note) |
| Logging (streams, timestamps, forwarding) | architecture, logging.md, 01/05 | **done** for MVP shape |
| Log rotation | prior-art (NSSM/d4 pattern only) | **open** (defer; not MVP) |
| Networking (localhost default, LAN firewall, port conflicts) | 02, 04, prior-art | **done** |
| llama-server (pin, CLI/`--fit`, `/health`, `/v1`, readiness) | [02](../02-llama-readiness.md), prior-art | **done** |
| Metrics endpoint | — | **open** / out of MVP (Phase 5+) |
| UI (tray ownership boundary) | 05, prior-art P1 desktop | **done** for process ownership |
| UI polish (native feel, dark mode, a11y) | — | **open** (Phase 2 design, not Phase 0 blockers) |

## Rewrite rules

1. Lead with links to [prior-art.md](../../prior-art.md) and `research/01`–`05` (index, not duplication).
2. Rename “Backend Management” → **Process & Server Manager** (appliance language).
3. Point config items at `config/default.yaml` / `app/src/server/` — never `packages/config`.
4. Checkbox `[x]` = Phase 0 research closed by a note; implementation still later.
5. Leave migration, schema validation, log rotation, metrics, UI polish unchecked with one-line “where it lands.”
6. Goals under installer stay: shortcuts, Start Menu, firewall (LAN only), auto-update deferred.

## Deliverable shape for `research.md`

```text
# Research Checklist (Phase 0)
Intro + links
## Notes index (01–05 table)
## Installer
## Hardware detection
## Process & Server Manager
## Configuration
## Logging
## Networking
## llama-server integration
## UI (Phase 2)
## Still open
```

No multi-backend, plugins, or package-workspace language.
