(function () {
  if (window.__rocciLiveReload) {
    return;
  }
  var KEY = "rocci-live-reload";
  var es = null;
  var dirty = false;
  function seedFromQuery() {
    try {
      if (new URLSearchParams(window.location.search).get("reload") === "0") {
        sessionStorage.setItem(KEY, "0");
      }
    } catch (err) {}
  }
  function enabled() {
    try {
      return sessionStorage.getItem(KEY) !== "0";
    } catch (err) {
      return true;
    }
  }
  function setEnabled(on) {
    try {
      sessionStorage.setItem(KEY, on ? "1" : "0");
    } catch (err) {}
    if (on && dirty) {
      location.reload();
    }
  }
  function connect() {
    if (es) {
      return;
    }
    es = new EventSource("/__rocci/events");
    es.addEventListener("reload", function () {
      if (enabled()) {
        location.reload();
      } else {
        dirty = true;
      }
    });
    es.onerror = function () {
      if (es) {
        es.close();
      }
      es = null;
      setTimeout(connect, 1000);
    };
  }
  window.__rocciLiveReload = { enabled: enabled, set: setEnabled };
  seedFromQuery();
  connect();
})();
