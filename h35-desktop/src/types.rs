use std::{
    fmt::{self, Display},
    sync::Arc,
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct WindowId(Arc<str>);

impl WindowId {
    pub fn new(label: impl AsRef<str>) -> Self {
        Self(label.as_ref().into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for WindowId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Debug)]
pub struct WindowConfig {
    pub label: String,
    pub title: String,
    pub url: String,
    pub width: f64,
    pub height: f64,
    pub min_width: Option<f64>,
    pub min_height: Option<f64>,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            label: "main".into(),
            title: "Preview".into(),
            url: "/".into(),
            width: 1040.0,
            height: 760.0,
            min_width: Some(720.0),
            min_height: Some(560.0),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WindowEvent {
    CloseRequested,
    Resized { width: u32, height: u32 },
    Focused(bool),
    Destroyed,
}
