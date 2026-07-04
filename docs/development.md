# Development

## Prerequisites

* Rust 1.78+ (`rustup`)
* Windows 10/11 (primary target)
* Optional: Node 20+ for agent skill tooling scripts

## Workspace

This is a Cargo workspace monorepo.

```bash
cargo check --workspace
cargo test --workspace
./scripts/check.sh
```

## Layout

```text
apps/          # desktop, installer, updater
packages/      # libraries (backend, launcher, config, …)
docs/          # product and contributor docs
scripts/       # developer scripts
tests/         # cross-cutting test suites
examples/      # sample configs
vendor/        # bundled third-party (llama.cpp)
.agents/skills # agent-agnostic skills (all agents)
```

## Adding a Package

1. Create `packages/<name>/` with `Cargo.toml` and `src/lib.rs`
2. Add to workspace `members` in root `Cargo.toml`
3. Add a `winserve-<name>` path dependency under `[workspace.dependencies]`
4. Document responsibility in `docs/architecture.md`

## Coding Standards

* Prefer small, focused crates over a monolith
* UI must not import `winserve-llama` or know about llama.cpp flags
* All orchestration goes through `winserve-launcher` (Server Manager)
* Configuration stays human-readable YAML — no backend-specific flags
* Stable over clever; optimize for contributors and future backends

## Prior art

Scaffolding recommendations from similar products and Windows process research live in [prior-art.md](prior-art.md).

## Agent Skills

Skills live in `.agents/skills/` (Agent Skills standard — agent-agnostic).

```bash
npm run skills:list
npm run skills:update
npm run skills:restore
```

See `AGENTS.md`.
