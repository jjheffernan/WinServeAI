# Canonical external sources

Index of upstream URLs cited across WinServeAI docs. Prefer linking here (or to research notes) over pasting long explanations into operator guides.

**Internal code anchors** (not external, but primary for claims):

| Claim | Path |
| --- | --- |
| Readiness probe (`GET /v1/models`) | `app/src/server/health.rs` |
| llama.cpp argv / flags | `app/src/runtime/llama.rs` |
| Spawn / stop | `app/src/runtime/process.rs` |
| YAML schema | `app/src/server/config.rs`, `config/default.yaml` |
| Orchestration | `app/src/server/manager.rs` |

Anti-drift: [doc-drift.md](./doc-drift.md). Research deep dives: [research/](../research/).

---

## llama.cpp (engine)

| Topic | URL |
| --- | --- |
| Repository | https://github.com/ggml-org/llama.cpp |
| Server (`tools/server`) | https://github.com/ggml-org/llama.cpp/tree/master/tools/server |
| Server README (`/v1`, `/health`, flags) | https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md |
| Releases (`b####` pins) | https://github.com/ggml-org/llama.cpp/releases |
| LICENSE (MIT) | https://github.com/ggml-org/llama.cpp/blob/master/LICENSE |
| AUTHORS | https://github.com/ggml-org/llama.cpp/blob/master/AUTHORS |
| `/health` refactor (503 loading, exit 1) | https://github.com/ggml-org/llama.cpp/pull/9056 |
| `/health` under load | https://github.com/ggml-org/llama.cpp/issues/20684 |
| Dynamic HTTP threads | https://github.com/ggml-org/llama.cpp/pull/20817 |
| `--fit` design (`-ngl` disables fit) | https://github.com/ggml-org/llama.cpp/discussions/18049 |
| Auto n-gpu-layers history | https://github.com/ggml-org/llama.cpp/pull/14067 |
| REST API changelog | https://github.com/ggml-org/llama.cpp/issues/9291 |
| Win11 sleep/wake orphans | https://github.com/ggml-org/llama.cpp/discussions/20648 |
| Code MIT vs model licenses | https://github.com/ggml-org/llama.cpp/discussions/472 |

Cited in: [api.md](../api.md), [backend.md](../backend.md), [research/02](../research/02-llama-readiness.md), [prior-art.md](../prior-art.md).

---

## Microsoft / Windows

| Topic | URL |
| --- | --- |
| Job Objects | https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects |
| `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` | https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_basic_limit_information |
| Console process groups | https://learn.microsoft.com/en-us/windows/console/console-process-groups |
| `GenerateConsoleCtrlEvent` | https://learn.microsoft.com/en-us/windows/console/generateconsolectrlevent |
| DXGI overview | https://learn.microsoft.com/en-us/windows/win32/direct3ddxgi/d3d10-graphics-programming-guide-dxgi |
| `IDXGIAdapter3::QueryVideoMemoryInfo` | https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_4/nf-dxgi1_4-idxgiadapter3-queryvideomemoryinfo |
| Netsh AdvFirewall | https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-server-2008-r2-and-2008/dd734783(v=ws.10) |

Cited in: [backend.md](../backend.md), [research/01](../research/01-windows-process.md), [research/03](../research/03-hardware-detection.md), [research/04](../research/04-installer-licensing.md).

---

## NVIDIA

| Topic | URL |
| --- | --- |
| CUDA Toolkit EULA (Attachment A) | https://docs.nvidia.com/cuda/eula/ |
| NVML process info (WDDM N/A) | https://docs.nvidia.com/deploy/nvml-api/structnvmlProcessInfo__v2__t.html |
| NVML WDDM forum | https://forums.developer.nvidia.com/t/nvml-problems-for-windows-not-available-in-wddm-driver-model/77557/2 |
| CUDA deploy / redistribute DLLs | https://forums.developer.nvidia.com/t/cuda-application-deployment-what-is-correct-deployment/15459 |

