(function () {
  if (window.__rdTocScroll) {
    return;
  }
  window.__rdTocScroll = true;
  var token = 0;
  var pending = null;
  function yNow() {
    return window.pageYOffset || document.documentElement.scrollTop || document.body.scrollTop || 0;
  }
  function ySet(y) {
    var html = document.documentElement;
    var body = document.body;
    if (html) {
      html.style.scrollBehavior = "auto";
    }
    if (body) {
      body.style.scrollBehavior = "auto";
    }
    if (window.scrollTo) {
      window.scrollTo(0, y);
    }
    if (html) {
      html.scrollTop = y;
    }
    if (body) {
      body.scrollTop = y;
    }
  }
  function restorePending() {
    if (pending) {
      if (!pending.el.id) {
        pending.el.id = pending.id;
      }
      pending = null;
    }
  }
  function tocLink(node) {
    while (node) {
      if (node.nodeType === 1 && node.classList && node.classList.contains("rd-toc-link")) {
        return node;
      }
      node = node.parentNode;
    }
    return null;
  }
  function animate(to, href) {
    var from = yNow();
    var dist = to - from;
    function done() {
      ySet(to);
      restorePending();
      if (history.replaceState) {
        history.replaceState(null, "", href);
      }
    }
    if (Math.abs(dist) < 2) {
      done();
      return;
    }
    var dur = Math.min(650, 400 + Math.abs(dist) * 0.05);
    var start = performance.now();
    var run = ++token;
    function frame(now) {
      if (run !== token) {
        return;
      }
      var t = (now - start) / dur;
      if (t >= 1) {
        done();
        return;
      }
      var k = t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;
      ySet(from + dist * k);
      requestAnimationFrame(frame);
    }
    requestAnimationFrame(frame);
  }
  document.addEventListener(
    "click",
    function (event) {
      var link = tocLink(event.target);
      if (!link) {
        return;
      }
      var href = link.getAttribute("href") || "";
      if (href.charAt(0) !== "#") {
        return;
      }
      var id = decodeURIComponent(href.slice(1));
      var el = document.getElementById(id);
      if (!el) {
        return;
      }
      var margin = parseFloat(window.getComputedStyle(el).scrollMarginTop);
      if (isNaN(margin)) {
        margin = 0;
      }
      var nav = document.querySelector("rocci-preview-nav");
      if (nav) {
        var chrome = nav.getBoundingClientRect().height;
        if (chrome > margin) {
          margin = chrome + margin;
        }
      }
      var to = el.getBoundingClientRect().top + yNow() - margin;
      event.preventDefault();
      if (event.stopImmediatePropagation) {
        event.stopImmediatePropagation();
      }
      restorePending();
      pending = { el: el, id: id };
      el.removeAttribute("id");
      animate(to, href);
    },
    true
  );
})();
