(function () {
  if (window.__rocciCopy) {
    return;
  }

  const ICON_COPY =
    '<svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true"><rect x="4.5" y="2.5" width="8" height="10" rx="1" fill="none" stroke="currentColor" stroke-width="1.25"/><path fill="none" stroke="currentColor" stroke-width="1.25" d="M6 2.5h6.5v8"/></svg>';
  const ICON_CHECK =
    '<svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true"><path fill="none" stroke="currentColor" stroke-width="1.25" stroke-linecap="round" stroke-linejoin="round" d="M3.5 8.5 6.5 11.5 12.5 4.5"/></svg>';

  function blockText(pre) {
    const code = pre.querySelector("code");
    const text = code ? code.innerText : pre.innerText;
    return text.replace(/\n$/, "");
  }

  function showCopied(button) {
    const label = button.getAttribute("aria-label") || "Copy code";
    button.innerHTML = ICON_CHECK;
    button.setAttribute("aria-label", "Copied");
    button.classList.add("is-copied");
    window.setTimeout(function () {
      button.innerHTML = ICON_COPY;
      button.setAttribute("aria-label", label);
      button.classList.remove("is-copied");
    }, 2000);
  }

  function enhance(root) {
    const scope = root || document;
    const blocks = scope.querySelectorAll("pre.rd-code-block:not([data-rocci-copy])");
    for (let i = 0; i < blocks.length; i++) {
      const pre = blocks[i];
      if (pre.closest(".rd-code-wrap")) {
        continue;
      }
      pre.setAttribute("data-rocci-copy", "1");
      const wrap = document.createElement("div");
      wrap.className = "rd-code-wrap";
      pre.parentNode.insertBefore(wrap, pre);
      wrap.appendChild(pre);
      const button = document.createElement("button");
      button.type = "button";
      button.className = "rd-copy";
      button.setAttribute("aria-label", "Copy code");
      button.innerHTML = ICON_COPY;
      wrap.insertBefore(button, pre);
    }
  }

  document.addEventListener("click", function (event) {
    const button = event.target && event.target.closest ? event.target.closest("button.rd-copy") : null;
    if (!button) {
      return;
    }
    const wrap = button.closest(".rd-code-wrap");
    const pre = wrap && wrap.querySelector("pre.rd-code-block");
    if (!pre) {
      return;
    }
    const text = blockText(pre);
    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(text).then(
        function () {
          showCopied(button);
        },
        function () {},
      );
    }
  });

  function init() {
    enhance(document);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }

  window.__rocciCopy = {
    enhance: enhance,
  };
})();
