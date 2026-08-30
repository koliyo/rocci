---
type: Implementation Plan
title: Restructure rocci.dev docs around the layered stack
description: Replace the Diátaxis academy chrome and Rocci-only learning path with a stack-first /docs/ corpus. Rocdown lives at /docs/rocdown/. Clean-cut URLs; no aliases.
tags: [domain/rocci, domain/rocdown, concern/publication, concern/developer-experience, concern/architecture, concern/navigation]
status: draft
generated: { by: process:cursor, at: 2026-08-22T12:52:00Z }
stale_after: 2026-11-22
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../../research/site/rocci-dev-docs-stack-ia.md
    title: rocci.dev documentation should follow the stack, not a documentation academy
    author: process:cursor
    last_modified: 2026-08-22
  - id: docs-plan
    resource: ../comprehensive-rocci-documentation.md
    title: Comprehensive Rocci documentation plan
    author: process:cursor
    last_modified: 2026-08-22
  - id: site-plan
    resource: ../rocci-dev-site.md
    title: rocci.dev site UX and authoring plan
    author: process:cursor
    last_modified: 2026-08-22
  - id: ux-audit
    resource: ../../audits/site/rocci-dev-site-ux-dx.md
    title: rocci.dev site UX and authoring DX review
    author: process:cursor
    last_modified: 2026-08-21
  - id: inventory
    resource: ../../../docs/inventory.toml
    title: Current docs truth map and dispositions
    author: process:git
    last_modified: 2026-08-22
  - id: coverage
    resource: ../../../docs/coverage.toml
    title: Author-facing coverage manifest
    author: process:git
    last_modified: 2026-08-22
  - id: docs-nav
    resource: ../../../docs/rocdown.toml
    title: Standalone documentation navigation
    author: process:git
    last_modified: 2026-08-22
  - id: site-nav
    resource: ../../../site/rocdown.toml
    title: Unified rocci.dev navigation
    author: process:git
    last_modified: 2026-08-22
  - id: docs-index
    resource: ../../../docs/index.rocdown
    title: Current documentation portal
    author: process:git
    last_modified: 2026-08-22
  - id: site-home
    resource: ../../../site/index.rocdown
    title: rocci.dev landing page
    author: process:git
    last_modified: 2026-08-22
  - id: stack-skill
    resource: ../../../.agents/skills/rocci-stack/SKILL.md
    title: Rocci stack composition rules
    author: process:git
    last_modified: 2026-08-22
---

# Restructure rocci.dev docs around the layered stack

## Purpose and authority

This plan executes the [stack-first IA research](/research/site/rocci-dev-docs-stack-ia.md).
It restructures the public `/docs/` corpus and removes the `/rocdown/` lane so
readers meet Rocci as a composition, not as a documentation academy.[^research]

It supersedes the **visible information architecture**, Rocci-only learning
path, academy page chrome, and URL-alias rule of the [comprehensive
documentation plan](/plans/rocdown/comprehensive-rocci-documentation.md). It keeps that
plan's other quality rules: one fact / one owner, checked-in example
sources, coverage for shipped features, and no planned-as-shipped
claims.[^docs-plan][^research]

Maintainer direction on 2026-08-22: the Rocdown layer is `/docs/rocdown/`;
do not keep URL or navigation aliases.

It supersedes the **Docs and Rocdown nav labels** in the [site UX
plan](rocci-dev-site.md). It does not reopen sidebar/breadcrumb chrome,
News dispositions, or the page-finder repair.[^site-plan][^ux-audit]
The current seven-group sidebar, sibling Rocdown lane, and Rocci-only
portal/inventory vocabulary are the baseline this rewrite replaces.[^docs-nav][^site-nav][^docs-index][^inventory]

The record is exploratory. Writing it does not start a phase.

## Goal

Give a programmer one `/docs/` manual that:

1. explains the stack in one screen — Roc + Datastar, `.rocci` templates,
   standalone or custom applications, optional Rocdown documents;
2. gets them to a visible result without a path chooser, prerequisites
   course, or `Kind: concept` chrome;
