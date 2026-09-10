(function () {
  var root = document.querySelector("[data-logs-root]");
  if (!root) return;
  var api = root.getAttribute("data-logs-root") || "/__rocci";
  var pane = document.getElementById("console-log");
  var body = pane && pane.querySelector("tbody");
  if (!pane || !body) return;
  var levels = { debug: true, info: true, warn: true, error: true };
  var stick = true;
  function near() {
    return pane.scrollHeight - pane.scrollTop - pane.clientHeight < 32;
  }
  function apply() {
    var rows = body.querySelectorAll("tr[data-level]");
    for (var i = 0; i < rows.length; i++) {
      rows[i].hidden = !levels[rows[i].getAttribute("data-level")];
    }
  }
  function row(line) {
    var tr = document.createElement("tr");
    tr.setAttribute("data-level", line.level || "info");
    var t = new Date(Number(line.t) || 0);
    var time = isNaN(t.getTime())
      ? String(line.t || "")
      : t.toLocaleTimeString();
    tr.innerHTML =
      "<td>" +
      time +
      "</td><td>" +
      esc(line.level) +
      "</td><td>" +
      esc(line.source) +
      "</td><td>" +
      esc(line.text) +
      "</td>";
    return tr;
  }
  function esc(v) {
    return String(v == null ? "" : v).replace(/[&<>\"]/g, function (ch) {
      return ch === "&"
        ? "&amp;"
        : ch === "<"
          ? "&lt;"
          : ch === ">"
            ? "&gt;"
            : "&quot;";
    });
  }
  pane.addEventListener("scroll", function () {
    stick = near();
  });
  var chips = root.querySelectorAll(".console-filters button[data-level]");
  for (var c = 0; c < chips.length; c++) {
    (function (btn) {
      btn.addEventListener("click", function () {
        var level = btn.getAttribute("data-level");
        levels[level] = !levels[level];
        btn.setAttribute("aria-pressed", levels[level] ? "true" : "false");
        apply();
      });
    })(chips[c]);
  }
  var clear = root.querySelector(".console-clear");
  if (clear) {
    clear.addEventListener("click", function () {
      fetch(api + "/logs/clear", { method: "POST" }).then(function () {
        body.innerHTML = "";
      });
    });
  }
  try {
    var es = new EventSource(api + "/logs/events");
    es.addEventListener("log", function (ev) {
      var keep = stick || near();
      try {
        body.appendChild(row(JSON.parse(ev.data)));
      } catch (err) {}
      apply();
      if (keep) {
        pane.scrollTop = pane.scrollHeight;
      }
    });
  } catch (err) {}
})();
