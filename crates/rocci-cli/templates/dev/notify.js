(function () {
  var p = new URLSearchParams(location.search);
  p.delete("route");
  var msg = { type: "h35-inspector", query: p.toString() };
  parent.postMessage(msg, "*");
})();
