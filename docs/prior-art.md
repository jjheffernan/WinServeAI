# Prior Art & Research Notes

Research snapshot for scaffolding WinServeAI. Prefer patterns that reinforce **Server Manager owns the process**, **no chat UI**, **no model marketplace**, and **backend-agnostic config**.

## Summary (what matters for WinServeAI)

1. **Closest peers are llama-server wrappers**, not Ollama/LM Studio. Several small GitHub projects already supervise `llama-server.exe` on Windows (health checks, restart, WinUI shells). Steal process/health patterns; avoid their model-download and multi-chat surfaces.
2. **Ollama / LocalAI / LM Studio define the market**, but they optimize for model management or chat. WinServeAI’s wedge is the opposite: one-click OpenAI endpoint, user-supplied GGUF path, stable bundled backend.
3. **Windows graceful shutdown is hard.** POSIX signals do not apply. Use `CREATE_NEW_PROCESS_GROUP` + `CTRL_BREAK_EVENT` / `GenerateConsoleCtrlEvent`, or a documented HTTP shutdown if llama-server exposes one. Do not rely on `kill()` alone.
4. **Hardware detection on Windows should prefer DXGI for VRAM**, not NVML alone. Under WDDM, NVML per-process VRAM is often `NOT_AVAILABLE`; `IDXGIAdapter3::QueryVideoMemoryInfo` is the reliable path. Device-wide NVML / `nvidia-smi` remain useful fallbacks.
5. **llama.cpp is MIT** — bundling `llama-server` in an installer is fine if copyright/license notices ship with the product. **Model licenses are separate** and out of scope (we do not download models).
6. **Sleep/wake on Windows** can leave zombie `llama-server` instances. Prior art uses Task Scheduler + kill-by-port/name before restart ([llama.cpp discussion #20648](https://github.com/ggml-org/llama.cpp/discussions/20648)).

## Similar products

| Name | URL | Steal | Avoid |
| --- | --- | --- | --- |
| **llama.cpp `llama-server`** | https://github.com/ggml-org/llama.cpp | OpenAI-compatible `/v1`, metrics, release binaries, pin build tags (`b####`) | Treating CLI flags as user-facing config |
| **llama-cpp-windows-manager** | https://github.com/alekk89/llama-cpp-windows-manager | Supervised `llama-server` sessions, per-model ports, logs/metrics UI separation | Runtime download ecosystem, multi-session gateway complexity for v1 |
| **Llama Server WinUI** | https://github.com/ottomixa/llama-server-winui | WinUI 3 “runtime manager” pattern, engine variants (CUDA/Vulkan/CPU), MVVM | Hardcoded ports, model/engine download UI as core product |
| **Llama-Orchestrator** | https://github.com/michaelprinc/Llama-Orchestrator | Health policies, auto-restart/backoff, NSSM/Task Scheduler integration | Multi-instance orchestration before single-instance is solid |
| **localmodelrouter** | https://github.com/g023/localmodelrouter | Process manager + health monitor + graceful shutdown design | HF downloads, Ollama API surface, multi-model VRAM eviction (Phase 5+) |
| **llamaman / llama-server-manager** | https://github.com/cmoro-deusto/llamaman, https://github.com/zero4281/llama-server-manager | Small manager CLIs as reference for arg building | GPL managers if we want MIT-only deps; feature sprawl |
| **Ollama** | https://ollama.com / https://github.com/ollama/ollama | Daemon mental model, simple local API, developer adoption | Model pull/library, non-OpenAI-primary API, opaque backend |
| **LocalAI** | https://localai.io / https://github.com/mudler/LocalAI | True OpenAI drop-in positioning, backend plugins | Multi-modal scope, Docker/K8s-first, config complexity |
| **LM Studio** | https://lmstudio.ai | Polished “start local server” UX, Windows-native feel | Closed source, chat-first, model browser, desktop-only headless limits |
| **Jan** | https://github.com/janhq/jan | Tauri + Rust spawning llama with `--n-gpu-layers=` form | Full chat/extension product surface |
| **KoboldCpp** | https://github.com/LostRuins/koboldcpp | Single-binary simplicity | Creative-writing UI, not infrastructure-shaped |
| **text-generation-webui** | https://github.com/oobabooga/text-generation-webui | Power-user launch flags reference | Python/Gradio stack, chat UI |
| **NSSM** | https://nssm.cc | Windows service install patterns for long-running servers | Shipping NSSM as a hard dependency for v1 (optional Phase 5) |

**Positioning takeaway:** WinServeAI sits between “raw llama-server” and “LM Studio”: native installer + dumb UI + Server Manager, without becoming a model hub.

## GitHub references

### Upstream

- [ggml-org/llama.cpp](https://github.com/ggml-org/llama.cpp) — MIT; prebuilt Windows CUDA/Vulkan/CPU releases; `llama-server` is the v1 backend.
- [llama.cpp releases](https://github.com/ggml-org/llama.cpp/releases) — pin a `b####` build per WinServeAI stable release (`vendor/llama.cpp/VERSION`).
- [llama.cpp discussion #20648](https://github.com/ggml-org/llama.cpp/discussions/20648) — Win11 sleep/wake duplicate instances; kill-by-port + Task Scheduler pattern.

### Managers / orchestrators (study process code)

- [alekk89/llama-cpp-windows-manager](https://github.com/alekk89/llama-cpp-windows-manager) — Windows control panel over `llama-server`.
- [ottomixa/llama-server-winui](https://github.com/ottomixa/llama-server-winui) — WinUI 3 runtime manager.
- [michaelprinc/Llama-Orchestrator](https://github.com/michaelprinc/Llama-Orchestrator) — health, restart, NSSM.
- [g023/localmodelrouter](https://github.com/g023/localmodelrouter) — `process.py` health monitor + graceful shutdown (Python, but architecture maps to `packages/process` + `packages/launcher`).
- [janhq/jan](https://github.com/janhq/jan) — Tauri app; note `--n-gpu-layers=N` (equals form) to avoid argv parsing pitfalls.

### Hardware / VRAM (Rust)

- [PCfVW/hypomnesis](https://github.com/PCfVW/hypomnesis) / [crates.io/hypomnesis](https://crates.io/crates/hypomnesis) — DXGI + NVML + `nvidia-smi` fallback; documents WDDM NVML limitations.
- [Microsoft: IDXGIAdapter3::QueryVideoMemoryInfo](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_4/nf-dxgi1_4-idxgiadapter3-queryvideomemoryinfo) — canonical Windows VRAM budget/usage API.

### Installer / firewall (patterns, not code)

- Inno Setup firewall examples and `netsh advfirewall` rules are common for LAN-bound local servers; prefer localhost default in examples (`examples/config/localhost.yaml`) and optional LAN bind with explicit firewall step in installer.

## Stack Exchange / Q&A

| Thread | Why it matters |
| --- | --- |
| [Can I send a ctrl-C (SIGINT) to an application on Windows?](https://stackoverflow.com/questions/813086/can-i-send-a-ctrl-c-sigint-to-an-application-on-windows) | `AttachConsole` + `GenerateConsoleCtrlEvent` patterns for graceful stop |
| [How to send CTRL+C/SIGINT to a subprocess on Windows?](https://stackoverflow.com/questions/40762545/how-to-send-a-ctrlc-sigint-to-a-subprocess-in-windows) | `CREATE_NEW_PROCESS_GROUP`; prefer `CTRL_BREAK_EVENT` (cannot be disabled) |
| [subprocess CTRL_C_EVENT does not work](https://stackoverflow.com/questions/69121177/subprocess-popen-send-signalctrl-c-event-does-not-work) | Process groups ignore Ctrl+C by default — implement carefully in `packages/process` |
| [Gracefully terminate a process on Windows](https://stackoverflow.com/questions/55092139/gracefully-terminate-a-process-on-windows) | Confirms Windows needs console control events or an app-level shutdown API |

**Scaffolding implication:** implement `ManagedProcess::graceful_stop` with a Windows-specific path (console control event + timeout + force kill), not a Unix-only signal.

## YouTube

| Video | Why relevant |
| --- | --- |
| [The Ultimate Local LLM Setup: llama.cpp + VS Code + Continue on Windows 11](https://www.youtube.com/watch?v=b1E7Au5Awbs) | End-user flow: start `llama-server`, point OpenAI-compatible client at `/v1`; CUDA/memory knobs |
| [Tutorial: Local AI Agent with llamacpp and OpenCode on Windows](https://www.youtube.com/watch?v=j1AskNNRpX0) | Using official Windows release zips; OpenAI base URL configuration |

Written companion (not video, same workflow): [OpenWebUI + llama.cpp on Windows](https://dev.to/christian35620/run-your-own-local-ai-chat-with-openwebui-and-llamacpp-windows-1k6c) — shows the **UI is optional**; WinServeAI only needs the `llama-server` half.

## X / Twitter

Sparse for this niche. Useful follows when researching release cadence and breakage:

- [@ggerganov](https://x.com/ggerganov) — llama.cpp upstream
- Occasional threads from LM Studio / Ollama accounts (product UX, not architecture)

Treat X as signal for **breaking CLI changes** and release announcements, not as a primary design source. Prefer GitHub releases + discussions.

## Licensing notes (bundling)

| Component | License | Action |
| --- | --- | --- |
| llama.cpp / llama-server | MIT | Ship copyright + MIT text in installer (`THIRD-PARTY-NOTICES`) |
| WinServeAI code | MIT | Already in `LICENSE` |
| User GGUF models | Varies | Do not redistribute; user supplies `model.path` |
| CUDA runtime DLLs | NVIDIA EULA | If bundling CUDA builds, follow NVIDIA redistribution rules for redistributable DLLs only |

Add `docs/licensing.md` (or a section in release-process) when packaging begins.

## Scaffolding recommendations (prioritized, YAGNI)

Concrete additions that fit the existing monorepo — do **not** implement chat, downloads, or multi-model routers yet.

### P0 — Phase 1 engine (do next)

1. **`packages/process` Windows stop path** — `CREATE_NEW_PROCESS_GROUP` on spawn; `CTRL_BREAK_EVENT` / `GenerateConsoleCtrlEvent`; timeout then force kill. Document in `docs/backend.md`.
2. **`packages/api` readiness probe** — poll `GET /health` or `GET /v1/models` with timeout; wire into `ServerManager::start`.
3. **`vendor/llama.cpp/VERSION`** — pin format (`b####` + release asset name for Windows CUDA).
4. **Port conflict check** — before start, bind-test `server.port`; clear error to UI/logs.
5. **Sleep/wake note** — detect unexpected exit / duplicate bind; restart policy stub (no auto-restart until Phase 4, but detect and surface `Crashed`).

### P1 — Hardware auto config

6. **`packages/hardware` DXGI path** — enumerate adapters + VRAM via `windows` crate / DXGI; optional NVML for CUDA capability; `nvidia-smi` fallback. Evaluate `hypomnesis` before writing custom FFI.
7. **Auto `gpu_layers`** — map VRAM + model file size heuristic (document formula; keep simple).
8. **CPU-only path** — `gpu_layers: 0` with explicit log warning (graceful limited support).

### P2 — Desktop / installer scaffolding

9. **Tauri commands = thin Server Manager RPC** — `start`, `stop`, `status`, `logs`, `get_config`, `set_config` only (matches `apps/desktop/README.md`).
10. **Inno Setup research spike** — firewall rule only when `host != 127.0.0.1`; shortcut; install dir layout: `bin/llama-server.exe`, `config/default.yaml`.
11. **`THIRD-PARTY-NOTICES` template** under `assets/` or `docs/`.

### P3 — Docs / ADR only (no code)

12. **ADR: pin llama.cpp release policy** — how often to bump, validation matrix.
13. **ADR: Windows graceful shutdown approach** — console control vs HTTP shutdown.
14. **Link this file** from `docs/research.md` and `docs/development.md`.

### Explicitly defer (seen in prior art, out of MVP)

- Model download / HF integration
- Multi-instance / multi-model VRAM eviction
- Ollama-compatible API surface
- NSSM Windows service (optional Phase 5)
- Chat UI, Open WebUI bundling
- Auto-update of llama.cpp from GitHub inside the app (updater app is Phase 5)

## Open research questions

1. Does current `llama-server` expose a reliable admin shutdown endpoint, or is console control the only graceful path?
2. Which Windows release asset (CUDA version) do we pin for the hardware matrix (3060–4090)?
3. Inno Setup vs WiX for firewall + Start Menu — confirm Inno for v0.4.
4. Is `hypomnesis` acceptable as a dependency, or do we want zero GPU crates and shell out to `nvidia-smi` only for v0.2?
5. Tray icon in Phase 2 or Phase 5? (Prior art often ships tray early; our roadmap says Phase 5.)
6. How do we version-detect `llama-server` CLI flag drift across `b####` builds?

## Sources checklist

- [x] GitHub (upstream, managers, Jan, LocalAI/Ollama positioning)
- [x] Stack Overflow (Windows process control)
- [x] YouTube (Windows llama-server + OpenAI client setup)
- [x] X/Twitter (sparse; upstream accounts only)
- [ ] Deeper code read of `localmodelrouter` process manager (recommended before implementing restart policy)
- [ ] Confirm llama-server `/health` contract for pinned release