3. lets them stop at the layer they need;
4. treats Rocdown as an experimental document layer at `/docs/rocdown/`;
5. keeps exhaustive reference, the coverage manifest, and the generated
   `/examples/` catalog;
6. uses one solid URL tree with no compatibility aliases.[^coverage][^stack-skill]

## Out of bound

- Changing `.rocci` or `.rocdown` grammar, lowering, runtime, or CLI
  behavior to simplify a page.
- Reworking marketing visual identity, publishing origin, or deployment.
- Implementing new Rocdown features so the document layer looks finished.
- Teaching full Roc, HTML, HTTP, or Datastar. Link out; keep optional
  primers as appendices.
- Teaching OKF or `rocci-okf`.
- Replacing `rocci-docs` or editing `dist/example-docs` by hand.
- Reopening the site chrome contract (sidebar except Home/FAQ, breadcrumb
  rules, News 410/308 table).
- Inventing first-use session results or shipping Phase 7 of the old plan
  as if the academy IA were still the target.
- Presenting planned hybrid, live hostnames, or language work as current.

## Constraints that do not move

| Constraint | Required behavior |
| --- | --- |
| Stack story | Public docs describe the four layers in the research. Standalone and custom are two depths of Applications, not two products. Rocdown is optional and experimental. |
| One docs corpus | `/docs/` covers templates, applications, and Rocdown. The layer path is `/docs/rocdown/`. There is no top-level `/rocdown/` lane. |
| No academy chrome | Page bodies do not open with Prerequisites, Kind, You will learn/do, Time, or Next. Catalog journey may still render previous/next. |
| Clean-cut URLs | No `@page` aliases, no vanity prefixes, no redirects from retired academy or product-lane routes. Rewrite every internal link in the same change. Old URLs 404. |
| One fact, one owner | Each contract has one canonical reference page. Guides summarize and link. |
| Truth | Current behavior comes from code, tests, crate READMEs, and runnable examples. |
| Examples | Copyable app source stays in `examples/` and `/examples/`. Manual pages do not grow parallel full-file copies. |
| Coverage | `docs/coverage.toml` stays the shipped-feature map. Example/canonical URLs update when pages move. |
| Maturity voice | development `main`; experimental / planned / removed labels stay honest. |
| Authoring format | Authored pages remain `.rocdown`. Knowledge records stay inert Markdown. |
| Build safety | Docs checks do not mutate shipped app sources. Failed site builds keep the previous tree. |

## Target information architecture

### Global site lanes

| Lane | Entry | Sidebar |
| --- | --- | --- |
| Docs | `/docs/` | Start, Templates, Applications, Rocdown, Reference, Troubleshooting |
| Examples | `/examples/` | Catalog (unchanged generator) |
| FAQ | `/faq/` | Unchanged chrome contract |
| Project | `/project/` | Overview, Status, Roadmap, Contributing |

No top-level Rocdown lane. Home is not a lane. Retired `/rocdown/*` and
academy `/docs/{start,tutorials,how-to,concepts}/*` routes are not
redirected.

### Docs groups

```text
/docs/
├── index                  # stack + three depths + link to reference
├── install                # toolchain + first verify (absorbs prerequisites)
├── five-minutes           # optional sibling; first visible component
├── the-stack              # composition; two app shapes; Rocdown experimental
├── templates/
│   ├── index              # when templates are enough
│   ├── components
│   ├── markup
│   └── directives
├── applications/
│   ├── index              # standalone vs custom
│   ├── standalone
│   ├── handlers
│   ├── custom
│   ├── tooling
│   └── package
├── rocdown/
│   ├── index              # experimental; uses templates + apps
│   ├── pages
│   ├── blocks
│   ├── sites
│   ├── hybrid
│   ├── language           # Rocdown lookup
│   └── cli
├── reference/             # Rocci language, runtime, rocci CLI, config
├── troubleshooting/
└── appendix/              # roc-for-rocci, web-foundations, glossary
```

