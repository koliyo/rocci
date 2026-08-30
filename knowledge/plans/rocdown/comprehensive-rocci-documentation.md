---
type: Implementation Plan
title: Comprehensive Rocci documentation for rocci.dev
description: Build a Rocdown-authored Rocci manual that takes a programmer from first contact through production authoring, while giving experienced authors complete, searchable, tested reference material.
tags: [domain/rocci, concern/publication, concern/developer-experience, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-08-22T12:52:00Z }
stale_after: 2026-11-21
authority: exploratory
owners: [human:nils]
sources:
  - id: root-readme
    resource: ../../../README.md
    title: Rocci workspace overview and current public-site workflow
    author: human:nils
    last_modified: 2026-08-21
  - id: template-readme
    resource: ../../../crates/rocci-template/README.md
    title: Implemented Rocci template-language contract
    author: process:git
    last_modified: 2026-08-21
  - id: cli-readme
    resource: ../../../crates/rocci-cli/README.md
    title: Implemented Rocci CLI and development workflow
    author: process:git
    last_modified: 2026-08-21
  - id: docs-config
    resource: ../../../docs/rocdown.toml
    title: Current standalone documentation navigation
    author: process:git
    last_modified: 2026-08-21
  - id: site-config
    resource: ../../../site/rocdown.toml
    title: Current unified rocci.dev navigation and documentation mount
    author: process:git
    last_modified: 2026-08-21
  - id: docs-index
    resource: ../../../docs/index.rocdown
    title: Current documentation portal
    author: process:git
    last_modified: 2026-08-21
  - id: rocci-reference
    resource: ../../../docs/reference/rocci.rocdown
    title: Current public Rocci language reference
    author: process:git
    last_modified: 2026-08-21
  - id: cli-reference
    resource: ../../../docs/reference/cli.rocdown
    title: Current combined product CLI reference
    author: process:git
    last_modified: 2026-08-21
  - id: configuration-reference
    resource: ../../../docs/reference/configuration.rocdown
    title: Current public Rocci configuration reference
    author: process:git
    last_modified: 2026-08-21
  - id: examples-index
    resource: ../../../docs/examples/index.rocdown
    title: Current combined examples index
    author: process:git
    last_modified: 2026-08-21
  - id: troubleshooting
    resource: ../../../docs/troubleshooting.rocdown
    title: Current combined troubleshooting page
    author: process:git
    last_modified: 2026-08-21
  - id: all-syntax
    resource: ../../../test/AllSyntax.rocci
    title: Current dense Rocci syntax fixture
    author: process:git
    last_modified: 2026-08-21
  - id: site-plan
    resource: ../rocci-dev-site.md
    title: rocci.dev site architecture and Rocdown evolution
    author: process:codex
    last_modified: 2026-08-18
  - id: app-docs-plan
    resource: ../rocci-app-docs.md
    title: Documentation generator for Rocci applications
    author: process:cursor
    last_modified: 2026-08-21
  - id: rocci-docs-readme
    resource: ../../../crates/rocci-docs/README.md
    title: Implemented Rocci application-documentation staging tool
    author: process:git
    last_modified: 2026-08-21
  - id: rocci-docs-stage
    resource: ../../../crates/rocci-docs/src/stage.rs
    title: Implemented Rocci application-documentation staging and generated pages
    author: process:git
    last_modified: 2026-08-21
  - id: rocci-docs-tests
    resource: ../../../crates/rocci-docs/tests/stage.rs
    title: Rocci application-documentation staging contract tests
    author: process:git
    last_modified: 2026-08-21
  - id: apps-catalog
    resource: ../../../examples/rocci/apps.toml
    title: Published Rocci application catalog
    author: process:git
    last_modified: 2026-08-21
  - id: site-workflow
    resource: ../../../.github/workflows/site.yml
    title: Site packaging workflow with Rocci application documentation staging
    author: process:git
    last_modified: 2026-08-21
  - id: ops-ci
    resource: ../../../tools/rocci-ops/src/rocci_ops/ci.py
    title: Local CI sequence for application docs and site checks
    author: process:git
    last_modified: 2026-08-21
  - id: ops-site-package
    resource: ../../../tools/rocci-ops/src/rocci_ops/site.py
    title: Local site packaging driven by the catalog live-app projection
    author: process:git
    last_modified: 2026-08-21
  - id: diataxis
    resource: https://diataxis.fr/start-here/
    title: Diataxis in five minutes
    author: human:daniele-procida
  - id: stripe-docs
    resource: https://docs.stripe.com/
    title: Stripe documentation portal
    author: organization:stripe
  - id: stripe-quickstarts
    resource: https://docs.stripe.com/quickstarts
    title: Stripe quickstart guides
    author: organization:stripe
  - id: django-docs
    resource: https://docs.djangoproject.com/en/6.0/
    title: Django documentation organization
    author: organization:django-software-foundation
  - id: mdn-learn
    resource: https://developer.mozilla.org/en-US/docs/Learn_web_development
    title: MDN Learn web development
    author: organization:mozilla
  - id: rust-docs
    resource: https://doc.rust-lang.org/stable/
    title: Rust documentation bookshelf
    author: organization:rust-project
  - id: stack-ia-research
    resource: ../../research/site/rocci-dev-docs-stack-ia.md
    title: rocci.dev documentation should follow the stack, not a documentation academy
    author: process:cursor
    last_modified: 2026-08-22
  - id: stack-ia-plan
    resource: ../rocci-dev-docs-stack-ia.md
    title: Restructure rocci.dev docs around the layered stack
    author: process:cursor
    last_modified: 2026-08-22
---

# Comprehensive Rocci documentation for rocci.dev

## Purpose and authority

This plan defines the **Rocci** documentation corpus for the main `rocci.dev`
site. It covers the `.rocci` language, author workflows, server interaction,
runtime behavior, tooling, configuration, testing, debugging, packaging, and
deployment. The documentation source is Rocdown, mounted into the unified site
at `/docs/`; generated application documentation is mounted at `/examples/`.
Both teach Rocci, not Rocdown.[^root-readme][^site-config][^rocci-docs-readme]

Later IA work: the visible Diátaxis curriculum, academy page chrome,
Rocci-only `/docs/` scope, and the “keep aliases when pages move” rule
are superseded by the stack-first documentation research and plan.
Rocdown lives at `/docs/rocdown/` with no compatibility URLs. Other
quality rules in this record (one fact / one owner, runnable examples,
coverage, no planned-as-shipped claims) remain. Phase 7 first-use
sessions are not a gate for that rewrite.[^stack-ia-research][^stack-ia-plan]

The record is exploratory. Phases 0–6 of the original curriculum are in the
tree. Writing the plan did not, and this revision does not, start the
stack-first rewrite.

