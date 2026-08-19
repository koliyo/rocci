(function () {
  if (window.__rocciPreviewNav) {
    return;
  }
  const HEIGHT = "48px";
  const STORAGE_KEY = "rocci-dev-panel";
  const LIVE_RELOAD_KEY = "rocci-live-reload";
  const VIEW_KEY = "rocci-dev-view";
  const TAB_KEY = "rocci-dev-tab";
  const DOCK_KEY = "rocci-dev-dock";
  const DOCK_SIZE_KEY = "rocci-dev-dock-size";
  const VIEWS = { source: true, ast: true, roc: true, html: true };
  const TABS = { performance: true, source: true, console: true };
  const DOCKS = { right: true, bottom: true };
  const DEFAULT_RIGHT = "28rem";
  const DEFAULT_BOTTOM = "36vh";
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
  const liveReload = shadow.getElementById("live-reload");
  const liveReloadOn = () => {
    try {
      return sessionStorage.getItem(LIVE_RELOAD_KEY) !== "0";
    } catch (err) {
      return true;
    }
  };
  const syncLiveReloadButton = () => {
    if (liveReload) {
      liveReload.setAttribute("aria-pressed", liveReloadOn() ? "true" : "false");
    }
  };
  if (liveReload) {
    syncLiveReloadButton();
    send("live-reload:" + (liveReloadOn() ? "1" : "0"));
    liveReload.addEventListener("click", () => {
      const next = !liveReloadOn();
      if (window.__rocciLiveReload && typeof window.__rocciLiveReload.set === "function") {
        window.__rocciLiveReload.set(next);
      } else {
        try {
          sessionStorage.setItem(LIVE_RELOAD_KEY, next ? "1" : "0");
        } catch (err) {}
      }
      syncLiveReloadButton();
      send("live-reload:" + (next ? "1" : "0"));
    });
  }
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
  let splitter = null;
  let dockRightBtn = null;
  let dockBottomBtn = null;
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
  const storedTab = () => {
    try {
      const value = sessionStorage.getItem(TAB_KEY);
      if (value && TABS[value]) {
        return value;
      }
    } catch (err) {}
    return "performance";
  };
  const setStoredTab = (value) => {
    if (!TABS[value]) {
      return;
    }
    try {
      sessionStorage.setItem(TAB_KEY, value);
    } catch (err) {}
  };
  const normalizeRoute = (value) => {
    let route = value || "/";
    try {
      route = decodeURIComponent(route);
    } catch (err) {}
    if (!route || route.charAt(0) !== "/") {
      route = "/" + route;
    }
    if (route.length > 1 && route.charAt(route.length - 1) !== "/") {
      route += "/";
    }
    return route;
  };
  const inspectorOriginPath = () => {
    const url = new URL(inspectorUrl, window.location.href);
    return { origin: url.origin, path: url.pathname };
  };
  const inspectorTuple = (route) => {
    const base = inspectorOriginPath();
    return {
      origin: base.origin,
      path: base.path,
      tab: storedTab(),
      route: normalizeRoute(route),
    };
  };
  const tuplesEqual = (left, right) =>
    !!(
      left &&
      right &&
      left.origin === right.origin &&
      left.path === right.path &&
      left.tab === right.tab &&
      left.route === right.route
    );
  const inspectorHref = (tuple, includeView) => {
    const url = new URL(inspectorUrl, window.location.href);
    const params = new URLSearchParams();
    params.set("tab", tuple.tab);
    params.set("route", tuple.route);
    if (includeView) {
      params.set("view", storedView());
    }
    url.search = params.toString();
    return url.href;
  };
  let lastTuple = null;
  let receivedInspectorMessage = false;
  const storedDock = () => {
    try {
      const value = sessionStorage.getItem(DOCK_KEY);
      if (value && DOCKS[value]) {
        return value;
      }
    } catch (err) {}
    return "right";
  };
  const setStoredDock = (value) => {
    if (!DOCKS[value]) {
      return;
    }
    try {
      sessionStorage.setItem(DOCK_KEY, value);
    } catch (err) {}
  };
  const storedSizes = () => {
    const sizes = { right: DEFAULT_RIGHT, bottom: DEFAULT_BOTTOM };
    try {
      const parsed = JSON.parse(sessionStorage.getItem(DOCK_SIZE_KEY) || "null");
      if (parsed && typeof parsed.right === "string" && parsed.right) {
        sizes.right = parsed.right;
      }
      if (parsed && typeof parsed.bottom === "string" && parsed.bottom) {
        sizes.bottom = parsed.bottom;
      }
    } catch (err) {}
    return sizes;
  };
  const setStoredSize = (side, value) => {
    const sizes = storedSizes();
    sizes[side] = value;
    try {
      sessionStorage.setItem(DOCK_SIZE_KEY, JSON.stringify(sizes));
    } catch (err) {}
  };
  const remPx = () => {
    const size = parseFloat(getComputedStyle(document.documentElement).fontSize);
    return size > 0 ? size : 16;
  };
  const clampRight = (px) => Math.min(window.innerWidth * 0.8, Math.max(remPx() * 20, px));
  const clampBottom = (px) => Math.min(window.innerHeight * 0.8, Math.max(remPx() * 8, px));
  const applyDock = () => {
    const open = !!(panel && panel.classList.contains("open"));
    const side = storedDock();
    const sizes = storedSizes();
    const root = document.documentElement;
    if (panel) {
      panel.classList.toggle("dock-right", side === "right");
      panel.classList.toggle("dock-bottom", side === "bottom");
    }
    if (splitter) {
      splitter.setAttribute("aria-orientation", side === "right" ? "vertical" : "horizontal");
    }
    if (dockRightBtn) {
      dockRightBtn.setAttribute("aria-pressed", side === "right" ? "true" : "false");
    }
    if (dockBottomBtn) {
      dockBottomBtn.setAttribute("aria-pressed", side === "bottom" ? "true" : "false");
    }
    if (!open) {
      root.style.setProperty("--rocci-chrome-right", "0px");
      root.style.setProperty("--rocci-chrome-bottom", "0px");
      return;
    }
    if (side === "right") {
      root.style.setProperty("--rocci-chrome-right", sizes.right);
      root.style.setProperty("--rocci-chrome-bottom", "0px");
    } else {
      root.style.setProperty("--rocci-chrome-right", "0px");
      root.style.setProperty("--rocci-chrome-bottom", sizes.bottom);
    }
  };
  const assignFrame = (tuple, includeView) => {
    lastTuple = tuple;
    frame.src = inspectorHref(tuple, includeView);
  };
  const syncFrame = (route) => {
    if (!frame || !inspectorUrl) {
      return;
    }
    const next = inspectorTuple(route);
    if (tuplesEqual(lastTuple, next)) {
      return;
    }
    assignFrame(next, true);
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
    applyDock();
    if (open) {
      syncFrame(routeOf(window.location.href));
    }
  };
  if (inspectorUrl && dev) {
    dev.hidden = false;
    panel = document.createElement("rocci-preview-dev");
    panel.classList.add("dock-right");
    splitter = document.createElement("div");
    splitter.className = "rocci-dev-splitter";
    splitter.setAttribute("role", "separator");
    splitter.setAttribute("aria-orientation", "vertical");
    const docks = document.createElement("div");
    docks.className = "rocci-dev-docks";
    dockRightBtn = document.createElement("button");
    dockRightBtn.type = "button";
    dockRightBtn.setAttribute("aria-label", "Dock right");
    dockRightBtn.textContent = "R";
    dockBottomBtn = document.createElement("button");
    dockBottomBtn.type = "button";
    dockBottomBtn.setAttribute("aria-label", "Dock bottom");
    dockBottomBtn.textContent = "B";
    docks.append(dockRightBtn, dockBottomBtn);
    dockRightBtn.addEventListener("click", (event) => {
      event.stopPropagation();
      setStoredDock("right");
      applyDock();
    });
    dockBottomBtn.addEventListener("click", (event) => {
      event.stopPropagation();
      setStoredDock("bottom");
      applyDock();
    });
    splitter.addEventListener("pointerdown", (event) => {
      if (event.button !== 0) {
        return;
      }
      event.preventDefault();
      splitter.setPointerCapture(event.pointerId);
      const side = storedDock();
      const move = (ev) => {
        if (side === "right") {
          setStoredSize("right", Math.round(clampRight(window.innerWidth - ev.clientX)) + "px");
        } else {
          setStoredSize("bottom", Math.round(clampBottom(window.innerHeight - ev.clientY)) + "px");
        }
        applyDock();
      };
      const up = (ev) => {
        splitter.releasePointerCapture(ev.pointerId);
        splitter.removeEventListener("pointermove", move);
        splitter.removeEventListener("pointerup", up);
      };
      splitter.addEventListener("pointermove", move);
      splitter.addEventListener("pointerup", up);
    });
    frame = document.createElement("iframe");
    frame.title = "Developer panel";
    window.addEventListener("message", (event) => {
      if (!frame || event.source !== frame.contentWindow) {
        return;
      }
      const data = event.data;
      if (!data || data.type !== "rocci-inspector") {
        return;
      }
      receivedInspectorMessage = true;
      if (data.tab) {
        setStoredTab(data.tab);
      }
      if (data.view) {
        setStoredView(data.view);
      }
      if (lastTuple) {
        lastTuple = {
          origin: lastTuple.origin,
          path: lastTuple.path,
          tab: storedTab(),
          route: lastTuple.route,
        };
      }
    });
    assignFrame(inspectorTuple(routeOf(window.location.href)), true);
    panel.append(docks, splitter, frame);
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
    syncLiveReload: syncLiveReloadButton,
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
    "; --rocci-chrome-right: 0px; --rocci-chrome-bottom: 0px; padding-top: var(--rocci-chrome-top) !important; padding-right: var(--rocci-chrome-right) !important; padding-bottom: var(--rocci-chrome-bottom) !important; box-sizing: border-box; } rocci-preview-nav { display: block; position: fixed; top: 0; left: 0; right: 0; width: 100%; min-width: 100%; height: " +
    HEIGHT +
    "; overflow: visible; background-color: #21252b; background-color: light-dark(#f7f7f8, #21252b); z-index: 2147483647; } rocci-preview-dev { display: none; position: fixed; z-index: 2147483646; box-sizing: border-box; background: #21252b; background: light-dark(#f7f7f8, #21252b); } rocci-preview-dev.open { display: block; } rocci-preview-dev.dock-right { top: var(--rocci-chrome-top, 48px); right: 0; bottom: 0; width: var(--rocci-chrome-right, 28rem); max-width: 80vw; border-left: 1px solid #3e4451; border-left-color: light-dark(#e4e4e7, #3e4451); } rocci-preview-dev.dock-bottom { left: 0; right: 0; bottom: 0; height: var(--rocci-chrome-bottom, 36vh); max-height: 80vh; border-top: 1px solid #3e4451; border-top-color: light-dark(#e4e4e7, #3e4451); } rocci-preview-dev iframe { display: block; width: 100%; height: 100%; border: 0; background: transparent; } rocci-preview-dev .rocci-dev-splitter { position: absolute; z-index: 1; touch-action: none; } rocci-preview-dev.dock-right .rocci-dev-splitter { top: 0; bottom: 0; left: 0; width: 6px; cursor: ew-resize; } rocci-preview-dev.dock-bottom .rocci-dev-splitter { top: 0; left: 0; right: 0; height: 6px; cursor: ns-resize; } rocci-preview-dev .rocci-dev-docks { position: absolute; z-index: 2; display: flex; gap: 2px; padding: 4px; } rocci-preview-dev.dock-right .rocci-dev-docks { top: 0; left: 8px; } rocci-preview-dev.dock-bottom .rocci-dev-docks { top: 8px; right: 8px; } rocci-preview-dev .rocci-dev-docks button { box-sizing: border-box; width: 24px; height: 24px; padding: 0; border: 1px solid #3e4451; border-radius: 4px; background: light-dark(#ffffff, #2c313c); color: inherit; cursor: pointer; font-size: 11px; } rocci-goto { right: var(--rocci-chrome-right, 0px); bottom: var(--rocci-chrome-bottom, 0px); }";
  document.documentElement.appendChild(spacer);
})();
