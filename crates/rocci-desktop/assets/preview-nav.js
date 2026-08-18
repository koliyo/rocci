(function () {
  if (window.__rocciPreviewNav) {
    return;
  }
  const HEIGHT = "48px";
  const STORAGE_KEY = "rocci-dev-panel";
  const inspectorUrl =
    typeof __ROCCI_INSPECTOR_URL__ === "string" ? __ROCCI_INSPECTOR_URL__ : null;
  const host = document.createElement("rocci-preview-nav");
  const shadow = host.attachShadow({ mode: "open" });
  const sheet = document.createElement("style");
  sheet.textContent = __ROCCI_PREVIEW_NAV_CSS__;
  const tpl = document.createElement("template");
  tpl.innerHTML = __ROCCI_PREVIEW_NAV_HTML__;
  shadow.append(sheet, tpl.content);
  const back = shadow.getElementById("back");
  const forward = shadow.getElementById("forward");
  const title = shadow.getElementById("title");
  const path = shadow.getElementById("path");
  const dev = shadow.getElementById("dev");
  const send = (command) => {
    if (window.ipc && window.ipc.postMessage) {
      window.ipc.postMessage(command);
    }
  };
  back.addEventListener("click", () => send("back"));
  forward.addEventListener("click", () => send("forward"));
  shadow.getElementById("home").addEventListener("click", () => send("home"));
  shadow.getElementById("reload").addEventListener("click", () => send("reload"));
  let panel = null;
  const panelOpen = () => {
    try {
      return sessionStorage.getItem(STORAGE_KEY) === "1";
    } catch (err) {
      return false;
    }
  };
  const setPanelOpen = (open) => {
    try {
      sessionStorage.setItem(STORAGE_KEY, open ? "1" : "0");
    } catch (err) {}
    if (panel) {
      panel.classList.toggle("open", open);
    }
    if (dev) {
      dev.setAttribute("aria-pressed", open ? "true" : "false");
    }
  };
  if (inspectorUrl && dev) {
    dev.hidden = false;
    panel = document.createElement("rocci-preview-dev");
    const frame = document.createElement("iframe");
    frame.title = "Developer panel";
    frame.src = inspectorUrl;
    panel.append(frame);
    dev.addEventListener("click", () => setPanelOpen(!panel.classList.contains("open")));
  }
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
    },
  };
  const mount = () => {
    if (!host.isConnected && document.documentElement) {
      document.documentElement.prepend(host);
    }
    if (panel && !panel.isConnected && document.documentElement) {
      document.documentElement.append(panel);
      setPanelOpen(panelOpen());
    }
  };
  mount();
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", mount);
  }
  const spacer = document.createElement("style");
  spacer.textContent =
    "html { --rocci-chrome-top: " +
    HEIGHT +
    "; padding-top: " +
    HEIGHT +
    " !important; box-sizing: border-box; } rocci-preview-nav { display: block; position: fixed; top: 0; left: 0; right: 0; width: 100%; min-width: 100%; height: " +
    HEIGHT +
    "; overflow: hidden; background-color: #21252b; background-color: light-dark(#f7f7f8, #21252b); z-index: 2147483647; } rocci-preview-dev { display: none; position: fixed; top: var(--rocci-chrome-top, 48px); right: 0; bottom: 0; width: 320px; max-width: 100%; z-index: 2147483646; border-left: 1px solid #3e4451; border-left-color: light-dark(#e4e4e7, #3e4451); background: #21252b; background: light-dark(#f7f7f8, #21252b); box-sizing: border-box; } rocci-preview-dev.open { display: block; } rocci-preview-dev iframe { display: block; width: 100%; height: 100%; border: 0; background: transparent; }";
  document.documentElement.appendChild(spacer);
})();
