//! Spawn, capture stdout/stderr, graceful stop, force kill.

use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;

#[derive(Debug, Error)]
pub enum ProcessError {
    #[error("already running")]
    AlreadyRunning,
    #[error("not running")]
    NotRunning,
    #[error("spawn failed: {0}")]
    Spawn(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

pub struct ChildProcess {
    child: Child,
    pub pid: u32,
}

impl ChildProcess {
    pub async fn spawn(
        program: PathBuf,
        args: Vec<String>,
        workdir: Option<PathBuf>,
        log_tx: mpsc::UnboundedSender<String>,
    ) -> Result<Self, ProcessError> {
        let mut cmd = Command::new(&program);
        cmd.args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        if let Some(dir) = workdir {
            cmd.current_dir(dir);
        }

        // Windows: new process group so we can send CTRL_BREAK later.
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
            cmd.creation_flags(CREATE_NEW_PROCESS_GROUP);
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| ProcessError::Spawn(format!("{}: {e}", program.display())))?;

        let pid = child.id().unwrap_or(0);

        if let Some(stdout) = child.stdout.take() {
            let tx = log_tx.clone();
            tokio::spawn(async move {
                let mut lines = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let _ = tx.send(format!("[stdout] {line}"));
                }
            });
        }

        if let Some(stderr) = child.stderr.take() {
            let tx = log_tx;
            tokio::spawn(async move {
                let mut lines = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let _ = tx.send(format!("[stderr] {line}"));
                }
            });
        }

        Ok(Self { child, pid })
    }

    pub fn is_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    /// Graceful stop: console control event on Windows, kill elsewhere; then force.
    pub async fn stop(&mut self, grace: Duration) -> Result<Option<i32>, ProcessError> {
        // Best-effort graceful signal.
        #[cfg(windows)]
        {
            // CTRL_BREAK_EVENT to process group (pid). Force kill if still alive.
            let _ = send_ctrl_break(self.pid);
        }
        #[cfg(not(windows))]
        {
            let _ = self.child.start_kill();
        }

        let wait = tokio::time::timeout(grace, self.child.wait()).await;
        match wait {
            Ok(Ok(status)) => Ok(status.code()),
            Ok(Err(e)) => Err(ProcessError::Io(e)),
            Err(_) => {
                let _ = self.child.start_kill();
                let status = self.child.wait().await?;
                Ok(status.code())
            }
        }
    }

    pub async fn try_exit_code(&mut self) -> Option<i32> {
        self.child.try_wait().ok().flatten().and_then(|s| s.code())
    }
}

#[cfg(windows)]
fn send_ctrl_break(pid: u32) -> std::io::Result<()> {
    // GenerateConsoleCtrlEvent requires attachment gymnastics; for MVP we rely on
    // kill_on_drop / start_kill after grace. Hook real CTRL_BREAK in Phase 1 hardening.
    let _ = pid;
    Ok(())
}
