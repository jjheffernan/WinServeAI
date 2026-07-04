# Configuration

Source of truth: `config/default.yaml`.

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

## Rules

* No raw llama.cpp flags in YAML
* Mapping to argv happens only in `app/src/runtime/llama.rs`
* `gpu.auto` + `layers: auto` → prefer `--fit on` when a GPU is present; else `--n-gpu-layers 0`
* Default host is localhost (safe). LAN bind (`0.0.0.0`) is opt-in and may need a firewall rule
