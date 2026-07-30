# llama.cpp tools/server — architecture research

**Upstream:** https://github.com/ggml-org/llama.cpp  
**Pinned:** `b9866` (commit `75a48a90559abf65df3f3616a53bb16e5afb9d07`, published 2026-07-03)  
**Verified:** 2026-07-30  
**License (source tree):** MIT (`LICENSE` @ `b9866`)

Focus: `tools/server` (`llama-server` binary). Not the training stack, not unrelated CLI tools except as they share release zips.

## Summary

`llama-server` is the in-tree HTTP inference process for llama.cpp: a C++ executable that loads GGUF weights (or runs as an experimental multi-model router) and exposes REST routes including OpenAI-compatible `/v1/*`. Architecturally it **is** the inference process — there is no separate parent supervisor inside `tools/server` for single-model mode. Configuration is CLI flags and env vars (plus optional router INI presets / HF download), not a YAML product config. An embedded Web UI ships enabled by default; there is no desktop tray or Windows service. WinServeAI treats this binary as an **external** child under `bin/llama-server.exe`.

## Claims

| Claim | Evidence (path @ rev) | Confidence |
| --- | --- | --- |
| Entry point is `main` → `llama_server()`; CMake target `llama-server` | `tools/server/main.cpp`, `tools/server/CMakeLists.txt` @ `b9866` | verified in source |
| Single-model mode: this process owns HTTP + model load + inference loop | `tools/server/server.cpp` (`load_model`, `start_loop`) @ `b9866` | verified in source |
| Router mode (no `-m` / HF model): parent can spawn child `llama-server` instances via `subprocess` | `tools/server/server.cpp` (router branch), `tools/server/server-models.cpp` (`subprocess_create_ex`) @ `b9866` | verified in source |
| No Windows Job Object / cgroup ownership in server code | no matches in `tools/server/server.cpp` / `server-models.cpp` @ `b9866` | verified in source |
| Runtime is in-process libllama + cpp-httplib; not a separate engine binary | `tools/server/CMakeLists.txt` (`llama-server-impl` → `server-context`/`llama-ui`/`cpp-httplib`; `llama` via `llama-common`), `tools/server/README-dev.md` @ `b9866` | verified in source |
| Model path via `-m` / HF (`--hf-repo`); optional download in apply path | `tools/server/README.md` flags; `tools/server/server.cpp` (`common_models_handler_apply`) @ `b9866` | verified in source |
| Default bind `127.0.0.1:8080` | `tools/server/README.md` (Usage) @ `b9866` | verified in source |
| OpenAI routes include `/v1/models`, `/v1/chat/completions`, `/v1/completions`, `/v1/embeddings`, `/v1/responses` | `tools/server/server.cpp` route registration @ `b9866` | verified in source |
| Optional `--api-key` / `--api-key-file`; `/health`, `/v1/health`, `/models`, `/v1/models` public | `tools/server/server-http.cpp` (`get_public_endpoints`, middleware) @ `b9866` | verified in source |
| While `!is_ready`, middleware returns HTTP 503 “Loading model” for routes | `tools/server/server-http.cpp` (`middleware_server_state`) @ `b9866` | verified in source |
| `GET /health` when ready returns `{"status":"ok"}` | `tools/server/server-context.cpp` (`get_health`); `tools/server/README.md` @ `b9866` | verified in source |
| Failed model load exits process with code 1 | `tools/server/server.cpp` (`load_model` failure → `return 1`) @ `b9866` | verified in source |
| Embedded Web UI default on; disable with `--no-ui` | `tools/server/README.md`; `tools/server/tests/unit/test_basic.py` (`test_no_ui`) @ `b9866` | verified in source |
| No native desktop / tray / installer product in `tools/server` | product surface is HTTP + optional Web UI (`README.md`, `README-dev.md`) @ `b9866` | inferred from source |
| Windows release packaging is zip assets containing `llama-server.exe` (+ DLLs) | release `b9866` asset `llama-b9866-bin-win-cpu-x64.zip` listing @ GitHub Releases | verified from release assets |
| Automated server tests: pytest under `tools/server/tests` spawning `llama-server` | `tools/server/tests/tests.sh`, `utils.py`, `unit/test_*.py` @ `b9866` | verified in source |

## Process ownership

Who owns the inference process? **Usually none above it — `llama-server` is the process.**

