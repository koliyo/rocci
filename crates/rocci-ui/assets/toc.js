(function () {
  if (window.__rocciToc) {
    return;
  }
  var token = 0;
  var pending = null;
  var spyFrame = 0;

  function tocLink(node) {
    while (node) {
      if (
        node.nodeType === 1 &&
        node.classList &&
        (node.classList.contains("rd-toc-link") || node.classList.contains("outline-link"))
      ) {
        return node;
      }
      node = node.parentNode;
    }
    return null;
  }

  function isScrollableY(node) {
    if (!node || node === document.body || node === document.documentElement) {
      return false;
    }
    var style = window.getComputedStyle(node);
    var overflowY = style.overflowY;
    return (
      (overflowY === "auto" || overflowY === "scroll" || overflowY === "overlay") &&
      node.scrollHeight > node.clientHeight + 1
    );
  }

  function scrollerFor(el) {
    var node = el.parentElement;
    while (node && node !== document.body && node !== document.documentElement) {
      if (isScrollableY(node)) {
        return node;
      }
      node = node.parentElement;
    }
    return null;
  }

  function yNow(scroller) {
    if (scroller) {
      return scroller.scrollTop;
    }
    return window.pageYOffset || document.documentElement.scrollTop || document.body.scrollTop || 0;
  }

  function ySet(scroller, y) {
    if (scroller) {
      scroller.scrollTop = y;
      return;
    }
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

  function chromeOffset() {
    var nav = document.querySelector("rocci-preview-nav");
    if (!nav) {
      return 0;
    }
    var height = nav.getBoundingClientRect().height;
    return height > 0 ? height : 0;
  }

  function targetY(el, scroller) {
    var margin = parseFloat(window.getComputedStyle(el).scrollMarginTop);
    if (isNaN(margin)) {
      margin = 0;
    }
    var chrome = chromeOffset();
    if (chrome + 8 > margin) {
      margin = chrome + 8;
    }
    if (scroller) {
      return (
        scroller.scrollTop +
        el.getBoundingClientRect().top -
        scroller.getBoundingClientRect().top -
        margin
      );
    }
    return el.getBoundingClientRect().top + yNow(null) - margin;
  }

  function restorePending() {
    if (pending) {
      if (!pending.el.id) {
        pending.el.id = pending.id;
      }
      pending = null;
    }
  }

  function animate(scroller, to, href) {
    var from = yNow(scroller);
    var dist = to - from;
    function done() {
      ySet(scroller, to);
      restorePending();
      if (history.replaceState) {
        history.replaceState(null, "", href);
      }
      syncSpy();
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
      ySet(scroller, from + dist * k);
      requestAnimationFrame(frame);
    }
    requestAnimationFrame(frame);
  }

  function tocLinks() {
    return document.querySelectorAll(".rd-toc-link[href^='#'], .outline-link[href^='#']");
  }

  function headingId(href) {
    if (!href || href.charAt(0) !== "#") {
      return "";
    }
    try {
      return decodeURIComponent(href.slice(1));
    } catch (err) {
      return href.slice(1);
    }
  }

  function syncSpy() {
    var links = tocLinks();
    if (!links.length) {
      return;
    }
    var mark = chromeOffset() + 48;
    var currentId = "";
    var firstId = "";
    for (var i = 0; i < links.length; i++) {
      var id = headingId(links[i].getAttribute("href") || "");
      if (!id) {
        continue;
      }
      var el = document.getElementById(id);
      if (!el) {
        continue;
      }
      if (!firstId) {
        firstId = id;
      }
      if (el.getBoundingClientRect().top <= mark) {
        currentId = id;
      }
    }
    if (!currentId) {
      currentId = firstId;
    }
    for (var j = 0; j < links.length; j++) {
      var on = headingId(links[j].getAttribute("href") || "") === currentId;
      links[j].classList.toggle("is-current", on);
      if (on) {
        links[j].setAttribute("aria-current", "location");
      } else if (links[j].getAttribute("aria-current") === "location") {
        links[j].removeAttribute("aria-current");
      }
    }
  }

  function requestSpy() {
    if (spyFrame) {
      return;
    }
    spyFrame = requestAnimationFrame(function () {
      spyFrame = 0;
      syncSpy();
    });
  }

  function enhance() {
    syncSpy();
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
      var id = headingId(href);
      var el = document.getElementById(id);
      if (!el) {
        return;
      }
      var scroller = scrollerFor(el);
      var to = Math.max(0, targetY(el, scroller));
      event.preventDefault();
      if (event.stopImmediatePropagation) {
        event.stopImmediatePropagation();
      }
      restorePending();
      pending = { el: el, id: id };
      el.removeAttribute("id");
      animate(scroller, to, href);
    },
    true
  );
  document.addEventListener("scroll", requestSpy, true);
  window.addEventListener("resize", requestSpy);
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", enhance);
  } else {
    enhance();
  }
  window.__rdTocScroll = true;
  window.__rocciToc = { enhance: enhance };
})();
