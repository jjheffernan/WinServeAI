# Third-party notices (stub)

This directory ships with WinServeAI install/release trees. Refresh contents
whenever the pinned `bin/llama-server` build changes (see `bin/README.md`).

Pin documented here: **llama.cpp `b9866`**.

---

## WinServeAI

MIT — see repository root `LICENSE`.

---

## llama.cpp (ggml-org/llama.cpp)

Upstream: https://github.com/ggml-org/llama.cpp  
Pinned release: https://github.com/ggml-org/llama.cpp/releases/tag/b9866  
License file at pin: https://github.com/ggml-org/llama.cpp/blob/b9866/LICENSE

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

Optional attribution list: upstream `AUTHORS` at the same pin tag.

---

## NVIDIA CUDA redistributables (CUDA builds only)

When packaging a CUDA build of `llama-server.exe`, redistribute only Attachment A
runtime libraries from the matching CUDA Toolkit, beside the exe. See the
[CUDA Toolkit EULA](https://docs.nvidia.com/cuda/eula/). End users still need a
compatible NVIDIA GPU driver (drivers are not redistributed here).

Full NVIDIA notice text for installer payloads is tracked as installer work (F4).

---

## Model weights

Not bundled. Model files under `models/` (or configured `model.path`) remain the
operator’s responsibility and carry their own licenses.
