# Tests

```text
Unit Tests
    ↓
Backend Tests
    ↓
Integration Tests
    ↓
Hardware Tests
    ↓
Installer Tests
```

| Suite | Path | Purpose |
| --- | --- | --- |
| Unit | `tests/unit/` | Pure logic, config, types |
| Backend | `tests/backend/` | Backend trait implementations |
| Integration | `tests/integration/` | Server Manager + process lifecycle |
| Hardware | `tests/hardware/` | GPU/CPU/RAM detection (matrix) |
| Installer | `tests/installer/` | Package/install/uninstall smoke |

Crate-level unit tests also live next to code (`#[cfg(test)]` modules). Prefer those for package-local logic; use this tree for cross-cutting suites.
