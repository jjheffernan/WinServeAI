# Research: release-process.md

**Target:** rewrite [docs/release-process.md](../../release-process.md)  
**Architecture lock:** single crate `winserve` (`app/`), llama.cpp only, pin in `bin/`.  
**Problem with stub:** checklist says “workspace packages”; layout is appliance, not `packages/*`.

---

## Sources

| Source | Takeaway |
| --- | --- |
| [architecture.md](../../architecture.md) | `app/` + `bin/llama-server.exe`; one orchestrator |
| [AGENTS.md](../../../AGENTS.md) / [README.md](../../../README.md) | Crate name `winserve`; MIT + bundle llama notices |
| [bin/README.md](../../../bin/README.md) | External binary; `THIRD_PARTY_NOTICES` with installer |
| [installer/README.md](../../../installer/README.md) | Ship `winserve.exe` + `bin/llama-server.exe` + notices |
| [research/02-llama-readiness.md](../02-llama-readiness.md) | Pin **`b####`**, not `master`; re-verify flags/API on bump |
| [research/04-installer-licensing.md](../04-installer-licensing.md) | MIT notices in install payload; CUDA DLL notes; no models |
| [development.md](../../development.md) | `cargo check/run -p winserve` |
| Stub `release-process.md` | Channels + hardware matrix worth keeping; checklist wrong |

---

## Gaps in current stub

1. **“All workspace tests pass”** — implies multi-package workspace (`packages/config`, `packages/launcher`, …). Reality: Cargo workspace member is only `app` (`name = "winserve"`). Checklist must target that crate.
2. **Backend pinning** — vague “vetted llama.cpp release.” Canonical form is upstream tag **`b####`** (e.g. `b9866`), binary placed at **`bin/llama-server.exe`** (plus CUDA DLLs beside it when shipping CUDA). Record pin in release notes / `bin/` manifest, not a multi-backend vendor matrix.
3. **Notices** — missing. MIT requires license text in binary distributions ([ollama#3185](https://github.com/ollama/ollama/issues/3185) caution). Ship `THIRD_PARTY_NOTICES` (and llama.cpp `LICENSE` / `AUTHORS` from the pin) with installer and zip artifacts.
4. **Tags** — only mentions `vX.Y.Z`. Need channel → tag mapping (stable vs preview/nightly) and what the tag covers (winserve version + pinned `b####`).
5. **Hardware matrix** — keep as smoke targets; CPU-only is graceful path, not hard fail ([research/03](../03-hardware-detection.md)).

---

## Decisions for rewrite

### Channels

Keep three promotion lanes from `main`:

| Channel | Purpose | Artifacts |
| --- | --- | --- |
| **nightly** | Continuous / scheduled builds from `main` | Unsigned or CI-only; may track latest known-good `b####` or a floating pin |
| **preview** | Pre-stable soak (RC) | Installer + zip; pin must be fixed |
| **stable** | Supported release | Installer + zip; pin fixed and validated on matrix |

Stable never ships an unpinned or `master` llama-server.

### Pin location

```text
bin/
  llama-server.exe     # from ggml-org/llama.cpp release b####
  VERSION              # optional: one-line "b####" (+ commit SHA)
  # CUDA builds: cudart / cublas DLLs beside the exe
```

Do **not** document `packages/*` or a separate backend crate. Pin bump = replace `bin/` contents + refresh notices + re-run checklist.

### Single crate checklist

Gate on:

```bash
cargo test -p winserve
cargo build -p winserve --release
```

Plus integration against the **pinned** `bin/llama-server.exe` (start → `GET /v1/models` ready), installer clean install/uninstall, notices present, changelog, tag.

### Tags

| Channel | Git tag pattern | Notes |
| --- | --- | --- |
| stable | `vX.Y.Z` | Semver of **winserve**; release body lists `llama.cpp b####` |
| preview | `vX.Y.Z-preview.N` or `vX.Y.Z-rc.N` | Same pin rules as stable |
| nightly | `nightly-YYYYMMDD` (optional) | Not a support commitment |

One tag = one winserve version + one documented llama pin. Do not tag workspace packages separately (there are none).

### Notices (minimum per release)

From research/04, adapted to current layout (`THIRD_PARTY_NOTICES` at install root or `notices/`):

1. llama.cpp MIT license text from the **pinned** `b####` tag
2. AUTHORS (or equivalent) from that tag
3. NVIDIA note if CUDA DLLs are bundled
4. Refresh whenever `bin/` pin changes

Models are never shipped; model licenses are out of scope.

### Hardware matrix (smoke)

Keep stub list as **target** smoke hosts, not a CI gate for every commit:

* RTX 3060, 3080, 4090
* RTX 5090 (future)
* CPU-only (must start and serve, or clear limited-support message — not a crash)

Stable release: run matrix **as available**; document which GPUs were exercised in the release notes.

---

## Out of scope for this doc

* Auto-updater implementation (Phase 5+)
* Multi-backend / plugin release matrices
* Model marketplace or HuggingFace pins
* Per-package versioning

---

## Rewrite outline

1. Channels (diagram + table)
2. Pin llama.cpp `b####` in `bin/`
3. Hardware matrix
4. Stable checklist (`winserve` only)
5. Notices
6. Tags and release notes contents

Link to [backend.md](../../backend.md), [installer.md](../../installer.md), research/02 and research/04 instead of duplicating flag/legal depth.
