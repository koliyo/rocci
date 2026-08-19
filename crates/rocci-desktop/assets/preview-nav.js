(function () {
  if (window.__rocciPreviewNav) {
    return;
  }
  const HEIGHT = "48px";
  const STORAGE_KEY = "rocci-dev-panel";
  const VIEW_KEY = "rocci-dev-view";
  const VIEWS = { source: true, ast: true, roc: true, html: true };
  const inspectorUrl =
    typeof __ROCCI_INSPECTOR_URL__ === "string" ? __ROCCI_INSPECTOR_URL__ : null;
  const hasSourceRoot = __ROCCI_HAS_SOURCE_ROOT__ === true;
  const revealLabel =
    typeof __ROCCI_REVEAL_LABEL__ === "string" ? __ROCCI_REVEAL_LABEL__ : "Reveal in Finder";
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
  const more = shadow.getElementById("more");
  const moreMenu = shadow.getElementById("more-menu");
  const reveal = shadow.getElementById("reveal");
  const copySource = shadow.getElementById("copy-source");
  const send = (command) => {
    if (window.ipc && window.ipc.postMessage) {
      window.ipc.postMessage(command);
    }
  };
  back.addEventListener("click", () => send("back"));
  forward.addEventListener("click", () => send("forward"));
  shadow.getElementById("home").addEventListener("click", () => send("home"));
  shadow.getElementById("reload").addEventListener("click", () => send("reload"));
  const routeOf = (value) => {
    try {
      const url = new URL(value, window.location.href);
      let pathname = url.pathname || "/";
      if (pathname.length > 1 && pathname.charAt(pathname.length - 1) !== "/") {
        pathname += "/";
      }
      return pathname;
    } catch (err) {
      return "/";
    }
  };
  let panel = null;
  let frame = null;
  const panelOpen = () => {
    try {
      return sessionStorage.getItem(STORAGE_KEY) === "1";
    } catch (err) {
      return false;
    }
  };
  const storedView = () => {
    try {
      const value = sessionStorage.getItem(VIEW_KEY);
      if (value && VIEWS[value]) {
        return value;
      }
    } catch (err) {}
    return "source";
  };
  const setStoredView = (value) => {
    if (!VIEWS[value]) {
      return;
    }
    try {
      sessionStorage.setItem(VIEW_KEY, value);
    } catch (err) {}
  };
  const rememberFrameView = () => {
    if (!frame) {
      return;
    }
    try {
      const href = frame.contentWindow.location.href;
      const view = new URL(href).searchParams.get("view");
      if (view) {
        setStoredView(view);
      }
    } catch (err) {}
  };
  const inspectorSrc = (route) => {
    const url = new URL(inspectorUrl, window.location.href);
    url.searchParams.set("route", route || "/");
    url.searchParams.set("view", storedView());
    return url.href;
  };
  const syncFrame = (route) => {
    if (!frame || !inspectorUrl) {
      return;
    }
    rememberFrameView();
    const next = inspectorSrc(route);
    if (frame.src !== next) {
      frame.src = next;
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
    if (open) {
      syncFrame(routeOf(window.location.href));
    }
  };
  if (inspectorUrl && dev) {
    dev.hidden = false;
    panel = document.createElement("rocci-preview-dev");
    frame = document.createElement("iframe");
    frame.title = "Developer panel";
    frame.src = inspectorSrc(routeOf(window.location.href));
    frame.addEventListener("load", rememberFrameView);
    panel.append(frame);
    dev.addEventListener("click", () => setPanelOpen(!panel.classList.contains("open")));
  }

  const sourceSpec = (rows) => {
    const current = routeOf(window.location.href);
    const list = rows || [];
    for (let i = 0; i < list.length; i++) {
      if (routeOf(list[i].url) === current && list[i].path) {
        return list[i].path;
      }
    }
    return current;
  };

  const loadRows = () => {
    const goto = window.__rocciPreviewNav && window.__rocciPreviewNav.goto;
    if (goto && typeof goto.loadCatalog === "function") {
      return goto.loadCatalog();
    }
    return Promise.resolve([]);
  };

  let menuOpen = false;
  const setMenuOpen = (open) => {
    menuOpen = open;
    if (moreMenu) {
      moreMenu.hidden = !open;
    }
    if (more) {
      more.setAttribute("aria-expanded", open ? "true" : "false");
    }
  };
  const closeMenu = () => setMenuOpen(false);

  if (hasSourceRoot && more && reveal && copySource) {
    more.hidden = false;
    reveal.textContent = revealLabel;
    more.addEventListener("click", function (event) {
      event.stopPropagation();
      setMenuOpen(!menuOpen);
    });
    reveal.addEventListener("click", function () {
      closeMenu();
      loadRows().then(function (rows) {
        send("reveal:" + sourceSpec(rows));
      });
    });
    copySource.addEventListener("click", function () {
      closeMenu();
      loadRows().then(function (rows) {
        send("copy-source:" + sourceSpec(rows));
      });
    });
    document.addEventListener(
      "mousedown",
      function (event) {
        if (!menuOpen) {
          return;
        }
        const path = event.composedPath ? event.composedPath() : [];
        if (path.indexOf(more) >= 0 || path.indexOf(moreMenu) >= 0) {
          return;
        }
        closeMenu();
      },
      true
    );
  }

  window.__rocciPreviewNav = {
    update(next) {
      if (typeof next.title === "string") {
        title.textContent = next.title;
      }
      if (typeof next.path === "string") {
        path.textContent = next.path;
        if (panel && panel.classList.contains("open")) {
          syncFrame(routeOf(next.path));
        }
      }
      if (typeof next.canBack === "boolean") {
        back.disabled = !next.canBack;
      }
      if (typeof next.canForward === "boolean") {
        forward.disabled = !next.canForward;
      }
      closeMenu();
    },
    closeMore: closeMenu,
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
    "; overflow: visible; background-color: #21252b; background-color: light-dark(#f7f7f8, #21252b); z-index: 2147483647; } rocci-preview-dev { display: none; position: fixed; top: var(--rocci-chrome-top, 48px); right: 0; bottom: 0; width: 28rem; max-width: 100%; z-index: 2147483646; border-left: 1px solid #3e4451; border-left-color: light-dark(#e4e4e7, #3e4451); background: #21252b; background: light-dark(#f7f7f8, #21252b); box-sizing: border-box; } rocci-preview-dev.open { display: block; } rocci-preview-dev iframe { display: block; width: 100%; height: 100%; border: 0; background: transparent; }";
  document.documentElement.appendChild(spacer);
})();
