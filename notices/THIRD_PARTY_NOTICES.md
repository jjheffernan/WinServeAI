# Third-party notices

This directory ships inside WinServeAI install/release trees (`{app}\notices\`).
Refresh whenever the pinned `bin/llama-server` build changes (see `bin/VERSION`,
`bin/README.md`, and `scripts/refresh-notices.sh`).

| Component | Upstream | License | Pin / version |
| --- | --- | --- | --- |
| WinServeAI | https://github.com/jjheffernan/WinServeAI | MIT | 0.1.0 (repo `LICENSE`) |
| llama.cpp (`llama-server`) | https://github.com/ggml-org/llama.cpp | MIT | **b9866** |
| NVIDIA CUDA redistributables | https://docs.nvidia.com/cuda/eula/ | CUDA EULA Attachment A | CUDA builds only |

---

## WinServeAI

MIT — full text in the repository / install root `LICENSE`.

---

## llama.cpp (ggml-org/llama.cpp)

- Release: https://github.com/ggml-org/llama.cpp/releases/tag/b9866
- Full license text: [`llama.cpp-LICENSE.txt`](llama.cpp-LICENSE.txt)
- Authors / attribution list at pin: [`llama.cpp-AUTHORS.txt`](llama.cpp-AUTHORS.txt)

```text
MIT License

Copyright (c) 2023-2026 The ggml authors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

---

## NVIDIA CUDA redistributables (CUDA builds only)

When the staged `bin/` payload includes NVIDIA CUDA runtime libraries
(`cudart64_*.dll`, `cublas*`, etc.) beside `llama-server.exe`, see
[`NVIDIA-CUDA-NOTICE.txt`](NVIDIA-CUDA-NOTICE.txt). CPU/Vulkan-only packages
do not redistribute those DLLs.

---

## Model weights

Not bundled. Files under `models/` or configured `model.path` remain the
operator’s responsibility and carry their own licenses.
