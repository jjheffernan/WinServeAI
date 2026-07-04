# Release Process

How WinServeAI ships: one crate (`winserve`), one external binary (`bin/llama-server.exe`), pinned llama.cpp **`b####`**.

See [architecture.md](architecture.md). Deep dives: [research/02-llama-readiness.md](research/02-llama-readiness.md) (pin policy), [research/04-installer-licensing.md](research/04-installer-licensing.md) (notices).

---

## Channels

```text
main
  │
  ├── nightly   # CI / soak from tip of main
  ├── preview   # RC before stable
  └── stable    # supported release
```

| Channel | When | Pin | Support |
| --- | --- | --- | --- |
| **nightly** | Scheduled or on-demand from `main` | Known-good `b####` (may move often) | Best-effort |
| **preview** | Feature-complete candidate | Fixed `b####` | Soak only |
| **stable** | After checklist | Fixed `b####`, validated | Supported |

Stable and preview never ship `master` or an unrecorded llama-server build.

---

## Pin llama.cpp (`b####` in `bin/`)

Upstream releases use tags like [`b9866`](https://github.com/ggml-org/llama.cpp/releases/tag/b9866). Each WinServeAI release documents **exactly one** pin.

```text
bin/
  llama-server.exe    # from ggml-org/llama.cpp release b####
  VERSION             # recommended: one line, e.g. b9866 (+ commit SHA)
  # CUDA builds: matching cudart / cublas DLLs beside the exe
```

Rules:

1. Place the binary at **`bin/llama-server.exe`** (external system — not compiled into `winserve`).
2. Record the pin in `bin/VERSION` (or equivalent) **and** in the GitHub release notes.
3. On pin bump: replace `bin/` contents, refresh notices, re-run the stable checklist (argv / readiness can drift — see research/02).
4. Do not embed llama.cpp source in the crate.

CUDA builds ship redistributable DLLs **next to** `llama-server.exe`. End users still need a compatible NVIDIA driver. Details: [research/04](research/04-installer-licensing.md).

---

## Hardware matrix (target smoke)

Run on as many of these as available before **stable**. Not every commit needs the full matrix.

| Target | Expectation |
| --- | --- |
| RTX 3060 | Start → ready (`GET /v1/models`) with a small GGUF |
| RTX 3080 | Same |
| RTX 4090 | Same |
| RTX 5090 | Future |
| CPU-only | Graceful path: starts without CUDA, or clear limited-support message — **not** a hard crash |

Note which GPUs were exercised in the release notes. Inventory policy: [research/03-hardware-detection.md](research/03-hardware-detection.md).

---

## Checklist (stable)

Single crate only — package name **`winserve`** (`app/`). There are no `packages/*` release units.

1. **Crate tests / build**
   ```bash
   cargo test -p winserve
   cargo build -p winserve --release
   ```
2. **Pinned binary present** — `bin/llama-server.exe` matches documented `b####` (`bin/VERSION`).
3. **Integration** — `winserve` start → wait ready on `/v1/models` → stop; smoke `print-cmd` if used in docs.
4. **Hardware smoke** — matrix above, as available.
5. **Installer** — clean install and uninstall ([installer.md](installer.md)); payload includes `winserve.exe`, `bin/llama-server.exe`, `config/default.yaml`, notices.
6. **Config** — migration notes from previous stable when YAML schema changed (when applicable).
7. **Notices** — see below; present in installer and any zip/portable artifact.
8. **Changelog** — winserve version + llama.cpp `b####` + notable changes.
9. **Tag and publish** — see Tags.

Preview uses the same checklist except hardware matrix may be partial. Nightly may skip installer/matrix.

---

## Notices

MIT requires copyright and permission notice in **binary** distributions. Repo `LICENSE` alone is not enough for shipped `llama-server.exe`.

Minimum per release artifact (installer and zip):

| Item | Source |
| --- | --- |
| `THIRD_PARTY_NOTICES` (rollup) | Component name, URL, license, **pinned `b####`** |
| llama.cpp `LICENSE` text | From the pinned tag |
| llama.cpp `AUTHORS` (preferred) | From the pinned tag |
| NVIDIA note | Only if CUDA DLLs are bundled |

Refresh notices whenever the `bin/` pin changes. Do not ship model weights (separate licenses; out of scope).

Install layout reference: [installer/README.md](../installer/README.md), [bin/README.md](../bin/README.md).

---

## Tags

| Channel | Tag pattern | Example |
| --- | --- | --- |
| stable | `vX.Y.Z` | `v0.1.0` |
| preview | `vX.Y.Z-rc.N` or `vX.Y.Z-preview.N` | `v0.1.0-rc.1` |
| nightly | `nightly-YYYYMMDD` (optional) | `nightly-20260704` |

- Tags version **winserve** only (the `app` crate). Do not invent per-package tags.
- Release body **must** list the llama.cpp pin (`b####` and commit if known).
- One tag ↔ one winserve version ↔ one documented `bin/` pin.

```text
# after checklist
git tag -a vX.Y.Z -m "vX.Y.Z (llama.cpp b####)"
git push origin vX.Y.Z
# attach installer + zip + checksums on the GitHub release
```
