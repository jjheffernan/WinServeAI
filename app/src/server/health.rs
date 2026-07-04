//! Readiness: wait until llama-server answers OpenAI routes.

use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HealthError {
    #[error("not ready within timeout")]
    Timeout,
    #[error("http: {0}")]
    Http(String),
}

/// Poll `GET {base}/v1/models` until 200 or timeout.
pub async fn wait_until_ready(base_url: &str, timeout: Duration) -> Result<(), HealthError> {
    let url = format!("{base_url}/v1/models");
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .map_err(|e| HealthError::Http(e.to_string()))?;

    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => return Ok(()),
            Ok(resp) if resp.status().as_u16() == 503 => {
                // Still loading — keep waiting.
            }
            Ok(_) | Err(_) => {}
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(HealthError::Timeout);
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}
