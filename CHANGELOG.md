# Changelog

All notable changes to WinServeAI are recorded here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versioning follows [SemVer](https://semver.org/). Stable tags are `vX.Y.Z` on `main`; preview tags are `vX.Y.Z-rc.N` on `dev`.

Each release also records the pinned llama.cpp build (`bin/VERSION`, e.g. `b9866`).

## [Unreleased]

### Fixed

- Config saves are atomic (write temp + replace) so a power cut mid-save cannot leave a half YAML.
- Wildcard bind (`0.0.0.0` / `::`) no longer appears in client / readiness URLs; Copy connect shows loopback and keeps the operator-owned bind host unchanged across updates.

### Changed

- Document host/IP as operator-owned: installer never rewrites `config/default.yaml` on upgrade; we never auto-pick a LAN or WSL interface IP.
- Smoke and docs cover the four common completion routes (chat, Responses, Anthropic Messages, legacy completions).
- GitHub releases publish notes from this file (plus the llama.cpp pin and setup hash).

### Added

- Runtime resilience notes for restarts, power loss, and upgrades (`docs/ops-resilience.md`).
- Full operator CLI: `version`, `doctor`, `debug`, `config` / `config set --apply`, `update`, plus `reload` IPC — fix installs at runtime without reinstall.
- Installer adds `{app}` to user PATH (checked-once task) and App Paths for `winserve.exe`.

## [0.1.0] - TBD

First packaged MVP (tray + manager + pinned llama-server).

### Added

- ServerManager lifecycle around `bin/llama-server.exe` (Job Object, CTRL_BREAK stop, readiness via `/v1/models`).
- YAML config (`config/default.yaml`) as source of truth; default bind `127.0.0.1:8080`.
- Tray: overview, sticky last error, Copy connect, Copy launch cmd.
- Inno Setup installer; `config\default.yaml` uses `onlyifdoesntexist` so upgrades keep operator edits.
- Optional LAN firewall task (default off).
