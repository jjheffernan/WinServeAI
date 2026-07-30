# WinServeAI

> Windows-native **llama.cpp appliance wrapper** — one click to an OpenAI-compatible endpoint.

Not a multi-backend platform. Not a chat app. Not a model marketplace.

```text
Windows App → ServerManager → llama-server.exe → http://localhost:8080/v1
```

## Layout

```text
app/           ServerManager + runtime + system probes
bin/           llama-server.exe (external)
config/        default.yaml
logs/          server.log, llama.log, error.log
models/        your GGUF files (not shipped)
installer/     native Windows installer
scripts/       start.ps1, stop.ps1, reset.ps1
docs/          architecture, prior-art, build-spec
```

## Branches

| Branch | Purpose |
| --- | --- |
| **`dev`** | Default working branch — experimental / preview |
| **`main`** | Releases only — stable tags and release artifacts |

Day-to-day work and PRs go to **`dev`**. Promote `dev` → `main` only for releases (see [docs/release-process.md](docs/release-process.md)).

## Quick start (dev)

On Windows, install [Build Tools for Visual Studio](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with **Desktop development with C++** first (`link.exe` is required even for `print-cmd`). Details: [docs/development.md](docs/development.md).

```bash
git checkout dev
git pull

# 1. Put llama-server in bin/ (pin b9866 — .\scripts\fetch-llama-pin.ps1 or see bin/README.md)
# 2. Edit config/default.yaml → model.path
cargo run -p winserve -- print-cmd
cargo run -p winserve -- start
```

API: `http://127.0.0.1:8080/v1`

## Rules

1. One backend: llama.cpp
2. One orchestrator: `ServerManager`
3. Raw llama flags only in `app/src/runtime/llama.rs`
4. YAML config is source of truth
5. No plugins, downloads, chat UI, or Docker

See [docs/architecture.md](docs/architecture.md) and [docs/build-spec.md](docs/build-spec.md).

## PR review loop

```bash
python3 -m scripts.pr_review_loop --config examples/pr_review_loop.json --dry-run --no-push
```

## License

MIT — see [LICENSE](LICENSE). Bundle llama.cpp MIT notices with binaries.
