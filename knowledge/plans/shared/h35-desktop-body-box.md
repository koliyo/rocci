---
type: Implementation Plan
title: Tested desktop host pins and a document-box authoring rule
description: Keep the h35-desktop split. State the file CSS :scope rule and pin each product to a tested compatible host revision. The original HEAD rule was superseded.
tags: [domain/rocci, domain/desktop, domain/okf, concern/architecture, concern/ui]
status: draft
generated: { by: process:cursor, at: 2026-09-23T12:55:04Z }
stale_after: 2026-12-23
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../../research/shared/h35-desktop-body-box.md
    title: Desktop host body box and file-scoped :scope
    author: process:cursor
    last_modified: 2026-09-23
  - id: desktop-readme
    resource: ../../../crates/rocci-desktop/README.md
    title: rocci-desktop facade contract
    author: process:git
    last_modified: 2026-09-11
  - id: author-skill
    resource: ../../../.agents/skills/rocci-author/SKILL.md
    title: Rocci author skill
    author: process:git
    last_modified: 2026-09-11
  - id: rocci-pin
    resource: ../../../Cargo.toml
    title: Rocci h35-desktop pin
    author: process:git
    last_modified: 2026-09-11
  - id: okmate-pin
    resource: ../../../../okmate/Cargo.toml
    title: Okmate h35-desktop pin
    author: process:git
    last_modified: 2026-09-11
  - id: okmate-desktop
    resource: ../../../../okmate/src/desktop.rs
    title: Okmate preview host call
    author: process:git
    last_modified: 2026-09-11
---

# Tested desktop host pins and a document-box authoring rule

## Goal

Keep `h35-desktop` as the shared window host. Make the document-box
rule visible to theme authors, and pin each product to a tested
compatible `h35-desktop` commit.[^research]

## Out of bound

- Changing `@scope` so only the component root is a scope root.
- Moving toolbar chrome back into Rocci or Okmate.
- Changing the host `html` grid.
- Re-doing the local `SiteShell` / Rocdown theme selector narrowing.
  That repair is already in this working tree and is evidence in the
  research record, not a phase here.[^research]
- Enabling tab shortcuts in Rocci. Okmate may keep them on.[^okmate-desktop]
- Floating the Cargo dependency on a branch name. Each pin stays an
  exact commit rev and advances after that product's compatibility check.

## Constraints that do not move

1. `h35-desktop` owns the window and toolbar row. `rocci-desktop` stays
   a facade. Okmate keeps calling `preview` directly.[^desktop-readme][^okmate-desktop]
2. Product shells fill `body`. They do not size with `100vh` or
   `100dvh`.[^desktop-readme]
3. File `@css` keeps stamping `data-rocci-css` on each element from
   that file.[^research]
4. Okmate's goto stays off. Its tab-shortcut flag stays an explicit
   host option, not a Rocci default.[^okmate-desktop]
5. Rocci and Okmate each pin a tested `h35-desktop` commit. The pins
   may differ while a product rolls forward.[^rocci-pin][^okmate-pin]

## Phase 1 — Authoring rule

**Bound**

- Add one sentence to the `rocci-desktop` README next to the existing
  "fill `body`" rule: bare `:scope` in file `@css` matches every
  stamped element; document height belongs on `html` / `body` or
  `:scope:is(html)`.
- Add the same sentence to the Rocci author skill where `@css` is
  described.[^author-skill]
- Do not change lowering or host CSS.

**Exit:** both documents state the rule; `cargo test -p rocci-desktop`
still passes if the README assertion covers the new sentence.

## Phase 2 — Pin both products to a tested host commit

**Bound**

- At initial drafting Rocci was on `57fbbec` and Okmate on `9e38d99`.
  Both later moved to the combined `c465c34` revision. The split audit
  replaced the always-HEAD rule with a tested-compatible-revision rule.
  This local implementation pins both to the published `85676e8` after
  targeted checks. Host CI passed on macOS and Linux (`35862994113`);
  product changes have no hosted CI result yet.[^rocci-pin][^okmate-pin]
- Keep `rev = "<commit>"` in each `Cargo.toml`. Update a product's rev
  when its compatibility checks pass, without requiring the other
  product to move at the same time.
- Rocci change lands in this repository. The Okmate pin change lands
  in the sibling Okmate repository.
- Confirm Rocci still builds `rocci-desktop` without passing
  `tab_shortcuts`. Confirm Okmate's `tab_shortcuts: true` still
  typechecks.
- Smoke the Rocci site preview and an Okmate `view` after the bump.
  Header stays one toolbar row plus the product header. The skip link
  stays off-screen.

**Exit:** each `Cargo.toml` pin resolves a published, tested commit;
desktop feature checks pass; one manual preview of each product shows
a normal page. Hosted CI and Knowledge workflows must pass before the
phase is logged complete.

[^research]: Split stays; blue bar was file-scoped `:scope` height; pins diverged after `0bee21a`.
[^desktop-readme]: Facade fills `body`; host toolbar is a layout row.
[^okmate-desktop]: Direct `preview` call; `goto: false`; `tab_shortcuts: true`.
[^okmate-pin]: The current Okmate manifest pins `85676e8`; `9e38d9915a848b0f4f92a28a58265e8ac13a9743` was the initial plan snapshot.
[^rocci-pin]: The current Rocci manifest pins `85676e8`; `57fbbec766f8272737c2c8265d15db3ba63033c2` was the initial plan snapshot.
[^author-skill]: Author skill is the place `@css` guidance lives.
