(function () {
  if (window.__rocciPreviewKeys) {
    return;
  }
  window.__rocciPreviewKeys = true;

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

  const find = () => window.__rocciPreviewNav && window.__rocciPreviewNav.find;
  const go = () => window.__rocciPreviewNav && window.__rocciPreviewNav.goto;

  window.addEventListener(
    "keydown",
    function (event) {
      if (event.isComposing) {
        return;
      }
      const finder = find();
      const goto = go();
      if (event.key === "Escape") {
        if (goto && goto.isOpen()) {
          event.preventDefault();
          goto.close();
          return;
        }
        if (finder && finder.isOpen()) {
          event.preventDefault();
          finder.close();
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
      }
    },
    true
  );
})();
