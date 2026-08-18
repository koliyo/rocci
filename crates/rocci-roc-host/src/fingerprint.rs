use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs, path::Path, time::UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputFingerprint {
    pub path: String,
    pub len: u64,
    pub mtime_ns: u128,
    pub sha256: String,
}

impl InputFingerprint {
    pub fn from_file(file_path: &Path, rel_path: &str) -> Result<Self> {
        let metadata = fs::metadata(file_path)
            .with_context(|| format!("failed to read metadata for {}", file_path.display()))?;
        let len = metadata.len();
        let mtime_ns = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let bytes = fs::read(file_path)
            .with_context(|| format!("failed to read bytes for {}", file_path.display()))?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let sha256 = format!("{:x}", hasher.finalize());
        Ok(Self {
            path: rel_path.to_string(),
            len,
            mtime_ns,
            sha256,
        })
    }

    pub fn from_bytes(rel_path: &str, bytes: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let sha256 = format!("{:x}", hasher.finalize());
        Self {
            path: rel_path.to_string(),
            len: bytes.len() as u64,
            mtime_ns: 0,
            sha256,
        }
    }

    pub fn matches_file(&self, base_dir: &Path) -> bool {
        let full_path = base_dir.join(&self.path);
        let Ok(meta) = fs::metadata(&full_path) else {
            return false;
        };
        if meta.len() != self.len {
            return false;
        }
        let mtime_ns = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        if mtime_ns != 0 && self.mtime_ns != 0 && mtime_ns == self.mtime_ns {
            return true;
        }
        let Ok(bytes) = fs::read(&full_path) else {
            return false;
        };
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let sha256 = format!("{:x}", hasher.finalize());
        sha256 == self.sha256
    }
}
