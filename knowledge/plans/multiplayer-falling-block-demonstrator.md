---
type: Implementation Plan
title: Multiplayer falling-block demonstrator on rocci.dev
description: "Build one same-origin falling-block arena for up to eight active players and a measured target of fifty spectators. Browser code owns responsive piece motion; Roc validates committed locks, owns boards, targeting, garbage, rounds, leases, and compact recovery snapshots. Exploratory; Phases 0–5 implemented on multiplayer-falling-block-demonstrator. Staging Tunnel soak is an operator gate."
tags: [domain/rocci, domain/runtime, concern/architecture, concern/performance, concern/publication, integration/datastar]
status: draft
generated: { by: process:cursor, at: 2026-08-21T16:45:00Z }
stale_after: 2026-11-21
authority: exploratory
owners: [human:nils]
sources:
  - id: server-state
    resource: ../decisions/server-owned-state.md
    title: Keep durable application state server-owned
    author: human:nils
    last_modified: 2026-08-16
  - id: cqrs-research
    resource: ../research/datastar-cqrs-action-responses.md
    title: Datastar SSE is a per-request transport; generated apps do not fan out
    author: process:cursor
    last_modified: 2026-08-21
  - id: bws-limits
    resource: ../research/basic-webserver-sse-http.md
    title: basic-webserver 0.16 SSE and HTTP limits for Rocci live streams
    author: process:cursor
    last_modified: 2026-08-21
  - id: publish-plan
    resource: rocci-dev-publish.md
    title: Deploy rocci.dev with Cloudflare, a small VPS, and CI
    author: process:cursor
    last_modified: 2026-08-21
  - id: app-docs-plan
    resource: rocci-app-docs.md
    title: Documentation generator for Rocci applications
    author: process:cursor
    last_modified: 2026-08-21
  - id: snake-game
    resource: ../../examples/rocci/custom/snake/Game.roc
    title: Pure eight-player Snake game rules
    author: process:git
    last_modified: 2026-08-20
  - id: snake-main
    resource: ../../examples/rocci/custom/snake/main.roc
    title: Authored multiplayer server with SQLite and SSE polling
    author: process:git
    last_modified: 2026-08-20
  - id: snake-view
    resource: ../../examples/rocci/custom/snake/Snake.rocci
    title: Eight-player and spectator Rocci views
    author: process:git
    last_modified: 2026-08-21
  - id: snake-input
    resource: ../../examples/rocci/custom/snake/assets/snake-input.js
    title: Small browser input island that sends intent
    author: process:git
    last_modified: 2026-08-20
  - id: dispatch
    resource: ../../crates/rocci-cli/src/dispatch.rs
    title: Generated live polling, keepalives, and empty command SSE
    author: process:git
    last_modified: 2026-08-21
  - id: caddy
    resource: ../../docker/cdn/Caddyfile
    title: Current same-origin static and island routing
    author: process:git
    last_modified: 2026-08-21
  - id: compose
    resource: ../../docker/compose.hybrid.yml
    title: Current small-origin Caddy and islands services
    author: process:git
    last_modified: 2026-08-20
  - id: apps-catalog
    resource: ../../examples/rocci/apps.toml
    title: Rocci example catalog and hosting classifications
    author: process:git
    last_modified: 2026-08-21
  - id: jstris-guide
    resource: https://github.com/jezevec10/jstris-guide/blob/master/guide.md
    title: Jstris multiplayer guide - garbage distribution, blocking, and delay
    author: human:jezevec10
  - id: garbage-reference
    resource: https://tetris.wiki/Garbage
    title: Multiplayer falling-block garbage and targeting reference
    author: organization:tetris-wiki
---

# Multiplayer falling-block demonstrator on rocci.dev

## Purpose and authority

This plan proposes a public Rocci demonstrator, provisionally called **Rocci
Blocks**, in the familiar competitive falling-block genre. It is exploratory:
it selects a coherent v1 for implementation and measurement, but it does not
approve public launch, claim a fifty-spectator capacity, or start a delivery
phase.

The plan applies the established server-owned-state decision. The browser may
own the high-rate motion of the currently falling piece, just as Snake uses a
small input island, while Roc remains authoritative for committed boards,
piece order, attacks, rounds, membership, and recovery.[^server-state][^snake-input]

## Goal

