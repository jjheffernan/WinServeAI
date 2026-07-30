//! Spawn, capture stdout/stderr, graceful stop, force kill.
//!
//! Windows: Job Object (`KILL_ON_JOB_CLOSE`) + `CTRL_BREAK_EVENT` to process group.

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
    #[cfg(windows)]
    job: Option<windows::Win32::Foundation::HANDLE>,
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

        // Windows: new process group (CTRL_BREAK) + CREATE_SUSPENDED so we can
        // AssignProcessToJobObject before the child runs (closes the assign race).
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
            const CREATE_SUSPENDED: u32 = 0x00000004;
            cmd.creation_flags(CREATE_NEW_PROCESS_GROUP | CREATE_SUSPENDED);
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| ProcessError::Spawn(format!("{}: {e}", program.display())))?;

        let pid = child.id().unwrap_or(0);

        #[cfg(windows)]
        let job = match win::assign_to_kill_on_close_job(pid) {
            Ok(job) => {
                if let Err(e) = win::resume_primary_thread(pid) {
                    win::close_handle(job);
                    let _ = child.start_kill();
                    return Err(ProcessError::Spawn(format!(
                        "resume after job assign failed: {e}"
                    )));
                }
                Some(job)
            }
            Err(e) => {
                let _ = child.start_kill();
                return Err(ProcessError::Spawn(format!("job assign failed: {e}")));
            }
        };

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

        Ok(Self {
            child,
            pid,
            #[cfg(windows)]
            job,
        })
    }

    pub fn is_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    /// Graceful stop: CTRL_BREAK on Windows process group, then force kill after grace.
    pub async fn stop(&mut self, grace: Duration) -> Result<Option<i32>, ProcessError> {
        #[cfg(windows)]
        {
            let _ = win::send_ctrl_break(self.pid);
        }
        #[cfg(not(windows))]
        {
            let _ = self.child.start_kill();
        }

        let wait = tokio::time::timeout(grace, self.child.wait()).await;
        match wait {
            Ok(Ok(status)) => {
                #[cfg(windows)]
                self.close_job();
                Ok(status.code())
            }
            Ok(Err(e)) => Err(ProcessError::Io(e)),
            Err(_) => {
                let _ = self.child.start_kill();
                let status = self.child.wait().await?;
                #[cfg(windows)]
                self.close_job();
                Ok(status.code())
            }
        }
    }

    pub async fn try_exit_code(&mut self) -> Option<i32> {
        self.child.try_wait().ok().flatten().and_then(|s| s.code())
    }

    #[cfg(windows)]
    fn close_job(&mut self) {
        if let Some(job) = self.job.take() {
            win::close_handle(job);
        }
    }
}

#[cfg(windows)]
impl Drop for ChildProcess {
    fn drop(&mut self) {
        // Closing the job with KILL_ON_JOB_CLOSE terminates remaining children.
        self.close_job();
    }
}

#[cfg(windows)]
mod win {
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
    use windows::Win32::System::Console::{
        AttachConsole, FreeConsole, GenerateConsoleCtrlEvent, SetConsoleCtrlHandler,
        CTRL_BREAK_EVENT,
    };
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD, THREADENTRY32,
    };
    use windows::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
        SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };
    use windows::Win32::System::Threading::{
        OpenProcess, OpenThread, ResumeThread, PROCESS_ALL_ACCESS, THREAD_SUSPEND_RESUME,
    };

    pub fn assign_to_kill_on_close_job(pid: u32) -> windows::core::Result<HANDLE> {
        unsafe {
            let job = CreateJobObjectW(None, PCWSTR::null())?;
            let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const _,
                std::mem::size_of_val(&info) as u32,
            )?;

            let process = OpenProcess(PROCESS_ALL_ACCESS, false, pid)?;
            let assign = AssignProcessToJobObject(job, process);
            let _ = CloseHandle(process);
            assign?;
            Ok(job)
        }
    }

    /// Resume the first thread belonging to `pid` (primary thread after CREATE_SUSPENDED).
    pub fn resume_primary_thread(pid: u32) -> windows::core::Result<()> {
        unsafe {
            let snap = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0)?;
            let mut entry = THREADENTRY32 {
                dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
                ..Default::default()
            };
            let mut found = false;
            if Thread32First(snap, &mut entry).is_ok() {
                loop {
                    if entry.th32OwnerProcessID == pid {
                        let thread = OpenThread(THREAD_SUSPEND_RESUME, false, entry.th32ThreadID)?;
                        ResumeThread(thread);
                        let _ = CloseHandle(thread);
                        found = true;
                        break;
                    }
                    if Thread32Next(snap, &mut entry).is_err() {
                        break;
                    }
                }
            }
            let _ = CloseHandle(snap);
            if found {
                Ok(())
            } else {
                Err(windows::core::Error::from_win32())
            }
        }
    }

    pub fn close_handle(handle: HANDLE) {
        unsafe {
            if handle != INVALID_HANDLE_VALUE && !handle.is_invalid() {
                let _ = CloseHandle(handle);
            }
        }
    }

    /// Send CTRL_BREAK to the process group (group id == root pid when CREATE_NEW_PROCESS_GROUP).
    pub fn send_ctrl_break(pid: u32) -> std::io::Result<()> {
        unsafe {
            let _ = FreeConsole();
            AttachConsole(pid).map_err(|e| std::io::Error::other(e.message()))?;
            // Ignore break in this process while we generate the event.
            let _ = SetConsoleCtrlHandler(None, true);
            let result = GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, pid);
            let _ = SetConsoleCtrlHandler(None, false);
            let _ = FreeConsole();
            result.map_err(|e| std::io::Error::other(e.message()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn spawn_and_stop_sleep_command() {
        let (tx, _rx) = mpsc::unbounded_channel();
        #[cfg(windows)]
        let program = PathBuf::from("cmd");
        #[cfg(windows)]
        let args = vec!["/C".into(), "ping".into(), "-n".into(), "5".into(), "127.0.0.1".into()];
        #[cfg(not(windows))]
        let program = PathBuf::from("sleep");
        #[cfg(not(windows))]
        let args = vec!["5".into()];

        let mut child = ChildProcess::spawn(program, args, None, tx)
            .await
            .expect("spawn");
        assert!(child.is_running());
        let code = child.stop(Duration::from_secs(2)).await.expect("stop");
        // Killed processes may have various codes; just ensure stop returns.
        let _ = code;
        assert!(!child.is_running());
    }
}
