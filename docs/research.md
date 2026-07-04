# Research Checklist (Phase 0)

## Installer

* [ ] NSIS
* [ ] Inno Setup
* [ ] WiX
* [ ] MSIX

Goal: desktop shortcut, firewall rules, start menu, auto updates (future).

## Hardware Detection

* [ ] NVML
* [ ] CUDA Runtime
* [ ] DirectX APIs
* [ ] WMI

Need: GPU, VRAM, CUDA capability, CPU, RAM.

## Backend Management

* [ ] Process spawning
* [ ] stdout capture
* [ ] stderr capture
* [ ] Graceful shutdown
* [ ] Crash detection
* [ ] Restart policy

## Configuration

* [x] YAML (chosen for v1)
* [ ] Migration strategy
* [ ] Schema validation (stub in `packages/config`)

## Logging

* [ ] Rotating logs
* [x] Timestamps (via tracing)
* [x] Backend output forwarding (stub)
* [x] Application logs (stub)

## Networking

* [ ] Firewall automation
* [ ] LAN binding
* [ ] Localhost mode
* [ ] Port conflicts

## llama.cpp Integration

* [ ] Release cadence
* [ ] CLI changes
* [ ] Metrics endpoint
* [ ] OpenAI compatibility
* [ ] Startup detection

## UI

* [ ] Native Windows feel
* [ ] Dark mode
* [ ] Tray support
* [ ] Accessibility
