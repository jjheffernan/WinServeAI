# Release Process

## Channels

```text
main
  │
  ├── nightly
  ├── preview
  └── stable
```

## Backend Pinning

Bundle a vetted `llama.cpp` release with each **stable** version. Only upgrade the bundled backend after validation against a representative set of models and hardware.

## Hardware Matrix (target)

* RTX 3060
* RTX 3080
* RTX 4090
* RTX 5090 (future)
* CPU-only (graceful failure or limited support)

## Checklist (stable)

1. All workspace tests pass
2. Integration tests against pinned llama-server
3. Hardware smoke tests on matrix (as available)
4. Installer builds and clean-uninstalls
5. Config migration from previous stable (when applicable)
6. Changelog updated
7. Tag `vX.Y.Z` and publish artifacts
