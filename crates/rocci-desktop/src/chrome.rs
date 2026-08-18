const PREVIEW_NAV_HTML: &str = include_str!("../assets/preview-nav.html");
const PREVIEW_NAV_CSS: &str = include_str!("../assets/preview-nav.css");
const REDUCED_MOTION_JS: &str = include_str!("../assets/reduced-motion.js");
const PREVIEW_NAV_JS: &str = include_str!("../assets/preview-nav.js");

pub fn initialization_script() -> String {
    format!(
        "{REDUCED_MOTION_JS}\nconst __ROCCI_PREVIEW_NAV_HTML__ = {};\nconst __ROCCI_PREVIEW_NAV_CSS__ = {};\n{PREVIEW_NAV_JS}",
        json_string(PREVIEW_NAV_HTML.trim_end()),
        json_string(PREVIEW_NAV_CSS.trim_end()),
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
        let script = initialization_script();
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
        for id in ["back", "forward", "home", "reload", "title", "path"] {
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
        ] {
            assert!(PREVIEW_NAV_HTML.contains(label), "missing {label}");
        }
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
