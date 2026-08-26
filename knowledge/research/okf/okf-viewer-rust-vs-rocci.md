---
type: Research Report
title: OKF viewer as a Rust HTML app versus a finished Rust-plus-Rocci shell
description: Dropping Rocci does not drop Datastar; the chosen product is okmate (Askama, Axum, official Datastar SDK) in a root crate that depends only on okf. Public rocci.dev mount stays a prefixed static build copy; live operations need a hosted origin.
tags: [domain/okf, domain/rocci-okf, integration/roc, concern/architecture, concern/rendering, concern/tooling]
status: draft
generated: { by: process:cursor, at: 2026-08-26T08:05:00Z }
stale_after: 2026-11-26
authority: exploratory
owners: [human:nils]
sources:
  - id: catalog-shell
    resource: ../../decisions/rust-catalog-rocci-shell.md
    title: Use a Rust catalog and a Rocci documentation shell
    author: process:okf-migration
    last_modified: 2026-08-24
  - id: static-okf
    resource: ../../decisions/static-okf-boundary.md
    title: Strict OKF Markdown and static rendering boundary
    author: process:okf-migration
    last_modified: 2026-08-24
  - id: generator
    resource: ../../architecture/rocdown-documentation-compiler.md
    title: Rocdown documentation generator
    author: process:cursor
    last_modified: 2026-08-24
  - id: compile-research
    resource: okf-compile-render-cost.md
    title: OKF preview compile and render cost after load-performance work
    author: process:cursor
    last_modified: 2026-08-24
  - id: compile-follow-ons
    resource: ../../plans/okf/okf-compile-render-follow-ons.md
    title: Deferred OKF compile and render follow-ons
    author: process:cursor
    last_modified: 2026-08-24
  - id: compile-status
    resource: ../../status/okf-compile-render-cost.md
    title: OKF preview compile and render cost results
    author: process:cursor
    last_modified: 2026-08-19
  - id: generation-research
    resource: ../rocci/rocci-components-in-generation.md
    title: Rocci components inside the content generation pipeline
    author: process:cursor
    last_modified: 2026-08-24
  - id: settings-ux
    resource: ../../plans/okf/settings-ux.md
    title: Settings UX for knowledge roots
    author: process:cursor
    last_modified: 2026-08-25
  - id: okmate
    resource: ../../plans/okf/okmate.md
    title: Okmate — extractable Rust OKF mate
    author: process:cursor
    last_modified: 2026-08-26
  - id: rust-datastar
    resource: ../../plans/okf/okf-viewer-rust-datastar.md
    title: In-place rocci-okf Askama rewrite (superseded as vehicle)
    author: process:cursor
    last_modified: 2026-08-26
  - id: host-surfaces
    resource: ../../plans/okf/okf-viewer-host-surfaces.md
    title: Short-term OKF viewer host surfaces (superseded)
    author: process:cursor
    last_modified: 2026-08-26
  - id: datastar-crate
    resource: ../../../crates/rocci-datastar/README.md
    title: Rust Datastar spec, SSE framing, signals, and assets
    author: process:git
    last_modified: 2026-08-24
  - id: datastar-eco
    resource: ../rocci/method-role-handlers-datastar-ecosystem.md
    title: Implemented method-role handlers versus Datastar SDKs
    author: process:cursor
    last_modified: 2026-08-23
  - id: site-lane
    resource: ../../plans/site/okf-viewer-site-lane.md
    title: Mount the OKF knowledge viewer on rocci.dev
    author: process:cursor
    last_modified: 2026-08-24
  - id: presentation
    resource: ../../../crates/rocci-okf/src/presentation.rs
    title: Dual Rust html_page_for and Rocci apply review site
    author: process:git
    last_modified: 2026-08-25
  - id: settings-rs
    resource: ../../../crates/rocci-okf/src/settings.rs
    title: Settings article HTML and POST actions
    author: process:git
    last_modified: 2026-08-25
  - id: okf-build
    resource: ../../../crates/rocci-okf/runtime/OkfBuild.roc
    title: Apply runtime that splices nav and article into OkfTheme
    author: process:git
    last_modified: 2026-08-24
  - id: okf-theme
    resource: ../../../crates/rocci-okf/templates/OkfTheme.rocci
    title: KnowledgeShell, PageOutline, and ConceptMeta composition
    author: process:git
    last_modified: 2026-08-24
  - id: settings-rocci
    resource: ../../../crates/rocci-okf/templates/Settings.rocci
    title: Stub SettingsPage that is not the live renderer
    author: process:git
    last_modified: 2026-08-25
  - id: review-rocci
    resource: ../../../crates/rocci-okf/templates/ReviewQueue.rocci
    title: ReviewQueue templates compiled but unused by OkfBuild
    author: process:git
    last_modified: 2026-08-24
  - id: runtime-rs
    resource: ../../../crates/rocci-okf/src/runtime.rs
    title: Embedded OkfTheme, ConceptMeta, ReviewQueue, Settings sources
    author: process:git
    last_modified: 2026-08-25
  - id: okf-readme
    resource: ../../../crates/rocci-okf/README.md
    title: view host auto, Rust shell when roc is missing
    author: process:git
    last_modified: 2026-08-25
  - id: ui-goto
    resource: ../../../crates/rocci-ui/assets/goto.js
    title: Shared chrome script that swaps #okf-main
    author: process:git
    last_modified: 2026-08-24
  - id: agents
    resource: ../../../AGENTS.md
    title: Layer owners and no Rust interpretation of Rocci templates
    author: process:git
    last_modified: 2026-08-25
  - id: askama
    resource: https://crates.io/crates/askama
    title: Askama type-safe Rust HTML templates
    author: organization:crates-io
  - id: maud
    resource: https://crates.io/crates/maud
    title: Maud compile-time HTML in Rust
    author: organization:crates-io
  - id: server-state
    resource: ../../decisions/server-owned-state.md
    title: Durable application state stays server-owned
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: okf-app
    resource: ../../plans/okf/rocci-okf-app.md
    title: Standalone Rocci OKF review and query application
    author: process:cursor
    last_modified: 2026-08-17
  - id: tools-research
    resource: okf-tools-and-workflows.md
    title: Agent-native OKF operations and review workflows
    author: process:codex
    last_modified: 2026-08-17
