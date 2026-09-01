---
type: Implementation Plan
title: Polish public Rocci documentation prose and page jobs
description: Keep the stack-layer /docs/ map, but rewrite page jobs and sentences so the manual reads as a human product guide rather than a generated coverage dump.
tags: [domain/rocci, domain/rocdown, concern/publication, concern/developer-experience, concern/navigation]
status: draft
generated: { by: process:cursor, at: 2026-09-01T11:55:00Z }
stale_after: 2026-12-01
authority: exploratory
owners: [human:nils]
sources:
  - id: audit
    resource: ../../audits/site/docs-editorial-quality.md
    title: Public Rocci docs editorial review
    author: process:cursor
    last_modified: 2026-09-01
  - id: stack-ia-plan
    resource: rocci-dev-docs-stack-ia.md
    title: Restructure rocci.dev docs around the layered stack
    author: process:cursor
    last_modified: 2026-09-01
  - id: stack-ia-research
    resource: ../../research/site/rocci-dev-docs-stack-ia.md
    title: rocci.dev documentation should follow the stack, not a documentation academy
    author: process:cursor
    last_modified: 2026-08-24
  - id: comprehensive-plan
    resource: ../rocdown/comprehensive-rocci-documentation.md
    title: Comprehensive Rocci documentation for rocci.dev
    author: process:cursor
    last_modified: 2026-08-31
  - id: docs-nav
    resource: ../../../docs/rocdown.toml
    title: Standalone documentation navigation
    author: process:git
    last_modified: 2026-09-01
  - id: site-nav
    resource: ../../../site/rocdown.toml
    title: Unified rocci.dev documentation groups
    author: process:git
    last_modified: 2026-08-28
  - id: docs-index
    resource: ../../../docs/index.rocdown
    title: Documentation portal
    author: process:git
    last_modified: 2026-08-22
  - id: five-minutes
    resource: ../../../docs/five-minutes.rocdown
    title: Rocci in five minutes
    author: process:git
    last_modified: 2026-08-25
  - id: install
    resource: ../../../docs/install.rocdown
    title: Install
    author: process:git
    last_modified: 2026-08-25
  - id: templates-components
    resource: ../../../docs/templates/components.rocdown
    title: Components guide
    author: process:git
    last_modified: 2026-08-25
  - id: apps-standalone
    resource: ../../../docs/applications/standalone.rocdown
    title: Standalone applications
    author: process:git
    last_modified: 2026-08-30
  - id: rocdown-sites
    resource: ../../../docs/rocdown/sites.rocdown
    title: Rocdown site configuration
    author: process:git
    last_modified: 2026-09-01
  - id: rocdown-blocks
    resource: ../../../docs/rocdown/blocks.rocdown
    title: Write documentation components
    author: process:git
    last_modified: 2026-08-22
  - id: rocdown-language
    resource: ../../../docs/rocdown/language.rocdown
    title: Rocdown language reference
    author: process:git
    last_modified: 2026-08-23
  - id: reference-index
    resource: ../../../docs/reference/index.rocdown
    title: Reference landing
    author: process:git
    last_modified: 2026-08-31
  - id: checklist
    resource: ../../../docs/reference/contributor/checklist.rocdown
    title: Documentation contributor checklist
    author: process:git
    last_modified: 2026-09-01
  - id: faq
    resource: ../../../site/faq/index.rocdown
    title: rocci.dev FAQ
    author: process:git
    last_modified: 2026-08-23
---

# Polish public Rocci documentation prose and page jobs

## Purpose and authority

The stack-layer `/docs/` map is kept. This plan rewrites **page jobs** and
**sentences** so the corpus reads as a small-language product manual.
It amends the stack-IA writing contract that forbade You will / Next and
required every guide to open as “this page is the X-layer guide.” It does
not restore Diátaxis sidebar groups, academy chrome, or URL aliases.[^audit][^stack-ia-plan][^stack-ia-research]

Coverage, one-fact-one-owner, checked-in examples, and no planned-as-shipped
claims stay from the comprehensive plan. The exhaustive how-to tree and
first-use session gate stay out.[^comprehensive-plan]

Exploratory. Writing this record does not start a phase.

## Goal

A programmer who can use a terminal can:

