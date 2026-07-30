# Release Process

How WinServeAI ships: one crate (`winserve`), one external binary (`bin/llama-server.exe`), pinned llama.cpp **`b####`**.

See [architecture.md](architecture.md). Deep dives: [research/02-llama-readiness.md](research/02-llama-readiness.md) (pin policy), [research/04-installer-licensing.md](research/04-installer-licensing.md) (notices).

---

## Branches

| Branch | Role |
| --- | --- |
| **`main`** | **Releases only.** Stable tags, release candidates, and release artifacts ship from here. Do not land day-to-day work directly on `main`. |
| **`dev`** | **Default working branch.** Experimental / preview integration. Feature branches PR into `dev`. |

```text
feature/* ──PR──► dev ──release PR──► main ──tag──► stable
                    │
                    └── preview builds / soak (optional)
```

Day-to-day development happens on `dev` (or short-lived branches off `dev`). Promote `dev` → `main` only when cutting a release.

## Channels

```text
dev
  │
  ├── preview   # experimental / RC soak from dev (or release PR)
  │
main
  │
  └── stable    # supported release (tags on main only)
```

| Channel | Branch | When | Pin | Support |
| --- | --- | --- | --- | --- |
| **preview** | `dev` (or RC tag) | Ongoing integration / release candidate | Known-good `b####` (may move on `dev`) | Best-effort / soak |
| **stable** | `main` | After checklist + release PR | Fixed `b####`, validated | Supported |

Stable never ships an unrecorded llama-server build. Preview builds may track `dev` tip.

---

## Current ship pin (MVP)

| Field | Value |
| --- | --- |
| **llama.cpp tag** | [`b9866`](https://github.com/ggml-org/llama.cpp/releases/tag/b9866) |
| **Recorded in** | [`bin/VERSION`](../bin/VERSION) (one line: `b9866`) |
| **Fetch defaults** | `scripts/fetch-llama-pin.ps1` / `.sh` default `PIN=b9866` |
| **Notices** | [`notices/THIRD_PARTY_NOTICES.md`](../notices/THIRD_PARTY_NOTICES.md) lists **b9866** |
| **Installer** | Stage via `scripts/stage-release.*` after fetch; ship that same pin |

Preview/`dev` may keep this pin until a deliberate bump. Stable tags must list the
same `b####` in release notes and match `bin/VERSION`.

---

## Pin llama.cpp (`b####` in `bin/`)

Upstream releases use tags like [`b9866`](https://github.com/ggml-org/llama.cpp/releases/tag/b9866). Each WinServeAI release documents **exactly one** pin.

```text
bin/
  llama-server.exe    # from ggml-org/llama.cpp release b####
  VERSION             # committed one-line pin, e.g. b9866
  # CUDA builds: matching cudart / cublas DLLs beside the exe
```

Rules:

1. Place the binary at **`bin/llama-server.exe`** (external system — not compiled into `winserve`).
2. Keep the committed pin in **`bin/VERSION`** in sync with fetch-script defaults, `bin/README.md`, and `notices/THIRD_PARTY_NOTICES.md`. List the same tag in GitHub release notes.
3. On pin bump: replace `bin/` binary contents, update `bin/VERSION`, refresh notices (`PIN=b#### ./scripts/refresh-notices.sh`), re-run the stable checklist (argv / readiness can drift — see research/02).
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

## Sources / See also

### Internal

- [architecture.md](./architecture.md) — single-crate appliance boundary
- [backend.md](./backend.md) — pin policy and argv drift
- [installer.md](./installer.md) — install payload and firewall
- [development.md](./development.md) — build/test commands
- [contributing.md](./contributing.md) — PR target is `dev`
- [research/02-llama-readiness.md](./research/02-llama-readiness.md) — pin checklist (`/v1/models`, fit flags)
- [research/03-hardware-detection.md](./research/03-hardware-detection.md) — hardware matrix inventory
- [research/04-installer-licensing.md](./research/04-installer-licensing.md) — notices and CUDA DLLs
- [policies/doc-drift.md](./policies/doc-drift.md) — run `check_doc_drift.py` after material changes
- [bin/README.md](../bin/README.md)

### Upstream

- [llama.cpp releases (`b####`)](https://github.com/ggml-org/llama.cpp/releases)
- [API changelog #9291](https://github.com/ggml-org/llama.cpp/issues/9291) — REST surface drift on pin bump
- [llama.cpp LICENSE](https://github.com/ggml-org/llama.cpp/blob/master/LICENSE)
- [ollama#3185](https://github.com/ollama/ollama/issues/3185) — notices must ship with binaries

Canonical external URL index: [policies/SOURCES.md](./policies/SOURCES.md).
