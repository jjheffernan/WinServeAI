# First run

On the machine that will run the model:

1. Open **WinServeAI** from the Start menu.
2. Click **Browse** and choose your `.gguf` file.
3. Click **Save**, then **Start**.
4. When the window says **Ready**, click **Copy URL** or **Open chat**.

The address is `http://127.0.0.1:8080/v1`. Leave the API key empty when only this PC talks to it.

WinServeAI does not ship model weights. You bring the `.gguf`.

## If you would rather edit the file

```yaml
model:
  path: D:\models\my-model.gguf
```

That file is `config/default.yaml`. Then `winserve start` or `winserve serve`.

A dev tree also needs `bin/llama-server.exe` (pin **b9866**: `scripts/fetch-llama-pin.ps1`). The installer already includes it.

## What not to do

- Do not use the installer or window as a model marketplace.
- Do not put raw llama.cpp flags in YAML (flags stay in `server/src/runtime/llama.rs`).

## See also

- [configuration.md](configuration.md) — YAML schema
- [development.md](development.md) — CLI / scripts
- [specs/F-installer.md](specs/F-installer.md) — install → set path → Ready
- `config/FIRST_RUN.txt` — the same four steps, shipped next to the program