Serve one continuously available arena at
`https://rocci.dev/play/blocks/` with:

- two to eight active players in one free-for-all round;
- up to fifty connected spectators as a **measured target**, not an assumed
  entitlement;
- compact, revisioned server snapshots over one long-lived SSE stream per
  browser;
- responsive keyboard, touch, and gamepad-ready local controls without a
  client copy of room authority;
- recognizable one-target garbage attacks, cancellation, warning, delay, and
  top-out;
- a small-origin operational budget compatible with the existing 2-vCPU / 4-GB
  rocci.dev VPS.[^publish-plan]

The demonstrator should make the Rocci boundary visible: pure game rules in
Roc, documents and state manifests in Rocci, Datastar morphing the authoritative
snapshot, and a deliberately small JavaScript island for animation and input.

## Out of bound

- Ranked matchmaking, accounts, chat, friend lists, tournaments, teams, bots,
  replays, or permanent leaderboards.
- More than one simultaneous public room in v1.
- Mid-round joining as an active player; late arrivals spectate and queue for
  the next round.
- Tournament-grade anti-cheat, rollback netcode, or server simulation of every
  keypress and animation frame.
- Manual targeting, badges, attacker bonuses, T-spin recognition, perfect-clear
  bonuses, 180-degree rotation, or a configurable rules laboratory in v1.
- Sending one attack to every opponent. That amplifies one clear by player
  count and makes eight-player load and balance harder to reason about.
- A general Rocci base-path feature or a new `.rocci` language declaration.
- Moving the whole rocci.dev site into the game process, adding Kubernetes, or
  resizing the VPS before measurement.
- Using Tetris logos, audio, visual assets, or implying endorsement. Public copy
  should say “falling-block arena”; “Tetris” remains a genre reference in
  planning only.

## Constraints that do not move

| Constraint | Required behavior |
| --- | --- |
| Server authority | Roc validates every committed piece lock and derives clears, garbage, target, next piece, elimination, and revision. The client never submits “I cleared four lines” as a trusted fact.[^server-state] |
| Responsive local play | Key repeat, rotation previews, gravity, ghost piece, and the falling piece render locally. A network round trip is required before the next piece becomes playable, not for every movement. |
| One read channel | The room stream owns shared DOM state. Lock commands do not also patch the same stable IDs.[^cqrs-research] |
| Full recovery | Every emitted arena snapshot is self-sufficient. Reconnect resumes from the current revision; no event replay is required. |
| Bounded fan-out | One residual attack goes to one opponent. Eight players do not turn one clear into seven writes. |
| Bounded admission | Eight player seats are reserved separately from fifty spectator stream leases. No unbounded room creation or connection count. |
| Main hostname | The playable document and protocol remain under `/play/blocks/` on `rocci.dev`; this is a demonstrator-specific Caddy route, not generic path mounting. |
| Build, then serve | CI emits the musl server artifact. The VPS image does not contain Roc or Rust toolchains.[^publish-plan] |
| Inert knowledge | This record plans the app; executable Roc, Rocci, CSS, and JavaScript belong under the app directory, not `knowledge/`. |

## Recommended product shape

### One public room

The first release has one room and a repeating round lifecycle:

1. **Lobby:** two to eight players occupy seats. Spectators may watch. With at
   least two ready players, a ten-second countdown begins.
2. **Round:** the seated roster is frozen. New visitors spectate and may queue
   for the next round.
3. **Eliminated:** a topped-out player keeps the reserved player stream and
   watches the rest of the round; they do not consume a spectator slot.
4. **Result:** one survivor wins. A five-minute timeout ranks surviving players
   by lower stack height, then lines sent, solely to keep the public process
   bounded.
5. **Reset:** show results for ten seconds, promote queued visitors into open
   seats, then return to the lobby/countdown.

No account is required. An opaque, random, HttpOnly, SameSite=Lax session
cookie reclaims a seat for a ten-second disconnect grace period. A stale seat
is eliminated after the grace period; it is not controlled by a replacement
visitor.

### Eight-player targeting

Multiplayer falling-block games commonly put generated garbage into a receiving
queue, allow outgoing attack to cancel incoming attack, and choose a target
when more than two players are alive. Target rotation is a common many-player
distribution; broadcasting to all is a separate and much more explosive
mode.[^garbage-reference][^jstris-guide]

Use a **deterministic rotating target**:

- Seats have a stable ring order for the round.
- Each player has a target cursor. The target is the next living opponent from
  that cursor; self and eliminated seats are skipped.
- After a player sends a non-zero residual attack, their cursor advances to the
  following living seat. A clear fully consumed by cancellation does not
  advance it.
- If a target tops out before commit, the transaction selects the next living
  seat.
- With two players left, both necessarily target each other.
- The current target is visible before a lock. Manual target selection can be a
  measured follow-on, not v1.

This gives every attack one destination, distributes repeated attacks, is
deterministic in tests, and costs one queue append regardless of room size.
There is no “attackers” bonus: being targeted by several players is already a
disadvantage and should not also multiply outgoing power.

### Clears, attack, and garbage

Start with a small, legible attack table:

| Rows cleared by one lock | Attack lines |
| --- | ---: |
| 0 or 1 | 0 |
| 2 | 1 |
| 3 | 2 |
| 4 | 4 |

This matches the common base relationship without requiring spin detection or
the complete scoring variants found in competitive clients.[^garbage-reference]
Consecutive four-line clears receive a +1 back-to-back bonus. Combos, spins,
and perfect clears stay out until the basic queue is understandable in play.

Each accepted lock is one transaction:

1. Validate the submitted piece id, rotation, location, sequence number, and
   expected board revision against the authoritative locked board.
2. Merge the piece, clear completed rows, and calculate attack from the server's
   result.
3. Cancel the player's oldest pending garbage one-for-one with that attack.
4. Append any residual attack to the current target's queue, then advance the
   target cursor.
5. Apply up to eight ready, uncancelled garbage rows to the locking player.
6. Detect top-out, choose the next server-issued piece, update the room snapshot,
   and increment its revision.

An attack becomes ready after 600 ms and is applied only at the receiver's next
accepted lock. The receiver sees the queued amount and delay immediately.
Each attack packet uses one hole column for readable “clean” garbage; a later
packet chooses a different column when possible. Applying at most eight rows
per lock avoids an unreadable twenty-row burst. These values are v1 balance
constants, not language or platform contracts. Jstris likewise distinguishes
queued/cancellable garbage from instant insertion and documents a 500 ms
default delay, so this choice should feel familiar without copying its entire
ruleset.[^jstris-guide]

## Ownership and protocol

### Browser-owned transient state

`blocks-client.js` owns only:

- key/touch timing and repeat;
- position and animation of the current falling piece;
- ghost and interpolation;
- canvas drawing from the latest authoritative manifest;
- one monotonically increasing client command sequence.

When a piece locks, the island posts its proposed `{ piece, rotation, x, y,
board_revision, sequence }`. It may animate the result optimistically, but it
does not spawn the next playable piece until the server acknowledges. A 409
response contains the authoritative player snapshot and forces resynchronization.
This is the same exception class as Snake's fetch-based direction input, but
with much lower network frequency because motion between locks is private.[^snake-input]

The JSON response is for this explicit JavaScript island, not a Datastar signal
model. Datastar continues to own the room read stream and DOM morph boundary.

### Server-owned durable state

Pure `Game.roc` owns piece geometry, the seven-piece bag, collision, row clears,
attack calculation, target selection, garbage insertion, and top-out. The
custom `main.roc` owns SQLite, sessions, validation, transactions, HTTP, leases,
and SSE. Rocci components render the document, lobby, HUD, results, and a
compact stable-ID state manifest. This follows the existing Snake split, which
already demonstrates eight players, spectators, pure game rules, authored
routes, SQLite, and per-viewer SSE.[^snake-game][^snake-main][^snake-view]

Suggested HTTP surface:

```text
GET  /play/blocks/                 document or lobby
GET  /play/blocks/stream           long-lived Datastar SSE
POST /play/blocks/join             reserve/queue seat, then redirect
POST /play/blocks/leave            release seat, then redirect
POST /play/blocks/command/ready    lobby intent
POST /play/blocks/command/lock     JSON island intent + JSON acknowledgement
GET  /health/blocks                Caddy/origin health only
```

All mutating requests require the session cookie, same-origin checks, bounded
JSON bodies, and a per-session sequence. Lock commands are idempotent by
`(player_id, sequence)`: retry returns the stored acknowledgement instead of
placing twice.

### Compact authoritative manifest