- **Single-model mode:** `llama_server()` in `tools/server/server.cpp` runs HTTP on a thread, loads the model in-process, then blocks on `ctx_server.start_loop()`. Shutdown is SIGINT/SIGTERM (POSIX) or `CTRL_C_EVENT` via `SetConsoleCtrlHandler` (Windows) → `shutdown_handler` → terminate loop / stop HTTP. There is no Job Object, cgroup, or pid-file single-instance lock in `tools/server`.
- **Router mode:** when no model path / HF repo is set, the same binary acts as a router and **spawns child** `llama-server` instances (`tools/server/server-models.cpp`, `subprocess_create_ex`, env `LLAMA_SERVER_CHILD_MODE`). Children notify the router of state. Parent cleanup calls `models.unload_all()`.
- **External wrappers** (including WinServeAI) may own the process as a child; that ownership is outside this tree. See [docs/research/01-windows-process.md](../01-windows-process.md) and [docs/prior-art.md](../../prior-art.md).

## Runtime boundary

What is “our code” vs external binary / library?

| Layer | Role @ `b9866` |
| --- | --- |
| `tools/server/main.cpp` | thin `main` → `llama_server` |
| `llama-server-impl` (`server.cpp`, `server-http.*`, `server-models.*`) | HTTP routes, router, process orchestration for children |
| `server-context` static lib | inference slots, queue, chat/task logic |
| Linked libs | `llama` / ggml, `llama-common`, `mtmd`, `cpp-httplib`, embedded `llama-ui` assets |

There is **one** inference backend family: llama.cpp in-process. Router mode multiplies **processes** of the same binary, not alternate engines. Web UI is served from embedded assets (`--ui` / `--no-ui`), not a separate desktop app.

For WinServeAI, **all of the above is external**: the shipped artifact is `bin/llama-server.exe` (+ sibling DLLs). WinServeAI code stops at argv construction and process lifecycle ([docs/backend.md](../../backend.md)).

## Model and config storage

- **Models:** local GGUF via `-m` / `LLAMA_ARG_MODEL`, or Hugging Face via `--hf-repo` / `--hf-file` (download path in `common_models_handler_apply` from `server.cpp`). Router: `--models-dir`, `--models-preset` (INI), cache / `POST /models` download routes documented in `tools/server/README.md`.
- **Config:** CLI + env (`LLAMA_ARG_*`, `LLAMA_API_KEY`). Optional `--ui-config` / `--ui-config-file` (JSON) for Web UI defaults only — not a full server YAML. No first-class product YAML inside `tools/server`.
- **Downloads:** supported when HF flags / router model APIs are used; not required for a local `-m` path.

## API surface

- **Transport:** HTTP (cpp-httplib). Default **`127.0.0.1:8080`** (`--host` / `--port`).
- **OpenAI-compatible (registered in `server.cpp`):** e.g. `GET /v1/models`, `POST /v1/completions`, `POST /v1/chat/completions`, `POST /v1/embeddings`, `POST /v1/responses`, plus aliases without `/v1` for some routes. Also Anthropic `/v1/messages`, legacy `/completion`, `/infill`, `/rerank`, `/slots`, `/metrics` (flag-gated), experimental `/tools`, GCP Vertex env compatibility in `server-http.cpp`.
- **Auth:** optional `--api-key` / `--api-key-file`. Middleware checks `Authorization: Bearer …` or `X-Api-Key`. Public without key: `/health`, `/v1/health`, `/models`, `/v1/models`, `/`, and UI assets (`server-http.cpp`). Empty key list → no auth.
- **Health / readiness:** while loading, `middleware_server_state` returns **503** with `unavailable_error` / “Loading model” for all non-OPTIONS traffic. When ready, `GET /health` and `GET /v1/health` return **200** `{"status":"ok"}`. Documented in `tools/server/README.md`. WinServeAI’s product probe is `GET /v1/models` — see [docs/research/02-llama-readiness.md](../02-llama-readiness.md).

## Lifecycle

| Event | Behavior (single-model) |
| --- | --- |
| Start | Parse args → init backend → start HTTP **before** model load → `load_model` → set `is_ready` → `start_loop` |
| Ready | `is_ready == true`; `/health` and `/v1/models` succeed |
| Load failure | clean up, join HTTP thread, **`return 1`** |
| Stop | signal → terminate server context / stop HTTP → `llama_backend_free` |
| Crash | process exits; no built-in supervisor restart in `tools/server` |
| Single-instance | **none** enforced in source; multiple listeners possible if ports differ |
| Idle sleep | optional `--sleep-idle-seconds`; `/health`, `/props`, `/models` do not wake/reset idle (`README.md`) |

