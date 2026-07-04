# Prior Art & Research Notes

Research snapshot for WinServeAI scaffolding (process ownership, hardware, installers, runtime). Not a product roadmap. Prefer links and concrete patterns over marketing claims.

Related internal docs: [architecture.md](./architecture.md), [vision.md](./vision.md), [research.md](./research.md) (checklist), [installer.md](./installer.md), [backend.md](./backend.md).

> **Path note (appliance layout):** This note was written against an earlier `packages/*` monorepo. Map old paths to the current single crate:
> `packages/launcher` → `app/src/server/manager.rs` · `packages/process` → `app/src/runtime/process.rs` · `packages/llama` → `app/src/runtime/llama.rs` · `packages/api` / readiness → `app/src/server/health.rs` + `app/src/api/` · `packages/hardware` → `app/src/system/` · `packages/config` → `app/src/server/config.rs` · `packages/logging` → `app/src/server/logs.rs` · `packages/backend` / traits → **removed** (no backend trait).

---

## Summary (what matters for WinServeAI)

WinServeAI’s niche is **Windows-native, headless, OpenAI-compatible inference with one process owner** — not chat, not model marketplaces, not Docker-first infra.

| Area | Steal | Avoid |
| --- | --- | --- |
| Positioning | Ollama’s “API is the product”; llama-server’s `/v1` surface | LM Studio / Jan / GPT4All chat-first UX; LocalAI’s multi-backend kitchen sink |
| Process ownership | Windows **Job Objects** (`KILL_ON_JOB_CLOSE`); single Server Manager | UI spawning `llama-server`; NSSM-as-product (optional later, not MVP) |
| Readiness | Poll **`GET /v1/models`** until 200 (`app/src/server/health.rs`); optional alternate `GET /health` (503 loading / 200 ready) when present | Treating process start as “ready”; short timeouts under load; treating `/health` as the only probe |
| Hardware | DXGI for Windows VRAM; NVML for NVIDIA totals; leave auto-fit to llama.cpp `--fit` | Reimplementing layer math; trusting NVML per-process under WDDM |
| Installer | Inno Setup + `netsh advfirewall` (program or port rule); uninstall deletes rule | Opening firewall for localhost-only binds |
| Licensing | Ship MIT notice for llama.cpp **with binaries** | Bundling models; omitting third-party notices (Ollama got called out for this) |
| Runtime | Single-crate appliance: spawn `bin/llama-server.exe` only; no backend trait | LocalAI-scale polyglot backends; reintroducing `packages/*` |

