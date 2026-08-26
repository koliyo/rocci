# OKF

Portable engine, review application, knowledge load and render, and bundle layout.

* [Nested OKF collections](nested-collections.md) - Type-first collections with closed product-area subdirectories under plans, research, and audits. Exploratory; engine and viewer work is in this revision. Decision: [nested collections](/decisions/nested-okf-collections.md).
* [Multiple knowledge roots](multi-knowledge-roots.md) - Pointer; canon is [okmate](https://github.com/koliyo/okmate/blob/main/knowledge/plans/okf/multi-knowledge-roots.md).
* [Settings UX for knowledge roots](settings-ux.md) - Pointer; canon is [okmate](https://github.com/koliyo/okmate/blob/main/knowledge/plans/okf/settings-ux.md).
* [Okmate — extractable Rust OKF mate](okmate.md) - Pointer; canon is [okmate](https://github.com/koliyo/okmate/blob/main/knowledge/plans/okf/okmate.md).
* [Rust-templated OKF viewer with Datastar](okf-viewer-rust-datastar.md) - Pointer; canon is [okmate](https://github.com/koliyo/okmate/blob/main/knowledge/plans/okf/okf-viewer-rust-datastar.md).
* [Short-term OKF viewer host surfaces](okf-viewer-host-surfaces.md) - Pointer; canon is [okmate](https://github.com/koliyo/okmate/blob/main/knowledge/plans/okf/okf-viewer-host-surfaces.md).
* [Standalone Rocci OKF review and query application](rocci-okf-app.md) - Pointer; canon is [okmate](https://github.com/koliyo/okmate/blob/main/knowledge/plans/okf/rocci-okf-app.md).
* [OKF load-performance improvements](okf-load-performance.md) - Pointer; canon is [okmate](https://github.com/koliyo/okmate/blob/main/knowledge/plans/okf/okf-load-performance.md).
* [Deferred OKF compile and render follow-ons](okf-compile-render-follow-ons.md) - Future work for the first three compile/render non-goals: keep embedded page Roc in the hash, skip-roc as an explicit host not the default, wasm apply-to-disk without embedding `roc`. Exploratory; no phase started.
* [OKF preview compile and render cost](okf-compile-render-cost.md) - After load-performance work, stop baking page HTML into the Roc renderer hash, write Rocci chrome from apply, and reuse the applicator across watch ticks. Exploratory; Phases 1–3 and 6 in this tree; Phases 4–5 skipped; not CI-complete. Follow-on: [deferred compile/render follow-ons](okf-compile-render-follow-ons.md).