Router mode: load/unload via `/models/load` etc.; children spawned/joined in `server-models.cpp`.

## Desktop vs service split

- **No** native desktop shell, tray, or Windows service in `tools/server`.
- **Optional embedded Web UI** (default enabled) at the listen URL; disable with `--no-ui` (`test_no_ui` in `tests/unit/test_basic.py`).
- **CLI-only** process for headless use; Docker images documented in `README.md` (`ghcr.io/ggml-org/llama.cpp:server`).
- README-dev scopes chat/agent UI features to the Web frontend; out-of-scope includes server-side agentic loops.

## Packaging

- **GitHub Releases** tag `b####` (here **`b9866`**): platform zip/tar assets, e.g. `llama-b9866-bin-win-cpu-x64.zip`, `llama-b9866-bin-win-cuda-13.3-x64.zip`, Vulkan/HIP/OpenVINO variants, plus Linux/macOS tarballs and `llama-b9866-ui.tar.gz`.
- Inspected Windows CPU zip contains **`llama-server.exe`**, `llama-server-impl.dll`, and many sibling ggml/llama DLLs (not a single static exe).
- Also: build-from-source (`cmake -t llama-server`), Docker server images per README.
- Not an Inno/MSI product installer for `tools/server` alone.

## Test strategy

| Kind | Location @ `b9866` | What it proves |
| --- | --- | --- |
| Integration (pytest) | `tools/server/tests/unit/*.py` via `tests.sh` | Spawns real `llama-server` (`tests/utils.py` `ServerProcess`), hits HTTP |
| Basic / health | `test_basic.py` | `GET /health` → 200 after start; `/models`; `--no-ui` |
| Security / auth | `test_security.py` | Public `/health` `/models`; 401 without key; Bearer / `X-Api-Key` |
| OpenAI / chat / embeddings / router / sleep / etc. | other `test_*.py` | API surface and router behavior |
| Slow / tool-call | gated by `SLOW_TESTS=1` in `tests.sh` | heavier model fetches |

These tests prove **API and process start/stop from a test harness**, not Windows Job Object ownership (that is WinServeAI’s concern — [docs/research/01-windows-process.md](../01-windows-process.md)).

## Layout (text diagram)

```text
                    (optional external parent, e.g. WinServeAI ServerManager)
                                        │ spawn/kill
                                        ▼
┌──────────────────────────────────────────────────────────────────────┐
│  llama-server  (tools/server) — THIS IS THE PROCESS                  │
│                                                                      │
│  main.cpp → llama_server()                                           │
│       │                                                              │
│       ├─ server_http_context (cpp-httplib)                           │
│       │     GET  /health  /v1/health  /v1/models  …                  │
│       │     POST /v1/chat/completions  /v1/completions  …            │
│       │     [optional] embedded Web UI static assets                 │
│       │                                                              │
│       ├─ single-model: server_context + libllama (in-process GGUF)   │
│       │                                                              │
│       └─ router mode (no -m): server_models                          │
│             └─ subprocess → child llama-server instances             │
└──────────────────────────────────────────────────────────────────────┘
            ▲
            │  HTTP :8080 default (127.0.0.1)
            │
         OpenAI / Anthropic / legacy clients
```

## Relevance to WinServeAI (facts only)

- **External boundary:** WinServeAI’s documented ship pin is **`b9866`** (`bin/VERSION`, [docs/release-process.md](../../release-process.md)); fetch scripts pull release zips and install `llama-server.exe` under `bin/`. That binary is **not** WinServeAI code ([docs/backend.md](../../backend.md), AGENTS.md).
- **Steal (verified):** OpenAI `/v1` surface; readiness signals (`503` while loading, `/health` + `/v1/models` when ready) documented in upstream README and exercised in `test_basic.py` / middleware — align probes with [docs/research/02-llama-readiness.md](../02-llama-readiness.md). Pin release zips, keep DLLs beside the exe.
- **Avoid treating as product shell:** embedded Web UI, HF download, router multi-model, and experimental `/tools` / CORS proxy are **upstream** features; WinServeAI’s niche is process ownership + YAML→argv + logs, not re-owning those surfaces ([docs/prior-art.md](../../prior-art.md)).
- **Process ownership gap:** upstream does not provide Job Objects / orphan reaping; wrappers must own lifecycle ([docs/research/01-windows-process.md](../01-windows-process.md), [docs/research/05-server-manager.md](../05-server-manager.md)).
- **Do not** grow multi-backend traits, chat UI, or model marketplaces to “match” llama-server’s broader README feature list.
