# API

WinServeAI is an **appliance wrapper** around `llama-server.exe`. It does not implement an OpenAI HTTP stack of its own. Clients talk to **llama-server’s OpenAI-compatible routes** at `/v1` once `ServerManager` reports READY.

```text
Client  →  http://{host}:{port}/v1/...   (llama-server)
App/CLI →  ServerManager                 (start / stop / wait ready)
```

There is **no chat UI**, no request proxy, and no plugin API. One backend only.

See [architecture.md](./architecture.md) for the full process boundary.

## Base URL

Derived from YAML (`server.host`, `server.port`):

```text
http://{server.host}:{server.port}/v1
```

Default (`config/default.yaml`):

```text
http://127.0.0.1:8080/v1
```

Helpers in `app/src/api/` only format these URLs for the manager and CLI. They are not an HTTP server.

## Endpoints

Served by **llama-server** (passthrough). Exact behavior follows the pinned llama.cpp release.

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/v1/models` | List loaded model(s); **readiness probe** |
| `POST` | `/v1/chat/completions` | Chat completions (primary client path) |
| `POST` | `/v1/completions` | Legacy text completions |
| `POST` | `/v1/embeddings` | Embeddings (when the build/model supports it) |

llama-server may also expose non-OpenAI routes (for example `GET /health`). WinServeAI does not require clients to use them.

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

Poll interval is ~250ms. Implementation: `app/src/server/health.rs`.

On success the manager logs and the CLI prints:

```text
READY http://127.0.0.1:8080/v1
```

Liveness (process alive) is not the same as readiness (model loaded). If the child exits during load, startup fails — do not treat a dead process as “still starting.”

## Client examples

Point any OpenAI-compatible client at the base URL. No API key is required for the default localhost bind unless you configure one on llama-server.

### curl — list models

```bash
curl http://127.0.0.1:8080/v1/models
```

### curl — chat completion

```bash
curl http://127.0.0.1:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "local",
    "messages": [{"role": "user", "content": "Hello"}],
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

## Out of scope

WinServeAI does not ship:

- A chat UI or web frontend for inference
- Model download / registry APIs
- Auth, multi-tenant gateways, or metrics dashboards
- Multi-backend or plugin endpoints

Those belong to external clients and later phases, not this appliance’s HTTP surface.

## Related

- [architecture.md](./architecture.md) — process ownership and startup sequence
- [configuration.md](./configuration.md) — `server.host` / `server.port` and model path
- [backend.md](./backend.md) — llama-server binary and pin notes
