---
type: Implementation Plan
title: Make directory plus index the Rocdown site section
description: Phases 0-4 shipped peel-by-id heading-as-landing, RD2205, and Contributor/Appendix indexes. Remaining work inserts a reserved first child Overview and equal sibling indent in NavList, then teaches that contract.
tags: [domain/rocdown, concern/publication, concern/developer-experience, concern/navigation, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-08-31T09:20:00Z }
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
    last_modified: 2026-08-25
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
It makes the sidebar forest match the file tree authors already write:
`index.rocdown` is the section landing; a nested index is a named
subsection; an indexless directory is not a section.[^research]

It does not reopen the stack-first group labels or URL cut. Those stay
Start / Templates / Applications / Rocdown / Reference /
Troubleshooting.[^stack-plan]

The record is exploratory. Writing it does not start a phase.

## Goal

Give documentation authors one filesystem rule they can rely on:

1. a directory with `index.rocdown` is a section whose heading links to
   that landing;
2. a nested `index.rocdown` is a named subsection;
3. two or more listed pages in a directory without an index warn;
4. `docs/` and `site/` follow that rule (Contributor, and Appendix if
   kept as a cluster);
5. `docs/rocdown/sites.rocdown` and the contributor checklist teach it.

## Out of bound

- Changing `.rocci` or `.rocdown` grammar, lowering, or runtime.
- Replacing explicit `[[nav]]` with filesystem-only autogeneration.
- Adding `_category_.json`, reserved Overview children, or
  `[[nav.groups]]` for every nested folder.
- Recursive sidebar depth beyond group → subsection → pages.
- Reopening stack-IA labels, academy-chrome bans, or clean-cut URLs.
- Interpreting `.rocci` in Rust to build navigation.
- Teaching OKF collections as a Rocdown feature.
- Making the new indexless-cluster diagnostic an error in v1 (warning
  only, so existing third-party sites keep checking).

## Constraints that do not move

| Constraint | Required behavior |
| --- | --- |
| Two layers | Rust catalog lists pages and nav ids. Planner builds the sidebar forest. Rocci `NavList` only renders.[^catalog-shell] |
| Explicit inclusion | `[[nav]]` / `[[nav.groups]]` still own order and listing. `RD2202` stays for unlisted authored pages.[^catalog] |
| Index owns the directory URL | `guides/index` → `/guides/`. Do not invent a second route for the landing.[^sites-ref] |
| Peel by id | Group-root `index` is the heading href even when the page title is Overview and the label is Start.[^research] |
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
    index.rocdown                    # optional; required if Appendix is a subsection
    glossary.rocdown
  templates/index.rocdown            # heading-as-landing (already)
  reference/
    index.rocdown
    language/index.rocdown           # named subsection (already)
    contributor/index.rocdown        # add
    contributor/checklist.rocdown
