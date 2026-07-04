# Desktop App

Thin Windows-native UI over the **Server Manager** (`winserve-launcher`).

## Responsibilities

The desktop only:

- edits config
- displays logs
- displays state
- start
- stop

Nothing else. No model downloads, no chat UI, no backend-specific controls.

## Architecture

```text
Desktop UI
    │
    ▼
Server Manager (winserve-launcher)
    │
    ▼
Backend Interface (winserve-backend)
    │
    ▼
llama.cpp (winserve-llama)
```

## Stack (planned)

- **Tauri 2** + Rust backend (reuses workspace crates)
- Native Windows feel, dark mode, tray support (Phase 5), accessibility

## Phase

Scaffold only. Implementation begins in **Phase 2**.
