---
type: Implementation Plan
title: SSE patch-target tests
description: "Add a wire-format rule that every datastar-patch-elements event has a selector or an id on each top-level element. Cover the style-sibling regression in rocci-datastar and a lowering fixture. Optional ignored HTTP GET /sse smoke. No browser Datastar in the default suite."
tags: [domain/rocci, domain/runtime, integration/datastar, concern/testing, concern/validation]
status: draft
generated: { by: process:cursor, at: 2026-08-30T00:20:00Z }
stale_after: 2026-11-30
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../../research/rocci/sse-patch-style-targets.md
    title: "Incident: untagged style siblings on patch-elements"
    author: process:cursor
    last_modified: 2026-08-30
  - id: strip-rs
    resource: ../../../crates/rocci-datastar/src/sse/events.rs
    title: strip_style_elements and PatchElements
    author: process:git
    last_modified: 2026-08-30
  - id: sse-tests
    resource: ../../../crates/rocci-datastar/tests/sse.rs
    title: Existing style-strip unit tests
    author: process:git
    last_modified: 2026-08-30
  - id: datastar-roc
    resource: ../../../crates/rocci-cli/runtime/Datastar.roc
    title: Runtime drop_style_elements
    author: process:git
    last_modified: 2026-08-30
  - id: lower-rs
    resource: ../../../crates/rocci-template/src/lower.rs
    title: Fragment embed_css still emits a style sibling
    author: process:git
    last_modified: 2026-08-30
  - id: compile-rs
    resource: ../../../crates/rocci-template/tests/compile.rs
    title: embed_css compile tests
    author: process:git
    last_modified: 2026-08-30
  - id: dispatch-rs
    resource: ../../../crates/rocci-cli/src/dispatch.rs
    title: Live SSE poll and patch_html
    author: process:git
    last_modified: 2026-08-30
  - id: live-counter
    resource: ../../../examples/rocci/standalone/live-counter/LiveCounter.rocci
    title: Canonical live GET /sse
    author: process:git
    last_modified: 2026-08-30
  - id: agents
    resource: ../../../AGENTS.md
    title: Default suite is sub-second; ignored tests are on demand
    author: process:git
    last_modified: 2026-08-30
  - id: bws-sse
    resource: ../../research/rocci/basic-webserver-sse-http.md
    title: Sse.Event is not Datastar-aware
    author: process:cursor
    last_modified: 2026-08-21
  - id: stack-skill
    resource: ../../../.agents/skills/rocci-stack/SKILL.md
    title: Transport policy stays out of the parser
    author: process:git
    last_modified: 2026-08-30
---

# SSE patch-target tests

The 2026-08-30 style-sibling incident shipped because no test read a
`datastar-patch-elements` `elements` payload. Unit strip tests now exist.
This plan makes the **target rule** explicit and load-bearing.[^research][^sse-tests][^strip-rs]

Exploratory. Do not start a phase until the user asks.

## Goal

CI rejects a `datastar-patch-elements` event that has no `selector` and
at least one top-level element without an `id`. The style-sibling case
is one fixture of that rule. Default tests stay in-process and
sub-second.[^agents]

## Out of bound

- Chromium / Datastar browser sessions in `cargo test` (default or CI
  required).
- Asserting that an id exists in a live document (needs a browser or a
  fake DOM).
- Forking or extending basic-webserver `Sse.Event` with HTML or Datastar
  rules.[^bws-sse]
- Parser, ungram, or `@css` grammar changes.[^stack-skill]
- Changing `embed_css`, hoisting all CSS into `<head>`, or moving `@css`
  off morph targets.
- Requiring Roc inside `rocci-datastar` tests.
- Production promote, Launch advertising, or origin smoke as a phase
  exit.

## Constraints that do not move

1. **Wire format, not DOM.** The rule is: selector field, or every
   top-level element has `id`. "Exists in the tab" stays client-side.
   [^research]
2. **Transport owns the policy.** `Datastar.patch_elements` / Rust
   `PatchElements` stay the implementation; the parser does not grow
   SSE awareness.[^stack-skill][^datastar-roc]
