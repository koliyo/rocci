use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path, time::SystemTime};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub version: String,
    pub created_at: String,
    pub last_used_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

impl Manifest {
    pub fn new(artifact_sha256: Option<String>, target: Option<String>) -> Self {
        let now = rfc3339_now();
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: now.clone(),
            last_used_at: now,
            artifact_sha256,
            target,
        }
    }

    pub fn touch(&mut self) {
        self.last_used_at = rfc3339_now();
    }
}

pub fn rfc3339_now() -> String {
    let now = SystemTime::now();
    let duration = now
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let days = secs / 86400;
    let (year, month, day) = days_to_ymd(days);
    format!("{year:04}-{month:02}-{day:02}T{h:02}:{m:02}:{s:02}Z")
}

fn days_to_ymd(days: u64) -> (u32, u32, u32) {
    let mut d = days as i64;
    let mut year = 1970;
    loop {
        let leap = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
            366
        } else {
            365
        };
        if d < leap {
            break;
        }
        d -= leap;
        year += 1;
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1;
    for &md in &month_days {
        if d < md as i64 {
            break;
        }
        d -= md as i64;
        month += 1;
    }
    (year as u32, month, (d + 1) as u32)
}

pub fn write_atomic_manifest(dir: &Path, manifest: &Manifest) -> Result<()> {
    let json = serde_json::to_string_pretty(manifest)?;
    let tmp_path = dir.join("manifest.json.tmp");
    let target_path = dir.join("manifest.json");
    fs::write(&tmp_path, json)
        .with_context(|| format!("failed to write {}", tmp_path.display()))?;
    fs::rename(&tmp_path, &target_path)
        .with_context(|| format!("failed to rename to {}", target_path.display()))?;
    Ok(())
}
