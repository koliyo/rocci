# rocci-wasi-http

WASI 0.3 `wasi:http/service` adapter for basic-webserver 0.16 Roc apps.

**Shipped (experimental):** request mapping, stub and linked hello-web `handle`,
SSE Wait as adapter clocks, one preopen `file_root`, sync sqlite (serializes
other `handle`s). **Not shipped:** replacing `rocci run` or musl publish;
`--host wasm` remains apply. Nested sqlite/file inside `respond!` does not
yield; SSE `Wait` does.

```sh
cargo test -p rocci-wasi-http
cargo run -q -p rocci-cli -- build --http-module App.rocci -o http-module.wasm
wasmtime serve http-module.wasm   # when the artifact is a 0.3 component
```

`rocci run` stays native 0.16. `rocci build --http-module` currently writes the
hello-web guest WAT (0.16 export names), not a compiled `.rocci` app.
