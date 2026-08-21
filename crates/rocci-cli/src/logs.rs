use std::{
    collections::VecDeque,
    io::{self, Write},
    sync::{Mutex, mpsc},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;

const CAPACITY: usize = 1000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LogLine {
    pub t: u64,
    pub level: String,
    pub source: String,
    pub text: String,
}

impl LogLine {
    pub fn runtime(level: LogLevel, text: impl Into<String>) -> Self {
        Self {
            t: now_millis(),
            level: level.as_str().to_string(),
            source: "runtime".into(),
            text: text.into(),
        }
    }
}

#[derive(Debug)]
pub struct LogHub {
    lines: Mutex<VecDeque<LogLine>>,
    waiters: Mutex<Vec<mpsc::Sender<LogLine>>>,
}

impl LogHub {
    pub fn new() -> Self {
        Self {
            lines: Mutex::new(VecDeque::new()),
            waiters: Mutex::new(Vec::new()),
        }
    }

    pub fn push(&self, level: LogLevel, text: impl Into<String>) {
        self.push_line(LogLine::runtime(level, text));
    }

    pub fn push_line(&self, line: LogLine) {
        let line = LogLine {
            text: crate::style::strip_ansi(&line.text),
            ..line
        };
        {
            let mut lines = self.lines.lock().unwrap_or_else(|err| err.into_inner());
            if lines.len() >= CAPACITY {
                lines.pop_front();
            }
            lines.push_back(line.clone());
        }
        self.waiters
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .retain(|tx| tx.send(line.clone()).is_ok());
    }

    pub fn snapshot(&self) -> Vec<LogLine> {
        self.lines
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .iter()
            .cloned()
            .collect()
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.snapshot()).unwrap_or_else(|_| "[]".into())
    }

    pub fn subscribe(&self) -> mpsc::Receiver<LogLine> {
        let (tx, rx) = mpsc::channel();
        self.waiters
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .push(tx);
        rx
    }

    pub fn clear(&self) {
        self.lines
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .clear();
    }
}

impl Default for LogHub {
    fn default() -> Self {
        Self::new()
    }
}

pub fn tee(hub: &LogHub, level: LogLevel, text: impl AsRef<str>) {
    let text = text.as_ref();
    emit(text);
    hub.push(level, text);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Progress {
    pub verbose: bool,
    pub quiet: bool,
}

impl Progress {
    pub fn step(self, message: impl AsRef<str>) {
        if self.quiet {
            return;
        }
        emit(message);
    }

    pub fn detail(self, message: impl AsRef<str>) {
        if self.quiet || !self.verbose {
            return;
        }
        emit(message);
    }
}

pub fn emit(message: impl AsRef<str>) {
    eprintln!("{}", message.as_ref());
    let _ = io::stderr().flush();
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_shape_includes_runtime_fields() {
        let hub = LogHub::new();
        hub.push(LogLevel::Warn, "watch failed");
        let value: serde_json::Value = serde_json::from_str(&hub.to_json()).unwrap();
        assert!(value.is_array());
        assert_eq!(value[0]["level"], "warn");
        assert_eq!(value[0]["source"], "runtime");
        assert_eq!(value[0]["text"], "watch failed");
        assert!(value[0]["t"].as_u64().unwrap() > 0);
    }

    #[test]
    fn hub_stores_plain_text_without_ansi() {
        let hub = LogHub::new();
        hub.push(
            LogLevel::Info,
            "\x1b[1;33mPOST\x1b[0m /actions/x -> \x1b[1;32mok\x1b[0m",
        );
        assert_eq!(hub.snapshot()[0].text, "POST /actions/x -> ok");
    }

    #[test]
    fn ring_drops_oldest_past_capacity() {
        let hub = LogHub::new();
        for i in 0..(CAPACITY + 3) {
            hub.push(LogLevel::Info, format!("line {i}"));
        }
        let lines = hub.snapshot();
        assert_eq!(lines.len(), CAPACITY);
        assert_eq!(lines[0].text, "line 3");
        assert_eq!(lines.last().unwrap().text, format!("line {}", CAPACITY + 2));
    }

    #[test]
    fn clear_empties_snapshot() {
        let hub = LogHub::new();
        hub.push(LogLevel::Info, "keep");
        hub.clear();
        assert!(hub.snapshot().is_empty());
        assert_eq!(hub.to_json(), "[]");
    }

    #[test]
    fn progress_quiet_skips_steps_and_verbose_requires_flag() {
        let quiet = Progress {
            verbose: true,
            quiet: true,
        };
        quiet.step("should not print");
        quiet.detail("should not print");
        let verbose = Progress {
            verbose: true,
            quiet: false,
        };
        assert!(verbose.verbose);
        let terse = Progress::default();
        assert!(!terse.verbose);
        assert!(!terse.quiet);
    }
}
