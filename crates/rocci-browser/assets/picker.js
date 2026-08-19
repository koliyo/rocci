(function () {
  if (window.__rocciBrowser) {
    return;
  }
  const CSS = __ROCCI_BROWSER_PICKER_CSS__;
  const HTML = __ROCCI_BROWSER_PICKER_HTML__;
  const host = document.createElement("rocci-browser-picker");
  const shadow = host.attachShadow({ mode: "open" });
  const sheet = document.createElement("style");
  sheet.textContent = CSS;
  const tpl = document.createElement("template");
  tpl.innerHTML = HTML;
  shadow.append(sheet, tpl.content);
  const queryInput = shadow.getElementById("query");
  const resultsEl = shadow.getElementById("results");
  const emptyEl = shadow.getElementById("empty");
  const hostSheet = document.createElement("style");
  hostSheet.textContent =
    "rocci-browser-picker{display:none;position:fixed;top:0;left:0;right:var(--rocci-chrome-right,0px);bottom:var(--rocci-chrome-bottom,0px);z-index:2147483646}rocci-browser-picker.open{display:block}";

  let targets = [];
  let documents = [];
  let stage = "targets";
  let selected = 0;
  let targetQuery = "";
  let highlighted = null;
  let emptyReason = "";

  const send = (payload) => {
    if (window.ipc && window.ipc.postMessage) {
      window.ipc.postMessage(payload);
    }
  };

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

  const visible = () => {
    const query = queryInput.value;
    if (stage === "targets") {
      return targets
        .map((row) => ({
          row: row,
          score: scoreEntry(query, {
            title: row.id,
            path: row.path,
            description: row.label,
            url: "",
          }),
        }))
        .filter((item) => item.score >= 0)
        .sort((a, b) => b.score - a.score || a.row.adapterId.localeCompare(b.row.adapterId));
    }
    return documents
      .map((row) => ({
        row: row,
        score: scoreEntry(query, {
          title: row.title,
          path: row.path,
          description: "",
          url: row.route || "",
        }),
      }))
      .filter((item) => item.score >= 0)
      .sort((a, b) => b.score - a.score || a.row.title.localeCompare(b.row.title));
  };

  const render = () => {
    const rows = visible();
    if (selected >= rows.length) {
      selected = 0;
    }
    resultsEl.innerHTML = "";
    rows.forEach((item, index) => {
      const li = document.createElement("li");
      li.className = "item" + (index === selected ? " is-selected" : "");
      const title = document.createElement("span");
      title.className = "title";
      title.textContent =
        stage === "targets"
          ? item.row.id + " [" + item.row.adapterId + "] " + (item.row.label || "")
          : item.row.title;
      const path = document.createElement("span");
      path.className = "path";
      path.textContent = item.row.path;
      li.append(title, path);
      li.addEventListener("mousedown", (event) => {
        event.preventDefault();
        selected = index;
        enter();
      });
      resultsEl.append(li);
    });
    const reason =
      emptyReason ||
      (rows.length ? "" : stage === "targets" ? "No matching targets" : "No matching documents");
    emptyEl.hidden = !reason || rows.length > 0;
    emptyEl.textContent = reason;
  };

  const isOpen = () => host.classList.contains("open");

  const open = () => {
    host.classList.add("open");
    send("browser:catalog");
    selected = 0;
    render();
    queryInput.focus();
    queryInput.select();
  };

  const close = () => {
    host.classList.remove("open");
    stage = "targets";
    documents = [];
    highlighted = null;
    emptyReason = "";
  };

  const enter = () => {
    const rows = visible();
    const item = rows[selected];
    if (!item) {
      return;
    }
    if (stage === "targets") {
      send(
        "browser:open:" +
          JSON.stringify({
            adapterId: item.row.adapterId,
            root: item.row.path,
          })
      );
      close();
      return;
    }
    send(
      "browser:open:" +
        JSON.stringify({
          adapterId: highlighted.adapterId,
          root: highlighted.path,
          document: item.row.id,
        })
    );
    close();
  };

  const tab = () => {
    if (stage !== "targets") {
      return;
    }
    const rows = visible();
    const item = rows[selected];
    if (!item) {
      return;
    }
    highlighted = item.row;
    targetQuery = queryInput.value;
    send(
      "browser:list:" +
        JSON.stringify({ adapterId: item.row.adapterId, root: item.row.path })
    );
  };

  const back = () => {
    if (stage !== "documents") {
      return;
    }
    stage = "targets";
    documents = [];
    queryInput.value = targetQuery;
    selected = 0;
    highlighted = null;
    emptyReason = "";
    render();
  };

  queryInput.addEventListener("keydown", (event) => {
    if (event.key === "Tab") {
      event.preventDefault();
      if (event.shiftKey) {
        back();
      } else {
        tab();
      }
      return;
    }
    if (event.key === "Enter") {
      event.preventDefault();
      enter();
      return;
    }
    if (event.key === "Escape") {
      event.preventDefault();
      if (stage === "documents") {
        back();
      } else {
        close();
      }
      return;
    }
    if (event.key === "ArrowDown") {
      event.preventDefault();
      const rows = visible();
      if (rows.length) {
        selected = (selected + 1) % rows.length;
        render();
      }
    }
    if (event.key === "ArrowUp") {
      event.preventDefault();
      const rows = visible();
      if (rows.length) {
        selected = (selected - 1 + rows.length) % rows.length;
        render();
      }
    }
  });
  queryInput.addEventListener("input", () => {
    selected = 0;
    emptyReason = "";
    render();
  });

  document.addEventListener("keydown", (event) => {
    if (event.key === "p" && (event.metaKey || event.ctrlKey) && !event.shiftKey && !event.altKey) {
      event.preventDefault();
      if (isOpen()) {
        close();
      } else {
        open();
      }
    }
  });

  window.__rocciBrowser = {
    open: open,
    close: close,
    setTargets(rows) {
      targets = Array.isArray(rows) ? rows : [];
      if (isOpen() && stage === "targets") {
        render();
      }
    },
    setDocuments(rows, reason) {
      if (!rows || !rows.length) {
        emptyReason = reason || "adapter returned no documents";
        stage = "targets";
        render();
        return;
      }
      documents = rows;
      stage = "documents";
      queryInput.value = "";
      selected = 0;
      emptyReason = "";
      render();
    },
  };

  const mount = () => {
    if (!host.isConnected && document.documentElement) {
      document.documentElement.prepend(host);
    }
    if (!hostSheet.isConnected && document.documentElement) {
      document.documentElement.append(hostSheet);
    }
    if (
      document.documentElement &&
      document.documentElement.hasAttribute("data-rocci-browser-launcher")
    ) {
      open();
    }
  };
  mount();
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", mount);
  }
})();
