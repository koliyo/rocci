const PICKER_HTML: &str = include_str!("../assets/picker.html");
const PICKER_CSS: &str = include_str!("../assets/picker.css");
const PICKER_JS: &str = include_str!("../assets/picker.js");

pub fn initialization_script() -> String {
    format!(
        "const __ROCCI_BROWSER_PICKER_HTML__ = {};\nconst __ROCCI_BROWSER_PICKER_CSS__ = {};\n{PICKER_JS}",
        json_string(PICKER_HTML.trim_end()),
        json_string(PICKER_CSS.trim_end()),
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
    fn picker_script_prevents_tab_default_and_uses_cmd_p() {
        let script = initialization_script();
        assert!(PICKER_JS.contains("event.key === \"Tab\""));
        assert!(PICKER_JS.contains("event.preventDefault()"));
        assert!(PICKER_JS.contains("event.key === \"p\""));
        assert!(PICKER_JS.contains("event.metaKey || event.ctrlKey"));
        assert!(!PICKER_JS.contains("event.key === \"k\""));
        assert!(PICKER_JS.contains("browser:open:"));
        assert!(PICKER_JS.contains("browser:list:"));
        assert!(script.contains("__ROCCI_BROWSER_PICKER_HTML__"));
        assert!(script.contains("rocci-browser-picker"));
    }
}
