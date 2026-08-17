//! Window state persistence and geometry tracking.

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use tao::{dpi::LogicalPosition, event_loop::EventLoopWindowTarget, window::Window};

/// Saved geometry and layout state for a native window.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct WindowState {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    #[serde(default)]
    pub is_maximized: bool,
}

impl WindowState {
    pub fn new(x: f64, y: f64, width: f64, height: f64, is_maximized: bool) -> Self {
        Self {
            x,
            y,
            width,
            height,
            is_maximized,
        }
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

/// Return the path to the user's `.rocci/state` directory.
pub fn state_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("ROCCI_STATE_DIR")
        && !dir.is_empty()
    {
        return Some(PathBuf::from(dir));
    }
    if let Ok(home) = std::env::var("ROCCI_HOME")
        && !home.is_empty()
    {
        return Some(PathBuf::from(home).join(".rocci").join("state"));
    }
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
    Some(PathBuf::from(home).join(".rocci").join("state"))
}

/// Return the path to the `windows.json` state file.
pub fn window_state_path() -> Option<PathBuf> {
    Some(state_dir()?.join("windows.json"))
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

/// Load the full window state store from the default `.rocci/state/windows.json`.
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
    let physical_size = window.inner_size();

    let logical_pos = physical_pos.to_logical::<f64>(scale);
    let logical_size = physical_size.to_logical::<f64>(scale);

    Some(WindowState {
        x: logical_pos.x,
        y: logical_pos.y,
        width: logical_size.width,
        height: logical_size.height,
        is_maximized,
    })
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
        let temp_dir = env::temp_dir().join(format!("rocci-test-state-{}", uuid::Uuid::new_v4()));
        let original = env::var("ROCCI_STATE_DIR").ok();
        unsafe { env::set_var("ROCCI_STATE_DIR", &temp_dir) };

        assert_eq!(state_dir().unwrap(), temp_dir);
        assert_eq!(window_state_path().unwrap(), temp_dir.join("windows.json"));

        match original {
            Some(val) => unsafe { env::set_var("ROCCI_STATE_DIR", val) },
            None => unsafe { env::remove_var("ROCCI_STATE_DIR") },
        }
    }

    #[test]
    fn save_and_load_window_state_round_trip() {
        let temp_file =
            env::temp_dir().join(format!("rocci-test-{}/windows.json", uuid::Uuid::new_v4()));

        let state1 = WindowState::new(120.0, 80.0, 1024.0, 768.0, false);
        let state2 = WindowState::new(200.0, 150.0, 1400.0, 900.0, true);

        save_window_state_to(&temp_file, "rocs", state1).unwrap();
        save_window_state_to(&temp_file, "rocci:dev.rocci.snake", state2).unwrap();

        let loaded1 = load_window_state_from(&temp_file, "rocs");
        let loaded2 = load_window_state_from(&temp_file, "rocci:dev.rocci.snake");
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
            env::temp_dir().join(format!("rocci-test-{}/windows.json", uuid::Uuid::new_v4()));
        if let Some(parent) = temp_file.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&temp_file, "{ invalid json...").unwrap();

        assert_eq!(load_window_state_from(&temp_file, "rocs"), None);

        // Saving over corrupt state should succeed and replace with valid JSON
        let state = WindowState::new(50.0, 50.0, 800.0, 600.0, false);
        save_window_state_to(&temp_file, "rocs", state).unwrap();
        assert_eq!(load_window_state_from(&temp_file, "rocs"), Some(state));

        if let Some(parent) = temp_file.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }

    #[test]
    fn updating_one_key_preserves_others() {
        let temp_file =
            env::temp_dir().join(format!("rocci-test-{}/windows.json", uuid::Uuid::new_v4()));

        let state1 = WindowState::new(100.0, 100.0, 800.0, 600.0, false);
        let state2 = WindowState::new(200.0, 200.0, 1024.0, 768.0, false);
        let state1_updated = WindowState::new(150.0, 120.0, 900.0, 700.0, true);

        save_window_state_to(&temp_file, "rocs", state1).unwrap();
        save_window_state_to(&temp_file, "rocci:app", state2).unwrap();
        save_window_state_to(&temp_file, "rocs", state1_updated).unwrap();

        assert_eq!(
            load_window_state_from(&temp_file, "rocs"),
            Some(state1_updated)
        );
        assert_eq!(
            load_window_state_from(&temp_file, "rocci:app"),
            Some(state2)
        );

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
}