## Goal

Create one coherent manual that lets a reader:

1. decide quickly whether Rocci fits their project;
2. install the actual supported toolchain and get a visible result without
   hidden prerequisites;
3. learn Rocci progressively even if they know little Roc or server-driven UI;
4. move from a first component to a stateful standalone application;
5. solve concrete authoring, debugging, testing, packaging, and deployment
   tasks without rereading a tutorial;
6. look up every stable author-facing syntax form, command, option,
   configuration field, runtime contract, and deliberate limitation; and
7. understand the design well enough to make good architectural choices in a
   substantial Rocci application.

“Absolute beginner” means new to Rocci and possibly new to Roc, HTML-over-the-
wire, Datastar, and desktop webviews, but able to read code, use a terminal,
and understand basic programming constructs. The docs do not teach programming
from zero.

## Out of bound

- Teaching the Rocdown language, Rocdown site construction, documentation
  components, hybrid Rocdown islands, or the `rocdown` CLI.
- Teaching OKF, `rocci-okf`, or knowledge-bundle maintenance.
- Reworking the public marketing site, visual identity, publishing origin, or
  deployment infrastructure already covered by the site and publishing plans.[^site-plan]
- Implementing new Rocci syntax, runtime behavior, packaging targets, editor
  features, or release channels merely to simplify a page.
- Presenting planned language or runtime work as shipped behavior.
- Replacing Roc's own language documentation, HTML/HTTP standards references,
  or Datastar reference documentation. Rocci pages teach the subset and mental
  model needed for the task, then link outward.
- Developer-internal compiler architecture except where it explains an
  author-visible contract such as lowering, source mapping, purity, or error
  boundaries.
- Building another application-documentation generator. `rocci-docs` already
  owns colocated example prose and full source-tree staging.[^rocci-docs-readme]

## Constraints that do not move

| Constraint | Required behavior |
| --- | --- |
| Product scope | The main track says Rocci throughout. Rocdown pages may remain elsewhere on the site, but they do not interrupt the Rocci learning path or share a supposedly exhaustive Rocci reference page. |
| Authoring format | Every authored manual page is `.rocdown`; source examples are included or tested through the Rocdown documentation toolchain. |
| Manual/example boundary | `docs/` owns the curated manual. Cataloged app prose stays beside the app as `.rocdown`; attached `## ` comments explain declarations on generated source pages; `rocci-docs` alone stages `/examples/`. |
| Truth | Current behavior comes from owning code, tests, crate READMEs, and runnable examples. Exploratory plans are labeled and never used as proof of shipped behavior. |
| One fact, one owner | Each contract has one canonical reference page. Tutorials, guides, examples, troubleshooting, and landing pages summarize and link instead of restating the full contract. |
| Progressive disclosure | Beginner pages expose the smallest safe model first. Expert detail remains one link away, not mixed into every first-run step. |
| Runnable examples | Copyable code is either compiled/tested from a canonical fixture or explicitly marked partial/pseudocode. No silent ellipses in a block presented as runnable. |
| Web foundations | Teach semantic HTML, HTTP method/response roles, and server-owned state as foundations. Do not describe Datastar as a client application framework. |
| Roc ownership | Ordinary Roc remains ordinary Roc. Rocci documentation clearly marks which regions are Rocci syntax, which are Roc, and what lowering produces. |
| Accessibility | The manual, examples, and demo applications must work with keyboard navigation, zoom, reduced motion, forced colors, and meaningful HTML. Accessibility is part of correctness, not a late editorial pass. |
| Stability | Stable routes keep aliases when pages move. Published pages identify the product/version status they describe. |
| Build safety | Documentation checks do not mutate shipped application sources; failed site builds preserve the prior output tree. |

## Evidence and lessons from leading documentation

Use Diataxis as the editorial separation, not as visible jargon readers must
learn. Tutorials teach through a controlled experience; how-to guides solve a
real task for a competent reader; reference states facts; explanation builds
understanding.[^diataxis] Rocci needs all four, but its navigation should use
plain labels: **Learn**, **How to**, **Understand**, and **Reference**.

Borrow five structural patterns from established systems:

1. **Start from reader intent.** Stripe opens with developer entry points,
   common use cases, quickstarts, samples, and reference instead of requiring a
   linear tour of the whole platform.[^stripe-docs]
2. **Make quickstarts complete.** Stripe's quickstarts are end-to-end, expose
   alternatives where relevant, and connect instructions to real code rather
   than isolated fragments.[^stripe-quickstarts] Rocci should provide one
   canonical first component and one canonical first application, each with a
   known starting state, expected result, complete source, and verification.
3. **Explain the documentation map.** Django explicitly distinguishes
   tutorials, topic explanations, how-to recipes, and reference, and tells
   newcomers where to begin.[^django-docs] Rocci's portal should do the same in
   one screen and then disappear from the reader's way.
4. **Define learning outcomes and checkpoints.** MDN gives beginners a bounded
   pathway, states prerequisites, uses skill checks, and aims to move a learner
   from beginner to comfortable before handing them to advanced material.[^mdn-learn]
5. **Offer different shelves for different experience levels.** Rust separates
   a narrative book, example-led learning, tooling books, API documentation,
   extended errors, and advanced references.[^rust-docs] Rocci should likewise
   avoid stretching one page from “what is a component?” through AST internals.

Do not copy product-specific machinery that does not fit Rocci. There is no
need for account-personalized snippets, multiple client-language selectors, or
a huge generated API explorer. The useful Stripe lesson is contextual,
complete code and strong task entry points. The useful Rust lesson is a set of
connected manuals, not duplication between them.

## Current baseline and primary gaps

The repository already has 24 Rocdown pages under `docs/`, mounted under
`/docs/` in the main site. They include a portal, three getting-started pages,
guides, concepts, references, examples, playground, and troubleshooting.
Navigation currently mixes Rocci, Rocdown, OKF-adjacent tooling, and the project
browser in the same lanes.[^docs-config][^site-config][^docs-index]

Keep and improve the useful starting material, but do not preserve its page
boundaries by default:

- The current Rocci reference is a compact overview while the owning template
  README documents substantially more author-visible behavior: defaulted
  fields, component calling, action attributes, whitespace, generated Roc,
  server handler semantics, and deliberate limits.[^rocci-reference][^template-readme]
- The public CLI page combines `rocci`, `rocdown`, and `rocci-okf`. The Rocci
  CLI README already describes a richer Rocci-only development surface,
  including build, run, view, browse, playground, rendering, inspection,
  release packaging, bundling, Datastar assets, and the preview
  inspector.[^cli-reference][^cli-readme]
