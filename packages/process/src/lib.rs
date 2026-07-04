//! Process lifecycle management for inference backends.

use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use winserve_shared::{Result, WinServeError};

/// Specification for launching a child process.
#[derive(Debug, Clone)]
pub struct ProcessSpec {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub working_dir: Option<PathBuf>,
}

/// Managed child process with stdout/stderr capture hooks.
pub struct ManagedProcess {
    child: Option<Child>,
    spec: ProcessSpec,
}

impl ManagedProcess {
    pub fn new(spec: ProcessSpec) -> Self {
        Self {
            child: None,
            spec,
        }
    }

    pub async fn spawn(&mut self) -> Result<()> {
        if self.child.is_some() {
            return Err(WinServeError::Process("process already running".into()));
        }

        let mut cmd = Command::new(&self.spec.program);
        cmd.args(&self.spec.args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        if let Some(dir) = &self.spec.working_dir {
            cmd.current_dir(dir);
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| WinServeError::Process(format!("spawn failed: {e}")))?;

        if let Some(stdout) = child.stdout.take() {
            tokio::spawn(async move {
                let mut lines = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    tracing::info!(target: "process.stdout", "{line}");
                }
            });
        }

        if let Some(stderr) = child.stderr.take() {
            tokio::spawn(async move {
                let mut lines = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    tracing::warn!(target: "process.stderr", "{line}");
                }
            });
        }

        self.child = Some(child);
        Ok(())
    }

    pub async fn graceful_stop(&mut self) -> Result<()> {
        let Some(mut child) = self.child.take() else {
            return Ok(());
        };

        // Platform-specific graceful shutdown lands in Phase 1 (CTRL_BREAK / SIGTERM).
        child
            .kill()
            .await
            .map_err(|e| WinServeError::Process(format!("kill failed: {e}")))?;
        Ok(())
    }

    pub fn is_running(&mut self) -> bool {
        match self.child.as_mut() {
            Some(child) => matches!(child.try_wait(), Ok(None)),
            None => false,
        }
    }
}
