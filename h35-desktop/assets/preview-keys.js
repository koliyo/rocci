(function () {
  if (window.__h35PreviewKeys) {
    return;
  }
  window.__h35PreviewKeys = true;

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

  const find = () => window.__h35PreviewNav && window.__h35PreviewNav.find;
  const go = () => window.__h35PreviewNav && window.__h35PreviewNav.goto;

  const deepestActive = (el) => {
    while (el && el.shadowRoot && el.shadowRoot.activeElement) {
      el = el.shadowRoot.activeElement;
    }
    return el;
  };

  const selectEditable = (el) => {
    if (!el) {
      return false;
    }
    if (typeof el.select === "function") {
      el.select();
      return true;
    }
    if (el.isContentEditable) {
      const range = document.createRange();
      range.selectNodeContents(el);
      const sel = window.getSelection();
      if (!sel) {
        return false;
      }
      sel.removeAllRanges();
      sel.addRange(range);
      return true;
    }
    return false;
  };

  const selectAll = () => {
    const active = deepestActive(document.activeElement);
    const tag = active && active.tagName;
    if (tag === "INPUT" || tag === "TEXTAREA" || (active && active.isContentEditable)) {
      return selectEditable(active);
    }
    const root = document.querySelector(
      "[data-h35-select-root], article.article, article.article"
    );
    if (!root) {
      return false;
    }
    const sel = window.getSelection();
    if (!sel) {
      return false;
    }
    const range = document.createRange();
    range.selectNodeContents(root);
    sel.removeAllRanges();
    sel.addRange(range);
    return true;
  };

  if (window.__h35PreviewNav) {
    window.__h35PreviewNav.selectAll = selectAll;
  }

  window.addEventListener(
    "keydown",
    function (event) {
      if (event.isComposing) {
        return;
      }
      const finder = find();
      const goto = go();
      if (event.key === "Escape") {
        const more = window.__h35PreviewNav && window.__h35PreviewNav.closeMore;
        if (goto && goto.isOpen()) {
          event.preventDefault();
          goto.close();
          return;
        }
        if (finder && finder.isOpen()) {
          event.preventDefault();
          finder.close();
          return;
        }
        if (more) {
          more();
        }
        return;
      }
      if (!isMod(event)) {
        return;
      }
      const key = event.key.length === 1 ? event.key.toLowerCase() : event.key;
      if (key === "f" && !event.shiftKey) {
        event.preventDefault();
        if (goto && goto.isOpen()) {
          goto.close();
        }
        if (finder) {
          finder.open();
        }
      } else if (key === "e" && !event.shiftKey) {
        event.preventDefault();
        if (finder) {
          finder.useSelection();
        }
      } else if (key === "g") {
        event.preventDefault();
        if (finder) {
          if (event.shiftKey) {
            finder.prev();
          } else {
            finder.next();
          }
        }
      } else if (key === "k" && !event.shiftKey) {
        event.preventDefault();
        if (finder && finder.isOpen()) {
          finder.close();
        }
        if (goto) {
          goto.open();
        }
      } else if (key === "a" && !event.shiftKey) {
        if (selectAll()) {
          event.preventDefault();
        }
      }
    },
    true
  );
})();
