---
type: Research Report
title: rocci.dev documentation should follow the stack, not a documentation academy
description: The shipped /docs/ corpus is accurate but organized as a Diátaxis curriculum that hides Rocci's layered composition and exiles Rocdown. Organize by stack layer; keep Diátaxis as an authoring lens.
tags: [domain/rocci, domain/rocdown, concern/publication, concern/developer-experience, concern/architecture, concern/navigation]
status: draft
generated: { by: process:cursor, at: 2026-08-22T12:52:00Z }
stale_after: 2026-11-22
authority: exploratory
owners: [human:nils]
sources:
  - id: docs-index
    resource: ../../../docs/index.rocdown
    title: Current documentation portal
    author: process:git
    last_modified: 2026-08-22
  - id: docs-nav
    resource: ../../../docs/rocdown.toml
    title: Standalone documentation navigation
    author: process:git
    last_modified: 2026-08-22
  - id: site-nav
    resource: ../../../site/rocdown.toml
    title: Unified rocci.dev navigation and mounts
    author: process:git
    last_modified: 2026-08-22
  - id: inventory
    resource: ../../../docs/inventory.toml
    title: Phase 0 truth map and page dispositions
    author: process:git
    last_modified: 2026-08-22
  - id: coverage
    resource: ../../../docs/coverage.toml
    title: Author-facing feature coverage manifest
    author: process:git
    last_modified: 2026-08-22
  - id: what-is
    resource: ../../../docs/start/what-is-rocci.rocdown
    title: What is Rocci?
    author: process:git
    last_modified: 2026-08-22
  - id: compilation
    resource: ../../../docs/concepts/compilation-model.rocdown
    title: Rocci compilation model
    author: process:git
    last_modified: 2026-08-22
  - id: choose-path
    resource: ../../../docs/start/choose-your-path.rocdown
    title: Choose your path
    author: process:git
    last_modified: 2026-08-22
  - id: prerequisites
    resource: ../../../docs/start/prerequisites.rocdown
    title: Documentation prerequisites
    author: process:git
    last_modified: 2026-08-22
  - id: five-minutes
    resource: ../../../docs/start/five-minutes.rocdown
    title: Rocci in five minutes
    author: process:git
    last_modified: 2026-08-22
  - id: first-component
    resource: ../../../docs/tutorials/first-component.rocdown
    title: First-component tutorial
    author: process:git
    last_modified: 2026-08-22
  - id: author-components
    resource: ../../../docs/how-to/author-components.rocdown
    title: Author-components how-to
    author: process:git
    last_modified: 2026-08-22
  - id: pure-components
    resource: ../../../docs/concepts/pure-components.rocdown
    title: Pure-components concept
    author: process:git
    last_modified: 2026-08-22
  - id: props-bodies
    resource: ../../../docs/concepts/props-bodies-composition.rocdown
    title: Props and bodies concept
    author: process:git
    last_modified: 2026-08-22
  - id: performance-model
    resource: ../../../docs/concepts/performance-model.rocdown
    title: Performance-model concept
    author: process:git
    last_modified: 2026-08-22
  - id: why-roc
    resource: ../../../docs/concepts/why-roc.rocdown
    title: Why Roc fits Rocci
    author: process:git
    last_modified: 2026-08-22
  - id: glossary
    resource: ../../../docs/glossary.rocdown
    title: Public glossary
    author: process:git
    last_modified: 2026-08-22
  - id: reference-index
    resource: ../../../docs/reference/index.rocdown
    title: Reference landing
    author: process:git
    last_modified: 2026-08-22
  - id: site-home
    resource: ../../../site/index.rocdown
    title: rocci.dev landing page
    author: process:git
    last_modified: 2026-08-22
  - id: rocdown-index
    resource: ../../../site/rocdown/index.rocdown
    title: Rocdown product-lane portal
    author: process:git
    last_modified: 2026-08-22
  - id: rocdown-pages
    resource: ../../../site/rocdown/pages.rocdown
    title: Write a Rocdown page
    author: process:git
    last_modified: 2026-08-22
  - id: docs-plan
    resource: ../../plans/rocdown/comprehensive-rocci-documentation.md
    title: Comprehensive Rocci documentation plan
    author: process:cursor
    last_modified: 2026-08-22
  - id: site-plan
    resource: ../../plans/site/rocci-dev-site.md
    title: rocci.dev site UX and authoring plan
    author: process:cursor
    last_modified: 2026-08-22
  - id: ux-audit
    resource: ../../audits/site/rocci-dev-site-ux-dx.md
    title: rocci.dev site UX and authoring DX review
    author: process:cursor
    last_modified: 2026-08-21
  - id: stack-skill
    resource: ../../../.agents/skills/rocci-stack/SKILL.md
    title: Rocci stack composition rules
    author: process:git
    last_modified: 2026-08-22
  - id: system-overview
    resource: ../../architecture/system-overview.md
    title: Rocci system overview
    author: process:cursor
    last_modified: 2026-08-18
  - id: root-readme
    resource: ../../../README.md
    title: Rocci workspace overview
    author: human:nils
    last_modified: 2026-08-22
  - id: first-use
    resource: ../../../docs/reference/contributor/first-use.rocdown
    title: First-use measurement protocol
    author: process:git
    last_modified: 2026-08-22
  - id: follow-on-plan
    resource: ../../plans/site/rocci-dev-docs-stack-ia.md
    title: Implementation plan for stack-first docs
    author: process:cursor
    last_modified: 2026-08-22
  - id: diataxis-start
    resource: https://diataxis.fr/start-here/
    title: Diátaxis in five minutes
    author: human:daniele-procida
  - id: svelte-docs
    resource: https://svelte.dev/docs
    title: Svelte documentation portal
    author: organization:svelte
  - id: sveltekit-tree
    resource: https://github.com/sveltejs/kit/tree/main/documentation/docs
    title: SvelteKit documentation directory
    author: organization:svelte
  - id: astro-start
    resource: https://docs.astro.build/en/getting-started/
    title: Astro documentation getting started
    author: organization:astro
