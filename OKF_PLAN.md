# Open Knowledge Format plan for Rocci

**Plan date:** 2026-08-16

**Repository state inspected:** current working tree, including the root reports that are not yet tracked

**Status:** active — Phase 1 implementation complete; exit validation is pending on one pre-existing Rocs test

**Standards baseline:** Open Knowledge Format (OKF) v0.2 and Design Tokens Community Group (DTCG) 2025.10

**Contract approved:** 2026-08-16 by `human:nils`

## Executive recommendation

Bootstrap one strict OKF v0.2 bundle at `knowledge/`, and make its concept bodies a deliberately static Rocdown profile:

- canonical knowledge records are `knowledge/**/*.md`, not `.rocdown`, because OKF v0.2 requires Markdown concept files with YAML frontmatter;
- their bodies use the Markdown subset already parsed and rendered by `rocci-rocdown`, plus footnotes, but no `@roc`, `@render`, `@component`, raw HTML, or other executable regions;
- a small OKF reader validates frontmatter, removes it with correct source-offset mapping, and hands the body to Rocdown's Markdown AST and Rocs catalog/rendering pipeline;
- existing `.rocdown` pages keep `@page { ... }` and their current extension. Do not add two competing metadata syntaxes to ordinary Rocdown documents merely to support OKF;
- the bundle is the source database. Search indexes, graph JSON, HTML, `llms.txt`, and any single-file export are derived artifacts;
- root reports are evidence and migration sources, not automatically canonical truth. Extract small current records and decisions from them while retaining the reports unchanged during migration;
- DTCG JSON is the approved source of truth for design values. Generated CSS preserves the existing `--rd-*` contract and supplies a second mapping for the Rocs shell. DTCG does not replace Rocci layouts, DOM classes, assets, or theme manifests;
- a root `DESIGN.md` will be the short, human-facing design contract. It explains intent and use and links to DTCG token sources rather than duplicating their raw values.

This is a filesystem knowledge bundle, not a new database server. That matches OKF's intentionally minimal model and keeps git history, review, and portability as the storage layer.

## Implementation progress

| Phase | Status | Progress record |
| --- | --- | --- |
| 0 — approve the contract | Complete (2026-08-16) | All decisions in section 13 have accepted answers; diagnostic namespaces and severity policy are fixed in section 11.1 |
| 1 — parser and three-record vertical slice | Implementation complete; exit pending (2026-08-16) | Parser, profile validation, footnotes, reserved files, JSON inspection, knowledge CLI/build, and three seed records are implemented; one pre-existing Rocs session test hangs in `roc build` |
| 2 — inventory and provenance migration | Not started | The source inventory is a plan input only; no report content has been migrated |
| 3 — decisions and human review | Not started | Phase 0 approval does not count as human verification of future migrated records |
| 4 — DTCG and `DESIGN.md` | Not started | DTCG authority is approved, but token sources, generated CSS, and `DESIGN.md` do not exist yet |
| 5 — CI, retrieval, and publication | Not started | Publication remains local/repository-visible pending the Phase 5 review |
| 6 — consolidation | Not started | Root reports remain in place and unchanged |

The Phase 0 baseline was refreshed against repository `HEAD` `c77fb38` on 2026-08-16. The Rocs Phase 3 work previously described as uncommitted is now committed: `BuildPlan`, fingerprinted assets and theme CSS, asset URL rewriting, CSP, `404.html`, and the structured `PageView` are present. The repository has twelve root Markdown files including this plan, twenty published `.rocdown` pages under `docs/`, and nine workspace crates. Four root files remain untracked at approval time: this plan plus `DATASTAR_ROCKET_IN_ROCCI_REPORT.md`, `ROCCI_SYNTAX_WEAK_POINTS_REPORT.md`, and `ROCDOWN_FORMAT_REPORT.md`.

Phase 0 changes only the project contract and progress record. It does not create the `knowledge/` bundle, implement parsing, migrate content, assign verification to future records, generate tokens, or alter existing `.rocdown` behavior.

## 1. Constraints established by the repository and the standards

### 1.1 Current Rocdown and Rocs behavior

The current implementation has these relevant properties:

- `rocci-rocdown` discovers document-root `@` declarations, parses Markdown through Comrak, and extracts a closed `PageMeta` from a Roc record in `@page`.
- The current `PageMeta` fields are `id`, `route`, `aliases`, `draft`, `layout`, `meta`, statically extracted `title` and `description`, `theme`, and `color_scheme`. Unknown `@page` fields are errors.
- Rocdown enables tables, strikethrough, task lists, autolinks, and title-after-pipe wikilinks. It does not currently enable or render Markdown footnotes, which OKF v0.2 uses for per-claim source attribution.
- Rocs discovers only `.rocdown`, derives IDs and routes from source paths, builds a directed link graph, validates navigation and draft links, renders static Markdown AST nodes in Rust, and evaluates `RocsTheme.rocci` once per build.
- The Phase 0 refresh found the Rocs Phase 3 work committed at `c77fb38`: `BuildPlan` owns fingerprinted assets and extracted theme CSS, asset URL rewriting, CSP, a generated 404 page, and additional shell metadata. Treat that commit and its tests as current implementation evidence; refresh it again when the relevant knowledge records are migrated.
- Rocs rejects dynamic Roc/Rocci regions, Datastar imports, and custom layouts in static pages today.
- `rocs check` and `rocs inspect` already provide useful foundations for schema, graph, navigation, page, and artifact inspection.

The OKF integration should reuse those foundations without making `@page` understand arbitrary YAML-shaped metadata or making OKF records executable.

### 1.2 The OKF compatibility boundary

OKF v0.2 defines a bundle as a directory of Markdown concept files with YAML frontmatter. Every non-reserved `.md` file must have a non-empty `type`; `index.md` and `log.md` have defined special forms. It recommends `title`, `description`, `resource`, and `tags`, and defines optional `sources`, `generated`, `verified`, `status`, and `stale_after`. It also says consumers must preserve unknown fields and tolerate unknown types and broken links. See the [official OKF v0.2 specification](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md).

Therefore, a directory of `.rocdown` files with `@page` records is not a conformant OKF bundle. The recommended boundary is:

| Concern | Canonical form | Consumer |
| --- | --- | --- |
| Internal knowledge | `knowledge/**/*.md` with OKF YAML | OKF reader, Rocs, agents |
| Product documentation | `docs/**/*.rocdown` with `@page` | Rocdown and Rocs |
| Rich/dynamic content | `.rocdown` declarations | Rocdown compiler/runtime |
| Search, graph, HTML, agent indexes | generated | Rocs/knowledge build |

Call the first row the **OKF/Rocdown static profile**: it is strict OKF on disk and uses Rocdown's static Markdown semantics for rendering. This avoids a forked “OKF-like” format.

### 1.3 Rocdown versus pure Markdown for the OKF database

