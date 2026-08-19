---
type: Implementation Plan
title: Custom Rocdown block schemas and renderers
description: "Phased delivery of a schema/renderer split for :kind blocks: generic child policy, structured parent children, theme block-pack overrides, optional debug painter, and site config. No document spelling change."
tags: [domain/rocdown, domain/rocci, concern/rendering, concern/architecture, concern/theming, concern/authoring]
status: draft
generated: { by: process:cursor, at: 2026-08-19T19:10:00Z }
stale_after: 2026-11-19
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../research/rocdown-block-renderers.md
    title: Custom Rocdown block schemas and renderers research
    author: process:cursor
    last_modified: 2026-08-19
  - id: block-research
    resource: ../research/generalized-rocdown-block-model.md
    title: Generalized Rocdown block model research
    author: process:cursor
    last_modified: 2026-08-19
  - id: block-plan
    resource: generalized-rocdown-block-model.md
    title: Generalized Rocdown block model implementation plan
    author: process:cursor
    last_modified: 2026-08-19
  - id: registry
    resource: ../../crates/rocci-rocdown/src/registry.rs
    title: Closed v1 article-block kind schema
    author: process:git
    last_modified: 2026-08-19
  - id: docs-rs
    resource: ../../crates/rocci-rocdown/src/docs.rs
    title: Special-case article-block validators
    author: process:git
    last_modified: 2026-08-19
  - id: imports-rs
    resource: ../../crates/rocci-rocdown/src/imports.rs
    title: "@use component-to-kind mapping"
    author: process:git
    last_modified: 2026-08-19
  - id: config-rs
    resource: ../../crates/rocci-rocdown/src/config.rs
    title: Rocdown site configuration schema
    author: process:git
    last_modified: 2026-08-19
  - id: plan-rs
    resource: ../../crates/rocci-rocdown/src/plan.rs
    title: PlannedNode emission and theme compile
    author: process:git
    last_modified: 2026-08-19
  - id: docs-rocci
    resource: ../../crates/rocci-rocdown/templates/DocsComponents.rocci
    title: Builtin article-block painters
    author: process:git
    last_modified: 2026-08-19
  - id: build-runtime
    resource: ../../crates/rocci-rocdown/runtime/RocdownBuild.roc
    title: Hand-written widget dispatcher
    author: process:git
    last_modified: 2026-08-19
  - id: template-readme
    resource: ../../crates/rocci-template/README.md
    title: Rocci component calling convention
    author: process:git
    last_modified: 2026-08-19
  - id: site-ref
    resource: ../../docs/reference/rocdown-site.rocdown
    title: Public Rocdown site configuration
    author: process:git
    last_modified: 2026-08-19
  - id: docs-guide
    resource: ../../docs/guides/docs-components.rocdown
    title: Public documentation-component guide
    author: process:git
    last_modified: 2026-08-19
  - id: rocdown-readme
    resource: ../../crates/rocci-rocdown/README.md
    title: Implemented Rocdown language reference
    author: process:git
    last_modified: 2026-08-19
  - id: catalog-shell
    resource: ../decisions/rust-catalog-rocci-shell.md
    title: Use a Rust catalog and a Rocci documentation shell
    author: process:okf-migration
    last_modified: 2026-08-18
  - id: pure-render
    resource: ../decisions/pure-render-components.md
    title: Keep Rocci render components pure
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: generation-research
    resource: ../research/rocci-components-in-generation.md
    title: Rocci components inside the content generation pipeline
    author: process:cursor
    last_modified: 2026-08-18
  - id: language-dev
    resource: ../../.agents/skills/rocci-language-dev/SKILL.md
    title: Rocci and Rocdown language-development skill
    author: process:git
    last_modified: 2026-08-18
  - id: rocci-author
    resource: ../../.agents/skills/rocci-author/SKILL.md
    title: Rocci and Rocdown authoring skill
    author: process:git
    last_modified: 2026-08-18
---

# Custom Rocdown block schemas and renderers

## Purpose and authority

This is the implementation plan for the [block schema and renderer
research](/research/rocdown-block-renderers.md). It is exploratory until a
human reviewer accepts a scope. It does not describe shipped behavior.
Architecture records and crate READMEs remain the current
contract.[^research][^rocdown-readme]

The [generalized block-model plan](generalized-rocdown-block-model.md)
owned source spelling and the closed builtin registry. That spelling is
in the tree. This plan does not reopen `:name[params]`. It owns how a
`BlockCall` is validated as an interface and painted by a site-selected
Rocci renderer.[^block-plan][^block-research]

Do not start a phase until the user asks. Use `rocci-language-dev` only if
a phase adds `@block` grammar. Use `rocci-author` for theme pack modules
and public docs examples.[^language-dev][^rocci-author]

## Goal

Ship a schema/renderer split so that:

- `:kind` stays a function interface (`named params` + children → `Html`)
- `:tabs` can require `:tab` (and custom kinds can declare the same)
- a site can replace `:note`'s HTML tree without editing documents
- builtin kinds keep a default painter; missing custom painters debug in
  preview and fail `rocdown build` unless opted in

```text
:note[title: "Watch"] {{ body }}     # document, unchanged
theme/Blocks.rocci  @component Note  # site renderer overlay
```

## Constraints that do not move

| Keep | Meaning |
| --- | --- |
| Markdown-first islands | No mid-paragraph `:note`; `@block` / `@use` stay modules or document root |
| Pure `@component` | Renderers are functions to `Html`[^pure-render] |
| Rust catalog / Rocci shell | Validation without Roc; one theme compile per build; no interpreting `.rocci` in Rust[^catalog-shell] |
| OKF Markdown-only | No `:note` in `knowledge/**/*.md` |
| Page chrome vs widgets | `SiteShell` / layouts stay chrome; block packs stay article widgets |

## Non-goals (all phases)

- Changing `:name[params]` / line-scope / `{{ }}` XOR `:kind.begin` ... `:kind.end`
- Inline decorations, named slots, MDX-in-prose
- Per-page `@component` on static `rocdown build`
- Generating the scanner from ungram
- Roc traits / `implements Block`
- Requiring CSS class stability (`rd-docs-*`) on overrides
- `@block` grammar in Phase 1–4 (follow-on Phase 8)
- Reviving `@docs` or any family-name alias for article blocks. That
  spelling was a short-lived experiment; it is removed. Article widgets
  are `:kind` only.

## V1 contract

These answers freeze the research open questions for delivery.[^research]

### Interface versus renderer

A kind has one schema row (params, child policy, defaults) and one bound
renderer name. Builtin rows stay in `registry.rs`. Site packs may override
the renderer name and, for new kinds only, add schema rows inferred from
the `@component` header (`parse_component_params`: names, `??` defaults,
extra body param). Named Roc type aliases are optional documentation; they
are not the v1 schema source and do not require Roc introspection.[^registry][^research]

### Child modes

- **Fragment:** extra arg is `Html` (`note`, `details`, `tab` body, …).
- **Typed list:** extra arg is `List` of `{ child params…, content: Html }`
  for `tabs`, `steps`, `card-grid`. Each child body still paints through
  its own renderer first.[^template-readme][^docs-guide]

### Child policy

`KindSpec` gains `accepts`, `accepts_markdown`, `requires` (rename of
`required_child_kinds` if needed). Exclusive `accepts: ["tab"]` with
`accepts_markdown: false` makes stray paragraphs in `:tabs` an error.
`steps` keeps a listed predicate: `:step+` XOR ordered Markdown list.
`figure` keeps exactly-one-image.[^docs-rs]

### Addressability

v1 scans `theme/Blocks.rocci` if present, else `theme/blocks/*.rocci`.
Component names matching `KindSpec.component` override. New PascalCase
exports become custom kinds (kebab). Helpers must not live in that pack
in v1. `@use` stays interactive-only and still auto-exports every
`@component` until Phase 8.[^imports-rs]

### Defaults and debug

Builtin kinds always resolve to `DocsComponents` if the pack omits them.
Unknown `:kind` remains an error. Known custom kind with no renderer:
debug painter in `rocdown run`; `rocdown build` / `check` error unless
`[blocks] debug = true`.

### Site config

```toml
[blocks]
pack = "theme/Blocks.rocci"   # optional; directory convention if omitted
debug = false

[blocks.override]
note = "Note"
```

Unknown `[blocks]` fields are errors (`deny_unknown_fields`). Omitting
the table preserves today's DocsComponents-only paint.[^config-rs][^site-ref][^docs-rocci]

### Dispatcher

Theme compile emits the widget `match` from the **merged** registry so
adding `:callout` does not require editing `RocdownBuild.roc` by hand.
The runtime file may remain a thin wrapper that includes generated
arms.[^build-runtime]

## Layer map

| Concern | Owner |
| --- | --- |
| Child policy data | `registry.rs` |
| Catalog diagnostics | `docs.rs`, `parse.rs` |
| Pack discovery, override merge | `config.rs`, `plan.rs` |
| Generated dispatcher | `plan.rs` or `runtime/` generated include |
| Builtin painters | `templates/DocsComponents.rocci` |
| Site painters | project `theme/Blocks.rocci` |
| Debug painter | `templates/BlockDebug.rocci` (name flexible) |
| Standalone preview | `lower.rs` may call the same debug shape for unbound custom kinds |
| Public contract | crate README, `docs/reference/rocdown-site.rocdown`, `docs/guides/docs-components.rocdown` |
| `@block` grammar | `rocci-template` (Phase 8 only) |

