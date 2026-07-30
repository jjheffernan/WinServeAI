# Ollama — architecture research

**Upstream:** https://github.com/ollama/ollama  
**Pinned:** `v0.32.5` (`eec8e0b9458b8a01be0c216a9cc53eefde24ef50`)  
**Verified:** 2026-07-30  
**License (source tree):** MIT (`LICENSE`)

## Summary

Ollama is a Go CLI/daemon plus a macOS/Windows desktop tray app. The daemon listens on `OLLAMA_HOST` (default `127.0.0.1:11434`), owns model pull/create/list storage under `OLLAMA_MODELS`, and schedules loaded runners. GGUF inference is not in-process: the scheduler spawns an upstream `llama-server` binary (pinned by `LLAMA_CPP_VERSION`) and talks to it over HTTP. Safetensors/MLX models take a separate path via `ollama runner --mlx-engine`. The desktop app is a parent that repeatedly launches `ollama serve` as a child, with Windows single-instance tray behavior and process reaping—not a Windows Service SCM product.

## Claims

| Claim | Evidence (path @ rev) | Confidence |
| --- | --- | --- |
| Default HTTP bind is `http` + `127.0.0.1:11434` via `OLLAMA_HOST` | `envconfig/config.go` @ `v0.32.5` | verified in source |
| `ollama serve` listens on `envconfig.Host().Host` then calls `server.Serve` | `cmd/cmd.go` (`RunServer`) @ `v0.32.5` | verified in source |
| GGUF models use `llama-server` subprocess; `NewLlamaServer` documents that path | `llm/server.go`, `llm/llama_server.go` @ `v0.32.5` | verified in source |
| `llama-server` located under packaged `lib/ollama` layouts | `llm/llama_binary.go` (`FindLlamaCppBinary`) @ `v0.32.5` | verified in source |
| Upstream llama.cpp pin is `b10091` | `LLAMA_CPP_VERSION` @ `v0.32.5` | verified in source |
| Runner stop uses `Process.Kill()` (no Job Object on this path) | `llm/llama_server.go` (`stopProcess`) @ `v0.32.5` | verified in source |
| Windows llama-server spawn: `CREATE_NO_WINDOW` \| `ABOVE_NORMAL_PRIORITY_CLASS` \| `CREATE_DEFAULT_ERROR_MODE` | `llm/llm_windows.go` @ `v0.32.5` | verified in source |
| Scheduler loads GGUF via `llm.NewLlamaServer`; MLX via `mlxrunner.NewClient` | `server/sched.go` @ `v0.32.5` | verified in source |
| MLX runner is `exec` of same binary with `runner --mlx-engine` | `x/mlxrunner/client.go` (`Load`) @ `v0.32.5` | verified in source |
| Models dir default `$HOME/.ollama/models`; override `OLLAMA_MODELS` | `envconfig/config.go` (`Models`) @ `v0.32.5` | verified in source |
| On-disk layout: `manifests/` + `blobs/` under models dir | `manifest/paths.go` @ `v0.32.5` | verified in source |
| Native API `/api/*` and OpenAI-compat `/v1/*` on same router | `server/routes.go` (`GenerateRoutes`) @ `v0.32.5` | verified in source |
| Keep-alive default 5m (`OLLAMA_KEEP_ALIVE`); scheduler expires/unloads runners | `envconfig/config.go`, `server/sched.go` @ `v0.32.5` | verified in source |
| Desktop app starts `ollama serve` child and restarts on exit | `app/server/server.go` (`Run`), `app/cmd/app/app.go` @ `v0.32.5` | verified in source |
| Windows desktop: pid under `%LOCALAPPDATA%\Ollama`, `CREATE_NEW_PROCESS_GROUP`, CTRL_BREAK stop, `taskkill /F /T` reap | `app/server/server_windows.go` @ `v0.32.5` | verified in source |
| Windows single-instance exits if tray already running | `app/cmd/app/app_windows.go` (`handleExistingInstance`) @ `v0.32.5` | verified in source |
| Job Object `KILL_ON_JOB_CLOSE` exists only for agent shell tools, not llama-server | `agent/tools/bash_windows.go` @ `v0.32.5` | verified in source |
| Windows packaging is Inno Setup (`OllamaSetup.exe`); ships `llama-server.exe` | `app/ollama.iss` @ `v0.32.5` | verified in source |
| Local API requires no auth; cloud/registry auth is separate | `docs/api/authentication.mdx` @ `v0.32.5` | verified in source |
| Integration tests are build-tag gated (`integration` + scope tags) | `integration/README.md` @ `v0.32.5` | verified in source |

