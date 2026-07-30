# LocalAI — architecture research

**Upstream:** https://github.com/mudler/LocalAI  
**Pinned:** `v4.7.1` / `b224c96db6f4b87306a33a808650bfce63b12588`  
**Verified:** 2026-07-30  
**License (source tree):** MIT (`LICENSE`)

## Summary

LocalAI is a Go HTTP control plane that exposes OpenAI-compatible and additional APIs, resolves per-model YAML configuration, and talks over gRPC to separately running inference backends. Its runtime is explicitly multi-backend: model configuration names a backend, aliases normalize backend names, and the repository contains many C++, Go, Python, and Rust backend packages. For local inference, LocalAI starts and stops backend child processes; it can instead attach to external gRPC addresses or route work to distributed workers. A separate Fyne desktop/tray launcher downloads and starts the `local-ai` server binary, while container and native-binary packaging are also present. **Confidence: verified in source.**

## Claims

| Claim | Evidence (path @ rev) | Confidence |
| --- | --- | --- |
| The HTTP server delegates model execution to backend processes over gRPC. | `pkg/model/initializers.go`, `pkg/model/process.go`, `pkg/grpc/client.go` @ `b224c96d` | verified in source |
| Local backends are child processes owned by `ModelLoader`; external backends may instead be remote gRPC addresses with no local process handle. | `pkg/model/initializers.go`, `pkg/model/process.go`, `pkg/model/model.go` @ `b224c96d` | verified in source |
| The runtime is multi-backend, including llama.cpp, IK llama.cpp, Transformers, Whisper, Stable Diffusion and numerous packaged C++/Go/Python/Rust backends. | `pkg/model/initializers.go`, `backend/cpp/`, `backend/go/`, `backend/python/`, `backend/rust/` @ `b224c96d` | verified in source |
| Models and backend configuration are selected through model YAML, CLI/environment paths, and external-backend maps. | `core/config/model_config.go`, `core/config/model_config_loader.go`, `core/cli/run.go` @ `b224c96d` | verified in source |
| Model and backend artifacts can be downloaded; supported model URI forms include Hugging Face, OCI, Ollama, GitHub, HTTP(S), and local files. | `pkg/downloader/uri.go`, `core/services/galleryop/`, `core/gallery/backends.go` @ `b224c96d` | verified in source |
| The server registers OpenAI-compatible chat, completion, embedding, audio, image, realtime, and model-list routes. | `core/http/routes/openai.go` @ `b224c96d` | verified in source |
| The default API bind is `:8080`; authentication can be absent, static-key based, or database-backed with bearer keys/sessions. | `core/cli/run.go`, `core/http/auth/middleware.go`, `core/config/application_config.go` @ `b224c96d` | verified in source |
| Graceful server shutdown stops all tracked gRPC backend processes. | `core/cli/run.go`, `core/application/application.go`, `pkg/model/process.go`, `pkg/signals/handler.go` @ `b224c96d` | verified in source |
| The watchdog can evict or stop backends for idle, busy, LRU-limit, or memory-pressure conditions. | `pkg/model/watchdog.go`, `core/cli/run.go` @ `b224c96d` | verified in source |
| The desktop launcher is a separate Fyne tray application that starts the server as its child and interrupts or kills it on stop. | `cmd/launcher/main.go`, `cmd/launcher/internal/launcher.go` @ `b224c96d` | verified in source |
| GitHub Releases include native server binaries, a macOS DMG launcher, a Linux launcher archive, and source archives; Docker packaging is via in-tree `Dockerfile` / compose and separate image workflows (not release zip assets). | `.goreleaser.yaml`, `.github/workflows/release.yaml` (release assets); `Dockerfile`, `docker-compose.yaml`, `.github/workflows/image*.yml` @ `b224c96d` | verified from release assets + source |
| Tests cover in-process HTTP/OpenAI behavior, backend gRPC contracts, lifecycle eviction, launcher behavior, integration, and distributed/e2e scenarios. | `tests/e2e/`, `tests/e2e-backends/`, `tests/integration/`, `pkg/model/watchdog_test.go`, `cmd/launcher/internal/launcher_test.go` @ `b224c96d` | verified in source |

