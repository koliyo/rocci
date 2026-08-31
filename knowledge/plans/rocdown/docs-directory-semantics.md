---
type: Implementation Plan
title: Make directory plus index the Rocdown site section
description: Phases 0-6 shipped peel-by-id, RD2205, Contributor/Appendix indexes, reserved Overview first child, equal sibling indent, and the public cookbook.
tags: [domain/rocdown, concern/publication, concern/developer-experience, concern/navigation, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-08-31T11:20:00Z }
stale_after: 2026-11-29
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../../research/rocdown/docs-directory-semantics.md
    title: Directory plus index is the Rocdown site section
    author: process:cursor
    last_modified: 2026-08-31
  - id: plan-rs
    resource: ../../../crates/rocci-rocdown/src/plan.rs
    title: Sidebar forest, peel-by-id, and nested fold
    author: process:git
    last_modified: 2026-08-31
  - id: catalog
    resource: ../../../crates/rocci-rocdown/src/catalog.rs
    title: Explicit navigation resolve, RD2202, and RD2205
    author: process:git
    last_modified: 2026-08-31
  - id: nav-list
    resource: ../../../crates/rocci-ui/templates/chrome/NavList.rocci
    title: Shared expandable sidebar renderer
    author: process:git
    last_modified: 2026-08-31
  - id: inventory
    resource: ../../../docs/inventory.toml
    title: Docs inventory including directory_semantics
    author: process:git
    last_modified: 2026-08-31
  - id: sites-ref
    resource: ../../../docs/rocdown/sites.rocdown
    title: Published Rocdown site configuration reference
    author: process:git
    last_modified: 2026-08-31
  - id: docs-nav
    resource: ../../../docs/rocdown.toml
    title: Standalone documentation navigation
    author: process:git
    last_modified: 2026-08-31
  - id: site-nav
    resource: ../../../site/rocdown.toml
    title: Unified rocci.dev navigation
    author: process:git
    last_modified: 2026-08-31
  - id: docs-index
    resource: ../../../docs/index.rocdown
    title: Docs portal titled Overview
    author: process:git
    last_modified: 2026-08-22
  - id: contributor-checklist
    resource: ../../../docs/reference/contributor/checklist.rocdown
    title: Documentation contributor checklist
    author: process:git
    last_modified: 2026-08-25
  - id: stack-plan
    resource: ../site/rocci-dev-docs-stack-ia.md
    title: Stack-first docs implementation plan
    author: process:cursor
    last_modified: 2026-08-31
  - id: compiler-arch
    resource: ../../architecture/rocdown-documentation-compiler.md
    title: Rocdown documentation generator architecture
    author: process:cursor
    last_modified: 2026-08-31
  - id: catalog-shell
    resource: ../../decisions/rust-catalog-rocci-shell.md
    title: Rust catalog and Rocci documentation shell
    author: process:okf-migration
    last_modified: 2026-08-24
---

# Make directory plus index the Rocdown site section

## Purpose and authority

This plan executes the [directory-plus-index research](/research/rocdown/docs-directory-semantics.md).
A directory with `index.rocdown` is a section; a nested index is a named
subsection; an indexless directory is not a section. After Phases 0–6,
the first sidebar child is **Overview** and same-level members share one
indent.[^research]

It does not reopen the stack-first group labels or URL cut. Those stay
Start / Templates / Applications / Rocdown / Reference /
Troubleshooting.[^stack-plan]

The record is exploratory. Writing it does not start a phase.

## Goal

Give documentation authors one filesystem rule they can rely on:

1. a directory with `index.rocdown` is a section;
2. the fold heading names the section and may link to the landing;
3. the first child of every section and subsection with an index is
   **Overview** (sidebar label only; `@page.meta.title` and the H1 stay
   the document title);
4. same-level members (Overview, nested folds, leaf pages) share one
   indent;
5. a nested `index.rocdown` is a named subsection with the same
   Overview-first rule inside the fold;
6. two or more listed pages in a directory without an index warn
   `RD2205`;
7. `docs/` and `site/` follow that rule (Contributor and Appendix
   landings already shipped);
8. `docs/rocdown/sites.rocdown` and the contributor checklist teach the
   Overview-first row, not heading-as-landing-only.

## Out of bound

- Changing `.rocci` or `.rocdown` grammar, lowering, or runtime.
- Replacing explicit `[[nav]]` with filesystem-only autogeneration.
- Adding `_category_.json` or `[[nav.groups]]` for every nested folder.
- Printing `INDEX` or repeating the section title as the first child
  (`Reference` / `Reference`).
- Recursive sidebar depth beyond group → subsection → pages.
- Reopening stack-IA labels, academy-chrome bans, or clean-cut URLs.
- Interpreting `.rocci` in Rust to build navigation.
- Teaching OKF collections as a Rocdown feature.
- Making `RD2205` an error in v1 (warning only, so existing third-party
  sites keep checking).

