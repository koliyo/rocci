use std::time::Instant;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileSnapshot {
    pub total_ms: u128,
    pub spans: Vec<ProfileSpan>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileSpan {
    pub name: String,
    pub duration_ms: u128,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Default)]
pub struct SpanRecorder {
    spans: Vec<ProfileSpan>,
}

impl SpanRecorder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn span<T, E>(
        &mut self,
        name: impl Into<String>,
        f: impl FnOnce() -> Result<T, E>,
    ) -> Result<T, E> {
        self.span_with_note(name, None, f)
    }

    pub fn span_with_note<T, E>(
        &mut self,
        name: impl Into<String>,
        note: Option<String>,
        f: impl FnOnce() -> Result<T, E>,
    ) -> Result<T, E> {
        let started = Instant::now();
        let result = f();
        self.spans.push(ProfileSpan {
            name: name.into(),
            duration_ms: started.elapsed().as_millis(),
            note,
        });
        result
    }

    pub fn push(&mut self, name: impl Into<String>, duration_ms: u128, note: Option<String>) {
        self.spans.push(ProfileSpan {
            name: name.into(),
            duration_ms,
            note,
        });
    }

    pub fn finish(self) -> ProfileSnapshot {
        let total_ms = self.spans.iter().map(|span| span.duration_ms).sum();
        ProfileSnapshot {
            total_ms,
            spans: self.spans,
        }
    }
}

impl ProfileSnapshot {
    pub fn merge(&mut self, other: Self) {
        self.total_ms += other.total_ms;
        self.spans.extend(other.spans);
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{\"total_ms\":0,\"spans\":[]}".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recorder_sums_spans_and_keeps_notes() {
        let mut recorder = SpanRecorder::new();
        recorder.push("read", 2, None);
        recorder.push("compile", 10, Some("cached".into()));
        let snapshot = recorder.finish();
        assert_eq!(snapshot.total_ms, 12);
        assert_eq!(snapshot.spans[1].note.as_deref(), Some("cached"));
        assert!(snapshot.to_json().contains("\"compile\""));
    }
}
