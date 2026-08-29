---
type: Research Report
title: Untagged style siblings on Datastar patch-elements
description: "Colocated @css embeds a sibling style with no id; Datastar patch-elements then logs PatchElementsNoTargetsFound on every live tick and one-shot fragment. The 2026-08-30 fix strips those tags in Datastar.patch_elements. Recommend wire-format SSE target tests, not browser e2e in the default suite."
tags: [domain/rocci, domain/runtime, domain/site, integration/datastar, concern/rendering, concern/testing, concern/validation]
status: draft
generated: { by: process:cursor, at: 2026-08-30T00:20:00Z }
stale_after: 2026-11-30
authority: exploratory
owners: [human:nils]
sources:
  - id: datastar-roc
    resource: ../../../crates/rocci-cli/runtime/Datastar.roc
    title: Datastar.patch_elements now drops style tags
    author: process:git
    last_modified: 2026-08-30
  - id: strip-rs
    resource: ../../../crates/rocci-datastar/src/sse/events.rs
    title: Rust strip_style_elements and PatchElements formatter
    author: process:git
    last_modified: 2026-08-30
  - id: sse-tests
    resource: ../../../crates/rocci-datastar/tests/sse.rs
    title: Unit tests for style strip
    author: process:git
    last_modified: 2026-08-30
  - id: lower-rs
    resource: ../../../crates/rocci-template/src/lower.rs
    title: embed_css wraps fragment components in a style sibling
    author: process:git
    last_modified: 2026-08-30
  - id: dispatch-rs
    resource: ../../../crates/rocci-cli/src/dispatch.rs
    title: Live poll and one-shot patch_html call Datastar.patch_elements
    author: process:git
    last_modified: 2026-08-30
  - id: live-counter
    resource: ../../../examples/rocci/standalone/live-counter/LiveCounter.rocci
    title: GET /sse returns LiveSlice
    author: process:git
    last_modified: 2026-08-30
  - id: live-counter-ui
    resource: ../../../examples/rocci/standalone/live-counter/LiveCounterUi.rocci
    title: CounterCount and CounterFeed colocate @css; LiveSlice is those two roots
    author: process:git
    last_modified: 2026-08-30
  - id: site-home
    resource: ../../../site/index.rocdown
    title: Production home splices CounterIsland and opens GET /sse
    author: process:git
    last_modified: 2026-08-30
  - id: islands-rs
    resource: ../../../crates/rocci-rocdown/src/islands.rs
    title: Site islands lower with embed_css false
    author: process:git
    last_modified: 2026-08-30
  - id: css-doc
    resource: ../../../docs/reference/language/css.rocdown
    title: Public CSS page says patches must not carry sibling style
    author: process:git
    last_modified: 2026-08-30
  - id: stack-skill
    resource: ../../../.agents/skills/rocci-stack/SKILL.md
    title: CSS does not own riding style on SSE patches
    author: process:git
    last_modified: 2026-08-30
  - id: compile-css
    resource: ../../../crates/rocci-template/tests/compile.rs
    title: embed_css true injects style; false keeps artifacts only
    author: process:git
    last_modified: 2026-08-30
  - id: bws-sse
    resource: basic-webserver-sse-http.md
    title: Platform SSE framing is not Datastar-aware
    author: process:cursor
    last_modified: 2026-08-21
  - id: cqrs
    resource: datastar-cqrs-action-responses.md
    title: Generated live poll and one-shot patch_html
    author: process:cursor
    last_modified: 2026-08-21
  - id: test-plan
    resource: ../../plans/rocci/sse-patch-target-tests.md
    title: Paired plan for SSE patch-target tests
    author: process:cursor
    last_modified: 2026-08-30
  - id: ds-error
    resource: https://data-star.dev/errors/patch_elements_no_targets_found
    title: Datastar PatchElementsNoTargetsFound
    author: organization:star-federation
---

# Untagged style siblings on Datastar patch-elements

## Claim

This is a **Rocci transport bug**, not an authoring mistake. Colocated
`@css` on a fragment component is the intended model. With
`embed_css: true` (standalone default), lowering emits that CSS as a
sibling `<style>` with no `id`. Datastar `datastar-patch-elements` morphs
top-level nodes by `id` or an explicit `selector`. A style tag matches
neither, so the client logs `PatchElementsNoTargetsFound` on every
changed live tick and on one-shot fragment patches.[^lower-rs][^ds-error][^css-doc][^stack-skill]

The count still updated because `#counter` and `#counter-feed` were valid
targets. The console spam was the style siblings riding the same
event.[^live-counter-ui]

## What operators saw

On the first-level staging live-counter host, DevTools showed a
long-lived `GET /sse?datastar=%7B%7D` and repeating

`Error: PatchElementsNoTargetsFound`

from the `datastar-patch-elements` watcher, with an empty element
object. The counter itself worked. A separate Chrome warning (Place
`<select>` without `id` or `name`) is autofill hygiene, not this
bug.[^live-counter]

