# Competitor research status

**Polled:** 2026-07-30  
**Basis:** Pinned briefs listed in `PLAN.md`; no new upstream research.

`PASS` means the brief has pinned, path-backed evidence for the architectural dimensions needed by the synthesis. `GAP` means a material claim remains inferred or unverified and should be checked before final comparison.

| Brief | Pin | Status | Evidence quality |
| --- | --- | --- | --- |
| `llama-cpp-server.research.md` | `b9866` / `75a48a90559abf65df3f3616a53bb16e5afb9d07` | PASS | Strong path-backed coverage of process modes, HTTP/readiness, packaging, and real-process tests. Native desktop/tray absence is only inferred, but it is not needed for the core engine topology. |
| `ollama.research.md` | `v0.32.5` / `eec8e0b9458b8a01be0c216a9cc53eefde24ef50` | GAP | Strong evidence for nested desktop→daemon→runner ownership, API, storage, packaging, and tests. cgroup absence is unverified; lack of a runner Job Object test is inferred. |
| `jan.research.md` | `v0.8.4` / `5f30aee467f08941964a83f946e2663e7ae0e01f` | GAP | Exceptionally detailed path and test evidence for router ownership, readiness, shutdown, proxying, downloads, and packaging. Minimum upstream router-mode build and abnormal parent-death reclamation remain inferred. |
| `localai.research.md` | `v4.7.1` / `b224c96db6f4b87306a33a808650bfce63b12588` | GAP | Strong evidence for local/remote/distributed gRPC backend ownership, YAML selection, lifecycle, packaging, and boundary tests. Job Object/cgroup absence and machine-wide launcher singleton behavior are inferred from scoped files. |
| `gpt4all.research.md` | `v3.10.0` / `228d5379cfb54e966449e153082c74c66b27c6c9` | PASS | Strong evidence for in-process inference, embedded localhost API, storage, packaging, and API tests. Historical Docker API material is explicitly unverified and excluded from the pinned architecture. The embedded-API pytest harness is Unix-gated (XDG ini override), so it is not Windows evidence, and the brief records no readiness probe. |

## Remaining gaps

Only gaps that could affect later architectural comparison are listed.

1. **Ollama containment:** verify repository-wide whether any daemon/runner path attaches `llama-server` to Windows Job Objects or Linux cgroups, and whether abnormal parent-death reclamation is tested.
2. **Jan router compatibility:** identify the minimum `llama-server` build that supports `--models-preset` and confirm whether Jan enforces it before launch.
3. **Jan abnormal termination:** verify behavior when the Tauri host is killed without cleanup, including router descendants beyond the direct-child PID sweep.
4. **LocalAI platform containment:** inspect platform-specific process-manager implementation before asserting the absence of Job Objects or cgroups.
5. **LocalAI singleton scope:** confirm whether code outside the cited launcher path supplies a machine-wide desktop/server singleton.

The inferred absence of a native desktop shell inside llama.cpp `tools/server` and GPT4All's stale historical Docker reference do not currently affect the ownership-family synthesis, provided the synthesis carries them as **inferred from source** rather than as facts.

## Source index

Missing CDN/model-host URLs were appended to [policies/SOURCES.md](../../policies/SOURCES.md) on 2026-07-30 (Jan CDN, Hugging Face, GPT4All gallery, `nomic-ai/llama.cpp`). Product repos + `janhq/llama.cpp` were already indexed.

## Accuracy review

Independent review published as [audit/competitor-accuracy.md](../../audit/competitor-accuracy.md). Overall gate: **PASS for comparison**. Residual uncertainties above remain accepted gaps (inferred/unverified), not hard facts.

## Recommended next

Done: accuracy review, `SOURCES.md` serialization, [comparison.md](../../comparison.md). Remaining optional: resolve residual containment gaps if a future research pass needs them as verified facts.