The stream patches one stable `#blocks-arena-state` element. It contains room
revision, phase/timing, target and queue metadata, and at most eight boards.
Each locked 10x20 board uses a fixed one-character cell alphabet (200 bytes
before HTML escaping); do not emit 1,600 individual cell elements. The canvas
island observes the morphed attributes and paints them. Spectators see committed
boards at lock cadence, not every opponent's falling-piece animation.

Every event is a full arena snapshot. A slow client may skip revisions safely;
the next event replaces the manifest. The server coalesces changes to at most
five emitted snapshots per second per connection and sends an inert keepalive
at least every ten seconds while idle. That stays below basic-webserver's
30-second response-idle deadline without emitting the current generated
100-ms keepalive rate.[^bws-limits][^dispatch]

The custom app is justified because generated `@live` owns one `/sse` path and
poll/render policy, while this game needs a prefixed route, viewer leases,
idempotent JSON lock acknowledgements, coalescing, and a slower keepalive. No
language change follows from that.

## State and failure model

Use one SQLite database and WAL mode. Mutations are short transactions. Keep:

- `room`: phase, deadline, round, seed, revision, encoded current snapshot;
- `players`: session id, seat, status, board revision, locked board, piece bag,
  target cursor, sequence, statistics, disconnect deadline;
- `garbage`: receiver, sender, rows, hole, ready time, order;
- `commands`: player/sequence and bounded acknowledgement for idempotency;
- `viewer_leases`: random connection id, role, expiry.

An accepted command updates the normalized rows and the compact room snapshot
in the same transaction. Each stream poll first reads `{ revision, snapshot }`
rather than joining all game tables. SQLite remains the source of truth; the
snapshot is a derived recovery projection.

If the process restarts, the active round moves to `Result(Interrupted)` and a
fresh lobby starts. V1 does not try to reconstruct falling pieces that existed
only in browsers. The database volume is backed up with the existing origin
procedure, although no game history is promised.[^publish-plan]

## Capacity answer: eight players plus fifty spectators

**Recommendation:** design and reserve for **8 player streams + 50 spectator
streams**, but expose fifty only after a staging load gate. The existing
evidence proves the architecture shape, not this concurrency number. Generated
live streams poll independently and basic-webserver runs blocking handler
transitions, so connection count, SQLite reads, rendering, and egress must be
measured together.[^cqrs-research][^bws-limits]

The initial budget is intentionally simple:

| Quantity | Design ceiling |
| --- | ---: |
| Active room | 1 |
| Player seats/streams | 8 |
| Spectator stream leases | 50 |
| Total game streams | 58 |
| Stream poll / emit ceiling | 5 Hz per connection |
| Lock command ceiling | 10/s per player; expected materially lower |
| Full snapshot target | <= 4 KiB uncompressed |
| Idle keepalive | 1 per 10 s per connection |

At the deliberately pessimistic `4 KiB × 5 Hz × 58`, origin egress is about
1.16 MiB/s (roughly 9.5 Mbit/s) before protocol overhead. Actual traffic should
be lower because full emits occur on room revision changes, not on every poll.
This arithmetic makes fifty plausible on the selected VPS; it is an estimate,
not evidence of CPU, SQLite, Tunnel, browser, or reconnect behavior.

Spectator admission uses expiring **connection leases**, not a visitor count.
Each stream receives a random connection id, renews a 15-second lease during
polling, and consumes one of fifty slots. Disconnect cleanup may lag until TTL.
Player leases come from the reserved eight and cannot be displaced by spectators.
The document may still load when live capacity is full, but the stream returns
a stable “Arena viewing capacity reached” fragment and a retry delay.

The staging exit gate for fifty spectators is a 30-minute soak with eight
synthetic players committing three locks per second each and fifty streams:

- no SQLite busy failures, duplicate locks, corrupt revisions, or 5xx responses;
- p95 lock acknowledgement below 200 ms at the origin;
- p95 committed-state visibility below 500 ms;
- p95 snapshot at or below 4 KiB;
- game container below 60% of two CPUs sustained and below 768 MiB RSS;
- whole VPS below 70% CPU and 2 GiB RAM sustained, preserving site headroom;
- sustained game egress below 12 Mbit/s;
- idle streams survive 30 minutes; forced reconnect recovers in one snapshot.

