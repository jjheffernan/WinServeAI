# Vision

> **A lightweight, Windows-native llama.cpp appliance** that turns a GPU PC into an OpenAI-compatible inference endpoint with one click.

## Product category

**Windows-native llama.cpp appliance wrapper** — not a mini LM Studio, not a multi-backend platform.

## Core principles

* Windows-first
* API-first (OpenAI-compatible via llama-server)
* Headless inference
* Zero chat UI
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