## Delivery phases

Each phase is one mergeable change. Later phases may assume earlier ones
have merged.

### Phase 1 — Generic child policy in the registry

**Bound:** data and catalog diagnostics only. No Rocci or config change.

**Does:**

- Drive `:tabs` / `:card-grid` / aside-forbids-tabs from `accepts` /
  `requires` / `forbids` / `accepts_markdown`.
- Error on Markdown or non-`tab` blocks inside `:tabs`.
- Keep steps XOR-list and figure one-image as named predicates on the
  spec, called from one `validate_children` helper.
- LSP completions inside a parent prefer `accepts` kinds.
- Tests in `colon_syntax.rs` / catalog tests: stray paragraph in `:tabs`
  fails; `:tab` outside `:tabs` still fails.

**Does not:** change Html, `RocdownBuild.roc`, or site config.

**Exit:** `cargo test -p rocci-rocdown`. `child_kinds` is either enforced
or removed as a dead field.

### Phase 2 — Typed child lists for tabs, steps, card-grid

**Bound:** planner + `DocsComponents` + generated apply data. Authors
still write the same `:tabs`.

**Does:**

- Emit child records (tab `id`/`label` + painted body, etc.) instead of
  `child_count` + flattened forest for those three parents.
- Change `Tabs` / `Steps` / `CardGrid` to take the list (or keep a
  temporary adapter that builds chrome from records while still wrapping
  default stacked markup).
- Default visual for tabs may stay stacked sections in this phase; the
  point is the parent *can* see labels. A real `tablist` is optional if
  it needs JS; do not add Datastar to `static` docs in this phase.
- Assert `plan.rs` tests contain tab ids in generated Roc, not only
  `child_count`.

**Does not:** site override, `@block`, debug painter.

**Exit:** `cargo test -p rocci-rocdown` and `rocdown build docs`. Docs
HTML for un-overridden tabs remains acceptable (not necessarily
byte-identical).

### Phase 3 — Theme block pack overlay for builtin renderers

**Bound:** a project theme can replace `Note` (and any other builtin
widget) with a different element tree.

**Does:**

- Discover `theme/Blocks.rocci` or `theme/blocks/*.rocci` next to the
  existing theme compile.
- Merge by `KindSpec.component` name; pack wins; missing kinds stay
  `DocsComponents`.
- Allow pack modules to `import DocsComponents` and wrap
  `DocsComponents.note`.
- Example or test site that overrides `:note` to `<section data-test-
  note>` (or similar) and checks generated HTML.
- Compile the pack once with the theme (renderer cache hash includes
  it).[^generation-research][^plan-rs]

**Does not:** custom new kinds, TOML `[blocks]`, `@use` on static
builds.

**Exit:** `docs/` unchanged without a pack. A fixture site with
`theme/Blocks.rocci` paints different `:note` HTML. `cargo test -p
rocci-rocdown`.

### Phase 4 — Site config `[blocks]`

**Bound:** `rocdown.toml` overlay and `debug` flag plumbing (flag unused
until Phase 5).

**Does:**

- Add `BlocksConfig { pack, debug, override }` with
  `deny_unknown_fields`.
- `pack` overrides directory convention when set.
- `[blocks.override]` remaps kind → component name; unknown kind keys
  error; unknown component names error at theme compile.
- Document in `docs/reference/rocdown-site.rocdown`.

**Does not:** implement the debug painter.

**Exit:** config tests; public site reference updated; `rocdown check
docs` green.

### Phase 5 — Debug painter

**Bound:** known schema, missing renderer.

**Does:**

- Add a first-party debug component (kind, params, nested children).
- `rocdown run`: bind it when the merged table has a schema row and no
  painter.
- `rocdown build` / `check`: error unless `[blocks] debug = true`.
- Unknown kinds stay errors (no debug).
- Standalone `lower.rs` may reuse the same shape for unbound `@use`
  kinds.

**Exit:** tests for both policies. Debug markup is distinctly unfinished
(`data-rocci-block-debug` or equivalent).

### Phase 6 — Custom static kinds from the pack

**Bound:** a theme pack can add `:callout` to static `rocdown build`.

**Does:**

