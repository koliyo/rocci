app [Context, program] {
    pf: platform "https://github.com/roc-lang/basic-webserver/releases/download/0.16.0/42jC1JT3auhHSmv2Ah8mW5F2MXiAakq1UQQ4NQceQjXw.tar.zst",
    http: "https://github.com/roc-lang/http/releases/download/1.0.0/6ZUwqYhCS8PU9Mo6MF7oV82ET2o7KYb57CLKDq4cq4sS.tar.zst",
}

import pf.Env
import pf.Path
import pf.Server
import pf.Sqlite
import pf.Sse
import pf.UnixTime
import http.Method
import http.Response
import Datastar
import Game
import Html
import Snake

Context : { db : Sqlite.Db }
IdParams : { id : Str }
DirParams : { id : Str, pending_dir : Str }
SeedParams : { seed : I64 }
TickParams : { now : I64, seed : I64, tick : I64 }
FoodParams : { id : I64, x : I64, y : I64 }
PlayerInsert : {
    id : Str,
    name : Str,
    color : Str,
    dir : Str,
    pending_dir : Str,
    alive : I64,
    respawn_in : I64,
    body : Str,
    score : I64,
    cam_x : I64,
    cam_y : I64,
}
PlayerUpdate : {
    id : Str,
    dir : Str,
    pending_dir : Str,
    alive : I64,
    respawn_in : I64,
    body : Str,
    score : I64,
    cam_x : I64,
    cam_y : I64,
}

program = { init!, respond!, shutdown! }

init! : () => Try({ config : Server.Config, context : Context }, [Exit(I64), ..])
init! = || {
    db_path =
        match Env.var!("DB_PATH") {
            Ok(path) => Path.from_os_str(path)
            Err(_) => Path.utf8("./snake.db")
        }
    db = Sqlite.open!(Sqlite.default_config(db_path)) ? |_| Exit(2)
    Sqlite.execute!(
        {
            db,
            query: "CREATE TABLE IF NOT EXISTS game (id INTEGER PRIMARY KEY CHECK (id = 1), revision INTEGER NOT NULL, last_tick_ms INTEGER NOT NULL, seed INTEGER NOT NULL, tick INTEGER NOT NULL)",
            params: {},
        },
    )
        ? |_| Exit(2)
    Sqlite.execute!(
        {
            db,
            query: "INSERT OR IGNORE INTO game (id, revision, last_tick_ms, seed, tick) VALUES (1, 0, 0, 1, 0)",
            params: {},
        },
    )
        ? |_| Exit(2)
    Sqlite.execute!(
        {
            db,
            query: "CREATE TABLE IF NOT EXISTS players (id TEXT PRIMARY KEY, name TEXT NOT NULL, color TEXT NOT NULL, dir TEXT NOT NULL, pending_dir TEXT NOT NULL, alive INTEGER NOT NULL, respawn_in INTEGER NOT NULL, body TEXT NOT NULL, score INTEGER NOT NULL, cam_x INTEGER NOT NULL, cam_y INTEGER NOT NULL)",
            params: {},
        },
    )
        ? |_| Exit(2)
    Sqlite.execute!(
        {
            db,
            query: "CREATE TABLE IF NOT EXISTS food (id INTEGER PRIMARY KEY, x INTEGER NOT NULL, y INTEGER NOT NULL)",
            params: {},
        },
    )
        ? |_| Exit(2)

    assets = Server.file_root({
        id: "assets",
        path: Path.utf8("assets"),
    })
    config =
        Server.default_config
        .with_listen({ host: "127.0.0.1", port: listen_port!({}) })
        .with_file_roots([assets])
        .with_native_routes({
            files: [
                Server.static_mount({ at: "/assets", files: assets }),
            ],
            liveness: [],
            readiness: [],
        })

    Ok({ config, context: { db: db } })
}

