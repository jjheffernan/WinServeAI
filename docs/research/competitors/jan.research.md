# Jan — architecture research

**Upstream:** https://github.com/janhq/jan
**Pinned:** `v0.8.4` @ `5f30aee467f08941964a83f946e2663e7ae0e01f`
**Verified:** 2026-07-30
**License (source tree):** Apache-2.0 — `LICENSE` (root); `src-tauri/Cargo.toml` declares `license = "MIT"` for the crate

## Summary

Jan is a Tauri 2 desktop application: a React webview (`web-app/`) over a Rust host process (`src-tauri/`), with inference backends implemented as Tauri plugins and driven by TypeScript "extensions". As of `v0.8.4` the llama.cpp path spawns exactly **one** `llama-server` child in upstream **router mode** (`--models-preset` + `--models-max`), with models loaded and unloaded on demand over that server's own HTTP API rather than by spawning one process per model. A second Rust component — the "Jan API server" (`src-tauri/src/core/server/proxy.rs`) — is a hyper reverse proxy on port `1337` that fronts the router *and* remote cloud providers behind one OpenAI-shaped surface. `llama-server` binaries are not vendored: they are downloaded at runtime from Jan's own `janhq/llama.cpp` fork releases. Jan is explicitly multi-backend (llamacpp, MLX, remote providers) and ships a chat UI, model hub, threads database, RAG, and MCP host — i.e. the opposite scope from WinServeAI.

## Claims

