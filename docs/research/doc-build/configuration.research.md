# Research: configuration.md

**Sources (code is truth):**

| Path | Role |
| --- | --- |
| `config/default.yaml` | Shipped default YAML |
| `app/src/server/config.rs` | Schema, defaults, load/save/validate, URL helpers |
| `app/src/runtime/llama.rs` | **Only** place that maps YAML → llama.cpp argv |
| `app/src/main.rs` | `WINSERVE_CONFIG` resolution, CLI (`print-config`, `print-cmd`) |
| `app/src/server/manager.rs` | Loads config, opens `logging.dir`, preflight model/port |
| `docs/architecture.md` | YAML source of truth; flags only in `runtime/llama` |
| `docs/installer.md` / `docs/research/04-installer-licensing.md` | Localhost vs LAN firewall |

**Current `configuration.md`:** thin example + three bullets. Needs full field table, GPU decision matrix, bind modes, env override.

---

## Schema (serde structs)

Top-level sections: `server`, `model`, `gpu`, `runtime`, `logging` (optional; defaults if omitted).

| Field | Type | Default | Validation |
| --- | --- | --- | --- |
| `server.host` | string | `127.0.0.1` | none in config; bind probe at start |
| `server.port` | u16 | `8080` | must be non-zero |
| `model.path` | path | `D:\models\model.gguf` (default struct / sample YAML) | non-empty string; **file must exist** at `start` |
| `gpu.auto` | bool | `true` | — |
| `gpu.layers` | string | `"auto"` | `"auto"` **or** integer as string (e.g. `"35"`, `"0"`) |
| `runtime.context` | u32 | `32768` | — |
| `runtime.flash_attention` | bool | `true` | — |
| `logging.dir` | path | `logs` | created on manager load |

`Config::load` / `save` both call `validate()`. Errors: `ConfigError::{Io, Parse, Validate}`.

URL helpers (not YAML):

* `base_url()` → `http://{host}:{port}`
* `openai_v1_url()` → `{base_url}/v1`

---

## Config path resolution

From `main.rs` `config_path`:

1. If `WINSERVE_CONFIG` is set → use that path (absolute or relative to process CWD).
2. Else → `{current_dir}/config/default.yaml`.

Missing file → CLI exits with “copy config/default.yaml and set model.path”.

Debug:

* `winserve print-config` — load + print resolved YAML
* `winserve print-cmd` — show binary + argv after hardware detect (via manager)

---

## GPU auto rules (`runtime/llama.rs`)

Hardware: `system::detect()` → `HardwareInfo.gpus` (empty ⇒ CPU-only path). GPU probe is still a stub (`detect_gpus` returns `[]` until DXGI/NVML); document the **intended** matrix as implemented in `build_command`.

| `gpu.auto` | `gpu.layers` | GPUs present? | Argv |
| --- | --- | --- | --- |
| `true` | `"auto"` | no | `--n-gpu-layers 0` |
| `true` | `"auto"` | yes | `--fit on` (no `-ngl`) |
| `false` | `"auto"` | no | `--n-gpu-layers 0` |
| `false` | `"auto"` | yes | `--n-gpu-layers 99` |
| any | not `"auto"` (e.g. `"0"`, `"35"`) | n/a | `--n-gpu-layers {layers}` |

Always mapped (not GPU-specific):

* `--host`, `--port`, `--model`, `--ctx-size` from server/model/runtime
* `runtime.flash_attention: true` → `-fa on`

**Invariant:** YAML never contains raw llama flags (`--fit`, `-ngl`, `-fa`, etc.). Users set high-level fields only.

---

## Localhost vs LAN

| `server.host` | Meaning | Firewall |
| --- | --- | --- |
| `127.0.0.1` / `localhost` | Loopback only (MVP default, safe) | Do **not** open inbound rule |
| `0.0.0.0` | All interfaces (LAN opt-in) | May need inbound allow for `llama-server.exe` / port |

Clients use `http://{host}:{port}/v1`. For LAN, clients use the machine’s LAN IP, not `0.0.0.0`.

Port preflight: `network::port_available(host, port)` — warning only if bind may fail (imperfect for `0.0.0.0` vs `127.0.0.1`).

---

## Out of scope for this doc

* Chat UI, model download, HuggingFace, auth, metrics
* Multi-backend / plugins
* Hot-reload of config while Ready (stop-then-start; not implemented as reload)

---

## Build checklist for `docs/configuration.md`

1. Full default YAML example (match `config/default.yaml`)
2. Field-by-field table with meanings + defaults
3. GPU decision matrix (auto / layers / CPU vs GPU)
4. Localhost vs LAN bind + firewall note
5. `WINSERVE_CONFIG` + default path
6. Explicit “no raw llama flags” + pointer to `app/src/runtime/llama.rs`
7. Link architecture / logging / api as needed; do not invent fields