respond! : Server.Request, Context => Try(Server.Outcome, [ServerErr(Str), ..])
respond! = |request, { db }| {
    path =
        match request.target() {
            Resource({ raw_path, .. }) => raw_path
            _ => ""
        }
    player_id = cookie_player(request.headers())

    match (Method.to_str(request.method()), path) {
        ("GET", "/") => {
            count = player_count!(db) ? |err| ServerErr("Failed to read players: ${Str.inspect(err)}")
            html_ok(Html.render(Snake.lobbyPage({ player_count: count, full: count >= Game.max_players })))
        }
        ("GET", "/play") => {
            view = load_view!(db, player_id) ? |err| ServerErr("Failed to load game: ${Str.inspect(err)}")
            html_ok(Html.render(Snake.playPage({ cells: view.cells, info: view.hud, marks: view.marks })))
        }
        ("GET", "/health") =>
            Ok(
                Server.respond(
                    Response.from_status(200)
                    .with_headers([{ name: "Content-Type", value: "text/plain; charset=utf-8" }])
                    .with_body(Str.to_utf8("ok")),
                ),
            )
        ("GET", "/sse") =>
            stream_game!(db, player_id)
        ("POST", "/join") =>
            join_player!(db)
        ("POST", "/leave") =>
            leave_player!(db, player_id)
        ("POST", "/api/direction") => {
            json = read_json_body!(request) ? |err| ServerErr("Failed to read direction: ${Str.inspect(err)}")
            set_dir!(db, player_id, dir_from_json(json))
        }
        _ =>
            Ok(
                Server.respond(
                    Response.from_status(404)
                    .with_body(Str.to_utf8("Not found")),
                ),
            )
    }
}

shutdown! : Server.ShutdownReason, Context => Try({}, [Exit(I64), ..])
shutdown! = |_reason, _context| Ok({})

html_ok = |body|
    Ok(
        Server.respond(
            Response.from_status(200)
            .with_headers([{ name: "Content-Type", value: "text/html; charset=utf-8" }])
            .with_body(Str.to_utf8(body)),
        ),
    )

redirect = |status, location, cookie|
    Ok(
        Server.respond(
            Response.from_status(status)
            .with_headers(
                List.concat(
                    [{ name: "Location", value: location }],
                    cookie,
                ),
            )
            .with_body([]),
        ),
    )

empty_sse! = ||
    Ok(Server.stream(Sse.unfold!(0, |_state| Ok(End))))

stream_game! = |db, player_id|
    Ok(
        Server.stream(
            Sse.unfold!(
                { revision: 0.I64, first: True, player_id },
                |state| {
                    view = load_view!(db, state.player_id)?
                    if !state.first and view.revision == state.revision {
                        Ok(Wait({ state, wake: After(125) }))
                    } else {
                        event = Datastar.patch_elements(Snake.gamePatch({ cells: view.cells, info: view.hud, marks: view.marks }))
                        Ok(
                            Emit(
                                {
                                    event,
                                    state: { revision: view.revision, first: False, player_id: state.player_id },
                                    wake: After(125),
                                },
                            ),
                        )
                    }
                },
            ),
        ),
    )

join_player! = |db| {
    world = load_world!(db) ? |err| ServerErr("Failed to load game: ${Str.inspect(err)}")
    if List.len(world.snakes).to_i64_wrap() >= Game.max_players {
        redirect(303, "/play", [])
    } else {
        used = List.map(world.snakes, |snake| snake.color)
        color = Game.next_color(used)
        if color == "" {
            redirect(303, "/play", [])
        } else {
            count = List.len(world.snakes).to_i64_wrap() + 1.I64
            id = new_player_id!(count)
            spawned = Game.spawn_player(world, id, Game.color_label(color), color)
            match List.last(spawned.snakes) {
                Err(_) => redirect(303, "/", [])
                Ok(snake) => {
                    insert_player!(db, snake) ? |err| ServerErr("Failed to join: ${Str.inspect(err)}")
                    bump_game!(db, spawned.seed) ? |err| ServerErr("Failed to join: ${Str.inspect(err)}")
                    redirect(
                        303,
                        "/play",
                        [{ name: "Set-Cookie", value: "snake=${id}; Path=/; HttpOnly; SameSite=Lax" }],
                    )
                }
            }
        }
    }
}

