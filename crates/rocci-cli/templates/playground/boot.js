import { PlaygroundApp } from "/app.js";

async function init() {
  try {
    const resp = await fetch("/api/session");
    const bootstrap = await resp.json();
    const root = document.getElementById("playground-root");
    new PlaygroundApp({ container: root, bootstrap });
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    document.body.textContent = "Failed to load playground session: " + message;
    document.body.style.padding = "24px";
    document.body.style.color = "red";
  }
}
init();
