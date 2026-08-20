const PREVIEW_NAV_HTML: &str = include_str!("../assets/preview-nav.html");
const PREVIEW_NAV_CSS: &str = include_str!("../assets/preview-nav.css");
const PREVIEW_FIND_HTML: &str = include_str!("../assets/preview-find.html");
const PREVIEW_FIND_CSS: &str = include_str!("../assets/preview-find.css");
const PREVIEW_FIND_JS: &str = include_str!("../assets/preview-find.js");
const GOTO_JS: &str = rocci_ui::GOTO_SCRIPT;
const PREVIEW_GOTO_JS: &str = include_str!("../assets/preview-goto.js");
const PREVIEW_KEYS_JS: &str = include_str!("../assets/preview-keys.js");
const REDUCED_MOTION_JS: &str = include_str!("../assets/reduced-motion.js");
const PREVIEW_NAV_JS: &str = include_str!("../assets/preview-nav.js");

pub const FIND_OPEN_SCRIPT: &str =
    "window.__rocciPreviewNav&&window.__rocciPreviewNav.find&&window.__rocciPreviewNav.find.open()";
pub const FIND_NEXT_SCRIPT: &str =
    "window.__rocciPreviewNav&&window.__rocciPreviewNav.find&&window.__rocciPreviewNav.find.next()";
pub const FIND_PREV_SCRIPT: &str =
    "window.__rocciPreviewNav&&window.__rocciPreviewNav.find&&window.__rocciPreviewNav.find.prev()";
pub const FIND_USE_SELECTION_SCRIPT: &str = "window.__rocciPreviewNav&&window.__rocciPreviewNav.find&&window.__rocciPreviewNav.find.useSelection()";
pub const GOTO_OPEN_SCRIPT: &str = "window.__rocciGoto&&window.__rocciGoto.open()||window.__rocciPreviewNav&&window.__rocciPreviewNav.goto&&window.__rocciPreviewNav.goto.open()";
pub const PICKER_OPEN_SCRIPT: &str =
    "window.__rocciBrowser&&window.__rocciBrowser.open&&window.__rocciBrowser.open()";
pub const SELECT_ALL_SCRIPT: &str = "window.__rocciPreviewNav&&window.__rocciPreviewNav.selectAll&&window.__rocciPreviewNav.selectAll()||document.execCommand(\"selectAll\")";

pub fn live_reload_set_script(enabled: bool) -> String {
    let on = if enabled { "true" } else { "false" };
    format!(
        "window.__rocciLiveReload&&window.__rocciLiveReload.set({on});window.__rocciPreviewNav&&window.__rocciPreviewNav.syncLiveReload&&window.__rocciPreviewNav.syncLiveReload()"
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
        "window.__rocciPreviewNav&&window.__rocciPreviewNav.setInspectorUrl&&window.__rocciPreviewNav.setInspectorUrl({})",
        json_string(url)
    )
}

pub fn initialization_script(
    inspector_url: Option<&str>,
    has_source_root: bool,
    live_reload: bool,
) -> String {
    let inspector = match inspector_url {
        Some(url) => json_string(url),
        None => "null".to_string(),
    };
    let seed = if live_reload {
        String::new()
    } else {
        "try{if(sessionStorage.getItem(\"rocci-live-reload\")===null)sessionStorage.setItem(\"rocci-live-reload\",\"0\")}catch(e){}\n".to_string()
    };
    format!(
        "{seed}{REDUCED_MOTION_JS}\nconst __ROCCI_PREVIEW_NAV_HTML__ = {};\nconst __ROCCI_PREVIEW_NAV_CSS__ = {};\nconst __ROCCI_PREVIEW_FIND_HTML__ = {};\nconst __ROCCI_PREVIEW_FIND_CSS__ = {};\nconst __ROCCI_INSPECTOR_URL__ = {inspector};\nconst __ROCCI_HAS_SOURCE_ROOT__ = {};\nconst __ROCCI_REVEAL_LABEL__ = {};\n{PREVIEW_NAV_JS}\n{PREVIEW_FIND_JS}\n{GOTO_JS}\n{PREVIEW_GOTO_JS}\n{PREVIEW_KEYS_JS}",
        json_string(PREVIEW_NAV_HTML.trim_end()),
        json_string(PREVIEW_NAV_CSS.trim_end()),
        json_string(PREVIEW_FIND_HTML.trim_end()),
        json_string(PREVIEW_FIND_CSS.trim_end()),
        if has_source_root { "true" } else { "false" },
        json_string(reveal_label()),
    )
}

