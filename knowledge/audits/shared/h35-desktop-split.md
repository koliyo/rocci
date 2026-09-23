---
type: Audit
title: h35-desktop split and host boundary review
description: The shared native host is a sound extraction; IPC and product-state gaps are addressed in the published host commits and local product adapters, pending cross-platform and hosted validation.
tags: [domain/rocci, domain/desktop, domain/okf, concern/architecture, concern/validation]
status: draft
generated: { by: process:cursor, at: 2026-09-23T12:55:04Z }
stale_after: 2026-12-23
authority: descriptive
owners: [human:nils]
sources:
  - id: architecture
    resource: ../../architecture/system-overview.md
    title: Rocci system overview
    author: process:cursor
    last_modified: 2026-09-10
  - id: host-readme
    resource: ../../../../h35-desktop/README.md
    title: h35-desktop contract
    author: process:git
    last_modified: 2026-09-23
  - id: host-preview
    resource: ../../../../h35-desktop/src/preview.rs
    title: Host options, IPC dispatch, and event loop
    author: process:git
    last_modified: 2026-09-23
  - id: host-trust
    resource: ../../../../h35-desktop/src/trust.rs
    title: Trusted preview origins and IPC bridge token
    author: process:git
    last_modified: 2026-09-23
  - id: host-window
    resource: ../../../../h35-desktop/src/window.rs
    title: Wry webview construction
    author: process:git
    last_modified: 2026-09-23
  - id: host-external
    resource: ../../../../h35-desktop/src/external.rs
    title: External navigation policy
    author: process:git
    last_modified: 2026-09-23
  - id: host-history
    resource: ../../../../h35-desktop/src/history.rs
    title: IPC message parsing and navigation history
    author: process:git
    last_modified: 2026-09-12
  - id: host-chrome
    resource: ../../../../h35-desktop/assets/preview-nav.js
    title: Injected chrome and inspector behavior
    author: process:git
    last_modified: 2026-09-23
  - id: host-state
    resource: ../../../../h35-desktop/src/state.rs
    title: Persisted window and inspector state
    author: process:git
    last_modified: 2026-09-23
  - id: rocci-facade
    resource: ../../../crates/rocci-desktop/src/lib.rs
    title: Rocci product adapter
    author: process:git
    last_modified: 2026-09-23
  - id: rocci-pin
    resource: ../../../Cargo.toml
    title: Rocci h35-desktop dependency
    author: process:git
    last_modified: 2026-09-23
  - id: rocci-lock
    resource: ../../../Cargo.lock
    title: Rocci resolved h35-desktop dependency
    author: process:git
    last_modified: 2026-09-23
  - id: okmate-adapter
    resource: ../../../../okmate/src/desktop.rs
    title: Okmate desktop call
    author: process:git
    last_modified: 2026-09-11
  - id: okmate-pin
    resource: ../../../../okmate/Cargo.toml
    title: Okmate h35-desktop dependency
    author: process:git
    last_modified: 2026-09-23
  - id: okmate-lock
    resource: ../../../../okmate/Cargo.lock
    title: Okmate resolved h35-desktop dependency
    author: process:git
    last_modified: 2026-09-23
  - id: body-box
    resource: ../../research/shared/h35-desktop-body-box.md
    title: Desktop host body box and file-scoped :scope
    author: process:cursor
    last_modified: 2026-09-23
  - id: wry-ipc
    resource: https://docs.rs/wry/0.56.1/wry/struct.WebViewBuilder.html#method.with_ipc_handler
    title: Wry 0.56.1 IPC handler documentation
    author: process:git
---

# h35-desktop split and host boundary review

## Scope and method

This is a code-based audit of the September 2026 extraction. The findings
below describe the version before the local implementation follow-up at the
end of this record. At initial audit, the host's 52 Rust unit tests, Rocci
facade's four tests, and Okmate's desktop-feature compilation passed locally.
No native-window interaction, cross-platform runtime smoke, or hostile-page
demonstration was run then. Findings distinguish code paths from demonstrated
effects.[^architecture][^host-preview][^rocci-facade][^okmate-adapter]

