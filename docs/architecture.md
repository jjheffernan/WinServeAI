# Architecture

> **Module readiness dashboard:** [readiness/README.md](./readiness/README.md)

WinServeAI is a **Windows-native llama.cpp appliance wrapper**, not a multi-backend platform.

```text
Windows App / CLI
       │
       ▼
  ServerManager     (app/server)  ← only orchestration API
       │
       ▼
  runtime/llama     builds argv (ONLY place with llama flags)
  runtime/process   spawn / monitor / kill
       │
       ▼
  bin/llama-server.exe   ← external system
       │
       ▼
  OpenAI API  http://host:port/v1
```

## Explicit runtime boundary

| Our system | External system |
| --- | --- |
| config (YAML) | `llama-server.exe` |
| process control | model GGUF files |
| logs | CUDA/runtime DLLs beside the binary |
| readiness (`/v1/models`) | |

## ServerManager API

```text
load_config()
detect_hardware()
build_command()
start()
stop()
restart()
status() / get_status()
```

Startup: read config → detect GPU → build command → spawn → wait `/v1/models` → READY.

Shutdown: signal → wait 5–10s → force kill → flush logs.

## Layout

```text
app/           # single Rust crate
  src/server/  # manager, config, health, logs
  src/runtime/ # llama + process (external boundary)
  src/api/     # OpenAI URL helpers (passthrough)
  src/system/  # gpu, memory, network
  ui/          # optional later (empty in MVP)
bin/           # llama-server.exe
config/        # default.yaml
logs/          # server.log, llama.log, error.log
models/        # user models (not shipped)
installer/
scripts/
docs/
```

## Rules (do not mess up)

1. **One backend:** llama.cpp only. No backend trait, no plugins.
2. **One orchestrator:** `ServerManager`. UI/CLI only call it.
3. **Raw flags only in `runtime/llama.rs`.**
4. **Config is YAML source of truth.**
5. **Unified logs** under `logs/`.
6. **No model downloads, chat UI, Docker, or multi-provider support.**

Abstractions for “many backends” come later *only if needed*.