Here, **pure Markdown** means OKF's canonical `.md` file with YAML frontmatter and a standard Markdown body. **Rocdown** means a `.rocdown` source that may use `@page` and executable Roc/Rocci declarations. The recommended static profile stores the former while reusing the latter's Markdown parser, AST, renderer, catalog, and theme infrastructure.

#### Option A — canonical Rocdown records

Pros:

- Dogfoods Rocci's own content format and keeps knowledge authoring close to the existing `docs/` workflow.
- Reuses current page metadata, route derivation, link graph, diagnostics, source maps, themes, and Rocs builds with less initial adapter code.
- Can embed typed Roc values, Rocci components, layouts, scoped CSS, and eventually islands when a knowledge page genuinely benefits from richer presentation.
- Gives project-specific syntax and editor tooling one obvious home; contributors do not switch between YAML metadata and Roc record metadata.
- Could generate polished public documentation directly from the same authored source.

Cons:

- `.rocdown` plus `@page` is not an OKF v0.2 concept representation. A second `.md` export would be required before the database could truthfully claim OKF conformance.
- The current closed `@page` extractor rejects unknown fields, whereas OKF consumers must tolerate and preserve extension metadata. Modeling `sources`, `generated`, `verified`, and future OKF fields in Roc records would tightly couple the language to a fast-moving external format.
- Executable declarations increase the trust and security surface. A consumer can no longer treat every record as inert Markdown, and builds may depend on Roc, Rocci compilation, layouts, assets, or runtime capabilities.
- Generic OKF, Markdown, static-site, note-taking, and agent tools cannot consume the canonical source without a Rocdown adapter.
- An export creates two representations. Lossless round trips, stable IDs, source spans, citations, and unknown fields become additional contracts, and generated Markdown can be mistaken for the editable source.
- Rocdown's page identity and web-route semantics are not the same as OKF concept IDs and bundle-relative paths. Reusing them implicitly risks links that work on the site but not in the bundle.
- Rich components can hide important knowledge in rendered output or computation instead of plain text, weakening grep, diff, retrieval, and graceful degradation.
- Rocdown does not currently support the keyed Markdown footnotes used by OKF v0.2 for per-claim attribution.

Canonical Rocdown is reasonable only if Rocci values dogfooding and rich publication above direct OKF interoperability, and is willing to call the authored database **OKF-inspired** while treating a generated `.md` tree as the conformant exchange artifact.

#### Option B — canonical pure Markdown records

Pros:

- Directly matches OKF's `.md` plus YAML contract; the checked-in database itself is the portable bundle rather than an export.
- Remains readable with a text editor, `rg`, git, generic Markdown tools, and agents that know nothing about Rocci.
- Keeps knowledge inert by default. Validation and rendering need not execute Roc, Rocci components, client code, or arbitrary plugins.
- Preserves unknown OKF fields naturally when the YAML representation is parsed generically rather than projected immediately into the closed Rocdown `PageMeta`.
- Makes source/citation review, patch generation, and external interchange straightforward because the reviewed and distributed representation is the same file.
- Separates concept identity from web routing: a renderer can map one canonical OKF path to different publication routes without changing the record.
- Reduces lock-in if Rocdown syntax, Rocs architecture, or the repository's preferred documentation generator changes.

Cons:

- Requires a new YAML/OKF reader, profile validator, reserved-file handling, footnote support, and careful frontmatter-to-body source-offset mapping.
- Introduces YAML as a second metadata syntax beside Rocdown's `@page` Roc record, so contributors must understand which collection uses which form.
- Cannot directly use `@render`, Roc values, components, per-page scoped CSS, custom layouts, or future islands while remaining portable pure Markdown.
- Rich knowledge presentations must be supplied by the renderer from metadata and Markdown structure, not authored as embedded components.
- Existing Rocs discovery, page model, link resolver, and diagnostics need an explicit collection adapter instead of being reused unchanged.
- Public docs and internal knowledge can drift if material is copied between `.rocdown` and `.md` instead of linked or selectively dual-published.
- YAML permits more shapes than the current strongly bounded `@page` syntax, increasing the importance of profile validation and unknown-field preservation tests.

#### Option C — strict Markdown source with a static Rocdown rendering profile

This plan recommends the hybrid boundary, not a hybrid file syntax:

- source remains pure, conformant OKF Markdown;
- Rocdown supplies only body parsing, semantic Markdown AST, HTML classes, source locations, and theme hooks;
- Rocs supplies catalog, graph, validation, search, and publication;
- all executable Rocdown declarations are forbidden in the knowledge collection;
- richer views are renderer-owned projections of typed records, never required to recover the underlying knowledge.

This captures most of pure Markdown's portability and safety while reusing Rocci's compiler infrastructure and visibly dogfooding its static content model. Its cost is a dedicated adapter and a deliberate two-collection rule: `.md` + YAML for OKF knowledge, `.rocdown` + `@page` for rich product documentation.

Recommendation: choose Option C for the canonical database. Permit a future `.rocdown` **view** to read or present OKF records, but do not make that view the record of authority. Revisit canonical Rocdown only if strict OKF interchange proves unimportant or the OKF specification explicitly standardizes alternative source syntaxes.

### 1.4 Authority order

When sources disagree, records should resolve claims in this order unless an accepted decision explicitly changes the intended design:

1. executable behavior in current code and tests;
2. current crate READMEs, CLI help, config, and published `docs/` pages;
3. accepted decision records and the active implementation plan;
4. dated research/design reports;
5. examples, which demonstrate behavior but can contain intentional stress cases or shortcuts;
6. generated `dist/` output and historical git material.

A record must distinguish **what ships**, **what has been decided**, and **what is only proposed**. Combining those states is the largest current knowledge-quality risk.

## 2. Knowledge inventory

### 2.1 Root Markdown

There are eleven root Markdown files before this plan: the operational `README.md`, `ROADMAP.md`, and nine substantial reports/plans. The reports contain roughly 7,500 lines and many external citations. They are valuable evidence, but most mix background, alternatives, recommendations, implementation status, and open decisions in one document.