- Configuration has a useful complete-shape page, but the eventual reference
  needs field types, defaults, requirements, platform support, interactions,
  validation failures, and task links for every supported field.[^configuration-reference]
- Examples and troubleshooting mix both products. Rocci readers need a
  filterable Rocci pattern catalog and failure guidance that maps directly to
  Rocci commands and diagnostics.[^examples-index][^troubleshooting]
- `test/AllSyntax.rocci` is valuable compiler coverage, but its density makes it
  a developer fixture rather than a beginner curriculum or a polished author
  reference.[^all-syntax]

The landed `rocci-docs` generator is the current application-example
infrastructure, not a later dependency:

- `examples/rocci/apps.toml` explicitly catalogs six apps. Counter,
  handler-matrix, styling, and snake are docs-only; live-counter and datastar
  are selected for separate live origins.[^apps-catalog]
- Every catalog row must resolve to an application and a colocated
  `index.rocdown`. The generator copies authored Rocdown pages, inventories
  publishable `.rocci`, authored `.roc`, `rocci.toml`, and asset files, and
  writes a generated catalog, source indexes, source pages, and safe snippet
  copies.[^rocci-docs-readme][^rocci-docs-stage]
- Attached Roc-style `## ` comments on top-level Rocci declarations are parsed
  through `rocci-template` and inserted above the complete highlighted source
  include. They are declaration-local source documentation, not a replacement
  for tutorials or the language reference.[^rocci-docs-readme][^rocci-docs-stage]
- The main site mounts `dist/example-docs` at `/examples/`; local CI and the
  site workflow stage it before checking or packaging the site.[^site-config][^ops-ci][^site-workflow]
- Staging tests cover catalog validation, include/exclude rules, safe relative
  includes, declaration extraction, generated routes, and deterministic
  output.[^rocci-docs-tests]

This removes the need for a second hand-authored examples portal or copied
full-source pages. It does **not** yet prove that every cataloged app compiles,
that the smoke commands embedded in app prose run, or that a failed stage
preserves the previous output: the current staging function deletes the output
directory before writing the replacement.[^rocci-docs-stage][^rocci-docs-tests]
Those are documentation-system hardening requirements in this plan, not reasons
to build a parallel generator.

The first implementation phase must therefore inventory and classify existing
content as **keep**, **split**, **rewrite**, **move**, or **retire**. It must
also record the authoritative code/test source for every current-behavior
claim before prose is expanded. The inventory includes both manual pages under
`docs/` and the generator inputs (`apps.toml`, each app's Rocdown pages, and
attached declaration docs); generated `dist/example-docs` is never edited.

## Audiences and entry states

Design for needs, not vague beginner/intermediate/advanced labels:

| Reader | What they already know | Immediate question | First destination |
| --- | --- | --- | --- |
| Evaluator | General programming | What is Rocci, what can it build, and what are its limits? | `/docs/` then “Rocci in five minutes” |
| New Rocci author | Programming and terminal basics | How do I get a working result? | Install then first component |
| Roc developer | Roc syntax and effects, little server-driven UI | Where are HTML, HTTP, Datastar, and state boundaries? | First app plus rendering model |
| Web developer | HTML/CSS/HTTP, little Roc | What Roc do I need and where does it go? | Roc-for-Rocci primer plus first component |
| Application author | Basic components and routes | How do I model state, patches, commands, live updates, errors, and requests? | Application tutorial and how-to guides |
| Experienced Rocci author | Shipped mental model | What is the exact syntax/default/response/tool behavior? | Search or reference |
| Maintainer/debugger | Generated Roc and toolchain awareness | Why did this fail and how do I inspect the boundary? | Diagnostics, inspection, source maps, troubleshooting |
| Shipping author | Working application | How do I configure, package, secure, and operate it? | Production and desktop guides |

Every portal card and page introduction should signal the assumed entry state.
Pages should use explicit fields in prose—**Prerequisites**, **You will build**,
**Time**, **Verify**, and **Next**—instead of colored “difficulty” badges that
mean different things to different readers.

## Documentation architecture

### Top-level navigation

Keep `/docs/` as the Rocci documentation portal. Its primary lanes are:

```text
/docs/
├── start/              # orientation, prerequisites, installation, first success
├── tutorials/          # controlled learning projects
├── how-to/             # task recipes for working authors
├── concepts/           # mental models and design explanations
├── reference/          # exhaustive factual contracts
├── troubleshooting/    # symptom- and diagnostic-led recovery
├── glossary/           # Rocci, Roc, HTML, HTTP, Datastar, and host terms
└── status/             # support matrix, compatibility, known limits, change policy

/examples/              # rocci-docs catalog, app prose, and complete source trees
```

This is a reader-facing taxonomy. Source directories should mirror routes so a
contributor can predict where a page lives. Preserve aliases for retained
current routes such as `/docs/getting-started/quickstart/` and
`/docs/guides/server-actions/`. Keep `/examples/` a first-class site lane whose
source is generated by `rocci-docs`; `/docs/examples/` remains only its
compatibility alias.[^rocci-docs-stage][^site-config]

Rocdown documentation should use its own visible product lane and route
prefix. Deciding its final curriculum is outside this plan. Shared subjects
such as the project browser may be linked as tools, but they must not make the
Rocci CLI reference claim to cover all Rocci-family binaries.

### Documentation portal

The portal should answer four questions without scrolling through a sitemap:

1. **What can I build?** A concise proposition, supported application shapes,
   a working screenshot/demo, and honest maturity/platform notes.
2. **Where should I start?** Two buttons: “New to Rocci” and “I know the
   basics; find a task or reference.”
3. **What does the code feel like?** One small, complete component plus a
   direct link to run/inspect it.
4. **Where is the exact answer?** Search plus direct links to language, CLI,
   configuration, handlers, and troubleshooting reference.

Below that, show learning tracks, common tasks, example applications, and the
latest compatibility notice. Do not reproduce the marketing home page or list
every document.

## Learning curriculum

### Start: first 30 minutes

| Page | Outcome | Notes |
| --- | --- | --- |
| What is Rocci? | Explain `.rocci` as Roc plus bounded HTML templates, the compile-to-Roc model, supported app shapes, and deliberate limits. | Rocdown gets one “separate content product” link, not a four-part Rocci definition. |
| Choose your path | Route “I know Roc,” “I know web development,” and “I know both” readers to the missing foundation. | Each path converges before the first app. |
| Prerequisites | State programming, terminal, Git, Roc, Rust/source-build, HTML, and platform assumptions precisely. | Link optional refreshers; do not teach all prerequisites inline. |
| Install Rocci | Give supported installation/source-build instructions for macOS, Linux, and Windows, then one deterministic version check. | Describe only an actually shipped release channel. |
| Roc for Rocci authors | Teach modules/imports, records, tags, functions, pipelines, `match`, effects, `?`, and the selected platform only to the depth needed by later examples. | Link canonical Roc docs for full language coverage. |
| Web foundations for Roc authors | Explain semantic HTML, attributes, forms, request methods, status codes, fragments, DOM IDs, SSE, and same-origin behavior. | Use one request/response diagram and one real handler. |
| Rocci in five minutes | Lower and preview a tiny component, show the authored source and generated Roc, and explain exactly one boundary. | This is evaluative, not the full tutorial. |