---

# rocci.dev documentation should follow the stack, not a documentation academy

## Verdict

The current `/docs/` corpus is factually close to the shipped language. The
problem is the **information architecture**. Readers meet a documentation
school — prerequisites, path chooser, `Kind: concept`, timed lessons, ability
checklists — before they meet the product: Roc and Datastar host HTML; `.rocci`
is the template layer; standalone or custom applications are an optional
opinionated structure on top; Rocdown is a more experimental document layer
that reuses that structure.[^docs-index][^what-is][^stack-skill][^docs-plan][^root-readme]

Diátaxis is a useful **authoring compass**. It is a poor **navigation
taxonomy** for a small experimental stack. The previous plan said the four
kinds should not become visible jargon, then required `Prerequisites` / `Kind`
/ `Next` on every page and named the sidebar Tutorials, How to, and
Understand.[^docs-plan][^diataxis-start][^compilation]

Rocdown does not belong on a sibling product lane that the manual tells
readers to ignore. It belongs in the same docs section, marked experimental,
as the content layer of the same stack.[^docs-index][^inventory][^rocdown-index]

Implementation: [stack-first docs plan](/plans/site/rocci-dev-docs-stack-ia.md).[^follow-on-plan]

## Method

This review read the authored `/docs/` tree (71 Rocdown pages), the `/rocdown/`
lane (8 pages), `docs/rocdown.toml`, `site/rocdown.toml`, `docs/inventory.toml`,
`docs/coverage.toml`, the comprehensive documentation plan, the site UX plan
and audit, the stack skill, and the system overview. It compared page jobs
and lengths, not generated `dist/` HTML.[^docs-nav][^site-nav][^ux-audit]

Industry comparison uses public documentation portals that document a
**layered** compiler-plus-app or content-plus-islands stack: Diátaxis as
authoring theory, Svelte/SvelteKit, and Astro. Django, MDN Learn, and Rust
appear only as the models the previous plan copied.[^docs-plan][^diataxis-start][^svelte-docs][^astro-start]

## What shipped

Phases 0–6 of the comprehensive documentation plan are in the tree. The
result is a Rocci-only manual with seven sidebar groups, a generated examples
catalog, a coverage manifest, and a separate Rocdown product lane.[^docs-plan][^docs-nav][^site-nav][^coverage]

| Surface | Count | Job as shipped |
| --- | --- | --- |
| `docs/` authored pages | 71 | Rocci manual |
| Start | 8 | Orientation, path chooser, two primers, five-minute eval |
| Tutorials | 7 | Six sequential lessons plus index |
| How to | 11 | Task recipes after the tutorials |
| Understand | 16 concepts + glossary | Mental models, each labeled `Kind: concept` |
| Reference | 22 | Language split, CLI, config, runtime, contributor protocol |
| Troubleshooting | 5 | Symptom pages |
| Status | 1 | Labels and known limits (also exists under Project) |
| `site/rocdown/` | 8 | Separate product: pages, blocks, hybrid, language, config, CLI, tree |
| `/examples/` | generated | Cataloged app prose and source |

