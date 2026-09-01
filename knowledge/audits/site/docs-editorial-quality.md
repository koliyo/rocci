---
type: Audit
title: Public Rocci docs are accurate but read as generated coverage
description: The landed /docs/ corpus follows the stack-layer map and is mostly factually careful, yet the prose is meta, constraint-first, and compressed so it looks agent-written rather than like a product manual.
tags: [domain/rocci, domain/rocdown, concern/publication, concern/developer-experience, concern/navigation]
status: draft
generated: { by: process:cursor, at: 2026-09-01T12:00:00Z }
stale_after: 2026-12-01
authority: descriptive
owners: [human:nils]
sources:
  - id: docs-index
    resource: ../../../docs/index.rocdown
    title: Documentation portal
    author: process:git
    last_modified: 2026-08-22
  - id: the-stack
    resource: ../../../docs/the-stack.rocdown
    title: The Rocci stack
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
  - id: docs-nav
    resource: ../../../docs/rocdown.toml
    title: Standalone documentation navigation
    author: process:git
    last_modified: 2026-09-01
  - id: templates-index
    resource: ../../../docs/templates/index.rocdown
    title: Templates landing
    author: process:git
    last_modified: 2026-08-22
  - id: templates-components
    resource: ../../../docs/templates/components.rocdown
    title: Components guide
    author: process:git
    last_modified: 2026-08-25
  - id: templates-markup
    resource: ../../../docs/templates/markup.rocdown
    title: Markup guide
    author: process:git
    last_modified: 2026-08-22
  - id: templates-directives
    resource: ../../../docs/templates/directives.rocdown
    title: Directives guide
    author: process:git
    last_modified: 2026-08-22
  - id: apps-index
    resource: ../../../docs/applications/index.rocdown
    title: Applications landing
    author: process:git
    last_modified: 2026-08-22
  - id: apps-standalone
    resource: ../../../docs/applications/standalone.rocdown
    title: Standalone applications
    author: process:git
    last_modified: 2026-08-30
  - id: apps-handlers
    resource: ../../../docs/applications/handlers.rocdown
    title: Handlers
    author: process:git
    last_modified: 2026-08-22
  - id: apps-custom
    resource: ../../../docs/applications/custom.rocdown
    title: Custom applications
    author: process:git
    last_modified: 2026-08-22
  - id: apps-tooling
    resource: ../../../docs/applications/tooling.rocdown
    title: Workflow
    author: process:git
    last_modified: 2026-08-22
  - id: rocdown-index
    resource: ../../../docs/rocdown/index.rocdown
    title: Rocdown landing
    author: process:git
    last_modified: 2026-08-22
  - id: rocdown-pages
    resource: ../../../docs/rocdown/pages.rocdown
    title: Write Rocdown pages
    author: process:git
    last_modified: 2026-09-01
  - id: rocdown-blocks
    resource: ../../../docs/rocdown/blocks.rocdown
    title: Write documentation components
    author: process:git
    last_modified: 2026-08-22
  - id: rocdown-sites
    resource: ../../../docs/rocdown/sites.rocdown
    title: Rocdown site configuration
    author: process:git
    last_modified: 2026-09-01
  - id: rocdown-hybrid
    resource: ../../../docs/rocdown/hybrid.rocdown
    title: Publish a hybrid Rocdown site
    author: process:git
    last_modified: 2026-08-23
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
  - id: lang-components
    resource: ../../../docs/reference/language/components.rocdown
    title: Components language reference
    author: process:git
    last_modified: 2026-08-22
  - id: lang-directives
    resource: ../../../docs/reference/language/directives.rocdown
    title: Template directives reference
    author: process:git
    last_modified: 2026-08-22
  - id: cli-ref
    resource: ../../../docs/reference/cli.rocdown
    title: Rocci CLI reference
    author: process:git
    last_modified: 2026-08-30
  - id: runtime-ref
    resource: ../../../docs/reference/runtime.rocdown
    title: Runtime and HTTP
    author: process:git
    last_modified: 2026-08-22
  - id: checklist
    resource: ../../../docs/reference/contributor/checklist.rocdown
    title: Documentation contributor checklist
    author: process:git
    last_modified: 2026-08-31
  - id: faq
    resource: ../../../site/faq/index.rocdown
    title: rocci.dev FAQ
    author: process:git
    last_modified: 2026-08-23
  - id: stack-ia-plan
    resource: ../../plans/site/rocci-dev-docs-stack-ia.md
    title: Restructure rocci.dev docs around the layered stack
    author: process:cursor
    last_modified: 2026-08-31
  - id: stack-ia-research
    resource: ../../research/site/rocci-dev-docs-stack-ia.md
    title: rocci.dev documentation should follow the stack, not a documentation academy
    author: process:cursor
    last_modified: 2026-08-24
  - id: comprehensive-plan
    resource: ../../plans/rocdown/comprehensive-rocci-documentation.md
    title: Comprehensive Rocci documentation for rocci.dev
    author: process:cursor
    last_modified: 2026-08-31
  - id: polish-plan
    resource: ../../plans/site/docs-editorial-polish.md
    title: Polish public Rocci documentation prose and page jobs
    author: process:cursor
    last_modified: 2026-09-01
