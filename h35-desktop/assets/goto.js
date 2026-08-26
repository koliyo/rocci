(function () {
  const aliasPreview = (api) => {
    if (window.__h35PreviewNav) {
      window.__h35PreviewNav.goto = api;
    }
  };

  const setupNavSections = () => {
    if (window.__h35NavSections) {
      return;
    }
    let transition = Promise.resolve();
    const currentLane = () => {
      const lane = document.querySelector(".lane-link.is-current");
      return lane && lane.textContent ? lane.textContent.trim() : "";
    };
    const storageKeyFor = (lane) =>
      lane ? "h35-nav-sections:" + lane : "h35-nav-sections";
    const storageKey = () => storageKeyFor(currentLane());
    const scrollStorageKey = "h35-nav-scroll-positions";
    const reducedMotion = () =>
      window.matchMedia && window.matchMedia("(prefers-reduced-motion: reduce)").matches;

    const forgetOtherLanes = () => {
      const lane = currentLane();
      if (lane !== "Docs") {
        try {
          sessionStorage.removeItem(storageKeyFor("Docs"));
        } catch (err) {}
      }
    };

    const readSectionState = () => {
      try {
        return JSON.parse(sessionStorage.getItem(storageKey()) || "{}");
      } catch (err) {
        return {};
      }
    };

    const sectionIsCurrent = (section) =>
      section && section.hasAttribute("data-h35-nav-current");

    const sectionsWithKey = () =>
      document.querySelectorAll("details[data-h35-nav-section]");

    const sectionOpenForKey = (key) => {
      const sections = sectionsWithKey();
      for (let i = 0; i < sections.length; i++) {
        const section = sections[i];
        if (section.getAttribute("data-h35-nav-section") !== key) {
          continue;
        }
        if (sectionIsCurrent(section) || section.open) {
          return true;
        }
      }
      return false;
    };

    const writeSectionState = (key, open) => {
      if (!key) {
        return;
      }
      const state = readSectionState();
      state[key] = !!open;
      try {
        sessionStorage.setItem(storageKey(), JSON.stringify(state));
      } catch (err) {}
    };

    const rememberSection = (section) => {
      const key = section.getAttribute("data-h35-nav-section");
      if (!key) {
        return;
      }
      writeSectionState(key, sectionIsCurrent(section) || sectionOpenForKey(key));
    };

    const rememberAllSections = () => {
      const state = readSectionState();
      const sections = sectionsWithKey();
      const seen = {};
      for (let i = 0; i < sections.length; i++) {
        const key = sections[i].getAttribute("data-h35-nav-section");
        if (!key || seen[key]) {
          continue;
        }
        seen[key] = true;
        if (sectionOpenForKey(key)) {
          state[key] = true;
        }
      }
      try {
        sessionStorage.setItem(storageKey(), JSON.stringify(state));
      } catch (err) {}
    };

    const restoreSections = () => {
      forgetOtherLanes();
      const state = readSectionState();
      const sections = sectionsWithKey();
      for (let i = 0; i < sections.length; i++) {
        const section = sections[i];
        const key = section.getAttribute("data-h35-nav-section");
        if (sectionIsCurrent(section) || (key && state[key])) {
          section.open = true;
          continue;
        }
        if (key && Object.prototype.hasOwnProperty.call(state, key)) {
          section.open = !!state[key];
        }
      }
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
      const sidebar = document.querySelector(".sidebar, .okf-chrome");
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
        const sidebar = document.querySelector(".sidebar, .okf-chrome");
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
        if (sectionIsCurrent(section) && section.open) {
          return;
        }
        const key = section.getAttribute("data-h35-nav-section");
        transition = transition.then(function () {
          const opening = !section.open;
          return finishFold(section, opening).then(function () {
            const copies = sectionsWithKey();
            for (let i = 0; i < copies.length; i++) {
              if (copies[i].getAttribute("data-h35-nav-section") === key) {
                copies[i].open = opening;
              }
            }
            writeSectionState(key, sectionIsCurrent(section) || opening);
          });
        });
      },
      true
    );
    restoreSections();
    restoreScrollPositions();
    window.addEventListener("pagehide", function () {
      rememberAllSections();
      rememberScrollPositions();
    });
    window.__h35NavSections = {
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

  if (window.__h35Goto) {
    aliasPreview(window.__h35Goto);
    return;
  }

  const CSS =
    ':host{all:initial;color-scheme:inherit;display:none;position:fixed;inset:0 var(--h35-chrome-right,0px) var(--h35-chrome-bottom,0px) 0;z-index:2147483646;font-family:system-ui,-apple-system,"Segoe UI",sans-serif;color:#d7dae0;color:light-dark(#18181b,#d7dae0)}:host(.open){display:block}.backdrop{box-sizing:border-box;display:flex;justify-content:center;align-items:flex-start;width:100%;height:100%;padding:12vh 16px 16px;background:rgba(0,0,0,.42);background:light-dark(rgba(24,24,27,.28),rgba(0,0,0,.42))}.palette{box-sizing:border-box;width:min(560px,100%);max-height:min(70vh,480px);display:flex;flex-direction:column;border:1px solid #3e4451;border-color:light-dark(#e4e4e7,#3e4451);border-radius:12px;background:#21252b;background:light-dark(#f7f7f8,#21252b);box-shadow:0 16px 48px rgba(0,0,0,.35);overflow:hidden}.query-row{display:flex;align-items:center;gap:10px;padding:0 14px;border-bottom:1px solid #3e4451;border-bottom-color:light-dark(#e4e4e7,#3e4451)}.query-row:focus-within{box-shadow:inset 0 -2px 0 #61afef;box-shadow:inset 0 -2px 0 light-dark(#2563eb,#61afef)}.query-icon{flex:none;color:#9da5b4;color:light-dark(#71717a,#9da5b4)}input{box-sizing:border-box;flex:1;min-width:0;height:48px;margin:0;padding:0;border:0;background:transparent;color:inherit;font:inherit;font-size:15px;outline:none;box-shadow:none;appearance:none;-webkit-appearance:none}input:focus,input:focus-visible{outline:none;box-shadow:none}input::placeholder{color:#9da5b4;color:light-dark(#71717a,#9da5b4);opacity:1}input::-webkit-search-decoration,input::-webkit-search-cancel-button,input::-webkit-search-results-button,input::-webkit-search-results-decoration{-webkit-appearance:none}#results{margin:0;padding:6px;list-style:none;overflow-y:auto;flex:1 1 auto}.item{display:flex;flex-direction:column;gap:2px;min-height:44px;box-sizing:border-box;padding:8px 10px;border-radius:8px;cursor:pointer}.item.is-selected,.item:hover{background:rgba(97,175,239,.16);background:light-dark(rgba(59,130,246,.12),rgba(97,175,239,.16))}.title{font-size:13.5px;font-weight:600}.path{color:#9da5b4;color:light-dark(#71717a,#9da5b4);font-size:12px;font-family:ui-monospace,Menlo,monospace;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.empty{margin:0;padding:18px 14px;color:#9da5b4;color:light-dark(#71717a,#9da5b4);font-size:13px}mark{background:transparent;color:#61afef;color:light-dark(#2563eb,#61afef);font-weight:700;padding:0}@media(max-width:480px){.backdrop{padding:8vh 8px 8px}.palette{max-height:84vh;border-radius:10px}}';
  const HTML =
    '<div class="backdrop" id="backdrop"><div class="palette" role="dialog" aria-modal="true" aria-label="Go to page"><div class="query-row"><svg class="query-icon" viewBox="0 0 16 16" width="16" height="16" aria-hidden="true" focusable="false"><circle cx="6.5" cy="6.5" r="4.25" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M9.75 9.75L13.25 13.25" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg><input type="search" id="query" placeholder="Go to page" autocomplete="off" autocorrect="off" spellcheck="false" aria-label="Go to page" aria-controls="results"/></div><ul id="results" role="listbox" aria-label="Matching documents"></ul><p id="empty" class="empty" hidden>No matching documents</p></div></div>';

  const host = document.createElement("h35-goto");
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
      [entry.collection, 0.9],
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

  const RETIRED_ROUTES = {
    "/docs/getting-started/quickstart/": "/docs/five-minutes/",
    "/docs/getting-started/quickstart": "/docs/five-minutes/",
    "/getting-started/quickstart/": "/docs/five-minutes/",
    "/getting-started/quickstart": "/docs/five-minutes/",
    "/docs/start/five-minutes/": "/docs/five-minutes/",
    "/docs/start/install/": "/docs/install/",
    "/docs/start/overview/": "/docs/",
    "/getting-started/installation/": "/docs/install/",
    "/getting-started/overview/": "/docs/",
  };

  const setupNotFoundRecovery = () => {
    const body = document.body;
    if (!body || body.getAttribute("data-h35-not-found") !== "true") {
      return;
    }
    const hint = document.getElementById("h35-not-found-hint");
    if (!hint) {
      return;
    }
    const pathname = window.location.pathname;
    const target =
      RETIRED_ROUTES[pathname] ||
      RETIRED_ROUTES[pathname.endsWith("/") ? pathname : pathname + "/"];
    if (!target) {
      return;
    }
    hint.hidden = false;
    hint.innerHTML =
      "Did you mean <a class=\"rd-link\" href=\"" +
      escapeText(target) +
      "\"><code class=\"rd-code\">" +
      escapeText(target) +
      "</code></a>?";
  };

  setupNotFoundRecovery();

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

  const isExampleSource = (url) => /\/examples\/[^/]+\/source(?:\/|$)/.test(String(url || ""));

  const normalizePages = (rows) => {
    const out = [];
    for (let i = 0; i < rows.length; i++) {
      const row = rows[i] || {};
      const url = row.route || row.url || "";
      if (!url || isExampleSource(url)) {
        continue;
      }
      out.push({
        title: row.title || url,
        url: url,
        path: row.path || "",
        description: row.description || "",
        collection: row.collection || "",
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
      const url = "/" + id.replace(/^\/+|\/+$/g, "") + "/";
      if (isExampleSource(url) || isExampleSource(id)) {
        continue;
      }
      out.push({
        title: title || id,
        url: url,
        path: row.path || "",
        description: description,
        collection: row.collection || "",
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
      if (seen[key] || isExampleSource(key)) {
        continue;
      }
      seen[key] = true;
      const title = (nodes[i].textContent || "").replace(/\s+/g, " ").replace(/^\s+|\s+$/g, "");
      out.push({
        title: title || key,
        url: key,
        path: key,
        description: "",
        collection: "",
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
    if (catalogPromise) {
      return catalogPromise;
    }
    const remember = (rows) => {
      catalog = rows || [];
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
        escapeText(
          entry.collection
            ? entry.collection + " · " + (entry.path || entry.url)
            : entry.path || entry.url
        ) +
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
      return (
        b.score - a.score ||
        a.entry.title.localeCompare(b.entry.title) ||
        String(a.entry.path || "").localeCompare(String(b.entry.path || ""))
      );
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
      /\/__h35\/reload\.js/.test(src)
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

  const NAV_KEEP = "#okf-nav, #site-nav";
  const MAIN_SWAP = "#okf-main, #main-content";
  const TOC_SWAP = "#okf-toc, .layout-navigated > .outline, .site-grid > .outline";
  const TOC_SHELL = ".rd-shell, .layout-navigated, .site-grid";
  const NAV_SYNC = ["#okf-nav", "#site-nav", ".mobile-panel"];

  const attrEscape = (value) =>
    String(value).replace(/\\/g, "\\\\").replace(/"/g, '\\"');

  const copyHead = (fromDoc) => {
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
  };

  const syncKeptNav = (nextNav, keepNav) => {
    keepNav.querySelectorAll("[data-h35-nav-current]").forEach((el) => {
      el.removeAttribute("data-h35-nav-current");
    });
    keepNav.querySelectorAll(".is-current").forEach((el) => {
      el.classList.remove("is-current");
    });
    keepNav.querySelectorAll('[aria-current="page"]').forEach((el) => {
      el.setAttribute("aria-current", "false");
    });
    nextNav.querySelectorAll("a.is-current, a[aria-current='page']").forEach((link) => {
      const href = link.getAttribute("href");
      if (!href) {
        return;
      }
      keepNav.querySelectorAll('a[href="' + attrEscape(href) + '"]').forEach((keep) => {
        keep.classList.add("is-current");
        keep.setAttribute("aria-current", "page");
      });
    });
    nextNav.querySelectorAll("[data-h35-nav-current]").forEach((section) => {
      const key = section.getAttribute("data-h35-nav-section");
      if (!key) {
        return;
      }
      keepNav
        .querySelectorAll('details[data-h35-nav-section="' + attrEscape(key) + '"]')
        .forEach((keep) => {
          keep.setAttribute("data-h35-nav-current", "");
          keep.open = true;
        });
    });
  };

  const syncKeptChrome = (fromDoc) => {
    NAV_SYNC.forEach((sel) => {
      const keep = document.querySelector(sel);
      const next = fromDoc.querySelector(sel);
      if (keep && next) {
        syncKeptNav(next, keep);
      }
    });
    const currentHrefs = {};
    fromDoc.querySelectorAll(".lane-link.is-current").forEach((el) => {
      const href = el.getAttribute("href");
      if (href) {
        currentHrefs[href] = true;
      }
    });
    document.querySelectorAll(".lane-link").forEach((el) => {
      const on = !!currentHrefs[el.getAttribute("href") || ""];
      el.classList.toggle("is-current", on);
      el.setAttribute("aria-current", on ? "true" : "false");
    });
  };

  const applyDocument = (fromDoc) => {
    const nextBody = fromDoc.body;
    if (!nextBody || !document.body) {
      return { ok: false, keepNav: false };
    }
    copyHead(fromDoc);
    const keepNav = document.querySelector(NAV_KEEP);
    const nextNav = fromDoc.querySelector(NAV_KEEP);
    const keepMain = document.querySelector(MAIN_SWAP);
    const nextMain = fromDoc.querySelector(MAIN_SWAP);
    if (keepNav && nextNav && keepMain && nextMain) {
      keepMain.replaceWith(nextMain);
      const keepToc = document.querySelector(TOC_SWAP);
      const nextToc = fromDoc.querySelector(TOC_SWAP);
      if (keepToc && nextToc) {
        keepToc.replaceWith(nextToc);
      } else if (keepToc && !nextToc) {
        keepToc.remove();
      } else if (!keepToc && nextToc) {
        const shell = document.querySelector(TOC_SHELL);
        if (!shell) {
          document.body.replaceWith(nextBody);
          return { ok: true, keepNav: false };
        }
        shell.appendChild(nextToc);
      }
      syncKeptChrome(fromDoc);
      return { ok: true, keepNav: true };
    }
    document.body.replaceWith(nextBody);
    return { ok: true, keepNav: false };
  };

  const hashTarget = (hash) => {
    if (!hash || hash === "#") {
      return null;
    }
    const id = decodeURIComponent(hash.replace(/^#/, ""));
    const escaped = id.replace(/\\/g, "\\\\").replace(/"/g, '\\"');
    return (
      document.querySelector('.rd-source-line[id="' + escaped + '"]') ||
      document.getElementById(id)
    );
  };

  const isScrollableY = (node) => {
    if (!node || node === document.body || node === document.documentElement) {
      return false;
    }
    const style = window.getComputedStyle(node);
    const overflowY = style.overflowY;
    return (
      (overflowY === "auto" || overflowY === "scroll" || overflowY === "overlay") &&
      node.scrollHeight > node.clientHeight + 1
    );
  };

  const scrollableAncestor = (el) => {
    let node = el.parentElement;
    while (node && node !== document.body && node !== document.documentElement) {
      if (isScrollableY(node)) {
        return node;
      }
      node = node.parentElement;
    }
    return document.scrollingElement || document.documentElement;
  };

  const resetDocumentScroll = () => {
    window.scrollTo(0, 0);
    if (document.scrollingElement) {
      document.scrollingElement.scrollTop = 0;
    }
    document.documentElement.scrollTop = 0;
    document.body.scrollTop = 0;
    document
      .querySelectorAll(".content-column, #okf-main, #main-content, article.article, article.article")
      .forEach((el) => {
        el.scrollTop = 0;
        const scroller = scrollableAncestor(el);
        if (
          scroller &&
          scroller !== document.scrollingElement &&
          scroller !== document.documentElement &&
          scroller !== document.body
        ) {
          scroller.scrollTop = 0;
        }
      });
  };

  const scrollToHash = (hash) => {
    if (!hash || hash === "#") {
      resetDocumentScroll();
      return;
    }
    const run = () => {
      const el = hashTarget(hash);
      if (!el) {
        return;
      }
      const scroller = scrollableAncestor(el);
      const margin = parseFloat(window.getComputedStyle(el).scrollMarginTop) || 0;
      if (scroller === document.scrollingElement || scroller === document.documentElement || scroller === document.body) {
        if (el.scrollIntoView) {
          el.scrollIntoView({ block: "start", inline: "nearest" });
        } else {
          const y =
            el.getBoundingClientRect().top +
            (window.pageYOffset || document.documentElement.scrollTop || 0) -
            margin;
          window.scrollTo(0, Math.max(0, y));
        }
        return;
      }
      scroller.scrollTop += el.getBoundingClientRect().top - scroller.getBoundingClientRect().top - margin;
    };
    run();
    if (window.requestAnimationFrame) {
      window.requestAnimationFrame(run);
    }
  };

  const fullLoad = (href) => {
    window.location.assign(href);
  };

  const displayPath = (href) => {
    try {
      const parsed = new URL(href, window.location.href);
      const path = parsed.pathname.replace(/\/+$/, "") || "/";
      return path + parsed.search;
    } catch (err) {
      return href;
    }
  };

  const reportLocation = () => {
    const href = window.location.href;
    if (window.ipc && window.ipc.postMessage) {
      window.ipc.postMessage("location:" + href);
    }
    if (window.__h35PreviewNav && window.__h35PreviewNav.update) {
      window.__h35PreviewNav.update({
        title: document.title,
        path: displayPath(href),
      });
    }
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
    if (window.__h35NavSections) {
      window.__h35NavSections.remember();
    }
    const samePath =
      url.pathname === window.location.pathname && url.search === window.location.search;
    if (samePath) {
      if (url.hash && url.hash !== window.location.hash) {
        if (mode === "push" && history.pushState) {
          history.pushState({ h35Goto: true }, "", target);
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
        const applied = applyDocument(parsed);
        if (!applied.ok) {
          fullLoad(target);
          return;
        }
        if (mode === "push" && history.pushState) {
          history.pushState({ h35Goto: true }, "", target);
        } else if (mode === "replace" && history.replaceState) {
          history.replaceState({ h35Goto: true }, "", target);
        }
        reportLocation();
        scrollToHash(url.hash);
        bindOpeners();
        if (!applied.keepNav && window.__h35NavSections) {
          window.__h35NavSections.restore();
        }
        if (window.__h35OnNavigate) {
          window.__h35OnNavigate(url.pathname, document.title);
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
      const finder = window.__h35PreviewNav && window.__h35PreviewNav.find;
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
      catalog = null;
      catalogPromise = null;
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
    const nodes = document.querySelectorAll("[data-h35-goto-open]");
    for (let i = 0; i < nodes.length; i++) {
      const node = nodes[i];
      if (!node.getAttribute("aria-keyshortcuts")) {
        node.setAttribute("aria-keyshortcuts", isMac ? "Meta+K" : "Control+K");
      }
      if (!node.getAttribute("title")) {
        node.setAttribute("title", "Go to page (" + shortcutLabel + ")");
      }
      const shortcut = node.querySelector("[data-h35-goto-shortcut]");
      if (shortcut) {
        shortcut.textContent = shortcutLabel;
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
      const opener = closest(event.target, "[data-h35-goto-open]");
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
        const finder = window.__h35PreviewNav && window.__h35PreviewNav.find;
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
  window.__h35Goto = api;
  aliasPreview(api);
})();
