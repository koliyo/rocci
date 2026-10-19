# Status

Current shipped-or-not snapshots first. Later bullets are dated measurements
or pointers; they do not replace the two current records. Do not nest this
collection by lifecycle.

* [Implementation](implementation.md) - Dated snapshot of shipped capabilities.
* [Known limitations](known-limitations.md) - Deliberately absent or incomplete capabilities.

Dated snapshots:

* [Worker source portability implementation evidence](worker-source-portability.md) - Frozen C0/C1/C2 source promises, per-phase microplans and actual Roc JSPI qualification; managed compatibility remains gated.
* [Private durable-actor backend results](portable-actor-backends.md) - Implemented private Cloudflare counter; hosted restore, rollback and automatic reconnect evidence, passing CI, resource measurements and explicit provider cleanup limits.
* [WASI HTTP module PR maturity](wasi-http-module.md) - Open PR 82 is experimental but capability-complete for compiled `.rocci` under `wasmtime serve`; not merge-ready (dirty vs main, no CI). Still omitted: Cmd, in-guest TLS, desktop URL. Not shipped on `main`.
* [OKF preview compile and render cost results](okf-compile-render-cost.md) - Machine-local debug `run --profile-report json` after Phases 1–3 and 6; Phases 4–5 skipped.
* [OKF load-performance improvement results](okf-load-performance.md) - Pointer; canon is [okmate](https://github.com/koliyo/okmate/blob/main/knowledge/status/okf-load-performance.md).
