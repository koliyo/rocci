# Knowledge log

## 2026-08-19

- Added draft [generalized Rocdown block model implementation plan](plans/generalized-rocdown-block-model.md): nine bounded phases from a closed Rust registry and per-kind Rocci components through dual-parse `:name[params]`, sugar unification, public `@docs` cutover, typed props, `@use`, and LSP. Exploratory; no phase started.
- Added draft [generalized Rocdown block model](research/generalized-rocdown-block-model.md) research: article blocks distinct from `@`; decided spelling `:name[params]` with `{{ }}` / `:end`; draft AST ungram at `crates/rocci-rocdown/Rocdown.AST.ungram` (nodes, not scanner). Syntax sketches under `knowledge/research/syntax/`.

## 2026-08-18

- Added a draft [preview window naming decision](decisions/preview-window.md): the native Tao/Wry shell is the preview window, overlay navigation is preview chrome, and compiler-derived panels stay on the preview origin as Rocci that consumes host JSON.
- Added draft [desktop host chrome versus Rocci inspector UI](research/desktop-host-chrome-and-inspector-ui.md) research: keep wry overlay navigation in HTML/CSS/JS, keep document/site chrome in Rocci, and author compiler-derived panels (parse timings, diagnostics) as preview-origin Rocci apps that consume host JSON. The record does not reverse the generation-pipeline chrome plan.
- Added draft [repository-hosting research](research/repository-hosting-and-distributed-governance.md), comparing GitHub's current Rocci CI, release, contributor, and policy advantages with Tangled's self-hosted knots, spindles, and cross-server collaboration. The exploratory recommendation keeps GitHub canonical for the public launch, pilots Tangled as a first-class mirror, and makes any later canonical-host change depend on demonstrated reliability and governance evidence.
- Added the draft [agent-model comparison for Rocci component-generation research](audits/agent-model-component-generation-comparison.md), recording the maintainer-supplied attribution of result A to Gemini 3.7 Flash and result B to Grok 4.6, the evidence-based 52/89 assessment, validation caveats, and the recommendation to use B as the stronger starting point while correcting its compiler-artifact and benchmark gaps.
- Implemented Phase 5 of the [First-party Rocci chrome library and generation host plan](plans/rocci-component-generation.md): created the base-Rocci `rocci-roc-host` crate providing two-tier renderer caching (`~/.rocci/cache/roc` and `~/.rocci/cache/renderers`) with SHA-256 integrity, a minimal embedded relocatable WASI WebAssembly platform (`wasm32`), and in-process Wasmtime evaluation (`HostChoice::Wasm`). Integrated wasm host evaluation into `rocci-rocdown` and `rocci-okf`, enabling pure in-memory batch page rendering across both native and WebAssembly targets.
- Added exploratory research and a delivery plan for using Rocci components more in content generation: [Rocci components in the generation pipeline](research/rocci-components-in-generation.md) and [First-party Rocci chrome library and generation host](plans/rocci-component-generation.md). The records keep Markdown and OKF governance in Rust, propose shared outline/nav/breadcrumb components in base Rocci, require both a native subprocess host and a Wasmtime host, persist generated Roc and compiled artifacts as two content-addressed cache tiers, and leave native glue documented as future potential. They remain draft until a reviewer answers the open product questions.
- Implemented phases 1–3 of the [CLI entry points plan](plans/cli-entry-points.md): file-aware `rocci-okf run`, boundary-safe `rocci` / `rocdown` hints that refuse OKF YAML dumps, and compact concept metadata in the OKF review viewer.
- Added the draft [CLI entry points for Rocci, Rocdown, and OKF preview plan](plans/cli-entry-points.md), recommending that `rocci`, `rocdown`, and `rocci-okf` stay separate binaries; rejecting `rocci-okf-cli` and a plugin host; and proposing file-aware `rocci-okf run` plus boundary-safe hints so OKF YAML is not previewed as Rocdown prose.
- Added the draft [rocci.dev site architecture and Rocdown evolution plan](plans/rocci-dev-site.md), proposing one static site and catalog for landing pages, documentation, news, FAQ, and project information; project-local Rocci shells and named layouts above Rust-owned catalog work; typed news collections as a later phase; and no separate `rocci-site` engine unless multiple sites demonstrate a distinct product boundary.
- Closed all remaining findings from the [Rocdown boundary refactor review](audits/rocdown-boundary-refactor-review.md):
  - Removed direct `rocci-theme` dependency from `rocci-rocdown-cli` (consumed via `rocci-rocdown`), supported `ROCDOWN_THEME` / `ROCDOWN_COLOR_SCHEME` with backward-compatible fallbacks, and documented `rocci-theme` role (F-08).
  - Internalized `load_config_named` in `rocci-rocdown`, leaving `CONFIG_FILE` and `load_config` as the sole canonical configuration loader API (F-09).
  - Documented the intentional base CLI UX error suggestion on `.rocdown` / `.md` inputs as an approved usability exception in the refactor decision and plan (F-10).
  - Included `rocci-okf` binary in `.github/workflows/release.yml` release builds and staging archive packages (F-11).
  - Reconciled decision and plan status, updated exit-gate assessment, and formally closed the review audit (F-06).
