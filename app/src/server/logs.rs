//! Unified log streams: server.log, llama.log, error.log
//!
//! Size- and age-based rotation keeps overnight runs from filling the disk.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;

use crate::server::config::LoggingSection;

#[derive(Debug, Error)]
pub enum LogError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Copy)]
pub struct RotatePolicy {
    pub max_bytes: u64,
    pub max_age_secs: u64,
    pub keep: u32,
}

impl From<&LoggingSection> for RotatePolicy {
    fn from(logging: &LoggingSection) -> Self {
        Self {
            max_bytes: logging.max_bytes,
            max_age_secs: logging.max_age_secs,
            keep: logging.keep,
        }
    }
}

struct RotatingFile {
    path: PathBuf,
    file: Option<File>,
    policy: RotatePolicy,
}

impl RotatingFile {
    fn open(path: PathBuf, policy: RotatePolicy) -> Result<Self, LogError> {
        let mut this = Self {
            file: Some(open_append(&path)?),
            path,
            policy,
        };
        this.maybe_rotate()?;
        Ok(this)
    }

    fn write_line(&mut self, line: &str) -> Result<(), LogError> {
        self.maybe_rotate()?;
        let file = self.file.as_mut().expect("log file open");
        writeln!(file, "{} {line}", timestamp())?;
        file.flush()?;
        Ok(())
    }

    fn maybe_rotate(&mut self) -> Result<(), LogError> {
        let meta = self.file.as_ref().expect("log file open").metadata()?;
        let over_size = self.policy.max_bytes > 0 && meta.len() >= self.policy.max_bytes;
        let over_age = if self.policy.max_age_secs > 0 {
            meta.modified()
                .ok()
                .and_then(|m| SystemTime::now().duration_since(m).ok())
                .is_some_and(|age| age >= Duration::from_secs(self.policy.max_age_secs))
        } else {
            false
        };
        if !over_size && !over_age {
            return Ok(());
        }

        // Close before rename (required on Windows).
        self.file.take();
        rotate_files(&self.path, self.policy.keep)?;
        self.file = Some(open_append(&self.path)?);
        Ok(())
    }
}

pub struct LogSinks {
    dir: PathBuf,
    server: Mutex<RotatingFile>,
    llama: Mutex<RotatingFile>,
    error: Mutex<RotatingFile>,
}

impl LogSinks {
    pub fn open(logging: &LoggingSection) -> Result<Self, LogError> {
        let dir = logging.dir.clone();
        std::fs::create_dir_all(&dir)?;
        let policy = RotatePolicy::from(logging);
        Ok(Self {
            server: Mutex::new(RotatingFile::open(dir.join("server.log"), policy)?),
            llama: Mutex::new(RotatingFile::open(dir.join("llama.log"), policy)?),
            error: Mutex::new(RotatingFile::open(dir.join("error.log"), policy)?),
            dir,
        })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn server(&self, line: &str) {
        write_line(&self.server, line);
    }

    pub fn llama(&self, line: &str) {
        write_line(&self.llama, line);
    }

    pub fn error(&self, line: &str) {
        write_line(&self.error, line);
    }
}

/// Last `max_lines` of a log file. Missing file → empty list (viewer-friendly).
pub fn tail_file(path: &Path, max_lines: usize) -> Result<Vec<String>, LogError> {
    if max_lines == 0 || !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(path)?;
    let lines: Vec<String> = text.lines().map(str::to_string).collect();
    if lines.len() <= max_lines {
        Ok(lines)
    } else {
        Ok(lines[lines.len() - max_lines..].to_vec())
    }
}

/// Tail the three unified streams under `dir`.
pub fn tail_dir(dir: &Path, max_lines: usize) -> Result<LogTail, LogError> {
    Ok(LogTail {
        server: tail_file(&dir.join("server.log"), max_lines)?,
        llama: tail_file(&dir.join("llama.log"), max_lines)?,
        error: tail_file(&dir.join("error.log"), max_lines)?,
    })
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LogTail {
    pub server: Vec<String>,
    pub llama: Vec<String>,
    pub error: Vec<String>,
}

fn open_append(path: &Path) -> Result<File, LogError> {
    Ok(OpenOptions::new().create(true).append(true).open(path)?)
}

fn rotate_files(active: &Path, keep: u32) -> Result<(), LogError> {
    if keep == 0 {
        let _ = fs::remove_file(active);
        return Ok(());
    }
    let oldest = rotated_path(active, keep);
    let _ = fs::remove_file(&oldest);
    for n in (1..keep).rev() {
        let from = rotated_path(active, n);
        let to = rotated_path(active, n + 1);
        if from.exists() {
            fs::rename(&from, &to)?;
        }
    }
    if active.exists() {
        fs::rename(active, rotated_path(active, 1))?;
    }
    Ok(())
}

fn rotated_path(active: &Path, n: u32) -> PathBuf {
    let mut name = active.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".{n}"));
    active.with_file_name(name)
}