## Process ownership

Who owns the inference process? Parent/child? Service? Job Objects / cgroup?

- **Daemon as parent of runners.** `server.Serve` starts `InitScheduler`; the scheduler’s `newServerFn` defaults to `llm.NewLlamaServer`, which spawns `llama-server` (`server/sched.go`, `llm/server.go`, `llm/llama_server.go` @ `v0.32.5`) — **verified in source**.
- **Child lifecycle.** `startLlamaServer` builds `exec.Command`, sets `LlamaServerSysProcAttr`, `cmd.Start()`, and reaps with `cmd.Wait` in a goroutine; `Close`/`stopProcess` calls `cmd.Process.Kill()` (`llm/llama_server.go` @ `v0.32.5`) — **verified in source**.
- **Readiness of child.** Status polled via `GET http://127.0.0.1:<port>/health` on the child (`getServerStatus` in `llm/llama_server.go` @ `v0.32.5`) — **verified in source**.
- **Desktop as parent of daemon.** On Windows/macOS, `app/cmd/app` starts `app/server.Server.Run`, which execs `ollama serve`, writes a pid file, and loops with `restartDelay` after exit (`app/server/server.go`, `app/cmd/app/app.go` @ `v0.32.5`) — **verified in source**.
- **Windows stop/reap of daemon.** `terminate` uses `AttachConsole` + `GenerateConsoleCtrlEvent` (`CTRL_BREAK` / `CTRL_C`); `reapServers` uses `wmic` + `taskkill /F /T /PID` for other `ollama.exe` serve processes (`app/server/server_windows.go` @ `v0.32.5`) — **verified in source**.
- **Job Objects.** `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` is used in `agent/tools/bash_windows.go` for agent PowerShell children, not for `llama-server` or `ollama serve` (`agent/tools/bash_windows.go`; absence in `llm/llama_server.go` / `app/server/*` @ `v0.32.5`) — **verified in source**.
- **Linux “service”.** Docs recommend a systemd unit with `ExecStart=/usr/bin/ollama serve` and `Restart=always` (`docs/linux.mdx` @ `v0.32.5`) — **verified in source**. That is packaging documentation, not an in-tree Windows Service implementation.
- **cgroup.** No cgroup ownership API observed in the pinned daemon/runner spawn paths — **unverified** (not present in inspected spawn code).

Internal WinServeAI context on Job Objects / orphan policy: [docs/research/01-windows-process.md](../01-windows-process.md), [docs/prior-art.md](../../prior-art.md).

## Runtime boundary

What is “our code” vs external binary / library? Multiple backends?

- **Ollama-owned (Go):** CLI (`cmd/`, `main.go`), HTTP API + scheduler (`server/`), env config (`envconfig/`), manifest/blob store (`manifest/`), OpenAI/Anthropic request shims (`openai/`, `middleware/`, `anthropic/`), desktop/tray (`app/`), discovery (`discover/`).
- **External inference binary (GGUF):** packaged `llama-server` under `lib/ollama` search paths; comment states “wraps the llama-server binary as a subprocess” (`llm/llama_server.go`, `llm/llama_binary.go` @ `v0.32.5`) — **verified in source**. Build pin: `LLAMA_CPP_VERSION` = `b10091`; CMake tree under `llama/server` (workflow references) — **verified in source**.
- **Second runner path (MLX / safetensors):** `Model.IsMLX()` when `Config.ModelFormat == "safetensors"` (`server/images.go`); scheduler calls `mlxrunner.NewClient`, which starts `os.Executable()` with args `runner --mlx-engine --model … --port …` (`server/sched.go`, `x/mlxrunner/client.go` @ `v0.32.5`) — **verified in source**.
- **Multiple backends:** at least llama.cpp server subprocess + MLX runner subprocess (+ imagegen path referenced in scheduler fields) — **verified in source**. Not a single-backend appliance.
- **In-tree ggml sources** also exist under `ml/backend/ggml/` — role relative to the subprocess path is packaging/build support; inference for GGUF at this pin goes through `llama-server` (`llm/server.go` comment “All GGML models are served via the upstream llama-server subprocess”) — **verified in source**.

