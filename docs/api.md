# API

WinServeAI exposes an **OpenAI-compatible** HTTP endpoint served by the active backend (llama-server in v1).

## Endpoint

Derived from config:

```text
http://{server.host}:{server.port}/v1
```

## Health

Readiness and liveness probes are owned by `packages/api` and the Server Manager.

Typical checks:

* Process is running
* Backend reports healthy
* HTTP health/models endpoint responds within timeout

## Client Usage

Any OpenAI-compatible client can point at the local base URL. WinServeAI does not ship a chat UI.
