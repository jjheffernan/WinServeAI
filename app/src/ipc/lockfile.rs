//! Resident-manager lockfile: PID + future named-pipe name.
//!
//! Windows path: `%LOCALAPPDATA%\WinServeAI\manager.lock`
//! Non-Windows (dev/tests): `$XDG_RUNTIME_DIR` / `$TMPDIR` / `/tmp` + `WinServeAI/manager.lock`.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Default pipe name recorded for G3 attach (listener lands later).
pub const DEFAULT_PIPE_NAME: &str = "winserve-manager";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockInfo {
    pub pid: u32,
    pub pipe: String,
}

#[derive(Debug, Error)]
pub enum LockError {
    #[error("manager already running (pid={pid}, pipe={pipe})")]
    AlreadyRunning { pid: u32, pipe: String },
    #[error("io: {0}")]
    Io(#[from] io::Error),
    #[error("invalid lockfile: {0}")]
    Invalid(String),
}

/// Platform lock path (create parent dir on write).
pub fn default_path() -> PathBuf {
    #[cfg(windows)]
    {
        let base = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        return base.join("WinServeAI").join("manager.lock");
    }
    #[cfg(not(windows))]
    {
        let base = std::env::var_os("XDG_RUNTIME_DIR")
            .or_else(|| std::env::var_os("TMPDIR"))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/tmp"));
        base.join("WinServeAI").join("manager.lock")
    }
}

/// Read lock if present. Does **not** remove stale entries.
pub fn read(path: &Path) -> Result<Option<LockInfo>, LockError> {
    if !path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(path)?;
    let info: LockInfo = serde_json::from_str(text.trim()).map_err(|e| LockError::Invalid(e.to_string()))?;
    Ok(Some(info))
}

/// True when the recorded PID still looks alive.
pub fn pid_alive(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }
    use sysinfo::{Pid, ProcessesToUpdate, System};
    let mut sys = System::new();
    let target = Pid::from_u32(pid);
    sys.refresh_processes(ProcessesToUpdate::Some(&[target]), true);
    sys.process(target).is_some()
}

/// Remove the lock if missing, invalid, or the owner PID is dead.
pub fn remove_if_stale(path: &Path) -> Result<bool, LockError> {
    match read(path)? {
        None => Ok(false),
        Some(info) if pid_alive(info.pid) => Ok(false),
        Some(_) => {
            let _ = fs::remove_file(path);
            Ok(true)
        }
    }
}

/// Acquire exclusive lock for this process. Removes stale locks first.
pub fn acquire(path: &Path, pipe: impl Into<String>) -> Result<LockInfo, LockError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let _ = remove_if_stale(path)?;
    if let Some(existing) = read(path)? {
        if pid_alive(existing.pid) {
            return Err(LockError::AlreadyRunning {
                pid: existing.pid,
                pipe: existing.pipe,
            });
        }
        let _ = fs::remove_file(path);
    }
    let info = LockInfo {
        pid: std::process::id(),
        pipe: pipe.into(),
    };
    // Create new exclusively when possible so two serves racing both fail closed.
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
    {
        Ok(mut f) => {
            use std::io::Write;
            let body = serde_json::to_string_pretty(&info)
                .map_err(|e| LockError::Invalid(e.to_string()))?;
            f.write_all(body.as_bytes())?;
            f.write_all(b"\n")?;
            Ok(info)
        }
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
            if let Some(existing) = read(path)? {
                Err(LockError::AlreadyRunning {
                    pid: existing.pid,
                    pipe: existing.pipe,
                })
            } else {
                Err(LockError::Io(e))
            }
        }
        Err(e) => Err(LockError::Io(e)),
    }
}

/// Drop the lock only if it still names this PID.
pub fn release(path: &Path) -> Result<(), LockError> {
    match read(path)? {
        Some(info) if info.pid == std::process::id() => {
            fs::remove_file(path)?;
            Ok(())
        }
        _ => Ok(()),
    }
}

/// RAII guard that releases on drop.
pub struct LockGuard {
    path: PathBuf,
}

impl LockGuard {
    pub fn acquire_default(pipe: impl Into<String>) -> Result<(Self, LockInfo), LockError> {
        let path = default_path();
        let info = acquire(&path, pipe)?;
        Ok((Self { path }, info))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = release(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_lock() -> PathBuf {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("winserve-lock-test-{n}.lock"))
    }

    #[test]
    fn acquire_release_roundtrip() {
        let path = temp_lock();
        let _ = fs::remove_file(&path);
        let info = acquire(&path, "test-pipe").expect("acquire");
        assert_eq!(info.pid, std::process::id());
        assert_eq!(info.pipe, "test-pipe");
        let read_back = read(&path).unwrap().expect("present");
        assert_eq!(read_back, info);
        release(&path).unwrap();
        assert!(read(&path).unwrap().is_none());
    }

    #[test]
    fn second_acquire_fails_while_held() {
        let path = temp_lock();
        let _ = fs::remove_file(&path);
        let _first = acquire(&path, "a").unwrap();
        let err = acquire(&path, "b").unwrap_err();
        assert!(matches!(err, LockError::AlreadyRunning { .. }));
        release(&path).unwrap();
    }

    #[test]
    fn stale_pid_is_reclaimed() {
        let path = temp_lock();
        let _ = fs::remove_file(&path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let stale = LockInfo {
            pid: u32::MAX, // almost certainly not a live process
            pipe: "stale".into(),
        };
        fs::write(&path, serde_json::to_string(&stale).unwrap()).unwrap();
        assert!(remove_if_stale(&path).unwrap());
        let info = acquire(&path, "fresh").unwrap();
        assert_eq!(info.pipe, "fresh");
        release(&path).unwrap();
    }
}