leave_player! = |db, player_id| {
    if player_id != "" {
        params : IdParams
        params = { id: player_id }
        Sqlite.execute!(
            {
                db,
                query: "DELETE FROM players WHERE id = :id",
                params,
            },
        )
            ? |err| ServerErr("Failed to leave: ${Str.inspect(err)}")
        bump_revision!(db) ? |err| ServerErr("Failed to leave: ${Str.inspect(err)}")
    } else {
        {}
    }
    redirect(303, "/", [{ name: "Set-Cookie", value: "snake=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0" }])
}

set_dir! = |db, player_id, dir| {
    match dir {
        Some(next) if player_id != "" and Game.valid_dir(next) => {
            params : DirParams
            params = { id: player_id, pending_dir: next }
            Sqlite.execute!(
                {
                    db,
                    query: "UPDATE players SET pending_dir = :pending_dir WHERE id = :id AND alive = 1",
                    params,
                },
            )
                ? |err| ServerErr("Failed to steer: ${Str.inspect(err)}")
            empty_sse!()
        }
        _ => empty_sse!()
    }
}

load_view! = |db, player_id| {
    maybe_tick!(db)?
    world = load_world!(db)?
    revision = read_revision!(db)?
    cam = Game.camera(world, player_id)
    Ok({
        revision,
        cells: Game.cells(world, cam),
        hud: build_hud(world, player_id, cam),
        marks: Game.minimap(world, cam),
    })
}

maybe_tick! = |db| {
    now = now_ms!()
    tx = Sqlite.begin!(db, Immediate)?
    row : { last_tick_ms : I64, seed : I64, tick : I64 }
    row = Sqlite.Transaction.query!(
        tx,
        {
            query: "SELECT last_tick_ms, seed, tick FROM game WHERE id = 1",
            params: {},
            limits: Sqlite.default_query_limits,
        },
    )?
    if now - row.last_tick_ms >= Game.tick_ms {
        snakes = load_snakes_tx!(tx)?
        food = load_food_tx!(tx)?
        stepped = Game.step({ snakes, food, seed: row.seed, tick: row.tick })
        save_snakes_tx!(tx, stepped.snakes)?
        save_food_tx!(tx, stepped.food)?
        tick_params : TickParams
        tick_params = { now: now, seed: stepped.seed, tick: stepped.tick }
        Sqlite.Transaction.execute!(
            tx,
            {
                query: "UPDATE game SET last_tick_ms = :now, revision = revision + 1, seed = :seed, tick = :tick WHERE id = 1",
                params: tick_params,
            },
        )?
        Sqlite.Transaction.commit!(tx)
    } else {
        Sqlite.Transaction.commit!(tx)
    }
}

load_world! = |db| {
    snakes = load_snakes_db!(db)?
    food = load_food_db!(db)?
    row : { seed : I64, tick : I64 }
    row = Sqlite.query!(
        {
            db,
            query: "SELECT seed, tick FROM game WHERE id = 1",
            params: {},
            limits: Sqlite.default_query_limits,
        },
    )?
    Ok({ snakes, food, seed: row.seed, tick: row.tick })
}

load_snakes_db! = |db| {
    rows : List({ id : Str, name : Str, color : Str, dir : Str, pending_dir : Str, alive : I64, respawn_in : I64, body : Str, score : I64, cam_x : I64, cam_y : I64 })
    rows = Sqlite.query_many!(
        {
            db,
            query: "SELECT id, name, color, dir, pending_dir, alive, respawn_in, body, score, cam_x, cam_y FROM players",
            params: {},
            limits: Sqlite.default_query_limits,
        },
    )?
    Ok(List.map(rows, Game.from_row))
}

load_food_db! = |db| {
    rows : List({ x : I64, y : I64 })
    rows = Sqlite.query_many!(
        {
            db,
            query: "SELECT x, y FROM food",
            params: {},
            limits: Sqlite.default_query_limits,
        },
    )?
    Ok(rows)
}

load_snakes_tx! = |tx| {
    rows : List({ id : Str, name : Str, color : Str, dir : Str, pending_dir : Str, alive : I64, respawn_in : I64, body : Str, score : I64, cam_x : I64, cam_y : I64 })
    rows = Sqlite.Transaction.query_many!(
        tx,
        {
            query: "SELECT id, name, color, dir, pending_dir, alive, respawn_in, body, score, cam_x, cam_y FROM players",
            params: {},
            limits: Sqlite.default_query_limits,
        },
    )?
    Ok(List.map(rows, Game.from_row))
}