pub fn update_script(title: &str, path: &str, can_back: bool, can_forward: bool) -> String {
    format!(
        "window.__rocciPreviewNav && window.__rocciPreviewNav.update({{title:{},path:{},canBack:{},canForward:{}}})",
        json_string(title),
        json_string(path),
        if can_back { "true" } else { "false" },
        if can_forward { "true" } else { "false" },
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
        let script = initialization_script(None, false, true);
        assert!(script.contains("window.ipc.postMessage"));
        assert!(script.contains("rocci-preview-nav"));
        assert!(script.contains("--rocci-chrome-top"));
        assert!(script.contains("tpl.innerHTML"));
        assert!(script.contains("const __ROCCI_PREVIEW_NAV_HTML__"));
        assert!(script.contains("const __ROCCI_PREVIEW_NAV_CSS__"));
        assert!(script.contains("width: 100%"));
        assert!(PREVIEW_NAV_JS.contains("__ROCCI_PREVIEW_NAV_HTML__"));
        assert!(PREVIEW_NAV_JS.contains("__ROCCI_PREVIEW_NAV_CSS__"));
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
        assert!(PREVIEW_NAV_JS.contains("__ROCCI_INSPECTOR_URL__"));
        assert!(PREVIEW_NAV_JS.contains("__ROCCI_HAS_SOURCE_ROOT__"));
        assert!(PREVIEW_NAV_JS.contains("__ROCCI_REVEAL_LABEL__"));
        assert!(PREVIEW_NAV_HTML.contains("aria-pressed=\"true\""));
        assert!(PREVIEW_NAV_JS.contains("rocci-live-reload"));
        assert!(PREVIEW_NAV_JS.contains("__rocciLiveReload"));
        assert!(PREVIEW_NAV_JS.contains("syncLiveReload"));
        assert!(PREVIEW_NAV_JS.contains("live-reload:"));
        assert!(live_reload_set_script(true).contains("set(true)"));
        assert!(live_reload_set_script(false).contains("set(false)"));
        assert!(live_reload_set_script(false).contains("syncLiveReload"));
        let paused = initialization_script(None, false, false);
        assert!(paused.contains("sessionStorage.setItem(\"rocci-live-reload\",\"0\")"));
        assert!(paused.contains("sessionStorage.getItem(\"rocci-live-reload\")===null"));
        assert!(
            !initialization_script(None, false, true)
                .contains("sessionStorage.setItem(\"rocci-live-reload\",\"0\")")
        );
        assert!(PREVIEW_NAV_CSS.contains("aria-pressed=\"true\""));
        assert!(PREVIEW_NAV_JS.contains("copy-source:"));
        assert!(PREVIEW_NAV_JS.contains("reveal:"));
        assert!(PREVIEW_NAV_JS.contains("rocci-preview-dev"));
        assert!(PREVIEW_NAV_JS.contains("const HEIGHT = \"48px\""));
        assert!(PREVIEW_NAV_JS.contains("if (inspectorUrl && dev)"));
        assert!(PREVIEW_NAV_JS.contains("params.set(\"tab\""));
        assert!(PREVIEW_NAV_JS.contains("params.set(\"route\""));
        assert!(PREVIEW_NAV_JS.contains("params.set(\"view\""));
        assert!(PREVIEW_NAV_JS.contains("rocci-dev-view"));
        assert!(PREVIEW_NAV_JS.contains("rocci-dev-tab"));
        assert!(PREVIEW_NAV_JS.contains("addEventListener(\"message\""));
        assert!(PREVIEW_NAV_JS.contains("rocci-inspector"));
        assert!(PREVIEW_NAV_JS.contains("tuplesEqual"));
        assert!(!PREVIEW_NAV_JS.contains("frame.src !== next"));
        assert!(PREVIEW_NAV_JS.contains("width: var(--rocci-chrome-right, 28rem)"));
        assert!(PREVIEW_NAV_JS.contains("--rocci-chrome-right"));
        assert!(PREVIEW_NAV_JS.contains("--rocci-chrome-bottom"));
        assert!(PREVIEW_NAV_JS.contains("padding-right: var(--rocci-chrome-right)"));
        assert!(PREVIEW_NAV_JS.contains("padding-bottom: var(--rocci-chrome-bottom)"));
        assert!(PREVIEW_NAV_JS.contains("dock-right"));
        assert!(PREVIEW_NAV_JS.contains("dock-bottom"));
        assert!(
            PREVIEW_NAV_JS
                .contains("rocci-preview-dev.open { display: flex; flex-direction: column; }")
        );
        assert!(
            PREVIEW_NAV_JS.contains(
                "rocci-preview-dev iframe { display: block; flex: 1 1 auto; min-height: 0;"
            )
        );
        assert!(
            PREVIEW_NAV_JS.contains("rocci-preview-dev .rocci-dev-docks { position: relative;")
        );
        assert!(
            !PREVIEW_NAV_JS
                .contains("rocci-preview-dev.dock-right .rocci-dev-docks { top: 0; left: 8px; }")
        );
        assert!(
            !PREVIEW_NAV_JS.contains(
                "position: absolute; z-index: 2; display: flex; gap: 2px; padding: 4px; }"
            )
        );
        assert!(PREVIEW_FIND_JS.contains("var(--rocci-chrome-right"));
        assert!(PREVIEW_NAV_JS.contains(
            "rocci-goto { right: var(--rocci-chrome-right, 0px); bottom: var(--rocci-chrome-bottom, 0px); }"
        ));
        assert!(PREVIEW_NAV_JS.contains("rocci-dev-splitter"));
        assert!(PREVIEW_NAV_JS.contains("setPointerCapture"));
        assert!(PREVIEW_NAV_JS.contains("max-width: 80vw"));
        assert!(PREVIEW_NAV_JS.contains("max-height: 80vh"));
        assert!(PREVIEW_FIND_JS.contains("var(--rocci-chrome-right"));
        assert!(!PREVIEW_NAV_JS.contains("width: 320px"));
        assert!(!PREVIEW_NAV_HTML.contains("<select"));
        assert!(!PREVIEW_NAV_HTML.contains("Original source"));
        assert!(PREVIEW_NAV_JS.contains("overflow: visible"));
        assert!(PREVIEW_NAV_CSS.contains("overflow: visible"));
        assert!(!initialization_script(None, false, true).contains("http://127.0.0.1"));
        assert!(
            initialization_script(None, false, true)
                .contains("const __ROCCI_HAS_SOURCE_ROOT__ = false")
        );
        assert!(
            initialization_script(None, true, true)
                .contains("const __ROCCI_HAS_SOURCE_ROOT__ = true")
        );
        assert!(initialization_script(None, true, true).contains(reveal_label()));
        let with_inspector =
            initialization_script(Some("http://127.0.0.1:9/__rocci/dev"), false, true);
        assert!(
            with_inspector
                .contains(r#"const __ROCCI_INSPECTOR_URL__ = "http://127.0.0.1:9/__rocci/dev""#)
        );
        assert!(
            initialization_script(None, false, true)
                .contains("const __ROCCI_INSPECTOR_URL__ = null")
        );
        let cargo = include_str!("../Cargo.toml");
        assert!(!cargo.contains("rocci-template"));
        assert!(!cargo.contains("rocci-rocdown"));
    }

    #[test]
    fn script_has_find_and_goto_overlays() {
        let script = initialization_script(None, false, true);
        assert!(script.contains("const __ROCCI_PREVIEW_FIND_HTML__"));
        assert!(script.contains("const __ROCCI_PREVIEW_FIND_CSS__"));
        assert!(script.contains("rocci-preview-find"));
        assert!(script.contains("rocci-goto"));
        assert!(script.contains("window.__rocciGoto"));
        assert!(script.contains("window.__rocciPreviewNav.find"));
        assert!(script.contains("window.__rocciPreviewNav.goto"));
        assert!(PREVIEW_FIND_HTML.contains("id=\"query\""));
        assert!(PREVIEW_FIND_HTML.contains("aria-label=\"Find in page\""));
        assert!(GOTO_JS.contains("aria-label=\"Go to page\""));
        assert!(PREVIEW_FIND_JS.contains("__rocciPreviewNav.find"));
        assert!(PREVIEW_FIND_JS.contains("useSelection"));
        assert!(GOTO_JS.contains("/pages.json"));
        assert!(GOTO_JS.contains("/catalog.json"));
        assert!(GOTO_JS.contains("loadCatalog"));
        assert!(GOTO_JS.contains("history.pushState"));
        assert!(PREVIEW_GOTO_JS.contains("__rocciGoto"));
        assert!(PREVIEW_KEYS_JS.contains("closeMore"));
        assert!(PREVIEW_KEYS_JS.contains("preventDefault"));
        assert!(PREVIEW_KEYS_JS.contains("selectAll"));
        assert!(PREVIEW_KEYS_JS.contains("key === \"a\""));
        assert!(
            PREVIEW_KEYS_JS.contains("[data-rd-select-root], article.rd-article, article.article")
        );
        assert!(!PREVIEW_NAV_HTML.contains("copy-mode"));
        assert!(FIND_OPEN_SCRIPT.contains("find.open"));
        assert!(GOTO_OPEN_SCRIPT.contains("goto.open"));
        assert!(SELECT_ALL_SCRIPT.contains("selectAll"));
        assert!(PICKER_OPEN_SCRIPT.contains("__rocciBrowser"));
        assert!(set_inspector_script("http://127.0.0.1:9/inspect").contains("setInspectorUrl"));
        assert!(PREVIEW_NAV_JS.contains("setInspectorUrl"));
    }

    #[test]
    fn update_script_escapes_title_and_path() {
        let script = update_script(r#"Rocdown "docs""#, "/guides/rocdown/", true, false);
        assert!(script.contains(r#"title:"Rocdown \"docs\"""#));
        assert!(script.contains(r#"path:"/guides/rocdown/""#));
        assert!(script.contains("canBack:true"));
        assert!(script.contains("canForward:false"));
    }
}
