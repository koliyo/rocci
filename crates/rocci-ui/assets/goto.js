(function () {
  const aliasPreview = (api) => {
    if (window.__rocciPreviewNav) {
      window.__rocciPreviewNav.goto = api;
    }
  };

  const setupNavSections = () => {
    if (window.__rocciNavSections) {
      return;
    }
    let transition = Promise.resolve();
    const storageKey = "rocci-nav-sections";
    const scrollStorageKey = "rocci-nav-scroll-positions";
    const reducedMotion = () =>
      window.matchMedia && window.matchMedia("(prefers-reduced-motion: reduce)").matches;

    const readSectionState = () => {
      try {
        return JSON.parse(sessionStorage.getItem(storageKey) || "{}");
      } catch (err) {
        return {};
      }
    };

    const rememberSection = (section) => {
      const key = section.getAttribute("data-rocci-nav-section");
      if (!key) {
        return;
      }
      const state = readSectionState();
      state[key] = section.open;
      try {
        sessionStorage.setItem(storageKey, JSON.stringify(state));
      } catch (err) {}
    };

    const rememberAllSections = () => {
      const sections = document.querySelectorAll("details[data-rocci-nav-section]");
      for (let i = 0; i < sections.length; i++) {
        rememberSection(sections[i]);
      }
    };

    const restoreSections = () => {
      const state = readSectionState();
      const sections = document.querySelectorAll("details[data-rocci-nav-section]");
      for (let i = 0; i < sections.length; i++) {
        const section = sections[i];
        const key = section.getAttribute("data-rocci-nav-section");
        if (Object.prototype.hasOwnProperty.call(state, key)) {
          section.open = !!state[key];
        }
      }
      rememberAllSections();
    };

    const readScrollPositions = () => {
      try {
        return JSON.parse(sessionStorage.getItem(scrollStorageKey) || "{}");
      } catch (err) {
        return {};
      }
    };

    const rememberScrollPositions = () => {
      const state = {};
      const sidebar = document.querySelector(".sidebar");
      const outline = document.querySelector(".layout-navigated > .outline");
      if (sidebar) {
        state.sidebar = sidebar.scrollTop;
      }
      if (outline) {
        state.outline = outline.scrollTop;
      }
      try {
        sessionStorage.setItem(scrollStorageKey, JSON.stringify(state));
      } catch (err) {}
    };

    const restoreScrollPositions = () => {
      const state = readScrollPositions();
      window.requestAnimationFrame(function () {
        const sidebar = document.querySelector(".sidebar");
        const outline = document.querySelector(".layout-navigated > .outline");
        if (sidebar && typeof state.sidebar === "number") {
          sidebar.scrollTop = state.sidebar;
        }
        if (outline && typeof state.outline === "number") {
          outline.scrollTop = state.outline;
        }
      });
    };

    const finishFold = (section, opening) => {
      const fold = section.querySelector(":scope > .nav-fold");
      if (!fold || reducedMotion() || typeof fold.animate !== "function") {
        section.open = opening;
        return Promise.resolve();
      }
      if (opening) {
        section.open = true;
      }
      const height = fold.scrollHeight;
      const frames = opening
        ? [
            { height: "0px", opacity: 0 },
            { height: height + "px", opacity: 1 },
          ]
        : [
            { height: height + "px", opacity: 1 },
            { height: "0px", opacity: 0 },
          ];
      const animation = fold.animate(frames, {
        duration: 180,
        easing: "ease",
        fill: "both",
      });
      return animation.finished
        .catch(function () {})
        .then(function () {
          section.open = opening;
          animation.cancel();
        });
    };

    document.addEventListener(
      "click",
      function (event) {
        const target = event.target && event.target.closest
          ? event.target.closest("details.nav-section > summary")
          : null;
        if (!target || event.button !== 0) {
          return;
        }
        event.preventDefault();
        const section = target.parentElement;
        transition = transition.then(function () {
          if (section.open) {
            return finishFold(section, false).then(function () {
              rememberSection(section);
            });
          }
          return finishFold(section, true).then(function () {
            rememberSection(section);
          });
        });
      },
      true
    );
    restoreSections();
    restoreScrollPositions();
    window.__rocciNavSections = {
      ready: true,
      restore: function () {
        restoreSections();
        restoreScrollPositions();
      },
      remember: function () {
        rememberAllSections();
        rememberScrollPositions();
      },
    };
  };

  setupNavSections();

  if (window.__rocciGoto) {
    aliasPreview(window.__rocciGoto);
    return;
  }

  const CACHE_KEY = "rocci-goto-catalog";
  const CSS =
    ':host{all:initial;color-scheme:inherit;display:none;position:fixed;inset:0 var(--rocci-chrome-right,0px) var(--rocci-chrome-bottom,0px) 0;z-index:2147483646;font-family:system-ui,-apple-system,"Segoe UI",sans-serif;color:#d7dae0;color:light-dark(#18181b,#d7dae0)}:host(.open){display:block}.backdrop{box-sizing:border-box;display:flex;justify-content:center;align-items:flex-start;width:100%;height:100%;padding:12vh 16px 16px;background:rgba(0,0,0,.42);background:light-dark(rgba(24,24,27,.28),rgba(0,0,0,.42))}.palette{box-sizing:border-box;width:min(560px,100%);max-height:min(70vh,480px);display:flex;flex-direction:column;border:1px solid #3e4451;border-color:light-dark(#e4e4e7,#3e4451);border-radius:12px;background:#21252b;background:light-dark(#f7f7f8,#21252b);box-shadow:0 16px 48px rgba(0,0,0,.35);overflow:hidden}input{box-sizing:border-box;width:100%;height:44px;margin:0;padding:0 14px;border:0;border-bottom:1px solid #3e4451;border-bottom-color:light-dark(#e4e4e7,#3e4451);background:transparent;color:inherit;font:inherit;font-size:15px}input:focus{outline:2px solid currentColor;outline-offset:-2px}#results{margin:0;padding:6px;list-style:none;overflow-y:auto;flex:1 1 auto}.item{display:flex;flex-direction:column;gap:2px;min-height:44px;box-sizing:border-box;padding:8px 10px;border-radius:8px;cursor:pointer}.item.is-selected,.item:hover{background:rgba(97,175,239,.16);background:light-dark(rgba(59,130,246,.12),rgba(97,175,239,.16))}.title{font-size:13.5px;font-weight:600}.path{color:#9da5b4;color:light-dark(#71717a,#9da5b4);font-size:12px;font-family:ui-monospace,Menlo,monospace;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.empty{margin:0;padding:18px 14px;color:#9da5b4;color:light-dark(#71717a,#9da5b4);font-size:13px}mark{background:transparent;color:#61afef;color:light-dark(#2563eb,#61afef);font-weight:700;padding:0}@media(max-width:480px){.backdrop{padding:8vh 8px 8px}.palette{max-height:84vh;border-radius:10px}}';
  const HTML =
    '<div class="backdrop" id="backdrop"><div class="palette" role="dialog" aria-modal="true" aria-label="Go to page"><input type="search" id="query" placeholder="Go to page" autocomplete="off" autocorrect="off" spellcheck="false" aria-label="Go to page" aria-controls="results"/><ul id="results" role="listbox" aria-label="Matching documents"></ul><p id="empty" class="empty" hidden>No matching documents</p></div></div>';

  const host = document.createElement("rocci-goto");
  const shadow = host.attachShadow({ mode: "open" });
  if (typeof CSSStyleSheet !== "undefined" && "adoptedStyleSheets" in shadow) {
    const sheet = new CSSStyleSheet();
    sheet.replaceSync(CSS);
    shadow.adoptedStyleSheets = [sheet];
  } else {
    const sheet = document.createElement("style");
    sheet.textContent = CSS;
    shadow.appendChild(sheet);
  }
  const tpl = document.createElement("template");
  tpl.innerHTML = HTML;
  shadow.append(tpl.content);
  const queryInput = shadow.getElementById("query");
  const resultsEl = shadow.getElementById("results");
  const emptyEl = shadow.getElementById("empty");
  let catalog = null;
  let catalogPromise = null;
  let filtered = [];
  let selected = 0;
  let lastAction = { name: "", at: 0 };
  let navigating = false;
  let lastFocused = null;
  let previousBodyOverflow = "";

  const once = (name, fn) => {
    const now = Date.now();
    if (lastAction.name === name && now - lastAction.at < 50) {
      return;
    }
    lastAction = { name: name, at: now };
    fn();
  };

  const isOpen = () => host.classList.contains("open");

  const fuzzy = (query, text) => {
    const q = query.toLowerCase();
    const t = String(text || "").toLowerCase();
    if (!q) {
      return 0;
    }
    if (!t) {
      return -1;
    }
    const exact = t.indexOf(q);
    if (exact >= 0) {
      return 1200 - exact + Math.min(q.length, 80);
    }
    let ti = 0;
    let score = 0;
    let run = 0;
    for (let qi = 0; qi < q.length; qi++) {
      const found = t.indexOf(q.charAt(qi), ti);
      if (found < 0) {
        return -1;
      }
      run = found === ti ? run + 1 : 1;
      score += run * 5 - (found - ti);
      ti = found + 1;
    }
    return score;
  };

  const scoreEntry = (query, entry) => {
    if (!query) {
      return 1;
    }
    let best = -1;
    const parts = [
      [entry.title, 1],
      [entry.path, 0.85],
      [entry.description, 0.4],
      [entry.url, 0.7],
    ];
    for (let i = 0; i < parts.length; i++) {
      const next = fuzzy(query, parts[i][0]);
      if (next >= 0) {
        best = Math.max(best, next * parts[i][1]);
      }
    }
    return best;
  };

  const escapeText = (value) => {
    return String(value || "")
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  };

  const highlightLabel = (text, query) => {
    const raw = String(text || "");
    if (!query) {
      return escapeText(raw);
    }
    const q = query.toLowerCase();
    const lower = raw.toLowerCase();
    const at = lower.indexOf(q);
    if (at < 0) {
      return escapeText(raw);
    }
    return (
      escapeText(raw.slice(0, at)) +
      "<mark>" +
      escapeText(raw.slice(at, at + query.length)) +
      "</mark>" +
      escapeText(raw.slice(at + query.length))
    );
  };

  const normalizePages = (rows) => {
    const out = [];
    for (let i = 0; i < rows.length; i++) {
      const row = rows[i] || {};
      const url = row.route || row.url || "";
      if (!url) {
        continue;
      }
      out.push({
        title: row.title || url,
        url: url,
        path: row.path || "",
        description: row.description || "",
        kind: row.kind || "",
        datastar: !!row.datastar,
      });
    }
    return out;
  };

  const normalizeCatalog = (rows) => {
    const out = [];
    for (let i = 0; i < rows.length; i++) {
      const row = rows[i] || {};
      const id = row.id || "";
      const meta = row.metadata || {};
      const title = typeof meta.title === "string" ? meta.title : id;
      const description = typeof meta.description === "string" ? meta.description : "";
      if (!id) {
        continue;
      }
      out.push({
        title: title || id,
        url: "/" + id.replace(/^\/+|\/+$/g, "") + "/",
        path: row.path || "",
        description: description,
        kind: "",
        datastar: false,
      });
    }
    return out;
  };

  const scrapeNav = () => {
    const seen = {};
    const out = [];
    const nodes = document.querySelectorAll(
      "aside a[href], a.nav-link[href], .lanes a[href], a.lane-link[href]"
    );
    for (let i = 0; i < nodes.length; i++) {
      const href = nodes[i].getAttribute("href");
      if (!href || href.charAt(0) === "#") {
        continue;
      }
      let url;
      try {
        url = new URL(href, window.location.href);
      } catch (err) {
        continue;
      }
      if (url.origin !== window.location.origin) {
        continue;
      }
      const key = url.pathname;
      if (seen[key]) {
        continue;
      }
      seen[key] = true;
      const title = (nodes[i].textContent || "").replace(/\s+/g, " ").replace(/^\s+|\s+$/g, "");
      out.push({
        title: title || key,
        url: key,
        path: key,
        description: "",
        kind: "",
        datastar: false,
      });
    }
    return out;
  };

  const fetchIndex = (path) => {
    return fetch(path, { credentials: "same-origin" }).then(function (res) {
      if (!res.ok) {
        throw new Error("missing");
      }
      return res.json();
    });
  };

  const loadCatalog = () => {
    if (catalog) {
      return Promise.resolve(catalog);
    }
    if (catalogPromise) {
      return catalogPromise;
    }
    let cached = null;
    try {
      cached = sessionStorage.getItem(CACHE_KEY);
    } catch (err) {}
    if (cached) {
      try {
        catalog = JSON.parse(cached);
        if (catalog && catalog.length) {
          return Promise.resolve(catalog);
        }
      } catch (err) {
        catalog = null;
      }
    }
    const remember = (rows) => {
      catalog = rows || [];
      if (catalog.length) {
        try {
          sessionStorage.setItem(CACHE_KEY, JSON.stringify(catalog));
        } catch (err) {}
      }
      return catalog;
    };
    catalogPromise = fetchIndex("/pages.json")
      .then(function (data) {
        if (!(Array.isArray(data) && data.length && data[0] && (data[0].route != null || data[0].url != null))) {
          throw new Error("skip");
        }
        return remember(normalizePages(data));
      })
      .catch(function () {
        return fetchIndex("/catalog.json").then(function (data) {
          if (!Array.isArray(data) || !data.length) {
            throw new Error("skip");
          }
          return remember(normalizeCatalog(data));
        });
      })
      .catch(function () {
        catalog = scrapeNav();
        return catalog;
      })
      .then(function (rows) {
        catalog = rows || catalog || [];
        catalogPromise = null;
        return catalog;
      });
    return catalogPromise;
  };

  const render = () => {
    resultsEl.textContent = "";
    if (!filtered.length) {
      emptyEl.hidden = false;
      return;
    }
    emptyEl.hidden = true;
    const query = queryInput.value;
    const limit = Math.min(filtered.length, 50);
    for (let i = 0; i < limit; i++) {
      const entry = filtered[i];
      const item = document.createElement("li");
      item.className = i === selected ? "item is-selected" : "item";
      item.setAttribute("role", "option");
      item.setAttribute("aria-selected", i === selected ? "true" : "false");
      item.innerHTML =
        '<span class="title">' +
        highlightLabel(entry.title, query) +
        '</span><span class="path">' +
        escapeText(entry.path || entry.url) +
        "</span>";
      item.addEventListener("mousedown", function (event) {
        event.preventDefault();
        go(entry.url, "push");
      });
      resultsEl.appendChild(item);
    }
    const active = resultsEl.children[selected];
    if (active && active.scrollIntoView) {
      active.scrollIntoView({ block: "nearest" });
    }
  };

  const filter = () => {
    const query = queryInput.value.replace(/^\s+|\s+$/g, "");
    const rows = catalog || [];
    const ranked = [];
    for (let i = 0; i < rows.length; i++) {
      const score = scoreEntry(query, rows[i]);
      if (score >= 0) {
        ranked.push({ score: score, entry: rows[i] });
      }
    }
    ranked.sort(function (a, b) {
      return b.score - a.score || a.entry.title.localeCompare(b.entry.title);
    });
    filtered = ranked.map(function (row) {
      return row.entry;
    });
    selected = 0;
    render();
  };

  const isChromeSrc = (src) => {
    if (!src) {
      return false;
    }
    return (
      /goto(\.[a-f0-9]+)?\.js(\?|#|$)/.test(src) ||
      /\/__rocci_okf\/goto\.js/.test(src) ||
      /\/__rocci_okf\/reload\.js/.test(src) ||
      /\/__rocci\/reload\.js/.test(src)
    );
  };

  const entryNeedsFullLoad = (entry) => {
    return !!(entry && (entry.kind === "live" || entry.datastar));
  };

  const docNeedsFullLoad = (doc) => {
    const scripts = doc.querySelectorAll("script");
    for (let i = 0; i < scripts.length; i++) {
      const src = scripts[i].getAttribute("src") || "";
      if (src) {
        if (!isChromeSrc(src)) {
          return true;
        }
        continue;
      }
      const text = scripts[i].textContent || "";
      if (text && text.indexOf("__rdTocScroll") < 0) {
        return true;
      }
    }
    return false;
  };

  const findEntry = (pathname) => {
    const rows = catalog || [];
    const key = pathname.endsWith("/") || pathname.endsWith(".html") ? pathname : pathname + "/";
    for (let i = 0; i < rows.length; i++) {
      const url = rows[i].url || "";
      if (url === pathname || url === key) {
        return rows[i];
      }
    }
    return null;
  };

  const copyMeta = (fromDoc, key, attr) => {
    const next = fromDoc.querySelector("meta[" + attr + '="' + key + '"]');
    let current = document.head.querySelector("meta[" + attr + '="' + key + '"]');
    if (!next) {
      if (current) {
        current.remove();
      }
      return;
    }
    if (!current) {
      current = document.createElement("meta");
      current.setAttribute(attr, key);
      document.head.appendChild(current);
    }
    current.setAttribute("content", next.getAttribute("content") || "");
  };

  const copyCanonical = (fromDoc) => {
    const next = fromDoc.querySelector('link[rel="canonical"]');
    let current = document.head.querySelector('link[rel="canonical"]');
    if (!next) {
      if (current) {
        current.remove();
      }
      return;
    }
    if (!current) {
      current = document.createElement("link");
      current.setAttribute("rel", "canonical");
      document.head.appendChild(current);
    }
    current.setAttribute("href", next.getAttribute("href") || "");
  };

  const applyDocument = (fromDoc) => {
    const nextBody = fromDoc.body;
    if (!nextBody || !document.body) {
      return false;
    }
    document.documentElement.lang = fromDoc.documentElement.lang || document.documentElement.lang;
    if (fromDoc.documentElement.className) {
      document.documentElement.className = fromDoc.documentElement.className;
    }
    document.title = fromDoc.title || document.title;
    copyMeta(fromDoc, "description", "name");
    copyMeta(fromDoc, "og:title", "property");
    copyMeta(fromDoc, "og:description", "property");
    copyMeta(fromDoc, "og:url", "property");
    copyCanonical(fromDoc);
    document.body.replaceWith(nextBody);
    return true;
  };

  const scrollToHash = (hash) => {
    if (!hash || hash === "#") {
      window.scrollTo(0, 0);
      return;
    }
    const id = decodeURIComponent(hash.replace(/^#/, ""));
    const el = document.getElementById(id);
    const content = el && (el.closest(".content-column") || document.querySelector(".content-column"));
    if (el && content) {
      content.scrollTop += el.getBoundingClientRect().top - content.getBoundingClientRect().top;
    } else if (el && el.scrollIntoView) {
      el.scrollIntoView({ block: "start" });
    } else {
      window.scrollTo(0, 0);
    }
  };

  const fullLoad = (href) => {
    window.location.assign(href);
  };

  const navigate = (href, mode) => {
    if (!href || navigating) {
      return;
    }
    let url;
    try {
      url = new URL(href, window.location.href);
    } catch (err) {
      fullLoad(href);
      return;
    }
    if (url.origin !== window.location.origin) {
      fullLoad(href);
      return;
    }
    const target = url.pathname + url.search + url.hash;
    if (window.__rocciNavSections) {
      window.__rocciNavSections.remember();
    }
    const samePath =
      url.pathname === window.location.pathname && url.search === window.location.search;
    if (samePath) {
      if (url.hash && url.hash !== window.location.hash) {
        if (mode === "push" && history.pushState) {
          history.pushState({ rocciGoto: true }, "", target);
        }
        scrollToHash(url.hash);
      }
      return;
    }
    const entry = findEntry(url.pathname);
    if (entryNeedsFullLoad(entry) || docNeedsFullLoad(document)) {
      fullLoad(target);
      return;
    }
    navigating = true;
    fetch(url.pathname + url.search, { credentials: "same-origin" })
      .then(function (res) {
        if (!res.ok) {
          throw new Error("missing");
        }
        return res.text();
      })
      .then(function (html) {
        const parsed = new DOMParser().parseFromString(html, "text/html");
        if (docNeedsFullLoad(parsed)) {
          fullLoad(target);
          return;
        }
        if (!applyDocument(parsed)) {
          fullLoad(target);
          return;
        }
        if (mode === "push" && history.pushState) {
          history.pushState({ rocciGoto: true }, "", target);
        } else if (mode === "replace" && history.replaceState) {
          history.replaceState({ rocciGoto: true }, "", target);
        }
        scrollToHash(url.hash);
        bindOpeners();
        if (window.__rocciNavSections) {
          window.__rocciNavSections.restore();
        }
      })
      .catch(function () {
        fullLoad(target);
      })
      .then(function () {
        navigating = false;
      });
  };

  const go = (url, mode) => {
    close();
    if (!url) {
      return;
    }
    navigate(url, mode || "push");
  };

  const open = () => {
    once("open", function () {
      const finder = window.__rocciPreviewNav && window.__rocciPreviewNav.find;
      if (finder && finder.isOpen && finder.isOpen()) {
        finder.close();
      }
      host.classList.add("open");
      lastFocused = document.activeElement;
      if (document.body) {
        previousBodyOverflow = document.body.style.overflow;
        document.body.style.overflow = "hidden";
      }
      queryInput.value = "";
      queryInput.focus();
      loadCatalog().then(filter);
    });
  };

  const close = () => {
    host.classList.remove("open");
    filtered = [];
    resultsEl.textContent = "";
    emptyEl.hidden = true;
    if (document.body) {
      document.body.style.overflow = previousBodyOverflow;
    }
    if (lastFocused && lastFocused.focus) {
      lastFocused.focus();
    }
    lastFocused = null;
  };

  const isMac = /Mac|iPhone|iPad/.test(navigator.platform || "");
  const isMod = (event) => {
    if (event.altKey) {
      return false;
    }
    if (isMac) {
      return event.metaKey && !event.ctrlKey;
    }
    return event.ctrlKey && !event.metaKey;
  };

  const shortcutLabel = isMac ? "⌘K" : "Ctrl+K";

  const bindOpeners = () => {
    const nodes = document.querySelectorAll("[data-rocci-goto-open]");
    for (let i = 0; i < nodes.length; i++) {
      const node = nodes[i];
      if (!node.getAttribute("aria-keyshortcuts")) {
        node.setAttribute("aria-keyshortcuts", isMac ? "Meta+K" : "Control+K");
      }
      if (!node.getAttribute("title")) {
        node.setAttribute("title", "Go to page (" + shortcutLabel + ")");
      }
    }
  };

  queryInput.addEventListener("input", filter);
  queryInput.addEventListener("keydown", function (event) {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      if (filtered.length) {
        selected = (selected + 1) % Math.min(filtered.length, 50);
        render();
      }
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      if (filtered.length) {
        const limit = Math.min(filtered.length, 50);
        selected = (selected - 1 + limit) % limit;
        render();
      }
    } else if (event.key === "Enter") {
      event.preventDefault();
      if (filtered[selected]) {
        go(filtered[selected].url, "push");
      }
    }
  });
  shadow.getElementById("backdrop").addEventListener("mousedown", function (event) {
    if (event.target && event.target.id === "backdrop") {
      close();
    }
  });

  const mount = () => {
    if (!host.isConnected && document.documentElement) {
      document.documentElement.appendChild(host);
    }
    bindOpeners();
  };
  mount();
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", mount);
  }

  const closest = (node, selector) => {
    while (node && node.nodeType !== 1) {
      node = node.parentNode;
    }
    return node && node.closest ? node.closest(selector) : null;
  };

  document.addEventListener(
    "click",
    function (event) {
      const opener = closest(event.target, "[data-rocci-goto-open]");
      if (opener) {
        event.preventDefault();
        open();
        return;
      }
      if (event.defaultPrevented || event.button !== 0) {
        return;
      }
      if (event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) {
        return;
      }
      const link = closest(event.target, "a[href]");
      if (!link || link.hasAttribute("download") || (link.getAttribute("target") || "") === "_blank") {
        return;
      }
      const href = link.getAttribute("href") || "";
      if (!href) {
        return;
      }
      let url;
      try {
        url = new URL(href, window.location.href);
      } catch (err) {
        return;
      }
      if (url.origin !== window.location.origin) {
        return;
      }
      const samePath =
        url.pathname === window.location.pathname && url.search === window.location.search;
      if (samePath && url.hash) {
        event.preventDefault();
        go(url.pathname + url.search + url.hash, "push");
        return;
      }
      event.preventDefault();
      go(url.pathname + url.search + url.hash, "push");
    },
    true
  );

  window.addEventListener(
    "keydown",
    function (event) {
      if (event.isComposing) {
        return;
      }
      if (event.key === "Escape" && isOpen()) {
        event.preventDefault();
        close();
        return;
      }
      if (!isMod(event)) {
        return;
      }
      const key = event.key.length === 1 ? event.key.toLowerCase() : event.key;
      if (key === "k" && !event.shiftKey) {
        event.preventDefault();
        const finder = window.__rocciPreviewNav && window.__rocciPreviewNav.find;
        if (finder && finder.isOpen && finder.isOpen()) {
          finder.close();
        }
        open();
      }
    },
    true
  );

  window.addEventListener("popstate", function () {
    navigate(window.location.pathname + window.location.search + window.location.hash, "pop");
  });

  const api = {
    open: open,
    close: close,
    isOpen: isOpen,
    loadCatalog: loadCatalog,
    navigate: navigate,
  };
  window.__rocciGoto = api;
  aliasPreview(api);
})();