| Source | Knowledge present | Current authority | Proposed treatment |
| --- | --- | --- | --- |
| `README.md` | Product definition, crate map, commands, runtime and packaging behavior | Current operational overview | Source for `architecture/system-overview.md` and `reference/workflows.md`; do not duplicate command detail unnecessarily |
| `ROADMAP.md` | Architectural direction, completed work, focus, limitations | Current status summary | Source for `status/implementation.md` and `status/known-limitations.md` |
| `ROC_TEMPLATE.md` | Template grammar rationale, alternatives, semantics, styling, tooling, open decisions | Dated proposal; parts are implemented and parts are not | Split stable language facts from unresolved proposals; source `architecture/template-language.md` and several decision records |
| `ROC_DATASTAR_COMPONENT_FILETYPE_REPORT.md` | Runtime boundary, server ownership, compilation architecture, security, delivery phases | Dated architecture/feasibility report | Distill `architecture/rendering-and-runtime.md`; retain research and rejected options as evidence |
| `ROCCI_SYNTAX_WEAK_POINTS_REPORT.md` | Audit findings, priorities, acceptance tests | Current working-tree audit dated 2026-08-14 | Create one audit summary plus issue/risk records only for accepted follow-up work |
| `SNAKE_DATASTAR_ARCHITECTURE_REPORT.md` | Input-boundary case study and client-island recommendation | Case-specific design report | Create `case-studies/snake-input.md`; link general conclusions to the client-boundary decision |
| `DATASTAR_ROCKET_IN_ROCCI_REPORT.md` | Browser-island architecture, lifecycle requirements, licensing concerns, staged plan | Illustrative future design | Create `research/client-islands.md`; promote a decision only after approval |
| `ROCDOWN_FORMAT_REPORT.md` | Original Rocdown design, grammar, security, migration, acceptance tests | Explicitly subordinate to current crate README for shipped behavior | Distill current format architecture separately from unshipped proposals |
| `ROCDOWN_DOCUMENTATION_GENERATOR_REPORT.md` | Documentation landscape, content design, proposed compiler, validation, delivery | Research and product design | Keep as a cited research source; distill only adopted Rocs architecture and content policy |
| `ROCDOWN_DOCUMENTATION_GENERATOR_IMPLEMENTATION_PLAN.md` | Active Rocs plan and phase status | Active implementation plan | Source for `plans/rocs.md`; keep phase state synchronized until the OKF record replaces it |
| `ROCDOWN_THEMING_REPORT.md` | Theme ecosystem research, package model, token and adapter recommendations | Proposed architecture; current code is smaller | Split `research/theme-ecosystems.md`, `architecture/theming.md`, and approval-bound decisions |

Migration should not mechanically copy each report into one record. A report can source multiple atomic records, and one canonical record can cite several reports plus current code.

### 2.2 Published Rocdown documentation

The `docs/` tree currently has twenty `.rocdown` pages organized as:

- 3 getting-started pages;
- 4 guides;
- 3 concepts;
- 5 references;
- examples, troubleshooting, project status, contributing, and the site index.

These pages are explicit product knowledge with curated navigation in `docs/rocs.toml`. They should remain public documentation sources. The OKF bundle should link to them or describe the resource they represent, not clone every page. A later dual-publish pipeline may render selected OKF records into the public site, but that is a publication decision rather than a bootstrap requirement.

### 2.3 Executable and implicit knowledge

| Source family | Examples | Knowledge encoded | Ingestion policy |
| --- | --- | --- | --- |
| Compiler AST and lowering | `rocci-template`, `rocci-rocdown` | Accepted grammar, metadata, emitted artifacts, source maps | Cite exact files and tests; summarize only stable contracts |
| Rocs catalog/build | `crates/rocs/src/{catalog,site,build}.rs` | Identity, routes, graph, nav, draft, diagnostics, artifact rules | Primary evidence for current Rocs records |
| Theme code | `rocci-theme` CSS/resolver and `RocsTheme.rocci` | Two current theming/token surfaces and DOM class contracts | Primary evidence for design-system records |
| Runtime crates | `rocci-core`, `rocci-wry`, `rocci-cli` | Backend, session, host, security, serving, packaging contracts | Record public boundaries, not every public symbol |
| Tests and fixtures | crate tests, `test/AllSyntax.*` | Executable examples and edge-case expectations | Treat as verification evidence; link acceptance tests from records |
| Examples | counter, styling, Rocdown, errors, Snake, Datastar, Rocs | End-to-end workflows and architectural stress cases | Create example/case-study records only when they teach a durable lesson |
| Configuration | `Cargo.toml`, `rocci.toml`, `docs/rocs.toml`, editor manifests | Workspace composition, defaults, navigation, integrations | Extract stable configuration facts; validate drift mechanically |
| Git history | recent Rocdown, theming, Rocs, linking, and docs commits | Decision chronology and provenance | Use for investigation and `last_modified`; do not make every commit a concept |
| Generated output | `dist/` | Build result | Never canonical; regenerate and compare where useful |

The knowledge base should describe meaningful contracts and decisions, not become a source-code index. A source-code symbol graph can be generated later and linked as a separate resource.

## 3. Target bundle and build architecture

### 3.1 Proposed source tree

```text
knowledge/
  index.md
  log.md
  architecture/
    index.md
    system-overview.md
    rendering-and-runtime.md
    template-language.md
    rocdown-format.md
    rocs-documentation-compiler.md
    theming.md
  decisions/
    index.md
    pure-render-components.md
    server-owned-state.md
    markdown-first-explicit-islands.md
    rust-catalog-rocci-shell.md
    client-behavior-islands.md
  status/
    index.md
    implementation.md
    known-limitations.md
  plans/
    index.md
    rocs.md
    knowledge-system.md
  research/
    index.md
    documentation-generators.md
    theme-ecosystems.md
    client-islands.md
  audits/
    index.md
    rocci-syntax.md
  case-studies/
    index.md
    snake-input.md
  reference/
    index.md
    crate-map.md
    workflows.md
  design/
    index.md
    design-system.md
    design-tokens.md
```

Keep the first migration nearer 15–20 concepts than 100. Add a record when it has a clear question, owner, lifecycle, and retrieval value.

The root index begins with only the version declaration permitted by OKF:

```markdown
---
okf_version: "0.2"
---

# Rocci knowledge

* [Architecture](architecture/) - Current system contracts and boundaries.
* [Decisions](decisions/) - Accepted and proposed choices with consequences.
```

### 3.2 Source and derived artifacts

Recommended derived tree:

```text
dist/knowledge/
  site/                 # rendered human view
  catalog.json          # normalized metadata and graph
  search.json           # retrieval index
  llms.txt              # concise agent discovery surface
  bundle.zip            # optional verbatim OKF bundle
  validation.json       # machine-readable diagnostics
```

No generated artifact should be edited or cited as the origin of a claim when its source record or repository source is available.

### 3.3 Implementation boundary

Recommended package ownership:

- add an `okf` module to `rocs` for the initial spike, because Rocs already owns catalog validation, graph resolution, inspection, and static output;
- keep YAML parsing and OKF normalization behind a library API with no theme dependency so it can become `rocci-okf` later if another consumer appears;
- expose or add a body-only Markdown parse API in `rocci-rocdown`; do not fake frontmatter removal without preserving byte offsets;
- add a configured knowledge collection to `rocs.toml` instead of making all `.md` files under `docs/` discoverable;
- map OKF `title`, `description`, `status`, and concept ID to the existing catalog model, then retain the full normalized metadata beside `ResolvedPage` rather than discarding it;
- keep OKF bundle paths distinct from published routes. A bundle link such as `/architecture/system-overview.md` must resolve from the bundle root even if the rendered site route is `/knowledge/architecture/system-overview/`.

Suggested CLI surface, subject to approval:

```text
rocs knowledge check knowledge
rocs knowledge inspect concept architecture/system-overview knowledge
rocs knowledge inspect graph knowledge
rocs knowledge build knowledge --output dist/knowledge
```

Extending `rocs check` with a collection selector is also reasonable. Do not overload the existing docs check until diagnostic codes and output schemas can distinguish Rocs page errors from OKF/profile errors.

## 4. Canonical record contract

### 4.1 Example

```markdown
---
type: Architecture
title: Rocdown format boundary
description: Rocdown keeps prose in Markdown and places executable regions behind explicit document-root declarations.
tags: [domain/rocdown, concern/syntax, concern/security]
status: draft
generated: { by: process:okf-migration, at: 2026-08-16T00:00:00Z }
verified:
  - { by: human:nils, at: 2026-08-16T00:00:00Z }
stale_after: 2027-02-16
authority: descriptive
owners: [human:nils]
sources:
  - id: parser
    resource: ../../crates/rocci-rocdown/src/parse.rs
    title: Current Rocdown parser
    author: process:git
    last_modified: 2026-08-16
  - id: format-report
    resource: ../../ROCDOWN_FORMAT_REPORT.md
    title: Original Rocdown format report
    last_modified: 2026-08-15
---

# Rocdown format boundary

## Current contract

Rocdown recognizes executable declarations only at the document root.[^parser]

## Rationale

The boundary keeps ordinary prose predictable and executable regions explicit.[^format-report]

## Not yet implemented

...

[^parser]: Current parser implementation.
[^format-report]: Original format investigation.
```

The sample actor IDs are placeholders until the actor/owner convention is approved.

### 4.2 Metadata policy

| Field | Policy in the Rocci profile |
| --- | --- |
| `type` | Required by OKF; must use the controlled type list below during bootstrap |
| `title` | Required by the Rocci profile; one clear noun phrase |
| `description` | Required; one sentence usable in indexes and retrieval results |
| `resource` | Use only when the concept describes one underlying asset; omit for abstract architecture or decisions |
| `tags` | Required; 1–6 controlled facet tags, with at least one `domain/*` tag |
| `status` | Required even though OKF defaults absence to `stable`; explicit state is safer during migration |
| `generated` | Required; records who or what made the current meaningful content revision |
| `verified` | Optional and evidence-based; never synthesize a human verifier |
| `stale_after` | Required for status, plan, audit, and time-sensitive research records; optional for durable decisions |
| `sources` | Required for any factual synthesis or migrated report; source entries used in claims need stable `id` values |
| `authority` | Rocci extension: `normative`, `descriptive`, `exploratory`, or `historical` |
| `owners` | Rocci extension: list of actors responsible for review, not authorship |

Unknown fields must survive parse/serialize round trips, as OKF requires. Keep custom fields few and documented in `knowledge/reference/record-profile.md` when implementation begins.

### 4.3 Concept types

OKF intentionally has no central type registry. Use this small repository vocabulary:

| Type | Purpose | Expected body |
| --- | --- | --- |
| `Architecture` | Current boundaries and component relationships | Contract, rationale, boundaries, evidence, open gaps |
| `Decision` | One accepted or proposed choice | Context, options, decision, consequences, supersession |
| `Specification` | Precise language or interface contract | Scope, normative rules, examples, diagnostics, compatibility |
| `Status` | Current implementation or support state | Snapshot date, shipped, missing, risks, next review |
| `Implementation Plan` | Ordered work toward an approved outcome | Goal, prerequisites, phases, validation, exit criteria |
| `Research Report` | Evidence and synthesis without normative force | Question, method/scope, findings, recommendation, limits |
| `Audit` | Findings against an implementation or policy | Scope, findings, severity, evidence, disposition |
| `Case Study` | A concrete example with generalizable lessons | Context, observed behavior, analysis, outcome, lessons |
| `Reference` | Lookup-oriented stable facts | Definitions or tables, examples, source pointers |
| `Design Standard` | Human-facing visual and interaction contract | Principles, token model, accessibility, component/content rules |

Do not create separate types for crate names or technologies. Those are tags.

### 4.4 Tag taxonomy

Directories provide the primary browsing hierarchy; tags provide orthogonal facets.

| Facet | Initial values |
| --- | --- |
| Domain | `domain/rocci`, `domain/rocdown`, `domain/rocs`, `domain/runtime`, `domain/desktop`, `domain/design-system` |
| Integration | `integration/roc`, `integration/datastar`, `integration/wry`, `integration/dtcg`, `integration/okf` |
| Concern | `concern/syntax`, `concern/rendering`, `concern/theming`, `concern/tooling`, `concern/security`, `concern/accessibility`, `concern/performance`, `concern/validation`, `concern/packaging` |
| Audience, only when useful | `audience/contributor`, `audience/maintainer`, `audience/agent` |

Rules:

- lowercase ASCII, slash-separated, singular where natural;
- do not repeat `type`, `status`, or directory names as maturity tags;
- new facet prefixes require review; new values within an existing facet may be added with a record;
- validation warns on unused one-off tags and errors on unknown prefixes.

### 4.5 Atomicity and body conventions

A concept should answer one durable question. Split a record when parts have different owners, lifecycle, verification, or authority. In particular, do not put three independent design decisions into one architecture overview.

Use ordinary Markdown links for concept relationships. Do not use Rocdown wikilink syntax in canonical OKF records even though the current parser accepts it; standard links maximize OKF portability. Tables and other supported GFM-style constructs are presentational conveniences and must not carry meaning unavailable in plain text.

Use these conventional headings where applicable:

- `# Current contract` for shipped facts;
- `# Decision` for the chosen direction;
- `# Rationale` and `# Consequences` for decisions;
- `# Evidence` for code, tests, or observed results;
- `# Not yet implemented` for approved but absent behavior;
- `# Open questions` only for real unresolved choices;
- `# Validation` for mechanical checks or acceptance tests;
- `# Computation` only for an actual OKF `Attested Computation`.

Rocci has no initial need for `Attested Computation`. Do not use it for build commands or tests: the OKF type has a specific executor/receipt/attester contract intended to attest computed values.

## 5. Source and citation policy

### 5.1 Source classes

1. **Repository evidence:** resolve relative paths from the concept file, such as `../../crates/rocs/src/catalog.rs` from a record directly under `knowledge/architecture/`. A directory or test suite may be a scope descriptor when one file is not sufficient.
2. **Internal reports/docs:** cite the original file as a source. If a report cites an external claim, either re-check that external primary source or phrase the new concept as reporting what the report concluded.
3. **External technical facts:** prefer official specifications, official documentation, standards, repositories, or research papers. Use stable versioned URLs when possible.
4. **Generated artifacts:** cite only when the artifact itself is the subject. Otherwise cite its generator and input.

### 5.2 Per-claim attribution