## Model and config storage

Where models live; config format; downloads?

- **Models root:** `envconfig.Models()` → `OLLAMA_MODELS` or `$HOME/.ollama/models` (`envconfig/config.go` @ `v0.32.5`) — **verified in source**.
- **Layout:** `…/manifests/<host>/<namespace>/<model>/<tag>` and `…/blobs/<digest>` (`manifest/paths.go`, `manifest/manifest.go` @ `v0.32.5`) — **verified in source**.
- **Downloads:** `server/download.go` pulls registry blobs into the blobs directory (`downloadBlob` → `manifest.BlobsPath`) — **verified in source**. Exposed as `POST /api/pull` (`server/routes.go` @ `v0.32.5`) — **verified in source**.
- **Create / Modelfile:** create API + Modelfile docs (`server/create.go`, `docs/modelfile.mdx` @ `v0.32.5`) — **verified in source**.
- **Runtime config:** primarily environment variables documented on `serve` (`cmd/cmd.go` env docs list; `envconfig/config.go`) — **verified in source**. Not a single YAML appliance file as source of truth.
- **Identity keys:** `~/.ollama/id_ed25519` generated on serve if missing (`cmd/cmd.go` `initializeKeypair`) — **verified in source**.
- **Windows paths (docs):** binaries under `%LOCALAPPDATA%\Programs\Ollama`; models under `%HOMEPATH%\.ollama`; logs under `%LOCALAPPDATA%\Ollama` (`docs/windows.mdx` @ `v0.32.5`) — **verified in source**.

## API surface

OpenAI-compatible? Ports? Auth?

- **Port / host:** default `127.0.0.1:11434`; overridable with `OLLAMA_HOST` (`envconfig/config.go`, `docs/windows.mdx` @ `v0.32.5`) — **verified in source**.
- **Native API:** `/api/generate`, `/api/chat`, `/api/embed`, `/api/embeddings`, `/api/tags`, `/api/pull`, `/api/push`, `/api/create`, `/api/ps`, `/api/version`, etc. (`server/routes.go` @ `v0.32.5`) — **verified in source**.
- **OpenAI compatibility:** `/v1/chat/completions`, `/v1/completions`, `/v1/embeddings`, `/v1/models`, `/v1/responses`, image/audio endpoints; middleware in `middleware/` (`server/routes.go`, `docs/api/openai-compatibility.mdx` @ `v0.32.5`) — **verified in source**. Docs show client `api_key='ollama'` as required-but-ignored for local OpenAI SDK use — **verified in source** (docs).
- **Anthropic-compat:** `POST /v1/messages` (`server/routes.go` @ `v0.32.5`) — **verified in source**.
- **Auth (local):** docs state no authentication required for local `http://localhost:11434` (`docs/api/authentication.mdx` @ `v0.32.5`) — **verified in source**.
- **Auth (cloud/registry):** sign-in / `OLLAMA_API_KEY` for ollama.com and private/publish flows (`docs/api/authentication.mdx`, `server/auth.go` registry challenge helpers @ `v0.32.5`) — **verified in source**.
- **Host gate:** when the server addr is loopback, `allowedHostsMiddleware` rejects non-allowed `Host` headers with 403 (`server/routes.go` @ `v0.32.5`) — **verified in source**. Binding off-loopback skips that loopback Host check (same function) — **verified in source**.

## Lifecycle

Start / stop / crash / single-instance behavior.

