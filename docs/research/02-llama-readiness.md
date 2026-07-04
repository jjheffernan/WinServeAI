# llama-server Readiness & Fit

> **Path note (appliance layout):** This note was written against an earlier `packages/*` monorepo. Map old paths to the current single crate:
> `packages/launcher` → `app/src/server/manager.rs` · `packages/process` → `app/src/runtime/process.rs` · `packages/llama` → `app/src/runtime/llama.rs` · `packages/api` / readiness → `app/src/server/health.rs` + `app/src/api/` · `packages/hardware` → `app/src/system/` · `packages/config` → `app/src/server/config.rs` · `packages/logging` → `app/src/server/logs.rs` · `packages/backend` / traits → **removed** (no backend trait).

Research for WinServeAI process readiness, health probes, auto GPU fit, release pinning, and port conflicts. No production code — actionable guidance for llama runtime (`app/src/runtime/llama.rs`) and readiness (`app/src/server/health.rs`).

Upstream: [ggml-org/llama.cpp `tools/server`](https://github.com/ggml-org/llama.cpp/tree/master/tools/server) · [server README](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md). Internal context: [prior-art.md](../prior-art.md).

## WinServeAI readiness (product)

**WinServeAI readiness probe today = `GET /v1/models`** (see [`app/src/server/health.rs`](../../app/src/server/health.rs)). Poll until **200**; treat connection refused / non-200 as still starting; overall timeout (e.g. 120s). Process alive ≠ ready.

`GET /health` (503 while loading, 200 when ready) is a **useful alternate** when present on the pinned `llama-server` build — do **not** treat it as the only probe, and do not document it as WinServeAI’s gate. Operator surface: [api.md](../api.md).

## Upstream facts

### `GET /v1/models` (WinServeAI primary readiness)

- OpenAI-compatible model list; single element in single-model mode.
- While loading: middleware **503** on routes (not a bypass).
- When ready: **200** with `data[0].id` (path or `--alias`). `meta` may be `null` in some modes while loading ([README](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md)).
- **Product code polls this endpoint** (`app/src/server/health.rs`). Also useful for client smoke (`scripts/smoke-openai.ps1`). Mental model: [llm-d readiness probes](https://github.com/llm-d/llm-d) (`/health` vs `/v1/models`) — liveness ≠ readiness.

### `GET /health` (optional alternate when available)

| State | Status | Body |
| --- | --- | --- |
| Model loading | **503** | `{"error":{"code":503,"message":"Loading model","type":"unavailable_error"}}` |
| Ready | **200** | `{"status":"ok"}` |

- Public (no API key). Alias: `/v1/health`. Present on many llama-server builds; **not** what WinServeAI polls today.
- [PR #9056](https://github.com/ggml-org/llama.cpp/pull/9056): `/health` is **not** “slot free”; slot checks live on `/slots`. While loading, **any** endpoint returns 503 via middleware. Failed model load → process **exits with code 1** (do not poll forever).
- Under heavy load, `/health` (and other routes) can queue behind inference and miss short timeouts ([#20684](https://github.com/ggml-org/llama.cpp/issues/20684)). Fast-path PR [#20799](https://github.com/ggml-org/llama.cpp/pull/20799) closed in favor of dynamic HTTP threads ([#20817](https://github.com/ggml-org/llama.cpp/pull/20817)). For MVP (single-user): use **generous probe timeouts** (5–30s), not 1s K8s-style liveness.
- Sleep mode (`--sleep-idle-seconds`): `/health` does **not** wake the model or reset idle ([README](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md)). Leave sleep **off** for MVP.

### Probe state machine (recommended — matches product)

```text
spawn llama-server
  │
  ├─ process exits (code ≠ 0)     → Failed (bad model / bind / crash)
  ├─ connection refused           → Starting (not listening yet)
  ├─ GET /v1/models → 503         → Starting (loading)
  ├─ GET /v1/models → 200         → Ready
  └─ timeout (e.g. 120–300s)      → Failed (stuck load)
```

Optional alternate when `/health` exists on the pin: same 503/200 semantics. Do not require `/health` for Ready.

### `--fit` auto VRAM

From [Discussion #18049](https://github.com/ggml-org/llama.cpp/discussions/18049) and current CLI:

| Flag | Role |
| --- | --- |
| `-fit, --fit on\|off` | Auto-adjust **unset** memory args to free VRAM. **Default: `on`**. |
| `-fitt, --fit-target` | Free MiB margin per device (default 1024). |
| `-fitc, --fit-ctx` | Min context fit may set (default 4096). |

**Critical:** if the user sets `--n-gpu-layers` / `-ngl`, `--tensor-split`, or `--override-tensor`, fit **does not** change those allocations. Explicit `--ctx-size` is also left alone. Collaborator confirmation: passing `-ngl` (even `999`) disables fit for layers ([#18049 reply](https://github.com/ggml-org/llama.cpp/discussions/18049#discussioncomment-15400000)).

Current CLI also documents `-ngl` default as `auto` and accepts `auto` / `all` ([README flags](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md)). Safest auto path: **omit `-ngl` entirely** so fit owns allocation. Fit can take **seconds to tens of seconds** (multi-GPU) — budget that into readiness timeout.

**Do not** reimplement layer math in `app/src/system/` for auto. Inventory (DXGI / NVML) is for UI and diagnostics only.

### Release pinning & CLI drift

- Pin binaries under **`bin/`** to a **`b####` tag** (e.g. [b9866](https://github.com/ggml-org/llama.cpp/releases/tag/b9866)), not `master`. Record tag + commit in `docs/backend.md` / release docs.
- REST surface churn: [API changelog #9291](https://github.com/ggml-org/llama.cpp/issues/9291). Re-verify `/v1/models` (and `/health` if used) bodies and fit flags on each pin bump.
- Recent drift examples: `/health` semantics (#9056), `--fit` (post-#16653 / #18049), `-ngl auto|all`, sleep idle, router mode. Treat argv mapping as **pin-specific** and regression-test on upgrade.

### Port conflicts

Patterns to steal (Ollama issues, llama sleep/wake):

1. **Preflight (optional):** before spawn, check if `server.port` is already listening (`Get-NetTCPConnection -LocalPort` / bind probe). If occupied by **our** PID → treat as already running; if foreign → clear `PortInUse { port, pid? }` error (do not auto-kill strangers).
2. **Post-spawn:** watch child exit + stderr for bind failures (`address already in use` / Windows “only one usage of each socket address”). Map to same error.
3. **Orphans:** Win11 sleep/wake can leave duplicate `llama-server` ([#20648](https://github.com/ggml-org/llama.cpp/discussions/20648)) — on start, if we own a prior job/PID, stop it; optionally kill **only** listeners we previously spawned (tracked PID), never arbitrary processes on the port.
4. **No auto port-hop** in MVP — fail with actionable message; user changes YAML `server.port`.
5. Ollama references: [ollama#3575](https://github.com/ollama/ollama/issues/3575), [ollama#12206](https://github.com/ollama/ollama/issues/12206) (Windows HNS “ghost” ports — rare; document reboot/HNS restart as last resort).

---

## Recommendations for llama runtime and readiness (`app/src/runtime/llama.rs`, `app/src/server/health.rs`)

### Gap today (historical monorepo stubs)

| Location (old → new) | Current behavior | Problem |
| --- | --- | --- |
| `packages/llama` → `app/src/runtime/llama.rs` `build_args` | Always passes `--n-gpu-layers` (auto → `auto_gpu_layers()` → `99`) | **Disables `--fit`**; invents layer counts prior-art says to avoid |
| llama `health` | `healthy: state == Running` | Process start ≠ model ready |
| llama `start` | Sets `Running` immediately after spawn | Should stay `Starting` until probe succeeds |
| readiness (`app/src/server/health.rs`) | Must poll HTTP | No stub — implement `GET /v1/models` |
| `packages/hardware` → `app/src/system/` `auto_gpu_layers` | Placeholder `99` / `0` | Wrong abstraction for auto |

### Readiness (`app/src/server/health.rs`)

- Implement `wait_until_ready(endpoint, timeout)`:
  - Poll **`GET /v1/models`** (primary; matches product).
  - **503** → continue (loading).
  - **200** → ready.
  - Connect errors → continue (not up).
  - Other 4xx/5xx → fail or retry with backoff (pin-dependent).
- Optional alternate: poll `GET /health` when present on the pin (503 loading / 200 ready) — never the only documented probe.
- Concurrently: caller watches process exit (code 1 = load failure).
- Defaults: interval ~250–500ms; overall timeout **≥ 120s** (large models + fit); per-request timeout **≥ 5s** (load under contention).
- Keep UI-agnostic: only host/port URLs, no llama flags.

### Llama runtime (`app/src/runtime/llama.rs`) (argv + lifecycle)

- **`gpu_layers: auto`:** omit `-ngl` / `--n-gpu-layers`. Rely on default `--fit on`. Do not call `auto_gpu_layers`.
- **`gpu_layers: "0"` / off:** pass `--n-gpu-layers 0` (CPU-only; fit won’t move layers).
- **`gpu_layers: N`:** pass `--n-gpu-layers N` (user override; fit leaves ngl alone).
- **`context: auto`:** omit `--ctx-size` (fit may shrink context).
- **`context: N`:** pass `--ctx-size N`.
- After spawn: state `Starting` → wait until `GET /v1/models` is ready → `Ready` / `Failed` / `Crashed`.
- Prefer live readiness probe over local state alone; map 200 → healthy, 503 → starting/not-ready, dead process → crashed.
- Surface bind/load failures from exit code + last stderr lines.

### Hardware inventory (`app/src/system/`)

- Inventory only (GPU name, VRAM, CUDA). Delete or stop using `auto_gpu_layers` for launch argv.
- Optional later: expose `recommended: { fit: true }` for UI copy — not layer numbers.

### Server Manager (`app/src/server/manager.rs`)

- Sole owner of start → wait-ready → status. Do not mark Ready until readiness succeeds.
- Port preflight before spawn.

---

## Config mapping (YAML field → argv, including auto)

Human YAML only; never store raw llama flags.

| YAML | Argv when `auto` / default | Argv when explicit |
| --- | --- | --- |
| `server.host` | `--host {host}` | same |
| `server.port` | `--port {port}` | same |
| `model.path` | `--model {path}` | same |
| `performance.context: auto` | *(omit `--ctx-size`)* — fit may set ≥ `--fit-ctx` | `context: 8192` → `--ctx-size 8192` |
| `performance.gpu_layers: auto` | *(omit `-ngl`)* — `--fit` default `on` owns VRAM | `gpu_layers: 0` → `--n-gpu-layers 0`; `gpu_layers: 35` → `--n-gpu-layers 35` |
| `performance.flash_attention: auto` | *(omit; pin-default)* | map only after verifying flag on pinned `b####` (`-fa` / `--flash-attn`) |
| *(implicit)* | rely on `--fit on` default; optional explicit `--fit on` for clarity in logs | explicit layers: optional `--fit off` (redundant if `-ngl` set) |

**Anti-pattern (current stub):** `gpu_layers: auto` → `--n-gpu-layers 99`. That turns fit off and over-commits VRAM on small GPUs.

**Optional advanced (not MVP YAML):** `--fit-target`, `--fit-ctx` — only if we add first-class fields later; do not expose as free-form flags.

---

## Links

| Topic | URL |
| --- | --- |
| Server README (`/health`, `/v1/models`, flags) | https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md |
| `/health` refactor (503 loading, exit 1 on fail) | https://github.com/ggml-org/llama.cpp/pull/9056 |
| `/health` under load | https://github.com/ggml-org/llama.cpp/issues/20684 |
| Dynamic HTTP threads (load mitigation) | https://github.com/ggml-org/llama.cpp/pull/20817 |
| `--fit` design & `-ngl` disables fit | https://github.com/ggml-org/llama.cpp/discussions/18049 |
| Auto n-gpu-layers history | https://github.com/ggml-org/llama.cpp/pull/14067 |
| REST API changelog | https://github.com/ggml-org/llama.cpp/issues/9291 |
| Releases (`b####` tags) | https://github.com/ggml-org/llama.cpp/releases |
| Win11 sleep/wake orphans | https://github.com/ggml-org/llama.cpp/discussions/20648 |
| Ollama port-in-use (Windows) | https://github.com/ollama/ollama/issues/3575 |
| Ollama port / HNS edge cases | https://github.com/ollama/ollama/issues/12206 |
| Prior art (this repo) | [docs/prior-art.md](../prior-art.md) |
| Operator API guide | [docs/api.md](../api.md) |
| Product readiness code | [`app/src/server/health.rs`](../../app/src/server/health.rs) |
| Canonical external URLs | [docs/policies/SOURCES.md](../policies/SOURCES.md) |

---

## Open questions

1. **Readiness timeout default** — 120s vs 300s for large GGUFs + fit on low-end Windows laptops?
2. **`--fit` on 6–8 GB shared display GPUs** — acceptable quality, or document “set `gpu_layers: 0` / explicit N” escape hatch only?
3. **Pin policy** — how often to bump `b####`; mandatory checklist: `/v1/models` (and `/health` if used) bodies, fit flags, `-fa` name, OpenAI `/v1` smoke.
4. **Health timeout while serving** — for tray “healthy?” under load, use PID+port liveness vs HTTP, or only HTTP with ≥5s timeout?
5. **Port preflight vs post-fail** — is bind-before-spawn worth it, or only parse child exit/stderr?
6. **`flash_attention: auto`** — confirm exact argv on chosen pin before mapping.
7. **Sleep mode later** — if enabled, does `/health` stay 200 while model unloaded? (MVP: leave off.)