---

# Public Rocci docs are accurate but read as generated coverage

## Executive verdict

The public `/docs/` corpus is a good skeleton. Stack-layer navigation, honest
maturity labels, one-owner reference pages, and links into generated
`/examples/` are the right product shape.[^docs-nav][^stack-ia-plan] The
problem the reader feels is not missing topics. It is **how the pages are
written**.

Guides announce themselves as documentation objects, lead with constraints
and ownership rules, then dump a fragment and a `Related:` list. First
success is inspecting a checked-in example from a Cargo workspace, not
making something. Rocdown pages that should teach “write a site” are
operator runbooks. Reference pages are often a status line plus a
contract table. Several pages still talk to contributors inside the user
manual.[^templates-components][^apps-standalone][^rocdown-sites][^reference-index]

That is why the docs look agent-generated. They were filled to satisfy the
comprehensive coverage plan and then compressed to satisfy the stack-IA
writing contract. The map is sound. The sentences are not yet a manual a
programmer would trust as human.[^comprehensive-plan][^stack-ia-plan]

Paired plan: [polish public Rocci documentation](/plans/site/docs-editorial-polish.md).[^polish-plan]

## What is already working

Keep these. Do not reopen them to “fix voice.”

- **Stack as the map.** Start / Templates / Applications / Rocdown /
  Reference / Troubleshooting is the right sidebar. Rocdown is a layer, not
  a peer product lane.[^docs-nav][^the-stack]
- **Stop-at-a-layer is the right idea.** Templates without a generated
  server, apps without Rocdown, are real product cuts.[^templates-index][^apps-index]
- **Honesty.** Development `main`, experimental labels, no fake install
  channel, Windows documented as unverified.[^install][^docs-index]
- **One fact, one owner (as intent).** Language forms live under
  `/docs/reference/language/`. Guides are supposed to summarize.[^lang-components]
- **Some pages already teach.** [Web foundations](../../../docs/appendix/web-foundations.rocdown)
  shows a request, a handler, and a button. Troubleshooting is
  symptom-led. The stack page is the one place the composition should be
  explained.[^the-stack]

## Root cause

Two successive plans produced this voice.

1. The [comprehensive documentation plan](/plans/rocdown/comprehensive-rocci-documentation.md)
   asked for Diátaxis completeness: tutorials with Prerequisites / You will /
   Time / Next, exhaustive how-tos, a coverage manifest, contributor
   checklists. Phases 0–6 filled a large tree.[^comprehensive-plan]
2. The [stack-IA research and plan](/research/site/rocci-dev-docs-stack-ia.md)
   correctly rejected academy chrome and a Rocci-only curriculum. The plan
   merged tutorials, how-tos, and concept pages into thin layer guides, and
   it forbade opening a page with Prerequisites, Kind, You will, Time, or
   Next. The writing contract says the first paragraph must state the
   layer and the job of the page.[^stack-ia-research][^stack-ia-plan][^checklist]

Agents then wrote the only prose that satisfies both: “This page is the
template-layer guide for `@component`. Canonical complete source is … Do
not copy the file into `docs/`.”[^templates-components]

The contributor checklist still encodes that ban in the public
Reference tree.[^checklist] Until that contract is amended, rewrites will
re-emit the same voice.

This is not a failure of stack-first navigation. It is an over-correction
in **page jobs** and **sentence shape**.

## How the pages are formulated

### Meta-documentation

User pages describe the documentation system instead of the product.

| Pattern | Where |
| --- | --- |
| “This page is the template-layer / application-layer guide” | Components, markup, directives, handlers[^templates-components][^templates-markup][^templates-directives][^apps-handlers] |
| “This page is the composition” / “This page is how you develop” | The stack, workflow[^the-stack][^apps-tooling] |
| “Canonical complete source … Do not copy the file into `docs/`” | Components, standalone[^templates-components][^apps-standalone] |
| “This section is **not reviewed**” | Reference landing[^reference-index] |
| “This page was automatically generated” | FAQ[^faq] |

A reader does not need to know the guide/reference split, the staging
rule, or review status before they see a component. Those are contributor
concerns. Putting them in the first paragraph is the strongest
agent-generated tell in the corpus.

### Constraint-first and negation-first

Pages often open by saying what not to do, what you do not need, or what
was removed.