- **Start (CLI):** `ollama serve` → listen → `server.Serve` → scheduler `Run` → on demand `load` spawns runner (`cmd/cmd.go`, `server/routes.go` `Serve`, `server/sched.go` @ `v0.32.5`) — **verified in source**.
- **Unload:** keep-alive timer / max-runners eviction sends `expiredCh` → `runner.unload()` (`server/sched.go`, `envconfig.KeepAlive` @ `v0.32.5`) — **verified in source**.
- **Daemon shutdown:** SIGINT/SIGTERM closes HTTP server, cancels scheduler, `unloadAllRunners` (`server/routes.go` `Serve` @ `v0.32.5`) — **verified in source**.
- **Child crash:** wait goroutine records `doneErr`; health/wait paths surface termination (`llm/llama_server.go` @ `v0.32.5`) — **verified in source**.
- **Desktop restart:** `Server.Run` loop restarts `ollama serve` after delay; exit code 1 may trigger `reapServers` once for port conflict (`app/server/server.go` @ `v0.32.5`) — **verified in source**.
- **Single-instance (Windows tray):** `handleExistingInstance` → `wintray.CheckAndFocusExistingInstance` then `os.Exit(0)` (`app/cmd/app/app_windows.go` @ `v0.32.5`) — **verified in source**.
- **CLI auto-start app:** Windows `startApp` launches `ollama app.exe --hide --fast-startup` if server heartbeat fails (`cmd/start_windows.go`, `cmd/cmd.go` `checkServerHeartbeat` @ `v0.32.5`) — **verified in source**.

Compare WinServeAI manager state machine (single child, Ready via `/v1/models`): [docs/research/05-server-manager.md](../05-server-manager.md).

## Desktop vs service split

GUI, CLI, daemon, tray — how they relate.

```text
[Windows/macOS tray app: ollama app / Ollama.app]
        │  spawns & restarts
        ▼
[ollama serve]  ←── also: CLI `ollama serve`, Linux systemd, Docker ENTRYPOINT
        │  schedules
        ├──► llama-server (GGUF)
        └──► ollama runner --mlx-engine (safetensors/MLX)

[CLI: ollama run/pull/…] ──HTTP──► OLLAMA_HOST (expects serve up; may start app)
[Desktop webview UI] ── local UI HTTP ──► chat/settings (app/ui)
```

- Tray/desktop owns **serve process** supervision; serve owns **runner** supervision — **verified in source** (`app/server/server.go`, `server/sched.go`).
- Linux docs treat systemd as optional wrapper around `ollama serve` (`docs/linux.mdx`) — **verified in source**.
- macOS LaunchAgent plist runs Squirrel `background` (updater/background helper), not `ollama serve` directly (`app/darwin/Ollama.app/Contents/Library/LaunchAgents/com.ollama.ollama.plist` @ `v0.32.5`) — **verified in source**.

## Packaging

Installer / brew / docker / portable.

- **Windows:** Inno Setup script `app/ollama.iss` → `OllamaSetup.exe`; default dir `{localappdata}\Programs\Ollama`; `PrivilegesRequired=lowest`; defines `LlamaServerExeName "llama-server.exe"` (`app/ollama.iss` @ `v0.32.5`) — **verified in source**. User docs: `OllamaSetup.exe /DIR=…`, uninstall via Apps & features (`docs/windows.mdx`) — **verified in source**.
- **macOS:** DMG / drag to Applications (`docs/macos.mdx`, `app/README.md` @ `v0.32.5`) — **verified in source**.
- **Linux:** `install.sh` and `.tar.zst` under `/usr`; optional systemd (`docs/linux.mdx` @ `v0.32.5`) — **verified in source**.
- **Docker:** `docker run … -p 11434:11434 ollama/ollama` with volume on `~/.ollama` (`docs/docker.mdx`); multi-stage `Dockerfile` builds llama-server GPU/CPU payloads into `lib/ollama/` (`Dockerfile` @ `v0.32.5`) — **verified in source**.

## Test strategy

What automated tests exist (unit/integration/e2e) that prove ownership or API?

- **Unit tests:** widespread `*_test.go` under `server/` (routes, sched), `llm/`, `openai/`, `envconfig/`, `manifest/`, `api/`, etc. (tree at pin) — **verified in source**.
- **Integration:** `integration/` disabled by default; requires `-tags=integration,<fast|release|library>`; can spawn server on random port (Unix) or use existing `OLLAMA_HOST`; Windows requires existing server on `OLLAMA_HOST` (`integration/README.md` @ `v0.32.5`) — **verified in source**.
- **CI:** `.github/workflows/test.yaml` builds/patches llama-server presets (CPU/CUDA/ROCm), Go tests, app change detection; references `LLAMA_CPP_VERSION` / `llama/server` (`.github/workflows/test.yaml` @ `v0.32.5`) — **verified in source**.
- **Ownership proof:** scheduler/runner tests and integration API tests exercise load/generate paths; there is no Job Object test for llama-server (Job Object code is agent-tool scoped) — **inferred from source**.

