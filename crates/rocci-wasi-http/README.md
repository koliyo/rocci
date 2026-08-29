# rocci-wasi-http

WASI 0.3 `wasi:http/service` adapter for basic-webserver 0.16 Roc apps.

**Shipped (experimental):** request mapping, stub and linked hello-web `handle`,
SSE Wait as adapter clocks, one preopen `file_root`, sync sqlite (serializes
other `handle`s). **Not shipped:** replacing `rocci run` or musl publish;
`--host wasm` remains apply. Nested sqlite/file inside `respond!` does not
yield; SSE `Wait` does.

```sh
cargo test -p rocci-wasi-http
cargo test -p rocci-wasi-http --no-default-features --features map
cargo run -q -p rocci-cli -- build --http-module App.rocci -o http-module.wasm
```

Default features are `map` (abi, guest stubs, `Adapter`, files) plus
`embedder` (Wasmtime, rusqlite, probe tests). The portable component
crate depends on `map` only.

`rocci run` stays native 0.16. `rocci build --http-module` writes **core wasm**
with 0.16 `roc_*_for_host` exports (hello-web WAT), not a compiled `.rocci` app
and not a WASI 0.3 component. `wasmtime serve` requires a component and will
refuse that file. Exercise `handle` with the crate embedder tests.
