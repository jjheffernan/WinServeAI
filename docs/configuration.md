# Configuration

Human-readable YAML is the source of truth. Schema and defaults live in `config/default.yaml` and `app/src/server/config.rs`.

**No raw llama.cpp flags in YAML.** Mapping to argv happens only in `app/src/runtime/llama.rs`.

## Default file

```yaml
server:
  port: 8080
  host: 127.0.0.1

model:
  path: D:\models\model.gguf

gpu:
  auto: true
  layers: auto

runtime:
  context: 32768
  flash_attention: true

logging:
  dir: logs
```

Copy `config/default.yaml`, set `model.path` to a real GGUF, then start.

## Where config is loaded

| Source | Path |
| --- | --- |
| Default | `{current working directory}/config/default.yaml` |
| Override | `WINSERVE_CONFIG` environment variable (absolute or CWD-relative path) |

```bash
# Windows PowerShell
$env:WINSERVE_CONFIG = "C:\path\to\winserve.yaml"
cargo run -p winserve -- start
```

```bash
# Unix-style (dev / CI)
WINSERVE_CONFIG=/path/to.yaml cargo run -p winserve -- start
```

If the file is missing, the CLI exits and tells you to copy the default and set `model.path`.

Useful checks:

```bash
cargo run -p winserve -- print-config   # load + print resolved YAML
cargo run -p winserve -- print-cmd      # binary + argv after hardware detect
```

## Schema

All top-level sections except `logging` are required in practice (they match the shipped default). `logging` may be omitted; it defaults to `dir: logs`.

### `server`

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `host` | string | `127.0.0.1` | Bind address passed to llama-server |
| `port` | u16 | `8080` | Listen port; must be non-zero |

OpenAI-compatible base URL: `http://{host}:{port}/v1` (see [api.md](api.md)).

### `model`

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `path` | path | `D:\models\model.gguf` (sample) | Path to a GGUF file |

Validation: path string must be non-empty. On `start`, the file must exist or startup fails.

### `gpu`

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `auto` | bool | `true` | Prefer llama.cpp auto-fit when layers are also `auto` |
| `layers` | string | `auto` | `auto`, or an integer layer count as a string (`"0"`, `"35"`, …) |

See [GPU rules](#gpu-rules) below. Do not put `--n-gpu-layers`, `--fit`, or other flags here.

### `runtime`

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `context` | u32 | `32768` | Context size (`--ctx-size`) |
| `flash_attention` | bool | `true` | When true, enables flash attention (`-fa on`) |

### `logging`

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `dir` | path | `logs` | Directory for `server.log`, `llama.log`, `error.log` |

See [logging.md](logging.md). Created when the manager loads config.

## Validation

`Config::validate()` (on load and save):

* `server.port` must be non-zero
* `model.path` must be non-empty

Additional checks at `start` (manager, not the YAML parser):

* `bin/llama-server` / `llama-server.exe` exists under the repo root
* `model.path` exists on disk
* Port bind probe may warn if the port looks taken (imperfect for `0.0.0.0` vs loopback)

## GPU rules

Hardware is detected at start (`system::detect()`). An empty GPU list means the CPU-only path.

| `gpu.auto` | `gpu.layers` | GPUs present? | What gets passed to llama-server |
| --- | --- | --- | --- |
| `true` | `auto` | no | `--n-gpu-layers 0` |
| `true` | `auto` | yes | `--fit on` (no `-ngl`) |
| `false` | `auto` | no | `--n-gpu-layers 0` |
| `false` | `auto` | yes | `--n-gpu-layers 99` |
| any | not `auto` (e.g. `"0"`, `"35"`) | ignored | `--n-gpu-layers {layers}` |

Shipped default (`auto: true`, `layers: auto`) is the recommended path: use llama.cpp fit when a GPU is present; otherwise force CPU layers to zero.

Escape hatches:

* Force CPU: `layers: "0"` (or rely on auto with no GPU)
* Pin a layer count: `layers: "35"` (disables fit for layer allocation)
* Disable fit preference but keep a coarse auto layer count: `auto: false` and `layers: auto` → `99` when a GPU is present

Always mapped from other sections (never written as flags in YAML):

* `--host`, `--port`, `--model`, `--ctx-size`
* `-fa on` when `runtime.flash_attention` is true

## Localhost vs LAN

| `server.host` | Who can connect | Firewall |
| --- | --- | --- |
| `127.0.0.1` or `localhost` | This machine only (MVP default, safe) | Do **not** open an inbound rule |
| `0.0.0.0` | All interfaces (LAN opt-in) | May need an inbound allow for `llama-server.exe` or the configured port |

LAN is opt-in. Clients on other machines should use the host’s LAN IP (for example `http://192.168.1.10:8080/v1`), not `http://0.0.0.0:8080/v1`.

Installer guidance: firewall rules only when bind is non-loopback ([installer.md](installer.md)).

## Examples

### Localhost, auto GPU (default)

```yaml
server:
  host: 127.0.0.1
  port: 8080
model:
  path: D:\models\my-model.gguf
gpu:
  auto: true
  layers: auto
runtime:
  context: 32768
  flash_attention: true
logging:
  dir: logs
```

### LAN bind, CPU-only

```yaml
server:
  host: 0.0.0.0
  port: 8080
model:
  path: D:\models\my-model.gguf
gpu:
  auto: true
  layers: "0"
```

### Explicit layer count

```yaml
gpu:
  auto: false
  layers: "35"
```

## Invariants

1. YAML is the only user-facing config format.
2. No raw llama.cpp flags in YAML — only high-level fields above.
3. Argv mapping lives solely in `app/src/runtime/llama.rs`.
4. UI / CLI talk to `ServerManager`; they do not invent flags ([architecture.md](architecture.md)).
