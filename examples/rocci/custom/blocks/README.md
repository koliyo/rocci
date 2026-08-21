# Rocci Blocks

Custom `main.roc` arena. Local run:

```sh
cargo run -q -p rocci-cli -- inspect --ast examples/rocci/custom/blocks/Blocks.rocci
cargo run -q -p rocci-cli -- run examples/rocci/custom/blocks
```

`rocci run` opens `/play/blocks/` (not `/`). Join starts a solo round immediately.

```sh
curl -s http://127.0.0.1:8000/health/blocks
curl -s http://127.0.0.1:8000/play/blocks/ | grep -E 'Join|Falling-block'
BLOCKS_BASE=http://127.0.0.1:8000 DB_PATH=./blocks.db python3 examples/rocci/custom/blocks/scripts/phase2-smoke.py
BLOCKS_BASE=http://127.0.0.1:8000 DB_PATH=./blocks.db python3 examples/rocci/custom/blocks/scripts/piece-flow.py
python3 examples/rocci/custom/blocks/scripts/phase3-rules.py
BLOCKS_BASE=http://127.0.0.1:8000 DB_PATH=./blocks.db python3 examples/rocci/custom/blocks/scripts/phase3-eight.py
BLOCKS_BASE=http://127.0.0.1:8000 BLOCKS_SPECTATOR_CAP=2 python3 examples/rocci/custom/blocks/scripts/phase4-harness.py
```

`BLOCKS_COUNTDOWN_MS`, `BLOCKS_RESULT_MS`, and `BLOCKS_ROUND_MS` shorten lobby timers in tests.
`DB_PATH` overrides the SQLite file. `BLOCKS_SPECTATOR_CAP` defaults to 20 (design ceiling 50).

Keyboard: arrows / WASD, Up/X rotate, Z CCW, Space hard drop.
Append `?debug=1` to the play URL to log stream snapshots and lock acknowledgements;
the log includes the player board revision used to discard stale stream snapshots.
