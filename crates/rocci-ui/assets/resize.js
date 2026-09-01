(function () {
  if (window.__rocciResize) {
    return;
  }
  var NAV_KEY = "rocci-nav-width";
  var OUTLINE_KEY = "rocci-outline-width";
  var NAV_HOSTS = "#okf-nav, #site-nav, aside.sidebar";
  var OUTLINE_HOSTS = "#okf-toc, .layout-navigated > .outline, .site-grid > .outline";
  var SHELLS = ".rd-shell, .layout-navigated, .site-grid";
  var dragging = null;

  function hostLayout() {
    var seed = window.__ROCCI_LAYOUT__;
    return seed && typeof seed === "object" ? seed : null;
  }

  function hasHostStore() {
    return !!(window.ipc && window.ipc.postMessage);
  }

  function readStore(key) {
    try {
      return window.localStorage.getItem(key) || "";
    } catch (err) {
      return "";
    }
  }

  function writeStore(key, value) {
    try {
      window.localStorage.setItem(key, value);
    } catch (err) {}
  }

  function remPx() {
    var size = parseFloat(window.getComputedStyle(document.documentElement).fontSize);
    return size > 0 ? size : 16;
  }

  function storedWidth(kind) {
    var seed = hostLayout() || {};
    var fromHost = kind === "nav" ? seed.nav : seed.outline;
    if (fromHost) {
      return fromHost;
    }
    if (hasHostStore()) {
      return "";
    }
    return readStore(kind === "nav" ? NAV_KEY : OUTLINE_KEY);
  }

  function applyVars() {
    var root = document.documentElement;
    var nav = storedWidth("nav");
    var outline = storedWidth("outline");
    if (nav) {
      root.style.setProperty("--rocci-nav-width", nav);
    }
    if (outline) {
      root.style.setProperty("--rocci-outline-width", outline);
    }
  }

  function persistWidths() {
    var seed = hostLayout() || {};
    var nav = document.documentElement.style.getPropertyValue("--rocci-nav-width") || seed.nav || "";
    var outline =
      document.documentElement.style.getPropertyValue("--rocci-outline-width") || seed.outline || "";
    window.__ROCCI_LAYOUT__ = { nav: nav, outline: outline };
    if (hasHostStore()) {
      window.ipc.postMessage("layout:" + JSON.stringify({ nav: nav, outline: outline }));
      return;
    }
    if (nav) {
      writeStore(NAV_KEY, nav);
    }
    if (outline) {
      writeStore(OUTLINE_KEY, outline);
    }
  }

  function visible(el) {
    if (!el) {
      return false;
    }
    var style = window.getComputedStyle(el);
    return style.display !== "none" && style.visibility !== "hidden" && el.clientWidth > 0;
  }

  function clamp(kind, px) {
    var rem = remPx();
    if (kind === "nav") {
      return Math.round(Math.min(Math.max(px, 12 * rem), Math.max(12 * rem, window.innerWidth * 0.42)));
    }
    return Math.round(Math.min(Math.max(px, 10 * rem), Math.max(10 * rem, window.innerWidth * 0.36)));
  }

  function setWidth(kind, px, persist) {
    var value = clamp(kind, px) + "px";
    var prop = kind === "nav" ? "--rocci-nav-width" : "--rocci-outline-width";
    document.documentElement.style.setProperty(prop, value);
    if (persist) {
      persistWidths();
    }
    placeAll();
  }

  function hostWidth(kind) {
    var host = document.querySelector(kind === "nav" ? NAV_HOSTS : OUTLINE_HOSTS);
    return host && visible(host) ? host.getBoundingClientRect().width : 0;
  }

  function placeHandle(handle) {
    var kind = handle.getAttribute("data-rocci-resize");
    var host = document.querySelector(kind === "nav" ? NAV_HOSTS : OUTLINE_HOSTS);
    if (!host || !visible(host) || !handle.parentElement) {
      handle.style.display = "none";
      return;
    }
    handle.style.display = "";
    handle.style.left = "";
    handle.style.right = "";
  }

  function placeAll() {
    document.querySelectorAll(".rocci-col-resizer").forEach(placeHandle);
  }

  function bindHandle(handle, kind) {
    if (handle.__rocciBound) {
      return;
    }
    handle.__rocciBound = true;
    handle.addEventListener("pointerdown", function (event) {
      if (event.button !== 0) {
        return;
      }
      event.preventDefault();
      handle.setPointerCapture(event.pointerId);
      dragging = {
        kind: kind,
        startX: event.clientX,
        startW: hostWidth(kind),
      };
      handle.classList.add("is-active");
      document.body.classList.add("is-col-resizing");
    });
    handle.addEventListener("pointermove", function (event) {
      if (!dragging || dragging.kind !== kind) {
        return;
      }
      var delta = event.clientX - dragging.startX;
      if (kind === "outline") {
        delta = -delta;
      }
      setWidth(kind, dragging.startW + delta, false);
    });
    handle.addEventListener("pointerup", function (event) {
      if (!dragging || dragging.kind !== kind) {
        return;
      }
      try {
        handle.releasePointerCapture(event.pointerId);
      } catch (err) {}
      dragging = null;
      handle.classList.remove("is-active");
      document.body.classList.remove("is-col-resizing");
      persistWidths();
    });
    handle.addEventListener("keydown", function (event) {
      var step = event.shiftKey ? 32 : 16;
      if (event.key === "ArrowLeft") {
        event.preventDefault();
        setWidth(kind, hostWidth(kind) + (kind === "nav" ? -step : step), true);
      } else if (event.key === "ArrowRight") {
        event.preventDefault();
        setWidth(kind, hostWidth(kind) + (kind === "nav" ? step : -step), true);
      } else if (event.key === "Home") {
        event.preventDefault();
        setWidth(kind, kind === "nav" ? 12 * remPx() : 10 * remPx(), true);
      }
    });
  }

  function mount(host, kind) {
    var shell = host.closest(SHELLS);
    if (!shell || !visible(host)) {
      return;
    }
    if (shell.querySelector(':scope > .rocci-col-resizer[data-rocci-resize="' + kind + '"]')) {
      return;
    }
    var handle = document.createElement("div");
    handle.className = "rocci-col-resizer";
    var scope = shell.getAttribute("data-rocci-css") || host.getAttribute("data-rocci-css");
    if (scope) {
      handle.setAttribute("data-rocci-css", scope);
    }
    handle.setAttribute("data-rocci-resize", kind);
    handle.setAttribute("role", "separator");
    handle.setAttribute("aria-orientation", "vertical");
    handle.setAttribute("aria-label", kind === "nav" ? "Resize navigation" : "Resize outline");
    handle.tabIndex = 0;
    shell.appendChild(handle);
    bindHandle(handle, kind);
  }

  function enhance() {
    applyVars();
    document.querySelectorAll(NAV_HOSTS).forEach(function (host) {
      mount(host, "nav");
    });
    document.querySelectorAll(OUTLINE_HOSTS).forEach(function (host) {
      mount(host, "outline");
    });
    placeAll();
  }

  applyVars();
  window.__rocciResize = { enhance: enhance };
  window.addEventListener("resize", placeAll);
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", enhance);
  } else {
    enhance();
  }
})();