**Closest analogues:** [llama-server](https://github.com/ggml-org/llama.cpp/tree/master/tools/server) (engine), [Ollama](https://github.com/ollama/ollama) (daemon/API ergonomics), [KoboldCpp](https://github.com/LostRuins/koboldcpp) (portable Windows binary + OpenAI `/v1`), small managers like [llama-cpp-windows-manager](https://github.com/alekk89/llama-cpp-windows-manager) / [llama-server-manager](https://github.com/zero4281/llama-server-manager) (lifecycle wrappers). WinServeAI should be **narrower** than all of them: no model pull, no chat UI, no multi-engine zoo.

---

## Similar products (table: name, URL, what to steal, what to avoid)

| Name | URL | Steal | Avoid |
| --- | --- | --- | --- |
| **llama.cpp / llama-server** | https://github.com/ggml-org/llama.cpp | OpenAI-compatible `/v1`; readiness via `/v1/models` (product) and optional `/health` (503 loading / 200 ready); `--fit` auto VRAM; pin release builds under `bin/` | Shipping the built-in web UI as product surface; exposing every CLI flag in YAML |
| **Ollama** | https://github.com/ollama/ollama · https://ollama.com | API-first daemon; sensible defaults; Windows service patterns in community (NSSM wrappers) | Model registry / `pull`; Go-embedded engine complexity; LAN bind without auth (exposed-server problem) |
| **LM Studio** | https://lmstudio.ai | One-click local server; hardware-aware defaults; polished tray/desktop feel | Closed source; chat + download ecosystem; server tied to GUI process |
| **LocalAI** | https://github.com/mudler/LocalAI | Backend trait / process isolation; OpenAI drop-in positioning | Docker/K8s-first; gRPC polyglot backends; gallery/multimodal scope |
| **Jan** | https://github.com/janhq/jan | Extension lifecycle (`onLoad`/`onUnload`); inference as pluggable engine | AGPL implications if forking; chat/agent/RAG extensions |
| **GPT4All** | https://github.com/nomic-ai/gpt4all · [Local API Server wiki](https://github.com/nomic-ai/gpt4all/wiki/Local-API-Server) | Optional OpenAI subset; **localhost-only by default** (safe default) | Chat app owns lifecycle; incomplete OpenAI surface; no LAN without hacks |
| **text-generation-webui** (oobabooga) | https://github.com/oobabooga/text-generation-webui | `--api` flag pattern; multi-backend experiments | Python stack, chat UI, extension sprawl |
| **KoboldCpp** | https://github.com/LostRuins/koboldcpp | Single portable Windows `.exe`; OpenAI `/v1` + legacy APIs; GPU layer knobs | Bundled chat UI; multi-API surface area; SillyTavern-centric defaults |
| **SillyTavern** (frontend only) | https://github.com/SillyTavern/SillyTavern · [self-hosted docs](https://docs.sillytavern.app/usage/how-to-use-a-self-hosted-model/) | Clear **frontend ≠ backend** split; documents KoboldCpp/Ollama/LM Studio as backends | Any chat/character features |
| **vLLM** | https://github.com/vllm-project/vllm | Production readiness probe patterns (`/health` vs `/v1/models`) | Python/CUDA server stack; multi-user throughput focus (out of MVP) |
| **llama-cpp-windows-manager** | https://github.com/alekk89/llama-cpp-windows-manager | Supervised `llama-server` sessions; per-model ports; logs/metrics UI ideas | Runtime download/update UI; multi-session gateway; WSL path |
| **llama-server-manager** | https://github.com/zero4281/llama-server-manager | Thin install/start/stop wrapper; OpenAI client examples | HF download flow; interactive install menus |
| **d4-ollama-win-service** | https://github.com/internetics-net/d4-ollama-win-service | NSSM wrap + firewall rule + log rotation + auto-restart | NSSM as required dependency for MVP tray app |

---

## GitHub references (repos, issues, code patterns with links)

### Core engine & API

| Reference | Why |
| --- | --- |
| [ggml-org/llama.cpp](https://github.com/ggml-org/llama.cpp) | Upstream; MIT; `llama-server` is the v1 backend binary |
| [tools/server README — `/health`](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md) | Optional alternate: **503** while loading, **200** `{"status":"ok"}` when ready; `/v1/health` alias. **WinServeAI primary readiness = `GET /v1/models`** |
| [PR #9056 — `/health` vs `/slots`](https://github.com/ggml-org/llama.cpp/pull/9056) | Health must not mean “slot free”; failed model load → process exit code 1 |
| [Issue #20684 — `/health` under load](https://github.com/ggml-org/llama.cpp/issues/20684) | Health can queue behind inference; use generous timeouts / extra HTTP threads if probing under load |
| [Discussion #18049 — `--fit`](https://github.com/ggml-org/llama.cpp/discussions/18049) | Auto GPU layers / context; disabled if user sets `-ngl` / `--tensor-split` |
| [PR #14067 — auto n-gpu-layers](https://github.com/ggml-org/llama.cpp/pull/14067) | History of auto-fit design |
| [Discussion #472 — commercial use](https://github.com/ggml-org/llama.cpp/discussions/472) | Code MIT OK; **model weights** have separate licenses |
| [LICENSE](https://github.com/ggml-org/llama.cpp/blob/master/LICENSE) | MIT — copyright + permission notice must travel with distributions |
| [API changelogs](https://github.com/ggml-org/llama.cpp/issues/9291) | Pin server REST surface per release |

### Process management (Windows)

| Reference | Why |
| --- | --- |
| [watchexec/process-wrap](https://github.com/watchexec/process-wrap) · [docs.rs](https://docs.rs/process-wrap) | Rust `JobObject` wrapper; `CREATE_SUSPENDED` then assign; preferred over hand-rolled Win32 |
| [anomalyco/opencode JobObject commit](https://github.com/anomalyco/opencode/commit/ddd9c71cca1f30a8214174fc10975e2ff3bb4635) | Tauri/desktop: attach child PID to job; kill on parent exit/crash |
| [NeuralNomadsAI/CodeNomad job object](https://github.com/NeuralNomadsAI/CodeNomad/commit/1ce58b9dd914e78728eabf40b5fcc645e885300f) | Graceful stop then drop job to reap orphans; `taskkill /T` alone is insufficient |
| [diffplug/dormouse PR #41](https://github.com/diffplug/mouseterm/pull/41) | Sidecar + `process-wrap`; notes Tauri sidecar lifecycle gaps |
| [internetics-net/d4-ollama-win-service](https://github.com/internetics-net/d4-ollama-win-service) | NSSM pattern: restart, logs, firewall — Phase 5 service mode, not tray MVP |
| [Discussion #20648 — Win11 sleep/wake](https://github.com/ggml-org/llama.cpp/discussions/20648) | Duplicate `llama-server` after resume; kill-by-port/name before restart |

### Hardware detection

| Reference | Why |
| --- | --- |
| [IDXGIAdapter3::QueryVideoMemoryInfo](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_4/nf-dxgi1_4-idxgiadapter3-queryvideomemoryinfo) | Windows-primary VRAM budget/usage (WDDM) |
| [SO: DirectX get VRAM](https://stackoverflow.com/questions/35527315/directx-get-vram-used-by-game) | Minimal DXGI sample |
| [ollama mem_nvml.cpp](https://github.com/ollama/ollama/blob/main/ml/backend/ggml/ggml/src/mem_nvml.cpp) | NVML for device free/total; notes Windows CUDA VRAM quirks |
| [PCfVW/hypomnesis](https://github.com/PCfVW/hypomnesis) | Rust: DXGI (Win) + NVML (Linux/device-wide); documents NVML per-process `NOT_AVAILABLE` under WDDM |
| [candle-mi memory.rs](https://docs.rs/candle-mi/latest/src/candle_mi/memory.rs.html) | Same DXGI-first Windows strategy |

### Runtime / process isolation (aspirational only)

| Reference | Why |
| --- | --- |
| [LocalAI backend/README](https://github.com/mudler/LocalAI/blob/master/backend/README.md) | `backend.proto` gRPC contract; child-process backends; WatchDog — **aspirational**, too heavy for MVP |
| [LocalAI Backend System (DeepWiki)](https://deepwiki.com/mudler/LocalAI/3.2-backend-system) | ModelLoader + process lifecycle overview |
| [Jan extensions](https://janhq-jan-19.mintlify.app/extensions/overview) | `BaseExtension` lifecycle; inference as extension type — UI-centric, but lifecycle shape is clean |
| WinServeAI appliance | **No** Rust `Backend` trait. Single crate owns spawn of `bin/llama-server.exe` via Server Manager (`app/src/server/manager.rs`) + runtime (`app/src/runtime/`) |

### Licensing compliance (cautionary)

| Reference | Why |
| --- | --- |
| [ollama#3185 — missing notices in binaries](https://github.com/ollama/ollama/issues/3185) | MIT requires notice in **binary** distributions, not only source |
| [HN discussion](https://news.ycombinator.com/item?id=44003741) | Community expectations for third-party notices |

### Installer / firewall

| Reference | Why |
| --- | --- |
| [SO: Inno Setup firewall via netsh](https://stackoverflow.com/questions/7701667/how-to-add-outbound-windows-firewall-exception) | `[Run]` / `[UninstallRun]` `netsh advfirewall firewall add/delete rule` |
| [SO: netsh Exec pitfalls](https://stackoverflow.com/questions/31970310/running-netsh-from-using-inno-setup-exec) | Correct `advfirewall firewall` syntax; check exit codes |
| [SO: Inno WorkingDir for child scripts](https://stackoverflow.com/questions/63009117/issue-with-installing-node-application-as-a-service-using-inno-setup-and-node-wi) | Installer temp cwd breaks child processes — set `WorkingDir` |

---

## Stack Exchange / Q&A (useful threads with links)

| Thread | Takeaway for WinServeAI |
| --- | --- |
| [What is a Job Object?](https://stackoverflow.com/questions/1414899/what-is-a-job-child-process-thing-in-windows-and-when-to-use-it) | Jobs group processes; one process → one job (classic limit) |
| [Automatically destroy child processes](https://stackoverflow.com/questions/53208/how-do-i-automatically-destroy-child-processes-in-windows) | `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` pattern |
| [Kill child when parent is killed](https://stackoverflow.com/questions/3342941/kill-child-process-when-parent-process-is-killed) | Same; C# samples map cleanly to Rust `windows` crate |
| [Job objects: grandchildren / breakaway](https://stackoverflow.com/questions/33424492/windows-api-job-objects-dont-pass-on-to-grandchildren) | `SILENT_BREAKAWAY_OK` / `CREATE_BREAKAWAY_FROM_JOB` if needed |
| [Force-kill parent still kills children](https://stackoverflow.com/questions/24012773/c-winapi-how-to-kill-child-processes-when-the-calling-parent-process-is-forcefully-terminated) | Suspended create → assign → resume race fix |
| [DXGI VRAM query](https://stackoverflow.com/questions/35527315/directx-get-vram-used-by-game) | Adapter enum + `QueryVideoMemoryInfo` |
| [Inno Setup firewall rules](https://stackoverflow.com/questions/7701667/how-to-add-outbound-windows-firewall-exception) | Prefer program-based allow for `llama-server.exe` or port-based for configured port |
| [Inno Setup enumerate FW rules](https://stackoverflow.com/questions/71915942/inno-setup-iterate-enumerate-through-firewall-rules-ienumvariant) | `HNetCfg.FwPolicy2` COM if netsh is insufficient |
| [Rust forum: process-wrap JobObject](https://users.rust-lang.org/t/killing-subprocesses-of-std-command/117905) | Community confirmation of `process-wrap` for Windows trees |
| [Send Ctrl+C / SIGINT on Windows](https://stackoverflow.com/questions/813086/can-i-send-a-ctrl-c-sigint-to-an-application-on-windows) | `AttachConsole` + `GenerateConsoleCtrlEvent` for graceful stop |
| [CTRL+C to a Windows subprocess](https://stackoverflow.com/questions/40762545/how-to-send-a-ctrlc-sigint-to-a-subprocess-in-windows) | `CREATE_NEW_PROCESS_GROUP`; prefer `CTRL_BREAK_EVENT` (cannot be disabled) |

**Readiness (non-SE but essential):** distinguish **liveness** (process alive) vs **readiness** (model loaded). **WinServeAI readiness probe today = `GET /v1/models`** (`app/src/server/health.rs`). Upstream `GET /health` (503 until loaded, 200 when ready) is a useful alternate when present — do not treat it as the only probe. Under heavy load probes may stall — see [llama.cpp#20684](https://github.com/ggml-org/llama.cpp/issues/20684). vLLM’s probe split (`/health` vs `/v1/models`) is documented in [llm-d readiness-probes.md](https://github.com/llm-d/llm-d/blob/main/docs/readiness-probes.md) — useful mental model even without K8s.

---

## YouTube (videos/channels with links and why relevant)

| Video / channel | Link | Why relevant |
| --- | --- | --- |
| **Ollama vs LM Studio vs llama.cpp: Which Should You Use?** | https://www.youtube.com/watch?v=crXFOd7gG_I | Clear stack layering: llama.cpp = engine, LM Studio = body, Ollama = orchestration; security note on LAN binds |
| **The Ultimate Local AI Developer Stack** | https://www.youtube.com/watch?v=MiweHNjuG04 | Practical `llama-server` + OpenAI-compatible clients; mentions `--fit` / VRAM edge cases |
| **FOSS United CFP: Inside the Architecture of llama.cpp** | https://fossunited.org/c/pune/2026-march/cfp/a0k9tnpnjd | Talk abstract on memory/backends under Ollama/LM Studio/Jan — good conceptual map |
| Search: “llama-server OpenAI” / “local LLM Windows CUDA” | YouTube search | Many demos; prefer ones that show **API clients**, not chat UIs |

Channels worth skimming (demo quality varies): creators covering **Continue.dev / Open WebUI + Ollama**, and Windows CUDA install walkthroughs (DLL path issues next to `llama-server.exe` — see also [Visokio Windows CUDA notes](https://help.visokio.com/support/solutions/articles/42000115649-how-to-run-local-llms-on-windows-with-nvidia-llama-cpp-cuda-)).

---

## X / Twitter (accounts/threads if findable, or note if sparse)

**Sparse for scaffolding.** Local-LLM Twitter is mostly release hype and model cards, not process/installer engineering.

| Account / topic | Notes |
| --- | --- |
| [@ggerganov](https://x.com/ggerganov) (if active) | llama.cpp author; release signals |
| Ollama / LM Studio official accounts | Product announcements; little Windows Job Object content |
| Threads on “exposed Ollama on 0.0.0.0” | Recurring security theme — reinforces **localhost default**, firewall only when LAN bind enabled |

For engineering signal, **GitHub issues/discussions and SO beat X**. Treat X as optional awareness, not a primary research source for this monorepo.

---

## Scaffolding recommendations (concrete additions to single-crate layout — prioritized, minimal, YAGNI)

Ordered for Phase 0–1. Do **not** implement chat, downloads, or multi-backend plugins yet. Layout is **`app/`** (crate `winserve`) + **`bin/llama-server.exe`** — no `packages/*`, no Rust `Backend` trait.

### P0 — process runtime (`app/src/runtime/process.rs`) (must)

1. Spawn `bin/llama-server.exe` with stdout/stderr pipes; record PID + exit code.
2. On Windows, wrap with **`process-wrap` `JobObject`** (`KILL_ON_JOB_CLOSE`). Graceful path: send Ctrl+C / `GenerateConsoleCtrlEvent` or llama-server’s preferred stop, wait timeout, then kill job.
3. Crash detection: `try_wait` / exit watcher → surface `Crashed { code }` to Server Manager; optional single restart with backoff (config flag, default off).
4. Unit-test with a tiny stub exe (not full llama) on Windows CI if available.

### P0 — llama runtime + readiness (`app/src/runtime/llama.rs`, `app/src/server/health.rs`)

1. Map high-level config → argv only (host, port, model path, context; prefer **omit `-ngl`** and let **`--fit`** run unless user overrides).
2. Readiness loop: poll **`GET /v1/models`** until 200 or timeout; treat 503 as “still loading”; connection refused as “not up yet”. Optional alternate: `GET /health` (503/200) when present on the pin — not the only probe.
3. Document pinned `bin/llama-server.exe` release tag (`b####`) + CUDA/CPU build matrix in `docs/backend.md` / release docs.

### P0 — hardware inventory (`app/src/system/`)

1. **Inventory:** DXGI adapter list (name, dedicated VRAM budget); WMI or `sysinfo` for CPU/RAM.
2. **NVIDIA totals:** optional dynamic load of `nvml.dll` for free/total (device-wide), not per-process.
3. Expose a small struct: `{ gpus: [...], total_ram_mb, recommended: { fit: true } }` — no layer calculator.

### P0 — config (`app/src/server/config.rs`)

1. Keep human YAML only (`host`, `port`, `model`, `context`, `gpu: auto|off|layers: N`).
2. Never store raw llama flags in v1; if advanced escape hatch is needed later, gate it.

### P0 — licensing / `bin/` pin

1. Add `THIRD_PARTY_NOTICES.md` (or `notices/`) including llama.cpp MIT text + copyright; **copy into installer payload**.
2. Script or doc step: when updating the `bin/llama-server.exe` pin (tag in release docs), refresh notices.

### P1 — Server Manager (`app/src/server/manager.rs`)

1. Sole owner of start/stop/status/health; UI/CLI only call into it.
2. State machine: `Stopped → Starting → Ready | Failed → Stopping → Stopped` (+ `Crashed`).
3. Forward process logs through `app/src/server/logs.rs`.

### P1 — installer (Inno Setup)

1. Prefer Inno Setup (see [installer.md](./installer.md)).
2. Firewall: only if bind is non-loopback; `netsh advfirewall firewall add rule` for `llama-server.exe` **or** configured port; matching `delete rule` on uninstall.
3. Bundle: `winserve` + tray (later) + `bin/llama-server.exe` + CUDA runtime DLLs **beside** the exe (PATH is unreliable on Windows).

### P1 — desktop (Tauri tray, Phase 2)

1. Tray menu: Start / Stop / Open config / Quit — **no chat**.
2. On quit: call Server Manager stop; rely on Job Object if force-killed.
3. Do not use Tauri sidecar for inference; call into `ServerManager` only.

### P2 — scripts / docs only

1. `scripts/smoke-openai.ps1` — `GET /v1/models` + minimal `chat/completions` against running server.
2. Job Object + readiness notes (this file + `docs/research/01–02`).
3. Update [research.md](./research.md) checkboxes as items land.

### Explicitly defer

| Item | Why defer |
| --- | --- |
| NSSM / Windows Service | Tray + Job Object covers MVP; service is Phase 5 |
| LocalAI-style gRPC plugins / Backend trait | Single-crate appliance is enough; no trait |
| Model download / HF | Out of MVP scope |
| WiX / MSIX | Inno first |
| Auto-update | Phase 5 |
| Auth / metrics dashboard | Out of scope |

---

## Open research questions

1. **Graceful stop signal for `llama-server` on Windows** — Does Ctrl+C via console event flush cleanly, or is terminate-after-timeout the practical path? Validate on current pinned build.
2. **`--fit` vs explicit layers in YAML** — Default `gpu: auto` → no `-ngl`; does `--fit` behave well on low-VRAM (6–8 GB) Windows laptops with shared display GPU?
3. **CUDA DLL bundling policy** — Ship runtime DLLs next to `llama-server.exe` vs require system CUDA. Affects installer size and support burden ([Windows CUDA path notes](https://help.visokio.com/support/solutions/articles/42000115649-how-to-run-local-llms-on-windows-with-nvidia-llama-cpp-cuda-)).
4. **AMD / Intel GPUs** — Vulkan builds of llama.cpp on Windows; detection via DXGI only, or need vendor APIs?
5. **Readiness under load** — For a single-user LAN server, is a 5–10s probe timeout on `GET /v1/models` enough, or do we need a separate “process alive” check (PID + port listen) vs HTTP readiness?
6. **Port conflicts** — Bind failure messaging and optional port scan; any prior art in Ollama/LM Studio worth copying?
7. **Job Object + already-in-a-job** — Installer/updater or sandboxed parent may place manager in a job; need `IsProcessInJob` / breakaway handling?
8. **Release cadence** — How often to pin llama.cpp releases given rapid `llama-server` API churn ([changelog #9291](https://github.com/ggml-org/llama.cpp/issues/9291))?
9. **License automation** — Minimal script to regenerate `THIRD_PARTY_NOTICES` when the `bin/` pin changes (llama.cpp + transitive MIT deps like cpp-httplib).
10. **Service mode later** — If Phase 5 adds a Windows Service, does the service host own the Job Object, or does NSSM remain an acceptable thin wrapper?

---

## See also

- Phase-0 deep dives: [research/01](./research/01-windows-process.md) · [02](./research/02-llama-readiness.md) · [03](./research/03-hardware-detection.md) · [04](./research/04-installer-licensing.md) · [05](./research/05-server-manager.md)
- Product readiness: [`app/src/server/health.rs`](../app/src/server/health.rs) (`GET /v1/models`)
- Canonical external URL index: [policies/SOURCES.md](./policies/SOURCES.md)

---

*Generated for scaffolding decisions. Prefer updating this file when a linked upstream behavior changes (especially readiness probes and `--fit`).*