---

# OKF viewer as a Rust HTML app versus a finished Rust-plus-Rocci shell

## Scope and authority

This is exploratory product research, not an approved decision and not an
implementation plan. It asks which presentation owner the `rocci-okf` review
viewer should have now that the dual path is visible in code: a Rust
application that emits all HTML, or a finished Rust-catalog plus Rocci-shell
pipeline that assumes a Roc applicator is available.[^presentation][^okf-readme][^catalog-shell]

It does not reopen the portable engine boundary. `okf` stays UI-neutral.
Knowledge records stay inert Markdown. Catalog checks must not require
Roc.[^static-okf][^agents][^generation-research]

Cost measurements and the generate/apply split already live in the
[compile/render research](/research/okf/okf-compile-render-cost.md) and its
[follow-on plan](/plans/okf/okf-compile-render-follow-ons.md). This record
uses those facts and asks the product question those records left open:
whether skip-Roc should become the *product*, or whether the Rocci path
should become the *only* static shell.[^compile-research][^compile-follow-ons]

## The current viewer is neither option

Default `view` already encodes the fork. When `roc` is on PATH, templates
lower to Roc, `OkfBuild` applies `OkfTheme.knowledgeShell`, and native apply
writes staging. When `roc` is missing, `build_review_site_pure_rust` writes a
parallel document with `html_page_for`. If apply omits a path, the write step
fills it with the same Rust shell.[^presentation][^okf-build][^okf-readme][^compile-status]

Both paths still get most of their pixels from Rust strings:

- Markdown bodies come from the engine as `article_html`.[^static-okf]
- Sidebar HTML is `render_nav_tree`, spliced into Roc as `nav_html` or inlined
  by `html_page_for`.[^presentation][^okf-build]
- Home governance, the review queue, and `/settings/` bodies are Rust
  (`render_home_page_governance`, `render_review_page`,
  `settings::render_article`).[^presentation][^settings-rs]
- Concept-meta HTML exists twice: `render_concept_meta` on the Rust path and
  `ConceptMeta.rocci` on the apply path.[^presentation][^okf-theme]
- `ReviewQueue.rocci` and `Settings.rocci` are compiled into the renderer
  hash and are not called from `OkfBuild.render_page`. The live settings
  surface is `settings.rs`. `Settings.rocci` is a thinner stub than that
  surface.[^runtime-rs][^review-rocci][^settings-rocci][^settings-rs][^settings-ux]

The viewer is also not a Datastar app. Pages are static documents plus
`goto.js` column swaps, session POST, reload script, and ordinary HTML forms.
There is no `@method:role` matrix and no `datastar.js` on the review
tree.[^ui-goto][^okf-readme][^settings-rs]

So the choice is not “keep the hybrid.” The hybrid is two shells, two
concept-meta renderers, and templates that look owned while Rust still
authors the widgets.