1. understand the stack from one page, then install, then cause a visible
   result they authored or at least edited;
2. learn components and a first standalone app by following pages that show
   complete examples before rule tables;
3. look up exact forms in Reference without being told the section is
   unreviewed;
4. learn Rocdown as “write Markdown, then optionally configure a site,”
   with hybrid publish as an advanced extra;
5. never be addressed as a documentation contributor on a user page.

Success is a human read-through of Start + Templates + Applications +
Rocdown index/pages, not a larger coverage manifest.[^audit]

## Out of bound

- Changing `.rocci` / `.rocdown` grammar, lowering, runtime, or CLI to
  simplify a sentence.
- Replacing stack-layer nav labels (Start, Templates, Applications,
  Rocdown, Reference, Troubleshooting) with Learn / How to / Concepts.
- Restoring `/docs/tutorials/`, `/docs/how-to/`, `/docs/concepts/`, or
  `/rocdown/` as groups or alias routes.[^stack-ia-plan]
- Building the comprehensive plan’s full how-to catalog or Phase 7
  first-use study as a gate.[^comprehensive-plan]
- Theme, sidebar chrome, search, or publishing origin work.
- OKF / `rocci-okf` teaching.
- Editing `dist/example-docs` by hand or replacing `rocci-docs`.
- Inventing a crates.io or installer channel.

## Constraints that do not move

| Constraint | Required behavior |
| --- | --- |
| Stack map | Four layers; standalone and custom are two depths; Rocdown optional and experimental. Said in full on The stack; later pages use it, they do not re-argue it. |
| One fact, one owner | Syntax, flags, and config fields stay on one reference page. Guides show one worked case and link. |
| Examples | Complete app source stays in `examples/` and `/examples/`. Manual pages may show a short complete snippet; they do not paste a second full file. |
| Truth | Current behavior from code, tests, crate READMEs, runnable examples. Experimental / planned / removed stay labeled, more quietly. |
| Clean URLs | No new `@page` aliases for retired academy paths. New pages under existing groups are allowed. |
| Checkout commands | Until an installed `rocci` exists, show **one** canonical checkout spelling on Install. Teaching pages may use `rocci …` with a one-line reminder, not `cargo run -q -p` on every block. |
| Coverage | `docs/coverage.toml` still maps shipped features; this plan does not grow it as the product. |

## Writing standard

This is the main deliverable. Apply it on every page this plan touches.

### Page jobs

| Kind | Job | Open with | Close with |
| --- | --- | --- | --- |
| Portal | What you can build and where to start | One proposition, then two paths (preview a template / run an app) | Cards are secondary |
| Composition | The stack, once | The four layers in reader language | Link to install or five minutes |
| Install | Tools on this machine | Imperative steps and a verify command | Link to five minutes, not a primer |
| First success | A result the reader caused | What they will see, then the file or edit | What to read next (one link) |
| Layer guide | How to do the common thing | A complete small example | Pointer to reference for forms/errors |
| Reference | Lookup | Syntax or table immediately | Errors / limits; no `Related:` dump |
| Troubleshooting | Recover | Symptom as heading | The command that proves the fix |
| Appendix | Optional primer | Who should skip this page | Back to the path they left |

Teaching pages **may** start with what you will do and end with one next
page. They must **not** grow Kind, Time, difficulty badges, or a
Prerequisites course. Catalog previous/next chrome may remain.[^audit][^checklist]

### Banned openers (user pages)

Do not start a user-facing page with:

- “This page is the … guide / contract / composition / map.”
- “Canonical complete source … Do not copy the file into `docs/`.”
- “This section is not reviewed.”
- “This page was automatically generated.”
- “Stop here if you only …” as sentence one (put stop-at-layer after the
  example, or on the layer landing only).
- A `**Status:** current` line before any explanation (status can be a
  short note after the form, or only when experimental/removed).
- Restating “Datastar is not a client framework” except on The stack and
  Web foundations.

Move “do not copy into docs,” coverage, and CI commands to the contributor
checklist or `CONTRIBUTING.md`.

### Sentence shape

Write to a person who will type.

**Before (landed):**

> This page is the template-layer guide for `@component`. Canonical
> complete source is Styling.rocci. Do not copy the file into `docs/`.

**After:**

> A component is a function that takes props and returns HTML. This one
> greets whoever you pass as `name`:

