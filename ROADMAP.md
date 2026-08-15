# Implementation roadmap

## Architectural direction

Rocci is a `.rocci` authoring language, a `.rocdown` content format, and a
small desktop host:

1. **Templates** — `.rocci` modules lower to ordinary Roc HTML. `@context` /
   `@init` / `@on` are parsed here and emitted as Roc functions plus route
   metadata. The compiler does not type-check Roc or spawn HTTP.
2. **Rocdown** — `.rocdown` files are Markdown with document-root `@`
   declarations. They lower to the same `Html`, CSS, and route artifacts.
   Full SSG, LSP, and `@island` are still ahead.
3. **Roc apps** — Standalone `rocci run App.rocci` generates a basic-webserver
   dispatcher. Authored `main.roc` apps keep full control of `init!` /
   `respond!`. `rocci run` stages `Html.roc` / `Datastar.roc` and starts the
   server.
4. **Shell** — `rocci-wry` opens a tao/wry preview window against the local
   server. `rocci bundle` wraps the same host plus a `roc build` server binary
   in an ad-hoc signed macOS `.app`.
5. **Tooling** — `rocci-cli` and `rocci-lsp` stay the front door for build,
   run, view, browse, and editor support.

The contract between UI and backend should stay usable in a normal browser.

## Current focus

- [x] `.rocci` parse, lower, and compile to Roc type modules
- [x] `rocci run` / `view` / `browse` with an embedded preview window
- [x] Standalone `rocci run App.rocci` from `@context` / `@init` / `@on`
- [x] Example apps: counter (standalone), styling, snake, and the Datastar gallery
- [x] `.rocdown` compiler core: Markdown-first pages with `@page` / `@roc` /
      `@render` and delegated Rocci declarations, lowering to Roc; `rocci run`
      for a single file. See [`crates/rocci-rocdown`](crates/rocci-rocdown).
- [ ] Rocdown SSG (multi-page routes, layouts, drafts, `dist/` output)
- [ ] `.rocdown` LSP and editor registration
- [ ] `@island` for `.rocci` and `.rocdown`
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
- Datastar.js is fetched into `~/.rocci/cache` and pinned per app; `rocci run`
  never auto-upgrades. Use `rocci datastar update` to bump a pin.