Phase 0 may approve collapsing `five-minutes` into `install` as a heading.
Default in this plan: keep `/docs/five-minutes/`; drop the "this is not
the tutorial" framing. Do not also keep `/docs/start/five-minutes/`.

### Writing contract

- First paragraph states the layer and the job of the page.
- No `Kind:` line. No handwritten course metadata.
- `Verify` only where a command is specified.
- Layer landings say when to stop (you do not need Rocdown to ship an
  app; you do not need Applications if you only preview components into
  an existing `main.roc`).
- Guides use one cataloged example. Reference owns forms and errors.

### Home

Replace the Rocci-versus-Rocdown peer cards with stack depths: preview a
template, run an application, author a document (experimental). Keep the
live-counter island and the maturity note.[^site-home]

## Page dispositions

`disposition` is the editorial action. **Do not copy `@page aliases`.**
Strip alias fields from every moved or rewritten page. Delete the source
file; do not leave a redirect stub.

### Start

| Current | Disposition | Target |
| --- | --- | --- |
| `docs/index.rocdown` | rewrite | stack portal; drop "Rocdown does not belong here" |
| `docs/start/what-is-rocci.rocdown` | merge | `docs/the-stack.rocdown`; delete the source |
| `docs/start/choose-your-path.rocdown` | retire | delete; no redirect |
| `docs/start/prerequisites.rocdown` | merge | `docs/install.rocdown` (move file from `start/`) |
| `docs/start/install.rocdown` | rewrite | install + absorbed prerequisites table |
| `docs/start/five-minutes.rocdown` | keep | drop academy chrome; link the-stack and templates |
| `docs/start/roc-for-rocci.rocdown` | move | `docs/appendix/roc-for-rocci.rocdown` |
| `docs/start/web-foundations.rocdown` | move | `docs/appendix/web-foundations.rocdown` |

### Templates (merge tutorial + how-to + thin concepts)

| Current | Disposition | Target |
| --- | --- | --- |
| `docs/tutorials/first-component.rocdown` | merge | `docs/templates/components.rocdown` |
| `docs/how-to/author-components.rocdown` | merge | same |
| `docs/concepts/pure-components.rocdown` | merge | same (purity is a paragraph, not a page) |
| `docs/concepts/props-bodies-composition.rocdown` | merge | same |
| `docs/how-to/write-markup.rocdown` | merge | `docs/templates/markup.rocdown` |
| `docs/concepts/html-and-identity.rocdown` | merge | same or handlers, whichever owns stable `id` |
| `docs/concepts/styles-and-ownership.rocdown` | merge | `docs/templates/markup.rocdown` (CSS section) or keep a short CSS heading |
| `docs/how-to/use-directives.rocdown` | merge | `docs/templates/directives.rocdown` |
| `docs/concepts/structural-control-flow.rocdown` | merge | same |
| `docs/tutorials/index.rocdown` | retire | delete |
| `docs/how-to/index.rocdown` | retire | delete |

New: `docs/templates/index.rocdown` — layer landing. CSS may stay a
heading inside markup unless the merged page is unwieldy; do not recreate
`styles-and-ownership` as a concept.

### Applications

