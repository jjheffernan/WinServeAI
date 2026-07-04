# Roadmap

## Phased Development

### Phase 0 — Research

* Installer options (Inno Setup, NSIS, WiX, MSIX)
* Architecture freeze
* llama.cpp integration
* Licensing
* Update strategy

**Deliverable:** Architecture frozen.

### Phase 1 — Core Engine

* Backend abstraction
* Process manager
* Config
* Logging

**Deliverable:** Can launch llama-server from code.

### Phase 2 — Desktop

* Start / stop
* Logs
* Settings
* Model picker

**Deliverable:** No CLI required.

### Phase 3 — Installer

* Packaged executable
* Desktop shortcut
* Firewall
* First-run wizard

**Deliverable:** Install → Start → API available.

### Phase 4 — Stability

* Crash recovery
* Better logging
* Config migration
* Extensive testing

**Deliverable:** Production ready.

### Phase 5 — Future

* Metrics dashboard
* Tray app
* Backend plugins
* Updater
* Remote dashboard
* Agent discovery

## Version Milestones

### v0.1 — Engine

* Launch `llama-server`
* Detect readiness
* Graceful shutdown

### v0.2 — Configuration

* Persist settings
* Auto-detect hardware
* Build launch command from config

### v0.3 — Desktop

* Native UI
* Start/Stop
* Logs
* Status
* Model selection

### v0.4 — Installer

* One-click installation
* Desktop shortcut
* Firewall configuration
* Bundled backend

### v1.0 — Stable

* Stable OpenAI-compatible endpoint
* Reliable process lifecycle
* Clear logging
* Hardware auto-configuration
* No CLI required
