use super::modes::PatchMode;
use std::fmt::Write;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatchElements<'a> {
    pub elements: &'a str,
    pub selector: Option<&'a str>,
    pub mode: PatchMode,
    pub use_view_transition: bool,
    pub focus_selector: Option<&'a str>,
    pub settle_duration_ms: Option<u32>,
}

impl<'a> PatchElements<'a> {
    pub fn new(elements: &'a str) -> Self {
        Self {
            elements,
            selector: None,
            mode: PatchMode::Outer,
            use_view_transition: false,
            focus_selector: None,
            settle_duration_ms: None,
        }
    }

    pub fn selector(mut self, selector: &'a str) -> Self {
        self.selector = Some(selector);
        self
    }

    pub fn mode(mut self, mode: PatchMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn view_transition(mut self, enable: bool) -> Self {
        self.use_view_transition = enable;
        self
    }

    pub fn focus_selector(mut self, focus: &'a str) -> Self {
        self.focus_selector = Some(focus);
        self
    }

    pub fn settle_duration(mut self, ms: u32) -> Self {
        self.settle_duration_ms = Some(ms);
        self
    }

    pub fn format_sse(&self) -> String {
        let mut out = String::from("event: datastar-patch-elements\n");
        if let Some(sel) = self.selector {
            let _ = writeln!(out, "data: selector {sel}");
        }
        if self.mode != PatchMode::Outer {
            let _ = writeln!(out, "data: mode {}", self.mode.as_str());
        }
        if self.use_view_transition {
            let _ = writeln!(out, "data: useViewTransition true");
        }
        if let Some(focus) = self.focus_selector {
            let _ = writeln!(out, "data: focusSelector {focus}");
        }
        if let Some(settle) = self.settle_duration_ms {
            let _ = writeln!(out, "data: settleDuration {settle}");
        }
        for line in self.elements.lines() {
            let _ = writeln!(out, "data: elements {line}");
        }
        out.push('\n');
        out
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatchSignals<'a> {
    pub signals: &'a str,
    pub only_if_missing: bool,
}

impl<'a> PatchSignals<'a> {
    pub fn new(signals: &'a str) -> Self {
        Self {
            signals,
            only_if_missing: false,
        }
    }

    pub fn only_if_missing(mut self, val: bool) -> Self {
        self.only_if_missing = val;
        self
    }

    pub fn format_sse(&self) -> String {
        let mut out = String::from("event: datastar-patch-signals\n");
        if self.only_if_missing {
            let _ = writeln!(out, "data: onlyIfMissing true");
        }
        let lf = self.signals.replace("\r\n", "\n");
        let normalized = lf.replace('\r', "\n");
        for line in normalized.split('\n') {
            let _ = writeln!(out, "data: signals {line}");
        }
        out.push('\n');
        out
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoveFragments<'a> {
    pub selector: &'a str,
    pub settle_duration_ms: Option<u32>,
    pub use_view_transition: bool,
}

impl<'a> RemoveFragments<'a> {
    pub fn new(selector: &'a str) -> Self {
        Self {
            selector,
            settle_duration_ms: None,
            use_view_transition: false,
        }
    }

    pub fn settle_duration(mut self, ms: u32) -> Self {
        self.settle_duration_ms = Some(ms);
        self
    }

    pub fn view_transition(mut self, enable: bool) -> Self {
        self.use_view_transition = enable;
        self
    }

    pub fn format_sse(&self) -> String {
        let mut out = String::from("event: datastar-remove-fragments\n");
        let _ = writeln!(out, "data: selector {}", self.selector);
        if let Some(settle) = self.settle_duration_ms {
            let _ = writeln!(out, "data: settleDuration {settle}");
        }
        if self.use_view_transition {
            let _ = writeln!(out, "data: useViewTransition true");
        }
        out.push('\n');
        out
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecuteScript<'a> {
    pub script: &'a str,
    pub auto_remove: bool,
    pub attributes: &'a [(&'a str, &'a str)],
}

impl<'a> ExecuteScript<'a> {
    pub fn new(script: &'a str) -> Self {
        Self {
            script,
            auto_remove: true,
            attributes: &[],
        }
    }

    pub fn auto_remove(mut self, auto_remove: bool) -> Self {
        self.auto_remove = auto_remove;
        self
    }

    pub fn attributes(mut self, attributes: &'a [(&'a str, &'a str)]) -> Self {
        self.attributes = attributes;
        self
    }

    pub fn format_sse(&self) -> String {
        let mut out = String::from("event: datastar-execute-script\n");
        if !self.auto_remove {
            let _ = writeln!(out, "data: autoRemove false");
        }
        for (k, v) in self.attributes {
            let _ = writeln!(out, "data: attributes {k} {v}");
        }
        for line in self.script.lines() {
            let _ = writeln!(out, "data: script {line}");
        }
        out.push('\n');
        out
    }
}