| Current | Disposition | Target |
| --- | --- | --- |
| `docs/tutorials/first-app.rocdown` | merge | `docs/applications/standalone.rocdown` |
| `docs/tutorials/compose.rocdown` | merge | same (composition section) |
| `docs/tutorials/commands-and-live.rocdown` | merge | `docs/applications/handlers.rocdown` |
| `docs/how-to/update-the-ui.rocdown` | merge | same |
| `docs/concepts/documents-fragments-commands-streams.rocdown` | merge | same |
| `docs/concepts/datastar-as-transport.rocdown` | merge | same |
| `docs/concepts/one-shot-versus-live.rocdown` | merge | same |
| `docs/concepts/state-and-effects.rocdown` | merge | same or standalone |
| `docs/concepts/standalone-versus-custom.rocdown` | merge | `docs/applications/index.rocdown` |
| `docs/tutorials/custom-main.rocdown` | rewrite | `docs/applications/custom.rocdown` |
| `docs/how-to/handle-requests-and-errors.rocdown` | merge | handlers or custom |
| `docs/how-to/develop-and-inspect.rocdown` | merge | `docs/applications/tooling.rocdown` |
| `docs/how-to/use-the-playground.rocdown` | merge | same |
| `docs/how-to/set-up-editors.rocdown` | merge | same |
| `docs/tutorials/ship.rocdown` | merge | `docs/applications/package.rocdown` |
| `docs/how-to/package-and-deploy.rocdown` | merge | same |
| `docs/how-to/configure-and-secure.rocdown` | merge | package; field tables stay in configuration reference |
| `docs/concepts/compilation-model.rocdown` | merge | `docs/the-stack.rocdown` + generated-Roc reference |
| `docs/concepts/development-and-delivery.rocdown` | merge | tooling or package |
| `docs/concepts/security-model.rocdown` | merge | configuration reference + package |
| `docs/concepts/performance-model.rocdown` | merge | runtime reference or retire if it adds no contract |
| `docs/concepts/error-boundaries.rocdown` | merge | diagnostics reference |

### Rocdown (move the product lane into `/docs/rocdown/`)

| Current | Disposition | Target |
| --- | --- | --- |
| `site/rocdown/index.rocdown` | rewrite | `docs/rocdown/index.rocdown` |
| `site/rocdown/pages.rocdown` | move | `docs/rocdown/pages.rocdown` |
| `site/rocdown/article-blocks.rocdown` | move | `docs/rocdown/blocks.rocdown` |
| `site/rocdown/site-config.rocdown` | move | `docs/rocdown/sites.rocdown` |
| `site/rocdown/hybrid.rocdown` | move | `docs/rocdown/hybrid.rocdown`; keep experimental note |
| `site/rocdown/language.rocdown` | move | `docs/rocdown/language.rocdown` |
| `site/rocdown/cli.rocdown` | move | `docs/rocdown/cli.rocdown` |
| `site/rocdown/tree.rocdown` | move | `docs/reference/contributor/rocdown-tree.rocdown` |

After the move, delete `site/rocdown/`. Rocdown landing must state:
static pages are Markdown-first; live pages reuse the application layer;
the layer is more experimental than templates.

### Reference, troubleshooting, leftover

| Current | Disposition | Target |
| --- | --- | --- |
| `docs/reference/language/*` | keep | strip academy chrome only |
| `docs/reference/cli.rocdown` | keep | Rocci CLI; link Rocdown CLI |
| `docs/reference/index.rocdown` | rewrite | link `/docs/rocdown/language/` and `/docs/rocdown/cli/` |
| `docs/troubleshooting/*` | keep | strip chrome; Rocci *and* Rocdown symptoms if distinct |
| `docs/status.rocdown` | fold | Project status + a portal sentence; delete the docs status page |
| `docs/glossary.rocdown` | move | appendix; rewrite Rocdown row |
| `docs/concepts/why-roc.rocdown` | move | appendix or `site/project/`; delete "does not teach Rocdown" |
| `docs/reference/contributor/first-use.rocdown` | retire | unpublish; do not run old Phase 7 against academy IA |
| `docs/reference/contributor/checklist.rocdown` | rewrite | match new paths and chrome rules |

## Phases

### Phase 0 — approve the stack IA

**Bound**

- Record maintainer approval already given for `/docs/rocdown/` and the
  clean-cut URL rule. Confirm remaining gates: academy-chrome ban,
  Choose-your-path retirement, first-use protocol unpublished.
- Freeze the disposition table above in `docs/inventory.toml` (replace the
  Rocci-only vocabulary and nav_labels). Drop the inventory's alias lists;
  list `@page aliases` fields to **remove**.
- List retired routes that will 404 (`/rocdown/*`, `/docs/start/*`,
  `/docs/tutorials/*`, `/docs/how-to/*`, `/docs/concepts/*`, historical
  `/guides/*` and `/getting-started/*`) so the site check can treat them
  as gone, not as missing aliases.

**Exit**

