use crate::error::{DatastarError, Result};

pub const DEFAULT_VERSION: &str = "1.0.2";

pub fn parse_version(raw: &str) -> Result<String> {
    let trimmed = raw.trim().strip_prefix('v').unwrap_or(raw.trim());
    let mut parts = trimmed.split('.');
    let (Some(major), Some(minor), Some(patch)) = (parts.next(), parts.next(), parts.next()) else {
        return Err(DatastarError::InvalidVersion(format!(
            "expected 'X.Y.Z', got '{raw}'"
        )));
    };
    if parts.next().is_some() {
        return Err(DatastarError::InvalidVersion(format!(
            "expected 'X.Y.Z', got '{raw}'"
        )));
    }
    if major.is_empty() || minor.is_empty() || patch.is_empty() {
        return Err(DatastarError::InvalidVersion(format!(
            "expected 'X.Y.Z', got '{raw}'"
        )));
    }
    if !major.chars().all(|c| c.is_ascii_digit())
        || !minor.chars().all(|c| c.is_ascii_digit())
        || !patch.chars().all(|c| c.is_ascii_digit())
    {
        return Err(DatastarError::InvalidVersion(format!(
            "expected numeric 'X.Y.Z', got '{raw}'"
        )));
    }
    Ok(format!("{major}.{minor}.{patch}"))
}

pub fn tag_name(version: &str) -> String {
    format!(
        "v{}",
        parse_version(version).unwrap_or_else(|_| version.trim().to_string())
    )
}
