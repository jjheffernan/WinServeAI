# Research: `api.md`

**Target:** rewrite `docs/api.md` for the appliance architecture.  
**Sources:** [architecture.md](../../architecture.md), [prior-art.md](../../prior-art.md), [02-llama-readiness.md](../02-llama-readiness.md), `app/src/api/`, `app/src/server/health.rs`.

## Problem with current stub

`docs/api.md` still says readiness is owned by `packages/api` and “the active backend.” That layout is gone. WinServeAI is a **single-crate appliance wrapper** around `llama-server.exe` — no `packages/*`, no backend trait, no plugins, no HTTP proxy.

## Product surface (what clients actually hit)

| Layer | Role |
| --- | --- |
| `ServerManager` (`app/src/server/`) | Start / stop / status; waits for readiness |
| `app/src/api/` | **URL helpers only** (`openai_v1_url`, `models_url`, `chat_completions_url`) — not an HTTP server |
| `bin/llama-server.exe` | Serves OpenAI-compatible routes at `http://{host}:{port}/v1` |

Clients talk to **llama-server directly**. WinServeAI does not re-implement chat completions, auth, or request routing.

Default bind (from `config/default.yaml`): `127.0.0.1:8080` → base URL `http://127.0.0.1:8080/v1`.

## OpenAI-compatible endpoints (passthrough)

Upstream: [llama.cpp tools/server README](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md). Surface drifts per pinned `b####` release ([changelog #9291](https://github.com/ggml-org/llama.cpp/issues/9291)).

Document the routes clients need for MVP smoke and OpenAI SDK usage:

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/v1/models` | List loaded model(s); **readiness probe** |
| `POST` | `/v1/chat/completions` | Chat (primary client path) |
| `POST` | `/v1/completions` | Legacy text completions |
| `POST` | `/v1/embeddings` | Embeddings (when model/build supports it) |

Optional upstream (mention only as “llama-server may also expose”): `/health`, `/v1/health` — **not** what WinServeAI polls today.

## Readiness (code truth)

Architecture and implementation agree: **READY = `GET /v1/models` returns 200**.

From `app/src/server/health.rs`:

- Poll `GET {base_url}/v1/models` every **250ms**
- Per-request HTTP timeout: **2s**
- Overall wait: **120s** (`ServerManager` startup)
- **503** → still loading (keep waiting)
- Other non-success / connect errors → retry until deadline
- Timeout → `HealthError::Timeout` → startup fails (not Ready)

Prior-art / `02-llama-readiness.md` recommend `GET /health` as the primary probe. That is valid upstream guidance, but **product code and architecture.md use `/v1/models`**. Document the implemented probe; note `/health` as an alternate clients may use, not as WinServeAI’s gate.

Liveness (process alive) ≠ readiness (model loaded). Process exit during load (code 1) is a hard failure — do not poll forever ([PR #9056](https://github.com/ggml-org/llama.cpp/pull/9056)). Under load, HTTP probes can stall ([#20684](https://github.com/ggml-org/llama.cpp/issues/20684)); MVP is single-user, generous timeout is enough.

Startup sequence (architecture):

```text
read config → detect GPU → build command → spawn → wait /v1/models → READY
```

On Ready, manager logs `READY {openai_v1_url}` and CLI prints the same.

## Client examples (required in `api.md`)

1. **curl** — `GET /v1/models`, minimal `POST /v1/chat/completions`
2. **OpenAI Python SDK** — `base_url="http://127.0.0.1:8080/v1"`, any API key (llama-server ignores unless configured)
3. Emphasize: **no chat UI** ships; API is the product (Ollama-style positioning from prior-art)

## Explicit non-goals (keep out of `api.md`)

- Chat UI, model downloads, HuggingFace, agents, RAG, MCP
- Multi-backend / plugin / gateway APIs
- Auth, metrics dashboards, remote management
- Re-documenting every llama-server flag or non-OpenAI route
- Any `packages/api` or monorepo package references

## Outline for rewrite

1. One-paragraph positioning: passthrough appliance, not a proxy platform
2. Base URL from config
3. Endpoint table
4. Readiness (`/v1/models`, 503, timeout)
5. Client examples (curl + OpenAI SDK)
6. Link to architecture / configuration; no chat UI note

## Links

| Topic | Path / URL |
| --- | --- |
| Architecture | [architecture.md](../../architecture.md) |
| Prior art (API-as-product) | [prior-art.md](../../prior-art.md) |
| Readiness deep dive | [02-llama-readiness.md](../02-llama-readiness.md) |
| URL helpers | `app/src/api/openai.rs` |
| Probe | `app/src/server/health.rs` |
| Upstream server | https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md |