- Inventory `nav_labels` include Rocdown (not Documents) and have no
  alias-preservation section.
- Approval note is in the inventory or this record. No public page has
  moved yet.

### Phase 1 — strip academy chrome and tell the stack story

**Bound**

- Remove Prerequisites / Kind / You will / Time / Next from every
  remaining public docs and Rocdown-lane page (mechanical pass).
- Rewrite `docs/index.rocdown` as the stack portal.
- Add `docs/the-stack.rocdown` from What is Rocci?, compilation, and
  standalone-versus-custom. Delete those sources in Phase 3; do not leave
  alias stubs.
- Rewrite `site/index.rocdown` cards to stack depths.
- Drop "Rocdown is not part of this learning path" from portal, glossary,
  and What is Rocci? if those files still exist.

**Out of this phase:** moving files, merging tutorials, changing
`[[nav]]` group names. Prefer adding the-stack and rewriting the portal
first. Temporary mid-rewrite 404s on old paths are acceptable.

**Exit**

```sh
rg -n '\\*\\*Kind:\\*\\*|\\*\\*Prerequisites:\\*\\*|\\*\\*You will |\\*\\*Time:\\*\\*|\\*\\*Next:\\*\\*' \
  docs site/rocdown --glob '*.rocdown'
```

That search is empty except any appendix that still needs a single
"assumes you can read Roc" sentence written as prose, not a Kind block.

```sh
cargo run -q -p rocci-rocdown-cli -- check site
cargo run -q -p rocci-rocdown-cli -- check docs
```

Portal and home state the four layers. No page tells the reader to leave
the manual to learn Rocdown.

### Phase 2 — fold Rocdown into `/docs/rocdown/`

**Bound**

- Move the eight `site/rocdown/` pages to `docs/rocdown/` and the
  contributor tree per the disposition table.
- Strip every `@page aliases` field on those pages.
- Delete `site/rocdown/` after the move.
- Remove the Rocdown `[[nav]]` from `site/rocdown.toml`. Add a Rocdown
  group under Docs pointing at `docs/rocdown/*`.
- Rewrite the landing: experimental, Markdown-first, live pages reuse
  Applications.

**Exit**

```sh
cargo run -q -p rocci-rocdown-cli -- check site
cargo run -q -p rocci-rocdown-cli -- inspect nav site
```

Nav has no top-level `/rocdown/` lane. Canonical pages are under
`/docs/rocdown/`. `/rocdown/` is absent from the catalog. Hybrid still
says experimental.

### Phase 3 — merge layer guides

**Bound**

- Create `docs/templates/` and `docs/applications/` pages from the
  disposition table.
- Delete tutorial, how-to, and thin concept files. Do not leave
  redirects.
- Move primers to `docs/appendix/`.
- Delete Choose your path.
- Fold prerequisites into install; move install to `docs/install.rocdown`.
  Old `/docs/start/install/` and `/getting-started/installation/` 404.
- Strip leftover `@page aliases` from every remaining docs page.
- Update `docs/rocdown.toml` and the Docs groups in `site/rocdown.toml` to
  Start, Templates, Applications, Rocdown, Reference, Troubleshooting.

**Exit**

- No public page remains under `docs/tutorials/`, `docs/how-to/`,
  `docs/concepts/`, `docs/start/`, or `site/rocdown/`.
- No authored `@page` block lists `aliases`.
- Each layer landing states when to stop.
- Components and handlers are one guide each plus their existing
  reference pages.
- `check site` and `check docs` pass.

### Phase 4 — reconcile reference, coverage, and examples prose

**Bound**

- Add Rocdown language and CLI to the reference index.
- Point `docs/coverage.toml` `canonical` / `example` fields at the new
  guides. Do not drop shipped-feature rows.
- Rewrite `docs/inventory.toml` page list to the new tree.
- Update colocated `examples/rocci/**/index.rocdown` and
  `examples/rocdown/**` links that still cite tutorials or "Kind" chrome.
- Rewrite the contributor checklist. Unpublish first-use.
- Delete `docs/status.rocdown`. Point the portal at `/project/status/`.
- Update root README docs pointers if they name old paths.

