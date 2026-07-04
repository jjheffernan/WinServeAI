# 0001. Appliance architecture

- **Status:** Accepted
- **Date:** 2026-07-04

## Context

WinServeAI could grow into a multi-backend platform (traits, plugins, many runtimes) or stay a thin Windows wrapper around one external binary. Platform shape increases surface area and invites chat UI, model marketplaces, and backend-specific config. Product intent is a **Windows-native llama.cpp appliance** — see [vision.md](../vision.md) and [architecture.md](../architecture.md).

## Decision

Ship a single-crate appliance:

1. **One orchestrator:** `ServerManager` (`app/src/server/`) is the only process-control API. UI/CLI call it; nothing else owns `llama-server.exe`.
2. **One backend:** llama.cpp only. No backend trait, no plugins, no multi-provider support.
3. **External boundary:** `bin/llama-server.exe` is an external system. Raw flags and argv live only in `app/src/runtime/llama.rs`.
4. **Config:** human-readable YAML is the source of truth; no raw backend flags in config.
5. **Logs:** unified under `logs/`. Readiness via OpenAI-compatible probes (`/v1/models` / health as documented).

Abstractions for many backends come later **only if needed**, and only by superseding this ADR and updating `architecture.md`.

## Consequences

* Agents and PRs that introduce backend traits, plugin loaders, or UI-owned process spawn are out of scope.
* MVP exclusions (chat UI, model downloads, Docker, auth, metrics dashboards, remote management) stay deferred; they are not reopened by this ADR.
* Pin policy, Windows stop semantics, and installer choice are separate ADRs (see [README.md](README.md) planned list).
