const PREVIEW_NAV_HTML: &str = include_str!("../assets/preview-nav.html");
const PREVIEW_NAV_CSS: &str = include_str!("../assets/preview-nav.css");
const PREVIEW_FIND_HTML: &str = include_str!("../assets/preview-find.html");
const PREVIEW_FIND_CSS: &str = include_str!("../assets/preview-find.css");
const PREVIEW_FIND_JS: &str = include_str!("../assets/preview-find.js");
fn goto_js() -> String {
    include_str!("../assets/goto.js").to_string()
}
const PREVIEW_GOTO_JS: &str = include_str!("../assets/preview-goto.js");
const PREVIEW_KEYS_JS: &str = include_str!("../assets/preview-keys.js");
const REDUCED_MOTION_JS: &str = include_str!("../assets/reduced-motion.js");
const PREVIEW_NAV_JS: &str = include_str!("../assets/preview-nav.js");

pub const FIND_OPEN_SCRIPT: &str =
    "window.__h35PreviewNav&&window.__h35PreviewNav.find&&window.__h35PreviewNav.find.open()";
pub const FIND_NEXT_SCRIPT: &str =
    "window.__h35PreviewNav&&window.__h35PreviewNav.find&&window.__h35PreviewNav.find.next()";
pub const FIND_PREV_SCRIPT: &str =
    "window.__h35PreviewNav&&window.__h35PreviewNav.find&&window.__h35PreviewNav.find.prev()";
pub const FIND_USE_SELECTION_SCRIPT: &str = "window.__h35PreviewNav&&window.__h35PreviewNav.find&&window.__h35PreviewNav.find.useSelection()";
pub const GOTO_OPEN_SCRIPT: &str = "window.__h35Goto&&window.__h35Goto.open()||window.__h35PreviewNav&&window.__h35PreviewNav.goto&&window.__h35PreviewNav.goto.open()";
pub const PICKER_OPEN_SCRIPT: &str =
    "window.__h35Picker&&window.__h35Picker.open&&window.__h35Picker.open()";
pub const SELECT_ALL_SCRIPT: &str = "window.__h35PreviewNav&&window.__h35PreviewNav.selectAll&&window.__h35PreviewNav.selectAll()||document.execCommand(\"selectAll\")";

pub fn live_reload_set_script(enabled: bool) -> String {
    let on = if enabled { "true" } else { "false" };
    format!(
        "window.__h35LiveReload&&window.__h35LiveReload.set({on});window.__h35PreviewNav&&window.__h35PreviewNav.syncLiveReload&&window.__h35PreviewNav.syncLiveReload()"
    )
}

pub fn reveal_label() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "Reveal in Finder"
    }
    #[cfg(target_os = "windows")]
    {
        "Show in Explorer"
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        "Show in Files"
    }
}

pub fn set_inspector_script(url: &str) -> String {
    format!(
        "window.__h35PreviewNav&&window.__h35PreviewNav.setInspectorUrl&&window.__h35PreviewNav.setInspectorUrl({})",
        json_string(url)
    )
}

pub fn initialization_script(
    inspector_url: Option<&str>,
    has_source_root: bool,
    live_reload: bool,
    goto: bool,
    find: bool,
    inspector_prefs: Option<&crate::state::InspectorState>,
    layout: Option<&crate::state::WindowState>,
) -> String {
    let inspector = match inspector_url {
        Some(url) => json_string(url),
        None => "null".to_string(),
    };
    let prefs = match inspector_prefs {
        Some(state) => serde_json::to_string(state).unwrap_or_else(|_| "null".into()),
        None => "null".to_string(),
    };
    let layout = match layout {
        Some(state) if state.has_layout() => {
            let mut map = serde_json::Map::new();
            if let Some(nav) = &state.nav {
                map.insert("nav".into(), serde_json::Value::String(nav.clone()));
            }
            if let Some(outline) = &state.outline {
                map.insert("outline".into(), serde_json::Value::String(outline.clone()));
            }
            serde_json::Value::Object(map).to_string()
        }
        _ => "null".to_string(),
    };
    let seed = if live_reload {
        String::new()
    } else {
        "try{if(sessionStorage.getItem(\"h35-live-reload\")===null)sessionStorage.setItem(\"h35-live-reload\",\"0\")}catch(e){}\n".to_string()
    };
    let goto_js = if goto { goto_js() } else { String::new() };
    let find_js = if find { PREVIEW_FIND_JS } else { "" };
    let goto_alias = if goto { PREVIEW_GOTO_JS } else { "" };
    format!(
        "{seed}{REDUCED_MOTION_JS}\nconst __H35_PREVIEW_NAV_HTML__ = {};\nconst __H35_PREVIEW_NAV_CSS__ = {};\nconst __H35_PREVIEW_FIND_HTML__ = {};\nconst __H35_PREVIEW_FIND_CSS__ = {};\nconst __H35_INSPECTOR_URL__ = {inspector};\nconst __H35_INSPECTOR_PREFS__ = {prefs};\nconst __H35_LAYOUT__ = {layout};\nconst __H35_HAS_SOURCE_ROOT__ = {};\nconst __H35_UNIFIED_TITLEBAR__ = {};\nconst __H35_REVEAL_LABEL__ = {};\n{PREVIEW_NAV_JS}\n{find_js}\n{goto_js}\n{goto_alias}\n{PREVIEW_KEYS_JS}",
        json_string(PREVIEW_NAV_HTML.trim_end()),
        json_string(PREVIEW_NAV_CSS.trim_end()),
        json_string(PREVIEW_FIND_HTML.trim_end()),
        json_string(PREVIEW_FIND_CSS.trim_end()),
        if has_source_root { "true" } else { "false" },
        if cfg!(target_os = "macos") {
            "true"
        } else {
            "false"
        },
        json_string(reveal_label()),
    )
}

