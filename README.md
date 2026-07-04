# WinServeAI

> A lightweight, Windows-native local AI server that turns any GPU-equipped PC into an OpenAI-compatible inference endpoint with one click.

**Windows-first · API-first · Headless · Backend-agnostic · Small surface area**

## Status

Phase 0 scaffold — architecture and monorepo layout are in place. Core engine implementation begins in Phase 1.

## Architecture

```text
Desktop UI
    │
    ▼
Server Manager   (packages/launcher)
    │
    ├── Configuration
    ├── Hardware Detection
    ├── Process Lifecycle
    ├── Health Monitoring
    ├── Logging
    └── Backend Adapter (llama.cpp)
              │
              ▼
        llama-server.exe
```

The UI only edits config, shows logs/state, and starts/stops the server. Everything else is owned by the Server Manager.

## Monorepo

```text
apps/
  desktop/       # Thin native UI (Phase 2)
  installer/     # Native Windows installer (Phase 3)
  updater/       # Auto-update (Phase 5)
packages/
  launcher/      # Server Manager
  backend/       # Backend trait
  llama/         # llama.cpp adapter
  process/       # Child process lifecycle
  config/        # YAML config
  hardware/      # GPU/CPU/RAM detection
  logging/       # Unified logs
  api/           # OpenAI compatibility helpers
  shared/        # Shared types
docs/            # Product & contributor docs
examples/        # Sample configs
vendor/llama.cpp # Bundled backend (pinned per release)
.agents/skills/  # Agent-agnostic skills
```

## Quick start (developers)

```bash
# Prerequisites: Rust 1.78+
cargo check --workspace
cargo test --workspace
```

Sample config: [`examples/config/default.yaml`](examples/config/default.yaml)

## Documentation

| Doc | Topic |
| --- | --- |
| [vision.md](docs/vision.md) | Product principles & MVP scope |
| [architecture.md](docs/architecture.md) | Layers, packages, backend interface |
| [roadmap.md](docs/roadmap.md) | Phases and version milestones |
| [development.md](docs/development.md) | Local development |
| [configuration.md](docs/configuration.md) | Config schema |
| [backend.md](docs/backend.md) | Backend abstraction |
| [contributing.md](docs/contributing.md) | Contributor guide |
| [prior-art.md](docs/prior-art.md) | Similar apps, patterns, scaffolding recs |

## Agent skills

Skills are **agent-agnostic** and live in [`.agents/skills/`](.agents/skills/).

| Invoke | Purpose |
| --- | --- |
| `/grill-me` | Stress-test a plan before building |
| `/ponytail` | YAGNI / minimal correct solution |
| `/caveman` | Compressed communication |

```bash
npm run skills:list
npm run skills:update
npm run skills:restore
```

See [`AGENTS.md`](AGENTS.md).

## License

MIT — see [LICENSE](LICENSE).