## Overall judgment

**Keep the repository and crate split.** Both products need the same Tao/Wry
window, menu, state, and injected toolbar. `h35-desktop` depends on neither
product nor its rendering stack; Rocci's facade supplies its icon, state path,
layout seed, and inspector URL; Okmate supplies its own options directly. The page
origin owns document content. This is a useful one-way dependency, and the
recent full-height site-bar problem does not justify merging the host back
into either product.[^host-readme][^rocci-facade][^okmate-adapter][^body-box]

The extraction is less complete as a **general-purpose host contract** than
its name suggests. The current API is a single blocking `preview()` session,
and some inspector and document conventions are baked into the host. Preserve
the split while narrowing those seams; a new windowing framework is not
supported by the evidence here.[^host-preview][^host-chrome][^host-state]

## Findings

### 1. IPC has no trusted-origin boundary (high priority)

The Wry handler passes each request body to the host parser or product
`on_ipc` callback without inspecting the request URI. The navigation handler
keeps *any* loopback HTTP(S) address in the webview and allows `file:` and
other unrecognized schemes. The host also installs `window.ipc` and its
initialization script for the webview. Thus, if the webview loads an
untrusted local page, that page can send host commands, including folder
picker, close window, and—when a source root is configured—source reveal or
copy. This is a trust-boundary risk inferred from the dispatch paths, not a
claim that an exploit has been reproduced.[^host-window][^host-external][^host-preview][^host-history]

An exact-origin allowlist should be part of `HostOptions`, updated explicitly
when a product changes its preview origin. Apply it before both built-in IPC
and `on_ipc`, and restrict top-level navigation schemes independently. Treat
an inspector iframe as a separate trust case: Wry documents that on Linux the
IPC request URL for an iframe is the main-frame URL, so URI equality alone
cannot authorize a privileged iframe message there. Keep privileged commands
unavailable to frames whose provenance cannot be established.[^host-preview][^wry-ipc]

### 2. Inspector and document semantics leak into the shared host (medium priority)

Dock position and window geometry belong in the host. Fixed inspector tabs
(`performance`, `source`, `console`), source views (`ast`, `roc`, `html`),
`route`/`tab`/`view` URL construction, the `rocci-inspector` message alias,
and persisted `nav`/`outline` columns are product concepts currently encoded
in host JavaScript or state. They cause a generic host release to know about
Rocci's inspector and document schema, even though Okmate does not enable the
inspector URL. Keep generic dock mechanics in h35; let the product inspector
own its tab and view names, URL parameters, and document-specific state, with
an opaque or explicitly configured exchange if the dock must remember them.
This can be a gradual API change.[^host-chrome][^host-state][^host-preview][^okmate-adapter]

### 3. The DOM layout contract is deliberate but costly (medium priority)

The injected host style sets `html` to a grid and applies `!important` sizing,
margin, and overflow rules to `body`. It makes the toolbar and dock reserve a
real content box, which fixed the earlier overlap. It also means a page that
assumes viewport-sized `body` or owns document overflow can break inside the
preview. The recent Rocci site-bar failure came from file-scoped `:scope`
height in the product theme; it does not prove the host grid is defective.
Document the body-box contract and add visual smoke cases for both products.
If arbitrary third-party pages become a product requirement, reconsider
placing host chrome outside the page DOM; the current evidence does not yet
justify that larger rewrite.[^host-chrome][^host-readme][^body-box]

### 4. Pin discipline needs a compatibility rule, not a HEAD rule (low priority)

Both current product manifests and lockfiles resolve `c465c343bfd5d623b62849c09af9214fe3c5f5c5`, the h35 merge commit.
The earlier body-box research and plan describe divergent pins and unfinished
pin work; that was accurate before the subsequent pin commits, but is stale
for this checkout. Exact revisions are good for reproducibility. Requiring
both products always to track host `main` HEAD would discard the useful
ability to test and roll forward each product separately. Prefer a tested
compatible revision per product, with a small cross-product contract check
before changing the host API.[^rocci-pin][^rocci-lock][^okmate-pin][^okmate-lock][^body-box]

