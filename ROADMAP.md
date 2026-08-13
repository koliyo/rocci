# Implementation roadmap

## Architectural direction

Keep the runtime small and separable:

1. **Shell** — tao owns the main-thread event loop and native windows; wry owns
   webviews. Window lifecycle events enter an internal Rust event bus.
2. **HTTP runtime** — an async Rust server binds an ephemeral loopback port,
   serves embedded or development assets, and mounts the application's router.
3. **Hypermedia transport** — handlers return HTML or SSE. The core knows only
   HTTP; optional Rust adapters make Datastar event generation ergonomic.
4. **Native capabilities** — dialogs, filesystem access, menus, notifications,
   and similar features are scoped resources mounted under a private HTTP API.
   Permissions are explicit and deny-by-default.
5. **Build tool** — configuration, asset embedding, icons, platform metadata,
   signing, and packaging remain outside the runtime crate.

The contract between UI and backend should stay usable in a normal browser.
That makes browser development, integration testing, htmx, and other SSR
libraries first-class rather than compatibility modes.

## Phase 1 — turn the POC into a reusable Rust runtime

- Split the crate into `roc-core`, `roc-wry`, `roc-http`, and a CLI, while
  keeping a facade crate with a small builder API.
- Accept an arbitrary Tower `Service`/Axum `Router` and provide lifecycle hooks,
  managed Rust state, graceful shutdown, and structured errors.
- Add multi-window support with window-scoped sessions and a typed Rust event
  model for tao callbacks.
- Add development mode with a configurable external frontend/backend URL,
  reload, inspector controls, and production asset embedding.
- Define stable configuration with window, security, asset, and development
  profiles. Validate it during builds.
- Test on macOS, Windows, Linux X11, and Linux Wayland in CI.

Exit criteria: a small Rust application can configure multiple windows, mount
its own routes, embed assets, and package an unsigned development build.

## Phase 2 — Datastar-first application ergonomics

- Wrap the official Rust Datastar SDK with response helpers, reconnect/replay
  support using `Last-Event-ID`, cancellation when a window closes, and
  broadcast fan-out with bounded backpressure.
- Offer patterns for per-window and application-wide signal stores without
  hiding the underlying HTTP endpoints.
- Add examples for forms, validation, background work, progress streams,
  optimistic updates, navigation, and file uploads.
- Provide CSRF/origin middleware suited to loopback webviews and document how
  it composes with application authentication.
- Keep plain HTML responses first-class and add tested htmx examples. Avoid a
  frontend-specific requirement in `roc-core` or `roc-http`.

Exit criteria: the example gallery covers the common workflows normally solved
by desktop IPC while using only HTTP, HTML, and SSE.

## Phase 3 — secure native capability services

- Threat-model loopback attacks, DNS rebinding, token leakage, navigation to
  untrusted origins, malicious dependencies, and compromised renderer content.
- Replace the single POC session with per-window, rotating capabilities and
  constant-time verification. Enforce expected origin/host, request limits,
  timeouts, and audit logging.
- Add permission-scoped HTTP APIs for dialogs, notifications, clipboard, menus,
  application paths, and narrowly scoped filesystem access.
- Stream large files and process output rather than buffering them. Define
  cancellation and cleanup semantics.
- Expose navigation/new-window/download policy hooks and deny unexpected
  external navigation by default. Open allowed external links in the system
  browser.
- Commission an external security review before calling the APIs production
  safe.

Exit criteria: capabilities are individually grantable, testable, observable,
and inaccessible to arbitrary local or remote pages.

## Phase 4 — packaging and platform integration

- Generate macOS app bundles, Windows installers, and Linux packages with icons,
  resources, version metadata, deep links, and file associations.
- Integrate code signing, notarization, update manifest generation, and
  reproducible release builds without owning signing credentials.
- Expand the initial macOS application menu and lifecycle support to Windows
  and Linux; add tray, single-instance behavior, activation/deep-link delivery,
  and accessibility checks.
- Design a signed updater as an optional crate with rollback and channel support.
- Measure startup, memory, binary size, SSE latency, and idle CPU against explicit
  budgets and representative alternatives.

Exit criteria: signed sample applications install, update, and uninstall cleanly
on all supported platforms.

## Phase 5 — ecosystem and additional backends

- Stabilize the HTTP contract before adding non-Rust backend sidecars.
- Specify process startup, readiness, authentication handoff, port negotiation,
  logging, crash recovery, and shutdown as a language-neutral sidecar protocol.
- Build one reference non-Rust adapter only after the Rust runtime and security
  model settle; any HTTP framework should otherwise work unchanged.
- Publish templates for Datastar, htmx, and framework-neutral SSR applications,
  plus migration guidance for applications that currently use Tauri commands.

## Deliberate POC limitations

- One window and one in-memory counter.
- Rust/Axum is the only managed backend.
- No native capabilities beyond the window and webview.
- Packaging is currently limited to a local, ad-hoc-signed macOS `.app`; there
  is no production signing, notarization, installer, updating, tray, deep links,
  or persistence.
- The bootstrap token is long-lived for the process and cookie transport is HTTP
  loopback only; production needs the Phase 3 hardening.
- No SSE replay buffer and only a small bounded broadcast channel.
- The frontend libraries are vendored snapshots and need a documented update
  and integrity process.
