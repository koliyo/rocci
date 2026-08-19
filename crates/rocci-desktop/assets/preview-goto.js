(function () {
  if (!window.__rocciPreviewNav || window.__rocciPreviewNav.goto) {
    return;
  }
  const CACHE_KEY = "rocci-goto-catalog";
  const host = document.createElement("rocci-preview-goto");
  const shadow = host.attachShadow({ mode: "open" });
  const sheet = document.createElement("style");
  sheet.textContent = __ROCCI_PREVIEW_GOTO_CSS__;
  const tpl = document.createElement("template");
  tpl.innerHTML = __ROCCI_PREVIEW_GOTO_HTML__;
  shadow.append(sheet, tpl.content);
  const queryInput = shadow.getElementById("query");
  const resultsEl = shadow.getElementById("results");
  const emptyEl = shadow.getElementById("empty");
  const hostSheet = document.createElement("style");
  hostSheet.textContent =
    "rocci-preview-goto{display:none;position:fixed;inset:0;z-index:2147483646}rocci-preview-goto.open{display:block}";
  let catalog = null;
  let catalogPromise = null;
  let filtered = [];
  let selected = 0;
  let lastAction = { name: "", at: 0 };

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
      });
    }
    return out;
  };

  const scrapeNav = () => {
    const seen = {};
    const out = [];
    const nodes = document.querySelectorAll("aside a[href], a.nav-link[href], .lanes a[href], a.lane-link[href]");
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
        if (!(Array.isArray(data) && data.length && data[0] && data[0].route != null)) {
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
        go(entry.url);
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

  const go = (url) => {
    close();
    if (!url) {
      return;
    }
    window.location.assign(url);
  };

  const open = () => {
    once("open", function () {
      if (window.__rocciPreviewNav.find && window.__rocciPreviewNav.find.isOpen()) {
        window.__rocciPreviewNav.find.close();
      }
      host.classList.add("open");
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
        go(filtered[selected].url);
      }
    }
  });
  shadow.getElementById("backdrop").addEventListener("mousedown", function (event) {
    if (event.target && event.target.id === "backdrop") {
      close();
    }
  });

  const mount = () => {
    if (hostSheet.isConnected === false && document.documentElement) {
      document.documentElement.appendChild(hostSheet);
    }
    if (!host.isConnected && document.documentElement) {
      document.documentElement.appendChild(host);
    }
  };
  mount();
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", mount);
  }

  window.__rocciPreviewNav.goto = {
    open: open,
    close: close,
    isOpen: isOpen,
  };
})();
