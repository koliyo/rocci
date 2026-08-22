# Rocci Blocks

Local run notes. Published tutorial: https://rocci.dev/examples/blocks/

```sh
cargo run -q -p rocci-cli -- run examples/rocci/standalone/blocks/backend/Blocks.rocci
```

Open two windows on the printed URL. A move in one tab morphs `#board` in both.

```sh
curl -s -D - -o /dev/null -X POST http://127.0.0.1:8000/actions/reset
# empty body; HTTP 204
```
