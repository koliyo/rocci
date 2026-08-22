# Datastar gallery

Local run notes. Published tutorial: https://rocci.dev/examples/datastar/

```sh
cargo run -q -p rocci-cli -- run examples/rocci/custom/datastar
```

`GET /actions/signals/compose` is the low-level authored-Roc ceiling: one
finite SSE response composes a typed `patch_elements` event with a typed
`patch_signals_with(..., [OnlyIfMissing(True)])` event. The source contains no
manual SSE framing and does not make browser signals durable domain state.

Planned hosted demo: https://datastar.examples.rocci.dev

```sh
curl -s http://127.0.0.1:8000/search
```
