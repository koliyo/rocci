(function () {
  if (window.__h35PreviewNav) {
    return;
  }
  const UNIFIED = __H35_UNIFIED_TITLEBAR__ === true;
  const HEIGHT = UNIFIED ? "52px" : "48px";
  const LIVE_RELOAD_KEY = "h35-live-reload";
  const LEGACY_PANEL_KEY = "h35-dev-panel";
  const LEGACY_VIEW_KEY = "h35-dev-view";
  const LEGACY_TAB_KEY = "h35-dev-tab";
  const LEGACY_DOCK_KEY = "h35-dev-dock";
  const LEGACY_DOCK_SIZE_KEY = "h35-dev-dock-size";
  const VIEWS = { source: true, ast: true, roc: true, html: true };
  const TABS = { performance: true, source: true, console: true };
  const DOCKS = { right: true, bottom: true };
  const DEFAULT_RIGHT = "28rem";
  const DEFAULT_BOTTOM = "36vh";
  const ICON_DOCK_RIGHT =
    '<svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true"><rect x="1.5" y="1.5" width="13" height="13" rx="1" fill="none" stroke="currentColor" stroke-width="1.25"/><rect x="9" y="2.75" width="4.5" height="10.5" fill="currentColor"/></svg>';
  const ICON_DOCK_BOTTOM =
    '<svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true"><rect x="1.5" y="1.5" width="13" height="13" rx="1" fill="none" stroke="currentColor" stroke-width="1.25"/><rect x="2.75" y="9" width="10.5" height="4.5" fill="currentColor"/></svg>';
  const ICON_EXPAND =
    '<svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true"><path fill="none" stroke="currentColor" stroke-width="1.25" d="M3 6.5V3h3.5M13 9.5V13H9.5M10 3h3v3M6 13H3V10"/><path fill="none" stroke="currentColor" stroke-width="1.25" d="M9.5 6.5 13 3M6.5 9.5 3 13"/></svg>';
  const ICON_WEB_INSPECTOR =
    '<svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true"><path fill="none" stroke="currentColor" stroke-width="1.25" stroke-linecap="round" stroke-linejoin="round" d="M5.5 3.5 2.5 8l3 4.5M10.5 3.5l3 4.5-3 4.5M9 3 7 13"/></svg>';
  let inspectorUrl =
    typeof __H35_INSPECTOR_URL__ === "string" ? __H35_INSPECTOR_URL__ : null;
  const hasSourceRoot = __H35_HAS_SOURCE_ROOT__ === true;
  const revealLabel =
    typeof __H35_REVEAL_LABEL__ === "string" ? __H35_REVEAL_LABEL__ : "Reveal in Finder";
  const host = document.createElement("h35-preview-nav");
  if (UNIFIED) {
    host.classList.add("unified");
  }
  const shadow = host.attachShadow({ mode: "open" });
  const sheet = document.createElement("style");
  sheet.textContent = __H35_PREVIEW_NAV_CSS__;
  const tpl = document.createElement("template");
  tpl.innerHTML = __H35_PREVIEW_NAV_HTML__;
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
  const legacyGet = (key) => {
    try {
      let value = localStorage.getItem(key);
      if (value === null) {
        value = sessionStorage.getItem(key);
      }
      return value;
    } catch (err) {
      return null;
    }
  };
  const legacyClear = (key) => {
    try {
      localStorage.removeItem(key);
      sessionStorage.removeItem(key);
    } catch (err) {}
  };
  const prefs = {
    open: false,
    dock: "right",
    right: DEFAULT_RIGHT,
    bottom: DEFAULT_BOTTOM,
    tab: "performance",
    view: "source",
  };
  const applyPrefSeed = (seed) => {
    if (!seed || typeof seed !== "object") {
      return false;
    }
    if (typeof seed.open === "boolean") {
      prefs.open = seed.open;
    }
    if (typeof seed.dock === "string" && DOCKS[seed.dock]) {
      prefs.dock = seed.dock;
    }
    if (typeof seed.right === "string" && seed.right) {
      prefs.right = seed.right;
    }
    if (typeof seed.bottom === "string" && seed.bottom) {
      prefs.bottom = seed.bottom;
    }
    if (typeof seed.tab === "string" && TABS[seed.tab]) {
      prefs.tab = seed.tab;
    }
    if (typeof seed.view === "string" && VIEWS[seed.view]) {
      prefs.view = seed.view;
    }
    return true;
  };
  if (!applyPrefSeed(typeof __H35_INSPECTOR_PREFS__ === "undefined" ? null : __H35_INSPECTOR_PREFS__)) {
    const legacyOpen = legacyGet(LEGACY_PANEL_KEY);
    const legacyDock = legacyGet(LEGACY_DOCK_KEY);
    const legacyTab = legacyGet(LEGACY_TAB_KEY);
    const legacyView = legacyGet(LEGACY_VIEW_KEY);
    let legacySizes = null;
    try {
      legacySizes = JSON.parse(legacyGet(LEGACY_DOCK_SIZE_KEY) || "null");
    } catch (err) {}
    if (legacyOpen !== null || legacyDock || legacyTab || legacyView || legacySizes) {
      applyPrefSeed({
        open: legacyOpen === "1",
        dock: legacyDock,
        right: legacySizes && legacySizes.right,
        bottom: legacySizes && legacySizes.bottom,
        tab: legacyTab,
        view: legacyView,
      });
      [
        LEGACY_PANEL_KEY,
        LEGACY_VIEW_KEY,
        LEGACY_TAB_KEY,
        LEGACY_DOCK_KEY,
        LEGACY_DOCK_SIZE_KEY,
      ].forEach(legacyClear);
      send(
        "inspector-prefs:" +
          JSON.stringify({
            open: prefs.open,
            dock: prefs.dock,
            right: prefs.right,
            bottom: prefs.bottom,
            tab: prefs.tab,
            view: prefs.view,
          })
      );
    }
  }
  const persistPrefs = () => {
    send(
      "inspector-prefs:" +
        JSON.stringify({
          open: prefs.open,
          dock: prefs.dock,
          right: prefs.right,
          bottom: prefs.bottom,
          tab: prefs.tab,
          view: prefs.view,
        })
    );
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
      if (window.__h35LiveReload && typeof window.__h35LiveReload.set === "function") {
        window.__h35LiveReload.set(next);
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
  let expandBtn = null;
  let webInspectorBtn = null;
  const panelOpen = () => !!prefs.open;
  const storedView = () => (VIEWS[prefs.view] ? prefs.view : "source");
  const setStoredView = (value) => {
    if (!VIEWS[value]) {
      return;
    }
    prefs.view = value;
    persistPrefs();
  };
  const storedTab = () => (TABS[prefs.tab] ? prefs.tab : "performance");
  const setStoredTab = (value) => {
    if (!TABS[value]) {
      return;
    }
    prefs.tab = value;
    persistPrefs();
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
  const onInspectorPage = () => {
    if (!inspectorUrl) {
      return false;
    }
    try {
      const insp = new URL(inspectorUrl, window.location.href);
      return window.location.origin === insp.origin && window.location.pathname === insp.pathname;
    } catch (err) {
      return false;
    }
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
  const storedDock = () => (DOCKS[prefs.dock] ? prefs.dock : "right");
  const setStoredDock = (value) => {
    if (!DOCKS[value]) {
      return;
    }
    prefs.dock = value;
    persistPrefs();
  };
  const storedSizes = () => ({
    right: prefs.right || DEFAULT_RIGHT,
    bottom: prefs.bottom || DEFAULT_BOTTOM,
  });
  const setStoredSize = (side, value) => {
    if (side === "right") {
      prefs.right = value;
    } else if (side === "bottom") {
      prefs.bottom = value;
    } else {
      return;
    }
    persistPrefs();
  };
  const remPx = () => {
    const size = parseFloat(getComputedStyle(document.documentElement).fontSize);
    return size > 0 ? size : 16;
  };
  const clampRight = (px) => Math.min(window.innerWidth * 0.8, Math.max(remPx() * 20, px));
  const clampBottom = (px) => Math.min(window.innerHeight * 0.8, Math.max(remPx() * 8, px));
  const NATIVE_RIGHT = "140px";
  const NATIVE_BOTTOM = "36px";
  const isNativeMode = () => panel && panel.classList.contains("native");
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
      root.style.setProperty("--h35-chrome-right", "0px");
      root.style.setProperty("--h35-chrome-bottom", "0px");
      return;
    }
    if (isNativeMode()) {
      if (side === "right") {
        root.style.setProperty("--h35-chrome-right", NATIVE_RIGHT);
        root.style.setProperty("--h35-chrome-bottom", "0px");
      } else {
        root.style.setProperty("--h35-chrome-right", "0px");
        root.style.setProperty("--h35-chrome-bottom", NATIVE_BOTTOM);
      }
      return;
    }
    if (side === "right") {
      root.style.setProperty("--h35-chrome-right", sizes.right);
      root.style.setProperty("--h35-chrome-bottom", "0px");
    } else {
      root.style.setProperty("--h35-chrome-right", "0px");
      root.style.setProperty("--h35-chrome-bottom", sizes.bottom);
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
  const setNativeMode = (on) => {
    if (!panel) {
      return;
    }
    const wasNative = panel.classList.contains("native");
    if (on === wasNative) {
      return;
    }
    panel.classList.toggle("native", on);
    if (frame) {
      frame.hidden = on;
    }
    if (splitter) {
      splitter.hidden = on;
    }
    if (webInspectorBtn) {
      webInspectorBtn.setAttribute("aria-pressed", on ? "true" : "false");
    }
    if (on) {
      send("devtools:1");
    } else {
      send("devtools:0");
      syncFrame(routeOf(window.location.href));
    }
    applyDock();
  };
  const setPanelOpen = (open) => {
    prefs.open = !!open;
    persistPrefs();
    if (panel) {
      panel.classList.toggle("open", open);
    }
    if (dev) {
      dev.setAttribute("aria-pressed", open ? "true" : "false");
    }
    if (!open && panel && panel.classList.contains("native")) {
      setNativeMode(false);
    }
    applyDock();
    if (open) {
      if (panel && panel.classList.contains("native")) {
        setNativeMode(false);
      } else {
        send("devtools:0");
        syncFrame(routeOf(window.location.href));
      }
    }
  };
  if (inspectorUrl && dev && !onInspectorPage()) {
    dev.hidden = false;
    panel = document.createElement("h35-preview-dev");
    panel.classList.add("dock-right");
    splitter = document.createElement("div");
    splitter.className = "h35-dev-splitter";
    splitter.setAttribute("role", "separator");
    splitter.setAttribute("aria-orientation", "vertical");
    splitter.setAttribute("aria-label", "Resize developer panel");
    const docks = document.createElement("div");
    docks.className = "h35-dev-docks";
    dockRightBtn = document.createElement("button");
    dockRightBtn.type = "button";
    dockRightBtn.setAttribute("aria-label", "Dock right");
    dockRightBtn.innerHTML = ICON_DOCK_RIGHT;
    dockBottomBtn = document.createElement("button");
    dockBottomBtn.type = "button";
    dockBottomBtn.setAttribute("aria-label", "Dock bottom");
    dockBottomBtn.innerHTML = ICON_DOCK_BOTTOM;
    expandBtn = document.createElement("button");
    expandBtn.type = "button";
    expandBtn.setAttribute("aria-label", "Open as page");
    expandBtn.innerHTML = ICON_EXPAND;
    webInspectorBtn = document.createElement("button");
    webInspectorBtn.type = "button";
    webInspectorBtn.setAttribute("aria-label", "Web Inspector");
    webInspectorBtn.setAttribute("title", "Native browser inspector");
    webInspectorBtn.innerHTML = ICON_WEB_INSPECTOR;
    docks.append(dockRightBtn, dockBottomBtn, expandBtn, webInspectorBtn);
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
    expandBtn.addEventListener("click", (event) => {
      event.stopPropagation();
      if (!inspectorUrl) {
        return;
      }
      const tuple = lastTuple || inspectorTuple(routeOf(window.location.href));
      window.location.href = inspectorHref(tuple, true);
    });
    webInspectorBtn.addEventListener("click", (event) => {
      event.stopPropagation();
      if (panel.classList.contains("native")) {
        setNativeMode(false);
      } else {
        if (!panel.classList.contains("open")) {
          setPanelOpen(true);
        }
        setNativeMode(true);
      }
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
      if (!data || data.type !== "h35-inspector") {
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
    dev.addEventListener("click", () => {
      if (panel.classList.contains("native")) {
        setNativeMode(false);
        if (!panel.classList.contains("open")) {
          setPanelOpen(true);
        }
      } else {
        setPanelOpen(!panel.classList.contains("open"));
      }
    });
  } else if (dev && onInspectorPage()) {
    dev.hidden = true;
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
    const goto = window.__h35PreviewNav && window.__h35PreviewNav.goto;
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

  window.__h35PreviewNav = {
    setInspectorUrl(url) {
      inspectorUrl = typeof url === "string" && url ? url : null;
      if (dev) {
        dev.hidden = !inspectorUrl || onInspectorPage();
      }
      if (inspectorUrl && panel && panel.classList.contains("open") && !onInspectorPage()) {
        syncFrame(routeOf(window.location.href));
      }
    },
    update(next) {
      if (typeof next.title === "string") {
        title.textContent = next.title;
      }
      if (typeof next.path === "string") {
        path.textContent = next.path;
        if (panel && panel.classList.contains("open") && !onInspectorPage()) {
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
  const chromeInteractive = (target) =>
    target &&
    target.closest &&
    target.closest("button, .more-menu, a, input, textarea, select, [role=\"menu\"]");
  const nav = shadow.querySelector("nav");
  if (UNIFIED && nav) {
    nav.addEventListener("mousedown", (event) => {
      if (event.button !== 0 || chromeInteractive(event.target)) {
        return;
      }
      send("drag");
    });
    nav.addEventListener("dblclick", (event) => {
      if (chromeInteractive(event.target)) {
        return;
      }
      send("zoom");
    });
  }
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
    "html { --h35-chrome-top: " +
    HEIGHT +
    "; --h35-chrome-right: 0px; --h35-chrome-bottom: 0px; height: 100%; overflow: hidden !important; padding: 0 !important; box-sizing: border-box; position: relative; } body { position: absolute !important; inset: var(--h35-chrome-top) var(--h35-chrome-right, 0px) var(--h35-chrome-bottom, 0px) 0 !important; overflow: auto !important; margin: 0 !important; padding: 0; box-sizing: border-box; } body > header, body > [role=\"banner\"] { top: 0 !important; } h35-preview-nav { display: block; position: fixed; top: 0; left: 0; right: 0; width: 100%; min-width: 100%; height: " +
    HEIGHT +
    "; overflow: visible; background-color: #21252b; background-color: light-dark(#f7f7f8, #21252b); z-index: 2147483647; } h35-preview-dev { display: none; position: fixed; z-index: 2147483646; box-sizing: border-box; overflow: hidden; overscroll-behavior: none; background: #21252b; background: light-dark(#f7f7f8, #21252b); } h35-preview-dev.open { display: flex; flex-direction: column; } h35-preview-dev.dock-right { top: var(--h35-chrome-top, 48px); right: 0; bottom: 0; width: var(--h35-chrome-right, 28rem); max-width: 80vw; border-left: 1px solid #3e4451; border-left-color: light-dark(#e4e4e7, #3e4451); } h35-preview-dev.dock-bottom { left: 0; right: 0; bottom: 0; height: var(--h35-chrome-bottom, 36vh); max-height: 80vh; border-top: 1px solid #3e4451; border-top-color: light-dark(#e4e4e7, #3e4451); } h35-preview-dev.native.dock-right { width: auto; max-width: none; } h35-preview-dev.native.dock-bottom { height: auto; max-height: none; } h35-preview-dev.native iframe { display: none !important; } h35-preview-dev.native .h35-dev-splitter { display: none; } h35-preview-dev iframe { display: block; flex: 1 1 auto; min-height: 0; width: 100%; height: auto; border: 0; background: transparent; } h35-preview-dev .h35-dev-splitter { position: absolute; z-index: 3; touch-action: none; } h35-preview-dev.dock-right .h35-dev-splitter { top: 0; bottom: 0; left: -4px; width: 8px; cursor: ew-resize; } h35-preview-dev.dock-bottom .h35-dev-splitter { top: -4px; left: 0; right: 0; height: 8px; cursor: ns-resize; } h35-preview-dev .h35-dev-splitter::after { content: \"\"; position: absolute; background: #3e4451; background: light-dark(#d4d4d8, #3e4451); opacity: 0.85; pointer-events: none; } h35-preview-dev.dock-right .h35-dev-splitter::after { top: 0; bottom: 0; left: 3px; width: 2px; } h35-preview-dev.dock-bottom .h35-dev-splitter::after { left: 0; right: 0; top: 3px; height: 2px; } h35-preview-dev .h35-dev-splitter:hover::after, h35-preview-dev .h35-dev-splitter:active::after { background: #61afef; background: light-dark(#3b82f6, #61afef); opacity: 1; } h35-preview-dev .h35-dev-docks { flex: 0 0 auto; display: flex; align-items: center; justify-content: flex-end; gap: 2px; padding: 4px 8px; border-bottom: 1px solid #3e4451; border-bottom-color: light-dark(#e4e4e7, #3e4451); } h35-preview-dev .h35-dev-docks button { box-sizing: border-box; display: inline-flex; align-items: center; justify-content: center; width: 24px; height: 24px; padding: 0; border: 1px solid #3e4451; border-color: light-dark(#d4d4d8, #3e4451); border-radius: 4px; background: light-dark(#ffffff, #2c313c); color: light-dark(#3f3f46, #d7dae0); cursor: pointer; } h35-preview-dev .h35-dev-docks button[aria-pressed=\"true\"] { border-color: #61afef; border-color: light-dark(#3b82f6, #61afef); background: light-dark(#eff6ff, #1e293b); } h35-preview-dev .h35-dev-docks button svg { display: block; } h35-goto { right: var(--h35-chrome-right, 0px); bottom: var(--h35-chrome-bottom, 0px); }";
  document.documentElement.appendChild(spacer);
})();
