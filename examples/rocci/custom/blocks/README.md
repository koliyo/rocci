# Rocci Blocks

Custom `main.roc` arena. Local run:

```sh
cargo run -q -p rocci-cli -- inspect --ast examples/rocci/custom/blocks/Blocks.rocci
cargo run -q -p rocci-cli -- run examples/rocci/custom/blocks
```

`--no-window` serves http://127.0.0.1:8000/play/blocks/

```sh
curl -s http://127.0.0.1:8000/health/blocks
curl -s http://127.0.0.1:8000/play/blocks/ | grep -E 'Join|Falling-block'
BLOCKS_BASE=http://127.0.0.1:8000 DB_PATH=./blocks.db python3 examples/rocci/custom/blocks/scripts/phase2-smoke.py
python3 examples/rocci/custom/blocks/scripts/phase3-rules.py
BLOCKS_BASE=http://127.0.0.1:8000 DB_PATH=./blocks.db python3 examples/rocci/custom/blocks/scripts/phase3-eight.py
```

`BLOCKS_COUNTDOWN_MS`, `BLOCKS_RESULT_MS`, and `BLOCKS_ROUND_MS` shorten lobby timers in tests.
`DB_PATH` overrides the SQLite file.

Keyboard: arrows / WASD, Up/X rotate, Z CCW, Space hard drop.
