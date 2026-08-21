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
