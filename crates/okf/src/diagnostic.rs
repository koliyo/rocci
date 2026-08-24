use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocation {
    pub start: u32,
    pub end: u32,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    pub code: &'static str,
    pub severity: Severity,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<SourceLocation>,
    pub message: String,
}

impl Diagnostic {
    pub fn error(
        code: &'static str,
        path: impl Into<String>,
        location: Option<SourceLocation>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            severity: Severity::Error,
            path: path.into(),
            location,
            message: message.into(),
        }
    }

    pub fn warning(
        code: &'static str,
        path: impl Into<String>,
        location: Option<SourceLocation>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            severity: Severity::Warning,
            path: path.into(),
            location,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let severity = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        if let Some(location) = &self.location {
            write!(
                f,
                "{} {severity} {}:{}:{}: {}",
                self.code, self.path, location.line, location.column, self.message
            )
        } else {
            write!(
                f,
                "{} {severity} {}: {}",
                self.code, self.path, self.message
            )
        }
    }
}

pub const DIAGNOSTIC_CODES: &[&str] = &[
    "OKF1001", "OKF1002", "OKF1003", "OKF1004", "OKF1005", "OKF1006", "OKF1007", "OKF1008",
    "OKF1009", "OKF1010", "OKF1011", "OKF1012", "OKF1021", "OKF1022", "OKF2001", "OKF2002",
    "OKF2003", "OKF2004", "OKF2005", "OKF2006", "OKF2007", "OKF2009", "OKF2010", "OKF3001",
    "OKF3002", "OKF3003", "OKF3004", "OKF3005", "OKF4001", "OKF4002", "OKF4003", "OKF4004",
    "OKF4005", "OKF4006", "OKF4007", "OKF4008", "OKF4010",
];

pub fn intern_diagnostic_code(code: &str) -> Option<&'static str> {
    DIAGNOSTIC_CODES
        .iter()
        .copied()
        .find(|&known| known == code)
}
