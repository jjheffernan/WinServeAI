# First run

WinServeAI does **not** ship model weights. You need a local **`.gguf`** before the
API can become Ready.

## Checklist

1. **Binary** — Installer includes `bin/llama-server.exe`. Dev trees: run
   `scripts/fetch-llama-pin.ps1` (or `.sh`) for pin **b9866**.
2. **Model path** — Point `model.path` at an existing `.gguf`:
   - **Desktop (`winserve-tray`)** — Settings → **Browse…** → **Save settings**
   - **YAML** — edit `config/default.yaml`:

     ```yaml
     model:
       path: D:\models\my-model.gguf
     ```

3. **Start** — tray **Start**, or `winserve start` / `winserve serve`.
4. **Client** — `GET http://127.0.0.1:8080/v1/models` (default loopback bind).

Empty `model.path` is valid in the shipped default so first-run is obvious;
`start` fails clearly until a real file is set.

## What not to do

- Do not use the installer or tray as a model download / marketplace UI.
- Do not put raw llama.cpp flags in YAML (flags stay in `app/src/runtime/llama.rs`).

## See also

- [configuration.md](configuration.md) — YAML schema
- [development.md](development.md) — CLI / scripts
- [specs/F-installer.md](specs/F-installer.md) — install → set path → Ready
- `config/FIRST_RUN.txt` — short copy shipped in `{app}`