## Two products are hidden in one question

Treat these as different surfaces before picking a template owner.

| Surface | What it is today | Good owner |
| --- | --- | --- |
| Static review documents | Prebuilt HTML per concept, collection, dashboard, review queue | One catalog-plus-shell pipeline |
| Settings and session | Loopback POST, `okf.toml`, desktop `pick-folder` IPC | Rust host, like preview chrome |
| Cmd-K / live reload | Shared `rocci-ui` script and preview assets | Stay JS + host; not Rocci compile |

`/settings/` is already specified as one-shot POST without live SSE, and the
settings-UX plan keeps `Settings.rocci` out of bound as the live
renderer.[^settings-ux][^settings-rs] Moving that page onto Rocci would mean
adopting handlers and Datastar, not “using the theme.” That is a different
migration.

A public `/knowledge/` lane can copy the static review tree. It does not need
a live Roc compiler on the edge if `build` already wrote HTML.[^site-lane]

`check`, `inspect`, and `search` stay Rust regardless of the viewer
choice.[^static-okf][^okf-readme]

## Option A — Pure Rust application

### What it would take

Delete or stop compiling the apply path for default `view` and `build`.
Keep `generate` only if you still want `pages.json`. Make `html_page_for` /
`render_*` the only document writer. Keep settings, session, and desktop IPC
where they are.

That is compile/render research option A taken as the *product*, which the
follow-on plan already called a product inversion relative to the catalog-shell
decision: machines with `roc` would stop serving `OkfTheme`.[^compile-research][^compile-follow-ons][^catalog-shell]

### “Proper Rust HTML templating”

The workspace has no Askama, Maud, Tera, or similar crate in `rocci-okf`.
Adding one would introduce a second markup language beside `.rocci`. The
catalog-shell decision and agent instructions already reject growing an
unrelated docs-template language in Rust, and reject interpreting Rocci
templates in Rust merely to skip compiling a theme.[^catalog-shell][^agents][^generator]

Rust string builders plus view structs are the current style
(`StatCardView`, `render_stat_grid`, settings forms). They are ugly and they
are honest: the host already owns those strings. A typed HTML builder in
Rust (functions that return escaped `String`, not a new file syntax) would
be a cleanup, not a product language.

If the motive for “proper templating” is maintainability of
`presentation.rs`, the cheaper fix is to delete the unused Rocci copies, or
to stop writing the Rust copies, not to add Askama.

### When A is the right product

Choose A if all of these stay true:

1. `Rocci Knowledge.app` and agent `view` must work with no `roc` and no
   shipped applicator binary.[^okf-readme]
2. You will not invest in wasm apply-to-disk plus a prebuilt
   `components.wasm` in the release archive (the follow-on already poses
   that question and has not answered it).[^compile-follow-ons]
3. OKF chrome is allowed to diverge from Rocdown’s `RocdownTheme` and to keep
   duplicating “On this page” and shell class names in Rust.[^generation-research][^generator]

A is a coherent Rust application. It matches how settings and preview chrome
already work. It is a reversal of “visible site chrome belongs in Rocci” for
this product only.[^catalog-shell]

## Option B — Finished Rust catalog plus Rocci shell

### What “truly rust + rocci” means

It does **not** mean putting Markdown or OKF metadata into Roc constructors.
It means the same split Rocdown already ships:[^catalog-shell][^generator][^generation-research]

1. Rust owns discovery, validation, routing, nav data, article HTML, and
   host orchestration.
2. Rocci owns the document chrome and the remaining product widgets
   (`OkfTheme`, `ConceptMeta`, and, if you want one markup owner,
   `ReviewQueue`).
3. Apply writes the files the server actually serves. Drop
   `html_page_for` once apply is complete. Do not keep a second shell “just
   in case.”

`OkfBuild` already splices trusted Rust HTML with
`Html.dangerously_include_unescaped_html`. That is the intended bridge, not a
temporary hack.[^okf-build][^generator]

Finishing B is mostly deleting the Rust chrome fallback and teaching
`ReviewQueue` (and only the *static* parts of settings, if any) to consume
the same view records Rust already builds. It is not a Datastar rewrite.

### Roc available is the wrong assumption

Requiring `roc` on PATH for every `view` is stronger than the product needs.

Three availability stories:

| Story | Who compiles | Who runs apply | Fits |
| --- | --- | --- | --- |
| Local `roc` | Developer machine | Native subprocess | Current default when `roc` exists |
| Cached applicator | First machine that had `roc` | Later `view` in the same cache | Already shipped for theme-stable rebuilds |
| Shipped applicator | CI / release | Wasmtime or bundled native apply | Follow-on Phase 3 / open question; not shipped |

