# Knowledge log

## 2026-08-17

- Added the [Rocdown product-boundary refactor completion review](audits/rocdown-boundary-refactor-review.md), distinguishing the successful package and CLI consolidation from red Phase 8 workflows, missing Rocdown editor wiring, an unnecessary OKF-to-Rocdown edge, over-extracted shared UI, and stale knowledge and operational documentation. Recorded follow-up CI fixes in `b353895` as locally validated but not yet verified by a visible GitHub run.
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