Then the example, then one paragraph of rules, then “The exact grammar is
in [Components](/docs/reference/language/components/).”

**Before:**

> Stay standalone until generated dispatch is in the way: extra routes, a
> different platform, or a runtime the dispatcher does not express.

**After:**

> Start with a standalone app: you declare state, a page, and a button
> action; Rocci generates the server. Switch to `main.roc` when you need
> routes or a platform that generator does not give you.

Prefer verbs (`Preview`, `Add a button`, `Split the site config`) over
noun stacks (`template-layer guide`, `one-shot-versus-live comparison`).
Prefer one next link over `**Related:**` with four destinations. If a
paragraph only exists to satisfy coverage, delete it from the guide and
keep it on the reference page.

### Examples in prose

- Show a **complete enough** snippet: `module` / imports when the reader
  would otherwise not know where the fragment lives; skip them when the
  surrounding page already established the file.
- Show expected output: the window text, the URL, the HTTP status.
- One Hello/Ada preview in the first-success page. Other pages link it or
  vary the example.
- Fenced code that is not runnable says so in the sentence above the
  fence, not with a coverage label.

## Structural moves (modest)

Keep groups. Change order and splits.[^docs-nav][^site-nav][^audit]

### Start

Target order:

1. Overview (portal)
2. The stack
3. Install
4. Five minutes
5. Appendix as a **separate last group** (landing + Roc + web + glossary),
   not interleaved in Start

Portal: drop the maturity note into one short sentence if Install already
covers it. Lead with what you can build. Cards after the Hello example, not
as the orientation.

Five minutes: the reader should **author or paste** a tiny component (or
edit a copy), then preview. Inspecting generated Roc is a follow-on
heading, not the session. Keep using the styling example as the
canonical full file via `/examples/`, not as the only action.[^five-minutes]

Install: toolchain, OS tabs, verify `roc` / `cargo build` / one `view`.
Move `@fixture` / `@test` to the components guide. One boxed note for
the `cargo run -p rocci-cli` spelling.[^install]

### Templates

Rewrite in place first. Add **at most one** new page,
`templates/first-component.rocdown`, only if the components guide cannot
stay short *and* teach typing. Default: rewrite `templates/components.rocdown`
as the teaching page (example, preview, fixtures, purity as a paragraph)
and leave `/docs/reference/language/components/` as the table.[^templates-components]

Markup and directives: example first, then the small table, then the
reference link. Stable `id`s stay with markup; Datastar manifesto does not.

### Applications

`applications/standalone.rocdown` becomes the first-app teaching page:
SQLite init, document, increment patch, run, what you should see. Nested
`backend/` / `ui/`, `apps.toml`, and `--http-module` entry rules move to
reference, custom, or contributor notes.[^apps-standalone]

Handlers: choose a shape from a goal (“I clicked a button and want this
card to update”), then the table, then link Counter vs live-counter.
Removed syntax belongs in grammar/troubleshooting, not paragraph two.

Custom: Notes as the default custom app; Datastar gallery as patterns;
Snake as an advanced stress demo — without “ceiling” branding.

Workflow: day-to-day `run` / `view` / `inspect`. Editors and `rocci-ops`
after that, shorter.

### Rocdown (largest split)

| After | Contents |
| --- | --- |
| `rocdown/index` | What it is, a 15-line `@page` + Markdown example, static vs later live in one sentence, cards |
| `rocdown/pages` | Write a page, preview it, links that work on a site. No knowledge-bundle wiki examples. Hydrate `@{expr}` as a later heading |
| `rocdown/blocks` | `:note`, `:figure` / `:img`, `:tabs` if needed for the install-style pattern. Drop banana, `hello.rs`, echo `:example`, badge, file-tree, `api-operation` from the teaching page; those stay in language reference |
| `rocdown/sites` | A small `rocdown.toml`: title, nav, build output, `check` / `build`. Field tables for `[site]` / `[[nav]]` only |
| New `rocdown/publish.rocdown` **or** keep hybrid as the advanced page | Two-artifact publish, `service_origin`, `--cdn-only`, Caddy sketch. Docker musl and `rocci-ops site` stay in `docker/README.md` / contributor checklist |
| Contributor-only | `[[mount]]` / `[[peer]]` as used by **this** repo, News 410, peel-by-id / `RD2205` cookbook — move to contributor checklist or crate README, not the user sites page |