## Process ownership

For a local backend, `ModelLoader.startProcess` creates a `go-processmanager` process with a temporary state directory, arguments, environment, and working directory, then calls `Run`. The loader retains the process handle in the loaded model, tails child stdout/stderr, records an intentional-stop marker, and logs the exit code when the child terminates. `deleteProcess` calls the backend's optional gRPC `Free` method and then `Process.Stop`; `Application.Shutdown` calls `StopAllGRPC` synchronously. `pkg/model/process.go`, `core/application/application.go` @ `b224c96d`. **Confidence: verified in source.**

An external backend path is also spawned locally, but a non-file external backend value is treated as a remote gRPC address and represented by a model with a nil process handle. Distributed mode similarly routes model loads to worker nodes rather than creating the child in the frontend process. `pkg/model/initializers.go`, `pkg/model/loader.go` @ `b224c96d`. **Confidence: verified in source.**

The desktop topology adds another owner: the Fyne launcher starts `local-ai run` with `exec.CommandContext`, captures server stdout/stderr, waits for exit, and sends `os.Interrupt` with `Kill` fallback. The server then owns its backend children. `cmd/launcher/internal/launcher.go` @ `b224c96d`. **Confidence: verified in source.**

The inspected ownership path shows process handles and signal-based cleanup, but no Windows Job Object or Linux cgroup attachment. That is a scoped conclusion about these cited files, not a claim about every deployment wrapper. `pkg/model/process.go`, `cmd/launcher/internal/launcher.go` @ `b224c96d`. **Confidence: inferred from source.**

## Runtime boundary

The API/control plane, configuration, scheduling, model loading, and gRPC protocol are LocalAI code. Inference crosses a process boundary to a backend launcher or a network boundary to an external/distributed gRPC backend. `spawnGRPCModel` allocates a loopback port for local children, waits on `HealthCheck`, and sends `LoadModel` with the model path and options. `pkg/model/initializers.go`, `pkg/grpc/interface.go` @ `b224c96d`. **Confidence: verified in source.**

Multi-backend behavior is explicit, not an inferred product claim. Model YAML carries a `backend` string; aliases map names such as `llama` to `llama-cpp`, while the backend tree packages implementations spanning llama.cpp, IK llama.cpp, vLLM, Transformers, MLX, Whisper, Stable Diffusion, TTS, vision, and other workloads. `core/config/model_config.go`, `pkg/model/initializers.go`, `backend/` @ `b224c96d`. **Confidence: verified in source.**

Backend implementations are present in the same source repository, but they are built or installed as separate runtime artifacts exposing LocalAI's backend gRPC contract. A configured backend may also be only a remote address, so source-repository ownership and runtime-process ownership are distinct boundaries. `backend/cpp/grpc/`, `tests/e2e-backends/backend_test.go`, `pkg/model/initializers.go` @ `b224c96d`. **Confidence: verified in source.**

## Model and config storage

The CLI defaults models to `${basepath}/models`, mutable data to `${basepath}/data`, dynamic configuration to `${basepath}/configuration`, and backends to `${basepath}/backends`, with a separate system backend path at `/var/lib/local-ai/backends`. Every path has a corresponding CLI/environment option. `core/cli/run.go` @ `b224c96d`. **Confidence: verified in source.**

Model configuration is YAML. The loader accepts either one model object or an array, reads `.yaml` and `.yml` files non-recursively from the model path, validates them, and indexes them by name. A model config includes the backend, model parameters, templates, gRPC options, download files, and workload-specific settings. `core/config/model_config_loader.go`, `core/config/model_config.go` @ `b224c96d`. **Confidence: verified in source.**

