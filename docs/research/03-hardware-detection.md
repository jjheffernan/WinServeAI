# Hardware Detection

> **Path note (appliance layout):** This note was written against an earlier `packages/*` monorepo. Map old paths to the current single crate:
> `packages/launcher` → `app/src/server/manager.rs` · `packages/process` → `app/src/runtime/process.rs` · `packages/llama` → `app/src/runtime/llama.rs` · `packages/api` / readiness → `app/src/server/health.rs` + `app/src/api/` · `packages/hardware` → `app/src/system/` · `packages/config` → `app/src/server/config.rs` · `packages/logging` → `app/src/server/logs.rs` · `packages/backend` / traits → **removed** (no backend trait).

Windows-first inventory for auto config. Goal: fill `app/src/system/` so Server Manager can choose **CPU-only vs GPU path** and prefer llama.cpp **`--fit`** over hand-rolled layer math. Historical stub lived under `packages/hardware` (`detect()` returns zeros; `auto_gpu_layers` returns `99` if any `cuda_capable`). Prior art: [`docs/prior-art.md`](../prior-art.md) (Hardware section + P0 system inventory).

**Do not** implement a VRAM layer calculator in WinServeAI. Inventory + recommendation only.

---

## Recommendations for hardware inventory (`app/src/system/`)

### Probe order (Windows)

| Priority | Source | Use for | Skip when |
| --- | --- | --- | --- |
| 1 | **DXGI** (`IDXGIFactory` enum + `IDXGIAdapter3::QueryVideoMemoryInfo`) | Adapter list, name, dedicated VRAM / process **Budget** / **CurrentUsage** (all vendors) | Non-Windows (cfg out) |
| 2 | **NVML** (`nvml.dll`, dynamic load) | NVIDIA **device-wide** free/total (and optional `reserved` via v2 memory info) | No NVIDIA driver / load fails |
| 3 | **CUDA Driver API** (`nvcuda.dll`, dynamic load) | `cuda_capable`, compute capability (`major.minor`) | DLL missing or `cuInit` fails |
| 4 | **sysinfo** | CPU name/cores, system RAM total/available | Never (always fill best-effort) |

Never fail `detect()` because a probe is missing. Missing GPU/CUDA/NVML → empty `gpus` or `cuda_capable: false` → **CPU-only graceful path**.

### DXGI (primary VRAM on Windows)