Do not add a seventh Rocdown nav item if hybrid can absorb publish. Prefer
**splitting sites** over adding a page. Language reference remains lookup;
delete the duplicated copy-to-clipboard paragraph.[^rocdown-sites][^rocdown-blocks][^rocdown-language]

Page-kind tables: one canonical copy (language or sites), others link.

### Reference

- Remove “This section is **not reviewed**.” If a page is untrusted, fix
  it or unpublish the claim.[^reference-index]
- Each language page: one complete mini example, then the contract table,
  then errors. Keep them short.
- CLI: `run`, `view`, `build`, `validate` first. `--http-module` under an
  Experimental heading.
- Runtime: handler matrix first; poll/keepalive as a short “Live streams”
  subsection, not research voice.
- Contributor tree + checklist: move the nav group to the end of Reference
  or behind the contributor landing only (Overview + three children). Do
  not list ungram dumps as siblings of CLI.

### FAQ (adjacent, same voice)

Drop “automatically generated.” Answer in the same human voice as the
rewritten portal. Still not a contract page.[^faq]

## Phase 0 — freeze the writing contract

**Bound**

- Amend the stack-IA “no You will / Next” rule and the public contributor
  checklist to match this record’s writing standard. Keep the ban on Kind /
  Time / difficulty badges and on academy URL aliases.[^stack-ia-plan][^checklist]
- Inventory every `docs/**/*.rocdown` page as keep / rewrite / split /
  move-out-of-user-path. No new groups beyond Appendix-as-group and an
  optional `templates/first-component`.
- Approve three voice samples in-tree (portal first paragraph, five-minutes
  opening, a rewritten components opening) before bulk rewrite.

**Exit**

- Maintainer agrees the writing standard above is the rule.
- Checklist no longer forbids a teaching closer of one Next link.
- Disposition table exists in this phase’s commit (in this record or a
  short appendix heading).

**Done in this revision.** Stack-IA writing contract and contributor
checklist match the Writing standard. Samples and disposition below are
the Phase 1+ input. Executing this plan treats the standard as the rule.

### Approved voice samples

Paste into the named pages in later phases. Do not grow Kind / Time chrome.

**Portal (`docs/index.rocdown`) first paragraph:**

> Rocci lets you write HTML in Roc, run a small app from `.rocci` files,
> and optionally publish Markdown on that same stack. This manual is for
> a programmer who can use a terminal.

**Five minutes (`docs/five-minutes.rocdown`) opening:**

> You will type a tiny greeting component, preview it, and see
> **Hello, Ada** in the window. Inspecting the generated Roc comes after
> that.

**Components (`docs/templates/components.rocdown`) opening:**

> A component is a function that takes props and returns HTML. This one
> greets whoever you pass as `name`:

Then the example, then one paragraph of rules, then the Components
reference link. No “do not copy into `docs/`” on this page.

### Disposition

Every `docs/**/*.rocdown` page. No new groups except Appendix as its own
last nav group. No `templates/first-component`. No `rocdown/publish`.[^docs-nav]

