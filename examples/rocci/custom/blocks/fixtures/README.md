# Rocci Blocks fixtures

Phase 0 input/output tables for the frozen v1 rules. Read
[`protocol.md`](protocol.md) first.

| File | Semantic example |
| --- | --- |
| `placement.json` | legal lock, overlap, out of bounds, rotation |
| `row-clear.json` | 0–4 row clears and the attack table, including back-to-back |
| `cancellation.json` | oldest-first cancel; cursor does not advance on full cancel |
| `target-rotation.json` | ring order, skipped eliminated seats, two players, retarget |
| `garbage-holes.json` | delay, 8-row cap, hole columns |
| `top-out.json` | spawn blocked after lock or garbage |
| `duplicate-sequence.json` | idempotent `(player_id, sequence)` |
| `reconnect.json` | full snapshot recovery; no event replay |
| `snapshot-budget.json` | 8-board manifest stays ≤ 4 KiB; 5 Hz / 10 s keepalive |

Validate:

```sh
python3 -c "import json, pathlib; p=pathlib.Path('examples/rocci/custom/blocks/fixtures');
files=list(p.glob('*.json'));
assert files, 'no fixtures'
for f in files:
    data=json.loads(f.read_text());
    rows=data if isinstance(data, list) else data['cases']
    for row in rows:
        assert 'id' in row and 'input' in row and 'output' in row, f
print(f'ok {len(files)} files')"
```
