# Rocci Blocks — frozen v1 protocol

Working name: **Rocci Blocks**. Public copy says “falling-block arena”.

These constants, schemas, and JSON tables freeze Phase 0. Later phases
implement them; they do not change the numbers without a new freeze.

## Constants

| Name | Value |
| --- | --- |
| Board | 10×20, row-major, `y = 0` at the top |
| Cell alphabet | `.` empty, `IJLOSTZ` locked pieces, `G` garbage |
| Board encoding | 200 characters, no separators |
| Attack table | 0/1 rows → 0; 2 → 1; 3 → 2; 4 → 4 |
| Back-to-back | +1 only on consecutive 4-row clears |
| Garbage delay | 600 ms until a packet is ready |
| Insertion cap | 8 rows applied per accepted lock |
| Hole policy | first packet hole 3; later packets `(hole + 3) mod 10` |
| Target | next living seat in ring order from the cursor |
| Cursor advance | only after a non-zero residual attack |
| Seats | 8, ring order 0–7 |
| Spectator leases | design ceiling 50; public default cap **20** until a 30-minute 8+50 soak |
| Countdown | 10 s after ≥2 ready players |
| Round timeout | 5 min, then rank by stack height then lines sent |
| Result display | 10 s |
| Disconnect grace | 10 s |
| Stream emit ceiling | 5 snapshots/s/connection |
| Idle keepalive | ≥1 per 10 s |
| Snapshot ceiling | 4096 bytes uncompressed HTML |
| Lock rate ceiling | 10 POST/s/player |
| Command body limit | 4096 bytes |
| Lease TTL | 15 s |
| Stream path | `GET /play/blocks/stream` |
| Lock path | `POST /play/blocks/command/lock` |
| Session cookie | `blocks`, HttpOnly, SameSite=Lax, Path=`/play/blocks` |

## Piece geometry

Origin is the top-left of the piece’s bounding box. Cells are `(x + dx, y + dy)`.

| Piece | rot 0 | rot 1 | rot 2 | rot 3 |
| --- | --- | --- | --- | --- |
| I | (0,1)(1,1)(2,1)(3,1) | (2,0)(2,1)(2,2)(2,3) | (0,2)(1,2)(2,2)(3,2) | (1,0)(1,1)(1,2)(1,3) |
| O | (1,0)(2,0)(1,1)(2,1) | same | same | same |
| T | (1,0)(0,1)(1,1)(2,1) | (1,0)(1,1)(2,1)(1,2) | (0,1)(1,1)(2,1)(1,2) | (1,0)(0,1)(1,1)(1,2) |
| S | (1,0)(2,0)(0,1)(1,1) | (1,0)(1,1)(2,1)(2,2) | (1,1)(2,1)(0,2)(1,2) | (0,0)(0,1)(1,1)(1,2) |
| Z | (0,0)(1,0)(1,1)(2,1) | (2,0)(1,1)(2,1)(1,2) | (0,1)(1,1)(1,2)(2,2) | (1,0)(0,1)(1,1)(0,2) |
| J | (0,0)(0,1)(1,1)(2,1) | (1,0)(2,0)(1,1)(1,2) | (0,1)(1,1)(2,1)(2,2) | (1,0)(1,1)(0,2)(1,2) |
| L | (2,0)(0,1)(1,1)(2,1) | (1,0)(1,1)(1,2)(2,2) | (0,1)(1,1)(2,1)(0,2) | (0,0)(1,0)(1,1)(1,2) |

Spawn origin is `(3, 0)` for every piece. Wall kicks try, in order:
`(0,0), (-1,0), (1,0), (0,1), (-1,1), (1,1), (0,-1)`. No 180° rotation.

Seven-bag: shuffle `IJLOSTZ` with the room LCG (`seed = (seed * 1103515245 + 12345) & 2147483647`).

## Lock command

```json
{
  "piece": "T",
  "rotation": 0,
  "x": 3,
  "y": 18,
  "board_revision": 4,
  "sequence": 12
}
```

`piece` is one of `IJLOSTZ`. `rotation` is `0..3`. Idempotency key is
`(player_id, sequence)`.

## Acknowledgements

200 body:

```json
{
  "ok": true,
  "board_revision": 5,
  "board": "<200 chars>",
  "next_piece": "I",
  "hold_queue": 0,
  "target": 2,
  "eliminated": false,
  "lines_sent": 0
}
```

409 body (resync): the same player snapshot plus `"ok": false` and `"error"`.

## Error tags

| Tag | When |
| --- | --- |
| `StaleRevision` | `board_revision` ≠ server board revision |
| `InvalidGeometry` | out of bounds or overlap |
| `UnknownPiece` | piece id ≠ current bag piece, or not `IJLOSTZ` |
| `BadRotation` | rotation not in `0..3` |
| `DuplicateSequence` | replay of an acknowledged sequence (returns stored 200) |
| `RateLimited` | more than 10 lock POSTs in any rolling second |
| `WrongPhase` | lock while lobby/countdown/result, or spectator lock |
| `InvalidOrigin` | missing/invalid Origin/Referer for a mutating request |
| `OversizedBody` | body larger than 4096 bytes |
| `Unauthenticated` | missing session cookie |
| `SeatGone` | grace elapsed; seat eliminated |
| `CapacityReached` | spectator lease pool exhausted |

## Round lifecycle

1. **Lobby** — seats fill; spectators watch. ≥2 ready players start a 10 s countdown.
2. **Countdown** — roster frozen if still ≥2 ready at fire; otherwise return to Lobby.
3. **Round** — late arrivals spectate or queue. Mid-round join as a player is forbidden.
4. **Eliminated** — topped-out player keeps the player stream; does not take a spectator lease.
5. **Result** — one survivor, or 5-minute timeout ranking by lower occupied rows then lines sent.
6. **Reset** — show Result 10 s, promote queue into open seats, return to Lobby.

Process restart moves the active round to `Result(Interrupted)` and opens a fresh Lobby.

## Stream manifest

One Datastar patch of `#blocks-arena-state`. Every event is a full snapshot.
Attributes on the root: `data-revision`, `data-phase`, `data-deadline-ms`,
`data-round`. One child per occupied seat:

```html
<div id="blocks-arena-state" data-revision="12" data-phase="round"
     data-deadline-ms="0" data-round="3">
  <div data-seat="0" data-status="alive" data-board="..." data-target="2"
       data-queue="3" data-ready="1" data-piece="T" data-you="1"></div>
</div>
```

`data-board` is the 200-character encoding. Spectators see committed boards,
not falling-piece poses. Slow clients skip revisions; the next event replaces
the manifest.

## JSON tables

Each file in this directory is one semantic family. Every object has `id`,
`input`, and `output`.