### Tutorials: guided competence

Tutorials form a deliberate sequence, but each begins from a downloadable or
checked-in starting point so readers may join later.

1. **Build your first component.** Create a typed component, pass props, use
   interpolation, add scoped CSS, add a fixture, inspect lowering, preview it,
   and verify the result. Use the cataloged styling app for the complete-source
   destination rather than copying its file into `docs/`.
2. **Build your first standalone app.** Add a full document, `@context`,
   `@init`, `@view`, one `@patch`, a Datastar action, persistent state, error
   propagation, and runtime logging. End with the cataloged counter app and its
   generated `Counter.rocci` source page.
3. **Compose a small application.** Split components into modules, add lists,
   conditional states, forms, validation, assets, and multiple routes. Teach
   authoring structure without introducing a custom `main.roc` prematurely.
4. **Add commands and live updates.** Convert the relevant mutation to
   `@command`, add `@live`, open two clients, inspect 204/JSON behavior, and
   explain when one-shot `@patch` is the simpler design. Compare the generated
   counter and live-counter application pages side by side.
5. **Take ownership with `main.roc`.** Move to a custom application only after
   the generated standalone boundary is understood. Reuse `.rocci` modules
   while owning routing and runtime effects in Roc; use datastar as the normal
   pattern gallery and snake as the explicitly advanced stress example.
6. **Ship the application.** Validate configuration, make a release server
   artifact, run it in the supported production shape, and—on macOS—build a
   local desktop bundle as a separate optional ending.

Every tutorial ends with:

- a visible or HTTP-verifiable result;
- a checklist phrased as abilities, not trivia;
- a link to the complete source at that tutorial checkpoint;
- a “what Rocci did for you” lowering/runtime recap;
- two next routes: continue learning or solve a related real task.

### How-to guides: working recipes

Use one goal per page. Initial manifest:

#### Components and markup

- Define and call a component.
- Pass record props and default fields.
- Accept and render a component body.
- Compose local and module-qualified components.
- Render fragments and void elements.
- Write static, dynamic, boolean, ARIA, and `data-*` attributes.
- Interpolate strings and convert non-string values.
- Render conditional content with `@if`.
- Render lists with `@for`.
- Render variants with `@match`.
- Bind derived render data with `@let`.
- Add file-level and component-scoped CSS.
- Add fixtures and preview fixture states.
- Organize a reusable component library.
- Preserve semantic HTML and accessible names in custom components.

#### Standalone applications

- Define state with `@context` and initialize it with `@init`.
- Serve a document with `@view`.
- Return a targeted one-shot fragment with `@patch`.
- Return data with `@command` for Datastar and ordinary HTTP clients.
- Add a shared stream with `@live`.
- Choose between `@patch`, `@command`, and `@live`.
- Read the request, body, headers, and route-specific data.
- Select PUT, PATCH, or DELETE for a mutation.
- Generate dynamic Datastar action URLs.
- Handle initialization and handler failures.
- Log from effectful code without polluting pure components.
- Use SQLite safely in the generated application boundary.
- Define multiple views and canonical route shapes.
- Serve and reference application assets.

#### Development workflow

- Run an app with and without the preview window.
- Pause live reload while preserving watch/rebuild.
- Preview one component with arguments.
- Browse components and fixtures.
- Inspect the AST, generated Roc, source-map segments, and generated HTML.
- Use the preview Dev panel: Performance, Source, and Console.
- Reduce a failure to a minimal `.rocci` reproduction.
- Use the playground and understand local versus browser limitations.
- Configure VS Code or Zed support.
- Test pure components, handlers, and HTTP responses at the right boundary.

#### Configuration, security, and shipping

- Create and validate `rocci.toml`.
- Configure windows, sizes, URLs, and development tools.
- Configure host, port, and trailing-slash behavior.
- Set allowed origins and understand the local HTTP security boundary.
- Pin and deliberately update Datastar assets.
- Build a release server for the host/container target.
- Package and run a Linux container artifact.
- Build and inspect an ad-hoc macOS `.app`.
- Diagnose platform webview prerequisites.
- Decide between a standalone `.rocci` entry and a custom app directory.

Do not create all pages merely to fill the tree. Merge adjacent recipes when a
reader cannot reasonably perform one without the other; split a page when it
has multiple independent goals or becomes hard to retrieve from search.

## Concept and explanation curriculum

Concept pages answer “why?” and “how should I think about this?” without
turning into hidden reference manuals.

| Concept | Central question |
| --- | --- |
| Rocci's compilation model | What stays Roc, what Rocci parses, what lowering generates, and where Roc type-checking begins? |
| Components are pure Roc functions | Why are render functions predictable, composable, and unsuitable for logging/effects? |
| Props, bodies, and composition | Why is there no magic `children` field or runtime component registry? |
| Structural control flow | Why do markup-producing branches use `@if`, `@for`, and `@match` rather than arbitrary embedded markup? |
| Styles and ownership | What is the difference between file CSS, scoped component CSS, and ordinary asset CSS? |
| Standalone versus custom applications | What does Rocci generate, and when should an author own `main.roc`? |
| State and effects | Where do initialization, durable state, request effects, and pure rendering live? |
| Documents, fragments, commands, and streams | How do `@view`, `@patch`, `@command`, and `@live` differ in intent and response behavior? |
| Datastar as transport | What happens from declarative browser action to server response and DOM morph? |
| One-shot versus shared live UI | When is a direct patch enough, and when does a stream justify its cost? |
| HTML and identity | Why do semantic structure and stable element IDs matter to accessibility and morphing? |
| Error boundaries and source maps | Which failures belong to Rocci parsing, Roc compilation, application effects, HTTP dispatch, or the webview? |
| Development and delivery | How do interpreted/iterative development, native compilation, server packaging, and desktop hosting relate? |
| Security model | What are the same-origin, asset, CSP, `unsafe-eval`, loopback, and desktop-host boundaries? |
| Performance model | What work happens on edit, compile, request, patch, stream poll, and bundle? What should authors measure? |

Retain the strongest parts of the current architecture, rendering-model, and
Why Roc pages, but split product-wide architecture from the smaller mental
models readers need while writing code.

