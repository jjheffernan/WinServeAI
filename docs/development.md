# Development

## Prerequisites

* Rust stable
* Windows 10/11 (primary target)
* `bin/llama-server.exe` from a pinned llama.cpp release

## Build / run

```bash
cargo check -p winserve
cargo run -p winserve -- print-cmd
cargo run -p winserve -- start
```

Override config: `WINSERVE_CONFIG=/path/to.yaml`

## Scripts

```powershell
.\scripts\start.ps1
.\scripts\stop.ps1
.\scripts\reset.ps1
```

## Architecture

See [architecture.md](architecture.md). Do not reintroduce package-per-concern monorepo layers without a concrete second backend.
