# Snake

Local run notes. Published tutorial: https://rocci.dev/examples/snake/

Datastar + Rocci stress demo. Catalog hosting is live; the public hostname is
planned until a staging origin serves it. The play HUD reports live SSE morph
bandwidth.

```sh
cargo run -q -p rocci-cli -- run examples/rocci/custom/snake
```

```sh
curl -s http://127.0.0.1:8000/health
curl -s http://127.0.0.1:8000/ | grep -E 'Join game|Spectate'
```
