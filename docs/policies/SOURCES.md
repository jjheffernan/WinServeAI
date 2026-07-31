# Canonical external sources

Index of upstream and reference URLs already cited in phase-0 research (`research/01`–`05`) and [prior-art.md](../prior-art.md). Prefer linking here or the research note over inventing new citations.

Internal policy: [doc-drift.md](./doc-drift.md).

## llama.cpp / llama-server

| Topic | URL | Cited in |
| --- | --- | --- |
| `tools/server` tree | https://github.com/ggml-org/llama.cpp/tree/master/tools/server | prior-art, research/02 |
| Server README (`/v1`, optional `/health`, flags) | https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md | prior-art, research/02, research/05 |
| Releases (`b####` pins) | https://github.com/ggml-org/llama.cpp/releases | research/02 |
| REST API changelog | https://github.com/ggml-org/llama.cpp/issues/9291 | prior-art, research/02 |
| `/health` refactor (503 loading; load fail → exit 1) | https://github.com/ggml-org/llama.cpp/pull/9056 | prior-art, research/01, research/02, research/05 |
| `/health` under load | https://github.com/ggml-org/llama.cpp/issues/20684 | prior-art, research/01, research/02, research/05 |
| Dynamic HTTP threads (load mitigation) | https://github.com/ggml-org/llama.cpp/pull/20817 | research/02 |
| `--fit` design (`-ngl` disables fit) | https://github.com/ggml-org/llama.cpp/discussions/18049 | prior-art, research/02, research/03 |
| Auto n-gpu-layers history | https://github.com/ggml-org/llama.cpp/pull/14067 | prior-art, research/02 |
| Win11 sleep/wake orphans | https://github.com/ggml-org/llama.cpp/discussions/20648 | prior-art, research/01, research/02 |
| MIT LICENSE | https://github.com/ggml-org/llama.cpp/blob/master/LICENSE | prior-art, research/04 |
| AUTHORS | https://github.com/ggml-org/llama.cpp/blob/master/AUTHORS | research/04 |
| Code MIT vs model licenses | https://github.com/ggml-org/llama.cpp/discussions/472 | prior-art, research/04 |

## Windows process (Job Objects, signals, process-wrap)

| Topic | URL | Cited in |
| --- | --- | --- |
| Job Objects | https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects | research/01 |
| `KILL_ON_JOB_CLOSE` | https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_basic_limit_information | research/01 |
| `GenerateConsoleCtrlEvent` | https://learn.microsoft.com/en-us/windows/console/generateconsolectrlevent | research/01 |
| Console process groups | https://learn.microsoft.com/en-us/windows/console/console-process-groups | research/01 |
| process-wrap (GitHub) | https://github.com/watchexec/process-wrap | prior-art, research/01 |
| process-wrap (docs.rs) | https://docs.rs/process-wrap | prior-art, research/01, research/05 |
| Auto-destroy children (SO) | https://stackoverflow.com/questions/53208/how-do-i-automatically-destroy-child-processes-in-windows | prior-art, research/01 |
| Kill child when parent killed (SO) | https://stackoverflow.com/questions/3342941/kill-child-process-when-parent-process-is-killed | prior-art, research/01 |
| Suspended create → assign race (SO) | https://stackoverflow.com/questions/24012773/c-winapi-how-to-kill-child-processes-when-the-calling-parent-process-is-forcefully-terminated | prior-art, research/01 |
| Job objects / grandchildren (SO) | https://stackoverflow.com/questions/33424492/windows-api-job-objects-dont-pass-on-to-grandchildren | prior-art, research/01 |
| CTRL_BREAK vs CTRL_C (SO) | https://stackoverflow.com/questions/40762545/how-to-send-a-ctrlc-sigint-to-a-subprocess-in-windows | prior-art, research/01 |
| CTRL_C attach pattern (SO) | https://stackoverflow.com/questions/813086/can-i-send-a-ctrl-c-sigint-to-an-application-on-windows | prior-art, research/01 |
| CTRL_C group ID pitfall | https://github.com/dotnet/docs/issues/53173 | research/01 |
| Rust forum: process-wrap JobObject | https://users.rust-lang.org/t/killing-subprocesses-of-std-command/117905 | prior-art, research/01 |

## Hardware (DXGI / NVML)