| Path | Action | Later phase |
| --- | --- | --- |
| `index.rocdown` | rewrite | 1 |
| `the-stack.rocdown` | rewrite | 1 |
| `install.rocdown` | rewrite; move `@fixture` / `@test` to components | 1 |
| `five-minutes.rocdown` | rewrite; reader pastes a tiny component | 1 |
| `appendix/index.rocdown` | keep; leave Start, become last group | 1 (nav) |
| `appendix/roc-for-rocci.rocdown` | keep | — |
| `appendix/web-foundations.rocdown` | keep (Datastar manifesto stays here) | — |
| `appendix/glossary.rocdown` | keep | 5 (Related dump) |
| `templates/index.rocdown` | rewrite | 2 |
| `templates/components.rocdown` | rewrite in place; no new page | 2 |
| `templates/markup.rocdown` | rewrite | 2 |
| `templates/directives.rocdown` | rewrite | 2 |
| `applications/index.rocdown` | rewrite | 2 |
| `applications/standalone.rocdown` | rewrite; extra assembly rules leave | 2 |
| `applications/handlers.rocdown` | rewrite; open from a user goal | 2 |
| `applications/custom.rocdown` | rewrite | 2 |
| `applications/tooling.rocdown` | rewrite | 2 |
| `applications/package.rocdown` | rewrite | 2 |
| `rocdown/index.rocdown` | rewrite | 3 |
| `rocdown/pages.rocdown` | rewrite | 3 |
| `rocdown/blocks.rocdown` | rewrite; drop widget zoo | 3 |
| `rocdown/sites.rocdown` | split: small `rocdown.toml` + field tables; mounts out | 3 |
| `rocdown/hybrid.rocdown` | rewrite; absorb publish; Docker out | 3 |
| `rocdown/language.rocdown` | rewrite; delete duplicated clipboard sentence | 3 |
| `rocdown/cli.rocdown` | keep | 5 |
| `reference/index.rocdown` | rewrite; drop unreviewed disclaimer | 4 |
| `reference/language/index.rocdown` | keep | 4 (mini-example pass if thin) |
| `reference/language/file-structure.rocdown` | rewrite: mini example, then table | 4 |
| `reference/language/components.rocdown` | rewrite: mini example, then table | 4 |
| `reference/language/tags.rocdown` | rewrite: mini example, then table | 4 |
| `reference/language/attributes.rocdown` | rewrite: mini example, then table | 4 |
| `reference/language/text.rocdown` | rewrite: mini example, then table | 4 |
| `reference/language/directives.rocdown` | rewrite: mini example, then table | 4 |
| `reference/language/css.rocdown` | rewrite: mini example, then table | 4 |
| `reference/language/fixtures.rocdown` | rewrite: mini example, then table | 4 |
| `reference/language/tests.rocdown` | rewrite: mini example, then table | 4 |
| `reference/language/server.rocdown` | rewrite: mini example, then table | 4 |
| `reference/language/comments.rocdown` | rewrite: mini example, then table | 4 |
| `reference/language/generated-roc.rocdown` | rewrite: mini example, then table | 4 |
| `reference/language/grammar.rocdown` | rewrite: mini example, then table | 4 |
| `reference/runtime.rocdown` | rewrite | 4 |
| `reference/cli.rocdown` | rewrite; `run` / `view` / `build` / `validate` first | 4 |
| `reference/configuration.rocdown` | rewrite | 4 |
| `reference/compatibility.rocdown` | keep | 5 |
| `reference/diagnostics.rocdown` | keep | 5 |
| `reference/contributor/index.rocdown` | keep; demote nav | 4 |
| `reference/contributor/checklist.rocdown` | rewrite writing rules (this phase); nav demote | 0, 4 |
| `reference/contributor/rocci-tree.rocdown` | keep generated; not a CLI sibling | 4 (nav) |
| `reference/contributor/rocdown-tree.rocdown` | keep generated; not a CLI sibling | 4 (nav) |
| `troubleshooting/index.rocdown` | keep | 5 |
| `troubleshooting/install.rocdown` | keep | — |
| `troubleshooting/compile.rocdown` | keep | 5 |
| `troubleshooting/runtime.rocdown` | keep | 5 |
| `troubleshooting/preview.rocdown` | keep | 5 |

**Move out of the user path (content, not new URLs):**

| From | To |
| --- | --- |
| `sites.rocdown` `[[mount]]` / `[[peer]]`, News 410, peel-by-id / `RD2205` | Contributor checklist or crate README |
| `hybrid.rocdown` Docker musl and `rocci-ops site` | `docker/README.md` / contributor checklist |
| `blocks.rocdown` banana, `hello.rs`, echo `:example`, badge, file-tree, `api-operation` | Language reference only |
| `standalone.rocdown` nested `backend/` / `ui/`, `apps.toml`, `--http-module` | Reference, custom, or checklist |
| `site/faq/index.rocdown` “automatically generated” | Delete the caution (Phase 4) |

### Frozen gates (Phase 1+)

This execution uses the plan’s stated defaults:

1. **Appendix** is its own last nav group (not interleaved in Start).
2. **Five minutes** pastes a tiny complete `@component` the reader types.
   Link the cataloged styling example as the canonical full file. Inspect
   generated Roc is a follow-on heading.
