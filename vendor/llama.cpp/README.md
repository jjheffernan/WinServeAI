# Vendored llama.cpp

Stable releases of WinServeAI bundle a **vetted** build of [llama.cpp](https://github.com/ggerganov/llama.cpp) (`llama-server`).

## Policy

* Pin a known-good release per stable WinServeAI version
* Upgrade only after validation on the hardware matrix (see `docs/release-process.md`)
* Prefer stability over bleeding-edge commits

## Layout (Phase 1+)

```text
vendor/llama.cpp/
├── README.md          # this file
├── VERSION            # pinned upstream tag/commit
└── bin/               # platform binaries (not committed until release packaging)
    └── llama-server.exe
```

Binaries are produced by release packaging, not day-to-day development commits.