- Closed high-priority findings from the [Rocdown boundary refactor review](audits/rocdown-boundary-refactor-review.md):
  - Removed unused `rocci-rocdown` and `rocci-core` dependencies from `rocci-okf` and deleted the `TEMPORARY_OKF_ROCDOWN_PRESENTATION` allowlist in `scripts/check-workspace-deps.py` (F-04).
  - Pruned `rocci-ui` to strictly domain-neutral view records (`PageView`, `SiteView`, `LaneView`, `NavItemView`, `BreadcrumbView`, `OutlineView`, `ResourceView`) and string escaping; removed speculative renderers and OKF-specific tones, self-containing all OKF presentation inside `rocci-okf` (F-05).
  - Renamed obsolete concept records `rocs-documentation-compiler.md` -> `rocdown-documentation-compiler.md` and `rocs-okf.md` -> `rocci-okf-app.md`, updated all citations, indexes, `PRIORITY_1_RECORDS` in `rocci-okf`, `system-overview.md` (14 workspace crates), and status records (F-06).
  - Corrected active operational commands across `docs/examples/index.rocdown`, `AGENTS.md`, and agent skills to distinguish `rocci-cli` and `rocci-rocdown-cli`; rebased future playground plan and added archival banners to historical plans (F-07).
  - Removed redundant `ROCDOWN_CONFIG_FILE` and `load_rocdown_config` aliases from `rocci-rocdown` (F-09).

## 2026-08-17

- Corrected the Phase 8 completion claim for revision `aa9b032`: documentation, knowledge indexes, and dependency-script updates landed on that commit, but CI (`32070977528`) and Knowledge (`32070977526`) failed until follow-up `b353895`. Green GitHub runs on the fix commit (CI `32072225878`, Knowledge `32072226074`) satisfy the workflow exit gate; treat `aa9b032` as documentation delivery only, not phase closure. A phase must not be logged complete until the required GitHub workflows have succeeded on that revision.
- Restored shipped Rocdown language-server wiring: added Rocdown-class `rocci-rocdown-lsp` as the `rocci-language-server` composition binary, demoted `rocci-lsp` to a library, registered `.rocdown` in VS Code and Zed, and made workspace-member classification fail on stale `CLASSES` entries.
- Added the [Rocdown product-boundary refactor completion review](audits/rocdown-boundary-refactor-review.md), distinguishing the successful package and CLI consolidation from red Phase 8 workflows, missing Rocdown editor wiring, an unnecessary OKF-to-Rocdown edge, over-extracted shared UI, and stale knowledge and operational documentation. Follow-up CI and Knowledge workflow fixes in `b353895` are green on GitHub (CI `32072225878`, Knowledge `32072226074`).
- Shipped Phase 8 of the Rocdown boundary refactor: updated root README, ROADMAP, DESIGN, AGENT_SKILLS_PLAN, all crate READMEs, public `docs/` site pages, canonical knowledge records, and knowledge indexes; audited obsolete Rocs references across active code and docs; verified workspace dependencies and test matrix.
- Shipped Phase 7 of the Rocdown boundary refactor: created `crates/rocci-ui` providing domain-neutral view models (`StatCardView`, `StatTone`), presentation renderers, and base styles (`themes/base.css`), eliminating duplication between `rocci-rocdown` and `rocci-okf`.
- Shipped Phase 6 of the Rocdown boundary refactor: extracted portable, zero-dependency `crates/okf` engine and created `crates/rocci-okf` CLI and dev server application. Completely removed `okf` module and `knowledge` subcommands from `rocci-rocdown` and `rocci-rocdown-cli`, allowlisting the temporary presentation edge in `scripts/check-workspace-deps.py`.
- Froze Phase 0 of the Rocdown product-boundary contract: approved names are `rocci-rocdown`, `rocci-rocdown-cli` / `rocdown`, `rocci-okf`, and portable engine `okf`; there is no `rocs` / `rocs.toml` compatibility window; `RDxxxx` diagnostic codes stay; `rocci-okf` may use a temporary Rocdown presentation adapter only with a tracking issue. Encoded the dependency rules in `scripts/check-workspace-deps.py`. Current architecture records remain descriptive of shipped behavior.
- Clarified the recommended naming layers: Rocdown is the product, format, configuration prefix, and `rocdown` executable; `rocci-rocdown` is the Rust facade and `rocci-rocdown-cli` its Cargo command package; `rocci-okf` remains the recommended Rocci application name while its portable engine requires a separate neutral name. Confirmed that Rocs disappears from every active surface after the compatibility window.
- Recorded maintainer approval of the Rocci/Rocdown product symmetry and separate CLI ownership: `rocci run` owns applications and `.rocci`, while `rocdown run` owns both single interactive documents and documentation sites. The decision remains draft pending evidence review and the narrower naming, compatibility, and temporary OKF-adapter choices.
- Recorded the proposed Rocdown product consolidation and a phased refactor plan: base Rocci becomes unaware of Rocdown; the format, static generator, CLI, themes, and document tooling become one Rocdown product; Rocs is retired; dependency arrows are mechanically enforced; OKF remains independent and strict Markdown; and shared Rocci UI is extracted only from demonstrated duplication. Reconciled the standalone OKF application and language-tooling plans with this boundary and left the OKF product name open.
- Updated the [Rocdown format](architecture/rocdown-format.md) architecture record for shipped ordinary footnotes and the `@img` / `@docs figure` alt contract; standalone preview copies local images without hashing.
- Researched the emerging OKF tool ecosystem and recorded a proposed standalone
  Rocci OKF direction: extract a portable UI-neutral engine from the current Rocs implementation, preserve
  direct agent Markdown authoring, make the browser an evidence review and
  query surface, bind decisions to exact revisions, serve authorized evidence
  through scoped API tokens and MCP, and add full-text then optional semantic
  retrieval only under versioned evaluation.
