# Contributing

Thanks for helping build WinServeAI. This is infrastructure software — prefer stability, clear boundaries, and small surface area.

## Before You Code

1. Read `docs/vision.md` and `docs/architecture.md`
2. Confirm the change fits MVP scope (or is explicitly Phase 5+)
3. Prefer extending Server Manager / Backend interfaces over special-casing the UI

## Pull Requests

* One concern per PR
* Update docs when behavior or architecture changes
* Add tests at the appropriate layer (unit → backend → integration → hardware → installer)
* Do not introduce chat UI, model download ecosystems, or backend-specific flags in config

## Agent Skills

Project skills live in `.agents/skills/` and work with any Agent Skills–compatible tool. Restore with:

```bash
npm run skills:restore
```

## Code of Conduct

Be respectful. Assume good intent. Optimize for long-term maintainability.