## Constraints that do not move

| Constraint | Required behavior |
| --- | --- |
| Two layers | Rust catalog lists pages and nav ids. Planner builds the sidebar forest. Rocci `NavList` only renders.[^catalog-shell][^nav-list] |
| Explicit inclusion | `[[nav]]` / `[[nav.groups]]` still own order and listing. `RD2202` stays for unlisted authored pages.[^catalog] |
| Index owns the directory URL | `guides/index` → `/guides/`. Do not invent a second route for the landing.[^sites-ref] |
| Peel by id | Group-root `index` still sets `group.href` even when the page title is Overview and the label is Start.[^research] |
| Overview first child | After peel, insert a first row titled Overview pointing at that href. Nested folds do the same.[^research] |
| Equal sibling indent | Leaves and subsection headings under the same parent share one indent. Extra indent is only inside a nested fold.[^nav-list] |
| Nested fold | `dir/nested/index` titles the subsection from the page title and collects listed descendants.[^plan-rs] |
| Depth cap | Visible nesting stays two levels. Deeper indexes flatten as today.[^plan-rs] |
| Warning, not error | Indexless clusters of two or more listed pages warn. Builds still succeed. |
| Docs format | Authored pages stay `.rocdown`. Knowledge stays inert Markdown. |
| Build safety | Failed site builds keep the previous output tree. |

## Target authoring model

```text
docs/
  index.rocdown                      # Start landing (H1 may stay Overview)
  install.rocdown
  appendix/
    index.rocdown                    # shipped; sidebar first child Overview
    glossary.rocdown
  templates/index.rocdown            # fold heading Templates; first child Overview
  reference/
    index.rocdown
    language/index.rocdown           # named subsection; first child Overview
    contributor/index.rocdown        # shipped
    contributor/checklist.rocdown
```

`[[nav]]` lists the landing first, then members, with each nested index
immediately before its members. Site toml uses the `docs/` prefix. The
generator inserts the Overview row; authors do not add a second
`overview.rocdown` unless they want a distinct member page.[^docs-nav][^site-nav]

## Phases

### Phase 0 — freeze the contract (implemented)

Implemented on `docs-directory-semantics` as `be4cfeb4`. Inventory
`[directory_semantics]` records peel-by-id, heading-as-landing, `RD2205`,
and an empty `indexless_clusters` list.[^inventory]

### Phase 1 — planner forest and diagnostics (implemented)

Implemented as `8d05f03b`. Peel-by-id replaced title-equality peel.
`RD2205` warns on listed indexless clusters. Tests lock heading-as-landing
(no Overview child after peel).[^plan-rs][^catalog]

### Phase 2 — docs tree follows the contract (implemented)

Implemented as `dfe44e51`. `docs/reference/contributor/index.rocdown`
and `docs/appendix/index.rocdown` are listed immediately before their
members in both toml files. `docs/index.rocdown` kept the Overview
document title.[^docs-nav][^site-nav][^docs-index]

### Phase 3 — public docs and architecture pointer (implemented)

Implemented as `e49192d8`. `sites.rocdown` names peel-by-id and
`RD2205`. The contributor checklist requires an index for a new listed
directory. The architecture record states the forest comes from listed
ids plus index files, not title equality.[^sites-ref][^contributor-checklist][^compiler-arch]

### Phase 4 — verify the reading paths (implemented)

Verify-only. `check site` and `build site` passed (112 pages). Start
heading links to `/docs/` with no Overview child; Language and
Contributor are named folds. That missing Overview child is the Phase 5
input, not a Phase 4 miss against the then-Bound.

### Phase 5 — Overview first child and equal sibling indent (implemented)

Implemented as `c746bcd1`. After peel-by-id, every real fold gets a first
`items` row titled Overview. Same-level leaves stay in `items`; only
nested folds go in `children`. FAQ remains a single-page group.
`NavList` extra-indents only links inside a nested fold.

**Bound**

- Keep peel-by-id: the group-root index still sets `group.href`. After
  peel, insert a first item `{ title: "Overview", href: landing }` for
  every section and nested fold that has an index.[^plan-rs][^research]
- A single-page group that is only an index (FAQ) stays one leaf. Do
  not invent a fold just to host Overview.
- Put same-level leaves in `items` and only real folds in `children` so
  `NavList` does not dump mixed siblings into `children`.[^nav-list]
- In `NavList.rocci`, drop the extra `.nav-fold .nav-fold .nav-child`
  offset on subsection headings relative to sibling leaf links. Extra
  indent is only inside a nested fold.
