# Research: `roadmap.md`

**Target:** rewrite `docs/roadmap.md`  
**Canonical lock:** [architecture.md](../../architecture.md), [vision.md](../../vision.md), [build-spec.md](../../build-spec.md)  
**Plan row:** [PLAN.md](PLAN.md) — still says backend abstraction / plugins

---

## Problem with current roadmap

| Location | Issue |
| --- | --- |
| Phase 1 — “Backend abstraction” | Contradicts architecture: **no** backend trait, no plugins, llama.cpp only |
| Phase 5 — “Backend plugins” | Reads as a planned product track; should be at most a distant maybe |
| Phase 1 bullets | Process / config / logging listed, but **readiness** missing (core ServerManager job) |
| Framing | Sounds like a multi-backend platform roadmap, not a **llama.cpp appliance** |
| Version milestones | Mostly fine, but “Bundled backend” and engine wording should say **llama-server** / appliance |

Phase 0–4 deliverables (research → engine → desktop → installer → stability) are still the right spine. Only the **engine definition** and **future language** need to change.

---

## Product alignment (appliance)

Success path from build-spec / vision:

1. Install
2. Start manager
3. OpenAI-compatible API at `http://host:port/v1`
4. Stop frees GPU

Roadmap phases should track that path, not “platform extensibility.”

| Phase | Appliance meaning |
| --- | --- |
| **0** | Freeze architecture, installer/licensing/llama research |
| **1** | **ServerManager engine** — process, config, logs, readiness |
| **2** | Desktop/tray calls ServerManager only (no CLI required) |
| **3** | One-click install → start → API available |
| **4** | Crash recovery, migration, production hardening |
| **5** | Optional extras (metrics, tray polish, updater, remote) — **not** multi-backend |

Deep dives already exist; link, don’t duplicate:

- Process: [research/01-windows-process.md](../01-windows-process.md)
- Readiness: [research/02-llama-readiness.md](../02-llama-readiness.md)
- Hardware: [research/03-hardware-detection.md](../03-hardware-detection.md)
- Installer: [research/04-installer-licensing.md](../04-installer-licensing.md)
- ServerManager states: [research/05-server-manager.md](../05-server-manager.md)

---

## Phase 1 scope (authoritative)

**Name:** ServerManager engine (not “Core Engine” with abstraction).

| Concern | In Phase 1 | Notes |
| --- | --- | --- |
| Process spawn / monitor / kill | Yes | `runtime/process`; Job Object on Windows |
| YAML config load → argv | Yes | Flags only in `runtime/llama.rs` |
| Unified logs | Yes | `logs/server.log`, `llama.log`, `error.log` |
| Readiness | Yes | Wait until `/v1/models` (or `/health`) → Ready; process alive ≠ Ready |
| Hardware detect (minimal) | Yes (for defaults) | GPU/VRAM/threads → safe launch defaults |
| Backend trait / plugins | **No** | Explicitly out |
| Desktop UI | No | Phase 2 |
| Installer | No | Phase 3 |

**Deliverable:** Launch `llama-server` from code, detect Ready, graceful shutdown — via `ServerManager` only.

---

## Phase 5 language

Allowed as **future maybe** (one line, not a commitment):

- Metrics dashboard, tray polish, auto-updater, remote management, Windows Service mode
- Alternate backends / plugins **only if** a real second backend appears — not a near-term milestone

Do **not** list “Backend plugins” as a peer of updater/tray.

---

## Version milestones (keep, retarget)

| Version | Align to |
| --- | --- |
| v0.1 Engine | ServerManager: launch, readiness, shutdown |
| v0.2 Configuration | Persist YAML, hardware defaults, build command from config |
| v0.3 Desktop | Native UI → ServerManager (start/stop/logs/status/model path) |
| v0.4 Installer | One-click, shortcut, firewall, **bundled llama-server** |
| v1.0 Stable | Appliance: stable `/v1`, reliable lifecycle, clear logs, no CLI |

Drop any implication that v0.1 builds a pluggable backend layer.

---

## Out of roadmap (all phases unless Phase 5+ explicit)

Chat UI, model downloads / HuggingFace, agents, RAG, MCP, Docker, auth, multi-backend platform.

---

## Rewrite checklist

- [x] Remove “backend abstraction” from Phase 1
- [x] Phase 1 = ServerManager (process, config, logs, readiness)
- [x] Phase 5: plugins only as distant maybe, not a bullet commitment
- [x] Appliance wording throughout
- [x] Link architecture / research notes lightly
- [x] Keep Phase 0–4 deliverables coherent with install → API path