load_food_tx! = |tx| {
    rows : List({ x : I64, y : I64 })
    rows = Sqlite.Transaction.query_many!(
        tx,
        {
            query: "SELECT x, y FROM food",
            params: {},
            limits: Sqlite.default_query_limits,
        },
    )?
    Ok(rows)
}

save_snakes_tx! = |tx, snakes|
    save_snake_help!(tx, snakes)

save_snake_help! = |tx, snakes|
    match snakes {
        [] => Ok({})
        [snake, .. as rest] => {
            alive = if snake.alive { 1.I64 } else { 0.I64 }
            params : PlayerUpdate
            params = {
                id: snake.id,
                dir: snake.dir,
                pending_dir: snake.pending_dir,
                alive,
                respawn_in: snake.respawn_in,
                body: Game.encode_body(snake.body),
                score: snake.score,
                cam_x: snake.cam.x,
                cam_y: snake.cam.y,
            }
            Sqlite.Transaction.execute!(
                tx,
                {
                    query: "UPDATE players SET dir = :dir, pending_dir = :pending_dir, alive = :alive, respawn_in = :respawn_in, body = :body, score = :score, cam_x = :cam_x, cam_y = :cam_y WHERE id = :id",
                    params,
                },
            )?
            save_snake_help!(tx, rest)
        }
    }

save_food_tx! = |tx, food| {
    Sqlite.Transaction.execute!(tx, { query: "DELETE FROM food", params: {} })?
    insert_food_help!(tx, food, 1.I64)
}

insert_food_help! = |tx, food, id|
    match food {
        [] => Ok({})
        [point, .. as rest] => {
            food_params : FoodParams
            food_params = { id, x: point.x, y: point.y }
            Sqlite.Transaction.execute!(
                tx,
                {
                    query: "INSERT INTO food (id, x, y) VALUES (:id, :x, :y)",
                    params: food_params,
                },
            )?
            insert_food_help!(tx, rest, id + 1.I64)
        }
    }

insert_player! = |db, snake| {
    alive = if snake.alive { 1.I64 } else { 0.I64 }
    params : PlayerInsert
    params = {
        id: snake.id,
        name: snake.name,
        color: snake.color,
        dir: snake.dir,
        pending_dir: snake.pending_dir,
        alive,
        respawn_in: snake.respawn_in,
        body: Game.encode_body(snake.body),
        score: snake.score,
        cam_x: snake.cam.x,
        cam_y: snake.cam.y,
    }
    Sqlite.execute!(
        {
            db,
            query: "INSERT INTO players (id, name, color, dir, pending_dir, alive, respawn_in, body, score, cam_x, cam_y) VALUES (:id, :name, :color, :dir, :pending_dir, :alive, :respawn_in, :body, :score, :cam_x, :cam_y)",
            params,
        },
    )
}

bump_game! = |db, next_seed| {
    params : SeedParams
    params = { seed: next_seed }
    Sqlite.execute!(
        {
            db,
            query: "UPDATE game SET revision = revision + 1, seed = :seed WHERE id = 1",
            params,
        },
    )
}

bump_revision! = |db|
    Sqlite.execute!(
        {
            db,
            query: "UPDATE game SET revision = revision + 1 WHERE id = 1",
            params: {},
        },
    )

player_count! = |db| {
    row : { n : I64 }
    row = Sqlite.query!(
        {
            db,
            query: "SELECT COUNT(*) AS n FROM players",
            params: {},
            limits: Sqlite.default_query_limits,
        },
    )?
    Ok(row.n)
}

read_revision! = |db| {
    row : { revision : I64 }
    row = Sqlite.query!(
        {
            db,
            query: "SELECT revision FROM game WHERE id = 1",
            params: {},
            limits: Sqlite.default_query_limits,
        },
    )?
    Ok(row.revision)
}

