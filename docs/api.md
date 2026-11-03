# API

> **Readiness:** api 3.2/5 (`mvp-partial`), server-health 3.8/5 (`mvp-ready`) — details in [readiness/api.md](./readiness/api.md), [readiness/server-health.md](./readiness/server-health.md)

WinServeAI is an **appliance wrapper** around `llama-server.exe`. It does not implement an HTTP API stack of its own. Clients talk to **llama-server** once `ServerManager` reports READY. Common completion wire formats (chat, Responses, Anthropic Messages, legacy completions) are all llama-server routes on one process.

```text
Client  →  http://{host}:{port}/…   (llama-server)
App/CLI →  ServerManager            (start / stop / wait ready)
```

The server does not render a page. Frontends are separate clients. See [clients/README.md](../clients/README.md). When `server.api_key` is set, llama-server expects `Authorization: Bearer <key>` (OpenAI clients) or `x-api-key` / Bearer (Anthropic-style clients). One backend only. No request proxy.

See [architecture.md](./architecture.md) for the full process boundary.

## Base URLs

Derived from YAML (`server.host`, `server.port`):

| Client style | Base URL | Notes |
| --- | --- | --- |
| **OpenAI** | `http://{host}:{port}/v1` | SDKs append `/chat/completions`, `/models`, … |
| **Anthropic** | `http://{host}:{port}` | SDKs append `/v1/messages` themselves — do **not** put `/v1` on this base |

Default (`config/default.yaml`):

```text
OpenAI:    http://127.0.0.1:8080/v1
Anthropic: http://127.0.0.1:8080
```

If an Anthropic client 404s on `/v1/v1/messages`, the base was set to `…/v1` by mistake — drop the trailing `/v1`.

Helpers in `server/src/api/` only format these URLs for the manager and CLI. They are not an HTTP server.

## Endpoints

Served by **llama-server** (passthrough). Exact behavior follows the pinned llama.cpp release (**b9866**).

### Completion methods (supported)

| Method | Path | Client / OpenClaw `api` | Notes |
| --- | --- | --- | --- |
| `POST` | `/v1/chat/completions` | OpenAI SDKs · `openai-completions` | Primary chat path |
| `POST` | `/v1/responses` | OpenAI Responses · `openai-responses` | Translated to chat internally; use OpenAI base (`…/v1`) |
| `POST` | `/v1/messages` | Anthropic SDKs · `anthropic-messages` | Use Anthropic base (`…` without `/v1`) |
| `POST` | `/v1/completions` | Legacy OpenAI text | Raw `prompt` string; no chat template |

`scripts/smoke-openai.ps1` hits all four after `GET /v1/models` is ready (unless `-SkipChat`).

### Related (not completions)

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/v1/models` | List loaded model(s); **readiness probe** |
| `POST` | `/v1/messages/count_tokens` | Anthropic token count (no generation) |
| `POST` | `/v1/embeddings` | Embeddings (when the build/model supports it) |

llama-server may also expose `GET /health` and other native routes. WinServeAI does not require clients to use them.

## Readiness

`ServerManager` does **not** treat process spawn as ready. After spawn it polls until the model is loaded:

```text
GET http://{host}:{port}/v1/models
```

| Response | Meaning |
| --- | --- |
| **200** | Ready — OpenAI routes are usable |
| **503** | Still loading — keep waiting |
| Connection refused / other errors | Not listening yet — keep waiting |
| No success within **120s** | Startup fails (timeout) |

Poll interval is ~250ms. Implementation: [`server/src/server/health.rs`](../server/src/server/health.rs) (`wait_until_ready`).

**Primary:** `GET {base}/v1/models` — HTTP 2xx ⇒ ready.  
**Fallback:** if models is not yet ready, try `GET {base}/health` — HTTP 2xx ⇒ ready (many llama-server builds return 503 while loading, 200 when ready).  
Either probe returning 503 (or connection errors) means keep waiting until the overall deadline.

On success the manager logs and the CLI prints:

```text
READY http://127.0.0.1:8080/v1
```

Liveness (process alive) is not the same as readiness (model loaded). If the child exits during load, startup fails — do not treat a dead process as “still starting.”

llama-server may also expose `GET /health` (503 while loading, 200 when ready) on many builds. WinServeAI uses it as a **fallback** readiness signal when `/v1/models` is not yet succeeding; `/v1/models` remains the primary probe. Upstream semantics: [llama.cpp server README](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md), [PR #9056](https://github.com/ggml-org/llama.cpp/pull/9056).

## Client examples

Point any OpenAI-compatible client at the base URL. No API key is required for the default localhost bind unless you configure one on llama-server.

### curl — list models

```bash
curl http://127.0.0.1:8080/v1/models
```

### curl — OpenAI chat completion

```bash
curl http://127.0.0.1:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "local",
    "messages": [{"role": "user", "content": "Hello"}],
    "max_tokens": 64
  }'
