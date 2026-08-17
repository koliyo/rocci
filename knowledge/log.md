# Knowledge log

## 2026-08-17

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