The catalog-shell decision already names a two-tier renderer cache and a
Wasmtime host. Embedding the Roc *compiler* in Rust is not available and is
a recorded non-goal.[^catalog-shell][^generation-research][^compile-follow-ons]

So B can mean “assume an applicator exists,” not “assume the user installed
Roc.” `ROCCI_REQUIRE_ROC=1` and `--host native` already express the compiler
story. A release that includes `components.wasm` (once wasm writes disk)
would express the runtime story without a local compiler.[^okf-readme][^compile-follow-ons]

Until that binary ships, B still leaves `Knowledge.app` and no-`roc` agent
boxes on the Rust shell, which is today’s hybrid.

### When B is the right product

Choose B if the review site is supposed to look and evolve like Rocdown
chrome: one `.rocci` theme, shared `PageOutline`, view records in, HTML
out.[^catalog-shell][^okf-theme][^generation-research]

That is what compile/render option C already treated as the durable
path.[^compile-research] Follow-ons then refused to make skip-Roc the
long-term *default* while templates keep changing, because that publishes an
unsupported authoring look.[^compile-follow-ons]

## Comparison

| Concern | Pure Rust (no new DSL) | Finished Rocci apply | Current hybrid |
| --- | --- | --- | --- |
| Markup owner | Rust functions / `format!` | `.rocci` theme + widgets | Both, and they drift |
| `roc` on PATH | Never required | Required unless applicator is cached or shipped | Optional; changes the pixel owner |
| First-open cost | Parse + write | Cold `roc build` unless cache or shipped wasm | Pays compile even when write is the site |
| Settings / IPC | Already fits | Must stay Rust unless Datastar is adopted | Rust (correct) |
| Dogfood | None | Theme and ConceptMeta | Compiles unused Settings/ReviewQueue |
| Rocdown alignment | Diverges on purpose | Matches catalog-shell | Pretends to match |
| Release app | One binary | Binary plus applicator artifact | One binary, two looks |
| New template crate | Unnecessary; harmful if it is Askama | Unnecessary | Unnecessary |

## Verdict

Do not migrate to a Rust HTML template language. That answers the wrong
pain. The pain is two owners, not the absence of Askama.[^catalog-shell][^agents]

Do not treat settings, folder pick, or session as Rocci work. Those are host
surfaces and should stay Rust.[^settings-ux][^settings-rs]

For the **static review shell**, prefer finishing Rust-plus-Rocci (option B)
and treat “Roc is available” as “an applicator is available”: local `roc`,
the existing renderer cache, or a later shipped wasm/native apply. That
keeps the approved chrome owner and stops the Rust `html_page_for` twin
once apply is the source of truth.[^catalog-shell][^compile-research][^okf-build]

If the product will **not** ship or cache that applicator, and
`Knowledge.app` / no-`roc` `view` must stay first-class, then choose option
A: one Rust HTML writer, delete the apply path from default `view`/`build`,
and leave `.rocci` files only if some other tool still compiles them. That
is an honest Rust app. It is also a local exception to the catalog-shell
decision and should be recorded as such if taken.[^compile-follow-ons][^okf-readme]

Do not keep the hybrid as a destination. A fallback that renders a different
shell trains the team to edit the wrong file.

## Product choice

The implementation plan is [okmate](/plans/okf/okmate.md): a new root
crate with Askama 0.16, Axum, and the official Datastar SDK, depending
only on `okf`. The in-place
[rocci-okf rust+datastar](/plans/okf/okf-viewer-rust-datastar.md) rewrite
is not the vehicle. This record remains the evidence.[^okmate][^rust-datastar]

Still open outside that plan:

- Whether wasm apply-to-disk and a prebuilt `components.wasm` land; that is
  the compile/render follow-on, still gated, and not needed for Horizon A.[^compile-follow-ons]
- Review approve / comment and in-UI agent jobs; those stay
  [rocci-okf-app](/plans/okf/rocci-okf-app.md).[^okf-app]
- Public `/knowledge/` packaging and the local-first publication
  amendment; [site lane](/plans/site/okf-viewer-site-lane.md) copies
  whatever static tree `build` emits.[^site-lane]

## First principles if catalog-shell does not bind

The section above treats “do not grow a Rust docs-template language” as given.
If that constraint is lifted, the question is only: what kind of application
is the viewer, who authors it, and what does a user feel.[^catalog-shell]

### What the application actually is

