# bin/

External **llama-server** binary (not built by this repo).

## Pin (local / dev)

| Field | Value |
| --- | --- |
| **Upstream** | [ggml-org/llama.cpp](https://github.com/ggml-org/llama.cpp) |
| **Pinned release** | [`b9866`](https://github.com/ggml-org/llama.cpp/releases/tag/b9866) |
| **Windows asset** | Prefer the CUDA build matching your driver (e.g. `llama-b9866-bin-win-cuda-12.4-x64.zip`), or CPU/Vulkan if no NVIDIA GPU |

```text
bin/llama-server.exe   # Windows (required for winserve start)
bin/llama-server       # non-Windows (optional for local experiments)
```

### Setup

**Helper (preferred):**

```powershell
# Windows — CPU build into bin/ (default pin b9866)
.\scripts\fetch-llama-pin.ps1

# Or CUDA / Vulkan
.\scripts\fetch-llama-pin.ps1 -Variant cuda-12.4
.\scripts\fetch-llama-pin.ps1 -Variant vulkan
```

```bash
# macOS/Linux helper (can also fetch Windows zips into bin/)
./scripts/fetch-llama-pin.sh --variant win-cpu
```

**Manual:**

1. Download the release asset for **b9866** from GitHub Releases.
2. Extract `llama-server.exe` (and CUDA runtime DLLs if present) into this `bin/` directory.
3. Keep DLLs **beside** `llama-server.exe` (PATH is unreliable on Windows).
4. `cargo run -p winserve -- print-cmd` then `start`.

Do **not** commit the binary or DLLs (gitignored). Document pin bumps in [release-process.md](../docs/release-process.md).

Ship MIT notices for llama.cpp with the installer — stub: [`notices/THIRD_PARTY_NOTICES.md`](../notices/THIRD_PARTY_NOTICES.md).

## See also

- [docs/backend.md](../docs/backend.md)
- [docs/release-process.md](../docs/release-process.md)
- [docs/research/02-llama-readiness.md](../docs/research/02-llama-readiness.md)
- [docs/research/04-installer-licensing.md](../docs/research/04-installer-licensing.md)