build_hud = |world, player_id, cam| {
    local = Game.player_snake(world.snakes, player_id)
    role =
        match local {
            Some(snake) => "You are ${Game.color_label(snake.color)}"
            None if player_id == "" => "Spectating"
            None => "Spectating"
        }
    respawning =
        match local {
            Some(snake) if !snake.alive => True
            _ => False
        }
    respawn_secs =
        match local {
            Some(snake) if !snake.alive => (snake.respawn_in + 7.I64).div_trunc_by(8.I64)
            _ => 0
        }
    {
        role,
        cam,
        respawning,
        respawn_secs,
        can_leave: player_id != "",
        players: List.map(
            world.snakes,
            |snake| {
                you = player_id != "" and snake.id == player_id
                row_class =
                    if you and !snake.alive {
                        "you dead"
                    } else if you {
                        "you"
                    } else if !snake.alive {
                        "dead"
                    } else {
                        ""
                    }
                {
                    row_class,
                    swatch_class: "swatch ${snake.color}",
                    label: if you {
                        "${Game.color_label(snake.color)} (you)"
                    } else {
                        Game.color_label(snake.color)
                    },
                    score: snake.score,
                }
            },
        ),
    }
}

now_ms! = || {
    ts = UnixTime.now!()
    secs = UnixTime.Timestamp.seconds_since_epoch(ts)
    nanos = UnixTime.Timestamp.subsecond_nanoseconds(ts)
    secs * 1000.I64 + nanos.to_i64().div_trunc_by(1_000_000.I64)
}

new_player_id! = |count| {
    ts = UnixTime.now!()
    secs = UnixTime.Timestamp.seconds_since_epoch(ts)
    "${secs.to_str()}-${count.to_str()}"
}

cookie_player = |headers|
    List.fold(
        headers,
        "",
        |acc, header|
            if acc != "" {
                acc
            } else if header.name == "cookie" or header.name == "Cookie" {
                cookie_value(header.value, "snake")
            } else {
                acc
            },
    )

cookie_value = |header_value, key|
    List.fold(
        Str.split_on(header_value, ";"),
        "",
        |acc, part| {
            trimmed = Str.trim(part)
            if acc != "" {
                acc
            } else if Str.starts_with(trimmed, "${key}=") {
                Str.drop_prefix(trimmed, "${key}=")
            } else {
                acc
            }
        },
    )

dir_from_json = |json| {
    value = json_str(json, "direction")
    if Game.valid_dir(value) {
        Some(value)
    } else {
        None
    }
}

read_json_body! = |request|
    request.body().with_limit(4 * 1024).read_all!().map_ok(Str.from_utf8_lossy)

json_str = |json, key| {
    needle = "\"${key}\":"
    parts = Str.split_on(json, needle)
    match List.get(parts, 1) {
        Ok(after) => json_string_value(skip_ws(after))
        Err(_) => ""
    }
}

skip_ws = |text| skip_ws_bytes(Str.to_utf8(text))

skip_ws_bytes = |bytes|
    match List.get(bytes, 0) {
        Ok(byte) if byte == 32 or byte == 9 or byte == 10 or byte == 13 =>
            skip_ws_bytes(List.drop_first(bytes, 1))
        _ => Str.from_utf8_lossy(bytes)
    }

json_string_value = |text|
    if Str.starts_with(text, "\"") {
        read_json_string(Str.to_utf8(Str.drop_prefix(text, "\"")), 0, [], False)
    } else {
        ""
    }

read_json_string = |bytes, index, acc, escape|
    match List.get(bytes, index) {
        Err(_) => Str.from_utf8_lossy(acc)
        Ok(34) if !escape => Str.from_utf8_lossy(acc)
        Ok(92) if !escape => read_json_string(bytes, index + 1, acc, True)
        Ok(byte) if escape =>
            read_json_string(
                bytes,
                index + 1,
                List.append(
                    acc,
                    match byte {
                        34 => 34
                        92 => 92
                        110 => 10
                        116 => 9
                        114 => 13
                        _ => byte
                    },
                ),
                False,
            )
        Ok(byte) => read_json_string(bytes, index + 1, List.append(acc, byte), False)
    }

listen_port! : {} => U16
listen_port! = |_| {
    match Env.var_str!("ROC_BASIC_WEBSERVER_PORT") {
        Ok(value) =>
            match U16.from_str(value) {
                Ok(0) => 8000
                Ok(port) => port
                Err(_) => 8000
            }
        Err(_) => 8000
    }
}