If the gate fails, reduce the public spectator cap to 20 first, then lower the
emit ceiling to 4 Hz, then simplify the manifest. Do not increase the VPS or
invent delta/event replay before profiling identifies the limit.

## Hosting on the main site

The current origin has Caddy serving the static tree and proxying the generated
island process. The game becomes a second precompiled service and SQLite volume.
Caddy handles `/play/blocks/*` before the static fallback and proxies the path
unchanged to the game process; the app itself registers the prefix. Existing
`/actions/*` and `/sse` continue to reach the site-islands process.[^caddy][^compose]

This deliberately differs from ordinary cataloged live examples, which use
dedicated hostnames because generated standalone apps own `/`, `/actions/`, and
`/sse`.[^app-docs-plan] Rocci Blocks is a first-party site demonstrator with an
authored `main.roc`, not a precedent for transparently mounting arbitrary apps.
Its documentation may still appear under `/examples/blocks/`; the **Play** link
is explicitly `/play/blocks/`. If the examples generator cannot express that
URL when implementation reaches publication, add one optional explicit
`live_url` catalog field rather than deriving a special case from app id.[^apps-catalog]

Cloudflare and Caddy bypass caching for the document, stream, and commands.
Only fingerprinted `/play/blocks/assets/*` may be cached. Application admission
and sequence validation remain required even when Cloudflare rate limiting is
enabled; edge rules are defense in depth, not game correctness.

## Proposed application layout

```text
examples/rocci/custom/blocks/
  main.roc                  # HTTP, SQLite, sessions, leases, SSE
  Game.roc                  # pure board, bag, clear, attack, target rules
  Blocks.rocci              # document, lobby, HUD, manifest, results
  assets/blocks-client.js   # input, local falling piece, canvas renderer
  assets/blocks.css
  index.rocdown             # source-led demonstrator documentation
  README.md                 # short local run and smoke commands
  rocci.toml
```

Use `match` for room phases, player status, command results, pieces, rotations,
and validation errors. Use `if` only for boolean conditions such as “queue is
empty” or “lease has expired.” Components stay pure; effects remain in
`main.roc` helpers and handlers.

## Delivery phases

Writing this plan does not start a phase. Each phase should be one reviewable
change and preserve the previous deploy on failure.

Phase 0 is implemented: working name **Rocci Blocks**, the v1 constants, command
schema, error tags, and input/output tables live under
`examples/rocci/custom/blocks/fixtures/` (`protocol.md` plus JSON families for
placement, row clear, cancellation, target rotation, garbage holes, top-out,
duplicate sequence, reconnect, and snapshot budget).

Phase 1 is implemented: `Game.roc` owns board, bag, rotation, lock, and clears;
`Blocks.rocci` hosts the document, canvas, HUD, and solo lock/reset commands;
`assets/blocks-client.js` owns local motion. Rejected locks return the
authoritative player snapshot in JSON (`ok: 0`) so the island resynchronizes.

Phase 2 is implemented: custom `main.roc` owns SQLite seats, lobby/countdown/round/result,
idempotent lock acknowledgements, reconnect-grace columns, and `/play/blocks/` routes.
Duplicate POSTs replay the stored ack; restarting during a round writes `Result(Interrupted)`
then returns to lobby.

Phase 3 is implemented: `Game.roc` owns oldest-first cancellation, one residual write, rotating
targets, 600 ms delay, eight-row insertion, and spawn top-out. The lock transaction updates at
most one opponent queue. Fixture tests cover two, three, and eight living seats; eight synthetic
clients complete a local round.

Phase 4 is implemented: spectator leases are separate from the eight player seats, the public
default cap is **20** (`BLOCKS_SPECTATOR_CAP`, design ceiling 50) because the 30-minute 8+50 soak
was not run, streams coalesce at 5 Hz with a 10 s keepalive, and reconnect restores from one
full snapshot. Origin packaging remains later phases.

Phase 5 is implemented in-repo: catalog `live_url` for `/play/blocks/`, `index.rocdown`,
`docker/blocks/Dockerfile`, Compose profile `blocks` with a 512 MiB limit, and Caddy
`/play/blocks/*` before static fallback (island `/actions/*` and `/sse` unchanged).
The Access-gated staging Tunnel soak is an operator gate and was not run.

### Phase 0 — Freeze rules, wire budget, and fixtures

**Bound:** documentation and pure fixture data only. No server, Caddy, catalog,
or production changes.