## Reference architecture

Reference is exhaustive, neutral, and organized like the author-facing product.
Each entry shows syntax or invocation, context, parameters/fields, defaults,
result, errors, at least one minimal valid example, related guide/concept links,
and support status.

### Rocci language reference

Split the current single overview into a landing page and stable subpages:

| Reference page | Required coverage |
| --- | --- |
| File structure and Roc regions | Module header, imports, types, helpers, copied Roc, top-level recognition, allowed ordering, and module exposure. |
| Components | Declaration grammar, PascalCase/lower-camel mapping, params, defaulted fields, body parameter, one-root shorthand, generated function shape, and qualified calls. |
| Tags and fragments | Intrinsic tags, local and qualified components, fragments, void tags, unknown components, name restrictions, self-closing and paired calls. |
| Attributes and actions | Static/dynamic/boolean attributes, props records, hyphenated names, `@get`/`@post`/`@put`/`@patch`/`@delete`, escaping, unsupported spread/dynamic names. |
| Text and interpolation | String requirement, bare Html body values, conversions, escaping, and literal delimiters. |
| Template directives | Exact `@let`, `@if`/`@else if`/`@else`, `@for`, and `@match` grammar, scope, guards, result shape, nesting, and invalid arbitrary markup contexts. |
| CSS | File-level versus component `@css`, scoping transformation, selector behavior, ordering, emitted style metadata, and limits. |
| Fixtures | Declaration grammar, local/qualified targets, values, discovery, view/browse consumption, and validation failures. |
| Server declarations | `@context`, `@init`, `@view`, `@patch`, `@command`, `@live`, optional params, request injection, paths, methods, return contracts, generated names, and conflicts. |
| Comments, escaping, and whitespace | Roc comments, template comments, HTML comments, doc comments, `@@`, logical header lines, indentation, and blank-line behavior. |
| Generated Roc | Representative lowering for every major construct, injected imports/assets, naming, source-map expectations, and what remains for Roc to reject. |
| Grammar and deliberate limits | Compact formal grammar or generated syntax table, reserved forms, unsupported JSX-like features, and compatibility/removal notes. |

The public grammar view should be derived or mechanically checked against the
owning grammar/AST fixtures where practical, but generated structure must not
replace human explanations and examples. `AllSyntax.rocci` remains a coverage
input, not the prose source.[^all-syntax]

### Runtime and HTTP reference

- Handler matrix by declaration, default/allowed methods, body kind, success
  response, Datastar response, ordinary-client response, generated route, and
  conflict rules.
- Request fields and supported access patterns.
- Generated standalone application lifecycle: initialization, listen, state
  ownership, request dispatch, errors, shutdown, and live polling.
- Datastar asset/action reference limited to what Rocci generates or pins;
  link to upstream for the wider Datastar expression language.
- HTML runtime surface authors call directly from ordinary Roc.
- Environment variables and process arguments.
- Static assets, content types, routing, redirects, and not-found behavior.
- Logging and inspection endpoints that are author-visible in development.

### CLI reference

Give each `rocci` command its own anchor or page: synopsis, accepted inputs,
defaults, options, output/artifacts, exit behavior, platform limits, examples,
and related task links.

Required commands and surfaces, verified against the current binary before
publication: `validate`, `build` (source lowering and release server modes),
`run`, `view`, `browse`, `playground`, `render`, `inspect`, `bundle`, and
`datastar pin/update`.[^cli-readme]

Keep the source-checkout `cargo run -p rocci-cli -- …` spelling in a clearly
labeled contributor/source-build note. Lead with the installed command only
when an installed distribution actually exists.

### Configuration reference

Document the complete `rocci.toml` schema by section:

- `[app]` identity and version;
- `[[windows]]` labels, routes, dimensions, constraints, and visibility;
- `[http]` host, port, route-shape redirect policy, and safety restrictions;
- `[security]` origins and script/resource policy implications;
- `[assets]` directory, embedding, and Datastar version pin;
- `[development]` reload and developer-tool behavior; and
- `[bundle]` application entry and platform-specific metadata.

Every field row needs type, required/default value, valid range, platform,
security effect, interaction with other fields, validation diagnostic, minimal
example, and “used by” commands. The full configuration example is illustrative;
the field tables are canonical.[^configuration-reference]

### Compatibility, diagnostics, and status reference

- Supported host OS, development OS, preview webview, bundle target, server
  target, architecture, and CI coverage matrix.
- Required/tested Roc and Rust toolchain revisions.
- Current experimental/stable feature labels and known limitations.
- Syntax removals and migration entries, distinct from deprecations.
- Diagnostic catalog organized by stable code if Rocci gains codes; until then,
  organize by exact leading message and owning phase without inventing IDs.
- Changelog/release-note links and a versioning policy before multiple public
  documentation versions are exposed.

### Contributor appendices

Keep AST/tree specs, inspect tags, generated Rust node names, compiler phase
internals, and source-map segment schemas in a clearly labeled **Contributor
reference** lane. Experienced application authors can reach them, but they do
not sit between the language reference and CLI reference.

## Example system

Examples serve three different jobs and should not be conflated:

1. **Snippets** prove one construct in reference or a how-to page.
2. **Tutorial checkpoints** show a coherent app growing through a learning
   sequence.
3. **Example applications** demonstrate realistic patterns, tradeoffs, and
   project organization.

The landed `/examples/` portal is generated from `apps.toml`; extend that
catalog and generator rather than creating a second portal.[^apps-catalog][^rocci-docs-stage]
The target portal may add filters for component, standalone app, custom
`main.roc`, state, forms, one-shot patch, command, live stream, desktop, and
deployment. If those filters ship, their metadata belongs in a validated
catalog schema (or a single shared coverage manifest), not hand-maintained
labels in generated prose. Each app should expose prerequisites, concepts,
complexity, persistence/network requirements, run and verification commands,
and whether a live demo is actually available.

Use the checked-in examples as the initial canonical set:

| Example | Documentation role |
| --- | --- |
| `standalone/styling` | First static component, file/component CSS, inspect and preview |
| `standalone/counter` | First stateful app and one-shot patch |
| `standalone/live-counter` | Command plus shared stream, explicitly compared with counter |
| `standalone/handler-matrix` | Exhaustive handler contract reference, not a beginner tutorial |
| `custom/datastar` | Pattern gallery for forms, search, editing, todos, tabs, and validation |
| `custom/snake` | Advanced custom runtime and multiplayer stress example |

Use the landed ownership split consistently:

| Material | Canonical source | Published result |
| --- | --- | --- |
| Application title, summary, entry, hosting class | `examples/rocci/apps.toml` | Generated examples index and catalog-driven local packaging selection[^ops-site-package] |
| Tutorial/pattern prose for one app | Colocated `index.rocdown` and optional additional pages | `/examples/<id>/…` |
| Concise explanation of one declaration | Attached `## ` lines immediately above its top-level `@` declaration | “Declarations” section on the generated source page |
| Complete app source | Checked-in `.rocci`, authored `.roc`, config, and selected assets | `/examples/<id>/source/…` through safe staged snippets |
| Cross-app curriculum, language concepts, and canonical contracts | Manual pages under `docs/` | `/docs/…`, linking into examples |
| Staged pages and snippets | `rocci-docs` output | Derived `dist/example-docs`; never edited or committed as source |

The manual should link to generated example and source pages rather than paste
divergent copies. App prose may summarize a relevant syntax form, but the
canonical contract remains in `/docs/reference/`. Declaration comments should
explain purpose, inputs, output, and noteworthy invariants local to that
declaration; they should not become miniature tutorials or duplicate the whole
component/handler reference.

The generated index currently appends a list of Rocdown examples. Remove that
cross-product list from the Rocci examples route as part of the content
separation; link to the separate Rocdown product lane instead. Live hosting
remains an explicit catalog/publishing concern and never a prerequisite for
using local documentation.[^rocci-docs-stage][^app-docs-plan]

## Page contracts and writing standard

### Shared page contract

Every page contains:

- a concrete title and one-sentence description written in reader language;
- one primary documentation kind and one intended reader state;
- prerequisites or “none beyond the docs prerequisites”;
- the version/support context when behavior is platform- or release-sensitive;
- meaningful headings and stable link targets;
- tested code or an explicit partial/pseudocode label;
- links to the canonical reference and one useful next action; and
- a repository edit/feedback route.

Avoid generic introductions, repeated product slogans, unexplained “simple” or
“obvious,” and sections named only “Overview.” Put the outcome first. Define a
term before its first required use and add it to the glossary only when readers
will encounter it across multiple pages.

### Tutorial contract

- State what the learner will make, see, and know how to do.
- List exact prerequisites, starting files, expected duration, and final files.
- Keep the learner on a supported path; postpone alternatives and deep theory.
- Provide a verification after every meaningful stage.
- Show expected terminal/HTTP/visual output, including common divergence.
- Never rely on code created in an unnumbered aside.
- End with a complete source comparison and ability checklist.

### How-to contract

- Title as a task: “Add…”, “Configure…”, “Diagnose…”.
- State when the recipe applies and the minimum starting state.
- Give the shortest safe procedure, then verification.
- Put alternatives, tradeoffs, and deeper explanation after the working path.
- Link rather than redefine syntax or conceptual background.

### Reference contract

- Mirror the product structure and use consistent tables.
- State accepted forms, defaults, outputs, errors, limits, and interactions.
- Use minimal examples for lookup; link to a guide for realistic composition.
- Keep planned, removed, deprecated, experimental, and stable labels distinct.
- Record the owning code/test source in contributor metadata or a coverage map.

### Concept contract

- Begin with the practical confusion the page resolves.
- Explain the model, boundary, reasons, and consequences.
- Use one grounded example and compare plausible alternatives when useful.
- Avoid step-by-step instructions and exhaustive option tables.

## Rocdown authoring and verification

Rocdown is the implementation medium. Authors should use ordinary Markdown for
prose, bounded article blocks for notes/steps/figures, and file includes for
canonical code. This plan does not require new Rocdown syntax.

### Canonical source policy

Maintain a documentation coverage manifest—data file or checked test fixture,
not prose—that maps each author-visible feature to:

- owning crate and implementation source;
- contract test or syntax fixture;
- canonical reference page/anchor;
- at least one guide, tutorial, or example where appropriate; and
- status: current, experimental, removed, or planned.

The manifest should fail CI for an implemented public feature with no reference
owner, a removed feature still labeled current, or a reference link to a
missing page. It should not force every internal function into public docs.

### Example verification levels

| Level | Meaning | Required verification |
| --- | --- | --- |
| Literal | A name, path, or one-line syntax fragment | Parser/link validation where applicable |
| Compile | Complete `.rocci` component/module | Rocci parse/lower plus Roc compilation against the pinned toolchain |
| Run | App or command expected to start/exit | Command execution with exit status, bounded timeout, and golden stdout/stderr where stable |
| HTTP | Handler/route contract | Start headless, issue deterministic request, assert status/content type/body shape, stop cleanly |
| Visual | Layout or interaction result | Render/screenshot at defined viewports plus keyboard/accessibility review; visual snapshots support, not replace, semantic assertions |

Tutorial code should reach Compile at minimum; server tutorial checkpoints
reach HTTP. Reference fragments may use Literal only when they are deliberately
non-complete and labeled as such.

### Required checks

During authoring:

```sh
cargo test -p rocci-docs
cargo run -q -p rocci-docs -- \
  --catalog examples/rocci/apps.toml \
  --output dist/example-docs
cargo run -q -p rocci-rocdown-cli -- check docs
cargo run -q -p rocci-rocdown-cli -- check site
cargo run -q -p rocci-rocdown-cli -- test docs
cargo run -q -p rocci-rocdown-cli -- build docs
```

The implementation phase may add focused documentation harness commands, but
the existing product commands remain the public base. CI should additionally
check:

- all internal links, aliases, headings, local images, and includes;
- syntax highlighting language tags and unescaped output;
- code compilation/runtime levels declared by each example;
- agreement between the app catalog, staged routes, source inventory, and
  live-app packaging selection;
- navigation reachability and orphan pages;
- duplicate canonical facts or titles where mechanically detectable;
- spelling of reserved Rocci forms and CLI flags against an exported inventory;
- accessibility semantics for generated pages and example output;
- deterministic output and an unchanged prior tree on failed build; and
- a clean source tree after checks.

The staging crate deliberately does not compile Roc, render HTML, or package
servers.[^rocci-docs-readme] Its focused tests remain responsible for catalog,
inventory, extraction, path safety, and deterministic staging. Separate Rocci
compile/run/HTTP checks are responsible for whether the documented applications
actually work. Site checks consume the staged tree only after generation.

## Navigation, retrieval, and reading experience

- Keep the global docs sidebar shallow: section, current group, current page.
  Use section landing pages for the full catalog.
- Add previous/next only to tutorials and curated learning sequences; reference
  pages should prioritize siblings and search.
- Search results show content kind, product, title, summary, matched heading,
  and support/version label.
- Support exact retrieval for every directive, declaration, CLI command, config
  field, common diagnostic phrase, and glossary term.
- Provide copyable code, visible language labels, and direct heading links. Add
  line emphasis only where it communicates the current step.
