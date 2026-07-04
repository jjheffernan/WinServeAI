# Research Checklist (Phase 0)

Prior art and external references: [prior-art.md](prior-art.md).

Deep dives: [research/01-windows-process.md](research/01-windows-process.md), [research/02-llama-readiness.md](research/02-llama-readiness.md), [research/03-hardware-detection.md](research/03-hardware-detection.md), [research/04-installer-licensing.md](research/04-installer-licensing.md), [research/05-server-manager.md](research/05-server-manager.md).

## Installer

* [x] NSIS
* [x] Inno Setup
* [x] WiX
* [x] MSIX

Goal: desktop shortcut, firewall rules, start menu, auto updates (future). Preferred starting point: **Inno Setup** ([installer.md](installer.md), [research/04](research/04-installer-licensing.md)).

## Hardware Detection

* [x] NVML
* [x] CUDA Runtime (driver API / `nvcuda.dll`; not full toolkit)
* [x] DirectX APIs (DXGI)
* [x] WMI (evaluated; prefer `sysinfo` for v1)

Need: GPU, VRAM, CUDA capability, CPU, RAM. See [research/03](research/03-hardware-detection.md).

## Process management (runtime)

* [x] Process spawning
* [x] stdout capture
* [x] stderr capture
* [x] Graceful shutdown
* [x] Crash detection
* [x] Restart policy

See [research/01](research/01-windows-process.md), [research/05](research/05-server-manager.md).

## Configuration

* [x] YAML (chosen for v1)
* [ ] Migration strategy
* [x] Schema validation (serde structs in `app/src/server/config.rs`; research in doc-build)

## Logging

* [ ] Rotating logs
* [x] Timestamps (via tracing)
* [x] Backend output forwarding (stub)
* [x] Application logs (stub)

## Networking

* [x] Firewall automation
* [x] LAN binding
* [x] Localhost mode
* [x] Port conflicts

## llama.cpp Integration

* [x] Release cadence (pin policy)
* [x] CLI changes / flag drift
* [ ] Metrics endpoint
* [x] OpenAI compatibility
* [x] Startup detection (readiness)

## UI

* [ ] Native Windows feel
* [ ] Dark mode
* [ ] Tray support
* [ ] Accessibility
