//! Window state persistence and geometry tracking.

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use serde::{Deserialize, Serialize};
use tao::{
    dpi::{LogicalPosition, LogicalSize},
    event_loop::EventLoopWindowTarget,
    window::Window,
};

/// Saved geometry and layout state for a native window.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WindowState {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    #[serde(default)]
    pub is_maximized: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nav: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outline: Option<String>,
}

/// Saved Dev inspector panel preferences for a preview window.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InspectorState {
    #[serde(default)]
    pub open: bool,
    #[serde(default = "default_dock")]
    pub dock: String,
    #[serde(default = "default_right")]
    pub right: String,
    #[serde(default = "default_bottom")]
    pub bottom: String,
    #[serde(default = "default_tab")]
    pub tab: String,
    #[serde(default = "default_view")]
    pub view: String,
}

fn default_dock() -> String {
    "right".into()
}

fn default_right() -> String {
    "28rem".into()
}

fn default_bottom() -> String {
    "36vh".into()
}

fn default_tab() -> String {
    "performance".into()
}

fn default_view() -> String {
    "source".into()
}

impl Default for InspectorState {
    fn default() -> Self {
        Self {
            open: false,
            dock: default_dock(),
            right: default_right(),
            bottom: default_bottom(),
            tab: default_tab(),
            view: default_view(),
        }
    }
}

impl InspectorState {
    pub fn sanitized(self) -> Self {
        let dock = match self.dock.as_str() {
            "bottom" => "bottom",
            _ => "right",
        }
        .to_string();
        let tab = match self.tab.as_str() {
            "source" | "console" => self.tab.clone(),
            _ => default_tab(),
        };
        let view = match self.view.as_str() {
            "ast" | "html" | "source" => self.view.clone(),
            _ => default_view(),
        };
        let right = if self.right.trim().is_empty() {
            default_right()
        } else {
            self.right
        };
        let bottom = if self.bottom.trim().is_empty() {
            default_bottom()
        } else {
            self.bottom
        };
        Self {
            open: self.open,
            dock,
            right,
            bottom,
            tab,
            view,
        }
    }
}

impl WindowState {
    pub fn new(x: f64, y: f64, width: f64, height: f64, is_maximized: bool) -> Self {
        Self {
            x,
            y,
            width,
            height,
            is_maximized,
            nav: None,
            outline: None,
        }
    }

    pub fn has_layout(&self) -> bool {
        self.nav.is_some() || self.outline.is_some()
    }

    pub fn position(&self) -> LogicalPosition<f64> {
        LogicalPosition::new(self.x, self.y)
    }
}

/// Store of all persistent window geometries, keyed by window or app identifier.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WindowStateStore {
    #[serde(flatten)]
    pub windows: HashMap<String, WindowState>,
}

/// Store of Dev inspector preferences, keyed like [`WindowStateStore`].
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct InspectorStateStore {
    #[serde(flatten)]
    pub panels: HashMap<String, InspectorState>,
}

static STATE_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Set the directory used for `windows.json` and `inspector.json`.
pub fn set_state_dir(dir: PathBuf) {
    let _ = STATE_DIR.set(dir);
}

/// Return the configured state directory, or `H35_STATE_DIR` when unset.
pub fn state_dir() -> Option<PathBuf> {
    if let Some(dir) = STATE_DIR.get() {
        return Some(dir.clone());
    }
    if let Ok(dir) = std::env::var("H35_STATE_DIR")
        && !dir.is_empty()
    {
        return Some(PathBuf::from(dir));
    }
    None
}

/// Return the path to the `windows.json` state file.
pub fn window_state_path() -> Option<PathBuf> {
    Some(state_dir()?.join("windows.json"))
}

/// Return the path to the `inspector.json` state file.
pub fn inspector_state_path() -> Option<PathBuf> {
    Some(state_dir()?.join("inspector.json"))
}