3. **Split `sites.rocdown`.** Hybrid absorbs two-artifact publish. Do not
   add `rocdown/publish.rocdown`.
4. **Rewrite `templates/components.rocdown` in place.** Do not add
   `templates/first-component.rocdown`.

## Phase 1 — Start path

**Bound**

- Reorder Start vs Appendix in `docs/rocdown.toml` and `site/rocdown.toml`.
- Rewrite `index`, `the-stack`, `install`, `five-minutes`.[^docs-index]
- Deduplicate Hello/Ada to five-minutes (portal may show the snippet once).
- Install loses fixture/test digression.

**Exit**

- Reading Start in sidebar order is a coherent first hour.
- Five minutes includes a result the reader caused, with expected output.
- `rocdown check docs` and `rocdown check site` pass.

## Phase 2 — Templates and Applications

**Bound**

- Rewrite templates and applications guides to the writing standard.
- Standalone is the first-app teaching page; extra assembly rules leave.
- Handlers open from a user goal.
- At most one new templates page, if Phase 0 approved it.

**Exit**

- Components and standalone can be followed without opening Reference,
  except for error lookup.
- No user page opens with “This page is the … guide” or “Do not copy the
  file into `docs/`.”

## Phase 3 — Rocdown user path

**Bound**

- Split `sites.rocdown` as specified. Rewrite index, pages, blocks.
- Hybrid stays advanced; lose duplicate page-kind and Docker-as-primary.
- Fix the duplicated sentence in `language.rocdown`.
- Update internal links in the same change (no aliases).

**Exit**

- A reader can publish a static Markdown site from pages + sites without
  reading mounts, OKF, or musl targets.
- Blocks teaching page is not a widget zoo.

## Phase 4 — Reference, contributor, FAQ

**Bound**

- Reference landing, CLI order, runtime subsectioning, language mini
  examples.
- Contributor pages demoted in nav.
- FAQ generated disclaimer removed; answers tightened.

**Exit**

- No public page tells the reader it is unreviewed or autogenerated.
- Contributor checklist is not a sibling of CLI in the first screen of
  Reference.

## Phase 5 — Voice sweep

**Bound**

- Repo-wide pass: `Related:` dumps, Datastar manifesto repeats, `cargo run
  -q -p` on teaching pages, leftover coverage diction.
- Update crate README links only if a title/route changed.
- Do not restyle the theme.

**Exit**

- Search the corpus for the banned openers; zero hits on user pages
  (contributor checklist may still mention them as forbidden).
- `rocdown check docs`, `rocdown check site`, `uv run rocci-ops check docs`
  as used today.

## Decision gates

Frozen in Phase 0 (see Frozen gates). Human can still override before
Phase 1; later phases follow those four defaults unless a later Bound
changes.

## Relationship to older plans

- [Stack-first IA](rocci-dev-docs-stack-ia.md) still owns nav groups and
  clean URLs. This plan amends only its writing contract and page-job
  merge (guides may teach; they must not become a second reference).
- [Comprehensive documentation](../rocdown/comprehensive-rocci-documentation.md)
  remains historical for coverage and example ownership. Do not execute
  its tutorial/how-to tree.

[^audit]: Landed `/docs/` is accurate but formulated as generated coverage.
[^stack-ia-plan]: Stack-layer nav and the writing contract this plan amends.
[^stack-ia-research]: Why academy chrome hid the composition.
[^comprehensive-plan]: Coverage and example ownership to keep; how-to tree not to execute.
[^docs-nav]: Current Start / Templates / Applications / Rocdown groups.
[^site-nav]: Unified site mount of the same groups.
[^docs-index]: Current documentation portal to rewrite in Phase 1.
[^five-minutes]: Current inspect-first five-minute page.
[^install]: Current source-build install page.
[^templates-components]: Current meta components guide.
[^apps-standalone]: Current first-app page with assembly rules.
[^rocdown-sites]: Kitchen-sink site configuration page.
[^rocdown-blocks]: Widget-catalog blocks page.
[^rocdown-language]: Language reference with duplicated clipboard paragraph.
[^reference-index]: Unreviewed disclaimer on the Reference landing.
[^checklist]: Contributor checklist writing rules; teaching Next is allowed.
[^faq]: Autogenerated disclaimer on the public FAQ.