- On narrow screens, code and tables scroll without moving the whole page;
  navigation and outline remain keyboard accessible.
- Print and no-JavaScript output retain the article, code, callouts, and link
  destinations.
- Provide a machine-readable Markdown/LLM view only from the same canonical
  resolved article; do not maintain a second AI-specific prose corpus.

## Versioning and lifecycle

Rocci is experimental, so freshness needs an explicit policy before the corpus
grows:

1. Documentation on `main` describes `main` and is labeled **development**
   until a versioned release channel exists.
2. A release page documents only behavior present in that release. Never use a
   future plan to fill a release gap.
3. Syntax or behavior changes update, in the same change, the owning README,
   canonical reference, affected guide/tutorial, examples, coverage manifest,
   and migration note.
4. Removed behavior leaves the current reference and moves to a concise
   migration/history entry. Do not teach obsolete syntax in beginner pages.
5. Platform and toolchain claims have an “as tested” revision/date and an owner.
6. Human documentation review is required for public-contract changes even
   when generated inventories and tests pass.

Do not implement a version selector until there are at least two supported
public versions. Define route and artifact policy first so later versioning
does not break canonical links.

## Ownership and maintenance

| Surface | Primary evidence owner | Documentation owner |
| --- | --- | --- |
| `.rocci` grammar/lowering/source maps | `rocci-template` code and tests | Language reference plus affected learning pages |
| Standalone dispatch/runtime behavior | `rocci-cli` dispatcher/run tests | Runtime/HTTP reference and application guides |
| Shared config | `rocci-core` config parser/tests | Configuration reference and shipping guides |
| Preview window/desktop bundle | `rocci-desktop` and `rocci-cli` | Workflow and desktop guides |
| Component/view primitives | `rocci-ui` and examples | Component guides/reference where author-facing |
| Editor behavior | LSP crates and editor packages | Editor setup and workflow how-tos |
| Example applications | Example source and its tests | Example page and linked tutorials |
| Documentation shell/catalog | Rocdown implementation | Build/validation infrastructure only; not Rocci product prose |

Add a pull-request checklist requiring authors to answer:

- Is this a public contract change?
- Which canonical reference owns it?
- Which learner/task pages are affected?
- Does a tested example prove the new behavior?
- Are support, migration, and known-limit labels accurate?
- Did the change introduce or remove a glossary term?

Run a quarterly documentation review while the product is changing quickly:
sample the first-run path on a clean environment, audit reference coverage,
review failed searches/support questions, and remove stale duplication. This
is an editorial review, not permission to rewrite stable routes casually.

## Delivery plan

### Phase 0 — freeze scope, vocabulary, and truth map

**Bound**

- Approve that `/docs/` is Rocci-first and Rocdown curriculum is separate.
- Approve reader personas, top-level lanes, canonical vocabulary, and route
  policy.
- Inventory all 24 current pages as keep/split/rewrite/move/retire.
- Inventory all six cataloged apps, their colocated Rocdown, attached `## `
  declaration docs, generated routes, source inventory, and hosting class.
- Build the feature-to-source-to-page coverage manifest for shipped Rocci.
- Mark planned and removed behavior distinctly.
- Choose the actual installation/release story to document; do not invent one.

**Exit**

- Every shipped author-facing Rocci feature has one evidence owner and proposed
  canonical reference owner.
- Every existing page has a disposition and redirect/alias plan.
- Every generator-owned input and derived output has one documented owner; no
  proposed manual page duplicates a generated app/source page.
- A maintainer approves the Rocci-only scope and vocabulary.

### Phase 1 — rebuild the portal and first-success path

**Bound**

- Replace the current portal with intent-based entry points.
- Write prerequisites, installation, path chooser, Roc-for-Rocci, web
  foundations, and Rocci-in-five-minutes.
- Rewrite the first-component tutorial against one checked-in canonical source.
- Link tutorial checkpoints to the landed `/examples/styling/` and generated
  source routes; do not create parallel full-source copies under `docs/`.
- Add explicit outcomes, verification, complete source, and next actions.
- Test on clean macOS and Linux environments; document Windows only to the
  level actually verified.

**Exit**

- A programmer unfamiliar with Rocci can reach a visible component from a
  clean supported environment using only public pages.
- Every command and code block on the path passes its declared verification.
- No step depends on undocumented source-tree knowledge.

### Phase 2 — teach the complete core application journey

**Bound**

- Deliver the first standalone app, composition, command/live, custom
  `main.roc`, and shipping tutorials.
- Teach error handling, logging, inspection, and accessibility inside the
  journey rather than as optional afterthoughts.
- Provide checkpoint source and deterministic HTTP assertions.
- Establish the explicit one-shot-versus-live comparison.

**Exit**

- A learner can build, debug, and package a nontrivial Rocci app and explain
  the boundaries between pure components, effects, HTTP responses, and live
  transport.
- Tutorial checkpoints compile and server checkpoints pass HTTP tests.

### Phase 3 — establish exhaustive reference

**Bound**

- Split and complete the language reference.
- Publish runtime/HTTP, Rocci-only CLI, configuration, compatibility, and
  status references.
- Move AST/tree material into contributor appendices.
- Add exact search aliases for syntax names, command flags, config keys, and
  common terminology.

**Exit**

- Coverage manifest has no shipped public feature without a canonical reference.
- Each reference entry states accepted forms, defaults, outcomes, errors,
  limits, example, and support status.
- Dense syntax fixture and CLI/config inventories reconcile with the reference.

### Phase 4 — fill task and concept coverage

**Bound**

- Prioritize how-to pages from real author workflows and known failure modes.
- Split and revise current concepts into the proposed mental-model set.
- Create glossary and focused troubleshooting routes.
- Cross-link learning, task, concept, and reference pages without duplicating
  canonical facts.

**Exit**

- Every tutorial capability has a post-tutorial task path and reference link.
- Every reference group has at least one grounded guide or example where useful.
- Search evaluation finds the right page for a maintained query set covering
  beginner language, expert syntax, commands, config, runtime, and failures.

### Phase 5 — curate and harden the generator-backed example system

**Bound**

- Keep `apps.toml` as the single published-app inventory and `/examples/` as
  the single application portal.
- Remove the Rocdown run-command list from the generated Rocci index and route
  readers to the separate Rocdown lane.
- Give each cataloged app an audience, purpose, prerequisites, run/test path,
  concepts, complexity, persistence needs, and support status. Extend the
  catalog schema only if generated filtering or cards require structured data.
- Review every colocated app page and attached declaration comment against the
  manual/example/declaration ownership split.
- Make staging write a complete sibling tree and replace the previous output
  only after success; a failed catalog, copy, or write keeps the previous tree.