| Topic | URL | Cited in |
| --- | --- | --- |
| DXGI `QueryVideoMemoryInfo` | https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_4/nf-dxgi1_4-idxgiadapter3-queryvideomemoryinfo | prior-art, research/03 |
| DXGI overview | https://learn.microsoft.com/en-us/windows/win32/direct3ddxgi/d3d10-graphics-programming-guide-dxgi | research/03 |
| DXGI VRAM sample (SO) | https://stackoverflow.com/questions/35527315/directx-get-vram-used-by-game | prior-art, research/03 |
| NVML process info (WDDM N/A) | https://docs.nvidia.com/deploy/nvml-api/structnvmlProcessInfo__v2__t.html | research/03 |
| NVML WDDM forum | https://forums.developer.nvidia.com/t/nvml-problems-for-windows-not-available-in-wddm-driver-model/77557/2 | research/03 |
| CUDA detect without toolkit (SO) | https://stackoverflow.com/questions/12828468/detecting-nvidia-gpus-without-cuda | research/03 |
| ollama `mem_nvml.cpp` | https://github.com/ollama/ollama/blob/main/ml/backend/ggml/ggml/src/mem_nvml.cpp | prior-art, research/03 |
| hypomnesis (DXGI + NVML) | https://github.com/PCfVW/hypomnesis | prior-art, research/03 |
| candle-mi DXGI-first | https://docs.rs/candle-mi/latest/src/candle_mi/memory.rs.html | prior-art, research/03 |

## Installer, licensing, notices

| Topic | URL | Cited in |
| --- | --- | --- |
| Ollama missing binary notices | https://github.com/ollama/ollama/issues/3185 | prior-art, research/04 |
| HN on notices | https://news.ycombinator.com/item?id=44003741 | prior-art, research/04 |
| Inno `[Icons]` | https://jrsoftware.org/ishelp/topic_iconssection.htm | research/04 |
| Inno admin install mode | https://jrsoftware.org/ishelp/topic_admininstallmode.htm | research/04 |
| Inno firewall via netsh (SO) | https://stackoverflow.com/questions/7701667/how-to-add-outbound-windows-firewall-exception | prior-art, research/04 |
| netsh `advfirewall` syntax (SO) | https://stackoverflow.com/questions/31970310/running-netsh-from-using-inno-setup-exec | prior-art, research/04 |
| Netsh AdvFirewall reference | https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-server-2008-r2-and-2008/dd734783(v=ws.10) | research/04 |
| CUDA EULA (Attachment A) | https://docs.nvidia.com/cuda/eula/ | research/04 |
| CUDA deploy / redistribute DLLs | https://forums.developer.nvidia.com/t/cuda-application-deployment-what-is-correct-deployment/15459 | research/04 |
| Windows CUDA path pitfalls | https://help.visokio.com/support/solutions/articles/42000115649-how-to-run-local-llms-on-windows-with-nvidia-llama-cpp-cuda- | prior-art, research/04 |
| NSSM + firewall prior art | https://github.com/internetics-net/d4-ollama-win-service | prior-art, research/04 |

## Readiness mental model (non-llama)

| Topic | URL | Cited in |
| --- | --- | --- |
| llm-d readiness probes (liveness ≠ readiness) | https://github.com/llm-d/llm-d/blob/main/docs/readiness-probes.md | prior-art, research/02, research/05 |

## Competitor / analogue upstreams

Cited from [prior-art.md](../prior-art.md) and Phase-0 research. Prefer path@rev when asserting code facts.

| Topic | URL | Cited in |
| --- | --- | --- |
| llama.cpp (server pin `b9866`) | https://github.com/ggml-org/llama.cpp | research/02, prior-art, bin/README |
| Ollama | https://github.com/ollama/ollama | prior-art |
| Jan | https://github.com/janhq/jan | prior-art |
| LocalAI | https://github.com/mudler/LocalAI | prior-art |
| GPT4All | https://github.com/nomic-ai/gpt4all | prior-art |

## Related internal docs

| Doc | Role |
| --- | --- |
| [prior-art.md](../prior-art.md) | Product analogues and scaffolding recommendations |
| [research/01-windows-process.md](../research/01-windows-process.md) | Job Objects, process-wrap, graceful stop |
| [research/02-llama-readiness.md](../research/02-llama-readiness.md) | Probes, `--fit`, pin policy |
| [research/03-hardware-detection.md](../research/03-hardware-detection.md) | DXGI / NVML inventory |
| [research/04-installer-licensing.md](../research/04-installer-licensing.md) | Inno, CUDA DLLs, MIT notices |
| [research/05-server-manager.md](../research/05-server-manager.md) | State machine and manager API |
