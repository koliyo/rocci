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
        let sha256 = hex_sha256(hasher);
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
        let sha256 = hex_sha256(hasher);
        Self {
            path: rel_path.to_string(),
            len: bytes.len() as u64,
            mtime_ns: 0,
            sha256,
        }
    }

    pub fn drift_from(expected: &[Self], stored: &[Self]) -> Option<String> {
        if expected.is_empty() {
            return None;
        }
        if stored.is_empty() {
            return Some("missing fingerprints.json".into());
        }
        let mut expected_sorted = expected.to_vec();
        let mut stored_sorted = stored.to_vec();
        expected_sorted.sort_by(|a, b| a.path.cmp(&b.path));
        stored_sorted.sort_by(|a, b| a.path.cmp(&b.path));
        let mut expected_i = 0;
        let mut stored_i = 0;
        while expected_i < expected_sorted.len() || stored_i < stored_sorted.len() {
            match (expected_sorted.get(expected_i), stored_sorted.get(stored_i)) {
                (Some(want), Some(have)) if want.path == have.path => {
                    if want.sha256 != have.sha256 {
                        return Some(format!("{} changed", want.path));
                    }
                    expected_i += 1;
                    stored_i += 1;
                }
                (Some(want), Some(have)) if want.path < have.path => {
                    return Some(format!("{} added", want.path));
                }
                (Some(_), Some(have)) => {
                    return Some(format!("{} removed", have.path));
                }
                (Some(want), None) => return Some(format!("{} added", want.path)),
                (None, Some(have)) => return Some(format!("{} removed", have.path)),
                (None, None) => break,
            }
        }
        None
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
        let sha256 = hex_sha256(hasher);
        sha256 == self.sha256
    }
}

fn hex_sha256(hasher: Sha256) -> String {
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::InputFingerprint;

    fn fp(path: &str, sha: &str) -> InputFingerprint {
        InputFingerprint {
            path: path.into(),
            len: 0,
            mtime_ns: 0,
            sha256: sha.into(),
        }
    }

    #[test]
    fn drift_reports_changed_and_added_inputs() {
        let stored = [fp("NavList.rocci", "aaa"), fp("Views.roc", "bbb")];
        let same = [fp("Views.roc", "bbb"), fp("NavList.rocci", "aaa")];
        assert_eq!(InputFingerprint::drift_from(&same, &stored), None);
        let changed = [fp("NavList.rocci", "ccc"), fp("Views.roc", "bbb")];
        assert_eq!(
            InputFingerprint::drift_from(&changed, &stored).as_deref(),
            Some("NavList.rocci changed")
        );
        let added = [
            fp("NavList.rocci", "aaa"),
            fp("Views.roc", "bbb"),
            fp("Html.roc", "ddd"),
        ];
        assert_eq!(
            InputFingerprint::drift_from(&added, &stored).as_deref(),
            Some("Html.roc added")
        );
        assert_eq!(
            InputFingerprint::drift_from(&same, &[]).as_deref(),
            Some("missing fingerprints.json")
        );
    }
}
