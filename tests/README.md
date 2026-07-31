# Tests

Crate tests live in `app/` (`cargo test -p winserve`).

Cross-cutting suites:

| Path | Role |
| --- | --- |
| `tests/scripts/stage-release-assert.sh` | Host-safe staged `{app}` layout asserts |
| `tests/windows/a2-smoke.ps1` | A2 wrapper → `scripts/smoke-openai.ps1` (**operator + GGUF**) |
| `tests/windows/orphan-quit.ps1` | Manager-kill leaves no `llama-server` |
| `tests/windows/install-ready.ps1` | Install → Ready operator checklist |
| `tests/windows/tray-quit.ps1` | Tray singleton + CLI attach; Quit orphan proof manual |

`app/tests/windows_ownership.rs` covers lockfile refusal, manager-drop reaping,
and resident attach (named pipe on Windows CI).