Almost every docs page opens with some of: **Prerequisites**, **You will
learn/do/build**, **Time**, **Verify**, **Kind**, **Next**. The compilation
page is the example the user named: prerequisites, `Kind: concept`, then a
fork to generated Roc or pure components.[^compilation][^docs-plan]

Most Understand pages are 25–36 lines. They restate a fact that already lives
in a how-to or reference page. `why-roc.rocdown` is the outlier at 287 lines
and spends a section explaining why it will not teach Rocdown.[^pure-components][^props-bodies][^performance-model][^why-roc]

## How the academy got into the product

The comprehensive plan made four lasting quality rules: one fact / one owner,
runnable examples from checked-in sources, aliases when routes move, and
never present a plan as shipped behavior. Those rules are still right.[^docs-plan]

It also imported a curriculum:

1. Treat Diátaxis as the editorial separation, then put Learn / How to /
   Understand / Reference in the sidebar.[^docs-plan][^diataxis-start]
2. Borrow MDN Learn's prerequisites, outcomes, and checkpoints.[^docs-plan]
3. Borrow Django's explicit documentation map on the portal.[^docs-plan]
4. Put **Prerequisites**, **You will build**, **Time**, **Verify**, and
   **Next** on every page instead of difficulty badges.[^docs-plan]
5. Keep Rocdown off the Rocci learning path.[^docs-plan][^inventory]

The inventory encodes that last decision as vocabulary: Rocdown is a
"separate Markdown-first document product. Not part of the Rocci learning
path." The portal, glossary, and What is Rocci? repeat it.[^inventory][^docs-index][^glossary][^what-is]

That was a reasonable way to finish a coverage gap. It is the wrong story
for a **composition**. The site now teaches documentation kinds more
loudly than it teaches the stack.

## What is working

Keep these. The rewrite is IA and chrome, not a factual reset.

- **Reference split.** File structure, components, tags, attributes,
  directives, CSS, fixtures, server declarations, generated Roc, runtime,
  CLI, and configuration are lookup-shaped and close to the template
  crate.[^reference-index][^coverage]
- **Checked-in examples.** Tutorials point at `examples/rocci/...` instead of
  pasting a second copy under `docs/`. `rocci-docs` staging at `/examples/`
  is the right ownership split.[^first-component][^docs-plan]
- **Handler contract.** Documents / fragments / commands / streams, one-shot
  versus live, and Datastar-as-transport match the stack skill and the
  verb-first language.[^stack-skill]
- **Honesty.** Development-`main` labeling, experimental `??` defaults, and
  the status labels are the right maturity voice.[^docs-index]
- **Install and five minutes.** After the path chooser, a reader can build
  the CLI and preview `Hello, Ada` from the styling example.[^five-minutes]
- **Site chrome plan.** Sidebar, breadcrumbs, and News removal are a
  different problem. This research does not reopen that contract except
  where nav *labels* change.[^site-plan][^ux-audit]

## What hurts DX

### 1. The sidebar teaches Diátaxis, not Rocci

A new reader must decide whether they are in Start, a Tutorial, a How-to, or
an Understand page before they know whether they are writing a template, an
application, or a document.[^docs-nav][^site-nav]

Diátaxis itself is a toolbox: four needs (lesson, task, fact, explanation),
a map of their relationships, and a compass for authors. Procida says there
is no exam and you should take only what helps.[^diataxis-start] The useful
move is: write tutorials as lessons, keep reference free of pep talks, and
do not turn the map into seven sidebar headings plus a `Kind:` line.

Django can afford a visible documentation map because the product is huge
and old. Rocci is experimental and small. Sixteen concept pages that each
say "Kind: concept" create the *appearance* of a large academy, not the
feeling of a sharp stack.[^performance-model][^docs-plan]

### 2. Academy chrome is a second navigation system

The theme already has previous/next from the catalog journey. Handwritten
**Next** / **Prerequisites** duplicate it and freeze a linear course
("finish the first component before the application sequence").[^first-component][^ux-audit]

`Kind: concept` is documentation theory in the reader's face. **Time:
about twenty minutes** and **Ability checklist** make a short reference
visit feel like skipping class. The first-use protocol in public reference
is maintainer machinery, not author documentation.[^first-use][^docs-plan]