It is a Rust host that also emits documents. Durable writes go to `okf.toml`
and the preview session file. Mutations are loopback form POSTs and desktop
IPC. In-page navigation is a shared JS swap, not Datastar. `check` never
renders. The process the user launches is `rocci-okf` or
`Rocci Knowledge.app`.[^settings-rs][^settings-ux][^ui-goto][^okf-readme]

That is a small server-rendered app plus a static snapshot, not a Rocci
application. A Rocci apply step is an out-of-process theme compiler. That
shape is a good fit for Rocdown, where the product *is* the theme and pages
are many. It is an extra runtime for OKF, where chrome, badges, and settings
change with the same Rust classifiers that already own the data.[^generator][^presentation]

So unbound, the design choice is:

| Design | Runtime model | Natural when |
| --- | --- | --- |
| Askama-class templates in the binary | One program; handlers and markup share Rust types | The host *is* the app |
| Rocci apply | Two programs; JSON/files are the contract | The theme is a separate product surface |

`format!` concatenation is not a third design. It is the current
implementation debt on the Rust side.[^presentation]

### Authoring

Assume one owner (the person changing `classify_concept_action` also changes
the banner).

Rust templates (Askama-like HTML files, or Maud macros) put the loop in one
compiler. A settings field, a review column, and a badge condition live next
to the struct that supplies them. `rustc` is the feedback. Askama stays
readable as HTML; Maud stays in rust-analyzer and looks less like a page.
Either is a real authoring surface compared with `presentation.rs`
strings.[^askama][^maud][^settings-rs]

Rocci is the better *markup* language in this repository: components, `@if`,
composition, the same hands that write `RocdownTheme`. The cost is a second
compiler and a view-record contract. Every widget becomes “Rust record +
`.rocci` + apply.” That is the authoring model that already produced
`Settings.rocci` as a stub while `settings.rs` is the live page, and
`ReviewQueue.rocci` as a compiled unused module.[^settings-rocci][^review-rocci][^okf-theme]

Rate of change decides which loop wins:

- Restyle the shell without touching OKF rules → Rocci (or CSS) is cheaper.
- Add a root field, a diagnostic code, a queue column → Rust templates stay
  on one side of the apply boundary; Rocci pays the sync tax every time.

This viewer’s recent work is the second kind (roots, settings, review
tables), not the first.[^settings-ux][^settings-rs]

Authoring UX of the *language product* (dogfooding `.rocci`) is real, and it
is a different user than the person shipping a settings form. Unbound, do
not optimize the knowledge app’s authoring loop for dogfood if that loop is
where drift starts.

### Perceived UX

Readers do not perceive Askama versus Rocci. They perceive whether the app
opened, whether settings saved, and whether the shell matches last time.

| Audience | What they feel | What actually drives it |
| --- | --- | --- |
| Desktop / agent `view` | Instant open, settings work, no Roc install | Rust host; applicator optional only if shipped |
| Same user, second launch | “Why does the sidebar look different?” | Hybrid two-shell, not the markup dialect |
| rocci.dev `/knowledge/` | “Is this the same site?” | CSS and layout, which either stack can emit |
| You as implementer | Fast edit vs split-brain vs `format!` pain | Authoring loop above |
| You as language author | “The viewer is written in Rocci” | Dogfood; invisible in the HTML |

Rocci can still win *brand* UX: the review site looks like other Rocci
chrome, and you practiced the theme path. That win exists only if every
user gets that shell. A prettier `OkfTheme` that machines without `roc` never
see is worse perceived UX than a plainer Askama page that is always the
page.[^okf-readme][^compile-status]

First-open hitch from `roc build` is user-visible. A Rust template change is
just another `cargo` increment. That gap closes if a prebuilt applicator
ships; it does not close by writing better `.rocci`.[^compile-research][^compile-follow-ons]

### Unbound verdict

If catalog-shell does not apply to this binary, **the better application
design is one Rust HTML layer** (Askama-class files, not a second invented
DSL, and not more `format!`). Settings, review, and shell become the same
kind of artifact as the host that owns them. End-user UX improves because
there is one shell and no Roc gate, not because Jinja-in-Rust is prettier
than `@component`.[^askama][^settings-rs][^okf-readme]

Finished Rocci apply remains the better design for **Rocdown and the public
docs site**, and the better *language-dogfood* design. It is the better
*viewer* design only after an applicator is a release artifact and the Rust
twin is deleted — the same operational condition as in the bound
verdict.[^generator][^compile-follow-ons]

