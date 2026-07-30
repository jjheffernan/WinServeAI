//! Readiness: wait until llama-server answers OpenAI routes.
//!
//! Primary probe: `GET /v1/models`. Optional fallback: `GET /health`
//! (503 while loading / 200 when ready on many llama-server builds).

use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HealthError {
    #[error("not ready within timeout")]
    Timeout,
    #[error("http: {0}")]
    Http(String),
}

/// Poll until ready or timeout.
///
/// Order per tick:
/// 1. `GET {base}/v1/models` — success ⇒ ready (primary)
/// 2. else `GET {base}/health` — success ⇒ ready (fallback)
/// 3. 503 on either ⇒ still loading; keep waiting
pub async fn wait_until_ready(base_url: &str, timeout: Duration) -> Result<(), HealthError> {
    let models_url = format!("{base_url}/v1/models");
    let health_url = format!("{base_url}/health");
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .map_err(|e| HealthError::Http(e.to_string()))?;

    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        if probe_ready(&client, &models_url).await {
            return Ok(());
        }
        if probe_ready(&client, &health_url).await {
            return Ok(());
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(HealthError::Timeout);
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}

/// Returns true on HTTP 2xx. 503 and other outcomes mean "not ready yet".
async fn probe_ready(client: &reqwest::Client, url: &str) -> bool {
    match client.get(url).send().await {
        Ok(resp) if resp.status().is_success() => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn times_out_when_nothing_listens() {
        let err = wait_until_ready("http://127.0.0.1:59999", Duration::from_millis(400))
            .await
            .unwrap_err();
        assert!(matches!(err, HealthError::Timeout));
    }

    #[tokio::test]
    async fn models_primary_503_then_200() {
        let hits = Arc::new(AtomicUsize::new(0));
        let hits2 = hits.clone();
        let base = spawn_mock(move |path| {
            if path.starts_with("/v1/models") {
                let n = hits2.fetch_add(1, Ordering::SeqCst);
                if n == 0 {
                    (503, "loading")
                } else {
                    (200, "{\"data\":[]}")
                }
            } else {
                (404, "no")
            }
        })
        .await;
        wait_until_ready(&base, Duration::from_secs(3))
            .await
            .expect("models should become ready");
        assert!(hits.load(Ordering::SeqCst) >= 2);
    }

    #[tokio::test]
    async fn health_fallback_when_models_absent() {
        let base = spawn_mock(|path| {
            if path.starts_with("/health") {
                (200, "ok")
            } else {
                (404, "missing")
            }
        })
        .await;
        wait_until_ready(&base, Duration::from_secs(2))
            .await
            .expect("health fallback should ready");
    }

    async fn spawn_mock(
        handler: impl Fn(&str) -> (u16, &'static str) + Send + Sync + 'static,
    ) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        let handler = Arc::new(handler);
        tokio::spawn(async move {
            loop {
                let Ok((mut sock, _)) = listener.accept().await else {
                    break;
                };
                let mut buf = vec![0u8; 2048];
                let Ok(n) = sock.read(&mut buf).await else {
                    continue;
                };
                let req = String::from_utf8_lossy(&buf[..n]);
                let path = req
                    .lines()
                    .next()
                    .unwrap_or("")
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or("/");
                let (code, body) = handler(path);
                let reason = if code == 200 {
                    "OK"
                } else if code == 503 {
                    "Service Unavailable"
                } else {
                    "Not Found"
                };
                let resp = format!(
                    "HTTP/1.1 {code} {reason}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = sock.write_all(resp.as_bytes()).await;
            }
        });
        format!("http://{addr}")
    }
}
