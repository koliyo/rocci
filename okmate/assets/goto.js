(() => {
  const dialog = document.getElementById("okmate-goto");
  const input = document.getElementById("okmate-goto-input");
  const list = document.getElementById("okmate-goto-list");
  if (!dialog || !input || !list) {
    return;
  }

  let pages = [];
  let hits = [];
  let active = 0;

  function render() {
    const query = input.value.trim().toLowerCase();
    hits = pages.filter((page) => {
      const haystack = `${page.title} ${page.route} ${page.path || ""} ${page.collection || ""}`.toLowerCase();
      return !query || haystack.includes(query);
    }).slice(0, 12);
    if (active >= hits.length) {
      active = 0;
    }
    list.replaceChildren(
      ...hits.map((page, index) => {
        const item = document.createElement("li");
        item.className = index === active ? "is-active" : "";
        item.textContent = page.title;
        const route = document.createElement("span");
        route.textContent = page.route;
        item.appendChild(route);
        item.addEventListener("mousedown", (event) => {
          event.preventDefault();
          go(page.route);
        });
        return item;
      }),
    );
  }

  function go(route) {
    dialog.close();
    window.location.assign(route);
  }

  async function openPalette() {
    if (!pages.length) {
      const response = await fetch("/pages.json");
      if (response.ok) {
        pages = await response.json();
      }
    }
    input.value = "";
    active = 0;
    render();
    dialog.showModal();
    input.focus();
  }

  window.addEventListener("keydown", (event) => {
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
      event.preventDefault();
      openPalette();
      return;
    }
    if (!dialog.open) {
      return;
    }
    if (event.key === "Escape") {
      dialog.close();
    } else if (event.key === "ArrowDown") {
      event.preventDefault();
      active = hits.length ? (active + 1) % hits.length : 0;
      render();
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      active = hits.length ? (active - 1 + hits.length) % hits.length : 0;
      render();
    } else if (event.key === "Enter" && hits[active]) {
      event.preventDefault();
      go(hits[active].route);
    }
  });

  input.addEventListener("input", () => {
    active = 0;
    render();
  });
})();