Downloads are part of the architecture. The downloader recognizes Hugging Face, OCI, Ollama, GitHub, HTTP(S), and local URI schemes; gallery operations install and remove model/backend artifacts. `pkg/downloader/uri.go`, `core/services/galleryop/`, `core/gallery/backends.go` @ `b224c96d`. **Confidence: verified in source.**

The desktop launcher keeps its server binary under `~/.localai/bin`, defaults models and backends under `~/.localai/`, and passes explicit data, configuration, generated-content, and upload paths to the child server. `cmd/launcher/internal/release_manager.go`, `cmd/launcher/internal/launcher.go` @ `b224c96d`. **Confidence: verified in source.**

## API surface

The Echo server registers OpenAI-compatible routes including `/v1/chat/completions`, `/v1/completions`, `/v1/embeddings`, `/v1/audio/transcriptions`, `/v1/audio/speech`, `/v1/images/generations`, `/v1/realtime`, and `/v1/models`; several also have unprefixed aliases. It additionally registers LocalAI, Ollama, Anthropic, Jina, ElevenLabs, MCP, administration, and web UI routes elsewhere in `core/http/routes/`. `core/http/routes/openai.go`, `core/http/app.go` @ `b224c96d`. **Confidence: verified in source.**

The default bind is `:8080`. With neither database authentication nor static API keys configured, auth middleware passes requests through. Static keys can be extracted from bearer/key headers or cookies; database-backed auth adds sessions, user API keys, roles, permissions, and model allowlists. `core/cli/run.go`, `core/http/auth/middleware.go`, `core/http/routes/auth.go` @ `b224c96d`. **Confidence: verified in source.**

## Lifecycle

Startup constructs the application and HTTP server, then listens on the configured address. Model backends are loaded on demand or through preload settings; local loads start a backend child, poll its gRPC health endpoint, and call `LoadModel`. Failed startup tears down the child, and consecutive load failures are subject to a configurable cooldown. `core/cli/run.go`, `pkg/model/initializers.go`, `pkg/model/loader.go` @ `b224c96d`. **Confidence: verified in source.**

On `SIGINT` or `SIGTERM`, registered handlers run before process exit; the application synchronously stops tracked backend children. Backend exits are logged as intentional or unexpected with an exit code when available. The exit-watcher code records the event but does not itself restart the child. `pkg/signals/handler.go`, `core/application/application.go`, `pkg/model/process.go` @ `b224c96d`. **Confidence: verified in source.**

The watchdog can stop models that exceed busy or idle timeouts and can evict by LRU limit or memory threshold. A maximum of one active backend is supported as a configuration mode, but the product's runtime remains capable of many backend types. `pkg/model/watchdog.go`, `core/cli/run.go` @ `b224c96d`. **Confidence: verified in source.**

Within one launcher process, a boolean prevents a second `StartLocalAI` call. The cited launcher path does not establish a machine-wide singleton lock; a separately started server instead encounters the normal address-binding boundary. `cmd/launcher/internal/launcher.go` @ `b224c96d`. **Confidence: inferred from source.**

## Desktop vs service split

`cmd/local-ai` is the server/CLI entry point. `cmd/launcher` is a separate Fyne desktop application with a settings window and system tray; closing the window hides it, and the launcher owns a downloaded server child. `cmd/local-ai/`, `cmd/launcher/main.go`, `cmd/launcher/internal/launcher.go` @ `b224c96d`. **Confidence: verified in source.**

Distributed deployments use the same LocalAI image with different commands: a frontend serves the API while worker processes self-register and run backends. The supplied distributed Compose file also defines PostgreSQL and NATS infrastructure. `docker-compose.distributed.yaml`, `core/services/worker/` @ `b224c96d`. **Confidence: verified in source.**

The pinned release workflow builds desktop launcher artifacts for macOS and Linux. Native server release configuration targets Linux and Darwin; Windows is commented out. `.github/workflows/release.yaml`, `.goreleaser.yaml` @ `b224c96d`. **Confidence: verified in source.**

## Packaging

