//! Bounded GGUF header check before spawn.
//!
//! Strata walks tensor metadata in `strata_control/gguf_preflight.py`. llama-server
//! still does the real load. This only rejects a file that is not a GGUF header
//! so a bad pick fails before the process starts.

use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, PartialEq, Eq)]
pub enum GgufError {
    Io(String),
    NotGguf,
    Truncated,
    BadVersion(u32),
}

impl std::fmt::Display for GgufError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(msg) => write!(f, "{msg}"),
            Self::NotGguf => write!(f, "file does not begin with a GGUF header"),
            Self::Truncated => write!(f, "GGUF header is truncated"),
            Self::BadVersion(v) => write!(f, "unsupported GGUF version {v}"),
        }
    }
}

/// Magic plus version. Versions 1–3 are what llama.cpp has shipped.
pub fn require_gguf(path: &Path) -> Result<u32, GgufError> {
    let mut file = File::open(path).map_err(|e| GgufError::Io(e.to_string()))?;
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)
        .map_err(|_| GgufError::Truncated)?;
    if &magic != b"GGUF" {
        return Err(GgufError::NotGguf);
    }
    let mut ver = [0u8; 4];
    file.read_exact(&mut ver).map_err(|_| GgufError::Truncated)?;
    let version = u32::from_le_bytes(ver);
    if !(1..=3).contains(&version) {
        return Err(GgufError::BadVersion(version));
    }
    Ok(version)
}

/// Eight-byte header tests and fakes can drop in as `model.gguf`.
pub fn stub_header() -> [u8; 8] {
    let mut buf = [0u8; 8];
    buf[..4].copy_from_slice(b"GGUF");
    buf[4..].copy_from_slice(&3u32.to_le_bytes());
    buf
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_file(bytes: &[u8]) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "winserve-gguf-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut f = File::create(&path).unwrap();
        f.write_all(bytes).unwrap();
        path
    }

    #[test]
    fn accepts_v3_magic() {
        let path = temp_file(&stub_header());
        assert_eq!(require_gguf(&path).unwrap(), 3);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn rejects_plain_text() {
        let path = temp_file(b"not a model");
        assert_eq!(require_gguf(&path).unwrap_err(), GgufError::NotGguf);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn rejects_version_zero() {
        let mut bytes = stub_header();
        bytes[4..].copy_from_slice(&0u32.to_le_bytes());
        let path = temp_file(&bytes);
        assert_eq!(require_gguf(&path).unwrap_err(), GgufError::BadVersion(0));
        let _ = std::fs::remove_file(path);
    }
}