| Claim | Evidence (path @ rev) | Confidence |
| --- | --- | --- |
| Exactly one `llama-server` spawn site exists in the llamacpp plugin | `src-tauri/plugins/tauri-plugin-llamacpp/src/router.rs:212` @ `5f30aee` (the plugin's only `.spawn()`; `device.rs:32` runs the backend exe to completion via `.output()` (`:55`) to probe `--list-devices`, and `deps_analyzer.rs:132` re-execs Jan's own binary — `current_exe()` at `:116` with `--internal-analyze-deps` — not the backend) | verified in source |
| llama.cpp runs in router mode: one process, many models, loaded via HTTP | `src-tauri/plugins/tauri-plugin-llamacpp/src/router.rs:1-6` ("spawn / health-check / shut down a **single** `llama-server` instance running in router mode (no `-m` / `-hf` flag, models are loaded on demand via the HTTP API)") | verified in source |
| Router argv is `--models-preset <ini> --models-max <n> --host 127.0.0.1 --port <p>` | `src-tauri/plugins/tauri-plugin-llamacpp/src/router.rs:114-135` (`router_args`) | verified in source |
| Router binds `127.0.0.1` only, on a random port chosen per start | `router.rs:126` (`--host 127.0.0.1`); `extensions/llamacpp-extension/src/index.ts:714` (`await this.getRandomPort()`) | verified in source |
| Readiness is detected by scraping child stdout/stderr for a log line, not by HTTP polling | `router.rs:30-36` (`is_ready_line`), `router.rs:221-316` (stdout+stderr reader tasks), `router.rs:332-367` (select loop) | verified in source |
| Five distinct ready-line phrasings are matched because upstream changed the wording | `router.rs:22-36` and regression test `router.rs:696-712` (`is_ready_line_matches_router_mode_wording`) | verified in source |
| Readiness timeout defaults to 60s, overridable via `LLAMA_ARG_TIMEOUT` in the env map | `router.rs:177-180` | verified in source |
| API key is passed via the `LLAMA_API_KEY` env var and deliberately never via argv | `router.rs:106-113` (doc comment: "argv is visible to any other process on the machine (`ps`/Task Manager, `/proc`), and we already log the full argv at startup"), `router.rs:140-143`, test `router.rs:721-729` | verified in source |
| **No Job Object is used.** Windows children get `CREATE_NO_WINDOW \| CREATE_NEW_PROCESS_GROUP` only | `src-tauri/utils/src/system.rs:551-562` (`setup_windows_process_flags`), called at `router.rs:188` | verified in source |
| Orphan containment is done by `sysinfo` PID-tree sweep (direct children only), plus `kill_on_drop(true)` | `router.rs:187` (`kill_on_drop`), `router.rs:588-611` / `614-646` (`force_kill_router_tree*`: enumerate `p.parent() == Some(rpid)`, kill router first, then children) | verified in source |
| Graceful shutdown is **not implemented on Windows**; the child is force-killed | `src-tauri/plugins/tauri-plugin-llamacpp/src/process.rs:28-60` (`force_terminate_process`), selected at `router.rs:574-577` | verified in source |
| The stated reason is that `llama-server` only handles `CTRL_C_EVENT`, which `GenerateConsoleCtrlEvent` can only deliver to group 0 (killing the parent too), while `CTRL_BREAK_EVENT` is ignored by the server | `process.rs:30-34` (comment) | verified in source |
| Unix path is SIGTERM → 5s wait → SIGKILL | `process.rs:1-26` (`graceful_terminate_process`) | verified in source |
| Windows llamacpp support is gated to `x86_64`; Windows ARM64 falls through to a plain `kill()` | `process.rs:28` (`#[cfg(all(windows, target_arch = "x86_64"))]`), fallback at `router.rs:578-582`; `system.rs:552`/`558` mirror the same gate | verified in source |
| Graceful stop is an HTTP protocol, not a signal: enumerate `/models`, POST `/models/unload` per model, poll `/slots` for `is_processing`, then terminate | `router.rs:396-482` (`try_graceful_stop_router`), `router.rs:484-567` (`list_busy_models`, `list_models_filtered`, `list_processing_models`) | verified in source |
| Default stop deadline is 10s; on deadline the whole PID tree is force-killed | `router.rs:380-393` (`stop_router` → `Duration::from_secs(10)` → `force_kill_router_tree`) | verified in source |
| Only one router may run at a time, enforced by a `Mutex<Option<RouterHandle>>` guard | `src-tauri/plugins/tauri-plugin-llamacpp/src/state.rs:14-22`, `commands.rs:821-825` ("Router is already running.") | verified in source |
| The router PID is mirrored into an `AtomicU32` so a force-kill can still find it when the handle is borrowed | `state.rs:16-18` (comment), `commands.rs:1005-1024` (`force_kill_router_tree` falls back to `force_kill_router_tree_by_pid`) | verified in source |
| Crash and OOM classes are detected by string-matching child log lines, then surfaced to the UI as Tauri events | `router.rs:38-85` (`is_oom_line`, `is_backend_error_line`), `commands.rs:827-845` (emits `llamacpp-router-oom` / `llamacpp-router-backend-error`) | verified in source |
| Error-line classification is rate-limited to one event per 3s per stream | `router.rs:248-256` and `router.rs:297-305` | verified in source |
| `llama-server` binaries are downloaded at runtime from Jan's **own fork's** releases, with a CDN mirror | `src-tauri/plugins/tauri-plugin-llamacpp/src/backend.rs:1112` (`api.github.com/repos/janhq/llama.cpp/releases`), `:1222` (`github.com/janhq/llama.cpp/releases/download/...`), `:1124`/`:1226` (`catalog.jan.ai` fallback) | verified in source |
| CUDA runtime (`cudart`) tarballs are fetched separately per CUDA major (11.7 / 12.0 / 13.0) | `backend.rs:1246-1290` | verified in source |
| Windows backend variants enumerated: cpu, cuda-11/12/13, vulkan, hip (x64) and a separate `win-arm64` | `backend.rs:216-234` (`determine_supported_backends`) | verified in source |
| Models live at `<jan_data>/llamacpp/models/<modelId>/model.yml` (+ optional `model.gguf`, `mmproj.gguf`); backends at `<jan_data>/llamacpp/backends/<version>/<type>/build/bin/llama-server[.exe]` | `extensions/llamacpp-extension/src/index.ts:347-359` (layout comment) | verified in source |
| Per-model YAML is compiled into a single `router.preset.ini` consumed by `--models-preset` | `extensions/llamacpp-extension/src/preset.ts:1-9` (file doc), `preset.ts:537-538` (writes `<providerPath>/router.preset.ini` via a `.tmp` then rename) | verified in source |
| Preset changes can be hot-reloaded via `GET /models?reload=1` without restarting the process (requires upstream b9023+) | `src-tauri/plugins/tauri-plugin-llamacpp/src/commands.rs:911-937` (`reload_router_models`), gated in TS at `extensions/llamacpp-extension/src/index.ts:784-799` | verified in source |
| Backend build number gates argv spelling: `--no-webui` before upstream b9222, `--no-ui` from b9222 | `extensions/llamacpp-extension/src/index.ts:754-759`; `router.rs:130-133` + test `router.rs:664-669` keep the flag out of the base argv for exactly this reason | verified in source |
| Jan's user-facing API server is a **reverse proxy**, not a server that owns inference | `src-tauri/src/core/server/proxy.rs:3057-3116` (`start_server_internal`, reached from `start_server` at `:2969`: hyper `http1` + `service_fn(proxy_request)`), route dispatch at `proxy.rs:1570` | verified in source |
| Default API server endpoint is `127.0.0.1:1337` with prefix `/v1`; host may be widened to `0.0.0.0` | `web-app/src/hooks/useLocalApiServer.ts:58-64` (`serverHost: '127.0.0.1'`, `serverPort: 1337`, `apiPrefix: '/v1'`), `proxy.rs:3033-3040` | verified in source |
| Proxy auth is an optional bearer key that defaults to empty, plus a `trusted_hosts` host-header check that is bypassed entirely when bound to `0.0.0.0` | `useLocalApiServer.ts:84` (`apiKey: ''`), `:69` (`trustedHosts: []`), `proxy.rs:3033-3040` (`host == "0.0.0.0"` ⇒ `trusted_hosts = ["*"]`) | verified in source |
| The proxy also translates Anthropic `/messages` to OpenAI `/chat/completions` | `proxy.rs:222`, `proxy.rs:1571`, `proxy.rs:2692-2722` | verified in source |
| Only one proxy server instance may run | `proxy.rs:3024-3027` ("Server is already running") | verified in source |
| App exit tears the router down synchronously with a 10s MCP-cleanup budget | `src-tauri/src/lib.rs:406-423` (`ExitRequested` → `handle_graceful_exit`), `:454-474` (`RunEvent::Exit` blocks on `cleanup_llama_processes`) | verified in source |
| Closing the window hides to tray instead of quitting **only while the proxy server is running** (Windows/Linux); macOS always hides | `src-tauri/src/lib.rs:372-397` | verified in source |
| Single-instance is enforced by `tauri-plugin-single-instance` | `src-tauri/src/lib.rs:225` | verified in source |
| A separate `jan-cli` binary reads the same data folder and settings file without an `AppHandle` | `src-tauri/Cargo.toml:18-21` (`[[bin]] jan-cli`, `required-features = ["cli"]`), `src-tauri/src/core/cli/mod.rs:1-3`, `src-tauri/src/core/app/settings_store.rs:1-14` | verified in source |
| `jan-cli` defaults its own model server to port `6767`, separate from the desktop app's 1337 | `src-tauri/src/bin/jan-cli.rs:150-152`, `:78-80` | verified in source |
| Data folder resolution: `JAN_DATA_FOLDER` env → `settings.json` → `dirs::data_dir()/Jan/data` | `src-tauri/src/core/app/commands.rs:119-148` (`resolve_jan_data_folder`) | verified in source |
| Config is JSON, not YAML: `settings.json` holds serialized webview Zustand blobs, debounced 500ms to disk | `src-tauri/src/core/app/settings_store.rs:1-14`, `:26-28` (`FLUSH_DEBOUNCE`) | verified in source |
| Secrets go to the OS keyring, with an encrypted `provider_secrets.enc` fallback file | `settings_store.rs:6-7`, `src-tauri/src/core/app/constants.rs:32-34` | verified in source |
| Model downloads are first-party: Hugging Face repo/file APIs are called directly | `web-app/src/services/models/default.ts:105`, `:162-212`; a default embedding GGUF URL is hardcoded at `extensions/llamacpp-extension/src/index.ts:168` | verified in source |
| Windows packaging targets are NSIS + MSI, bundling `jan-cli.exe` as a resource and `bun`/`uv` as external binaries | `src-tauri/tauri.windows.conf.json:31-41` | verified in source |
| WebView2 is installed at runtime via silent download bootstrapper | `src-tauri/tauri.windows.conf.json:35-40` | verified in source |
| Jan is multi-backend by construction: 7 TS extensions and 6 Tauri plugins, incl. `mlx`, `rag`, `vector-db`, `websearch` | `extensions/` listing; `src-tauri/plugins/` listing; `src-tauri/src/core/server/commands.rs:41-52` (MLX session map) | verified in source |
| 525 Rust `#[test]`/`#[tokio::test]` functions and 276 TS/TSX test files exist | counted across `src-tauri/{src,plugins,utils}` and repo-wide `*.test.ts{,x}` @ `5f30aee` | verified in source |
| Router argv construction and readiness parsing are covered by in-file unit tests | `router.rs:648-774` (11 tests incl. `router_args_never_carries_the_api_key`, `is_ready_line_*`, `classifies_backend_crash_lines`) | verified in source |
| CI runs TS coverage (`yarn test:coverage`) and Rust coverage (`cargo-llvm-cov`) plus Playwright e2e, on Linux/macOS/Windows runners | `.github/workflows/jan-linter-and-test.yml` (jobs at lines 44, 84, 130, 171, 210, 259, 316) | verified in source |
| Windows CI includes a dedicated antivirus-tools runner matrix | `.github/workflows/jan-linter-and-test.yml:171` (`runs-on: windows-desktop-${{ matrix.antivirus-tools }}`) | verified in source |
| A Python-driven "autoqa" GUI-automation suite exists alongside the unit tests | `autoqa/` (`main.py`, `test_runner.py`, `screen_recorder.py`, `reportportal_handler.py`), `.github/workflows/autoqa-*.yml` | verified in source |
| The per-model session map that router mode replaced is being removed incrementally | `router.rs:5-6` ("This module is intentionally standalone — it does NOT touch the existing per-model session map"), yet no per-model spawn remains in the plugin (see spawn-site claim) | inferred from source |
| Router mode requires a `llama-server` new enough to support `--models-preset`; older backends are unusable in this path | no in-tree minimum-build assertion for `--models-preset` was found; only `MTP_MIN_BUILD = 9193` (`preset.ts:56`), b9023 for reload, and b9222 for `--no-ui` are gated | inferred from source |

