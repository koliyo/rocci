# Implementation roadmap

## Architectural direction

Rocci and Rocdown provide clean, symmetrical application and document pipelines:

1. **Templates & Apps (`rocci`)** — `.rocci` modules lower to ordinary Roc HTML. `@context` /
   `@init` / `@method:role` declare standalone HTTP apps. Authored `main.roc` apps keep full
   control of `init!` / `respond!`. `rocci run` stages `Html.roc` / `Datastar.roc`
   and starts the server.
2. **Documents & Sites (`rocdown`)** — `.rocdown` files are Markdown with document-root
   `@` declarations. Single documents compile to Roc and run via `rocdown run FILE.rocdown`.
   Multi-page documentation sites compile via `rocdown build` using a Rust catalog,
   article rendering, and a once-compiled `RocdownTheme.rocci` shell. Dynamic islands and
   `@island` remain ahead.
3. **Open Knowledge Format (`okf` & `rocci-okf`)** — The portable `okf` engine manages
   parsing, validation, graphs, search, and benchmarks without dependencies. The
   `rocci-okf` application provides review, live reload, and query workflows.
4. **Shell & Presentation (`rocci-desktop` & `rocci-ui`)** — `rocci-desktop` opens a Tao/Wry
   preview window against the local server and bundles macOS apps. `rocci-ui` provides
   domain-neutral view records and presentation components.
5. **Tooling (`rocci-lsp` & `rocci-highlight`)** — Pinned Tree-sitter highlighter and common
   LSP server for `.rocci` and `.rocdown`.

The contract between UI and backend should stay usable in a normal browser.

## Current focus

- [x] `.rocci` parse, lower, and compile to Roc type modules
- [x] `rocci run` / `view` / `browse` with an embedded preview window
- [x] Standalone `rocci run App.rocci` from `@context` / `@init` / `@method:role`
- [x] Example apps: counter (standalone), styling, snake, and the Datastar gallery
- [x] `.rocdown` compiler core: Markdown-first pages with `@page` / `@roc` /
      `@render` and delegated Rocci declarations, lowering to Roc; `rocdown run`
      for a single file. See [`crates/rocci-rocdown`](crates/rocci-rocdown).
- [x] Rocdown SSG: Rust catalog + article HTML, nested routes, graph/nav/validation,
      drafts, hashed assets, CSP, and `rocdown run` / `build`
- [x] Product consolidation: decouple Rocdown from base Rocci, retire Rocs
- [x] Portable OKF engine (`okf`) and standalone review app (`rocci-okf`)
- [x] Domain-neutral presentation components in `rocci-ui`
- [x] Rename `rocci-wry` to `rocci-desktop`
- [x] `.rocdown` and `.rocci` LSP and editor registration (VS Code, Zed)
- [ ] `@island` for `.rocci` and `.rocdown` (dynamic island splicing)
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
