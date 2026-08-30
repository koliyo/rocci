# rocci-wasi-http-component

Portable WASI 0.3 `wasi:http/service` component. `GET /` calls the linked
Roc `roc_*_for_host` object (`fixtures/env_log.o`). That fixture reads
`GREETING`, logs `env-log` on stderr, and returns ordinary HTML
`<p>${GREETING}</p>`. Other routes echo mapped request fields. No
Wasmtime in this crate. Phase 2 hello-web object stays at
`fixtures/roc_app.o`.

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
wasmtime serve -Sp3 -Scli --env GREETING=phase3-greeting --addr 127.0.0.1:8080 \
  target/wasm32-wasip2/debug/rocci_wasi_http_component.wasm
curl -i http://127.0.0.1:8080/
```

`GET /` is 200 `text/html` with `<p>phase3-greeting</p>`. The serve host
stderr includes a line `env-log`.

Mapped fields (same names as `maps_get_path_query_and_headers` /
`buffers_post_body`) echo as `text/plain` on other routes:

```sh
curl -s http://127.0.0.1:8080/hello?x=1 -H 'accept: text/html'
# path=/hello query=x=1 header.accept=text/html
curl -s -X POST http://127.0.0.1:8080/ --data-binary abc
# method=7 body=abc content_length=3
```

SSE (`stream<u8>`, Wait via `wasi:clocks` `wait-for`; idle timeout is the serve host's):

```sh
curl -s -D - http://127.0.0.1:8080/sse-empty   # immediate End
curl -s -D - http://127.0.0.1:8080/sse-wait    # 200ms Wait then one Emit
```

Two overlapping `/sse-wait` connections stay near one wait (~220ms), not two.

Preopen (`wasmtime serve --dir=crates/rocci-wasi-http/fixtures/static::/`):

```sh
curl -s http://127.0.0.1:8080/hello.txt   # preopen-bytes
```

## SQLite

SQLite stays **embedder-only**. `libsqlite3-sys` (bundled rusqlite) needs a C
toolchain for `wasm32-wasip2`; host `clang` on this line has no
`wasm32-unknown-wasip2` target (`WASI_SYSROOT` unset). This crate does not
depend on the native `embedder` rusqlite feature. Nested sqlite inside
`respond!` still **serializes** other `handle`s (parent Phase 0
measurement); fibers do not park that path.

`-Sp3` selects WASI 0.3. `-Scli` satisfies the `wasi:cli@0.2.9` imports
that Rust `std` on `wasm32-wasip2` still pulls in. That is not a 0.2
`proxy` export: `wasm-tools component wit` shows
`export wasi:http/handler@0.3.0` with `handle: async func`.