## Recommended order

1. Define trusted page origins and an IPC capability matrix in h35; test
   accepted preview messages and rejected foreign-origin messages on each
   supported platform, accounting for Wry's iframe limitation.[^host-preview][^wry-ipc]
2. Make inspector tab/view state product-owned while preserving host dock
   geometry. Earlier product-specific saved fields can be discarded rather
   than migrated.[^host-chrome][^host-state]
3. Keep the body-box contract and test representative Rocci and Okmate pages
   in a real webview. Treat a separate native chrome surface as conditional on
   a demonstrated arbitrary-page requirement.[^host-readme][^body-box]
4. Correct the older draft research/plan's current-pin wording before using
   them as an implementation checklist.[^rocci-pin][^okmate-pin][^body-box]

## Local implementation follow-up — 2026-09-23

The host now restricts top-level navigation to the configured preview, Home,
and inspector origins. IPC requires both one of those origins and a per-window
bridge token installed only in the top frame; the check precedes built-in
commands and product callbacks. The inspector is deliberately in the same
capability class as a trusted preview page. This resolves the dispatch gap
identified above in code, including Wry's reported iframe-URI ambiguity,
but hostile-page and cross-platform native-window demonstrations remain
unrun.[^host-trust][^host-preview][^host-window][^host-readme]

The host now persists an opaque inspector query and generic layout map.
Rocci constructs its own inspector URL with `route` and owns its tab/view
selection. Earlier product-specific saved fields and preference keys are
ignored; the Rocci adapter has no migration or compatibility aliases. Product
page scripts use the host interfaces directly. Okmate's page commands use the
authenticated bridge, and its settings page listens to `h35-pick-folder`
directly. The body-box layout contract remains documented.
These are local implementation observations, not a claim of a shipped
release.[^host-chrome][^host-state][^rocci-facade][^host-readme]

The local host's 54 tests, Rocci's workspace suite (including four desktop
adapter tests), and Okmate's full desktop-feature suite passed. Rocci and Okmate
pages rendered normally in a browser, and both native preview processes
launched. The browser checks did not include injected host chrome; visual
native-window inspection and Linux/Windows runtime checks remain open.
Host CI passed on `85676e8` (run `35862994113`, macOS and Linux). The local
product changes have no hosted CI result. Both product manifests and lockfiles
pin `85676e8`; Rocci's locked offline workspace suite and Okmate's locked
offline desktop-feature suite pass.
The products can advance their pins independently;
the older draft research and plan have been
corrected to describe their original pins as historical evidence rather
than an always-HEAD rule.[^rocci-pin][^okmate-pin][^body-box]

[^architecture]: Recorded one-way Rocci product boundaries, checked against current manifests and adapters.
[^host-readme]: Host responsibility and product-page body-box contract.
[^host-preview]: The audited unconditional dispatch was replaced by origin and token checks; the one-session event loop remains.
[^host-trust]: Current trusted-origin classification and token-authenticated bridge implementation.
[^host-window]: Wry navigation, initialization-script, and IPC registration.
[^host-external]: The audited navigation policy allowed any loopback HTTP(S) origin and unknown schemes; the preview now uses a narrower policy.
[^host-history]: Built-in IPC messages and their parsed effects.
[^host-chrome]: The grid remains; fixed inspector tab/view names have been removed from the host script.
[^host-state]: Window geometry remains host-owned; generic layout/query fields replace product-specific persisted fields.
[^rocci-facade]: Rocci's product defaults, layout seed, and inspector URL construction.
[^rocci-pin]: Rocci's exact h35 dependency revision.
[^rocci-lock]: Rocci's resolved h35 commit.
[^okmate-adapter]: Okmate's direct host call and inspector-disabled configuration.
[^okmate-pin]: Okmate's exact h35 dependency revision.
[^okmate-lock]: Okmate's resolved h35 commit.
[^body-box]: Existing exploratory explanation of the recent CSS failure and earlier pin state.
[^wry-ipc]: Wry documents that Linux and Android report the main-frame URL for iframe IPC.
