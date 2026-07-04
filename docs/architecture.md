# Architecture

## Layering

```text
Desktop UI / Tray / CLI / Management API
              │
              ▼
        Server Manager   (packages/launcher)
              │
   ┌──────────┼──────────┬──────────────┐
   ▼          ▼          ▼              ▼
 Config   Hardware   Logging        Backend
 (config) (hardware) (logging)   (backend trait)
                                        │
                                        ▼
                                 LlamaBackend
                                 (packages/llama)
                                        │
                                        ▼
                                  llama-server.exe
                                  (vendor/llama.cpp)
```

The UI knows almost nothing. It edits config, displays logs and state, and calls start/stop on the Server Manager.

## Package Responsibilities

| Package | Responsibility |
| --- | --- |
| `launcher` | **Server Manager** — single owner of inference lifecycle |
| `process` | Process lifecycle management |
| `llama` | Interface to llama-server |
| `hardware` | GPU/RAM/CPU detection |
| `config` | Reads/writes config |
| `api` | OpenAI compatibility utilities |
| `logging` | Unified logs |
| `backend` | Backend abstraction layer |
| `shared` | Shared types/utilities |

## Backend Interface

Never let the UI know about llama.cpp.

```text
Backend
  Initialize()
  LoadModel()
  Start()
  Stop()
  Status()
  Metrics()
  Health()
  Version()
```

Eventually drop-in replacements:

* `LlamaBackend` (v1)
* `OllamaBackend`
* `vLLMBackend`
* `TensorRTBackend`

## Responsibility Chart

| Component | Owns |
| --- | --- |
| Desktop | User interaction |
| Config | Persistent settings |
| Launcher (Server Manager) | Starts/stops backend |
| Process Manager | Child process lifecycle |
| Backend | Translates config to backend |
| Logger | Unified logging |
| Hardware | System detection |
| Installer | Initial setup |
| Updater | Version management |

## Design Rules

1. **Server Manager is the only process owner.** Tray, CLI, and future management APIs talk to it — never spawn `llama-server` themselves.
2. **No raw llama.cpp flags in config or UI.** High-level knobs only (`auto` / explicit values).
3. **Backend-agnostic types live in `shared` and `backend`.** Concrete adapters live in their own packages.
4. **Stable over bleeding edge.** Bundle a vetted llama.cpp release per stable version.