### 3. The same noun is taught four times

`@component` is a representative collision:

| Page | Kind | What it actually says |
| --- | --- | --- |
| Five minutes | evaluative | Preview `Hello`, one lowering row |
| First component | tutorial | Same styling file, fixtures, inspect, run |
| Author components | how-to | Declare, call, body, fixtures, `components/` |
| Pure components | concept | Render is a pure `Html` function |
| Props and bodies | concept | No magic `children` |
| Components reference | reference | Canonical forms |

The tutorial already states purity. The how-to already states the body
parameter. The two concept pages add almost no new claim.[^five-minutes][^first-component][^author-components][^pure-components][^props-bodies]

Handlers repeat the same split: tutorial (commands and live), how-to
(update the UI), three concepts (documents/fragments, Datastar transport,
one-shot versus live), plus runtime and server reference.

This is the Diátaxis failure mode the previous plan warned about and then
implemented: "avoid stretching one page from what is a component through
AST internals" became four thin pages instead of one guide plus one
reference.[^docs-plan]

### 4. Gates before the product

Start currently asks a reader to pick a background, read a prerequisites
table, optionally take a Roc primer or a web primer, then do a five-minute
eval that builds nothing, then a tutorial that still does not author a new
file.[^choose-path][^prerequisites][^five-minutes][^first-component]

Programmers who already ship HTML or Roc do not want a path chooser. They
want: what is the stack, how do I install, show me a `.rocci` file, then
the layer I am here for.

The primers are useful **appendices**. They are harmful **gates**.

### 5. Rocdown is exiled from the story it depends on

The portal: "Rocdown is a separate Markdown-first product. It does not
belong in this learning path."[^docs-index]

What is Rocci?: "Rocdown is a separate content product, not a fourth piece
of Rocci."[^what-is]

Why Roc: a section titled "Why this page does not teach Rocdown."[^why-roc]

Home: two peer cards, Getting started versus Rocdown.[^site-home]

The Rocdown lane then has to re-teach `@component`, `@get:view`, and
`@method:role` because those declarations are how a live document becomes
an application.[^rocdown-pages][^rocdown-index]

That split matches a **product-boundary** decision (Rocdown owns catalog
and `.rocdown`; Rocci owns templates) and a **docs-scope** decision (the
manual will not mention Rocdown). The first is still correct for crates.
The second is wrong for readers. Rocdown is a layer that *uses* Rocci, the
way SvelteKit uses Svelte — not a second storefront.[^system-overview][^svelte-docs]

Home and `/docs/` currently imply you choose Rocci *or* Rocdown. The stack
is Rocci *then optionally* Rocdown.

### 6. The composition is missing

What is Rocci? lists four **shapes** (component module, standalone app,
custom app, desktop bundle) and a compile-to-Roc note.[^what-is] That is
accurate and still not the composition:

1. **Roc + Datastar** host a web application. Datastar does not care how
   HTML was produced.
2. **`.rocci` templates** are the templating layer for Roc: `@component`,
   `@css`, `@fixture`, markup, directives.
3. **Applications** add an opinionated server structure. Standalone puts
   `@context` / `@init` / `@method:role` in the `.rocci` file. Custom keeps
   an authored `main.roc` and uses `.rocci` as pure templates.
4. **Rocdown** builds content-driven documents and sites on that structure.
   It is more experimental.

The stack skill already states this ownership table. Public docs do not
lead with it.[^stack-skill] Without it, "standalone versus custom" looks
like a fork in a course instead of two supported depths of the same layer.

## Industry pattern: document the composition

Layered web tools that feel good to learn share one move: **nav follows
the stack**, and pedagogy types stay in the writing, not the chrome.

**Svelte / SvelteKit.** The portal asks what you are doing (new, migrating,
playground). Kit docs are Getting started, Core concepts, Build and deploy,
Advanced, Best practices, Reference — capabilities of the app layer, not
Tutorial versus How-to versus Understand. The Svelte language is a
prerequisite, not a rival product with a "do not enter this learning
path" banner.[^svelte-docs][^sveltekit-tree]

**Astro.** Getting started, then Learn (features, islands, components,
template syntax) and Extend (integrations, content collections). There is
one optional blog tutorial. Islands are a **concept because they are the
architecture**, not because every noun needs a `Kind: concept` page.[^astro-start]

**Diátaxis, used well.** Write a lesson as a lesson. Keep reference as
facts. Put "why HTTPS" in a short explanation and link it. Do not label
the lesson `Kind: tutorial` or require a prerequisites page before
`npm create`.[^diataxis-start]

