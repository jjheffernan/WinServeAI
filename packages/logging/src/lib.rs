//! Unified logging for application and backend output.

use tracing_subscriber::{fmt, EnvFilter};

/// Initialize the global subscriber.
///
/// Level comes from config (`logging.level`) or `WINSERVE_LOG` env override.
pub fn init(level: &str) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level));

    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(false)
        .compact()
        .init();
}

/// Forward a line of backend stdout/stderr into the unified log stream.
pub fn backend_line(stream: &str, line: &str) {
    tracing::info!(target: "backend", stream = %stream, "{line}");
}