- Infer schema from `@component` headers via `parse_component_params`
  (required vs `??` defaults vs extra body parameter). Do not parse
  named Roc type aliases, invoke Roc, or use glue/introspection.
  Child policy defaults to fragment + any children until Phase 8
  metadata exists. Worked sketches:
  [research examples](../research/rocdown-block-renderers.md#worked-examples).
- Merge into the site registry; other sites without the pack still
  error on `:callout`.
- Generated dispatcher includes the new arm.
- Reject pack names that collide with module reserved words (`page`,
  `use`, …).
- Document that helpers must not sit in the block pack in v1.

**Does not:** `@block`, interactive `@use` on static builds, qualified
names.

**Exit:** fixture site with `:callout` builds; `docs/` still has no
`@use`. Parser tests remain Roc-free; generator tests may use the
fixture.

### Phase 7 — Generated dispatcher as source of truth

**Bound:** stop hand-maintaining one Roc arm per kind in
`RocdownBuild.roc`.

**Does:**

- Emit match arms (or equivalent) from the merged registry at plan
  time.
- Keep a small handwritten `render_forest!` / IO wrapper.
- Drift test: every `paints_as_widget` kind has an arm; pack extras
  too.

**Exit:** adding a pack kind does not edit `runtime/RocdownBuild.roc`.
`cargo test -p rocci-rocdown` plus a docs build.

### Phase 8 — `@block` opt-in (follow-on)

**Bound:** `rocci-template` grammar. Skip if pack convention is enough.

**Does:**

- Recognize `@block` like `@component`, optional `as: "note"` and
  `accepts: [tab]` (exact spelling in a short design note before
  coding).
- Interactive `@use` only imports `@block` exports, not every
  `@component`.
- Extract child policy from the declaration into the site registry.

**Does not:** inline `@block` in `.rocdown` article bodies.

**Exit:** `cargo test -p rocci-template` and `rocci-rocdown`. Public
Rocci / Rocdown references. `rocci-author` idioms.

### Phase 9 — Public docs and heading-sugar override (optional)

**Bound:** documentation and, if cheap, `H2` pack override.

**Does:**

- Update `docs/guides/docs-components.rocdown` for override + custom
  kinds.
- Update `rocci-author` static-vs-interactive note (`@use` vs pack).
- Optional: allow pack `H2` to paint heading sugar; ids stay Rust.

**Exit:** `rocdown build docs`. Architecture/status knowledge follow-up
*after* a phase ships, not in this plan commit.

## Suggested merge order

1 → 2 can be separate PRs (policy then typed lists). 3 before 4. 5 can
land with 3 if the debug component is the missing-pack fallback. 6 after
3–5. 7 after 6 so generated arms include custom kinds. 8 last among
language features. 9 trails 3 or 6.

## Validation

```text
cargo test -p rocci-rocdown
cargo fmt --all -- --check
```

After syntax or public-contract changes:

```text
cargo test -p rocci-template
cargo run -q -p rocci-rocdown-cli -- check docs
cargo run -q -p rocci-rocdown-cli -- build docs
```

After knowledge edits:

```text
cargo run -q -p rocci-okf -- check knowledge --profile rocci --format terminal
```

Do not set `ROCCI_REQUIRE_ROC=1` unless a phase must prove apply. Do not
log a phase complete in `knowledge/log.md` until CI and Knowledge
workflows succeed on that revision.

## Follow-ons (not v1)

- Named slots
- Heading / Markdown-sugar painters as a full Prose layer (`P`, `A`, …)
- Qualified kind names if two packs export `Note`
- Interactive `@use` honored on static builds (likely never; packs are
  the static path)
- Real JS tablist on `static` pages via hashed article script
- Renaming `rd-docs-*` once overrides are common

## Open questions that would still change the plan

1. Pack-only vs `@block` in the first custom-kind phase (this plan
   defers `@block` to Phase 8).
2. Whether default tabs UI must become a `tablist` once typed children
   exist, or stays stacked until a JS follow-on.
3. Whether `[blocks.override]` is needed besides filename convention in
   v1.
4. Whether a later phase should scan `Name : { field : Type }` opaque
   Roc spans for schema. Frozen as no for v1.

[^research]: Exploratory schema/renderer split, child modes, pack overlay, debug policy.
[^block-research]: Syntax and uniform BlockCall; not renderer override.
[^block-plan]: Spelling delivery; `@use` interactive-only.
[^registry]: Current `KindSpec`; `child_kinds` unused as exclusive policy.
[^docs-rs]: Special-case tabs/steps/figure validators.
[^imports-rs]: `@use` auto-exports every `@component`; cannot override builtins.
[^config-rs]: No `[blocks]` table today.
[^plan-rs]: Theme compile; flattened `child_count` emission.
[^docs-rocci]: Builtin painters; opaque `content` on `Tabs`.
[^build-runtime]: Hand-written match and Html concatenation.
[^template-readme]: `|{ props }, content|`; `??` defaults.
[^site-ref]: `build.theme` is chrome only.
[^docs-guide]: Public widget guide.
[^rocdown-readme]: Shipped file shape and `@use` static rejection.
[^catalog-shell]: Rust catalog; Rocci chrome; no Rocci interpreter in Rust.
[^pure-render]: Pure render functions to Html.
[^generation-research]: One renderer compile per theme, not per page.
[^language-dev]: Grammar work only if Phase 8 runs.
[^rocci-author]: Theme pack and docs examples after syntax exists.