**Exit**

- Coverage has no dangling tutorial/how-to/concept URLs.
- Example app prose links into Templates or Applications, not the retired
  tutorial index.
- `cargo run -q -p rocci-okf -- check knowledge --profile base --format terminal`
  is not required for docs-only files; run it if this phase also edits
  knowledge records.

### Phase 5 — verify the reading paths

**Bound**

- Walk three paths on a built site: template-only, standalone app,
  Rocdown page. Each path uses only public pages.
- Confirm the Phase 0 retired-route list is absent from
  `inspect nav` and `pages.json`.
- Confirm no academy chrome and no `@page aliases` regressed.
- `cargo fmt` is not required unless Rust changed (it should not).

**Exit**

```sh
cargo run -q -p rocci-docs -- stage   # if example prose changed
cargo run -q -p rocci-rocdown-cli -- check site
cargo run -q -p rocci-rocdown-cli -- build site
```

Spot-check `/`, `/docs/`, `/docs/the-stack/`, `/docs/templates/`,
`/docs/applications/`, `/docs/rocdown/`, `/docs/reference/`,
`/examples/styling/`. Failed build keeps the previous `dist/` tree.

## Decision gates

Human approval before Phase 1:

Recorded 2026-08-22 and not reopened:

1. Nav labels: Start, Templates, Applications, Rocdown, Reference,
   Troubleshooting.
2. Rocdown lives at `/docs/rocdown/`. No top-level `/rocdown/` lane.
3. No URL or navigation aliases.

Still required before Phase 1 if not already accepted:

4. Academy chrome is banned in page bodies.
5. Choose your path is retired; primers are optional appendix.
6. First-use protocol is unpublished.

## Relationship to other records

| Record | After this plan |
| --- | --- |
| [Comprehensive Rocci documentation](/plans/rocdown/comprehensive-rocci-documentation.md) | Historical for Phases 0–6 content fill. IA, Rocci-only scope, academy chrome, and the URL-alias rule are superseded. Other quality rules remain. Phase 7 first-use sessions are not a gate for this rewrite. |
| [rocci.dev site UX](rocci-dev-site.md) | Chrome contract remains. Docs/Rocdown lane table is superseded. News 308 targets retarget to `/docs/the-stack/`, `/docs/rocdown/sites/`, and `/docs/applications/package/`. |
| [rocci-app-docs](/plans/rocdown/rocci-app-docs.md) | `/examples/` ownership unchanged. |
| `$rocci-stack` | Public docs should match the skill's layer table; do not duplicate the skill into pages. |

## Acceptance

- `/docs/` leads with the stack. Rocdown is a layer, not a warning.
- No public page uses Kind / Prerequisites / You will / Time / Next
  chrome.
- A reader can preview a component, understand standalone versus custom,
  and open `/docs/rocdown/` without changing site lanes.
- Reference remains the Rocci lookup owner. Rocdown language and CLI live
  under `/docs/rocdown/`. Guides do not restate full contracts.
- Coverage matches the new tree. Retired academy and `/rocdown/` URLs are
  gone from the catalog.

[^research]: Current corpus counts, academy-chrome diagnosis, industry comparison, and recommended stack IA.
[^docs-plan]: Prior Diátaxis curriculum, Rocci-only scope, and quality rules.
[^site-plan]: Site lane table that still lists Tutorials / How to / Understand and a Rocdown product lane.
[^ux-audit]: Chrome versus content; this plan changes labels and copy, not the sidebar-except-home contract.
[^inventory]: Current Rocci-only vocabulary and page dispositions.
[^coverage]: Feature-to-URL map that still points at tutorials and how-tos.
[^docs-nav]: Seven-group Start / Tutorials / How to / Understand sidebar.
[^site-nav]: Docs groups plus a sibling Rocdown lane.
[^docs-index]: Portal text that sends Rocdown out of the learning path.
[^site-home]: Peer cards for Getting started versus Rocdown.
[^stack-skill]: Authoritative layer ownership used as the public composition source.
