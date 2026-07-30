//! Local IPC transport for the resident manager.
//!
//! Windows: named pipe `\\.\pipe\<name>`
//! Non-Windows (dev/tests): Unix socket beside the lock dir.

use crate::server::resident::{Command, CommandKind, Reply};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;

#[derive(Debug, Error)]
pub enum PipeError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("protocol: {0}")]
    Protocol(String),
    #[error("manager channel closed")]
    ChannelClosed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Request {
    cmd: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Response {
    pub ok: bool,
    pub status: String,
    pub endpoint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Response {
    fn from_reply(reply: Reply) -> Self {
        let error = reply.error;
        Self {
            ok: error.is_none(),
            status: reply.status.as_str().to_string(),
            endpoint: reply.endpoint,
            error,
        }
    }
}

fn parse_kind(cmd: &str) -> Result<CommandKind, PipeError> {
    match cmd.trim().to_ascii_lowercase().as_str() {
        "status" => Ok(CommandKind::Status),
        "start" => Ok(CommandKind::Start),
        "stop" => Ok(CommandKind::Stop),
        "restart" => Ok(CommandKind::Restart),
        "health" => Ok(CommandKind::Health),
        "endpoint" => Ok(CommandKind::Endpoint),
        other => Err(PipeError::Protocol(format!("unknown cmd: {other}"))),
    }
}

/// Endpoint path/name recorded in the lockfile `pipe` field.
pub fn endpoint_for(pipe_name: &str) -> String {
    #[cfg(windows)]
    {
        format!(r"\\.\pipe\{pipe_name}")
    }
    #[cfg(not(windows))]
    {
        // Dev/test transport: loopback TCP; port published beside the lock dir.
        match read_tcp_port(pipe_name) {
            Some(port) => format!("127.0.0.1:{port}"),
            None => format!("127.0.0.1:? ({pipe_name})"),
        }
    }
}

#[cfg(not(windows))]
fn port_file(pipe_name: &str) -> std::path::PathBuf {
    let base = crate::ipc::lockfile::default_path()
        .parent()
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    base.join(format!("{pipe_name}.port"))
}

#[cfg(not(windows))]
fn read_tcp_port(pipe_name: &str) -> Option<u16> {
    std::fs::read_to_string(port_file(pipe_name))
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

/// Accept clients until the command channel closes or the task is aborted.
pub async fn listen(pipe_name: String, commands: mpsc::Sender<Command>) -> Result<(), PipeError> {
    #[cfg(windows)]
    {
        listen_windows(pipe_name, commands).await
    }
    #[cfg(not(windows))]
    {
        listen_tcp(pipe_name, commands).await
    }
}

async fn handle_connection<R, W>(
    reader: R,
    mut writer: W,
    commands: &mpsc::Sender<Command>,
) -> Result<(), PipeError>
where
    R: tokio::io::AsyncRead + Unpin,
    W: tokio::io::AsyncWrite + Unpin,
{
    let mut lines = BufReader::new(reader).lines();
    while let Some(line) = lines.next_line().await? {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let response = match serde_json::from_str::<Request>(line) {
            Ok(req) => match parse_kind(&req.cmd) {
                Ok(kind) => {
                    let (cmd, ack) = Command::new(kind);
                    if commands.send(cmd).await.is_err() {
                        return Err(PipeError::ChannelClosed);
                    }
                    match ack.await {
                        Ok(reply) => Response::from_reply(reply),
                        Err(_) => Response {
                            ok: false,
                            status: "Failed".into(),
                            endpoint: String::new(),
                            error: Some("manager dropped reply".into()),
                        },
                    }
                }
                Err(e) => Response {
                    ok: false,
                    status: "Failed".into(),
                    endpoint: String::new(),
                    error: Some(e.to_string()),
                },
            },
            Err(e) => Response {
                ok: false,
                status: "Failed".into(),
                endpoint: String::new(),
                error: Some(format!("invalid json: {e}")),
            },
        };
        let mut out = serde_json::to_string(&response)
            .map_err(|e| PipeError::Protocol(e.to_string()))?;
        out.push('\n');
        writer.write_all(out.as_bytes()).await?;
        writer.flush().await?;
    }
    Ok(())
}

/// One-shot client request against a running listener.
pub async fn request(pipe_name: &str, cmd: &str) -> Result<Response, PipeError> {
    #[cfg(windows)]
    {
        request_windows(pipe_name, cmd).await
    }
    #[cfg(not(windows))]
    {
        request_tcp(pipe_name, cmd).await
    }
}

#[cfg(windows)]
async fn listen_windows(pipe_name: String, commands: mpsc::Sender<Command>) -> Result<(), PipeError> {
    use tokio::net::windows::named_pipe::ServerOptions;

    let endpoint = endpoint_for(&pipe_name);
    let mut first = true;
    loop {
        let mut opts = ServerOptions::new();
        if first {
            opts = opts.first_pipe_instance(true);
            first = false;
        }
        let server = opts.create(&endpoint)?;
        server.connect().await?;
        let (reader, writer) = tokio::io::split(server);
        if let Err(e) = handle_connection(reader, writer, &commands).await {
            if matches!(e, PipeError::ChannelClosed) {
                return Ok(());
            }
            tracing::warn!("ipc client error: {e}");
        }
    }
}

#[cfg(windows)]
async fn request_windows(pipe_name: &str, cmd: &str) -> Result<Response, PipeError> {
    use tokio::net::windows::named_pipe::ClientOptions;

    let endpoint = endpoint_for(pipe_name);
    let client = ClientOptions::new().open(endpoint)?;
    let (reader, mut writer) = tokio::io::split(client);
    let body = serde_json::to_string(&Request {
        cmd: cmd.to_string(),
    })
    .map_err(|e| PipeError::Protocol(e.to_string()))?;
    writer.write_all(body.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    let mut lines = BufReader::new(reader).lines();
    let line = lines
        .next_line()
        .await?
        .ok_or_else(|| PipeError::Protocol("empty response".into()))?;
    serde_json::from_str(&line).map_err(|e| PipeError::Protocol(e.to_string()))
}

#[cfg(not(windows))]
async fn listen_tcp(pipe_name: String, commands: mpsc::Sender<Command>) -> Result<(), PipeError> {
    use tokio::net::TcpListener;

    let path = port_file(&pipe_name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    std::fs::write(&path, format!("{port}\n"))?;
    loop {
        let (stream, _) = listener.accept().await?;
        let (reader, writer) = stream.into_split();
        if let Err(e) = handle_connection(reader, writer, &commands).await {
            if matches!(e, PipeError::ChannelClosed) {
                let _ = std::fs::remove_file(&path);
                return Ok(());
            }
            tracing::warn!("ipc client error: {e}");
        }
    }
}

#[cfg(not(windows))]
async fn request_tcp(pipe_name: &str, cmd: &str) -> Result<Response, PipeError> {
    use tokio::net::TcpStream;

    let port = read_tcp_port(pipe_name).ok_or_else(|| {
        PipeError::Protocol(format!("no listener port for {pipe_name}"))
    })?;
    let stream = TcpStream::connect(("127.0.0.1", port)).await?;
    let (reader, mut writer) = stream.into_split();
    let body = serde_json::to_string(&Request {
        cmd: cmd.to_string(),
    })
    .map_err(|e| PipeError::Protocol(e.to_string()))?;
    writer.write_all(body.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    let mut lines = BufReader::new(reader).lines();
    let line = lines
        .next_line()
        .await?
        .ok_or_else(|| PipeError::Protocol("empty response".into()))?;
    serde_json::from_str(&line).map_err(|e| PipeError::Protocol(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::manager::Status;
    use crate::server::resident::{Command, CommandKind, Reply};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[tokio::test]
    async fn roundtrip_status_on_local_transport() {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let pipe_name = format!("winserve-test-{n}");
        let (tx, mut rx) = mpsc::channel::<Command>(4);

        // Minimal fake manager loop: answer Status without a real ServerManager.
        let server = tokio::spawn(async move {
            while let Some(Command { kind, reply }) = rx.recv().await {
                let _ = reply.send(Reply {
                    status: Status::Stopped,
                    endpoint: "http://127.0.0.1:8080/v1".into(),
                    error: if matches!(
                        kind,
                        CommandKind::Status | CommandKind::Health | CommandKind::Endpoint
                    ) {
                        None
                    } else {
                        Some("no manager in test".into())
                    },
                });
            }
        });
        let listen = tokio::spawn(listen(pipe_name.clone(), tx));
        let mut resp = None;
        for _ in 0..50 {
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            if let Ok(r) = request(&pipe_name, "status").await {
                resp = Some(r);
                break;
            }
        }
        let resp = resp.expect("status");
        assert!(resp.ok);
        assert_eq!(resp.status, "Stopped");
        assert!(resp.endpoint.contains("/v1"));

        let resp = request(&pipe_name, "health").await.expect("health");
        assert!(resp.ok);

        listen.abort();
        server.abort();
        #[cfg(not(windows))]
        {
            let _ = std::fs::remove_file(port_file(&pipe_name));
        }
    }
}
