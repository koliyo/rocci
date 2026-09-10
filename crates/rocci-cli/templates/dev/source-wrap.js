(function () {
  var key = "rocci-dev-wrap";
  var pane = document.querySelector(".code-pane");
  var box = document.getElementById("source-wrap");
  if (!pane || !box) return;
  function apply(on) {
    pane.classList.toggle("no-wrap", !on);
    try {
      sessionStorage.setItem(key, on ? "1" : "0");
    } catch (err) {}
  }
  var stored = null;
  try {
    stored = sessionStorage.getItem(key);
  } catch (err) {}
  var wrap = stored !== "0";
  box.checked = wrap;
  apply(wrap);
  box.addEventListener("change", function () {
    apply(box.checked);
  });
})();