Cited in: [research/03](../research/03-hardware-detection.md), [research/04](../research/04-installer-licensing.md), [installer.md](../installer.md).

---

## Rust crates

| Crate | URL |
| --- | --- |
| process-wrap (Job Objects) | https://github.com/watchexec/process-wrap · https://docs.rs/process-wrap · https://crates.io/crates/process-wrap |
| windows | https://docs.rs/windows · https://crates.io/crates/windows |
| sysinfo | https://docs.rs/sysinfo · https://crates.io/crates/sysinfo |
| nvml-wrapper | https://docs.rs/nvml-wrapper · https://crates.io/crates/nvml-wrapper |
| hypomnesis (reference only) | https://github.com/PCfVW/hypomnesis · https://crates.io/crates/hypomnesis |

Cited in: [research/01](../research/01-windows-process.md), [research/03](../research/03-hardware-detection.md), [backend.md](../backend.md).

---

## Installer / packaging

| Topic | URL |
| --- | --- |
| Inno Setup `[Icons]` | https://jrsoftware.org/ishelp/topic_iconssection.htm |
| Inno admin install mode | https://jrsoftware.org/ishelp/topic_admininstallmode.htm |
| Inno + netsh firewall (SO) | https://stackoverflow.com/questions/7701667/how-to-add-outbound-windows-firewall-exception |
| netsh Exec pitfalls (SO) | https://stackoverflow.com/questions/31970310/running-netsh-from-using-inno-setup-exec |
| Windows CUDA path pitfalls | https://help.visokio.com/support/solutions/articles/42000115649-how-to-run-local-llms-on-windows-with-nvidia-llama-cpp-cuda- |

Cited in: [installer.md](../installer.md), [research/04](../research/04-installer-licensing.md).

---

## Prior-art products & compliance

| Topic | URL |
| --- | --- |
| Ollama | https://github.com/ollama/ollama |
| Ollama missing binary notices | https://github.com/ollama/ollama/issues/3185 |
| Ollama port-in-use (Windows) | https://github.com/ollama/ollama/issues/3575 |
| Ollama NVML usage | https://github.com/ollama/ollama/blob/main/ml/backend/ggml/ggml/src/mem_nvml.cpp |
| GPT4All Local API (localhost default) | https://github.com/nomic-ai/gpt4all/wiki/Local-API-Server |
| llama-cpp-windows-manager | https://github.com/alekk89/llama-cpp-windows-manager |
| llm-d readiness probes | https://github.com/llm-d/llm-d/blob/main/docs/readiness-probes.md |
| OpenCode JobObject | https://github.com/anomalyco/opencode/commit/ddd9c71cca1f30a8214174fc10975e2ff3bb4635 |
| CodeNomad job object | https://github.com/NeuralNomadsAI/CodeNomad/commit/1ce58b9dd914e78728eabf40b5fcc645e885300f |

Full product table: [prior-art.md](../prior-art.md).

---

## Stack Overflow (high-signal)

| Topic | URL |
| --- | --- |
| Auto-destroy children (Job Objects) | https://stackoverflow.com/questions/53208/how-do-i-automatically-destroy-child-processes-in-windows |
| Kill child when parent killed | https://stackoverflow.com/questions/3342941/kill-child-process-when-parent-process-is-killed |
| Suspended spawn → assign race | https://stackoverflow.com/questions/24012773/c-winapi-how-to-kill-child-processes-when-the-calling-parent-process-is-forcefully-terminated |
| CTRL_BREAK / process group | https://stackoverflow.com/questions/40762545/how-to-send-a-ctrlc-sigint-to-a-subprocess-in-windows |
| DXGI VRAM sample | https://stackoverflow.com/questions/35527315/directx-get-vram-used-by-game |

More threads: [prior-art.md](../prior-art.md), [research/01](../research/01-windows-process.md).
