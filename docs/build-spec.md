# Agent build spec

Drop-in prompt for coding agents.

---

Refactor WinServeAI into a minimal Windows-native local inference server manager.

### Goal

Build a single-process manager for llama.cpp that exposes an OpenAI-compatible API on localhost or LAN.

### Hard constraints

* Only ONE backend: llama.cpp
* No backend abstraction layers
* No plugin system
* No Docker
* No model download system
* No multi-provider support

### Required modules

1. ServerManager (core orchestrator)
   * start()
   * stop()
   * status()
   * restart()
2. Runtime layer
   * spawns llama-server.exe
   * captures stdout/stderr
   * handles shutdown gracefully
3. Config system
   * YAML-based config
   * maps directly to runtime build command
4. OpenAI API passthrough layer
   * ensures /v1/chat/completions compatibility (via llama-server)
5. Hardware detection (minimal)
   * GPU name
   * VRAM
   * CPU threads

### Output requirement

* Single executable or installer target
* Desktop shortcut launches server manager
* API must be reachable at: http://localhost:8080/v1

### Non-goals

* UI dashboards
* model marketplaces
* agent frameworks
* multi-backend support

### Success condition

User can:

1. install app
2. double click shortcut
3. server runs
4. OpenAI client connects
5. stop app to free GPU
