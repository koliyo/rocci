---
type: Status
title: Known Rocci limitations
description: Rocci deliberately lacks dynamic Rocdown islands, full-text documentation-site search, production packaging, broad native APIs, and full cross-platform validation.
tags: [domain/rocci, domain/rocdown, domain/desktop, concern/validation, concern/packaging]
status: draft
generated: { by: process:cursor, at: 2026-08-22T12:00:00Z }
verified:
  - { by: human:nils, at: 2026-08-16T18:14:13Z }
stale_after: 2026-11-19
authority: descriptive
owners: [human:nils]
sources:
  - id: roadmap
    resource: ../../ROADMAP.md
    title: Implementation roadmap
    author: human:nils
    last_modified: 2026-08-17
  - id: status-doc
    resource: ../../docs/project/status.rocdown
    title: Published project status
    author: human:nils
    last_modified: 2026-08-17
  - id: rocdown-site
    resource: ../../crates/rocci-rocdown/src/site.rs
    title: Current Rocdown site loader
    author: process:git
    last_modified: 2026-08-17
  - id: rocdown-article
    resource: ../../crates/rocci-rocdown/src/article.rs
    title: Current Rocdown static-document feature gate
    author: process:git
    last_modified: 2026-08-17
  - id: roadmap-plan
    resource: ../plans/rocdown/rocdown-boundary-refactor.md
    title: Rocdown refactor plan
    author: process:codex
    last_modified: 2026-08-17
  - id: okf
    resource: ../../crates/okf/README.md
    title: OKF portable engine
    author: process:git
    last_modified: 2026-08-17
  - id: goto-js
    resource: ../../crates/rocci-ui/assets/goto.js
    title: Shared go-to-page palette
    author: process:cursor
    last_modified: 2026-08-19
  - id: fuzzy-plan
    resource: ../plans/shared/fuzzy-navigation.md
    title: Cmd-K fuzzy navigation plan
    author: process:cursor
    last_modified: 2026-08-19
  - id: site-ref
    resource: ../../docs/reference/rocdown-site.rocdown
    title: Public Rocdown site configuration
    author: process:git
    last_modified: 2026-08-19
  - id: bws-sse
    resource: ../research/rocci/basic-webserver-sse-http.md
    title: basic-webserver 0.16 SSE and HTTP limits
    author: process:cursor
    last_modified: 2026-08-21
---

# Known Rocci limitations

## Snapshot date

2026-08-21.

## Static documentation

Rocdown site builds reject pages containing `@render`, Roc blocks, Rocci templates, handlers, file CSS, or custom layouts; the dynamic-island splice path is not implemented. This includes document-root `<Tag>` islands because Rocdown classifies them as Rocci template items before applying its static feature gate. `:kind` article blocks are allowed on static pages.[^rocdown-site][^rocdown-article]

Cmd/Ctrl-K fuzzy page navigation ships on Rocdown sites, rocci.dev, OKF review HTML, and desktop preview. It ranks `pages.json` / `catalog.json` titles and paths and swaps already-rendered HTML; it is not full-text search.[^goto-js][^fuzzy-plan] Full-text documentation-site search, clean per-page Markdown artifacts, and some machine-output polish remain in the ordinary Rocdown backlog. Markdown and search text functions already exist for `:kind` article nodes so those outputs stay honest when they land. The separate OKF knowledge path emits a heading-chunk search index, supports filtered CLI search, and measures a fixed lexical retrieval benchmark; that does not add a full-text search interface to ordinary generated documentation sites. Watch/serve, aliases, and live reload are already implemented, and the public status page reflects that boundary.[^roadmap-plan][^status-doc][^okf]

## Runtime and desktop delivery

Authored Roc apps can be wrapped with `rocci bundle` into a local, ad-hoc-signed macOS `.app`. Production signing, notarization, update delivery, Windows and Linux installers, tray and deep-link integration, and full platform CI remain absent.[^roadmap]

The desktop host exposes the current window/webview boundary but not general native capabilities such as dialogs, filesystem access, or notifications. Multi-window application lifecycle is also not connected to authored Roc apps.[^roadmap]

Pinned **basic-webserver 0.16** still logs opaque HTTP/1.1 Body-stream errors on client abort of an open SSE, and plaintext `rocci run` stays on HTTP/1.1 (browsers do not use cleartext HTTP/2). Generated `@get:live` keepalives and empty-SSE `@method:command` responses work around the 30s silent-`Wait` idle timeout and Safari 204 Preview noise; ordinary command callers receive 204 instead of JSON. Rocci does not fork the platform. Details: [basic-webserver SSE and HTTP](/research/rocci/basic-webserver-sse-http.md).[^bws-sse]

## Language and client behavior

There is no implemented `@island` construct. Rich browser-owned behavior therefore remains an explicit future boundary rather than a capability authors can rely on today. Documentation tabs ship as stacked no-JS sections; tab persistence JavaScript is not shipped. The tabs parent painter receives concatenated Html even though the dispatcher builds typed child records; a site cannot yet compose a `tablist` from labels as data. Custom static kinds inferred from a theme pack cannot declare exclusive child policy (`accepts`); `@block` is not shipped, so helpers must not live in the block pack.[^roadmap][^site-ref]

## Validation

Review this record when a cited source changes or on its `stale_after` date. The published status page is supporting evidence, not final authority where current code or the active implementation plan differs.

[^roadmap]: Current deliberate limitations and unchecked roadmap items.
[^status-doc]: Published audience-facing limitations after the Phase 6 stale-status correction.
[^rocdown-site]: Static-page feature rejection in the current site loader.
[^rocdown-article]: Exact static-item allowlist and Rocci-template rejection.
[^roadmap-plan]: Current Rocdown refactor plan and remaining outputs.
[^okf]: Current local search and machine-output support in portable OKF engine.
[^goto-js]: Shared Cmd/Ctrl-K palette and History-API HTML swap.
[^fuzzy-plan]: Document navigation versus full-text search boundary.
[^site-ref]: Pack-inferred custom kinds default to any children; helpers must not live in the pack.
[^bws-sse]: Host idle timeout, HTTP/1.1 on plaintext run, and disconnect log noise; Rocci keepalives and empty SSE are workarounds.
