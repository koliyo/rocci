//! Rocci preview window: product defaults on top of `h35-desktop`.

use std::path::PathBuf;

use rocci_core::{Result, WindowConfig};

pub use h35_desktop::{NavigateHandler, PreviewEvent, PreviewSink, display_path};

const ICON_PNG: &[u8] = include_bytes!("../assets/rocci-icon.png");

const COMPAT_SCRIPT: &str = r#"
(function () {
  if (window.__h35PreviewNav && !window.__rocciPreviewNav) {
    window.__rocciPreviewNav = window.__h35PreviewNav;
  }
  if (window.__h35Goto && !window.__rocciGoto) {
    window.__rocciGoto = window.__h35Goto;
  }
  if (window.__rocciGoto && window.__h35PreviewNav && !window.__h35PreviewNav.goto) {
    window.__h35PreviewNav.goto = window.__rocciGoto;
  }
  if (window.__h35LiveReload && !window.__rocciLiveReload) {
    window.__rocciLiveReload = window.__h35LiveReload;
  }
  if (window.__h35Picker && !window.__rocciBrowser) {
    window.__rocciBrowser = window.__h35Picker;
  }
  window.addEventListener("h35-pick-folder", function (event) {
    window.dispatchEvent(new CustomEvent("rocci-pick-folder", { detail: event.detail }));
  });
})();
"#;

pub type IpcHandler = h35_desktop::IpcHandler;

pub struct PreviewOptions {
    pub url: String,
    pub title: String,
    pub width: f64,
    pub height: f64,
    pub devtools: bool,
    pub state_key: Option<String>,
    pub inspector_url: Option<String>,
    pub source_root: Option<PathBuf>,
    pub live_reload: bool,
    pub extra_initialization_script: Option<String>,
    pub on_ipc: Option<IpcHandler>,
    pub on_navigate: Option<NavigateHandler>,
    pub home_url: Option<String>,
    pub picker: bool,
}

impl Default for PreviewOptions {
    fn default() -> Self {
        let defaults = WindowConfig::default();
        Self {
            url: String::new(),
            title: defaults.title,
            width: defaults.width,
            height: defaults.height,
            devtools: true,
            state_key: None,
            inspector_url: None,
            source_root: None,
            live_reload: true,
            extra_initialization_script: None,
            on_ipc: None,
            on_navigate: None,
            home_url: None,
            picker: false,
        }
    }
}

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

pub fn preview(options: PreviewOptions) -> Result<()> {
    let identifier = options
        .state_key
        .clone()
        .unwrap_or_else(|| "preview".to_string());
    let state_dir = state_dir().unwrap_or_else(|| PathBuf::from("."));
    let mut extra = COMPAT_SCRIPT.to_string();
    if let Some(more) = &options.extra_initialization_script {
        extra.push('\n');
        extra.push_str(more);
    }
    h35_desktop::preview(h35_desktop::HostOptions {
        title: options.title,
        identifier,
        state_dir,
        url: options.url,
        home_url: options.home_url,
        icon_png: Some(ICON_PNG),
        live_reload: options.live_reload,
        source_root: options.source_root,
        inspector_url: options.inspector_url,
        picker: options.picker,
        goto: true,
        find: true,
        width: options.width,
        height: options.height,
        devtools: options.devtools,
        extra_initialization_script: Some(extra),
        on_ipc: options.on_ipc,
        on_navigate: options.on_navigate,
        ..h35_desktop::HostOptions::default()
    })
    .map_err(rocci_core::Error::message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compat_script_aliases_product_names() {
        assert!(COMPAT_SCRIPT.contains("__rocciPreviewNav"));
        assert!(COMPAT_SCRIPT.contains("__rocciGoto"));
        assert!(COMPAT_SCRIPT.contains("rocci-pick-folder"));
        assert!(COMPAT_SCRIPT.contains("h35-pick-folder"));
    }

    #[test]
    fn host_options_use_state_key_as_identifier() {
        let options = PreviewOptions {
            state_key: Some("rocci:view".into()),
            url: "http://127.0.0.1:9/".into(),
            ..PreviewOptions::default()
        };
        let identifier = options.state_key.unwrap();
        assert_eq!(identifier, "rocci:view");
    }

    #[test]
    fn ipc_handler_type_is_send() {
        let _handler: Option<IpcHandler> = None;
    }
}