The same live card is composed on the production home page: a CDN
snapshot of `CounterIsland` plus `data-init=@get("/sse", …)` against the
site islands service, whose live handler is the same
`LiveCounter.rocci` `LiveSlice`.[^site-home][^live-counter]

Site islands lower with `embed_css: false`, so that path does not inject
a style sibling today. Standalone `rocci run` and the live-counter
example image use the default `embed_css: true`. If production still
logged the same watcher error, it was the same event class (untagged
top-level patch nodes) from an older image, a standalone-compiled
service, or another untagged root — not a distinct home-page
bug.[^islands-rs][^compile-css]

## Mechanism

1. `CounterCount` and `CounterFeed` each have `@css` plus a stable-id
   root. `LiveSlice` returns both. That is correct CQRS: one stream, two
   morph targets; controls stay off the stream.[^live-counter-ui][^cqrs]
2. `lower_html_value_with_style` for a non-document component emits a
   fragment whose first child is a `style` element and whose second
   child is the id'd root.[^lower-rs]
3. Generated `@get:live` polls every 100 ms, compares `Html.render`, and
   on change calls `Datastar.patch_elements`. Relative "ago" text changes
   often, so the error repeats. One-shot `patch_html!` uses the same
   helper.[^dispatch-rs]
4. Docs and the stack skill already forbade sibling `<style>` on
   patches. Nothing in the runtime enforced it, and no test read the SSE
   `elements` payload.[^css-doc][^stack-skill][^sse-tests]

Moving `@css` off the morph targets would hide the symptom in one app
and teach authors a false constraint. The generic fix is transport
policy.[^stack-skill]

## Fix (2026-08-30)

`Datastar.patch_elements` drops `<style>…</style>` after render, before
the SSE `elements` field. The Rust `PatchElements` formatter does the
same. First-paint HTML still embeds styles on standalone documents.
Authors keep `@css` on live fragments.[^datastar-roc][^strip-rs][^sse-tests]

Commit on `live-examples-play-path`: `b0affe5d`. Staging and production
keep the old image until that revision is promoted.

basic-webserver `Sse.Event` cannot validate this. It frames bytes. It
does not parse HTML or see the client DOM. "The target exists" is a
browser fact. The check that *is* possible without a document — selector
present, or every top-level node has an `id` — belongs on
`Datastar.patch_elements`, not `Sse.keyed`.[^bws-sse]

## Should we test the SSE feed?

**Yes, at the wire format. No, not as a browser Datastar session in the
default suite.**

What would have caught this before shipping:

- A fixture HTML string
  `<style>…</style><section id="counter">3</section>` run through
  `strip_style_elements` / `PatchElements::format_sse`, asserting the
  event has no untagged top-level node.
- A compile fixture with `@css` on a fragment, proving lowering still
  emits the style sibling (so the strip stays load-bearing).
- Optionally, `GET /sse` against a built live-counter, parse the first
  `datastar-patch-elements` event, apply the same target rule.

What would not have been worth the default `<2s` suite:

- Driving Datastar in Chromium against staging.
- Asserting "the element exists in a live document" (needs a browser or
  a fake DOM).
- Putting Datastar policy in `rocci-template` parser tests.

The strip unit tests in `rocci-datastar` are the start of that floor,
not the whole contract. The paired plan designs the rest: a shared
target-rule helper, a lowering regression that the sibling still
exists, and an optional ignored HTTP smoke. It does not start until
asked.[^test-plan][^sse-tests]

## Disposition

Exploratory incident record. The strip is implemented in this revision;
promotion and the test plan are not. Do not treat this file as a
decision to change `@css` or `embed_css`.

[^datastar-roc]: Runtime `patch_elements` wraps render with `drop_style_elements`.
[^strip-rs]: Rust `strip_style_elements` and the SSE formatter share the policy.
[^sse-tests]: `rocci-datastar` tests cover strip, not a live `/sse` body.
[^lower-rs]: Fragment `embed_css` injects a style sibling before the markup root.
[^dispatch-rs]: Live unfold and `patch_html!` both call `Datastar.patch_elements`.
[^live-counter]: `@get:live("/sse")` returns `LiveSlice`.
[^live-counter-ui]: Count and feed own `@css` and the morph ids; `LiveSlice` is those two calls.
[^site-home]: Home `@render`s `CounterIsland` and authors `data-init=@get("/sse", …)`.
[^islands-rs]: Island and service compile options set `embed_css: false`.
[^css-doc]: Language CSS page already said patches must not carry sibling style.
[^stack-skill]: CSS layer does not own riding style on SSE; transport is Datastar.
[^compile-css]: Compile tests pin style injection only when `embed_css` is true.
[^bws-sse]: Platform `Sse.Event` frames fields; it does not parse HTML.
[^cqrs]: Generated live poll plus one-shot fragment patches.
[^test-plan]: Paired implementation plan for wire-format target tests.
[^ds-error]: Datastar documents `PatchElementsNoTargetsFound` when no id or selector matches.
