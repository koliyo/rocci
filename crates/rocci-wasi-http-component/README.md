# rocci-wasi-http-component

Portable WASI 0.3 `wasi:http/service` component. App routes call the
linked Roc `roc_*_for_host` object (`ROCCI_ROC_APP_O`, or
`fixtures/sqlite_row.o` when that env is unset). The default fixture
opens in-memory sqlite and returns `hello-sqlite`. `rocci build
--http-module` supplies the compiled `.rocci` object. Hosted
`hosted_sqlite_*` is sync and serializes other `handle`s. `GET /assets/*`
is a preopen file (`wasmtime serve --dir=http-module.assets::/assets`).
`GET /hello.txt` is the fixture preopen; `/sse-empty` and `/sse-wait` stay
fixture guests.
No Wasmtime in this crate. Earlier fixtures stay at `fixtures/roc_app.o`
and `fixtures/env_log.o`.

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

`GET /` is 200 `text/html` with `hello-sqlite` from in-component sqlite.
Nested sqlite inside `respond!` / `init!` serializes other `handle`s.

SSE fixtures (`stream<u8>`, Wait via `wasi:clocks` `wait-for`; idle timeout is the serve host's):

```sh
curl -s -D - http://127.0.0.1:8080/sse-empty   # immediate End
curl -s -D - http://127.0.0.1:8080/sse-wait    # 200ms Wait then one Emit
```

Two overlapping `/sse-wait` connections stay near one wait (~220ms), not two.

Preopen (`wasmtime serve --dir=crates/rocci-wasi-http/fixtures/static::/`
for `/hello.txt`, and `--dir=http-module.assets::/assets` after
`--http-module`):

```sh
curl -s http://127.0.0.1:8080/hello.txt           # preopen-bytes
curl -sI http://127.0.0.1:8080/assets/datastar.js # 200 text/javascript
curl -sI http://127.0.0.1:8080/assets/rocci.css   # 200 text/css after --http-module
```

## SQLite

The component hosts `hosted_sqlite_*` against wasm-safe sqlite3.c (zig
`wasm32-wasi-musl`; `WASI_SYSROOT` unset). It does not depend on the native
`embedder` rusqlite feature. Nested sqlite inside `respond!` / `init!`
**serializes** other `handle`s; fibers do not park that path.

`-Sp3` selects WASI 0.3. `-Scli` satisfies the `wasi:cli@0.2.9` imports
that Rust `std` on `wasm32-wasip2` still pulls in. That is not a 0.2
`proxy` export: `wasm-tools component wit` shows
`export wasi:http/handler@0.3.0` with `handle: async func`.