- Tests in `plan.rs` / `rocci-ui`:
  - Start href `/docs/` and first child titled Overview.
  - Language fold first child Overview, then File structure, …
  - Contributor fold first child Overview.
  - Templates still peels by id and gains an Overview child even when
    the page title equals the label.
  - Explicit nested groups stay inside the parent.
  - `navListNested` (or a sibling fixture) includes a leaf sibling next
    to a fold so indent cannot regress.

**Out of this phase:** rewriting public docs or inventory keys.

**Exit**

```sh
cargo test -p rocci-rocdown --lib plan
cargo test -p rocci-ui --test ui
cargo fmt --all -- --check
```

On `/docs/reference/`, Overview, Language, Runtime, and Contributor
share one indent. Language's Overview sits inside that fold.

### Phase 6 — teach Overview-first-child (implemented)

**Bound**

- Revise the Phase 3 cookbook on the site-configuration page: first
  child is Overview; heading may still link; do not print `INDEX` or
  repeat the section title.[^sites-ref]
- Short note on the Write Rocdown pages guide if it still says the
  index is not a child.
- Checklist stays "new listed directory needs an index"; add that the
  sidebar landing row is Overview.[^contributor-checklist]
- One sentence on
  [Rocdown documentation generator](/architecture/rocdown-documentation-compiler.md):
  landing row is Overview; heading href is still the index. Do not mint
  a new Decision.[^compiler-arch]
- Update `docs/inventory.toml` `[directory_semantics]`: keep
  `peel_by = "id"`; replace heading-as-landing-only with Overview first
  child (key name is local to the inventory).[^inventory]

**Exit**

```sh
cargo run -q -p rocci-rocdown-cli -- check docs
cargo run -q -p rocci-rocdown-cli -- check site
okmate check knowledge --profile base --format terminal
```

`sites.rocdown` names Overview as the reserved landing row. Knowledge
check has no new errors.

## Decision gates

Approved before Phase 1 (shipped):

1. Peel-by-id (heading href is `/docs/` for Start).
2. `RD2205` is a warning, not an error.
3. Contributor and Appendix get indexes.

Approved 2026-08-31 before Phase 5:

4. Reserved first child labeled **Overview** (not `INDEX`, not a
   repeated section title). Heading may still link to the same URL.
5. Same-level members share one indent.

Not reopened: stack-IA nav labels; no URL aliases; Rust catalog / Rocci
shell.

## Relationship to other records

| Record | After this plan |
| --- | --- |
| [Directory-plus-index research](/research/rocdown/docs-directory-semantics.md) | Evidence, shipped heading-as-landing finding, and Overview-first contract. This plan is the work. |
| [Stack-first docs](/plans/site/rocci-dev-docs-stack-ia.md) | Group labels and corpus moves stay. Directory semantics were out of that plan's Bound. |
| [Rocdown documentation generator](/architecture/rocdown-documentation-compiler.md) | Phase 3 added peel-by-id. Phase 6 adds the Overview-row sentence. |
| [Rust catalog / Rocci shell](/decisions/rust-catalog-rocci-shell.md) | Unchanged ownership. |

## Acceptance

- One public rule: directory + index is a section; first sidebar child
  is Overview; nested index is a named subsection with the same first
  child; same-level members share one indent.
- Start, Templates, Language, and Contributor are instances of that
  rule, not special cases.
- Authors can follow `sites.rocdown` without reading `plan.rs`.
- Existing sites without clustered indexless directories keep a clean
  check; clustered ones get a warning they can fix by adding an index.

Writing a plan is not executing it. Phases 0–6 are implemented on
`docs-directory-semantics`. Do not log complete until CI and Knowledge
succeed.

[^research]: Three observations, peel/fold analysis, shipped heading-as-landing finding, Overview-first rules, rejected INDEX and repeated section title.
[^plan-rs]: Peel-by-id, Overview first child, leaves in `items`, folds in `children`.
[^catalog]: Explicit nav resolve, unlisted `RD2202`, cluster warning `RD2205`.
[^nav-list]: Fold headings as links; extra indent only inside a nested fold.
[^inventory]: `[directory_semantics]` peel-by-id and `overview_first_child`.
[^sites-ref]: Id/route table, peel-by-id, reserved Overview row, `RD2205`.
[^docs-nav]: Start and Reference item lists including Contributor and Appendix indexes.
[^site-nav]: Mounted `docs/` prefixes of the same lists.
[^docs-index]: Portal titled Overview under group Start.
[^contributor-checklist]: Contributor landing and the docs-PR checklist.
[^stack-plan]: Stack-layer groups this plan must not rename.
[^compiler-arch]: Architecture record; Phase 3 peel-by-id sentence is in; Overview row is Phase 6.
[^catalog-shell]: Planner stays in Rust; theme stays a view consumer.