/// Load the full window state store from a specific file path.
pub fn load_all_window_states_from(path: &Path) -> WindowStateStore {
    let Ok(content) = fs::read_to_string(path) else {
        return WindowStateStore::default();
    };
    serde_json::from_str(&content).unwrap_or_default()
}

/// Load the saved state for a specific key from a given file path.
pub fn load_window_state_from(path: &Path, key: &str) -> Option<WindowState> {
    load_all_window_states_from(path).windows.remove(key)
}

/// Save the given window state to a file path atomically.
pub fn save_window_state_to(path: &Path, key: &str, state: WindowState) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut store = load_all_window_states_from(path);
    store.windows.insert(key.to_string(), state);

    let json = serde_json::to_string_pretty(&store)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;

    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, json.as_bytes())?;
    fs::rename(&tmp_path, path)?;
    Ok(())
}

/// Load the full window state store from the configured state directory.
pub fn load_all_window_states() -> WindowStateStore {
    let Some(path) = window_state_path() else {
        return WindowStateStore::default();
    };
    load_all_window_states_from(&path)
}

/// Load the saved state for a specific key from the default state location.
pub fn load_window_state(key: &str) -> Option<WindowState> {
    let path = window_state_path()?;
    load_window_state_from(&path, key)
}

/// Save the given window state to the default state location atomically.
pub fn save_window_state(key: &str, state: WindowState) {
    let Some(path) = window_state_path() else {
        tracing::warn!(key, "cannot resolve state directory to save window state");
        return;
    };
    if let Err(err) = save_window_state_to(&path, key, state) {
        tracing::warn!(%err, key, path = %path.display(), "failed to save window state");
    }
}

/// Load the full inspector state store from a specific file path.
pub fn load_all_inspector_states_from(path: &Path) -> InspectorStateStore {
    let Ok(content) = fs::read_to_string(path) else {
        return InspectorStateStore::default();
    };
    serde_json::from_str(&content).unwrap_or_default()
}

/// Load the saved inspector prefs for a specific key from a given file path.
pub fn load_inspector_state_from(path: &Path, key: &str) -> Option<InspectorState> {
    load_all_inspector_states_from(path)
        .panels
        .remove(key)
        .map(InspectorState::sanitized)
}

/// Save inspector prefs to a file path atomically.
pub fn save_inspector_state_to(
    path: &Path,
    key: &str,
    state: InspectorState,
) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut store = load_all_inspector_states_from(path);
    store.panels.insert(key.to_string(), state.sanitized());

    let json = serde_json::to_string_pretty(&store)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;

    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, json.as_bytes())?;
    fs::rename(&tmp_path, path)?;
    Ok(())
}

/// Load inspector prefs for a key from the configured state directory.
pub fn load_inspector_state(key: &str) -> Option<InspectorState> {
    let path = inspector_state_path()?;
    load_inspector_state_from(&path, key)
}

/// Save inspector prefs to the default state location atomically.
pub fn save_inspector_state(key: &str, state: InspectorState) {
    let Some(path) = inspector_state_path() else {
        tracing::warn!(
            key,
            "cannot resolve state directory to save inspector state"
        );
        return;
    };
    if let Err(err) = save_inspector_state_to(&path, key, state) {
        tracing::warn!(%err, key, path = %path.display(), "failed to save inspector state");
    }
}

/// Parse inspector prefs JSON from an IPC payload.
pub fn parse_inspector_state_json(value: &str) -> Option<InspectorState> {
    serde_json::from_str::<InspectorState>(value)
        .ok()
        .map(InspectorState::sanitized)
}

#[derive(Clone, Debug, Default, Deserialize)]
struct LayoutPatch {
    nav: Option<String>,
    outline: Option<String>,
}

fn sanitize_track(value: Option<&str>) -> Option<String> {
    let value = value?.trim();
    if value.is_empty() {
        return None;
    }
    let bytes = value.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == 0 {
        return None;
    }
    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        let frac = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i == frac {
            return None;
        }
    }
    match &value[i..] {
        "px" | "rem" => Some(value.to_string()),
        _ => None,
    }
}