- Templates and Applications landings: “Stop here if…” before showing a
  working file.[^templates-index][^apps-index]
- Handlers: role-first forms are **removed**, then the choice table.[^apps-handlers]
- Grammar reference is a list of things Rocci does not provide (correct
  as lookup; wrong as the only teaching of limits).
- Custom apps: “Do not copy that unfold,” “Do not treat Blocks as a
  custom-main example,” “Snake is the ceiling.”[^apps-custom]

Limits belong after a working picture, or in reference. Leading with them
reads like a knowledge record, not a guide.

### Coverage-manifest diction

The prose copies internal planning language:

- Canonical / owner / disposition / coverage fixture
- `**Status:** current` as the first body line on most language reference
  pages[^lang-components][^lang-directives]
- `**Related:**` link dumps instead of one next step (dozens of pages)
- “Owner: generated dispatch in `rocci-cli` (`dispatch.rs`)” on a public
  runtime page[^runtime-ref]
- Install explaining `@fixture` and `@test` before Rust and Roc exist on
  the machine[^install]

### Workspace README voice

Almost every command is `cargo run -q -p rocci-cli -- …` or
`cargo run -p rocci-rocdown-cli -- …`. That is the documented checkout
spelling, but repeating it on every teaching page makes the product feel
like a Cargo workspace rather than a toolchain. The install page already
says this is temporary; the rest of the manual never lets `rocci` be the
verb.[^install][^five-minutes][^cli-ref]

Five minutes is “lower this checked-in file and inspect `hello =`,” not
“write a greeting.”[^five-minutes] The same `Hello` / `--arg name=Ada`
block appears on the portal, install, five minutes, and the components
guide.[^docs-index][^install][^five-minutes][^templates-components]

### Manifesto repetition

Datastar-as-transport, no magic `children`, no runtime registry, and
“you do not need Rocdown to ship an app” are true. They are restated on
the stack, handlers, markup, glossary, web foundations, custom apps, and
landings. After the stack page, later pages should *use* the model, not
re-argue it.[^the-stack][^apps-handlers]

### Card grids as a substitute for orientation

The portal and every layer landing use `:link-card` grids the way a
generated index does. That is fine *after* a short human paragraph. It is
not a first-success story.[^docs-index][^rocdown-index]

## Structure that still fights the reader

The sidebar groups are right. The **order and bulk inside groups** are
not.

### Start mixes a first-run path with primers

Start is Overview, Install, Five minutes, The stack, then Appendix (Roc,
web foundations, glossary).[^docs-nav] A new reader hits install before
the composition, then three background pages before Templates. Appendix
belongs after the main path, or as its own last group. The stack should
be early: what this is, then install, then a result.

### Guides collapsed into mini-reference

Templates/components is not a walkthrough. After the meta intro it lists
naming rules, `children`, experimental `??`, then a fragment with no
module header, then three CLI commands including `inspect --ast`.[^templates-components]
The language reference page for components is the same facts as a
tighter table.[^lang-components] The reader gets two compressed copies
and still never types a file.

Standalone is closer to teaching, then it spends a heading on “do not
pass a directory,” `apps.toml` is not a run manifest, and nested
`backend/` / `ui/` — contributor-grade detail on the first-app
page.[^apps-standalone]

There is no `docs/tutorials/` tree. That was intentional.[^stack-ia-plan]
The missing piece is not a Diátaxis group. It is **one typed component
session and one typed app session** that still sit under Templates and
Applications.

### Rocdown is the messiest lane

| Page | What a reader needs | What they get |
| --- | --- | --- |
| Rocdown index | Why documents, a tiny `@page` example, next step | Stack restatement, a figure, a note about islands, then cards, then a layout-name list (`home`, `faq`, `product`…) that is rocci.dev chrome[^rocdown-index] |
| Pages | Write Markdown, preview it | Metadata, hydrate `@roc` islands, wiki links into `knowledge/`, Paper vs site outline, `run docs/guide.rocdown` (not a real first file)[^rocdown-pages] |
| Blocks | Notes, figures, maybe tabs | A widget catalog: banana fixture, `hello.rs` include, echo `:example`, file-tree, badge, definition, custom painters, reserved `api-operation`[^rocdown-blocks] |
| Sites | `rocdown.toml` for a small site | 400+ lines: CSP, Docker musl, OKF, `[[mount]]` / `[[peer]]`, News 410, `uv run rocci-ops site`, peel-by-id, `RD2205`, layout ownership[^rocdown-sites] |
| Hybrid | Optional advanced publish | Caddy, Docker targets, CORS-not-shipped, live-counter wiring, inspector URLs — a staging runbook[^rocdown-hybrid] |
| Language | Lookup | Useful tables, plus a duplicated copy-to-clipboard paragraph, plus page-kind matrix again, plus generated export names[^rocdown-language] |

