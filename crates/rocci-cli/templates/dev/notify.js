(function () {
  var p = new URLSearchParams(location.search);
  var msg = {
    type: "h35-inspector",
    tab: p.get("tab") || "performance",
    view: p.get("view") || "source",
  };
  parent.postMessage(msg, "*");
})();
