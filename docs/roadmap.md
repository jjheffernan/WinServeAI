# Roadmap

WinServeAI is a **Windows-native llama.cpp appliance**: install → start ServerManager → OpenAI-compatible API at `/v1` → stop frees the GPU.

Phases track that path. There is no near-term multi-backend or plugin track. See [architecture.md](architecture.md) and [vision.md](vision.md).

## Phased Development

### Phase 0 — Research

* Installer options (Inno Setup preferred; NSIS, WiX, MSIX noted)
* Architecture freeze (appliance wrapper, ServerManager only)
* llama.cpp / `llama-server` integration and readiness
* Licensing
* Update strategy (defer implementation)

**Deliverable:** Architecture frozen. Deep dives: [research/01–05](research/).

### Phase 1 — ServerManager engine

* Process ownership (spawn, monitor, graceful stop, force kill)
* YAML config load and argv build (`runtime/llama` only for flags)
* Unified logging (`logs/`)
* Readiness (`/v1/models` or `/health` → Ready; process alive ≠ Ready)
* Minimal hardware detection for safe defaults

**Deliverable:** Launch `llama-server` from code, detect Ready, shut down cleanly — all through `ServerManager`.

### Phase 2 — Desktop

* Start / stop / status via ServerManager only
* Log viewer
* Settings (YAML-backed)
* Model path picker (local path; no downloads)

**Deliverable:** No CLI required.

### Phase 3 — Installer

* Packaged app + bundled `llama-server`
* Desktop / Start Menu shortcut
* Firewall rules (when LAN bind is enabled)
* First-run wizard (model path, basic settings)

**Deliverable:** Install → Start → API available.

### Phase 4 — Stability

* Crash recovery (clear Failed / Crashed handling)
* Better logging and diagnostics
* Config migration
* Extensive testing on representative hardware

**Deliverable:** Production-ready appliance.

### Phase 5 — Future (optional)

* Metrics dashboard
* Tray polish / always-on tray
* Auto-updater
* Remote management / service mode

Alternate backends or a plugin system are **not** planned. Revisit only if a real second backend is required.

## Version Milestones

### v0.1 — Engine

* `ServerManager` launches `llama-server`
* Detect readiness
* Graceful shutdown

### v0.2 — Configuration

* Persist YAML settings
* Auto-detect hardware (minimal)
* Build launch command from config

### v0.3 — Desktop

* Native UI
* Start / stop
* Logs and status
* Model path selection

### v0.4 — Installer

* One-click installation
* Desktop shortcut
* Firewall configuration
* Bundled `llama-server`

### v1.0 — Stable appliance

* Stable OpenAI-compatible endpoint (`/v1`)
* Reliable process lifecycle
* Clear logging
* Hardware-aware defaults
* No CLI required

## See also

- [PLAN.md](./PLAN.md) — actionable milestones (code path)
- [TODO.md](./TODO.md) — priority backlog from readiness scores
- [vision.md](./vision.md) — MVP in/out
- [architecture.md](./architecture.md) — appliance rules
- [research.md](./research.md) — Phase 0 checklist
- [readiness/README.md](./readiness/README.md) — current maturity
- [policies/doc-drift.md](./policies/doc-drift.md) — keep scores and banners aligned
- [policies/SOURCES.md](./policies/SOURCES.md) — canonical external URLs