**Work:**

1. Freeze the round lifecycle, four-row attack table, rotating target cursor,
   600-ms queue delay, eight-row insertion cap, board encoding, command schema,
   and error tags.
2. Write table fixtures covering placement, row clear, cancellation, target
   rotation with eliminated seats, garbage holes, top-out, duplicate sequence,
   and reconnect recovery.
3. Freeze the 4-KiB snapshot and five-Hz stream ceilings.

**Exit:** every semantic example in this record has an input/output fixture;
the user approves the game feel and public working name. No executable phase
starts implicitly.

### Phase 1 — Local single-player interaction boundary

**Bound:** `Game.roc`, `Blocks.rocci`, colocated CSS/JavaScript, and local
preview. No SQLite room, multiplayer attacks, or deployment.

**Work:** implement pure board/bag/rotation/lock/clear functions; author the
document and canvas host; keep movement local; commit each lock to an in-process
single-player handler. Add Rocci fixtures for lobby, active, queued-garbage,
eliminated, and result views.

**Exit:** keyboard and touch play are responsive; a rejected lock visibly
resynchronizes; `rocci inspect` succeeds for `Blocks.rocci`; the component
fixtures render without server I/O.

### Phase 2 — Authoritative room and command protocol

**Bound:** custom `main.roc` plus local SQLite. Two players, no spectators or
public origin.

**Work:** add sessions, seats, lobby/countdown/round/result transitions,
authoritative lock validation, idempotent command acknowledgements, reconnect
grace, and the compact room snapshot transaction. Reject oversized bodies,
cross-origin commands, stale board revisions, invalid geometry, impossible
piece ids, and command rates above the ceiling.

**Exit:** two browsers complete repeated rounds; duplicate/reordered POST tests
place no extra pieces; killing and restarting the server produces an
interrupted result then a clean lobby; SQLite integrity check succeeds.

### Phase 3 — Eight-player attack semantics

**Bound:** pure game rules and room transaction. Still local; no origin files.

**Work:** implement server-derived attack, oldest-first cancellation, delayed
garbage, eight-row insertion, rotating target cursor, top-out attribution, and
eight seats. Keep spins, combos, manual targeting, and attacker bonuses absent.

**Exit:** deterministic tests cover two, three, and eight living players;
simultaneous attacks serialize without lost rows; no rule operation writes more
than one opponent queue per lock; eight real/synthetic clients finish a round.

### Phase 4 — Spectator stream and admission control

**Bound:** authored Datastar SSE, stable manifest, canvas observer, viewer
leases, and local load harness. No Caddy or production hostname.

**Work:** add full revision snapshots, five-Hz coalescing, ten-second
keepalives, player/spectator lease pools, fifty-slot rejection fragment, and
reconnect. Build a harness that holds streams open and submits validated lock
fixtures; record payload, latency, SQLite, CPU, memory, and egress.

**Exit:** 8+50 passes the local 30-minute gate or the checked-in default cap is
reduced to the measured safe value. Idle streams do not hit the 30-second body
timeout. A dropped stream recovers from one full snapshot.

### Phase 5 — Package and same-origin staging route

**Bound:** app catalog/docs, musl package, one game container/volume, Caddy
prefix route, Cloudflare staging cache/rate rules. Production remains unchanged.

**Work:**

1. Add the app and its source documentation; keep the live URL explicit if the
   catalog requires the optional field described above.
2. Build with `rocci build --release --target x64musl`; keep Roc and Rust out
   of the runtime image.
3. Add `blocks` to the production Compose topology with resource limits and its
   own SQLite volume/healthcheck.
4. Proxy `/play/blocks/*` unchanged before static fallback. Do not disturb the
   site island `/actions/*` or `/sse` routes.
5. Deploy to Access-gated staging and repeat the 8+50 soak through Cloudflare
   Tunnel, including reconnect and rollback drills.

**Exit:** staging serves the game at the final path; ordinary site pages and the
home island still pass smoke tests; the runtime image has no toolchain; the
through-Tunnel soak satisfies the capacity gate at the configured cap.

### Phase 6 — Public launch and observation

**Bound:** production route, admission limits, operator dashboard/logs, and
rollback. No rule expansion.

