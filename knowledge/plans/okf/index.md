# OKF

Portable engine, review application, knowledge load and render, and bundle layout.

* [Nested OKF collections](nested-collections.md) - Type-first collections with closed product-area subdirectories under plans, research, and audits. Exploratory; engine and viewer work is in this revision. Decision: [nested collections](/decisions/nested-okf-collections.md).
* [Multiple knowledge roots](multi-knowledge-roots.md) - User-level TOML registry of directory and git OKF roots, cached checkouts, directed edge policy, settings UI, and agent path listing. Exploratory; no phase started.
* [Standalone Rocci OKF review and query application](rocci-okf-app.md) - Portable `okf` engine extraction and a `rocci-okf` application for evidence review and authenticated retrieval.
* [OKF load-performance improvements](okf-load-performance.md) - Phased reduction of `okf::load` latency: split load spans, batch git provenance, preview-without-provenance, watch parse cache. Phases 1–4 implemented; Phase 5 skipped after a sub-second release remeasure.
* [Deferred OKF compile and render follow-ons](okf-compile-render-follow-ons.md) - Future work for the first three compile/render non-goals: keep embedded page Roc in the hash, skip-roc as an explicit host not the default, wasm apply-to-disk without embedding `roc`. Exploratory; no phase started.
* [OKF preview compile and render cost](okf-compile-render-cost.md) - After load-performance work, stop baking page HTML into the Roc renderer hash, write Rocci chrome from apply, and reuse the applicator across watch ticks. Exploratory; Phases 1–3 and 6 in this tree; Phases 4–5 skipped; not CI-complete. Follow-on: [deferred compile/render follow-ons](okf-compile-render-follow-ons.md).