- API: [`IDXGIAdapter3::QueryVideoMemoryInfo`](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_4/nf-dxgi1_4-idxgiadapter3-queryvideomemoryinfo) with `DXGI_MEMORY_SEGMENT_GROUP_LOCAL`.
- Fields that matter: `Budget` (OS-assigned process budget), `CurrentUsage` (this process). Also read `DXGI_ADAPTER_DESC::DedicatedVideoMemory` / `Description` for inventory totals and names.
- Minimal sample: [SO: DirectX get VRAM](https://stackoverflow.com/questions/35527315/directx-get-vram-used-by-game).
- Same strategy as [candle-mi memory.rs](https://docs.rs/candle-mi/latest/src/candle_mi/memory.rs.html) and [hypomnesis](https://github.com/PCfVW/hypomnesis) (DXGI for Windows per-process / budget; NVML for device-wide on NVIDIA).

Use DXGI for **every** adapter (NVIDIA, AMD, Intel). Shared/iGPU budgets are soft — still report them; do not treat iGPU VRAM as a hard dedicated pool when recommending GPU offload.

### NVML on Windows WDDM (device-wide only)

- Under **WDDM**, the OS (not the NVIDIA driver) owns GPU memory. NVML **per-process** memory is **`NVML_VALUE_NOT_AVAILABLE`** / `N/A` — documented on [`nvmlProcessInfo_v2_t::usedGpuMemory`](https://docs.nvidia.com/deploy/nvml-api/structnvmlProcessInfo__v2__t.html) and [NVIDIA forum](https://forums.developer.nvidia.com/t/nvml-problems-for-windows-not-available-in-wddm-driver-model/77557/2).
- **Do use** NVML for device-wide free/total (what `nvidia-smi` shows in the summary line). Optional: `nvmlDeviceGetMemoryInfo_v2` reserved carve-out (hypomnesis `reserved_bytes`).
- **Do not** use NVML per-process for auto config or metrics on consumer Windows.
- Load via `libloading` / `nvml-wrapper` so absence of `nvml.dll` is a soft miss (CPU-only or DXGI-only inventory).
- Reference: [ollama `mem_nvml.cpp`](https://github.com/ollama/ollama/blob/main/ml/backend/ggml/ggml/src/mem_nvml.cpp).

### CUDA capability (no toolkit)

- Need **driver** only: dynamic `LoadLibrary("nvcuda.dll")` + `cuInit` / `cuDeviceGetCount` / `cuDeviceGetAttribute` (compute capability major/minor). **Not** the CUDA Toolkit, **not** link-time `cudart`.
- Pattern: [SO: detecting NVIDIA GPUs without CUDA](https://stackoverflow.com/questions/12828468/detecting-nvidia-gpus-without-cuda); [CheckForNvidiaNCuda](https://github.com/malikmizery/CheckForNvidiaNCuda) (same idea in .NET).
- Map CUDA devices to DXGI adapters by name/index best-effort; if mapping fails, still set `cuda_capable` on the NVIDIA adapter(s).
- `cuda_capable == true` only means “driver reports a CUDA device,” not “bundled `llama-server` is a CUDA build.” Binary selection stays in launcher/installer.

### CPU / RAM

- Prefer **`sysinfo`** (`System::cpus()`, `total_memory()`, `available_memory()`). Enough for auto config; no COM/WMI dependency.
- WMI (`Win32_Processor`, `Win32_ComputerSystem`) only if you need a friendlier CPU marketing name and sysinfo is insufficient — defer.

### Auto config policy (feeds `app/src/runtime/llama.rs`)

Product argv in [`app/src/runtime/llama.rs`](../../app/src/runtime/llama.rs): when `gpu.auto` and `layers: auto`, pass **`--fit on`** if any GPU is present, else **`--n-gpu-layers 0`**. Explicit `layers` still maps to `--n-gpu-layers {N}`. Historical monorepo stubs always passed `-ngl 99` (disables fit). Inventory policy:

| Condition | Recommendation |
| --- | --- |
| No CUDA-capable GPU, or `gpu: off` | CPU path: `gpu_layers = 0` (or omit GPU flags); never error |
| CUDA-capable + `gpu: auto` | Prefer **omit `-ngl`** and pass **`--fit`** ([llama.cpp discussion #18049](https://github.com/ggml-org/llama.cpp/discussions/18049)); do **not** invent layer counts from VRAM |
| User sets explicit layers | Pass through as today |

Inventory should expose `recommended.use_fit` (or empty `gpus` → CPU), not a magic layer count. DXGI/NVML remain diagnostics only.

### CPU-only graceful path

1. `detect()` always `Ok(profile)` — probes return `Option` / empty vecs.
2. No NVIDIA driver / no `nvcuda.dll` / DXGI lists only Microsoft Basic Render → `gpus` empty or all `cuda_capable: false`.
3. Manager starts **CPU** `llama-server` build (or CUDA build with `-ngl 0`); readiness still polls **`GET /v1/models`** (optional alternate: `/health` when present).
4. UI/logs: “No CUDA GPU detected; running CPU-only” — not a hard failure.

---

## Proposed HardwareProfile fields

Extend the existing stub types; keep serde-friendly and UI-safe (no backend flags).

```text
HardwareProfile
  gpus: Vec<GpuInfo>
  cpu: CpuInfo
  memory: MemoryInfo
  recommended: RecommendedSettings   # NEW

GpuInfo
  index: u32                         # NEW (DXGI / NVML ordinal)
  name: String
  vendor: GpuVendor                  # NEW: Nvidia | Amd | Intel | Other
  vram_total_mb: u64                 # rename from vram_mb; DedicatedVideoMemory or NVML total
  vram_budget_mb: Option<u64>        # NEW; DXGI Budget (LOCAL)
  vram_free_mb: Option<u64>          # NEW; NVML device-wide free only
  cuda_capable: bool
  cuda_compute_capability: Option<String>  # "8.9"

CpuInfo
  name: String
  physical_cores: u32
  logical_cores: u32

MemoryInfo
  total_mb: u64
  available_mb: u64

RecommendedSettings                  # NEW
  use_fit: bool                      # true when any cuda_capable && auto
  prefer_gpu: bool                   # false → CPU-only path
```

Public API stays small:

- `detect() -> Result<HardwareProfile>` — never fails for missing GPU.
- `auto_gpu_layers(profile) -> u32` — deprecate toward `recommended`; if kept, return `0` when `!prefer_gpu`, and **do not** return `99`.

---

## Crate choices (prefer minimal)

| Choice | Role | Verdict |
| --- | --- | --- |
| **`windows`** (features: `Win32_Graphics_Dxgi*`) | DXGI enum + `QueryVideoMemoryInfo` | **Yes** — primary Windows path; `cfg(windows)` only |
| **`sysinfo`** | CPU + RAM | **Yes** — simpler than WMI |
| **`libloading`** + thin NVML/CUDA FFI **or** **`nvml-wrapper`** | Optional NVIDIA totals / CUDA presence | **Optional**; prefer dynamic load so CPU-only machines pay nothing at link time |
| **`hypomnesis`** | DXGI + NVML measurement crate | **Reference only** for MVP — correct WDDM story, but measurement-oriented (RSS, `hmn ps`, macOS Metal), MSRV 1.88, extra surface. Steal patterns; avoid hard dep unless inventory needs grow |
| WMI crates (`wmi`, etc.) | CPU marketing strings | **No** for v1 |

Minimal dep set for v1: `windows` + `sysinfo` + optional `nvml-wrapper` (or ~50 lines of `libloading` for `nvml.dll` / `nvcuda.dll`).

---

## Links

| Topic | URL |
| --- | --- |
| DXGI `QueryVideoMemoryInfo` | https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_4/nf-dxgi1_4-idxgiadapter3-queryvideomemoryinfo |
| DXGI VRAM sample | https://stackoverflow.com/questions/35527315/directx-get-vram-used-by-game |
| NVML process info (WDDM N/A) | https://docs.nvidia.com/deploy/nvml-api/structnvmlProcessInfo__v2__t.html |
| NVML WDDM forum | https://forums.developer.nvidia.com/t/nvml-problems-for-windows-not-available-in-wddm-driver-model/77557/2 |
| hypomnesis (DXGI + NVML) | https://github.com/PCfVW/hypomnesis · https://crates.io/crates/hypomnesis |
| nvml-wrapper | https://docs.rs/nvml-wrapper · https://crates.io/crates/nvml-wrapper |
| sysinfo | https://docs.rs/sysinfo · https://crates.io/crates/sysinfo |
| windows crate | https://docs.rs/windows · https://crates.io/crates/windows |
| CUDA detect without toolkit | https://stackoverflow.com/questions/12828468/detecting-nvidia-gpus-without-cuda |
| ollama NVML usage | https://github.com/ollama/ollama/blob/main/ml/backend/ggml/ggml/src/mem_nvml.cpp |
| candle-mi DXGI-first | https://docs.rs/candle-mi/latest/src/candle_mi/memory.rs.html |
| llama.cpp `--fit` | https://github.com/ggml-org/llama.cpp/discussions/18049 |
| DXGI overview | https://learn.microsoft.com/en-us/windows/win32/direct3ddxgi/d3d10-graphics-programming-guide-dxgi |
| Product argv (`--fit` / `-ngl`) | [`app/src/runtime/llama.rs`](../../app/src/runtime/llama.rs) |
| Internal prior art | [docs/prior-art.md](../prior-art.md) |
| Canonical external URLs | [docs/policies/SOURCES.md](../policies/SOURCES.md) |

---

## Open questions

1. **`--fit` vs explicit `-ngl` in `app/src/runtime/llama.rs`** — Change `gpu: auto` to omit `-ngl` and pass `--fit`, or keep a numeric layers API for YAML? Prior art prefers `--fit`.
2. **Multi-GPU / iGPU+dGPU** — Report all DXGI adapters; default offload to first `cuda_capable` only, or expose selection later?
3. **AMD/Intel Vulkan builds** — DXGI inventory only for v1, or detect Vulkan without vendor SDKs?
4. **CUDA build vs capability** — How does installer/launcher pick CPU vs CUDA `llama-server.exe` when profile says `cuda_capable` but only CPU binary is installed?
5. **Shared display GPU (6–8 GB laptops)** — Is DXGI `Budget` enough headroom signal, or always lean on `--fit`?
6. **hypomnesis as dep later** — Worth it if we add live VRAM metrics in Phase 5, or stay on thin DXGI forever?
