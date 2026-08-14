# Roc + Datastar multiplayer snake

A **Datastar/Rocci multiplayer stress demo**: author the lobby and board in `Snake.rocci`, compile them to Roc, and serve HTML plus a long-lived Datastar SSE stream from [basic-webserver](https://github.com/roc-lang/basic-webserver) 0.16.0. Game state lives in SQLite. `rocci run` opens the server in an embedded window; use `--no-window` to serve only.

This is not a prescription for browser games. It shows one authoritative world, cookies, join/leave, and SSE patches to stable `#board`, `#hud`, and `#minimap` boundaries. Full viewport morphing at 8 Hz is intentionally demonstrative. Keyboard and touch steering live in a tiny `snake-input.js` island; they are not Datastar attribute programs.

Pinned together:

- Roc nightly **2026-08-08** (the platform release was built against 2026-08-10)
- `basic-webserver` **0.16.0**
- Datastar **1.0.2** from `assets/datastar.js`

Open the start page, then spectate or join. Up to eight snakes share a **100×100** discrete grid. Walls kill; fruit grows you immediately; living snakes also gain a segment about every two seconds. The server ticks at about 8 Hz; each SSE client parks on `After(125)` and morphs the board, HUD, and minimap. WASD, arrow keys, or the on-screen pad steer. Death respawns after about two seconds.

If you already ran an older build, delete `examples/roc-snake/snake.db` (and `-wal`/`-shm`) so the `tick` column and bounded world can be created fresh.

## Run

From the repository root, with `roc` and `cargo` on `PATH`:

```sh
./scripts/run-roc-snake.sh
```

This opens an embedded window at [http://127.0.0.1:8000](http://127.0.0.1:8000). Pass `--no-window` to serve only (then open that URL yourself, or curl it). Override the port with `ROC_BASIC_WEBSERVER_PORT`. SQLite state lives in `examples/roc-snake/snake.db` (created on first start). Set `DB_PATH` to use another file.

The script copies `datastar.js` into `examples/roc-snake/assets/` and runs `rocci run`, which compiles `Snake.rocci` to a Roc type module (`Snake.roc`, gitignored) and executes `main.roc`. `snake-input.js` is already in that assets folder. If assets are already in place:

```sh
cargo run -q -p rocci-cli -- run examples/roc-snake/main.roc
```

`Game.roc` owns ticks, collisions, food, growth, and the viewport. `main.roc` owns HTTP, cookies, SQLite, and SSE. `snake-input.js` sends `{direction, sequence}` to `POST /api/direction`.

## Smoke checks

With the server running (`--no-window` if you do not want an embedded window):

```sh
curl -s http://127.0.0.1:8000/health
# ok

curl -s http://127.0.0.1:8000/ | grep -E 'Join game|Spectate'

curl -s -D - -o /dev/null -X POST http://127.0.0.1:8000/api/join
# HTTP/1.1 303
# set-cookie: snake=...
# location: /play

curl -s http://127.0.0.1:8000/play | grep -E 'id="board"|id="hud"|id="minimap"|datastar.js|snake-input.js'

curl -s -X POST http://127.0.0.1:8000/api/direction \
  -H 'Content-Type: application/json' \
  -d '{"direction":"up","sequence":1}'
```

Open two browser windows: join in one, spectate in the other. Both boards should tick together. Fruit should show on the board and as gold dots on the minimap. The joined window should show the on-screen pad; the spectator should not.