The bound recommendation (prefer B when an applicator will ship, else
honest A, never hybrid, never a new language “because we already have
Rocci”) is a workspace-consistency rule. The unbound recommendation for
**today’s static viewer** is A with real templates. Those two disagree only
when you will both ship an applicator *and* keep using Rocci for chrome you
could have authored in Rust. That last case is dogfood, not app design.

If review operations and in-UI agent tasks are in the product, that unbound
verdict is only about the *document* half. The operations half is a different
application. See the next section.

## When the UI becomes interactive

This is not designed. The existing application plan already names the
destination in outline: a browser review queue with revision-bound approve /
request-changes / comment, and query that returns authorized evidence before
optional generated answers. Agents today are supposed to use indexes, search,
and the CLI, not a write-capable UI. “Operations on the queue” and “query and
tasks from the OKF UI” reopen that last sentence.[^okf-app][^tools-research]

Do not treat that future as “the static theme needs more `@component`s,” and
do not treat it as “therefore Askama for everything.” Split the product
again.

| Surface | Kind of interactivity | Stack that fits |
| --- | --- | --- |
| Concept / collection article | Still a document | Rust `article_html` either way |
| Finite queue mutations (verify, dismiss, transition) | One-shot command, morph a region | Rust POST already, or Rocci `@post` |
| Agent query / task / progress from the UI | Job on the server, streamed result | Live hypermedia; Datastar in this repo |

### Finite mutations are not a new app

A review-queue button that writes a verification event and returns a new
`#okf-queue` fragment is the same shape as today’s settings POST, with a
stable id. Server-owned state already says: validate, write, re-read, render
the coherent region, morph.[^server-state][^settings-rs]

Either language can do that. Choosing Askama here is fine. Choosing Rocci
`@post` is fine. Choosing a client store or a JSON-updated table is the
mistake.

### Agent query and tasks *are* a new app

A panel that starts a search, a retrieval job, or an agent task, shows
progress, allows cancel, and patches in evidence is not a static apply
step and not a full-page form. Users will compare it to an agent workbench,
not to a documentation theme. Full reload after “run” is the bad perceived
UX.[^okf-app][^tools-research][^server-state]

In this repository the designed transport for that is Datastar (one-shot
fragment or live SSE), with durable job state on the server. Rust can speak
that protocol. Rocci is the authoring stack already built for it. Askama
plus a hand-rolled SSE layer is a second interactive-app framework next to
Counter and friends.

The machine API should stay the existing CLI / inspect / search contract.
The UI is another client of those operations, not a second write path.
Agents outside the window keep using `rocci-okf`; the window invokes the
same commands and renders the same structured results.[^tools-research][^okf-readme]

### Authoring once operations exist

The person adding “approve this revision” or “run this query” is no longer
only syncing a badge to a classifier. They are declaring a command, a
response region, and maybe a live stream. That is Rocci handler authoring,
or it is reinventing handlers in Rust.

Askama-for-the-whole-chrome then becomes the expensive choice: you invest
in a Rust template language *and* you still need an interaction runtime.
Finishing static-only `OkfTheme` apply is also the wrong investment: apply
is a batch renderer and does not host `@post` or live GET.

The durable split:

1. Keep `okf` and Markdown `article_html` in Rust. Knowledge records stay
   inert.[^static-okf]
2. Put new operations on an interactive app surface (Rocci+Datastar unless
   you explicitly decide to host Datastar from Rust).
3. Do not pick today’s static-shell winner as if it were the operations
   winner.

Settings-UX currently keeps live SSE out of bound for the registry page.
Queue decisions and agent jobs would be a new plan; they would have to
allow one-shot fragments first and live streams only where a job actually
runs.[^settings-ux][^server-state]

### Perceived UX once operations exist

| Audience | What they feel | Failure mode |
| --- | --- | --- |
| Reviewer | Click approve, the row and the concept meta update | Full reload, or a queue that lies about Git state |
| Someone starting an agent task | Progress, cancel, evidence in place | Spinner then a new document, or a JS client model |
| Agent in Cursor / CLI | Same inspect/search/check as today | UI-only mutations the CLI cannot see |
| You as author | One handler story for apps | Askama app + Rocci examples as two crafts |

### Revised unbound pick

- **Documents:** still Rust HTML. Do not move prose into Rocci.
- **Today’s static chrome:** Askama is still the better *host-page* cleanup
  if you will not build operations soon.
- **Queue operations + in-UI agent tasks:** treat that as a Rocci
  application hosted by `rocci-okf`, not as more template files. That is
  when “truly rust + rocci” stops meaning “compile a theme once” and starts
  meaning “Rust owns the engine and jobs; Rocci owns the interactive
  shell.”[^okf-app][^server-state]
