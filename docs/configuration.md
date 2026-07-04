# Configuration

Human-readable YAML. Never expose raw llama.cpp flags.

## Schema

```yaml
server:
  host: 0.0.0.0
  port: 8080

model:
  path: D:\Models\model.gguf

performance:
  context: auto
  gpu_layers: auto
  flash_attention: auto

logging:
  level: info
```

## Rules

* `auto` values are resolved by hardware detection at start time
* Explicit values override auto
* Schema validation runs on load and before start
* Migrations (Phase 4) must preserve user intent across versions

## Examples

See `examples/config/`.

## Research Notes

* **YAML** chosen for readability and familiarity on Windows
* TOML remains a possible alternate; migration strategy required if we ever switch
* Schema validation lives in `packages/config`