## Process ownership

The Rust host process (Tauri app, or `jan-cli`) is the direct parent of one `llama-server` child. Ownership is a single `Mutex<Option<RouterHandle>>` in `LlamacppState` (`state.rs:14-22`); `start_router` refuses to run if that slot is occupied (`commands.rs:821-825`), so single-instance is a guard on the handle rather than a lock file or named mutex.

There is **no Job Object**. Containment relies on three weaker mechanisms:

1. `kill_on_drop(true)` on the tokio `Command` (`router.rs:187`) — covers ordinary drops, not a hard parent kill.
2. `CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW` creation flags on Windows (`system.rs:551-562`).
3. A `sysinfo` sweep that enumerates processes whose `parent()` is the router PID and kills them, router first so it "can't spawn new ones mid-sweep" (`router.rs:613-646`).

That sweep is one level deep — only direct children — so grandchildren are not reached. If the Jan host process is terminated without unwinding, nothing in-tree reclaims the `llama-server`. This is precisely the gap a Job Object with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` closes; see `docs/research/01-windows-process.md`.

Notably, `CREATE_NEW_PROCESS_GROUP` is set but then *cannot* be used for graceful shutdown, and `process.rs:30-34` documents why: `llama-server`'s console handler only reacts to `CTRL_C_EVENT`, and `GenerateConsoleCtrlEvent(CTRL_C_EVENT, pid)` is only deliverable with group ID 0 — which would kill the sender too — while `CTRL_BREAK_EVENT`, which *can* target a specific group, is ignored by the server. So on Windows Jan force-kills, and the new process group buys only console isolation.

## Runtime boundary

"Our code" is the Rust host plus the TS extension layer. The external binary is `llama-server[.exe]`, which Jan does **not** vendor and does **not** take from upstream `ggml-org/llama.cpp` releases. It is downloaded at runtime from Jan's own fork, `janhq/llama.cpp` (`backend.rs:1112`, `:1222`), with `catalog.jan.ai` as a mirror (`:1124`, `:1226`), landing in `<jan_data>/llamacpp/backends/<version>/<type>/build/bin/`. CUDA runtime DLLs are fetched as separate `cudart` tarballs per CUDA major (`backend.rs:1246-1290`).

The boundary is versioned and mutable at runtime: the TS layer parses the backend build number and changes its own behavior accordingly — `--no-webui` vs `--no-ui` at b9222 (`index.ts:754-759`), hot-reload availability at b9023 (`commands.rs:914-915`), MTP at b9193 (`preset.ts:56`). Jan therefore carries an explicit compatibility matrix against a moving external CLI.

Jan is genuinely multi-backend, not llama.cpp-only: `src-tauri/plugins/` holds `tauri-plugin-llamacpp`, `-mlx`, `-rag`, `-vector-db`, `-websearch`, `-hardware`; `extensions/` holds seven TS engines; and the proxy multiplexes local backends with remote cloud providers behind one endpoint.

## Model and config storage

```text
<jan_data>/                            # JAN_DATA_FOLDER → settings.json → dirs::data_dir()/Jan/data
├── llamacpp/
│   ├── models/<modelId>/model.yml     # per-model config (required)
│   │                    model.gguf    # optional, if downloaded by URL
│   │                    mmproj.gguf   # optional, vision projector
│   ├── router.preset.ini              # generated: compiled from all model.yml
│   ├── backends/<version>/<type>/build/bin/llama-server[.exe]
│   └── lib/                           # e.g. libcudart.so.12
├── models/  mlx/  openclaw/
├── threads/  assistants/
├── extensions/  logs/  .npx/  .uvx/
├── mcp_config.json
├── settings.json                      # zustand blobs, debounced 500ms
└── provider_secrets.enc               # keyring fallback
```

Sources: `extensions/llamacpp-extension/src/index.ts:347-359`, `src-tauri/src/core/app/constants.rs:14-51`, `preset.ts:537`, `commands.rs:119-148`.

Config is **JSON, not YAML**, at the app level, and it is not hand-authored: `settings.json` is a map of namespace → serialized webview Zustand store blob (`settings_store.rs:1-14`). YAML appears only per-model (`model.yml`), and that YAML is not what the backend reads — it is compiled into `router.preset.ini`, written atomically via `.tmp` + rename (`preset.ts:537-538`). Model downloads are first-party against the Hugging Face API (`web-app/src/services/models/default.ts:105-212`).

## API surface

Two distinct HTTP surfaces, and only the outer one is stable:

| | Jan API server | llama.cpp router |
| --- | --- | --- |
| Code | `src-tauri/src/core/server/proxy.rs` | external `llama-server` |
| Bind | `127.0.0.1:1337`, prefix `/v1` (`useLocalApiServer.ts:58-64`) | `127.0.0.1:<random>` (`router.rs:126`, `index.ts:714`) |
| Auth | optional bearer, default `''` | `LLAMA_API_KEY` env, HMAC-derived |
| Role | reverse proxy / multiplexer | actual inference |

The outer server is a hyper `http1` + `service_fn` proxy (`proxy.rs:3057-3116`) that dispatches on `(method, path)` (`proxy.rs:1570`) across `POST /chat/completions`, `GET /models`, and Anthropic `POST /messages` — the last translated into OpenAI shape (`proxy.rs:222`, `:2692-2722`). It fronts both the local router and remote cloud providers.

Two auth details are worth flagging as facts. The default API key is the empty string (`useLocalApiServer.ts:84`) and `trustedHosts` defaults to `[]` (`:69`). And when the user selects `0.0.0.0`, the host-header allowlist is replaced with `["*"]` outright (`proxy.rs:3033-3040`) — the code comments this as intentional so LAN clients aren't rejected, but the effect is that widening the bind also disables that check.

The router's own key never appears in argv, only in `LLAMA_API_KEY` (`router.rs:106-113`), with a unit test asserting the invariant (`router.rs:721-729`). The rationale given in-tree is that argv is world-readable via Task Manager/`ps` *and* that Jan logs the full argv at startup (`router.rs:173`).

## Lifecycle

**Start**: the TS caller generates `router.preset.ini`, picks a random free port, and derives the API key (`index.ts:703`, `:714-715`; the HMAC-SHA256 derivation itself is `commands.rs:739-747`), then passes all three into `start_router` (`router.rs:150-376`), which spawns with stdout+stderr piped → spawns two reader tasks → `try_wait()` early-exit check → `select!` loop polling every 50ms until a ready line arrives or the 60s (default) timeout expires. On timeout or early exit the child is killed and buffered stderr is folded into the error (`router.rs:319-326`, `:347-364`).

Readiness is **log-line scraping, not HTTP polling**. `is_ready_line` (`router.rs:30-36`) matches five phrasings, and the regression test at `router.rs:696-704` records the cost of that coupling: router mode logs `llama_server: listening on ...` with a colon, which the older colon-free pattern missed, so startup hung for the full timeout while the server was already serving. Compare WinServeAI's approach in `docs/research/02-llama-readiness.md`.

**Stop** (`router.rs:396-482`) is an HTTP-mediated drain rather than a signal: `GET /models` for loaded/loading models → if any slot reports `is_processing`, bail out and return the busy list to the caller → otherwise `POST /models/unload` per model → poll until clear → terminate. `stop_router` wraps this with a 10s deadline and force-kills the tree on expiry (`router.rs:380-393`). Termination itself is SIGTERM→5s→SIGKILL on Unix (`process.rs:1-26`) and unconditional force-kill on Windows (`process.rs:28-60`).

**Crash** detection is also string-matching on child output. `is_oom_line` and `is_backend_error_line` (`router.rs:38-85`) classify CUDA errors, `GGML_ASSERT`, Vulkan/Metal errors, device-loss, uncaught C++ `terminate`, and glibc heap-corruption aborts; matches fire a Tauri event (`llamacpp-router-oom` / `llamacpp-router-backend-error`) and trigger an unload sweep (`commands.rs:827-845`), rate-limited to one per 3s per stream. The comments state the motivation plainly: without classifying SIGABRT-class lines "the crash is silent and the model load appears to hang/loop forever" (`router.rs:69-72`).

**Single-instance** is enforced at two levels: `tauri-plugin-single-instance` for the app (`lib.rs:225`) and the occupied-handle check for the router (`commands.rs:823`).

## Desktop vs service split

There is no OS service. Three surfaces share one data folder:

- **Desktop app** — Tauri host + React webview, tray icon (`lib.rs:334-335`, `setup.rs:247-263`). On Windows/Linux, closing the window quits *unless* the proxy server is running, in which case it hides to tray; the comment notes the router alone is deliberately not a reason to stay resident (`lib.rs:384-396`). macOS always hides.
- **`jan-cli`** — separate binary behind the `cli` feature (`Cargo.toml:18-21`), bundled as a Windows resource (`tauri.windows.conf.json:33`). It reuses core logic through an adapter layer with no `AppHandle` (`core/cli/mod.rs:1-3`) and reads the same `settings.json`; the app flushes the debounced settings writer on exit so the CLI never sees a stale file (`lib.rs:427-429`, `settings_store.rs:11-14`). Subcommands: `serve`, `launch`, `threads`, `models` (`jan-cli.rs:55-112`), with its own default port `6767` and a `--detach` mode (`jan-cli.rs:150-152`, `:177-179`).
- **Mobile** — Android/iOS Tauri configs exist, and the server port defaults to `0` (auto-assign) there (`useLocalApiServer.ts:60-61`).

## Packaging

| Platform | Targets | Evidence |
| --- | --- | --- |
| Windows | `nsis`, `msi`; bundles `resources/bin/jan-cli.exe`; external bins `bun`, `uv`; WebView2 via silent download bootstrapper | `src-tauri/tauri.windows.conf.json:31-41` |
| Linux | per-platform conf + Flatpak manifest and Flathub metadata | `src-tauri/tauri.linux.conf.json`, `flatpak/ai.jan.Jan.yml`, `flatpak/flathub.json` |
| macOS / Android / iOS | dedicated Tauri confs | `src-tauri/tauri.{macos,android,ios}.conf.json` |
| Portable | dedicated manual workflow | `.github/workflows/manual-build-portable.yml` |

Build orchestration is a root `Makefile` plus ~10 reusable `template-tauri-build-*` workflows, including "external" variants for fork PRs. Crucially, the inference binary is **not** in the installer — it is a runtime download (see Runtime boundary), so a fresh install cannot serve until it fetches a backend.

## Test strategy

- **Rust unit tests, 525 functions** across `src-tauri/{src,plugins,utils}`. The router module is the relevant example: 11 in-file tests at `router.rs:648-774` covering argv construction (`router_args_contains_required_flags`), the no-key-in-argv invariant (`router_args_never_carries_the_api_key`), all five readiness phrasings plus negative cases, `models_max = 0` passthrough, and crash-line classification. `evaluate_load_poll` is factored out as a pure function specifically to be testable (`commands.rs:411`, tests at `:1026`).
- **TS/TSX, 276 test files** under Vitest (`vitest.config.ts`), including `extensions/llamacpp-extension/src/{preset,util}.test.ts` and `src/test/{backend,index,migrateLegacyModels,no-localstorage,settings-store}.test.ts`.
- **CI** (`.github/workflows/jan-linter-and-test.yml`) runs TS coverage and Rust `cargo-llvm-cov` with base-branch coverage diffing, plus Playwright e2e, across Linux/macOS/Windows. One Windows job runs on a self-hosted matrix parameterized by antivirus product (`:171`) — a Windows-specific concern WinServeAI shares.
- **AutoQA**: a Python GUI-automation suite (`autoqa/main.py`, `test_runner.py`, `screen_recorder.py`, ReportPortal integration) driven by `autoqa-*.yml` workflows.

What the tests **do** prove: argv shape, readiness-line parsing, and the API-key-placement invariant. What no in-tree test appears to cover: that the `llama-server` child is actually reclaimed when the parent dies abnormally. The kill paths (`force_kill_router_tree`, `graceful_terminate_process`) have no unit tests — consistent with there being no Job Object to assert on. (inferred from source)

## Layout (text diagram)

```text
                    ┌─────────────────────────────────────┐
   OpenAI clients ──▶│ Jan API server  127.0.0.1:1337 /v1 │  proxy.rs (hyper)
   (LAN if 0.0.0.0)  │ reverse proxy + Anthropic translate │  optional bearer, default ''
                    └──────────┬──────────────┬───────────┘
                               │              └──────────▶ remote cloud providers
                               ▼
  ┌────────────────────────────────────────────────────────────────┐
  │ Tauri host (Rust)                              src-tauri/src/  │
  │   single-instance · tray · ExitRequested drain                 │
  │                                                                │
  │   ┌──────────────────────────────────────────────────────┐     │
  │   │ tauri-plugin-llamacpp                                │     │
  │   │   LlamacppState { Mutex<Option<RouterHandle>>,       │     │
  │   │                   AtomicU32 router_pid }             │     │
  │   │   start/stop_router · load/unload_llama_model        │     │
  │   └──────────────────────────────────────────────────────┘     │
  │   tauri-plugin-{mlx,rag,vector-db,websearch,hardware}          │
  └──────────────────────┬─────────────────────────────────────────┘
                         │ spawn (ONE child) — router.rs:212
                         │ CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP
                         │ kill_on_drop(true) — NO Job Object
                         │ LLAMA_API_KEY via env, never argv
                         ▼
  ══════════════════ external binary boundary ══════════════════════
  ┌────────────────────────────────────────────────────────────────┐
  │ llama-server  --models-preset router.preset.ini                │
  │               --models-max N --host 127.0.0.1 --port <random>  │
  │   ROUTER MODE: one process, N models loaded on demand          │
  │   /models  /models/load  /models/unload  /slots  /v1/*         │
  │   downloaded from janhq/llama.cpp releases (fork, not upstream)│
  └────────────────────────┬───────────────────────────────────────┘
                           │ may fork children → reaped only by a
                           │ one-level sysinfo parent-PID sweep
                           ▼
                    (model worker processes)

  readiness : scrape child stdout/stderr for is_ready_line(), 60s default
  crash     : scrape for OOM / CUDA / GGML_ASSERT / glibc-abort strings
  stop      : GET /models → POST /models/unload → poll /slots → terminate
              (10s deadline, then force-kill PID tree)
  windows   : NO graceful stop — force-kill only (process.rs:30-34)

  webview (React, web-app/)   ──▶ Tauri invoke ──▶ plugins
  jan-cli (separate binary)   ──▶ same <jan_data> + settings.json, port 6767
```

## Relevance to WinServeAI (facts only)

WinServeAI is a Windows llama.cpp appliance wrapper with one `ServerManager` and no multi-backend layer (`AGENTS.md`). Jan is a multi-backend desktop chat product. The comparable surface is narrow — Jan's `tauri-plugin-llamacpp` versus our `runtime/` + `server/manager.rs` — and everything below is scoped to that.

**Steal — Windows shutdown analysis (`process.rs:30-34`, verified in source).** The most directly reusable artifact is a negative result Jan paid for: `llama-server`'s console handler responds only to `CTRL_C_EVENT`; `GenerateConsoleCtrlEvent(CTRL_C_EVENT, pid)` is deliverable only with group 0, which would kill the sender; and `CTRL_BREAK_EVENT`, which *can* target one group, is ignored by the server. Jan's conclusion is that graceful console-signal shutdown of `llama-server` on Windows is not achievable, so it force-kills. This is a concrete constraint for `docs/research/01-windows-process.md` and means a WinServeAI graceful-stop design cannot rest on console signals.

**Steal — HTTP-mediated drain before kill (`router.rs:396-482`, verified in source).** Because signals are unavailable, Jan drains over HTTP instead: enumerate loaded models, refuse to stop while `/slots` reports `is_processing`, request unload, poll, then terminate, with a hard 10s deadline that escalates to force-kill. The pattern — bounded graceful attempt with an unconditional escalation — maps onto `ServerManager` stop semantics without importing any of Jan's structure.

**Steal — keep the API key out of argv (`router.rs:106-113`, test at `:721-729`, verified in source).** Jan passes `LLAMA_API_KEY` by environment and asserts in a unit test that argv never contains it, for two stated reasons: argv is readable by any process via Task Manager, and Jan logs full argv at startup (`router.rs:173`). If WinServeAI ever logs argv from `runtime/llama.rs`, the same split applies.

**Steal — pin readiness detection with a regression test (`router.rs:696-712`, verified in source).** Jan matches five different ready-line phrasings because upstream reworded the line; the test docstring records a real hang where a colon in `llama_server: listening on` defeated the older pattern. Whatever `docs/research/02-llama-readiness.md` settles on, the failure mode is documented upstream evidence that log-scraping readiness needs versioned test coverage.

**Note — Jan has no Job Object (`system.rs:551-562`, `router.rs:588-646`, verified in source).** Containment is `kill_on_drop` + `CREATE_NEW_PROCESS_GROUP` + a one-level `sysinfo` parent-PID sweep. Grandchildren are not reached and nothing reclaims the child if the host dies abnormally. Jan is therefore evidence of the gap rather than a solution to it; the Job Object work in `docs/research/01-windows-process.md` has no counterpart here to copy.

**Note — Windows llamacpp is `x86_64`-gated (`process.rs:28`, `system.rs:552`, verified in source).** On Windows ARM64 both the creation flags and the force-kill helper compile out, leaving a bare `kill()` (`router.rs:578-582`), even though `win-arm64` is an advertised backend (`backend.rs:234`). A relevant data point for WinServeAI's target-matrix decision.

**Avoid — the runtime binary download (`backend.rs:1112-1290`, verified in source).** Jan fetches `llama-server` from its own `janhq/llama.cpp` fork plus a CDN mirror, and separately fetches CUDA runtime tarballs per CUDA major. The consequences are visible in-tree: a build-number compatibility matrix in the TS layer (b9023 reload, b9193 MTP, b9222 `--no-ui`), argv spellings that must be chosen at runtime, and an installer that cannot serve until it downloads. WinServeAI's `bin/llama-server.exe` as a fixed, explicit external boundary avoids all of it. Per `AGENTS.md`, model downloads stay out of scope.

**Avoid — the proxy-in-front-of-everything design (`proxy.rs`, verified in source).** Jan's 1337 endpoint is a 3,532-line reverse proxy that multiplexes local backends and remote providers and translates Anthropic to OpenAI. WinServeAI passes through to `llama-server`'s own `/v1` (`app/src/api/`), so none of that layer is applicable. Two facts from it are still worth recording: the default API key is `''` (`useLocalApiServer.ts:84`), and selecting `0.0.0.0` replaces the host-header allowlist with `["*"]` (`proxy.rs:3033-3040`).

**Open question — router mode vs one-process-per-model.** The single largest architectural difference from WinServeAI's current shape is that Jan v0.8.4 no longer spawns a process per model. One `llama-server` runs with `--models-preset` + `--models-max`, models are loaded and unloaded over its HTTP API, and preset edits hot-reload via `GET /models?reload=1` without a restart (`router.rs:1-6`, `commands.rs:911-937`; verified in source). Jan's own module docstring calls this "Phase 1 of the router refactor," and no per-model spawn site remains in the plugin. Whether `ServerManager` should own one long-lived process with a generated preset instead of a process per configuration is a design question for `docs/research/05-server-manager.md`; the requirement it implies is a `llama-server` build new enough to accept `--models-preset`, which no in-tree assertion pins (inferred from source).

**Note — process-ownership discipline is achievable in ~800 lines.** Jan's entire router lifecycle — spawn, readiness, crash classification, graceful drain, force-kill — is `router.rs` at 774 lines plus `process.rs` at 60, with 11 unit tests, one spawn site, and one state mutex. It is a useful scale reference for `ServerManager` and does not require the surrounding plugin architecture.

---

## Sources

```json
{
  "product": "Jan",
  "pinned": "v0.8.4 @ 5f30aee467f08941964a83f946e2663e7ae0e01f",
  "date_verified": "2026-07-30",
  "urls": [
    "https://github.com/janhq/jan",
    "https://api.github.com/repos/janhq/jan/tags",
    "https://api.github.com/repos/janhq/jan/releases/latest",
    "https://github.com/janhq/jan/releases/tag/v0.8.4",
    "https://github.com/janhq/jan/commit/5f30aee467f08941964a83f946e2663e7ae0e01f",
    "https://github.com/janhq/jan/blob/v0.8.4/LICENSE",
    "https://github.com/janhq/jan/blob/v0.8.4/src-tauri/plugins/tauri-plugin-llamacpp/src/router.rs",
    "https://github.com/janhq/jan/blob/v0.8.4/src-tauri/plugins/tauri-plugin-llamacpp/src/process.rs",
    "https://github.com/janhq/jan/blob/v0.8.4/src-tauri/plugins/tauri-plugin-llamacpp/src/state.rs",
    "https://github.com/janhq/jan/blob/v0.8.4/src-tauri/plugins/tauri-plugin-llamacpp/src/commands.rs",
    "https://github.com/janhq/jan/blob/v0.8.4/src-tauri/plugins/tauri-plugin-llamacpp/src/cleanup.rs",
    "https://github.com/janhq/jan/blob/v0.8.4/src-tauri/plugins/tauri-plugin-llamacpp/src/backend.rs",
    "https://github.com/janhq/jan/blob/v0.8.4/src-tauri/plugins/tauri-plugin-llamacpp/src/path.rs",
    "https://github.com/janhq/jan/blob/v0.8.4/src-tauri/plugins/tauri-plugin-llamacpp/src/lib.rs",
    "https://github.com/janhq/jan/blob/v0.8.4/src-tauri/utils/src/system.rs",
    "https://github.com/janhq/jan/blob/v0.8.4/src-tauri/src/lib.rs",
    "https://github.com/janhq/jan/blob/v0.8.4/src-tauri/src/core/setup.rs",
    "https://github.com/janhq/jan/blob/v0.8.4/src-tauri/src/core/server/proxy.rs",
    "https://github.com/janhq/jan/blob/v0.8.4/src-tauri/src/core/server/commands.rs",
    "https://github.com/janhq/jan/blob/v0.8.4/src-tauri/src/core/app/commands.rs",
    "https://github.com/janhq/jan/blob/v0.8.4/src-tauri/src/core/app/constants.rs",
    "https://github.com/janhq/jan/blob/v0.8.4/src-tauri/src/core/app/settings_store.rs",
    "https://github.com/janhq/jan/blob/v0.8.4/src-tauri/src/core/cli/mod.rs",
    "https://github.com/janhq/jan/blob/v0.8.4/src-tauri/src/bin/jan-cli.rs",
    "https://github.com/janhq/jan/blob/v0.8.4/src-tauri/Cargo.toml",
    "https://github.com/janhq/jan/blob/v0.8.4/src-tauri/tauri.conf.json",
    "https://github.com/janhq/jan/blob/v0.8.4/src-tauri/tauri.windows.conf.json",
    "https://github.com/janhq/jan/blob/v0.8.4/extensions/llamacpp-extension/src/index.ts",
    "https://github.com/janhq/jan/blob/v0.8.4/extensions/llamacpp-extension/src/preset.ts",
    "https://github.com/janhq/jan/blob/v0.8.4/web-app/src/hooks/useLocalApiServer.ts",
    "https://github.com/janhq/jan/blob/v0.8.4/web-app/src/services/models/default.ts",
    "https://github.com/janhq/jan/blob/v0.8.4/.github/workflows/jan-linter-and-test.yml",
    "https://github.com/janhq/jan/tree/v0.8.4/autoqa",
    "https://github.com/janhq/jan/tree/v0.8.4/flatpak",
    "https://github.com/janhq/llama.cpp",
    "https://api.github.com/repos/janhq/llama.cpp/releases",
    "https://catalog.jan.ai/llama.cpp/releases/releases.json",
    "https://huggingface.co"
  ]
}
```
