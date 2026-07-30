//! Resident manager loop — owns `ServerManager` for the process lifetime.
//!
//! ```text
//! serve → run(manager, commands) → Status | Start | Stop | Restart
//! ```
//!
//! The loop outlives the child process: a stopped, failed, or crashed
//! `llama-server` leaves the owner resident so a later command can start it
//! again. Callers (CLI attach, tray) send [`Command`] values; nothing else
//! touches the manager.

use crate::server::manager::{ServerManager, Status};
use tokio::sync::{mpsc, oneshot};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandKind {
    Status,
    Start,
    Stop,
    Restart,
    Health,
    Endpoint,
}

#[derive(Debug, Clone)]
pub struct Reply {
    pub status: Status,
    pub endpoint: String,
    pub error: Option<String>,
}

#[derive(Debug)]
pub struct Command {
    pub kind: CommandKind,
    pub reply: oneshot::Sender<Reply>,
}

impl Command {
    pub fn new(kind: CommandKind) -> (Self, oneshot::Receiver<Reply>) {
        let (reply, rx) = oneshot::channel();
        (Self { kind, reply }, rx)
    }
}

/// Apply one command and build a reply.
pub async fn handle(manager: &mut ServerManager, kind: CommandKind) -> Reply {
    let error = match kind {
        CommandKind::Status | CommandKind::Health | CommandKind::Endpoint => None,
        CommandKind::Start => manager.start().await.err().map(|e| e.to_string()),
        CommandKind::Stop => manager.stop().await.err().map(|e| e.to_string()),
        CommandKind::Restart => manager.restart().await.err().map(|e| e.to_string()),
    };
    Reply {
        status: manager.get_status(),
        endpoint: manager.openai_base(),
        error,
    }
}

/// Serve commands until every sender is dropped.
pub async fn run(manager: &mut ServerManager, commands: &mut mpsc::Receiver<Command>) {
    while let Some(Command { kind, reply }) = commands.recv().await {
        let _ = reply.send(handle(manager, kind).await);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::config::Config;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_mgr() -> (PathBuf, ServerManager) {
        let root = std::env::temp_dir().join(format!(
            "winserve-res-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("config")).unwrap();
        std::fs::create_dir_all(root.join("logs")).unwrap();
        let path = root.join("config").join("default.yaml");
        let mut cfg = Config::default();
        cfg.logging.dir = root.join("logs");
        cfg.model.path = root.join("missing.gguf");
        cfg.save(&path).unwrap();
        let mgr = ServerManager::load_config(&root, &path).unwrap();
        (root, mgr)
    }

    #[tokio::test]
    async fn status_and_endpoint_without_start() {
        let (root, mut mgr) = temp_mgr();
        let reply = handle(&mut mgr, CommandKind::Status).await;
        assert_eq!(reply.status, Status::Stopped);
        assert!(reply.error.is_none());
        assert!(reply.endpoint.ends_with("/v1"));
        let reply = handle(&mut mgr, CommandKind::Endpoint).await;
        assert_eq!(reply.status, Status::Stopped);
        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn start_command_surfaces_missing_binary() {
        let (root, mut mgr) = temp_mgr();
        let reply = handle(&mut mgr, CommandKind::Start).await;
        assert_eq!(reply.status, Status::Failed);
        assert!(
            reply
                .error
                .as_deref()
                .unwrap_or("")
                .contains("llama-server not found"),
            "{:?}",
            reply.error
        );
        let _ = std::fs::remove_dir_all(root);
    }
}
