pub const INITIALIZATION_SCRIPT: &str = concat!(
    r#"
(function () {
  var nativeMatchMedia = window.matchMedia.bind(window);
  window.matchMedia = function (query) {
    var mql = nativeMatchMedia(query);
    if (String(query).toLowerCase().indexOf("prefers-reduced-motion") !== -1) {
      try {
        Object.defineProperty(mql, "matches", {
          configurable: true,
          get: function () {
            return false;
          },
        });
      } catch (err) {}
    }
    return mql;
  };
  var css = document.createElement("style");
  css.textContent =
    "@media (prefers-reduced-motion: reduce) { html, body { scroll-behavior: smooth; } }";
  if (document.documentElement) {
    document.documentElement.appendChild(css);
  } else {
    document.addEventListener("DOMContentLoaded", function () {
      document.documentElement.appendChild(css);
    });
  }
})();
"#,
    r#"
(function () {
  if (window.__rocciPreviewNav) {
    return;
  }
  const HEIGHT = "40px";
  const host = document.createElement("rocci-preview-nav");
  const shadow = host.attachShadow({ mode: "open" });
  shadow.innerHTML = `
    <style>
      :host {
        all: initial;
        display: block;
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        height: ${HEIGHT};
        z-index: 2147483647;
      }
      nav {
        box-sizing: border-box;
        display: flex;
        align-items: center;
        gap: 4px;
        height: ${HEIGHT};
        padding: 0 8px;
        border-bottom: 1px solid #d4d4d8;
        background: #f4f4f5;
        color: #18181b;
        font-family: system-ui, -apple-system, sans-serif;
        user-select: none;
      }
      button {
        box-sizing: border-box;
        width: 28px;
        height: 28px;
        padding: 0;
        border: 1px solid transparent;
        border-radius: 6px;
        background: transparent;
        color: inherit;
        font-size: 14px;
        line-height: 1;
        cursor: pointer;
      }
      button:hover:not(:disabled) {
        border-color: #d4d4d8;
        background: #e4e4e7;
      }
      button:disabled {
        cursor: default;
        opacity: 0.35;
      }
      .meta {
        min-width: 0;
        flex: 1;
        padding: 0 8px;
        line-height: 1.2;
      }
      .title, .path {
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
      }
      .title {
        font-size: 12px;
        font-weight: 600;
      }
      .path {
        font-size: 11px;
        color: #71717a;
      }
    </style>
    <nav>
      <button type="button" id="back" aria-label="Back" disabled>←</button>
      <button type="button" id="forward" aria-label="Forward" disabled>→</button>
      <button type="button" id="home" aria-label="Home">⌂</button>
      <button type="button" id="reload" aria-label="Reload">↻</button>
      <div class="meta">
        <div class="title" id="title"></div>
        <div class="path" id="path"></div>
      </div>
    </nav>
  `;
  const back = shadow.getElementById("back");
  const forward = shadow.getElementById("forward");
  const title = shadow.getElementById("title");
  const path = shadow.getElementById("path");
  const send = (command) => {
    if (window.ipc && window.ipc.postMessage) {
      window.ipc.postMessage(command);
    }
  };
  back.addEventListener("click", () => send("back"));
  forward.addEventListener("click", () => send("forward"));
  shadow.getElementById("home").addEventListener("click", () => send("home"));
  shadow.getElementById("reload").addEventListener("click", () => send("reload"));
  window.__rocciPreviewNav = {
    update(next) {
      if (typeof next.title === "string") {
        title.textContent = next.title;
      }
      if (typeof next.path === "string") {
        path.textContent = next.path;
      }
      if (typeof next.canBack === "boolean") {
        back.disabled = !next.canBack;
      }
      if (typeof next.canForward === "boolean") {
        forward.disabled = !next.canForward;
      }
    }
  };
  const mount = () => {
    if (!host.isConnected && document.documentElement) {
      document.documentElement.prepend(host);
    }
  };
  mount();
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", mount);
  }
  const spacer = document.createElement("style");
  spacer.textContent = "html { --rocci-chrome-top: " + HEIGHT + "; padding-top: " + HEIGHT + " !important; box-sizing: border-box; } rocci-preview-nav { display: block; position: fixed; top: 0; left: 0; right: 0; height: " + HEIGHT + "; z-index: 2147483647; }";
  document.documentElement.appendChild(spacer);
})();
"#
);

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
        for label in [
            "aria-label=\"Back\"",
            "aria-label=\"Forward\"",
            "aria-label=\"Home\"",
            "aria-label=\"Reload\"",
        ] {
            assert!(INITIALIZATION_SCRIPT.contains(label), "missing {label}");
        }
        assert!(INITIALIZATION_SCRIPT.contains("window.ipc.postMessage"));
        assert!(INITIALIZATION_SCRIPT.contains("rocci-preview-nav"));
        assert!(INITIALIZATION_SCRIPT.contains("--rocci-chrome-top"));
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