3. **Keepalives are not patches.** Empty `Sse.Event.data("")` must not
   be parsed as `datastar-patch-elements`.[^dispatch-rs]
4. **Selector modes keep working.** Append/prepend with `selector` may
   send nodes that have no id.[^research]
5. **Lowering still embeds a fragment style sibling** when `embed_css: true`.
   The strip remains the fix; tests must notice if someone "fixes"
   lowering by deleting the sibling and calling the incident closed
   without transport coverage.[^lower-rs][^compile-rs]
6. **Default suite stays fast.** Roc-backed HTTP belongs behind
   `#[ignore]` or `ROCCI_REQUIRE_ROC=1`.[^agents]

## Phase 1 — Target-rule helper

Bound:

- Add `patch_elements_targets_ok(html, selector: Option<&str>) -> bool`
  (name may match crate style) next to `strip_style_elements`.
- Top-level scan is good enough: split on sibling elements after strip.
  Do not take a full HTML5 parser dependency.
- `selector: Some(_)` is always ok (including empty elements).
- After strip, every remaining top-level tag must have an `id`
  attribute. A `style` tag without an id fails before strip, passes after.
- Unclosed style tags are out of contract; do not invent recovery beyond
  today's strip.
- Tests in `crates/rocci-datastar/tests/sse.rs`: style+id fixture
  fails before strip and passes after; two id roots pass; a bare
  `div` with only a class fails; `selector: Some("#host")` plus a bare `tr`
  passes.

**Exit:** `cargo test -p rocci-datastar` and
`cargo fmt --all -- --check`.

## Phase 2 — Lowering regression

Bound:

- In `crates/rocci-template/tests/compile.rs`, a fragment `@component`
  with `@css` and `embed_css: true` still emits `"style"` in generated
  Roc (already true for file+component CSS; pin a **non-document**
  fragment so the sibling path cannot regress silently).
- `embed_css: false` still has no `"style"` in Roc.
- Do not call Datastar from this crate.

**Exit:** `cargo test -p rocci-template` and
`cargo fmt --all -- --check`.

## Phase 3 — Optional live HTTP smoke

Bound:

- One `#[ignore]` test (cli or a small ops helper) that starts or
  assumes live-counter, `GET /sse`, reads until the first
  `event: datastar-patch-elements`, joins `data: elements` lines, and
  asserts `patch_elements_targets_ok` after the same strip the runtime
  uses.[^live-counter]
- Skip or ignore when Roc is missing. Do not add
  `ROCCI_REQUIRE_ROC=1` to the default workspace suite.
- Empty keepalive frames are ignored, not failed.
- Do not hit staging or production.

**Exit:**
`cargo test -p rocci-cli --test <name> -- --ignored --nocapture`
documented in the crate README or AGENTS "on demand" list. Default
`cargo test -p rocci-datastar` still green without Roc.

## Phase 4 — Point the public contract

Bound:

- One sentence on the CSS language page or the streams concept page:
  CI checks patch top-level ids (or selector), not a live DOM.
- Do not duplicate the incident narrative.

**Exit:** `okmate check knowledge --profile base`. Public doc change is
in the same commit as the sentence.

## Tests

Phases 1–2 are the required floor. Phase 3 is optional proof that a
real `/sse` body matches the helper. Phase 4 is documentation only.

[^research]: Incident record for the style-sibling `PatchElementsNoTargetsFound` spam.
[^strip-rs]: `strip_style_elements` is the current transport fix, not the target rule.
[^sse-tests]: Existing tests cover strip only.
[^datastar-roc]: Roc `patch_elements` is the runtime owner of the strip.
[^lower-rs]: Fragment lowering still emits the style sibling when embedding CSS.
[^compile-rs]: Template compile tests already distinguish `embed_css` true vs false.
[^dispatch-rs]: Live keepalives are empty data events, not patch-elements.
[^live-counter]: Canonical live app for an optional HTTP smoke.
[^agents]: Default workspace tests stay sub-second; ignored tests are on demand.
[^bws-sse]: Platform SSE helpers must stay Datastar-free.
[^stack-skill]: SSE and CQRS policy stay out of the `.rocci` parser.
