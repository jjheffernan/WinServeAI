# Logging

## Requirements

* Rotating logs
* Timestamps
* Backend output (stdout/stderr)
* Application logs

## Design

`packages/logging` owns the unified log stream.

* Application events use `tracing`
* Backend process lines are forwarded with `target = "backend"` (or `process.stdout` / `process.stderr`)
* Level comes from `logging.level` in config (`info` default)
* Env override: `WINSERVE_LOG` / `RUST_LOG`

## Desktop

The desktop displays the unified log stream. It does not parse backend-specific formats.
