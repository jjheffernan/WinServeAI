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
    // Readiness only talks to the server we just spawned. A system proxy
    // (HTTP_PROXY, Windows proxy, PAC) must not intercept 127.0.0.1.
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .no_proxy()
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

/// One llama-server slot, enough for the overview. Missing fields stay zero.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotSnap {
    pub state: &'static str,
    pub prompt_tokens: u64,
    pub output_tokens: u64,
    pub context_used: u64,
    pub context_size: u64,
}

pub fn parse_slots(body: &str) -> SlotSnap {
    let empty = SlotSnap {
        state: "Idle",
        prompt_tokens: 0,
        output_tokens: 0,
        context_used: 0,
        context_size: 0,
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(body) else {
        return empty;
    };
    let slot = value
        .as_array()
        .and_then(|rows| rows.first())
        .unwrap_or(&value);
    let num = |key: &str| slot.get(key).and_then(|v| v.as_u64()).unwrap_or(0);
    let prompt = num("n_prompt_tokens");
    let decoded = num("n_decoded");
    let n_ctx = num("n_ctx");
    let n_past = num("n_past");
    let processing = slot
        .get("is_processing")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let state = if !processing {
        "Idle"
    } else if decoded == 0 {
        "Reading"
    } else {
        "Generating"
    };
    SlotSnap {
        state,
        prompt_tokens: prompt,
        output_tokens: decoded,
        context_used: if n_past > 0 { n_past } else { prompt.saturating_add(decoded) },
        context_size: n_ctx,
    }
}

pub async fn read_slots(openai_base: &str) -> SlotSnap {
    let origin = openai_base.trim_end_matches('/').trim_end_matches("/v1");
    let url = format!("{origin}/slots");
    let Ok(client) = reqwest::Client::builder()
        .timeout(Duration::from_secs(1))
        .no_proxy()
        .build()
    else {
        return parse_slots("");
    };
    let Ok(resp) = client.get(&url).send().await else {
        return parse_slots("");
    };
    if !resp.status().is_success() {
        return parse_slots("");
    }
    parse_slots(&resp.text().await.unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::parse_slots;

    #[test]
    fn slot_generating_uses_decoded_tokens() {
        let snap = parse_slots(
            r#"[{"is_processing":true,"n_ctx":32000,"n_decoded":100,"n_past":400,"n_prompt_tokens":300}]"#,
        );
        assert_eq!(snap.state, "Generating");
        assert_eq!(snap.output_tokens, 100);
        assert_eq!(snap.context_used, 400);
        assert_eq!(snap.context_size, 32000);
    }

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
