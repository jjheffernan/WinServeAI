//! Unified log streams: server.log, llama.log, error.log

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LogError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

pub struct LogSinks {
    dir: PathBuf,
    server: Mutex<File>,
    llama: Mutex<File>,
    error: Mutex<File>,
}

impl LogSinks {
    pub fn open(dir: impl AsRef<Path>) -> Result<Self, LogError> {
        let dir = dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&dir)?;
        Ok(Self {
            dir: dir.clone(),
            server: Mutex::new(open_append(dir.join("server.log"))?),
            llama: Mutex::new(open_append(dir.join("llama.log"))?),
            error: Mutex::new(open_append(dir.join("error.log"))?),
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

fn open_append(path: PathBuf) -> Result<File, LogError> {
    Ok(OpenOptions::new().create(true).append(true).open(path)?)
}

fn write_line(file: &Mutex<File>, line: &str) {
    if let Ok(mut f) = file.lock() {
        let _ = writeln!(f, "{line}");
        let _ = f.flush();
    }
}