```

### curl — OpenAI Responses

```bash
curl http://127.0.0.1:8080/v1/responses \
  -H "Content-Type: application/json" \
  -d '{
    "model": "local",
    "input": "Hello",
    "max_output_tokens": 64
  }'
```

### curl — Anthropic Messages

```bash
curl http://127.0.0.1:8080/v1/messages \
  -H "Content-Type: application/json" \
  -d '{
    "model": "local",
    "max_tokens": 64,
    "messages": [{"role": "user", "content": "Hello"}]
  }'
```

### curl — legacy OpenAI completions

```bash
curl http://127.0.0.1:8080/v1/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "local",
    "prompt": "Hello",
    "max_tokens": 64
  }'
```

Use the `id` from `GET /v1/models` as `model` if your client requires an exact match.

### OpenAI Python SDK

```python
from openai import OpenAI

client = OpenAI(
    base_url="http://127.0.0.1:8080/v1",
    api_key="not-needed",
)

print(client.models.list())

resp = client.chat.completions.create(
    model="local",
    messages=[{"role": "user", "content": "Hello"}],
    max_tokens=64,
)
print(resp.choices[0].message.content)
```

### OpenAI Node SDK

```js
import OpenAI from "openai";

const client = new OpenAI({
  baseURL: "http://127.0.0.1:8080/v1",
  apiKey: "not-needed",
});

const models = await client.models.list();
console.log(models);

const resp = await client.chat.completions.create({
  model: "local",
  messages: [{ role: "user", content: "Hello" }],
  max_tokens: 64,
});
console.log(resp.choices[0].message.content);
```

### OpenClaw (custom provider)

Same Ready process; pick one `api` per provider entry. Smoke all completion routes with `scripts/smoke-openai.ps1`.

| OpenClaw `api` | `baseUrl` | Route |
| --- | --- | --- |
| `openai-completions` | `http://127.0.0.1:8080/v1` | `/v1/chat/completions` |
| `openai-responses` | `http://127.0.0.1:8080/v1` | `/v1/responses` |
| `anthropic-messages` | `http://127.0.0.1:8080` | `/v1/messages` |

```json5
{
  models: {
    providers: {
      winserve: {
        baseUrl: "http://127.0.0.1:8080/v1", // or …:8080 for anthropic-messages
        apiKey: "local",
        api: "openai-completions", // or openai-responses | anthropic-messages
        models: [{ id: "local", name: "WinServeAI", contextWindow: 32768, maxTokens: 8192 }]
      }
    }
  }
}
```

## Out of scope

WinServeAI does not ship:

- A chat page inside the server. The optional page is `clients/chat`
- Model download / registry APIs
- Auth, multi-tenant gateways, or metrics dashboards
- Multi-backend or plugin endpoints
- A separate multi-protocol gateway — completion methods are llama-server routes on one process
- Guarantees that every strict Responses streaming client (e.g. some AI SDK versions) accepts upstream SSE shape — prefer chat or Anthropic if a client rejects `/v1/responses` streams

Those belong to external clients and later phases, not this appliance’s HTTP surface.

## Sources / See also

### Internal

- [architecture.md](./architecture.md) — process ownership and startup sequence
- [configuration.md](./configuration.md) — `server.host` / `server.port` and model path
- [backend.md](./backend.md) — llama-server binary and pin notes
- [research/02-llama-readiness.md](./research/02-llama-readiness.md) — probes, `--fit`, pin policy
- [research/05-server-manager.md](./research/05-server-manager.md) — Ready vs Starting state machine
- [prior-art.md](./prior-art.md) — liveness vs readiness patterns
- [`server/src/server/health.rs`](../server/src/server/health.rs) — primary probe implementation
- [`server/src/api/`](../server/src/api/) — OpenAI URL helpers (passthrough only)

### Upstream

- [llama.cpp `tools/server` README](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md) — `/v1` routes, optional `/health`
- [PR #9056](https://github.com/ggml-org/llama.cpp/pull/9056) — `/health` is not “slot free”; load fail → exit 1
- [Issue #20684](https://github.com/ggml-org/llama.cpp/issues/20684) — probes can stall under load
- [llm-d readiness probes](https://github.com/llm-d/llm-d/blob/main/docs/readiness-probes.md) — mental model: liveness ≠ readiness

Canonical external URL index: [policies/SOURCES.md](./policies/SOURCES.md).
