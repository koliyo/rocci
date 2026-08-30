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

`rocci run` stays native 0.16. `rocci build --http-module App.rocci` lowers
the app, `roc build --target=wasm32` against sibling `../roc-basic-webserver`
(or `ROCCI_BASIC_WEBSERVER`), and writes a `wasi:http/service` **WASI 0.3**
component. Serve with `wasmtime serve -Sp3 -Scli`. Sqlite apps need
`--env DB_PATH=…` and `--dir=host::guest`. `--host wasm` stays apply.
Still omitted: Cmd, in-guest TLS, desktop URL.
