# Counter

Local run notes. Published tutorial: https://rocci.dev/examples/counter/

```sh
cargo run -q -p rocci-cli -- run examples/rocci/standalone/counter/Counter.rocci
```

`--no-window` serves http://127.0.0.1:8000. `DB_PATH` overrides the SQLite file.

```sh
curl -s http://127.0.0.1:8000/health
curl -s -X POST http://127.0.0.1:8000/actions/counter/increment
```

Experimental WASI 0.3 component (`rocci run` stays native):

```sh
cargo run -q -p rocci-cli -- build --http-module \
  examples/rocci/standalone/counter/Counter.rocci -o http-module.wasm
mkdir -p .counter-data
wasmtime serve -Sp3 -Scli --env DB_PATH=./counter.db \
  --dir=.counter-data::. --dir=./http-module.assets::/assets \
  --addr 127.0.0.1:8080 http-module.wasm
curl -s http://127.0.0.1:8080/
curl -s -X POST http://127.0.0.1:8080/actions/counter/increment
curl -s -X POST http://127.0.0.1:8080/actions/counter/reset
```

Increment morphs only the tab that posted. Other tabs do not subscribe; use
[live-counter](../live-counter/) for `GET /sse` across clients.
