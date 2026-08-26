(function () {
  if (!window.__h35PreviewNav || window.__h35PreviewNav.find) {
    return;
  }
  const SKIP_TAGS = {
    SCRIPT: true,
    STYLE: true,
    NOSCRIPT: true,
    TEXTAREA: true,
    INPUT: true,
    SELECT: true,
    OPTION: true,
  };
  const SKIP_HOSTS = {
    "H35-PREVIEW-NAV": true,
    "H35-PREVIEW-DEV": true,
    "H35-PREVIEW-FIND": true,
    "H35-PREVIEW-GOTO": true,
  };
  const canHighlight =
    typeof CSS !== "undefined" && CSS.highlights && typeof Highlight === "function";
  const host = document.createElement("h35-preview-find");
  const shadow = host.attachShadow({ mode: "open" });
  const sheet = document.createElement("style");
  sheet.textContent = __H35_PREVIEW_FIND_CSS__;
  const tpl = document.createElement("template");
  tpl.innerHTML = __H35_PREVIEW_FIND_HTML__;
  shadow.append(sheet, tpl.content);
  const queryInput = shadow.getElementById("query");
  const countEl = shadow.getElementById("count");
  const prevBtn = shadow.getElementById("prev");
  const nextBtn = shadow.getElementById("next");
  const highlightSheet = document.createElement("style");
  highlightSheet.textContent =
    "::highlight(h35-find){background-color:#ffe08a;color:inherit}::highlight(h35-find-current){background-color:#f5a623;color:#18181b}@media (prefers-color-scheme:dark){::highlight(h35-find){background-color:#8a6d1b;color:inherit}::highlight(h35-find-current){background-color:#d19a66;color:#1e1e1e}}mark.h35-find-mark{background:#ffe08a;color:inherit;padding:0}mark.h35-find-mark.current{background:#f5a623;color:#18181b}h35-preview-find{display:none;position:fixed;top:var(--h35-chrome-top,48px);right:calc(var(--h35-chrome-right, 0px) + 12px);z-index:2147483646}h35-preview-find.open{display:block}";
  let query = "";
  let matches = [];
  let index = 0;
  let shown = false;
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

  const acceptNode = (node) => {
    if (node.nodeType === 1) {
      if (SKIP_HOSTS[node.tagName] || SKIP_TAGS[node.tagName]) {
        return NodeFilter.FILTER_REJECT;
      }
      return NodeFilter.FILTER_SKIP;
    }
    if (!node.nodeValue) {
      return NodeFilter.FILTER_REJECT;
    }
    return NodeFilter.FILTER_ACCEPT;
  };

  const textNodes = () => {
    const root = document.body || document.documentElement;
    if (!root) {
      return [];
    }
    const nodes = [];
    const walker = document.createTreeWalker(root, NodeFilter.SHOW_ELEMENT | NodeFilter.SHOW_TEXT, {
      acceptNode: acceptNode,
    });
    let node = walker.nextNode();
    while (node) {
      nodes.push(node);
      node = walker.nextNode();
    }
    return nodes;
  };

  const clearMarks = () => {
    const marks = document.querySelectorAll("mark.h35-find-mark");
    for (let i = 0; i < marks.length; i++) {
      const mark = marks[i];
      const parent = mark.parentNode;
      if (!parent) {
        continue;
      }
      while (mark.firstChild) {
        parent.insertBefore(mark.firstChild, mark);
      }
      parent.removeChild(mark);
      parent.normalize();
    }
  };

  const clearHighlights = () => {
    if (canHighlight) {
      CSS.highlights.delete("h35-find");
      CSS.highlights.delete("h35-find-current");
    }
    clearMarks();
  };

  const collectMatches = (needle) => {
    const found = [];
    if (!needle) {
      return found;
    }
    const q = needle.toLowerCase();
    const qLen = needle.length;
    const nodes = textNodes();
    for (let n = 0; n < nodes.length; n++) {
      const node = nodes[n];
      const text = node.nodeValue;
      const lower = text.toLowerCase();
      let from = 0;
      while (from + qLen <= lower.length) {
        const at = lower.indexOf(q, from);
        if (at < 0) {
          break;
        }
        const range = document.createRange();
        range.setStart(node, at);
        range.setEnd(node, at + qLen);
        found.push(range);
        from = at + Math.max(qLen, 1);
      }
    }
    return found;
  };

  const reveal = () => {
    const total = matches.length;
    prevBtn.disabled = total === 0;
    nextBtn.disabled = total === 0;
    if (total === 0) {
      shown = false;
      countEl.textContent = query ? "0 of 0" : "";
      clearHighlights();
      return;
    }
    if (index < 0) {
      index = total - 1;
    } else if (index >= total) {
      index = 0;
    }
    shown = true;
    countEl.textContent = index + 1 + " of " + total;
    if (canHighlight) {
      const current = matches[index];
      const rest = [];
      for (let i = 0; i < matches.length; i++) {
        if (i !== index) {
          rest.push(matches[i]);
        }
      }
      CSS.highlights.delete("h35-find");
      CSS.highlights.delete("h35-find-current");
      if (rest.length) {
        CSS.highlights.set("h35-find", new Highlight(...rest));
      }
      CSS.highlights.set("h35-find-current", new Highlight(current));
    } else {
      clearMarks();
      for (let i = matches.length - 1; i >= 0; i--) {
        const mark = document.createElement("mark");
        mark.className = i === index ? "h35-find-mark current" : "h35-find-mark";
        try {
          matches[i].surroundContents(mark);
        } catch (err) {}
      }
      const marks = document.querySelectorAll("mark.h35-find-mark");
      const nextMatches = [];
      for (let i = 0; i < marks.length; i++) {
        const range = document.createRange();
        range.selectNodeContents(marks[i]);
        nextMatches.push(range);
      }
      if (nextMatches.length) {
        matches = nextMatches;
      }
    }
    const current = matches[index];
    if (!current) {
      return;
    }
    const node = current.startContainer;
    const el = node.nodeType === 1 ? node : node.parentElement;
    if (el && el.scrollIntoView) {
      el.scrollIntoView({ block: "center", inline: "nearest" });
    }
  };

  const apply = (needle, shouldReveal) => {
    query = needle;
    if (queryInput.value !== needle) {
      queryInput.value = needle;
    }
    clearHighlights();
    matches = collectMatches(needle);
    index = 0;
    shown = false;
    if (shouldReveal) {
      reveal();
    }
  };

  const selectionText = () => {
    const sel = window.getSelection && window.getSelection();
    return sel ? String(sel).replace(/^\s+|\s+$/g, "") : "";
  };

  const open = () => {
    once("open", function () {
      const selected = selectionText();
      if (selected) {
        query = selected;
      }
      host.classList.add("open");
      queryInput.value = query;
      apply(query, true);
      queryInput.focus();
      queryInput.select();
    });
  };

  const close = () => {
    host.classList.remove("open");
    clearHighlights();
    matches = [];
    shown = false;
    countEl.textContent = "";
  };

  const next = () => {
    once("next", function () {
      if (!query && selectionText()) {
        apply(selectionText(), false);
      }
      if (!matches.length && query) {
        apply(query, false);
      }
      if (!matches.length) {
        return;
      }
      if (shown) {
        index = (index + 1) % matches.length;
      }
      reveal();
    });
  };

  const prev = () => {
    once("prev", function () {
      if (!matches.length && query) {
        apply(query, false);
      }
      if (!matches.length) {
        return;
      }
      if (shown) {
        index = (index - 1 + matches.length) % matches.length;
      } else {
        index = matches.length - 1;
      }
      reveal();
    });
  };

  const useSelection = () => {
    once("useSelection", function () {
      const selected = selectionText();
      if (!selected) {
        return;
      }
      apply(selected, isOpen());
    });
  };

  queryInput.addEventListener("input", function () {
    apply(queryInput.value, true);
  });
  queryInput.addEventListener("keydown", function (event) {
    if (event.key === "Enter") {
      event.preventDefault();
      if (event.shiftKey) {
        prev();
      } else {
        next();
      }
    }
  });
  prevBtn.addEventListener("click", prev);
  nextBtn.addEventListener("click", next);
  shadow.getElementById("close").addEventListener("click", close);

  const mount = () => {
    if (highlightSheet.isConnected === false && document.documentElement) {
      document.documentElement.appendChild(highlightSheet);
    }
    if (!host.isConnected && document.documentElement) {
      document.documentElement.appendChild(host);
    }
  };
  mount();
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", mount);
  }

  window.__h35PreviewNav.find = {
    open: open,
    close: close,
    next: next,
    prev: prev,
    useSelection: useSelection,
    isOpen: isOpen,
  };
})();
