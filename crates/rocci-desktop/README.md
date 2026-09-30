# rocci-desktop

Rocci preview facade over [`h35-desktop`](https://github.com/koliyo/h35-desktop). Product CLIs still call `preview(PreviewOptions)`.

This crate supplies Rocci defaults: `~/.rocci/state` (or `ROCCI_STATE_DIR`), the Rocci icon, inspector URL construction, and Rocci layout seed. Windowing, toolbar chrome, IPC, and generic persistence live in `h35-desktop`. Rocci owns inspector tab/view names; the host docks the inspector page and remembers an opaque query. There is no migration for earlier desktop state.

Trusted top-level pages use `window.__h35HostSend(message)` for host commands. Rocci page scripts use the host's `__h35Goto`, `__h35LiveReload`, `__h35PreviewNav`, and `--h35-chrome-*` interfaces directly. `PreviewOptions.goto` forwards to the host (default on); rocci-browser turns it off. Fill `body`; do not size shells with `100vh` / `100dvh`. Bare `:scope` in file `@css` matches every stamped element; document height belongs on `html` / `body` or `:scope:is(html)`. The host toolbar is a layout row, not an overlay to pad around.
