use crate::span::{SourceFile, Span};
use std::fmt;
use std::io::IsTerminal;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub span: Span,
    pub severity: Severity,
    pub message: String,
    pub code: Option<&'static str>,
}

impl Diagnostic {
    pub fn error(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            severity: Severity::Error,
            message: message.into(),
            code: None,
        }
    }

    pub fn warning(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            severity: Severity::Warning,
            message: message.into(),
            code: None,
        }
    }

    pub fn error_code(code: &'static str, span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            severity: Severity::Error,
            message: message.into(),
            code: Some(code),
        }
    }

    pub fn warning_code(code: &'static str, span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            severity: Severity::Warning,
            message: message.into(),
            code: Some(code),
        }
    }

    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticFrame {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub severity: Severity,
    pub message: String,
    pub code: Option<&'static str>,
    pub source_line: String,
    pub caret_start: usize,
    pub caret_len: usize,
}

impl DiagnosticFrame {
    pub fn from_source(source: SourceFile<'_>, diagnostic: &Diagnostic) -> Self {
        let (line, column) = source.line_col(diagnostic.span.start);
        let (end_line, end_column) = source.line_col(diagnostic.span.end);
        let source_line = line_contents(source.src, line);
        let line_chars = source_line.chars().count();
        let caret_start = (column.saturating_sub(1) as usize).min(line_chars);
        let caret_end = if end_line == line && diagnostic.span.end > diagnostic.span.start {
            (end_column.saturating_sub(1) as usize).min(line_chars)
        } else {
            line_chars
        };
        let caret_len = caret_end.saturating_sub(caret_start).max(1);
        Self {
            file: source.name.to_string(),
            line,
            column,
            severity: diagnostic.severity,
            message: diagnostic.message.clone(),
            code: diagnostic.code,
            source_line,
            caret_start,
            caret_len,
        }
    }

    pub fn severity_label(&self) -> &'static str {
        match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        }
    }

    pub fn kind_label(&self) -> String {
        match self.code {
            Some(code) => format!("{}[{code}]", self.severity_label()),
            None => self.severity_label().to_string(),
        }
    }

    pub fn caret_line(&self) -> String {
        format!(
            "{}{}",
            " ".repeat(self.caret_start),
            "^".repeat(self.caret_len)
        )
    }

    pub fn render(&self) -> String {
        self.render_with_color(false)
    }

    pub fn render_for_stderr(&self) -> String {
        self.render_with_color(supports_ansi(std::io::stderr().is_terminal()))
    }

    pub fn render_with_color(&self, color: bool) -> String {
        let kind = self.kind_label();
        let (kind_code, caret_code) = match self.severity {
            Severity::Error => ("1;31", "1;31"),
            Severity::Warning => ("1;33", "1;33"),
        };
        let gutter = self.line.to_string().len().max(1);
        let pad = " ".repeat(gutter);
        let line_no = format!("{:>width$}", self.line, width = gutter);
        let arrow = paint(color, "1;34", "-->");
        let bar = paint(color, "1;34", "|");
        format!(
            "{kind}: {message}\n {arrow} {file}:{line}:{column}\n{pad} {bar}\n{line_no} {bar} {source}\n{pad} {bar} {caret}",
            kind = paint(color, kind_code, &kind),
            message = paint(color, "1", &self.message),
            file = self.file,
            line = self.line,
            column = self.column,
            source = self.source_line,
            caret = paint(color, caret_code, &self.caret_line()),
        )
    }
}

pub fn supports_ansi(is_terminal: bool) -> bool {
    if env_nonempty("NO_COLOR") {
        return false;
    }
    if env_enabled("CLICOLOR_FORCE") {
        return true;
    }
    is_terminal
}

fn env_nonempty(name: &str) -> bool {
    std::env::var_os(name).is_some_and(|value| !value.is_empty())
}

fn env_enabled(name: &str) -> bool {
    std::env::var_os(name).is_some_and(|value| value != "0" && !value.is_empty())
}

fn paint(color: bool, codes: &str, text: &str) -> String {
    if color {
        format!("\x1b[{codes}m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}

fn line_contents(src: &str, line: u32) -> String {
    src.lines()
        .nth(line.saturating_sub(1) as usize)
        .unwrap_or("")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::SourceFile;

    #[test]
    fn frame_points_at_the_span() {
        let src = "hello\n@page {\n";
        let source = SourceFile::new("Guide.rocdown", src);
        let at = src.find("@page").unwrap();
        let diagnostic = Diagnostic::error(Span::new(at, at + 5), "expected `{` to open `@page`");
        let frame = DiagnosticFrame::from_source(source, &diagnostic);
        assert_eq!(frame.line, 2);
        assert_eq!(frame.column, 1);
        assert_eq!(frame.source_line, "@page {");
        assert_eq!(frame.caret_start, 0);
        assert_eq!(frame.caret_len, 5);
        let rendered = frame.render();
        assert!(rendered.starts_with("error: expected `{` to open `@page`"));
        assert!(!rendered.contains("[RC"));
        assert!(rendered.contains(" --> Guide.rocdown:2:1"));
        assert!(rendered.contains("2 | @page {"));
        assert!(rendered.contains("  | ^^^^^"));
        let colored = frame.render_with_color(true);
        assert!(colored.contains("\x1b[1;31merror\x1b[0m"));
        assert!(colored.contains("\x1b[1;31m^^^^^\x1b[0m"));
        assert!(colored.contains("\x1b[1;34m-->\x1b[0m"));
        assert!(!rendered.contains('\x1b'));
    }

    #[test]
    fn coded_frame_includes_brackets_only_when_set() {
        let src = "@context {}\n@context {}\n";
        let source = SourceFile::new("App.rocci", src);
        let at = src.rfind("@context").unwrap();
        let diagnostic = Diagnostic::error_code(
            crate::codes::RC2001,
            Span::new(at, at + 8),
            "duplicate `@context`; a module may declare app state once",
        );
        let frame = DiagnosticFrame::from_source(source, &diagnostic);
        assert_eq!(frame.code, Some(crate::codes::RC2001));
        let rendered = frame.render();
        assert!(rendered.starts_with("error[RC2001]: duplicate `@context`"));
        let colored = frame.render_with_color(true);
        assert!(colored.contains("\x1b[1;31merror[RC2001]\x1b[0m"));
    }

    #[test]
    fn point_span_still_gets_a_caret() {
        let src = "abc";
        let source = SourceFile::new("x.rocci", src);
        let diagnostic = Diagnostic::error(Span::new(3, 3), "unexpected end of file");
        let frame = DiagnosticFrame::from_source(source, &diagnostic);
        assert_eq!(frame.source_line, "abc");
        assert_eq!(frame.caret_len, 1);
        assert!(frame.render().contains(" |    ^"));
    }

    #[test]
    fn warning_frames_use_yellow_when_colored() {
        let src = "hello";
        let source = SourceFile::new("x.rocci", src);
        let diagnostic = Diagnostic::warning(Span::new(0, 5), "unused");
        let frame = DiagnosticFrame::from_source(source, &diagnostic);
        let colored = frame.render_with_color(true);
        assert!(colored.contains("\x1b[1;33mwarning\x1b[0m"));
        assert!(colored.contains("\x1b[1;33m^^^^^\x1b[0m"));
    }

    #[test]
    fn ansi_stays_off_without_a_tty() {
        if std::env::var_os("CLICOLOR_FORCE").is_some() {
            return;
        }
        assert!(!supports_ansi(false));
    }
}