- **Do not** implement the workbench in Askama unless you are prepared to
  own Datastar (or equivalent) in Rust for years beside the Rocci stack.

The undesigned part is which operations ship first. Until that plan exists,
avoid sinking cost into a whole-app Askama rewrite *or* a finished
static-only Rocci apply. Both optimize the document viewer. The next
product is the operations surface.

Short-term crate work is [okmate](/plans/okf/okmate.md). In-place
[rust+datastar](/plans/okf/okf-viewer-rust-datastar.md) and
[host surfaces](/plans/okf/okf-viewer-host-surfaces.md) are not started.[^okmate][^rust-datastar][^host-surfaces]

## Datastar without Rocci

Datastar is a **browser transport**: `datastar.js`, `data-on-*` / `@post`,
`Datastar-Request: true`, and responses that are HTML morphs, SSE
`datastar-patch-elements` / `patch-signals`, or 204. Official SDKs exist
outside Roc. This repo already has the protocol in Rust
(`rocci-datastar`: SSE builders, signal extractors, asset staging). Rocci
`@method:role` is one *authoring* frontend for that protocol, not the
protocol.[^datastar-crate][^datastar-eco]

Dropping Rocci from the viewer therefore does **not** drop Datastar. A
pure-Rust `rocci-okf` can:

1. Ship `datastar.js` next to `goto.js` (same asset pipeline the CLI
   already uses).
2. Emit `data-on-click="@post('/__rocci_okf/…')"` from Rust HTML
   (`format!`, later Askama).
3. On POST, branch on `Datastar-Request`: return a fragment with a stable
   `id` (`text/html` morph, or one `patch-elements` event). Ordinary form
   POST / `curl` can keep today’s full article or 204.[^datastar-eco][^settings-rs]
4. For agent jobs, hold durable state in Rust and stream SSE the same way
   generated `@live` does, using `rocci_datastar::sse` instead of generated
   Roc dispatch.[^datastar-crate][^server-state]

What you lose is Rocci authoring: no `@component`, no `@post:fragment`
lowering, no Roc `Html` type, no shared handler matrix with Counter. What
you keep is the **user-visible** interaction model (morph by id, optional
live stream). Perceived UX of a queue row update does not depend on whether
the fragment was rendered by Roc or by Rust.[^server-state]

The earlier “second framework” cost is **server authoring**, not a second
client. You would own two ways to write Datastar handlers (Rocci apps vs
`rocci-okf` extra_http). That is real, and it is smaller than inventing a
JSON SPA. It is the honest price of “pure Rust viewer + Datastar.”

Settings today already returns HTML from extra_http. The jump to Datastar
is: add the script, put a stable `id` on `#okf-settings`, return that node
instead of a full document, and keep a non-Datastar fallback. That can
happen without Askama and without Rocci.[^settings-rs][^ui-goto]

Short-term host-surfaces still keeps Datastar off settings. This section is
the later option if the operations surface is Rust-hosted.[^host-surfaces]

## Mounting a non-Rocci viewer on rocci.dev

The site lane already treats the viewer as a **foreign static app**, not a
Rocdown `[[mount]]` and not an iframe. `rocci-okf build` writes HTML +
`pages.json`; package copies that tree to `/knowledge/`; Caddy
`try_files` serves it; `SiteShell` only gets a header lane to
`/knowledge/`. Chrome join is a thin site-lane strip *inside* the OKF
document, not wrapping every page in `SiteShell`.[^site-lane][^static-okf]

None of that requires `OkfTheme` or Roc. A Rust-only `html_page_for` tree
with prefixed `/knowledge/…` hrefs and `/knowledge/__rocci_okf/app.css`
is the same artifact class. The public visitor sees documents, Cmd-K over
`/knowledge/pages.json`, and the review queue as HTML. They do not see
which compiler produced the shell.[^site-lane][^ui-goto]

What a Rocci-less tree does **not** change:

| Need | How |
| --- | --- |
| Same origin, no catalog merge | Prefix `/knowledge/`; do not reuse site `/pages.json` |
| Inert records | Still `knowledge/**/*.md`; still not a Rocdown mount |
| Preview-only scripts | Still strip `reload.js` / session POST on the packaged tree |
| Publication gate | Still generated HTML of the committed bundle only |

What it does change: you inject the site lane strip from Rust (or a small
HTML partial), not from `OkfTheme`. That is easier than teaching Rocdown
to own OKF pages, and it is the same work the lane plan already assigned
to the OKF document.[^site-lane]

### Live operations vs the static lane

