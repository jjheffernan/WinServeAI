# Vision

> **A lightweight, Windows-native llama.cpp appliance** that turns a GPU PC into an OpenAI-compatible inference endpoint with one click.

## Product category

**Windows-native llama.cpp appliance wrapper** — not a mini LM Studio, not a multi-backend platform.

## Core principles

* Windows-first, easy, GUI-first (tray). Same install → pick a local model → localhost API loop as Strata. Chat is an addon client of `/v1`, not part of the server
* Tinkerable through the YAML fields the GUI already edits (host, port, GPU fit/layers, context, flash attention, API key). No freeform llama flag box
* API-first (OpenAI-compatible via llama-server)
* Headless inference
* No chat inside the server. Optional frontends live in `apps/chat` and authenticate with `server.api_key`
* Zero model management ecosystem
* Stable over bleeding edge
* Native installer
* Predictable YAML configuration
* Small surface area — one backend, one orchestrator

## MVP included

* Native Windows installer
* Hardware detection (minimal)
* llama.cpp only
* Start / stop server
* Model path in config
* Settings persistence
* Logging
* Auto GPU defaults
* Health / readiness (`/v1/models`)

## Explicitly excluded

* Chat UI
* Model downloads / HuggingFace
* Agent frameworks, RAG, MCP
* Docker
* Authentication
* Dashboard metrics
* Remote management
* Multi-backend / plugins

## See also

- [architecture.md](./architecture.md) — runtime boundary and rules
- [build-spec.md](./build-spec.md) — agent drop-in constraints
- [roadmap.md](./roadmap.md) — phased plan
- [PLAN.md](./PLAN.md) — ordered implementation milestones
- [adr/0001-appliance-architecture.md](./adr/0001-appliance-architecture.md) — accepted decision
- [policies/doc-drift.md](./policies/doc-drift.md) — keep claims aligned with code
- [policies/SOURCES.md](./policies/SOURCES.md) — canonical external URLs
