# Plans

* [Ungrammar AST codegen for Rocci and Rocdown](ungram-ast.md) - Phased owned-struct generation from per-language ungrams via a shared `rocci-ungram` CLI; scanners and parsers stay hand-written. Exploratory; Phases 1–5 implemented on `ungram-ast-implementation`, not CI-complete.
* [Ungram follow-on backends after owned-struct codegen](ungram-follow-ons.md) - Freeze inspect tags and generate exhaustive `format_ast` walkers, add `NodeKind` highlighter coverage, generate `MdNode` from a Markdown ungram, and emit a public tree appendix. Do not generate `SyntaxKind`, highlighters, or a CST. Exploratory; no phase started.
* [OKF preview compile and render cost](okf-compile-render-cost.md) - After load-performance work, stop baking page HTML into the Roc renderer hash, write Rocci chrome from apply, and reuse the applicator across watch ticks. Exploratory; no phase started.
* [OKF load-performance improvements](okf-load-performance.md) - Phased reduction of `okf::load` latency: split load spans, batch git provenance, preview-without-provenance, watch parse cache. Phases 1–4 implemented; Phase 5 skipped after a sub-second release remeasure.
* [Hybrid Rocdown islands for CDN-static sites](hybrid-rocdown-islands.md) - Phases 1–8 are on the hybrid branch (BlockCall rebase plus dual apply: widget forest and island splice); phases 9–10 remain. Exploratory; not shipped.
* [Generalized Rocdown block model](generalized-rocdown-block-model.md) - Phased delivery of uniform article `BlockCall` nodes, `:name[params]` spelling, a closed builtin registry, and per-kind Rocci renderers. Exploratory; not shipped.
* [Full Rocci and Rocdown language tooling](language-server.md) - Proposed region-aware editor tooling with shared token spans and product-owned server composition under the boundary refactor.
* [Public-preview branding and community](public-preview-community.md) - Reversible launch gate, Roc and Datastar feedback sequence, and evidence-based naming and identity decisions.
* [rocci.dev site architecture and Rocdown evolution](rocci-dev-site.md) - Proposed site structure, Rocdown/Rocci authoring split, named layouts, collections, and the decision boundary for a possible `rocci-site` profile.
* [Standalone Rocci OKF review and query application](rocci-okf-app.md) - Portable `okf` engine extraction and a `rocci-okf` application for evidence review and authenticated retrieval.
* [Rocdown product-boundary refactor](rocdown-boundary-refactor.md) - Phased consolidation of the Rocdown format and static generator, removal from base Rocci, Rocs retirement, and OKF separation.
* [CLI entry points for Rocci, Rocdown, and OKF preview](cli-entry-points.md) - Keep the three product CLIs, reject a plugin host, and make `rocci-okf run` the file-aware OKF viewer.
* [First-party Rocci chrome library and generation host](rocci-component-generation.md) - Extract demonstrated outline/nav/breadcrumb chrome into base Rocci, host Roc through a cached native subprocess and Wasmtime, and persist both generated Roc and compiled artifacts.
* [Mobile chrome for OKF, Rocdown, and rocci.dev](mobile-chrome.md) - No-JS details menus, OKF nav split from TOC, rocci.dev menu restore, table overflow. Exploratory; no phase started.
* [Preview inspector source views](inspector-source-views.md) - Extend the preview Dev panel with a dropdown for original source, AST, generated Roc, and generated HTML. Exploratory; no phase started.

Other implementation-plan concepts have not yet been migrated. Priority-2 plan migration remains scheduled after the Phase 2 priority-1 corpus.