- Evaluated the branding report as a Rocdown long-form authoring case study.
  Recorded strengths, cross-consumer image-field loss, validated links that are
  not rewritten for static output, citation and asset gaps, report-presentation
  needs, and a phased path toward frictionless static report generation without
  expanding Rocdown into a word processor.
- Made the branding and community foundation report a Rocdown document with
  page metadata and native `@img` declarations for all seven visual concept
  sheets; updated its canonical research and plan source references.
- Added exploratory branding and community research, a public-preview plan,
  three generated logo-direction probes, and a detailed report. The working
  recommendation keeps Rocci as the preview masterbrand, endorses Rocdown as
  its document format, and presents Rocs publicly as Rocci Docs while retaining
  implementation names for compatibility. Corrected the logo brief to exclude
  the existing `r` placeholder as evidence, added four zero-based probes, and
  retained the orange folded R only as the maintainer's current preference
  alongside non-letter and wordmark-only comparison routes.
  Added a dated exact-name package, repository, and Rocweave-domain snapshot;
  prepared Roc-first and Datastar-focused launch messages plus structured
  feedback and synthesis templates; and specified and responsively tested a
  dedicated landing-page direction that uses no candidate logo.
- Extended the language-server report and exploratory plan with a shared
  token-span architecture for static Rocs syntax highlighting, including
  fenced code, `@docs include`, and `@docs example`; recorded that current
  Rocs output is escaped but not highlighted per token.
- Added the draft [language-tooling architecture](architecture/language-tooling.md), separating the current common LSP and editor registrations from the still-unconsumed Roc/CSS embedded ranges and recording the current `@docs` build regression.
- Added the exploratory [full language-server plan](plans/language-server.md) and its detailed root report, recommending server-owned region/projection composition and reuse of the pinned Roc Tree-sitter grammar/query work rather than merging the whole Zed Roc extension.
- Updated the implementation snapshot to distinguish the shipped basic host-language LSP from the proposed embedded-language and Roc-semantic work, and to account for the new third root plan.

## 2026-08-16

- Added the strict OKF parser, static Rocdown body profile, knowledge CLI, and the first three representative records.
- Added all priority-1 architecture, decision, and status records; progressive directory indexes; and the report-section migration matrix.
- Added lifecycle and git source-drift diagnostics with stable `OKF4004`–`OKF4007` codes.
- Added the exploratory [client-behavior island decision](decisions/client-behavior-islands.md) without presenting it as approved or implemented.
- Added the [priority-1 review checklist](reference/priority-1-review.md) with verification initially pending.
- Recorded `human:nils` verification for all ten priority-1 records and promoted those revisions to `stable`.
- Added the draft root design reference plus design-system and design-token knowledge records, inventorying the two current CSS surfaces and keeping DTCG strictly in research scope.
- Corrected the theming and implementation-status records after the Phase 4 contract amendment; their historical verification remains recorded, but their revised content returned to `draft` pending human review.
- Added deterministic `search.json` and `llms.txt` outputs, filtered catalog inspection and knowledge search, CI validation/determinism checks, and a draft local-only publication decision.
- Updated the Rocs compiler and known-limitations records to distinguish shipped OKF retrieval from still-missing public documentation-site search; both revisions returned to `draft` pending review.
- Archived seven dated root reports without deleting them, retained the two active implementation/design plans at the root, and recorded that no canonical concept is currently superseded.
- Added a seven-question lexical retrieval benchmark with lifecycle and authority expectations, CLI measurement, and CI enforcement before any embeddings or database service.
- Corrected the public project-status page now that Rocs aliases, watch mode, and live reload ship; kept audience-facing status prose because knowledge publication remains local-only.
- Shipped bounded `@docs` components, catalog includes, Markdown/search projections, and opt-in `rocs test`; knowledge records distinguish that from still-unshipped `api-operation` and tab JS.
- Clarified that document-root HTML-shaped syntax is a structured Rocci template island, not raw HTML; recorded the separate trusted uses of `dangerously_include_unescaped_html` and the consequence that Rocs rejects root template islands while internally composing escaped Rust-rendered fragments through that bridge.