pub fn update_script(title: &str, path: &str, can_back: bool, can_forward: bool) -> String {
    format!(
        "window.__h35PreviewNav && window.__h35PreviewNav.update({{title:{},path:{},canBack:{},canForward:{}}})",
        json_string(title),
        json_string(path),
        if can_back { "true" } else { "false" },
        if can_forward { "true" } else { "false" },
    )
}

pub fn update_title_script(title: &str) -> String {
    format!(
        "window.__h35PreviewNav && window.__h35PreviewNav.update({{title:{}}})",
        json_string(title)
    )
}

fn json_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_has_navigation_controls() {
        let script = initialization_script(None, false, true, true, true, None, None);
        assert!(script.contains("window.ipc.postMessage"));
        assert!(script.contains("h35-preview-nav"));
        assert!(script.contains("--h35-chrome-top"));
        assert!(script.contains("tpl.innerHTML"));
        assert!(script.contains("const __H35_PREVIEW_NAV_HTML__"));
        assert!(script.contains("const __H35_PREVIEW_NAV_CSS__"));
        assert!(script.contains("width: 100%"));
        assert!(PREVIEW_NAV_JS.contains("__H35_PREVIEW_NAV_HTML__"));
        assert!(PREVIEW_NAV_JS.contains("__H35_PREVIEW_NAV_CSS__"));
        assert!(PREVIEW_NAV_CSS.contains("flex-wrap: nowrap"));
        for id in [
            "back",
            "forward",
            "home",
            "reload",
            "live-reload",
            "title",
            "path",
            "dev",
            "more",
            "reveal",
            "copy-source",
        ] {
            assert!(
                PREVIEW_NAV_HTML.contains(&format!("id=\"{id}\"")),
                "missing id {id}"
            );
        }
        for label in [
            "aria-label=\"Back\"",
            "aria-label=\"Forward\"",
            "aria-label=\"Home\"",
            "aria-label=\"Reload\"",
            "aria-label=\"Live reload\"",
            "aria-label=\"Developer panel\"",
            "aria-label=\"More actions\"",
        ] {
            assert!(PREVIEW_NAV_HTML.contains(label), "missing {label}");
        }
        assert!(PREVIEW_NAV_HTML.contains("Copy original document"));
        assert!(PREVIEW_NAV_JS.contains("__H35_INSPECTOR_URL__"));
        assert!(PREVIEW_NAV_JS.contains("__H35_HAS_SOURCE_ROOT__"));
        assert!(PREVIEW_NAV_JS.contains("__H35_REVEAL_LABEL__"));
        assert!(PREVIEW_NAV_HTML.contains("aria-pressed=\"true\""));
        assert!(PREVIEW_NAV_JS.contains("h35-live-reload"));
        assert!(PREVIEW_NAV_JS.contains("__h35LiveReload"));
        assert!(PREVIEW_NAV_JS.contains("syncLiveReload"));
        assert!(PREVIEW_NAV_JS.contains("live-reload:"));
        assert!(live_reload_set_script(true).contains("set(true)"));
        assert!(live_reload_set_script(false).contains("set(false)"));
        assert!(live_reload_set_script(false).contains("syncLiveReload"));
        let paused = initialization_script(None, false, false, true, true, None, None);
        assert!(paused.contains("sessionStorage.setItem(\"h35-live-reload\",\"0\")"));
        assert!(paused.contains("sessionStorage.getItem(\"h35-live-reload\")===null"));
        assert!(
            !initialization_script(None, false, true, true, true, None, None)
                .contains("sessionStorage.setItem(\"h35-live-reload\",\"0\")")
        );
        assert!(PREVIEW_NAV_CSS.contains("aria-pressed=\"true\""));
        assert!(PREVIEW_NAV_JS.contains("copy-source:"));
        assert!(PREVIEW_NAV_JS.contains("reveal:"));
        assert!(PREVIEW_NAV_JS.contains("h35-preview-dev"));
        assert!(PREVIEW_NAV_JS.contains("const UNIFIED = __H35_UNIFIED_TITLEBAR__ === true"));
        assert!(PREVIEW_NAV_JS.contains("const HEIGHT = UNIFIED ? \"52px\" : \"48px\""));
        assert!(PREVIEW_NAV_JS.contains("host.classList.add(\"unified\")"));
        assert!(PREVIEW_NAV_CSS.contains(":host(.unified)"));
        assert!(PREVIEW_NAV_CSS.contains("padding-left: 78px"));
        assert!(PREVIEW_NAV_JS.contains("send(\"drag\")"));
        assert!(PREVIEW_NAV_JS.contains("send(\"zoom\")"));
        assert!(PREVIEW_NAV_JS.contains("chromeInteractive"));
        assert!(script.contains("const __H35_UNIFIED_TITLEBAR__ = "));
        if cfg!(target_os = "macos") {
            assert!(script.contains("const __H35_UNIFIED_TITLEBAR__ = true"));
        } else {
            assert!(script.contains("const __H35_UNIFIED_TITLEBAR__ = false"));
        }
        assert!(PREVIEW_NAV_JS.contains("if (inspectorUrl && dev && !onInspectorPage())"));
        assert!(PREVIEW_NAV_JS.contains("params.set(\"tab\""));
        assert!(PREVIEW_NAV_JS.contains("params.set(\"route\""));
        assert!(PREVIEW_NAV_JS.contains("params.set(\"view\""));
        assert!(PREVIEW_NAV_JS.contains("h35-dev-view"));
        assert!(PREVIEW_NAV_JS.contains("h35-dev-tab"));
        assert!(PREVIEW_NAV_JS.contains("addEventListener(\"message\""));
        assert!(PREVIEW_NAV_JS.contains("h35-inspector"));
        assert!(PREVIEW_NAV_JS.contains("tuplesEqual"));
        assert!(!PREVIEW_NAV_JS.contains("frame.src !== next"));
        assert!(PREVIEW_NAV_JS.contains("width: var(--h35-chrome-right, 28rem)"));
        assert!(PREVIEW_NAV_JS.contains("--h35-chrome-right"));
        assert!(PREVIEW_NAV_JS.contains("--h35-chrome-bottom"));
        assert!(PREVIEW_NAV_JS.contains("overflow: hidden !important"));
        assert!(PREVIEW_NAV_JS.contains("overflow: auto !important"));
        assert!(PREVIEW_NAV_JS.contains(
            "inset: var(--h35-chrome-top) var(--h35-chrome-right, 0px) var(--h35-chrome-bottom, 0px) 0"
        ));
        assert!(PREVIEW_NAV_JS.contains("dock-right"));
        assert!(PREVIEW_NAV_JS.contains("dock-bottom"));
        assert!(
            PREVIEW_NAV_JS
                .contains("h35-preview-dev.open { display: flex; flex-direction: column; }")
        );
        assert!(
            PREVIEW_NAV_JS.contains(
                "h35-preview-dev iframe { display: block; flex: 1 1 auto; min-height: 0;"
            )
        );
        assert!(
            PREVIEW_NAV_JS.contains(
                "h35-preview-dev .h35-dev-docks { flex: 0 0 auto; display: flex; align-items: center; justify-content: flex-end;"
            )
        );
        assert!(PREVIEW_NAV_HTML.contains("h35-dev-mark"));
        assert!(PREVIEW_NAV_HTML.contains("class=\"icon h35-mark\""));
        assert!(!PREVIEW_NAV_JS.contains("dev-product"));
        assert!(PREVIEW_NAV_JS.contains("setNativeMode"));
        assert!(PREVIEW_NAV_JS.contains("classList.contains(\"native\")"));
        assert!(!PREVIEW_NAV_JS.contains("setPanelOpen(false);\n      send(\"devtools:1\")"));
        assert!(PREVIEW_NAV_JS.contains("overscroll-behavior: none"));
        assert!(PREVIEW_NAV_JS.contains("Open as page"));
        assert!(PREVIEW_NAV_JS.contains("Web Inspector"));
        assert!(PREVIEW_NAV_JS.contains("ICON_WEB_INSPECTOR"));
        assert!(PREVIEW_NAV_JS.contains("devtools:1"));
        assert!(PREVIEW_NAV_JS.contains("devtools:0"));
        assert!(PREVIEW_NAV_JS.contains("ICON_DOCK_RIGHT"));
        assert!(PREVIEW_NAV_JS.contains("persistPrefs"));
        assert!(PREVIEW_NAV_JS.contains("inspector-prefs:"));
        assert!(PREVIEW_NAV_JS.contains("__H35_INSPECTOR_PREFS__"));
        assert!(PREVIEW_NAV_JS.contains("onInspectorPage"));
        assert!(PREVIEW_NAV_JS.contains("Resize developer panel"));
        assert!(PREVIEW_NAV_JS.contains("h35-dev-splitter::after"));
        assert!(!PREVIEW_NAV_JS.contains("prefGet"));
        assert!(!PREVIEW_NAV_JS.contains("localStorage.setItem(VIEW_KEY"));
        let seeded = initialization_script(
            None,
            false,
            true,
            true,
            true,
            Some(&crate::state::InspectorState {
                open: true,
                dock: "bottom".into(),
                right: "30rem".into(),
                bottom: "40vh".into(),
                tab: "source".into(),
                view: "html".into(),
            }),
            Some(&crate::state::WindowState {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
                is_maximized: false,
                nav: Some("264px".into()),
                outline: Some("216px".into()),
            }),
        );
        assert!(seeded.contains("const __H35_INSPECTOR_PREFS__ = {"));
        assert!(seeded.contains("\"open\":true"));
        assert!(seeded.contains("\"dock\":\"bottom\""));
        assert!(seeded.contains("const __H35_LAYOUT__ = {"));
        assert!(seeded.contains("\"nav\":\"264px\""));
        assert!(
            initialization_script(None, false, true, true, true, None, None)
                .contains("const __H35_INSPECTOR_PREFS__ = null")
        );
        assert!(
            initialization_script(None, false, true, true, true, None, None)
                .contains("const __H35_LAYOUT__ = null")
        );
        assert!(
            !PREVIEW_NAV_JS
                .contains("h35-preview-dev.dock-right .h35-dev-docks { top: 0; left: 8px; }")
        );
        assert!(
            !PREVIEW_NAV_JS.contains(
                "position: relative; z-index: 2; flex: 0 0 auto; display: flex; gap: 2px; padding: 4px 8px; }"
            )
        );
        assert!(!PREVIEW_NAV_JS.contains("dockRightBtn.textContent = \"R\""));
        assert!(!PREVIEW_NAV_JS.contains("dockBottomBtn.textContent = \"B\""));
        assert!(PREVIEW_FIND_JS.contains("var(--h35-chrome-right"));
        assert!(PREVIEW_NAV_JS.contains(
            "h35-goto { right: var(--h35-chrome-right, 0px); bottom: var(--h35-chrome-bottom, 0px); }"
        ));
        assert!(PREVIEW_NAV_JS.contains("h35-dev-splitter"));
        assert!(PREVIEW_NAV_JS.contains("setPointerCapture"));
        assert!(PREVIEW_NAV_JS.contains("max-width: 80vw"));
        assert!(PREVIEW_NAV_JS.contains("max-height: 80vh"));
        assert!(PREVIEW_FIND_JS.contains("var(--h35-chrome-right"));
        assert!(!PREVIEW_NAV_JS.contains("width: 320px"));
        assert!(!PREVIEW_NAV_HTML.contains("<select"));
        assert!(!PREVIEW_NAV_HTML.contains("Original source"));
        assert!(PREVIEW_NAV_JS.contains("overflow: visible"));
        assert!(PREVIEW_NAV_JS.contains("body > header"));
        assert!(PREVIEW_NAV_CSS.contains("overflow: visible"));
        assert!(
            !initialization_script(None, false, true, true, true, None, None)
                .contains("http://127.0.0.1")
        );
        assert!(
            initialization_script(None, false, true, true, true, None, None)
                .contains("const __H35_HAS_SOURCE_ROOT__ = false")
        );
        assert!(
            initialization_script(None, true, true, true, true, None, None)
                .contains("const __H35_HAS_SOURCE_ROOT__ = true")
        );
        assert!(
            initialization_script(None, true, true, true, true, None, None)
                .contains(reveal_label())
        );
        let with_inspector = initialization_script(
            Some("http://127.0.0.1:9/__h35/dev"),
            false,
            true,
            true,
            true,
            None,
            None,
        );
        assert!(
            with_inspector
                .contains(r#"const __H35_INSPECTOR_URL__ = "http://127.0.0.1:9/__h35/dev""#)
        );
        assert!(
            initialization_script(None, false, true, true, true, None, None)
                .contains("const __H35_INSPECTOR_URL__ = null")
        );
        let cargo = include_str!("../Cargo.toml");
        assert!(!cargo.contains("rocci"));
        assert!(!cargo.contains("okmate"));
    }

    #[test]
    fn script_has_find_and_goto_overlays() {
        let script = initialization_script(None, false, true, true, true, None, None);
        assert!(script.contains("const __H35_PREVIEW_FIND_HTML__"));
        assert!(script.contains("const __H35_PREVIEW_FIND_CSS__"));
        assert!(script.contains("h35-preview-find"));
        assert!(script.contains("h35-goto"));
        assert!(script.contains("window.__h35Goto"));
        assert!(script.contains("window.__h35PreviewNav.find"));
        assert!(script.contains("window.__h35PreviewNav.goto"));
        assert!(PREVIEW_FIND_HTML.contains("id=\"query\""));
        assert!(PREVIEW_FIND_HTML.contains("aria-label=\"Find in page\""));
        assert!(goto_js().contains("aria-label=\"Go to page\""));
        assert!(PREVIEW_FIND_JS.contains("__h35PreviewNav.find"));
        assert!(PREVIEW_FIND_JS.contains("useSelection"));
        assert!(goto_js().contains("/pages.json"));
        assert!(goto_js().contains("/catalog.json"));
        assert!(goto_js().contains("loadCatalog"));
        assert!(goto_js().contains("history.pushState"));
        assert!(PREVIEW_GOTO_JS.contains("__h35Goto"));
        assert!(PREVIEW_KEYS_JS.contains("closeMore"));
        assert!(PREVIEW_KEYS_JS.contains("preventDefault"));
        assert!(PREVIEW_KEYS_JS.contains("selectAll"));
        assert!(PREVIEW_KEYS_JS.contains("key === \"a\""));
        assert!(
            PREVIEW_KEYS_JS.contains("[data-h35-select-root], article.article, article.article")
        );
        assert!(!PREVIEW_NAV_HTML.contains("copy-mode"));
        assert!(FIND_OPEN_SCRIPT.contains("find.open"));
        assert!(GOTO_OPEN_SCRIPT.contains("goto.open"));
        assert!(SELECT_ALL_SCRIPT.contains("selectAll"));
        assert!(PICKER_OPEN_SCRIPT.contains("__h35Picker"));
        assert!(set_inspector_script("http://127.0.0.1:9/inspect").contains("setInspectorUrl"));
        assert!(PREVIEW_NAV_JS.contains("setInspectorUrl"));
    }

    #[test]
    fn update_script_escapes_title_and_path() {
        let script = update_script(r#"Docs "docs""#, "/guides/docs", true, false);
        assert!(script.contains(r#"title:"Docs \"docs\"""#));
        assert!(script.contains(r#"path:"/guides/docs""#));
        assert!(script.contains("canBack:true"));
        assert!(script.contains("canForward:false"));
        assert!(update_title_script("Guide").contains(r#"title:"Guide""#));
        assert!(!update_title_script("Guide").contains("path:"));
    }

    #[test]
    fn crate_has_no_product_dependency() {
        let manifest = include_str!("../Cargo.toml");
        assert!(!manifest.contains("rocci"));
        assert!(!manifest.contains("okmate"));
        assert!(!manifest.contains("datastar"));
    }

    #[test]
    fn readme_describes_dock_chrome() {
        let readme = include_str!("../README.md");
        assert!(readme.contains("right (default"));
        assert!(readme.contains("bottom"));
        assert!(readme.contains("inspector.json"));
        assert!(readme.contains("sidebar column widths"));
        assert!(readme.contains("Open as page"));
        assert!(readme.contains("Web Inspector"));
        assert!(readme.contains("DevTools-style icons"));
        assert!(readme.contains("does not assign `iframe.src` for a Source `view`-only change"));
        assert!(readme.contains("unified titlebar"));
        assert!(readme.contains("52px overlay"));
        assert!(!readme.contains("undock"));
        assert!(!readme.contains("flex chrome strip"));
        assert!(!readme.contains("persist in `localStorage`"));
    }
}
