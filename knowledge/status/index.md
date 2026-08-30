# Status

* [Implementation](implementation.md) - Dated snapshot of shipped capabilities.
* [Known limitations](known-limitations.md) - Deliberately absent or incomplete capabilities.
* [WASI HTTP module PR maturity](wasi-http-module.md) - Open PR 82 is experimental but capability-complete for compiled `.rocci` under `wasmtime serve`; not merge-ready (dirty vs main, no CI). Still omitted: Cmd, in-guest TLS, desktop URL. Not shipped on `main`.
* [OKF preview compile and render cost results](okf-compile-render-cost.md) - Machine-local debug `run --profile-report json` after Phases 1–3 and 6; Phases 4–5 skipped.
* [OKF load-performance improvement results](okf-load-performance.md) - Pointer; canon is [okmate](https://github.com/koliyo/okmate/blob/main/knowledge/status/okf-load-performance.md).
