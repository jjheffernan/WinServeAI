//! Controllable stand-in for `llama-server` used by ServerManager lifecycle tests.
//!
//! Understands `--host` / `--port` (llama-server shape). Behavior from cwd file
//! `FAKE_BEHAVIOR` (written by tests) or env `FAKE_LLAMA_BEHAVIOR`:
//! - `ready` (default) — `GET /v1/models` → 200, stay alive
//! - `never_ready` — bind and answer 503 forever
//! - `exit_after_ready` — answer 200 once, then exit (crash)

use std::env;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::thread;
use std::time::Duration;

fn arg_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1).cloned())
}

fn behavior() -> String {
    if let Ok(s) = std::fs::read_to_string("FAKE_BEHAVIOR") {
        let t = s.trim().to_string();
        if !t.is_empty() {
            return t;
        }
    }
    env::var("FAKE_LLAMA_BEHAVIOR").unwrap_or_else(|_| "ready".into())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let host = arg_value(&args, "--host").unwrap_or_else(|| "127.0.0.1".into());
    let port = arg_value(&args, "--port").unwrap_or_else(|| "8080".into());
    let behavior = behavior();

    let listener = TcpListener::bind(format!("{host}:{port}")).unwrap_or_else(|e| {
        eprintln!("fake-llama-server: bind {host}:{port}: {e}");
        std::process::exit(2);
    });
    // Touch marker so tests can wait for bind if needed.
    let _ = std::fs::write(Path::new("FAKE_BOUND"), format!("{host}:{port}\n"));

    let mut served_ready = false;
    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(s) => s,
            Err(_) => continue,
        };
        let mut buf = [0u8; 1024];
        let _ = stream.read(&mut buf);
        let req = String::from_utf8_lossy(&buf);
        let path = req.lines().next().unwrap_or("");
        let ready_path = path.contains("/v1/models") || path.contains("/health");

        let (code, body) = if !ready_path {
            (404, "not found")
        } else if behavior == "never_ready" {
            (503, "loading")
        } else {
            served_ready = true;
            (200, "{\"data\":[]}")
        };

        let resp = format!(
            "HTTP/1.1 {code} OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let _ = stream.write_all(resp.as_bytes());
        let _ = stream.flush();

        if behavior == "exit_after_ready" && served_ready {
            thread::sleep(Duration::from_millis(50));
            return;
        }
    }
}