- Add live demos only for explicitly supported apps and keep source/local paths
  fully useful without them.

**Exit**

- Every advertised example builds and runs at its declared level.
- Example source shown in docs is byte-identical to or mechanically selected
  from the checked-in canonical source.
- Staging is deterministic and atomic, and generated output remains derived.
- The portal distinguishes learning examples, reference matrices, advanced
  stress examples, and publicly hosted demos.

### Phase 6 — harden authoring, CI, and accessibility

**Bound**

- Enforce page metadata/content-kind contracts.
- Enforce the example verification levels and coverage manifest.
- Run `rocci-docs` tests and staging before site checks; run separate
  compile/run/HTTP verification for cataloged apps because staging does not.
- Add clean-tree, link, orphan, route-alias, deterministic-output, and
  accessibility checks.
- Review responsive code/table behavior, keyboard navigation, print,
  no-JavaScript, forced colors, reduced motion, and zoom.
- Document the contributor workflow and public-contract checklist.

**Exit**

- A documentation-only pull request has one reliable local check path matching
  CI.
- A behavior change cannot silently leave its canonical reference unowned.
- Blocking accessibility failures and broken runnable examples fail CI.

### Phase 7 — measure and revise from use

**Bound**

- Maintain an anonymous or privacy-preserving query set from site search,
  repository issues, discussions, and user tests.
- Run first-use sessions with programmers from the Roc-first and web-first
  entry states.
- Measure success, time to first visible result, failed steps, zero-result
  searches, task completion, stale example rate, and reference coverage.
- Revise navigation and content from evidence; do not optimize for raw page
  views or tutorial completion alone.

**Exit**

- At least two distinct newcomer entry states complete the first-success path
  without maintainer intervention.
- Maintained retrieval queries meet their agreed success threshold.
- Every observed failure has a page/content fix, product issue, or explicit
  non-goal disposition.

## Acceptance criteria

- The main documentation portal clearly teaches Rocci and routes Rocdown to a
  separate product lane.
- A supported newcomer path works from clean environment to visible result and
  then to a stateful application.
- Tutorials, how-tos, concepts, and reference have distinct jobs and consistent
  page contracts.
- Every shipped author-facing syntax form, command, option, configuration field,
  runtime response contract, platform restriction, and deliberate limit has one
  canonical reference owner.
- Experienced authors can retrieve exact answers by directive, handler kind,
  command, flag, config key, diagnostic phrase, or glossary term.
- All runnable tutorial and server examples pass their declared compile/run/HTTP
  verification against the pinned toolchain.
- Examples shown in prose do not drift from checked-in canonical sources.
- `/examples/` is produced only from `apps.toml`, colocated Rocdown, attached
  declaration docs, and inventoried source; failed generation preserves the
  previous staged tree.
- Current, experimental, planned, removed, and unsupported behavior are visually
  and textually distinct.
- Documentation remains useful with keyboard only, zoom, reduced motion, forced
  colors, print, narrow screens, and no JavaScript.
- Documentation changes are reviewed and validated like product changes, while
  prose remains pleasant to edit as ordinary Rocdown.

## Decision gates

Human approval is required before implementation for:

1. making `/docs/` Rocci-first and choosing the separate Rocdown route lane;
2. the final top-level names (`Start`, `Tutorials`, `How to`, `Concepts`,
   `Reference`, `Examples`, `Troubleshooting`, `Status`);
3. the supported installation/release channel and version-label policy;
4. the first canonical tutorial application and whether SQLite belongs in the
   beginner path;
5. the coverage-manifest format and which omissions fail CI;
6. whether discoverability metadata for generated example filters extends
   `apps.toml` or lives in the shared documentation coverage manifest;
7. the minimum platform matrix for claiming the installation path works; and
8. any new Rocdown feature proposed solely for documentation presentation.

[^root-readme]: Current product, workspace, run, packaging, CLI, site, and validation overview.
[^template-readme]: Implemented `.rocci` language, lowering, handler, whitespace, generated-Roc, and limitation contract.
[^cli-readme]: Implemented Rocci command and development-inspector surface.
[^docs-config]: Current standalone docs navigation across start, build, understand, reference, and resources.
[^site-config]: Current main-site mount of `docs/` at `/docs/` and combined navigation.
[^docs-index]: Current compact documentation portal and its section links.
[^rocci-reference]: Current public one-page Rocci language overview.
[^cli-reference]: Current public reference combining four product/tool binaries.
[^configuration-reference]: Current public complete-shape configuration page.
[^examples-index]: Current mixed Rocci/Rocdown example catalog.
[^troubleshooting]: Current mixed-product symptom-led troubleshooting page.
[^all-syntax]: Dense syntax coverage fixture for parser/lowering inspection.
[^site-plan]: Existing site-architecture scope for routes, layouts, content ownership, and publishing boundaries rather than documentation coverage.
[^app-docs-plan]: Exploratory ownership and rollout plan for application documentation and live example origins.
[^rocci-docs-readme]: Checked-in application catalog, staging command, colocated Rocdown pages, and complete source-tree output.
[^rocci-docs-stage]: Implemented catalog index, authored-page copy, source inventory pages, safe snippets, attached declaration documentation, and current replace-in-place staging behavior.
[^rocci-docs-tests]: Focused validation, inclusion/exclusion, safe include, declaration documentation, route, and determinism tests for staging.
[^apps-catalog]: Six cataloged Rocci applications, their entry points, summaries, and docs/live hosting selection.
[^site-workflow]: Site packaging stages application documentation and builds the two catalog-selected live application artifacts before packaging rocci.dev.
[^ops-ci]: Local CI stages `dist/example-docs` before checking the unified site and standalone documentation catalog.
[^ops-site-package]: Local site packaging uses `rocci-docs --print-live` so the catalog selects which app servers are built.
[^diataxis]: Four documentation needs and the distinction between learning, task, information, and understanding work.
[^stripe-docs]: Intent-led developer portal with quickstarts, sample projects, product paths, and API reference.
[^stripe-quickstarts]: End-to-end quickstarts with connected code and stepwise implementation.
[^django-docs]: Explicit documentation map separating tutorials, topic guides, how-to recipes, and reference.
[^mdn-learn]: Bounded beginner-to-comfortable learning pathway with prerequisites, skill checks, and challenges.
[^rust-docs]: Separate learning, example, tool, API, error, and advanced-reference shelves for programmers with different needs.
[^stack-ia-research]: Later review: visible Diátaxis curriculum and Rocci-only `/docs/` scope hide the layered stack.
[^stack-ia-plan]: Follow-on implementation: stack-layer nav, `/docs/rocdown/`, academy-chrome removal, clean-cut URLs.