## Layout (text diagram)

```text
┌─────────────────────────────────────────────────────────────┐
│  Desktop (Windows/macOS): app/cmd/app + wintray / cocoa     │
│    • single-instance tray                                   │
│    • app/server.Server.Run → exec "ollama serve"            │
│    • logs: %LOCALAPPDATA%\Ollama\server.log (Windows)       │
└────────────────────────────┬────────────────────────────────┘
                             │ parent/child
                             ▼
┌─────────────────────────────────────────────────────────────┐
│  ollama serve (cmd → server.Serve)                          │
│    listen OLLAMA_HOST (default 127.0.0.1:11434)             │
│    gin routes: /api/*  +  /v1/*  (+ /v1/messages)           │
│    Scheduler (server/sched.go)                              │
│      keep-alive / max-loaded unload                         │
└───────────────┬──────────────────────────┬──────────────────┘
                │                          │
                ▼                          ▼
   ┌────────────────────────┐   ┌─────────────────────────────┐
   │ llama-server.exe/.bin  │   │ ollama runner --mlx-engine  │
   │ (lib/ollama/, pin      │   │ (x/mlxrunner; safetensors)  │
   │  LLAMA_CPP_VERSION)    │   │                             │
   │ HTTP :ephemeral        │   │ HTTP :ephemeral             │
   │ health → /health       │   │                             │
   └────────────────────────┘   └─────────────────────────────┘

Disk: $OLLAMA_MODELS or ~/.ollama/models/{manifests,blobs}
CLI clients / OpenAI SDKs → http://127.0.0.1:11434
```

## Relevance to WinServeAI (facts only)

WinServeAI is a Windows llama.cpp appliance wrapper: one `ServerManager`, raw flags only in `runtime/llama.rs`, YAML config source of truth, no multi-backend / no model marketplace ([AGENTS.md](../../../AGENTS.md), [docs/prior-art.md](../../prior-art.md)).

| Observed in Ollama (verified) | Fact relative to WinServeAI niche |
| --- | --- |
| Daemon schedules **multiple** runner kinds (`llama-server` + MLX) (`server/sched.go`) | Opposite of WinServeAI’s single-backend rule — do not copy multi-runner abstraction. |
| GGUF path already wraps **external `llama-server`** with HTTP readiness (`llm/llama_server.go`) | Same external boundary class as WinServeAI’s `bin/llama-server.exe`; Ollama adds registry/scheduler/desktop around it. |
| Inference child stop is **`Process.Kill`**, not Job Object (`llm/llama_server.go`); Job Object only in agent bash (`agent/tools/bash_windows.go`) | WinServeAI’s Job Object orphan backstop ([docs/research/01-windows-process.md](../01-windows-process.md)) is not mirrored on Ollama’s llama-server path at this pin. |
| Desktop owns **`ollama serve`**, serve owns runners (`app/server/server.go`) | Similar “one owner above the engine” idea; Ollama’s owner still includes pull/UI/multi-engine. |
| Default **localhost:11434**, local API unauthenticated (`envconfig/config.go`, `docs/api/authentication.mdx`) | API-as-product default matches prior-art “steal” note; LAN bind without auth remains an exposure class when `OLLAMA_HOST` is changed ([docs/prior-art.md](../../prior-art.md)). |
| Models via **pull + manifests/blobs** (`server/download.go`, `manifest/paths.go`) | WinServeAI explicitly excludes model downloads; do not import registry/manifest store. |
| Config via **env vars**, not appliance YAML (`envconfig/config.go`) | Different source-of-truth model than `config/default.yaml`. |
| OpenAI `/v1` is a **compat layer over native handlers** (`server/routes.go`, `middleware/`) | WinServeAI treats llama-server `/v1` as the passthrough product surface ([docs/research/05-server-manager.md](../05-server-manager.md)) — thinner stack. |
| Windows install: **Inno Setup**, user-local, ships `llama-server.exe` (`app/ollama.iss`) | Installer family overlaps WinServeAI’s Inno interest ([docs/research/04-installer-licensing.md](../04-installer-licensing.md)); still ships pull/tray/UI product surface Ollama owns. |
