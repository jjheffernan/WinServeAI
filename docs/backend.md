# Backend

## Abstraction

All inference engines implement `winserve_backend::Backend`:

* `initialize`
* `load_model`
* `start`
* `stop`
* `status`
* `metrics`
* `health`
* `version`

## Current: llama.cpp

`packages/llama` adapts high-level config into `llama-server` process arguments.

Bundled under `vendor/llama.cpp/`. Each stable release pins a vetted llama.cpp build.

## Future Backends

Drop-in replacements:

* `OllamaBackend`
* `vLLMBackend`
* `TensorRTBackend`

Plugins (Phase 5) load additional backends without changing the Server Manager or UI.

## Process Management Research

* Process spawning
* stdout / stderr capture
* Graceful shutdown
* Crash detection
* Restart policy

Owned by `packages/process`; backends compose it rather than reimplementing it.