GoReleaser produces standalone `local-ai` server binaries for Linux amd64/arm64 and Darwin arm64, plus checksums and a source archive. The tag workflow separately builds a signed/notarized macOS DMG and a Linux launcher tarball. `.goreleaser.yaml`, `.github/workflows/release.yaml`, `contrib/macos/sign-and-notarize.sh` @ `b224c96d`. **Confidence: verified in source.**

The root `Dockerfile` is a multi-stage build with CPU and accelerator-specific requirements, and `docker-compose.yaml` exposes port 8080 with persistent model, data, backend, and configuration volumes. `Dockerfile`, `docker-compose.yaml` @ `b224c96d`. **Confidence: verified in source.**

## Test strategy

The ordinary Go/Ginkgo suite covers packages and core services, with a mock backend used to exercise the HTTP layer without downloading large models. `Makefile`, `tests/e2e/e2e_suite_test.go`, `tests/e2e/mock_backend_test.go` @ `b224c96d`. **Confidence: verified in source.**

`tests/e2e-backends/backend_test.go` starts a packaged backend's `run.sh`, connects with the generated gRPC client, and tests selectable capabilities such as health, load, prediction, streaming, embeddings, tools, transcription, TTS, and image generation. This tests the runtime boundary directly. `tests/e2e-backends/backend_test.go` @ `b224c96d`. **Confidence: verified in source.**

Lifecycle tests exercise LRU/single-backend eviction and busy-model behavior; launcher tests cover default paths, run arguments, and basic start/stop error handling. Distributed and integration suites cover worker/node and store behavior. `pkg/model/watchdog_test.go`, `cmd/launcher/internal/launcher_test.go`, `tests/e2e/distributed/`, `tests/integration/` @ `b224c96d`. **Confidence: verified in source.**

The cited launcher suite does not start a real downloaded server binary, so end-to-end desktop ownership across launcher, server, and inference child is not proven by that suite alone. `cmd/launcher/internal/launcher_test.go` @ `b224c96d`. **Confidence: verified in source.**

## Layout (text diagram)

```text
Optional desktop:
Fyne launcher/tray
  └─ exec: local-ai run
       ├─ Echo HTTP server :8080
       │    ├─ /v1 OpenAI-compatible routes
       │    ├─ LocalAI/Ollama/Anthropic/etc. routes
       │    └─ embedded Web UI
       │
       └─ ModelLoader
            ├─ model YAML + models directory
            ├─ local backend child (gRPC on 127.0.0.1:<free-port>)
            │    └─ llama.cpp | vLLM | Transformers | MLX | audio/image/etc.
            ├─ external backend gRPC address
            └─ distributed router
                 └─ worker node → installed backend child

Packaging:
native server binaries | macOS DMG launcher | Linux launcher archive | Docker
```

## Relevance to WinServeAI (facts only)

LocalAI's verified unit of orchestration is a model-selected gRPC backend drawn from many backend families; WinServeAI's documented unit is one bundled `llama-server.exe` controlled by one `ServerManager`. The difference is structural: LocalAI resolves backend names and installs or connects to backend runtimes, while WinServeAI keeps raw llama.cpp flags in one adapter and has no backend selection layer. See [Windows process research](../01-windows-process.md), [ServerManager research](../05-server-manager.md), and [prior art](../../prior-art.md). **Confidence: verified in source.**

LocalAI includes model/backend download flows, a broad API and embedded web UI, distributed workers, native binaries, desktop launchers, and Docker packaging. WinServeAI's repository rules define a Windows-native, single-backend appliance without model downloads, chat UI, Docker, or provider abstractions. This is a scope contrast, not a feature recommendation. LocalAI evidence: `pkg/downloader/uri.go`, `core/http/app.go`, `docker-compose.distributed.yaml`, `.goreleaser.yaml`, `Dockerfile` @ `b224c96d`; WinServeAI evidence: `AGENTS.md`, `docs/architecture.md`. **Confidence: verified in source.**