```

`[[nav]]` lists the landing first, then members, with each nested index
immediately before its members. Site toml uses the `docs/` prefix.[^docs-nav][^site-nav]

## Phases

### Phase 0 — freeze the contract

**Bound**

- Record the eight rules from the research in `docs/inventory.toml` under
  a `[directory_semantics]` table (or equivalent comments plus keys):
  directory+index, heading-as-landing, peel-by-id, nested fold, no-index
  flat, cluster warning, explicit nav, depth cap.
- Do not change planner behavior or public pages.

**Exit**

- Inventory states peel-by-id and the contributor/appendix cluster list.
- `cargo test -p rocci-rocdown plan::` still matches today's
  title-matching peel (Phase 1 changes that).

### Phase 1 — planner forest and diagnostics

**Bound**

- Replace title-equality `peel_matching_index` with peel-by-id: if the
  first listed item is the group's root `index` (`section_root_dir` or
  a lone `index` / `{prefix}/index`) and there is more than one item,
  peel it into `group.href`.[^plan-rs]
- Keep `is_fold_index` nested folds and `flatten_group_depth`.
- Add catalog warning `RD2205` (code may shift if 2205 is taken): two or
  more listed, non-draft pages share a directory that has no listed
  `…/index` page. Message names the directory and the page ids.
- Tests in `plan.rs` / `catalog.rs`:
  - Start + Overview title peels to href `/docs/` and does not keep an
    Overview child.
  - Language fold still nests descendants and opens ancestors.
  - Appendix / contributor without index stay flat and emit `RD2205`.
  - Single-page FAQ lane unchanged.
  - Templates-style title==label peel still attaches the heading.

**Out of this phase:** adding `contributor/index.rocdown`, rewriting
public docs.

**Exit**

```sh
cargo test -p rocci-rocdown --lib plan
cargo test -p rocci-rocdown --lib catalog
cargo fmt --all -- --check
```

`appendix_without_index_stays_flat` (or its replacement) expects three
member rows and Start href `/docs/` when the first item is `docs/index`
titled Overview.

### Phase 2 — docs tree follows the contract

**Bound**

- Add `docs/reference/contributor/index.rocdown`: short landing, title
  Contributor (or Contributor appendix), links to the three existing
  pages. No academy chrome.
- List `reference/contributor/index` (and `docs/reference/contributor/index`
  on the site) immediately before the contributor members.[^docs-nav][^site-nav]
- Add `docs/appendix/index.rocdown` titled Appendix and list it before
  the three primers, **or** document in the inventory that appendix stays
  a flat Start cluster and accept `RD2205` there. Default: add the index
  so Start matches the cookbook.
- Do not retitle `docs/index.rocdown` unless the H1 Overview becomes
  confusing after peel; the page may keep Overview as the document
  title.[^docs-index]

**Exit**

```sh
cargo run -q -p rocci-rocdown-cli -- check docs
cargo run -q -p rocci-rocdown-cli -- check site
cargo run -q -p rocci-rocdown-cli -- inspect nav docs
```

Inspected Reference items include `reference/contributor/index` first in
that cluster. `RD2205` is absent for contributor (and for appendix if
the index was added). `RD2202` is not newly introduced.

### Phase 3 — public docs and architecture pointer

**Bound**

- Expand the published Rocdown site-configuration page Navigation with the
  cookbook: id/route table (already there), then directory+index,
  heading-as-landing, nested subsection, indexless cluster warning, depth
  cap, and when to use `[[nav.groups]]` versus an index file.[^sites-ref]
- One short section on the Write Rocdown pages guide: catalog sites omit
  `route`; `index.rocdown` is the section landing.
- Update the documentation contributor checklist: a new directory of
  listed pages needs an index in the same change.[^contributor-checklist]
- Add one descriptive paragraph to
  [Rocdown documentation generator](/architecture/rocdown-documentation-compiler.md)
  stating that the sidebar forest is derived from listed ids plus index
  files, not from title equality. Do not mint a new Decision.[^compiler-arch]

**Exit**

```sh
cargo run -q -p rocci-rocdown-cli -- check docs
cargo run -q -p rocci-rocdown-cli -- check site
okmate check knowledge --profile base --format terminal
```

`sites.rocdown` names peel-by-id and `RD2205`. Knowledge check has no
new errors.

### Phase 4 — verify the reading paths

**Bound**

- On a built or `rocdown view` site, walk `/docs/`, `/docs/templates/`,
  `/docs/reference/`, `/docs/reference/language/`, and
  `/docs/reference/contributor/`.
- Confirm Start heading links to `/docs/` with no Overview child;
  Language remains a named fold; Contributor is a named fold whose
  heading is the new landing.
- Confirm `inspect nav` still lists every member page (forest is a view,
  not a second catalog).

**Exit**

```sh
cargo run -q -p rocci-rocdown-cli -- check site
cargo run -q -p rocci-rocdown-cli -- build site
```

Spot-check the five routes. Failed build keeps the previous `dist/`
tree. `cargo fmt` is required only if Phase 4 touched Rust (it should
not).

## Decision gates

Human approval before Phase 1:

1. Peel-by-id (Start loses the Overview child; heading links to `/docs/`).
2. `RD2205` is a warning, not an error.
3. Contributor gets an index. Appendix default is also an index.

Not reopened: stack-IA nav labels; no URL aliases; Rust catalog / Rocci
shell.

## Relationship to other records

| Record | After this plan |
| --- | --- |
| [Directory-plus-index research](/research/rocdown/docs-directory-semantics.md) | Evidence and rejected alternatives. This plan is the work. |
| [Stack-first docs](/plans/site/rocci-dev-docs-stack-ia.md) | Group labels and corpus moves stay. Directory semantics were out of that plan's Bound. |
| [Rocdown documentation generator](/architecture/rocdown-documentation-compiler.md) | Phase 3 adds the forest derivation sentence after the planner ships. |
| [Rust catalog / Rocci shell](/decisions/rust-catalog-rocci-shell.md) | Unchanged ownership. |

## Acceptance

- One public rule: directory + index is a section; heading is the
  landing; nested index is a named subsection.
- Start, Templates, and Language are three instances of that rule, not
  three special cases.
- Contributor appears as a subsection because it has an index.
- Authors can follow `sites.rocdown` without reading `plan.rs`.
- Existing sites without clustered indexless directories keep a clean
  check; clustered ones get a warning they can fix by adding an index.

[^research]: Three observations, peel/fold analysis, recommended nouns and rules, rejected Overview-child and filesystem-only nav.
[^plan-rs]: Current title-matching peel, nested fold, depth flatten, and locking tests.
[^catalog]: Explicit nav resolve and unlisted `RD2202`.
[^sites-ref]: Id/route table and groups; missing forest contract.
[^docs-nav]: Current Start and Reference item lists.
[^site-nav]: Mounted `docs/` prefixes of the same lists.
[^docs-index]: Portal titled Overview under group Start.
[^contributor-checklist]: Contributor pages and the docs-PR checklist to extend.
[^stack-plan]: Stack-layer groups this plan must not rename.
[^compiler-arch]: Architecture record to update after the forest rule ships.
[^catalog-shell]: Planner stays in Rust; theme stays a view consumer.
