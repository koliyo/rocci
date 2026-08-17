use crate::error::{DatastarError, Result};
use serde::de::DeserializeOwned;

pub fn from_json_str<T: DeserializeOwned>(json_str: &str) -> Result<T> {
    serde_json::from_str(json_str).map_err(|e| DatastarError::SignalDeserialization(e.to_string()))
}

pub fn from_query_str<T: DeserializeOwned>(query_str: &str) -> Result<Option<T>> {
    let query_clean = query_str.strip_prefix('?').unwrap_or(query_str);
    for pair in query_clean.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (key, val) = match pair.split_once('=') {
            Some((k, v)) => (k, v),
            None => (pair, ""),
        };
        if key == "datastar" {
            let decoded = percent_decode(val);
            let parsed: T = serde_json::from_str(&decoded)
                .map_err(|e| DatastarError::SignalDeserialization(e.to_string()))?;
            return Ok(Some(parsed));
        }
    }
    Ok(None)
}

fn percent_decode(input: &str) -> String {
    let mut bytes = Vec::new();
    let mut chars = input.bytes();
    while let Some(b) = chars.next() {
        if b == b'%' {
            let h1 = chars.next().and_then(hex_val);
            let h2 = chars.next().and_then(hex_val);
            if let (Some(h1), Some(h2)) = (h1, h2) {
                bytes.push((h1 << 4) | h2);
                continue;
            }
        } else if b == b'+' {
            bytes.push(b' ');
            continue;
        }
        bytes.push(b);
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}
