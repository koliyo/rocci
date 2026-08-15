# Implementation roadmap

## Architectural direction

Rocci is a `.rocci` authoring language plus a small desktop host:

1. **Templates** — `.rocci` modules lower to ordinary Roc HTML. The compiler
   does not type-check Roc or own HTTP behavior.
2. **Roc apps** — `main.roc` on [basic-webserver](https://github.com/roc-lang/basic-webserver)
   serves HTML and Datastar SSE. `rocci run` compiles sibling `.rocci` files
   and starts that server.
3. **Shell** — `rocci-wry` opens a tao/wry preview window against the local
   server. `rocci bundle` wraps the same host plus a `roc build` server binary
   in an ad-hoc signed macOS `.app`.
4. **Tooling** — `rocci-cli` and `rocci-lsp` stay the front door for build,
   run, view, browse, and editor support.

The contract between UI and backend should stay usable in a normal browser.

## Current focus

- [x] `.rocci` parse, lower, and compile to Roc type modules
- [x] `rocci run` / `view` / `browse` with an embedded preview window
- [x] Example apps: counter, snake, and the Datastar gallery
- [x] macOS ad-hoc `.app` packaging that wraps a compiled Roc server
- [ ] Test on macOS, Windows, Linux X11, and Linux Wayland in CI
- [ ] Windows and Linux installers; production signing and notarization
- [ ] Native capability APIs (dialogs, filesystem, notifications) as
      authenticated HTTP resources

## Deliberate remaining limitations

- Packaging is a local, ad-hoc-signed macOS `.app`. There is no production
  signing, notarization, updater, tray, or deep links.
- The preview host is a single window. Multi-window desktop chrome, menus, and
  dock lifecycle are not wired to Roc apps yet.
- No native capabilities beyond the window and webview.
- The frontend libraries are vendored snapshots and need a documented update
  and integrity process.