**Work:** enable the production Caddy/Cloudflare route, start at the staging-
proven spectator cap, link the demonstrator from rocci.dev, document the
experimental status and controls, and watch command errors, stream leases,
reconnects, latency, SQLite busy counts, CPU, memory, and egress.

**Exit:** a 24-hour public observation stays within the staging budgets and the
previous release can be restored atomically. If it does not, roll back or lower
spectator admission; do not silently degrade authoritative validation.

## Validation matrix

| Boundary | Minimum validation |
| --- | --- |
| Pure rules | placement/rotation fixtures; clear/attack table; cancellation; target ring; garbage; top-out |
| Rocci authoring | AST inspect; fixtures for every room/player state; no effects in components |
| HTTP | join/leave redirects; session and origin checks; lock 200/409/429; idempotent retry; body limits |
| Stream | stable manifest id; first full snapshot; coalescing; keepalive; reconnect; capacity rejection |
| Persistence | transaction rollback, restart interruption, WAL/busy behavior, volume backup smoke |
| Packaging | x64musl server, assets, no Roc/Rust in image, healthcheck |
| Routing | `/play/blocks/*` reaches game; site `/actions/*`, `/sse`, static assets, and `/health` remain correct |
| Capacity | local and through-Tunnel 8+50 soak against the explicit latency/resource/egress gates |

After implementation edits, run the narrow app/package checks while iterating,
`cargo fmt --all -- --check` for Rust changes, relevant crate tests for any
packaging/catalog work, `rocdown check site`, the game HTTP/stream harness, and
the Rocci-profile knowledge check. A public capacity claim requires the staging
soak results; a successful local preview is not enough.

## Decision gates

Before Phase 1, a human should approve:

1. Public working name **Rocci Blocks** versus another non-trademarked name.
2. The deliberately small v1 rule set: no spins, combos, manual targets, or
   attacker bonuses.
3. Lock-driven remote/spectator boards rather than broadcasting falling-piece
   poses.

Before Phase 5, a human should approve:

4. The main-hostname `/play/blocks/` exception instead of the ordinary
   `<id>.examples.rocci.dev` live-example hostname.
5. One additional process and SQLite volume on the existing origin.
6. The measured spectator cap if staging cannot sustain fifty.

## Dependency order

```text
0 rules/protocol freeze
  -> 1 local interaction
  -> 2 authoritative room
  -> 3 attacks and eight seats
  -> 4 spectators and capacity evidence
  -> 5 same-origin staging package
  -> 6 public observation
```

[^server-state]: Durable application state stays server-owned; high-rate interaction may be an explicit browser-owned island that reconciles to server output.
[^cqrs-research]: Shared views require a long-lived GET; each generated stream polls independently and commands must not dual-patch the streamed region.
[^bws-limits]: Silent waits hit a 30-second response idle timeout; blocking stream transitions and SQLite contention require measurement.
[^publish-plan]: Existing planned origin is a 2-vCPU / 4-GB amd64 VPS running precompiled artifacts behind Cloudflare Tunnel, with atomic release and SQLite-volume operations.
[^app-docs-plan]: Ordinary cataloged live apps use dedicated hostnames because they own root action and stream paths.
[^snake-game]: Existing pure Roc game module caps multiplayer Snake at eight and separates rules from HTTP/rendering.
[^snake-main]: Existing authored ceiling uses SQLite, player cookies, revision polling, one SSE stream per viewer, and empty command responses.
[^snake-view]: Existing Rocci UI exposes eight active players plus a spectator path and stable patch regions.
[^snake-input]: Existing JavaScript island converts high-rate browser input into small intent POSTs without owning the authoritative world.
[^dispatch]: Generated `@live` polls at 100 ms and emits inert keepalives on unchanged renders; generated commands use the separate write response path.
[^caddy]: Current same-origin Caddy owns route precedence between live proxies and the static fallback.
[^compose]: Current hybrid production shape is a precompiled islands service plus Caddy and a named SQLite volume.
[^apps-catalog]: The current app catalog distinguishes docs and live hosting but has no explicit per-app live URL.
[^jstris-guide]: Target rotation, one-target distribution, cancellable queued garbage, and configurable garbage delay are established multiplayer choices; “to all” is explicitly more explosive.
[^garbage-reference]: Common base attacks are 0/1/2/4 for single/double/triple/four-row clears; outgoing attack can cancel queued incoming garbage; many-player games select targets.