`sites.rocdown` is three documents glued together: user site config,
hybrid publish, and **this repository’s** docs pipeline. That glue is the
largest single editorial failure.

Page-kind (`static` / `hydrate` / `live`) tables appear in sites, hybrid,
and language.[^rocdown-sites][^rocdown-hybrid][^rocdown-language]

### Reference is a lookup index that apologizes

The Reference landing tells the reader the section is not
reviewed.[^reference-index] That destroys trust. Either review it or do
not publish the sentence.

Language subpages are the right *shape* for lookup (grammar, table,
errors) but they starve the reader of one complete file-shaped
example.[^lang-directives] CLI reference leads with workspace Cargo and
puts experimental WASI `--http-module` (sibling `../roc-basic-webserver`
fork) before ordinary `run`.[^cli-ref] Runtime documents 100 ms live
poll and 30 s idle timeout in the same voice as a handler matrix —
correct, but written like a research note.[^runtime-ref]

Contributor tree specs and the docs-PR checklist sit in the main
Reference nav. Maintainers need them; evaluators do not.[^checklist][^docs-nav]

### Duplication across layers

- Standalone vs custom table lives on the stack and again on
  Applications.[^the-stack][^apps-index]
- Handler shape table lives on handlers and runtime.
- Hello/Ada preview command lives in four Start/Templates pages.

## Adjacent surface

The FAQ still opens with “This page was automatically generated. Treat
answers as orientation, not a reviewed contract.”[^faq] That is the same
unwillingness to stand behind prose. It is not `/docs/`, but it is the
other public explanation of the product.

## What “polished” would mean here

Not Stripe-scale. Not a restored academy. A small language’s manual
should feel like htmx or Phoenix LiveView guides: **outcome, complete
example, short explanation, one next step.** Reference stays tables.
The stack is said once. Contributor process stays in `CONTRIBUTING` or a
clearly labeled appendix.

The paired plan lists page jobs, a writing standard with banned
openers, modest nav order changes, and split/rewrite work for the
Rocdown kitchen sinks.[^polish-plan]

## Non-findings

- Do not replace stack-layer nav with Learn / How to / Reference.
- Do not restore URL aliases or `/docs/tutorials/` as a group.
- Do not treat coverage.toml or example staging as the editorial
  problem; they are infrastructure.
- Factual care (experimental vs removed, no planned-as-shipped) should
  survive the rewrite. The labels should get quieter, not disappear.

[^docs-index]: Portal cards, Hello snippet, and maturity note.
[^the-stack]: Four-layer composition and standalone-versus-custom table.
[^five-minutes]: First-success path is lower/inspect of a checked-in file.
[^install]: Source-build install; fixture/test aside before toolchain.
[^docs-nav]: Start includes appendix pages after five-minutes and the stack.
[^templates-index]: Stop-here landing before a working file.
[^templates-components]: Meta opener, copy-into-docs rule, inspect-first teaching.
[^templates-markup]: “This page is the template-layer guide” opener.
[^templates-directives]: Same guide-object opener for control flow.
[^apps-index]: Stop-here landing and duplicated depth table.
[^apps-standalone]: Canonical-source / do-not-copy plus assembly rules on the first-app page.
[^apps-handlers]: Removed-syntax lead-in and Datastar manifesto.
[^apps-custom]: “Snake is the ceiling” and negation-led custom-app page.
[^apps-tooling]: “This page is how you develop” opener.
[^rocdown-index]: Card grid, island note, rocci.dev layout-name list.
[^rocdown-pages]: Hydrate islands and knowledge-bundle wiki examples on the write-a-page guide.
[^rocdown-blocks]: Widget zoo (banana, hello.rs, echo example).
[^rocdown-sites]: Kitchen-sink site page (CSP, Docker, OKF, mounts, this-repo pipeline).
[^rocdown-hybrid]: Operator runbook as the hybrid teaching page.
[^rocdown-language]: Lookup page with duplicated copy-to-clipboard paragraph.
[^reference-index]: “This section is **not reviewed**.”
[^lang-components]: Status line plus contract table as the component reference.
[^lang-directives]: Directive contract table without a complete file example.
[^cli-ref]: Checkout Cargo spelling; WASI module before ordinary run.
[^runtime-ref]: `dispatch.rs` owner line and live-poll research voice.
[^checklist]: Public ban on You will / Next; CI checklist in Reference.
[^faq]: “This page was automatically generated.”
[^stack-ia-plan]: Merged tutorials into thin guides; forbade teaching chrome.
[^stack-ia-research]: Stack-first diagnosis that academy chrome hid the layers.
[^comprehensive-plan]: Diátaxis completeness and coverage-first fill.
[^polish-plan]: Paired rewrite of page jobs and sentences.
