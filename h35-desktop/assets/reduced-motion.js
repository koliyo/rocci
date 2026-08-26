(function () {
  var nativeMatchMedia = window.matchMedia.bind(window);
  window.matchMedia = function (query) {
    var mql = nativeMatchMedia(query);
    if (String(query).toLowerCase().indexOf("prefers-reduced-motion") !== -1) {
      try {
        Object.defineProperty(mql, "matches", {
          configurable: true,
          get: function () {
            return false;
          },
        });
      } catch (err) {}
    }
    return mql;
  };
  var css = document.createElement("style");
  css.textContent =
    "@media (prefers-reduced-motion: reduce) { html, body { scroll-behavior: smooth; } }";
  if (document.documentElement) {
    document.documentElement.appendChild(css);
  } else {
    document.addEventListener("DOMContentLoaded", function () {
      document.documentElement.appendChild(css);
    });
  }
})();
