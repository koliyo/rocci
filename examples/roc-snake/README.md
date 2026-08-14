# Roc + Datastar multiplayer snake

A browser-only example: author the lobby and board in `Snake.rocci`, compile them to Roc, and serve HTML plus a long-lived Datastar SSE stream from [basic-webserver](https://github.com/roc-lang/basic-webserver) 0.16.0. Game state lives in SQLite. There is no tao/wry shell.

Pinned together:

- Roc nightly **2026-08-08** (the platform release was built against 2026-08-10)
- `basic-webserver` **0.16.0**
- Datastar **1.0.2** from `assets/datastar.js`

Open the start page, then spectate or join. Up to eight snakes share a **100×100** discrete grid. Walls kill; fruit grows you immediately; living snakes also gain a segment about every two seconds. The server ticks at about 8 Hz; each SSE client parks on `After(125)` and morphs `#board`, `#hud`, and `#minimap`. WASD or arrow keys steer. Death respawns after about two seconds.

If you already ran an older build, delete `examples/roc-snake/snake.db` (and `-wal`/`-shm`) so the `tick` column and bounded world can be created fresh.

## Run

From the repository root, with `roc` and `cargo` on `PATH`:

```sh
./scripts/run-roc-snake.sh
```

Then open [http://127.0.0.1:8000](http://127.0.0.1:8000). Override the port with `ROC_BASIC_WEBSERVER_PORT`. SQLite state lives in `examples/roc-snake/snake.db` (created on first start). Set `DB_PATH` to use another file.

The script copies shared assets into `examples/roc-snake/assets/` and runs `rocci run`, which compiles `Snake.rocci` to a Roc type module (`Snake.roc`, gitignored) and executes `main.roc`. If assets are already in place:

```sh
cargo run -q -p rocci-cli -- run examples/roc-snake/main.roc
```

`Game.roc` owns ticks, collisions, food, growth, and the viewport. `main.roc` owns HTTP, cookies, SQLite, and SSE.

## Smoke checks

With the server running:

```sh
curl -s http://127.0.0.1:8000/health
# ok

curl -s http://127.0.0.1:8000/ | grep -E 'Join game|Spectate'

curl -s -D - -o /dev/null -X POST http://127.0.0.1:8000/api/join
# HTTP/1.1 303
# set-cookie: snake=...
# location: /play

curl -s http://127.0.0.1:8000/play | grep -E 'id="board"|id="hud"|id="minimap"|datastar.js'
```

Open two browser windows: join in one, spectate in the other. Both boards should tick together. Fruit should show on the board and as gold dots on the minimap.
