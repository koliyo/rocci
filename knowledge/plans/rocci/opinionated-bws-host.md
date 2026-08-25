---
type: Implementation Plan
title: Opinionated basic-webserver overlay for method-role constructors
description: "Exploratory overlay of basic-webserver 0.16 that exposes pf.Rocci wraps. App still match-dispatches. A List of constructors linked then crashed at runtime. Hosted register needs a rebuilt libhost.a. Not a shipped host and not an SSE-idle fork."
tags: [domain/rocci, domain/runtime, integration/roc, integration/datastar, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-08-25T17:20:00Z }
stale_after: 2026-11-25
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../../research/rocci/method-role-handlers-as-roc-library.md
    title: Method-role handlers as a pure Roc library or platform
    author: process:cursor
    last_modified: 2026-08-25
  - id: bws-sse
    resource: ../../research/rocci/basic-webserver-sse-http.md
    title: basic-webserver 0.16 SSE and HTTP limits
    author: process:cursor
    last_modified: 2026-08-21
  - id: dispatch
    resource: ../../../crates/rocci-cli/src/dispatch.rs
    title: Pinned BWS 0.16.0 platform URL and generated wraps
    author: process:git
    last_modified: 2026-08-22
  - id: overlay
    resource: ../../../experiments/rocci-web/overlay/Rocci.roc
    title: Platform-owned method-role wraps
    author: process:git
    last_modified: 2026-08-25
  - id: probe
    resource: ../../../experiments/rocci-web/overlay/ROUTES_PROBE.md
    title: List-of-constructors probe notes
    author: process:git
    last_modified: 2026-08-25
---

# Opinionated basic-webserver overlay for method-role constructors

## Goal

Keep [basic-webserver 0.16](https://github.com/roc-lang/basic-webserver) as the
HTTP/SSE/SQLite host and overlay an opinionated Roc `pf.Rocci` so a hybrid
gallery can look more like Rocci without changing `@method:role` or generated
dispatch.[^research][^dispatch]

## Out of bound

- `.rocci` grammar, `legal_pair`, generated `main.roc`
- Forking the host for SSE idle timeout or pub/sub[^bws-sse]
- `Rocci.component` I/O; cutting over `examples/`
- Promoting the overlay to a shipped platform

## Constraints that do not move

- Same BWS 0.16.0 tarball as `dispatch.rs`; no host rebuild.
- Gallery markup stays `Ui.rocci` (`@component` / `@css` only).
- Knowledge records stay inert Markdown plus OKF YAML.

## What compiled

`experiments/rocci-web/fetch-bws.sh` unpacks the same 0.16.0 tarball as
`dispatch.rs` into gitignored `vendor/`, then copies
`experiments/rocci-web/overlay/Rocci.roc`.[^overlay][^dispatch]
The gallery `pf` is `"../rocci-web/vendor/main.roc"`. Wraps (view, fragment,
events, unfold, command-from-header-list) live on `pf.Rocci`. The app still
`match`es `(method, path)`. Markup stays `Ui.rocci`.

`request.headers()` inside a non-app module still crashes; the app passes
`request.headers()` into `Rocci.command!`.[^overlay]

## Constructor encodings that stopped

`requires { routes }` cannot name `Rocci.Route` (requires is parsed before
`import Rocci`). A `Routes : route_list` type variable stays opaque in
`respond_for_host!`.[^probe]

A two-route `List` of `Rocci.view` / `Rocci.fragment` plus `Rocci.dispatch!`
**linked**, then crashed at runtime (`dispatch on a value that can never
exist`). Not a codegen SIGSEGV.[^probe]

Hosted `register_*` needs a rebuilt `libhost.a` from basic-webserver source.
The release tarball is prebuilt; this overlay does not vendor that Rust
crate.[^research]

## Status

Exploratory experiment on `method-role-lib`. Not a product host. Do not log
phases complete until CI and Knowledge succeed.

[^research]: Library on BWS is the recommended counterfactual; a custom platform is mostly who owns `respond!`; do not fork for live idle timeout.
[^bws-sse]: Keepalives and empty SSE; do not fork the platform for idle timeout.
[^dispatch]: `PLATFORM` URL `…/0.16.0/42jC1JT3…tar.zst`.
[^overlay]: `pf.Rocci` wraps; command takes a header list.
[^probe]: `experiments/rocci-web/overlay/ROUTES_PROBE.md`.
