# rocci-wasi-http-component

Portable WASI 0.3 `wasi:http/service` component. Phase 1 maps WASI
requests onto `rocci-wasi-http` `Adapter` + `StubGuest` (`map` feature,
no Wasmtime). No Roc.

## Pins (Phase 0)

| Item | Pin |
| --- | --- |
| Wasmtime CLI | 48.0.1 |
| WIT | `wasi:http@0.3.0` world `service` (`handle: async func`) |
| Bindings | `wasip3` 0.8.0 (`+wasi-0.3.0`) |
| Rust target | `wasm32-wasip2` (`wasm32-wasip3` is Tier 3, no prebuilt std) |
| `async func` lifts | Yes, on this `wasip3` + Wasmtime 48 line |

`wstd` / `#[wstd::http_server]` is WASI 0.2 `proxy` (`incoming-handler`). That is
compatibility only, not this crate's export.

## Build and serve

```sh
cargo build -p rocci-wasi-http-component --target wasm32-wasip2
wasmtime serve -Sp3 -Scli --addr 127.0.0.1:8080 \
  target/wasm32-wasip2/debug/rocci_wasi_http_component.wasm
curl -i http://127.0.0.1:8080/
```

`GET /` is 200 `text/html` with
`<!doctype html><html><body>hello-web</body></html>`.

`-Sp3` selects WASI 0.3. `-Scli` satisfies the `wasi:cli@0.2.9` imports
that Rust `std` on `wasm32-wasip2` still pulls in. That is not a 0.2
`proxy` export: `wasm-tools component wit` shows
`export wasi:http/handler@0.3.0` with `handle: async func`.