**Django / MDN Learn / Rust bookshelf.** Good for enormous, mature
surfaces. The previous plan copied their *visible* curriculum. Rocci's
surface is closer to SvelteKit-plus-a-Markdown-layer than to Django.[^docs-plan]

**Stripe.** Intent and complete code. No Kind line. Worth keeping as a
portal lesson: "New here" versus "I need the contract," not "choose your
academic track."

A second pattern from these systems: **one tutorial-quality path is
enough**. Astro has one. Svelte's interactive tutorial is a separate
product. Rocci does not need six sequential lessons, ten how-tos, and
sixteen concepts covering the same nouns.

## Recommended contract

### Organize `/docs/` by stack layer

```text
Start          install, first visible result, the stack
Templates      .rocci as HTML for Roc (pure components, CSS, directives)
Applications   standalone and custom; handlers; tooling; package
Rocdown        pages and sites (experimental) at /docs/rocdown/
Reference      facts, structured like the languages and tools
```

Troubleshooting stays as a short symptom appendix. It is not a curriculum
group. Status labels stay in Project plus a line on the portal; they do
not need a seventh docs heading.

`/examples/` stays a first-class site lane. Generated app docs are not the
manual.

Global site nav becomes Docs, Examples, FAQ, Project. There is no
top-level `/rocdown/` lane and no compatibility aliases. Old academy and
product-lane URLs are deleted with the pages that owned them.

### One composition page

`/docs/the-stack/` (or the rewritten portal body) states the four layers,
the two application shapes, and that Rocdown is optional and experimental.
What is Rocci?, compilation model, and standalone-versus-custom collapse
into that page plus the layer landings.

### Merge the quadruplets

Each author-facing noun gets **one guide** (how it fits the layer, one
worked example from a cataloged app) and **one reference** (forms,
defaults, errors, limits). Tutorials become the opening of the guide, not
a parallel track.

Fold into the Templates guide: first component, author components, pure
components, props/bodies, write markup, use directives, styles, structural
control flow.

Fold into the Applications guide: first app, compose, commands and live,
update the UI, documents/fragments/commands/streams, Datastar transport,
one-shot versus live, state and effects, custom main, ship.