- Every externally derived or non-obvious claim gets a footnote whose label exactly matches `sources[].id`.
- One source can support several claims; use a stable semantic ID, not a list position.
- A source entry may be present without a footnote when it supports the concept as a whole, but the validator should warn about unused IDs so accidental leftovers are visible.
- Quotations should be rare and short. Prefer a precise paraphrase with a link.
- Do not add a legacy `# Citations` list; OKF v0.2 supersedes it with `sources` plus keyed footnotes.
- Record the source's own `last_modified` when known. Do not confuse it with `generated.at`, which dates the concept revision.
- `usage_count` and `usage_window` are not useful for the bootstrap corpus and should be omitted until a real, consistently measured usage source exists.

### 5.3 Repository drift

For local sources, validation should compare the latest git modification of each referenced path with the concept's latest verification. A newer source is a warning, not an automatic proof that the concept is wrong. The warning should identify the changed source and record owner.

Untracked files have no reliable git provenance. The three currently untracked reports must either be committed before migration or be cited with their explicit investigation dates and marked as unverified inputs.

## 6. Lifecycle, trust, and ownership

### 6.1 State model

Use only OKF's lifecycle values:

```text
draft -> stable -> deprecated
  |         |
  +---------+----> draft (substantive revision requiring new review)
```

- `draft`: incomplete, unresolved, or not reviewed for the stated authority.
- `stable`: ready for consumption at its declared authority level. “Stable exploratory research” still does not become normative; `authority` carries that distinction.
- `deprecated`: retained for incoming links and history. The body must link to its replacement or explain why there is none.
- “superseded,” “accepted,” “rejected,” and “implemented” are not extra lifecycle values. Express them in decision/status bodies and links.

### 6.2 Trust rules

- Imported content starts `draft` with `generated.by: process:okf-migration` and no human `verified` entry.
- Mechanical validation may add a `process:*` verification event only when a defined check actually compared the record with its sources.
- A human review adds `human:<id>` and produces OKF's human-reviewed trust tier.
- A human review older than `generated.at` is retained as history, but the Rocci validator warns that the current revision has not been human-confirmed.
- Changing claims, decisions, examples, or conclusions updates `generated.at`. Formatting-only changes need not do so.
- Never store a subjective trust score. OKF deliberately derives trust from provenance and verification signals.

### 6.3 Freshness defaults

These are initial policy defaults, not OKF requirements:

| Type | Suggested freshness |
| --- | --- |
| `Status` | 30 days |
| `Implementation Plan` | 45 days while active |
| `Audit` | 90 days or after watched code changes |
| `Research Report` about external products/specs | 90 days |
| `Architecture` / `Specification` | 180 days or after source changes |
| `Decision` | no date when durable; re-open on an explicit trigger |
| `Reference` | 180 days or after source changes |

`stale_after` is an absolute date and should be advanced only after review. Stale does not mean deprecated; it means consumers should seek confirmation.

### 6.4 Ownership

Start with one explicit maintainer actor and add team identities only when they correspond to real review responsibility. Ownership is for routing review, not establishing truth. The exact actor IDs need approval before records are generated.

## 7. Canonical seed records

The first useful bundle should contain these records, in this order:

| Priority | Record | Main sources | Purpose |
| --- | --- | --- | --- |
| 1 | `architecture/system-overview.md` | README, ROADMAP, crate manifests | Retrieval entry point and boundaries |
| 1 | `status/implementation.md` | ROADMAP, docs status, code/tests | Date-stamped shipped state |
| 1 | `status/known-limitations.md` | ROADMAP, docs, Rocs diagnostics | Prevent agents from presenting planned features as shipped |
| 1 | `architecture/rocdown-format.md` | crate README/code/tests, format report | Current format vs future design |
| 1 | `architecture/rocs-documentation-compiler.md` | Rocs code, active plan, docs | Catalog/build/theme boundary |
| 1 | `architecture/theming.md` | `rocci-theme`, Rocs theme, theming report | Explain the two current theme surfaces and target convergence |
| 1 | `decisions/pure-render-components.md` | template/runtime reports, compiler | Preserve render component semantics |
| 1 | `decisions/server-owned-state.md` | runtime report, examples | Preserve server/browser ownership boundary |
| 1 | `decisions/markdown-first-explicit-islands.md` | format report, parser | Preserve the Rocdown language boundary |
| 1 | `decisions/rust-catalog-rocci-shell.md` | Rocs plan/code | Preserve static compiler architecture |
| 2 | `architecture/template-language.md` | compiler, tests, ROC_TEMPLATE | Consolidate shipped syntax and remaining gaps |
| 2 | `audits/rocci-syntax.md` | syntax audit plus current tests | Track semantic hazards without inflating the language |
| 2 | `plans/rocs.md` | active implementation plan | Migrate phase state and acceptance criteria |
| 2 | `research/documentation-generators.md` | generator report | Retain researched rationale without normative status |
| 2 | `research/theme-ecosystems.md` | theming report | Retain external adapter research |
| 2 | `research/client-islands.md` | Rocket report, Snake report | Consolidate browser-island evidence |
| 2 | `case-studies/snake-input.md` | Snake source/report | Preserve a concrete boundary case |
| 3 | `design/design-system.md` | `DESIGN.md`, token files, theme code | Index the human contract |
| 3 | `design/design-tokens.md` | DTCG files and generator | Explain token layers and outputs |

Indexes should expose these progressively. Do not add per-function or per-config-key records in the bootstrap.

## 8. DTCG integration

### 8.1 Standards findings

