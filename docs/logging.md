# Logging

Unified streams under `logs/`:

| File | Contents |
| --- | --- |
| `server.log` | Manager lifecycle (start, stop, ready, errors) |
| `llama.log` | llama-server stdout/stderr |
| `error.log` | Failures, crash notes |

Always capture: stdout, stderr, exit codes, restart reasons.