Keep a single first-success page (today's five minutes) under Start.

### Kill academy chrome

Remove from page bodies: **Prerequisites**, **Kind**, **You will learn/do**,
**Time**, **Next**. The catalog journey already supplies previous/next.
An opening paragraph can say what the page is. **Verify** stays only on
pages that run commands.

Move Roc-for-Rocci and web foundations to an optional appendix. Delete
Choose your path as a required gate. Fold the prerequisites table into
Install.

Retire or unpublish the first-use measurement protocol until someone
actually runs those sessions. It is not a reader-facing contract.[^first-use]

### Put Rocdown in the same manual

Rocdown is a first-class docs group at `/docs/rocdown/`. Mark the layer
experimental on the landing page, not with a "leave this manual" link.
Live / hybrid pages must say they reuse the application layer
(`@method:role`, `@component`) instead of pretending Rocdown is a second
runtime.

Home should offer stack depths (preview a template, run an app, author a
Rocdown page), not two products.

### Clean cut on URLs

Do not keep `@page` aliases, `/rocdown/` vanity prefixes, or redirects
from `/docs/start|tutorials|how-to|concepts/*`. Rewrite internal links
in the same change. The previous curriculum already piled alias debt;
a second move that preserves it would freeze the academy tree as a
shadow sitemap.[^inventory][^docs-plan]

### Keep the quality rules

Do not drop: one canonical reference owner, coverage for shipped features,
checked-in example sources, experimental/planned/removed labels,
keyboard and no-JS usefulness. Those are why the current facts are
trustworthy.[^docs-plan][^coverage] URL stability is not one of those
rules for this rewrite.

## Target shape (author-facing)

Approximate landing inventory after merge. Reference language pages stay
split; they are already lookup-sized.

| Group | Pages | Notes |
| --- | --- | --- |
| Start | 3 | Portal, install, the stack (five minutes can stay as the first-success heading on install or a sibling) |
| Templates | 4–5 | Layer landing + components + markup/CSS + directives |
| Applications | 5–6 | Layer landing + standalone + handlers + custom + tooling + package |
| Rocdown | 6–7 | `/docs/rocdown/` landing + pages + blocks + sites + hybrid + language + CLI |
| Reference | ~18 | Current Rocci language/runtime/CLI/config; Rocdown lookup lives under `/docs/rocdown/` |
| Troubleshooting | 5 | Keep |
| Appendix | 3 | Roc primer, web primer, glossary; why-roc can live under Project |

That is fewer **nav groups** and fewer **overlapping guides**, not a
thinner reference. A reader who needs `@for` still gets a contract page.

## Risks

- **Broken inbound links.** A clean cut 404s `/rocdown/`, `/docs/start/*`,
  `/guides/*`, and other academy URLs. That is accepted. Do not reintroduce
  aliases to soften it.[^inventory]
- **Coverage manifest.** `docs/coverage.toml` points example fields at
  tutorial and how-to URLs. Those strings must move with the pages.[^coverage]
- **Site UX plan.** It still lists Start / Tutorials / How to / Understand
  and a Rocdown lane. Content IA here supersedes those labels; chrome
  rules (sidebar except Home/FAQ) stay. News 308 targets must be retargeted
  to the new canonical pages, not kept as docs aliases.[^site-plan]
- **Empty layer landings.** A Templates index that only links is another
  academy map. Each layer landing must state when to stop: templates are
  enough if you already have `main.roc`; Rocdown is optional.
- **Rocdown maturity.** Folding it in must not imply it is as settled as
  `@component`. The experimental note is the honesty, not the exile.

## Decisions recorded

Maintainer direction on 2026-08-22:

1. Sidebar groups are **Start / Templates / Applications / Rocdown /
   Reference / Troubleshooting**.
2. The Rocdown layer lives at `/docs/rocdown/`, not `/docs/documents/`
   and not a top-level `/rocdown/` lane.
3. No URL or navigation aliases. Delete old routes with the pages.

Still open:

1. Whether five minutes stays a URL or becomes a section of Install.
2. Whether why-roc stays in docs appendix or moves to Project.

Writing this record does not start the plan.

[^docs-index]: Portal text that sends Rocdown out of the Rocci learning path.
[^docs-nav]: Seven-group Start / Tutorials / How to / Understand sidebar.
[^site-nav]: Docs groups plus a sibling Rocdown product lane.
[^inventory]: Rocci-only vocabulary and page dispositions from the previous curriculum.
[^coverage]: Feature-to-URL map whose example fields still point at tutorials and how-tos.
[^what-is]: Shape table and “Rocdown is not a fourth piece of Rocci.”
[^compilation]: Academy chrome example: Prerequisites, Kind: concept, Next fork.
[^choose-path]: Background gate before install.
[^prerequisites]: Standalone course page for assumptions the install page already needs.
[^five-minutes]: Evaluative preview of the styling example; builds nothing new.
[^first-component]: Tutorial that restates purity and fixtures already in the how-to.
[^author-components]: Task recipe for declare / call / body / fixtures.
[^pure-components]: Thin concept restating that `@component` is a pure `Html` function.
[^props-bodies]: Thin concept restating the extra `Html` body parameter.
[^performance-model]: 30-line Kind: concept page with no new contract.
[^why-roc]: Long explanation that includes “Why this page does not teach Rocdown.”
[^glossary]: Defines Rocdown as not part of the Rocci learning path.
[^reference-index]: Lookup landing that still points Rocdown at the sibling lane.
[^site-home]: Peer cards for Getting started versus Rocdown.
[^rocdown-index]: Separate product portal that must re-teach Rocci declarations.
[^rocdown-pages]: Live Rocdown pages reuse `@component` and `@method:role`.
[^docs-plan]: Prior plan that required academy chrome and Rocci-only scope.
[^site-plan]: Site lane table that still lists Tutorials / How to / Understand and Rocdown.
[^ux-audit]: Chrome review; content IA is a different problem than sidebar visibility.
[^stack-skill]: Layer ownership table used as the public composition source.
[^system-overview]: Crate boundary: Rocdown owns catalog; Rocci owns templates.
[^root-readme]: Workspace product list: templates, Rocdown, desktop runtime.
[^first-use]: Maintainer measurement protocol published as reference.
[^follow-on-plan]: Implementation sequence for this IA.
[^diataxis-start]: Four needs, map, compass, and “do what you like.”
[^svelte-docs]: Portal by reader intent, not documentation kind.
[^sveltekit-tree]: Kit docs grouped as getting started, core concepts, deploy, reference.
[^astro-start]: Learn/Extend by capability; one optional tutorial.