`file_server` cannot run settings POST, review approve, or an agent job.
Those need `extra_http` (local `rocci-okf view`) or a hosted OKF origin.

| Public surface | Fits a copied tree | Needs a live host |
| --- | --- | --- |
| Read dashboard, concepts, review queue | Yes | No |
| Cmd-K, catalog.json, llms.txt | Yes | No |
| `/settings/` mutations | No (omit or link to local app) | Yes |
| Queue decisions / agent tasks | No | Yes |

So a Rust+Datastar workbench and a rocci.dev `/knowledge/` lane are
**different deploys**. The lane stays a snapshot. Interactive OKF stays
loopback, or later a subdomain / reverse-proxied `rocci-okf` — which the
lane plan already left as a follow-on if a prefix is not enough, and which
the local-first publication decision still treats as a separate
review.[^site-lane]

Do not iframe the viewer inside a Rocdown page to “embed” it. Deep links,
history, and Cmd-K break. Do not mount `knowledge/` as Rocdown to “make it
Rocci.” Copy the built tree, whatever engine wrote the HTML.[^site-lane]

[^catalog-shell]: Approved ownership: Rust catalog and article HTML; visible chrome in Rocci; do not grow a second docs-template language in Rust.
[^static-okf]: Canonical records are inert Markdown; knowledge builds must not execute Roc or Rocci content.
[^generator]: Rocdown already implements the catalog/shell split and the trusted HTML splice into Roc.
[^compile-research]: Option A skips unused Roc; option C externalizes pages and writes Rocci HTML; hybrid write-fallback was the measured served site before apply writes landed.
[^compile-follow-ons]: Skip-Roc as the long-term default is a product inversion; shipping prebuilt wasm apply is an open question; do not embed the Roc compiler.
[^compile-status]: Phases 1–3 and 6 in tree; default host auto does not force Roc; missing roc uses the Rust shell.
[^generation-research]: Move chrome and widgets into Rocci, not Markdown bodies; `rocci-okf` must not import Rocdown templates; compiler embed is unavailable.
[^settings-ux]: Mutations stay one-shot POSTs; `Settings.rocci` is not the live renderer.
[^site-lane]: Public knowledge lane packages the existing static review tree; not a Rocdown mount of `knowledge/`.
[^presentation]: `build_review_site_with_session` branches on roc availability; write fills missing apply paths with `html_page_for`; nav, review, and home HTML are Rust.
[^settings-rs]: `/__rocci_okf/settings` POST returns a Rust article string; forms are `method="post"`.
[^okf-build]: `render_page` splices `nav_html` and `article_html` into `OkfTheme.knowledgeShell`.
[^okf-theme]: Shell owns outline, ConceptMeta, and article slot; it does not render review or settings widgets.
[^settings-rocci]: Stub add-directory/add-git forms only; live UI is richer and lives in Rust.
[^review-rocci]: Review widgets exist as Rocci and as `render_review_page`; apply uses the Rust article.
[^runtime-rs]: Theme, ConceptMeta, ReviewQueue, and Settings sources are embedded and compiled together.
[^okf-readme]: Cached Rocci renderer when roc is on PATH; Rust knowledge shell otherwise; macOS `Rocci Knowledge.app` is a bundled preview host.
[^ui-goto]: Shared chrome script swaps `#okf-main` / `#okf-toc`; not Datastar.
[^agents]: Do not interpret Rocci templates in Rust merely to avoid compiling a theme; catalog tests should not require Roc.
[^askama]: HTML-file templates with typed Rust context structs; the usual “proper Rust templating” meaning.
[^maud]: HTML written as Rust macros; stronger rust-analyzer, weaker page-shaped files.
[^server-state]: Action writes through the server, re-reads, renders a stable-id region; browser does not own durable domain state.
[^okf-app]: Planned review decisions bound to a revision; query returns evidence before optional generated answers; CLI remains the agent contract.
[^tools-research]: Ecosystem tools treat machine operations as a public CLI/schema contract; the viewer is a projection, not a second source of truth.
[^okmate]: Extractable `okmate/` crate; Askama + Axum + official Datastar; only workspace dep is `okf`.
[^rust-datastar]: Superseded in-place `rocci-okf` Askama plan; not the implementation vehicle.
[^host-surfaces]: Superseded; settings extra_http and unused Rocci unplug are Phase 1 of the rust+datastar plan.
[^datastar-crate]: Rust crate already builds patch-elements/signals SSE, reads `?datastar=`, and stages `datastar.js`.
[^datastar-eco]: Official SDKs keep the protocol in handler bodies; Rocci `@method:role` is compile-time classification of the same wire types.