fn timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // ISO-ish UTC epoch seconds; keep deps minimal (no chrono).
    format!("ts={secs}")
}

fn write_line(file: &Mutex<RotatingFile>, line: &str) {
    if let Ok(mut f) = file.lock() {
        let _ = f.write_line(line);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_logging(max_bytes: u64, max_age_secs: u64, keep: u32) -> (PathBuf, LoggingSection) {
        let dir = std::env::temp_dir().join(format!(
            "winserve-logs-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&dir);
        (
            dir.clone(),
            LoggingSection {
                dir,
                max_bytes,
                max_age_secs,
                keep,
            },
        )
    }

    #[test]
    fn open_write_prefixes_timestamp() {
        let (dir, logging) = temp_logging(1024 * 1024, 0, 3);
        let sinks = LogSinks::open(&logging).expect("open");
        sinks.server("hello");
        let text = fs::read_to_string(dir.join("server.log")).expect("read");
        assert!(text.contains("ts="), "got: {text}");
        assert!(text.contains("hello"), "got: {text}");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn size_rotation_renames_active() {
        let (dir, logging) = temp_logging(64, 0, 2);
        let sinks = LogSinks::open(&logging).expect("open");
        for i in 0..20 {
            sinks.server(&format!("line-{i}-xxxxxxxxxxxxxxxxxxxx"));
        }
        assert!(
            dir.join("server.log.1").exists(),
            "expected rotated sibling under {}",
            dir.display()
        );
        let active = fs::read_to_string(dir.join("server.log")).unwrap();
        assert!(
            !active.is_empty(),
            "active file should be reopened after rotation"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn age_rotation_on_open() {
        let (dir, logging) = temp_logging(1024 * 1024, 1, 2);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("server.log");
        fs::write(&path, "old\n").unwrap();
        let file = OpenOptions::new().write(true).open(&path).unwrap();
        file.set_modified(SystemTime::now() - Duration::from_secs(10))
            .unwrap();
        drop(file);

        let sinks = LogSinks::open(&logging).expect("open");
        sinks.server("fresh");
        assert!(
            dir.join("server.log.1").exists(),
            "age rotate should produce server.log.1"
        );
        let text = fs::read_to_string(dir.join("server.log")).unwrap();
        assert!(text.contains("fresh"), "got: {text}");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn tail_file_returns_last_n_lines() {
        let dir = std::env::temp_dir().join(format!(
            "winserve-tail-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("server.log");
        fs::write(&path, "a\nb\nc\nd\ne\n").unwrap();
        let lines = tail_file(&path, 3).unwrap();
        assert_eq!(lines, vec!["c".to_string(), "d".into(), "e".into()]);
        assert!(tail_file(&dir.join("missing.log"), 10).unwrap().is_empty());
        let _ = fs::remove_dir_all(dir);
    }
}