The DTCG 2025.10 Format Module is a stable Final Community Group Report, although it is not a W3C Recommendation. It standardizes JSON token files, `$value`, `$type`, `$description`, `$deprecated`, groups, aliases, and preservable vendor extensions. Token and group names cannot start with `$` or contain `.`, `{`, or `}`. Types must be explicit or inherited and must not be guessed from values. See the [DTCG Format Module 2025.10](https://www.w3.org/community/reports/design-tokens/CG-FINAL-format-20251028/).

The stable Resolver Module models ordered sets and modifiers for contexts such as light and dark. It can express theme resolution, but order and overlapping modifiers need deliberate design. See the [DTCG Resolver Module 2025.10](https://www.w3.org/community/reports/design-tokens/CG-FINAL-resolver-20251028/). Colors should use the structured representation from the [DTCG Color Module 2025.10](https://www.w3.org/community/reports/design-tokens/CG-FINAL-color-20251028/), rather than treating a CSS hex string as the portable source value.

### 8.2 Role in Rocci

DTCG should own portable **design values and their semantic aliases**. It should not own:

- `.rd-*` Markdown DOM classes;
- Rocs site-shell structure, navigation, breadcrumbs, or outline;
- CSS selectors, cascade layers, responsive rules, print behavior, or `@scope` strategy;
- theme package assets, fonts, layouts, scripts, compatibility metadata, or lockfiles;
- the distinction between article and presentation renderers.

Those remain Rocdown/Rocs theme contracts. A DTCG translator turns resolved design values into the CSS variables those contracts consume.

### 8.3 Current model and target mapping

Today there are two separate surfaces:

1. `rocci-theme` defines `paper` and `rocci` CSS files scoped to `.rd-document`, with `--rd-*` variables and `light-dark()` values. `chrome.css` maps those variables onto semantic Markdown classes.
2. `RocsTheme.rocci` owns a large site shell and a separate source-level palette (`--canvas`, `--surface`, `--ink`, `--accent`, and others), plus layout and component rules. In-progress working-tree code extracts that `@css` to a fingerprinted stylesheet, but it does not yet make the palette a shared token source.

The target should be one token source with two generated adapters:

| DTCG layer | Examples | Output use |
| --- | --- | --- |
| Foundation | color ramps, font families, spacing, radii, widths, durations | Never consumed directly by components unless explicitly documented |
| Semantic | canvas, surface, text primary/muted, accent, border, code background/text | Shared meaning across Rocdown and Rocs |
| Content | heading levels, paragraph, link, blockquote, code, table | Existing `--rd-*` compatibility variables |
| Shell | header, sidebar, outline, navigation, journey, mobile menu | Generated variables consumed by `RocsTheme.rocci` |

Semantic and component tokens should alias foundation tokens. Every token gets `$description`; `$type` should be explicit at group level where unambiguous and explicit on exceptions. Use `$deprecated` when replacing a public token. Reserve a reverse-domain `$extensions` key only for optional generation hints; essential meaning must remain in standard token fields.

### 8.4 Proposed token layout

```text
design/
  tokens/
    foundation.tokens.json
    semantic.tokens.json
    content.tokens.json
    shell.tokens.json
    themes/
      paper-light.tokens.json
      paper-dark.tokens.json
      rocci-light.tokens.json
      rocci-dark.tokens.json
    paper.resolver.json
    rocci.resolver.json
  generated/
    paper.css
    rocci.css
    rocs-shell.css
    tokens.manifest.json
```

Use one resolver per named theme initially, with a `scheme` modifier for `light` and `dark`. That is easier to validate than one multi-brand resolver whose brand and scheme modifiers both overwrite the same semantic paths. Reconsider a combined resolver only when theme packages need cross-brand composition.

### 8.5 Generation and compatibility

Recommended pipeline:

1. Parse and validate all token files against DTCG 2025.10 rules.
2. Resolve aliases and both scheme contexts; reject missing values, type mismatch, cycles, invalid names, and unresolved references.
3. Translate DTCG values to CSS with explicit color-space conversion/fallback policy.
4. Generate the existing `--rd-*` names so third-party/local themes and `chrome.css` do not break.
5. Generate named shell variables for `RocsTheme.rocci`; replace its literal palette incrementally, not in one unreviewed rewrite.
6. Preserve `data-rd-color-scheme="light|dark"` and `auto` behavior. The generator may continue emitting `light-dark()` or emit selectors/media queries, but output must be equivalent under tests.
7. Commit generated CSS and a manifest, and make CI regenerate into a temporary directory and fail on drift. This keeps Rust `include_str!` builds offline and deterministic while the DTCG JSON remains the declared source of truth.

The checked-in-generated-output choice requires approval. An alternative is build-time generation, but that adds a token tool to every Rust build and makes third-party theme diagnostics harder to isolate.

### 8.6 DTCG validation matrix

- format validation for names, groups, `$value`, `$type`, structured colors, dimensions, and composites;
- alias and `$extends` resolution with cycle and type checks;
- resolver validation for every declared context and deterministic resolution order;
- a complete token matrix for `paper × {light,dark}` and `rocci × {light,dark}`;
- generated CSS snapshot tests;
- compatibility tests proving every variable currently read by `chrome.css` is emitted;
- Rocs shell tests proving no literal color/spacing value remains where a governed token is required;
- contrast tests for normal text, links, code, focus states, and selected navigation;
- browser screenshots at narrow/wide viewports, forced light/dark, system auto, print, forced colors, and reduced motion;
- a round-trip test preserving unknown `$extensions` data.

## 9. Recommended role of `DESIGN.md`

Create `DESIGN.md` at the repository root during the DTCG phase, not during the initial OKF parser spike. It should be a concise normative design manual for contributors and agents, while `design/tokens/*.json` is the machine-readable value source and theme code is the implementation.

Recommended contents:

1. design goals and character: what Rocci should feel like, and what it should avoid;
2. user and content priorities: documentation, app UI, desktop shell, and presentation boundaries;
3. accessibility baseline: contrast, focus, keyboard use, motion, forced colors, zoom, and print;
4. token architecture: foundation → semantic → content/shell, alias rules, naming, scheme resolution, and deprecation;
5. supported themes and schemes, including the meaning of `paper`, `rocci`, `none`, and `auto`;
6. Rocdown content DOM contract: `.rd-*` classes and which parts are stable public hooks;
7. Rocs shell contract: header, navigation, article, outline, responsive transitions, and which details belong to layout rather than tokens;
8. typography, spacing, color, code, table, and media usage guidance;
9. component states and interaction guidance, including hover, focus, active, disabled, loading, error, and empty states;
10. source-of-truth and generation workflow, including how to propose, review, test, deprecate, and release token changes;
11. links to the DTCG sources, generated token reference, visual tests, and relevant accepted decisions.

`DESIGN.md` should not contain:

- a hand-maintained dump of every token value;
- raw CSS that can drift from generated output;
- speculative theme-adapter research;
- general product architecture already owned by knowledge records;
- an implementation roadmap.

Make `DESIGN.md` the `resource`/primary source of `knowledge/design/design-system.md`; keep that OKF record as a short discovery and lifecycle wrapper rather than a duplicate manual.

## 10. Migration phases

### Phase 0 — approve the contract (complete 2026-08-16)

Deliverables:

- decide the compatibility boundary, bundle path, publication scope, actors, taxonomy, and custom metadata;
- decide whether DTCG JSON and checked-in generated CSS will be authoritative;
- define diagnostic namespaces (`OKFxxxx`, `RDxxxx`, `DTCGxxxx`) and severity policy;
- record the accepted choices before implementation.

Exit: all decisions in section 13 have explicit answers.

Result: complete. The recommended defaults were accepted without exceptions, `human:nils` was approved as the initial maintainer actor, the repository baseline was refreshed, and sections 11.1 and 13 are now the normative diagnostic and decision records for subsequent phases. Verification on the approval baseline passed all 283 workspace tests and `rocs check docs` on 2026-08-16.

### Phase 1 — parser and three-record vertical slice

Deliverables:

- parse strict YAML frontmatter while preserving unknown fields and source spans;
- enable Markdown footnotes in the body-only Rocdown path and render them accessibly;
- recognize reserved `index.md` and `log.md` forms;
- normalize one concept model and expose JSON inspection;
- build three representative records: an architecture record, a decision, and a status record;
- render them through a minimal Rocs knowledge collection without changing existing `.rocdown` behavior.

Validation: OKF conformance, round trip, source-offset diagnostics, keyed citations, bundle-root links, and unchanged existing Rocdown tests.

Result on 2026-08-16: implementation complete on `codex/okf-phase1`.

- `rocci-rocdown` exposes a Markdown-only body parser with original-source byte spans, standard links, and opt-in footnotes; ordinary `.rocdown` parsing keeps footnotes disabled and retains its existing wikilink behavior.
- `rocs::okf` discovers strict `.md` bundles, parses YAML to a normalized metadata map without discarding extension fields, separates base OKF from the Rocci profile, recognizes `index.md` and `log.md`, rejects executable declarations/raw HTML, validates keyed sources, and resolves bundle-root concept links.
- `rocs knowledge check|inspect|build` provides terminal/JSON diagnostics, normalized concept/catalog/graph inspection, and deterministic minimal HTML plus `catalog.json` and `validation.json` output.
- `knowledge/` contains the root index and representative Architecture, Decision, and Status records. All three are `draft`, generated by `process:okf-migration`, owned by `human:nils`, and unverified as required by the Phase 0 trust decision.
- The real bundle passes `rocs knowledge check knowledge` with zero diagnostics; concept and graph inspection and two identical builds succeed. Focused OKF/body-parser tests pass, `cargo check --workspace` passes, `cargo test --workspace --exclude rocs` passes 246 tests, 53 of 54 Rocs library tests pass, `rocs check docs` passes, and existing Rocdown tests remain green.

Exit validation is pending because the pre-existing `build::tests::session_reuses_apply_binary_when_roc_sources_are_unchanged` test from the Rocs dev-server snapshot does not finish its `roc build main.roc --output ...` subprocess. It hangs both in the parallel workspace run and when run alone; the adjacent normal build and repeat-build tests pass. Phase 1 did not modify `crates/rocs/src/build.rs`, `dev.rs`, `plan.rs`, or `runtime.rs`. Do not mark this phase fully complete until that baseline test finishes or its separate defect is resolved.

### Phase 2 — inventory and provenance migration

Deliverables:

- create the proposed directory indexes;
- migrate priority-1 records from current code/docs/reports;
- mark all machine-extracted records `draft` and unverified;
- populate source paths, source IDs, dates, owners, authority, and freshness;
- generate a migration matrix mapping every root report section to a record, retained source, or explicit “not migrated” disposition.

Do not move, delete, or rewrite root reports in this phase.

### Phase 3 — decisions and human review

Deliverables:

- extract atomic decisions from reports;
- resolve contradictions between shipped behavior and proposals;
- review priority-1 concepts, add human verification, and promote eligible records to `stable`;
- establish `log.md` updates for material record additions, deprecations, and replacements;
- add source-drift checks from git.

Exit: an agent can answer “what ships, what is decided, and what is proposed?” without reading a whole root report.

### Phase 4 — DTCG and `DESIGN.md`

Deliverables:

- inventory current Rocdown and Rocs variables;
- create foundation, semantic, content, and shell token sources;
- add per-theme light/dark resolvers;
- generate compatibility CSS and a manifest;
- replace duplicated literal theme values incrementally;
- write and review `DESIGN.md`;
- add design-system knowledge records and visual/accessibility validation.

Exit: current themes render equivalently, token sources are authoritative, and documented design intent is reviewable separately from raw values.

### Phase 5 — CI, retrieval, and publication

Deliverables:

- run schema/profile, graph, provenance, freshness, source-drift, DTCG, and deterministic-build checks in CI;
- emit `catalog.json`, `search.json`, `llms.txt`, and validation JSON;
- add type, tag, status, authority, trust-tier, and stale filters to inspection/search;
- decide whether the rendered knowledge site is public, private, or local-only;
- publish a verbatim bundle archive only if all referenced sources and licenses permit it.

### Phase 6 — consolidation

Deliverables, only after stable records exist:

- mark superseded records `deprecated` and link replacements;
- decide whether each root report remains, moves to an archive, or is deleted in a separate reviewed change;
- remove duplicate status/decision prose from public docs only when a generated or linked canonical source is better;
- measure retrieval quality with a fixed question set before adding embeddings or a database service.

## 11. Validation specification

### 11.1 Conformance and profile

Diagnostic codes are stable public identifiers with a namespace plus four decimal digits. Once released, a code keeps its meaning; retired codes are not reused.

| Namespace/range | Ownership |
| --- | --- |
| `OKF1000`–`OKF1999` | Base OKF syntax, reserved files, and standard field shapes |
| `OKF2000`–`OKF2999` | Rocci profile fields, controlled types, tags, and body contracts |
| `OKF3000`–`OKF3999` | Bundle paths, links, headings, reachability, and graph conflicts |
| `OKF4000`–`OKF4999` | Sources, keyed attribution, trust, freshness, and repository drift |
| `OKF5000`–`OKF5999` | Knowledge normalization, deterministic artifacts, and publication safety |
| `RDxxxx` | Existing Rocdown/Rocs diagnostics; preserve current allocations (`RD1xxx` parse, `RD20xx` identity/routes, `RD21xx` links/assets, `RD22xx` navigation/drafts, `RD23xx` unsupported static-page features) |
| `DTCG1000`–`DTCG1999` | Token file syntax, names, groups, and standard field shapes |
| `DTCG2000`–`DTCG2999` | Types, aliases, inheritance, extensions, and cycles |
| `DTCG3000`–`DTCG3999` | Resolver definitions, modifiers, contexts, and resolution order |
| `DTCG4000`–`DTCG4999` | CSS translation, compatibility variables, manifests, and generated-output drift |
| `DTCG5000`–`DTCG5999` | Contrast, theme matrices, visual behavior, and accessibility checks |

Severity is intrinsic to the diagnostic code, not chosen independently by each output format. An error means the requested check/build cannot produce a valid result and must exit nonzero without committing output. A warning identifies consumable but incomplete, stale, unverified, non-portable, or review-worthy knowledge and does not fail the default command. CI may promote selected warning codes, or all warnings in a strict job, without changing their stored severity. Terminal and JSON output must report the same code and severity. Base OKF conformance and the Rocci profile are separate validation modes; profile failures must not make an external base-conformant bundle appear nonconformant. Network links are not followed during default validation.

Errors:

- invalid UTF-8, YAML, or frontmatter delimiters;
- missing/empty `type` in a non-reserved `.md` file;
- malformed reserved `index.md` or `log.md`;
- invalid standard field shapes or dates;
- missing Rocci-profile `title`, `description`, `status`, `generated`, or domain tag;
- unknown lifecycle value or tag prefix;
- duplicate concept ID;
- source footnote referring to no `sources[].id`;
- path escaping an allowed repository boundary during build.

Warnings:

- unknown type or extension field, while still preserving it;
- broken concept link, as OKF requires consumers to tolerate it;
- unused source ID;
- stale record;
- latest human verification older than `generated.at`;
- local source modified after verification;
- stable record with unresolved “TODO,” “TBD,” or open decision language;
- deprecated record without replacement/explanation;
- record not reachable from an index.

Strict Rocci-profile checks must be selectable separately from base OKF conformance so an externally supplied conformant bundle is not rejected for lacking Rocci's optional conventions.

### 11.2 Graph and retrieval

- resolve absolute bundle links from `knowledge/`, not repository or site root;
- classify links as concepts, headings, repository resources, assets, or external URLs;
- check headings and duplicate anchors;
- report orphaned concepts but allow deliberately unlisted drafts;
- detect conflicting canonical records for the same subject/resource;
- include metadata and heading chunks in search while keeping the concept as the lifecycle unit;
- test retrieval with questions covering architecture, current status, known gaps, language behavior, theming, and decisions;
- require answers to surface `draft`, stale, exploratory, or unverified state.

### 11.3 Content contracts by type

- decisions require context, decision, consequences, and current disposition;
- status records require a snapshot date and current evidence;
- implementation plans require prerequisites, phases, validation, and exit criteria;
- research and audits require scope/method, findings, limits, and sources;
- architecture/specification records must separate current contract from future work;
- design standards must link machine-readable tokens and validation.

### 11.4 Regression and determinism

- existing `cargo test --workspace` remains green;
- existing `rocs check docs` and docs output remain unchanged unless an approved integration intentionally changes them;
- two knowledge builds from the same tree produce byte-identical normalized metadata, indexes, and generated CSS;
- YAML key order and body formatting are not rewritten by a read-only check;
- validation never follows network links by default. An explicit scheduled link check may do so with caching and rate limits.

## 12. Risks and non-goals

| Risk | Mitigation |
| --- | --- |
| Calling a `.rocdown`/`@page` directory OKF when it is not | Keep strict `.md` + YAML at the canonical boundary |
| Adding YAML to all Rocdown files creates two page metadata systems | Isolate OKF parsing to configured knowledge collections |
| Reports become “canonical” without resolving proposal vs implementation | Extract atomic records and apply authority order |
| Frontmatter becomes an ungoverned schema | Keep only two initial extensions and validate the profile separately |
| Footnote offsets or rendering regress diagnostics | Add a body parse API and span/golden tests before migration |
| Absolute OKF links collide with web routes | Model bundle IDs and published routes separately |
| DTCG is mistaken for a complete theme package | Document the token/layout boundary in code and `DESIGN.md` |
| Token migration breaks local themes | Preserve `--rd-*`, publish deprecations, and use compatibility tests |
| Freshness dates become busywork | Apply dates only to volatile types and add source-triggered review |
| Knowledge duplicates public docs and drifts | Link to docs/resources; dual-publish only selected records |
| A larger retrieval stack is added before the corpus is sound | Validate a small git-native bundle and benchmark questions first |

Non-goals for bootstrap:

- a graph database, vector database, hosted knowledge service, or MCP server;
- automatic conversion of every code symbol, issue, commit, or documentation page into a concept;
- executing knowledge records or introducing `Attested Computation` without a real computed-value use case;
- replacing root reports before migrated records are reviewed;
- making arbitrary Markdown files Rocs pages;
- redesigning the visual system during token extraction;
- supporting external presentation themes as part of the OKF work.

## 13. Approved decision register

All decisions were approved on 2026-08-16 as part of Phase 0. “Approved” establishes the implementation contract; it does not mark migrated knowledge as verified or stable.

| # | Decision | Accepted answer | Status |
| --- | --- | --- | --- |
| 1 | Canonical syntax and execution boundary | Option C in section 1.3: canonical `knowledge/**/*.md` with YAML, rendered through a static Rocdown profile with declarations forbidden | Approved |
| 2 | Bundle location | `knowledge/` as one self-contained bundle | Approved |
| 3 | Knowledge scope | Internal architecture, decisions, research, audits, status, plans, and design; public docs remain sources | Approved |
| 4 | Initial custom metadata | Only `authority` and `owners` | Approved |
| 5 | Type and tag vocabularies | Adopt sections 4.3 and 4.4 for bootstrap | Approved |
| 6 | Actor IDs and owner | Use `human:nils` as the initial maintainer/owner and `process:okf-migration` for initial extraction | Approved |
| 7 | Initial trust | All migrated records start `draft` and unverified until evidence-based review | Approved |
| 8 | Root report disposition | Retain unchanged through phases 1–5; archive or delete only in a later reviewed change | Approved |
| 9 | Rocs implementation home | Start as an isolated `rocs::okf` module with a separable API | Approved |
| 10 | Publication | Local/repository-visible first; decide public rendering in Phase 5 | Approved |
| 11 | DTCG authority | DTCG 2025.10 JSON is the source of truth; generated CSS is checked in | Approved |
| 12 | Token resolver layout | Use one resolver per theme with light/dark contexts | Approved |
| 13 | CSS compatibility | Preserve all current `--rd-*` names for the first tokenized release | Approved |
| 14 | `DESIGN.md` role | Use a root normative human guide; keep values in DTCG files and make the OKF record a thin lifecycle/discovery wrapper | Approved |
| 15 | Freshness defaults | Adopt section 6.3 as warnings for the first release | Approved |

Any change to an accepted answer is a contract amendment: update this register with the date and rationale before implementing the divergent behavior.

## 14. Completion criteria

The bootstrap is complete when:

- `knowledge/` passes base OKF v0.2 conformance and the documented Rocci profile;
- every seed record has a unique ID, type, description, controlled tags, explicit status, generation provenance, owner, sources, and appropriate freshness;
- priority-1 records are human-reviewed and clearly separate current, decided, and proposed behavior;
- every root report has a recorded migration disposition;
- keyed footnotes render correctly and round-trip source IDs;
- Rocs can inspect, validate, graph, search, and render the bundle without changing existing `.rocdown` semantics;
- source drift and stale knowledge appear in machine-readable and terminal diagnostics;
- DTCG sources reproduce the existing Rocdown themes and Rocs palette through checked compatibility outputs;
- `DESIGN.md` states design intent and governance without duplicating token values;
- generated outputs are deterministic and never treated as canonical;
- repository tests, docs checks, link checks, token checks, and visual/accessibility checks pass at the levels appropriate to their phase.

## Primary standards sources

- Google Cloud Platform, [Open Knowledge Format v0.2 specification](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md)
- W3C Design Tokens Community Group, [Design Tokens Format Module 2025.10](https://www.w3.org/community/reports/design-tokens/CG-FINAL-format-20251028/)
- W3C Design Tokens Community Group, [Design Tokens Resolver Module 2025.10](https://www.w3.org/community/reports/design-tokens/CG-FINAL-resolver-20251028/)
- W3C Design Tokens Community Group, [Design Tokens Color Module 2025.10](https://www.w3.org/community/reports/design-tokens/CG-FINAL-color-20251028/)