/// Merge sidebar widths from an IPC payload into the saved window record.
pub fn merge_layout_json(key: &str, value: &str) {
    let Ok(patch) = serde_json::from_str::<LayoutPatch>(value) else {
        return;
    };
    let nav = sanitize_track(patch.nav.as_deref());
    let outline = sanitize_track(patch.outline.as_deref());
    if nav.is_none() && outline.is_none() {
        return;
    }
    let mut state =
        load_window_state(key).unwrap_or_else(|| WindowState::new(0.0, 0.0, 0.0, 0.0, false));
    if nav.is_some() {
        state.nav = nav;
    }
    if outline.is_some() {
        state.outline = outline;
    }
    save_window_state(key, state);
}

/// Capture live window geometry and persist it when the size is still usable.
pub fn persist_window_state(key: &str, window: &Window) {
    let Some(mut state) = capture_window_state(window) else {
        return;
    };
    if state.width < 100.0 || state.height < 100.0 {
        return;
    }
    if let Some(existing) = load_window_state(key) {
        state.nav = existing.nav;
        state.outline = existing.outline;
    }
    save_window_state(key, state);
}

/// Check whether the given logical coordinates place at least a usable portion of the window
/// (e.g. its title bar / top-left area) on one of the currently available monitors.
pub fn is_position_visible<T>(
    event_loop: &EventLoopWindowTarget<T>,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> bool {
    let mut monitor_count = 0;
    for monitor in event_loop.available_monitors() {
        monitor_count += 1;
        let scale = monitor.scale_factor();
        if scale <= 0.0 {
            continue;
        }
        let pos = monitor.position();
        let size = monitor.size();
        let mon_x = pos.x as f64 / scale;
        let mon_y = pos.y as f64 / scale;
        let mon_w = size.width as f64 / scale;
        let mon_h = size.height as f64 / scale;

        let check_w = width.clamp(20.0, 100.0);
        let check_h = height.clamp(20.0, 40.0);

        if x + check_w > mon_x && x < mon_x + mon_w && y + check_h > mon_y && y < mon_y + mon_h {
            return true;
        }
    }

    // If no monitors were reported by tao (e.g. headless/mock env), assume visible
    monitor_count == 0
}

/// Capture the current logical position, inner size, and maximized state from a live Window.
pub fn capture_window_state(window: &Window) -> Option<WindowState> {
    let scale = window.scale_factor();
    if scale <= 0.0 {
        return None;
    }
    let is_maximized = window.is_maximized();
    let physical_pos = window.outer_position().ok()?;
    let logical_pos = physical_pos.to_logical::<f64>(scale);
    let logical_size = content_logical_size(window, scale)?;

    Some(WindowState {
        x: logical_pos.x,
        y: logical_pos.y,
        width: logical_size.width,
        height: logical_size.height,
        is_maximized,
        nav: None,
        outline: None,
    })
}

fn content_logical_size(window: &Window, scale: f64) -> Option<LogicalSize<f64>> {
    let outer = window.outer_size().to_logical::<f64>(scale);
    match (window.outer_position(), window.inner_position()) {
        (Ok(outer_pos), Ok(inner_pos)) => Some(content_size_from_frame(
            outer,
            outer_pos.to_logical(scale),
            inner_pos.to_logical(scale),
        )),
        _ => {
            let inner = window.inner_size().to_logical::<f64>(scale);
            if inner.width >= 1.0 && inner.height >= 1.0 {
                Some(inner)
            } else {
                None
            }
        }
    }
}

pub(crate) fn content_size_from_frame(
    outer: LogicalSize<f64>,
    outer_pos: LogicalPosition<f64>,
    inner_pos: LogicalPosition<f64>,
) -> LogicalSize<f64> {
    let chrome_left = (inner_pos.x - outer_pos.x).max(0.0);
    let chrome_top = (inner_pos.y - outer_pos.y).max(0.0);
    let chrome_right = chrome_left;
    let chrome_bottom = chrome_left;
    LogicalSize::new(
        (outer.width - chrome_left - chrome_right).max(1.0),
        (outer.height - chrome_top - chrome_bottom).max(1.0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn state_file_path_honors_env_override() {
        let _lock = ENV_LOCK.lock().unwrap();
        let temp_dir = env::temp_dir().join(format!("h35-test-state-{}", uuid::Uuid::new_v4()));
        let original = env::var("H35_STATE_DIR").ok();
        unsafe { env::set_var("H35_STATE_DIR", &temp_dir) };

        assert_eq!(state_dir().unwrap(), temp_dir);
        assert_eq!(window_state_path().unwrap(), temp_dir.join("windows.json"));
        assert_eq!(
            inspector_state_path().unwrap(),
            temp_dir.join("inspector.json")
        );

        match original {
            Some(val) => unsafe { env::set_var("H35_STATE_DIR", val) },
            None => unsafe { env::remove_var("H35_STATE_DIR") },
        }
    }

    #[test]
    fn save_and_load_window_state_round_trip() {
        let temp_file =
            env::temp_dir().join(format!("h35-test-{}/windows.json", uuid::Uuid::new_v4()));

        let state1 = WindowState::new(120.0, 80.0, 1024.0, 768.0, false);
        let state2 = WindowState::new(200.0, 150.0, 1400.0, 900.0, true);

        save_window_state_to(&temp_file, "docs", state1.clone()).unwrap();
        save_window_state_to(&temp_file, "h35:app.snake", state2.clone()).unwrap();

        let loaded1 = load_window_state_from(&temp_file, "docs");
        let loaded2 = load_window_state_from(&temp_file, "h35:app.snake");
        let loaded3 = load_window_state_from(&temp_file, "nonexistent");

        assert_eq!(loaded1, Some(state1));
        assert_eq!(loaded2, Some(state2));
        assert_eq!(loaded3, None);

        if let Some(parent) = temp_file.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }

    #[test]
    fn corrupt_json_falls_back_gracefully() {
        let temp_file =
            env::temp_dir().join(format!("h35-test-{}/windows.json", uuid::Uuid::new_v4()));
        if let Some(parent) = temp_file.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&temp_file, "{ invalid json...").unwrap();

        assert_eq!(load_window_state_from(&temp_file, "docs"), None);

        // Saving over corrupt state should succeed and replace with valid JSON
        let state = WindowState::new(50.0, 50.0, 800.0, 600.0, false);
        save_window_state_to(&temp_file, "docs", state.clone()).unwrap();
        assert_eq!(load_window_state_from(&temp_file, "docs"), Some(state));

        if let Some(parent) = temp_file.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }

    #[test]
    fn updating_one_key_preserves_others() {
        let temp_file =
            env::temp_dir().join(format!("h35-test-{}/windows.json", uuid::Uuid::new_v4()));

        let state1 = WindowState::new(100.0, 100.0, 800.0, 600.0, false);
        let state2 = WindowState::new(200.0, 200.0, 1024.0, 768.0, false);
        let state1_updated = WindowState::new(150.0, 120.0, 900.0, 700.0, true);

        save_window_state_to(&temp_file, "docs", state1).unwrap();
        save_window_state_to(&temp_file, "h35:app", state2.clone()).unwrap();
        save_window_state_to(&temp_file, "docs", state1_updated.clone()).unwrap();

        assert_eq!(
            load_window_state_from(&temp_file, "docs"),
            Some(state1_updated)
        );
        assert_eq!(load_window_state_from(&temp_file, "h35:app"), Some(state2));

        if let Some(parent) = temp_file.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }

    #[test]
    fn window_state_position_helper() {
        let state = WindowState::new(123.5, 456.7, 800.0, 600.0, true);
        assert_eq!(state.position(), LogicalPosition::new(123.5, 456.7));
        assert!(state.is_maximized);
    }

    #[test]
    fn content_size_from_frame_subtracts_title_bar() {
        let size = content_size_from_frame(
            LogicalSize::new(1200.0, 828.0),
            LogicalPosition::new(80.0, 40.0),
            LogicalPosition::new(80.0, 68.0),
        );
        assert_eq!(size, LogicalSize::new(1200.0, 800.0));
    }

    #[test]
    fn content_size_from_frame_subtracts_side_chrome() {
        let size = content_size_from_frame(
            LogicalSize::new(816.0, 639.0),
            LogicalPosition::new(10.0, 20.0),
            LogicalPosition::new(18.0, 51.0),
        );
        assert_eq!(size, LogicalSize::new(800.0, 600.0));
    }

    #[test]
    fn save_and_load_inspector_state_round_trip() {
        let temp_file =
            env::temp_dir().join(format!("h35-test-{}/inspector.json", uuid::Uuid::new_v4()));

        let state = InspectorState {
            open: true,
            dock: "bottom".into(),
            right: "30rem".into(),
            bottom: "40vh".into(),
            tab: "source".into(),
            view: "html".into(),
        };
        save_inspector_state_to(&temp_file, "preview", state.clone()).unwrap();
        assert_eq!(
            load_inspector_state_from(&temp_file, "preview"),
            Some(state)
        );
        assert_eq!(load_inspector_state_from(&temp_file, "missing"), None);

        if let Some(parent) = temp_file.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }

    #[test]
    fn inspector_state_sanitizes_unknown_fields() {
        let dirty = InspectorState {
            open: true,
            dock: "left".into(),
            right: String::new(),
            bottom: "  ".into(),
            tab: "metrics".into(),
            view: "wasm".into(),
        };
        assert_eq!(
            dirty.sanitized(),
            InspectorState {
                open: true,
                dock: "right".into(),
                right: "28rem".into(),
                bottom: "36vh".into(),
                tab: "performance".into(),
                view: "source".into(),
            }
        );
        assert_eq!(
            parse_inspector_state_json(
                r#"{"open":true,"dock":"bottom","right":"32rem","bottom":"30vh","tab":"console","view":"html"}"#
            ),
            Some(InspectorState {
                open: true,
                dock: "bottom".into(),
                right: "32rem".into(),
                bottom: "30vh".into(),
                tab: "console".into(),
                view: "html".into(),
            })
        );
    }

    #[test]
    fn layout_merge_keeps_geometry_and_rejects_css() {
        let _lock = ENV_LOCK.lock().unwrap();
        let temp_dir = env::temp_dir().join(format!("h35-test-{}", uuid::Uuid::new_v4()));
        let temp_file = temp_dir.join("windows.json");
        let original = env::var("H35_STATE_DIR").ok();
        unsafe { env::set_var("H35_STATE_DIR", &temp_dir) };

        let geometry = WindowState::new(40.0, 50.0, 1100.0, 720.0, false);
        save_window_state_to(&temp_file, "okf", geometry.clone()).unwrap();
        merge_layout_json("okf", r#"{"nav":"264px","outline":"12.5rem"}"#);
        merge_layout_json("okf", r#"{"nav":"url(evil)"}"#);

        let loaded = load_window_state("okf").unwrap();
        assert_eq!(loaded.x, 40.0);
        assert_eq!(loaded.y, 50.0);
        assert_eq!(loaded.width, 1100.0);
        assert_eq!(loaded.height, 720.0);
        assert_eq!(loaded.nav.as_deref(), Some("264px"));
        assert_eq!(loaded.outline.as_deref(), Some("12.5rem"));

        match original {
            Some(val) => unsafe { env::set_var("H35_STATE_DIR", val) },
            None => unsafe { env::remove_var("H35_STATE_DIR") },
        }
        let _ = fs::remove_dir_all(temp_dir);
    }
}
