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
}
PlayerUpdate : {
    id : Str,
    dir : Str,
    pending_dir : Str,
    alive : I64,
    respawn_in : I64,
    body : Str,
    score : I64,
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
            query: "CREATE TABLE IF NOT EXISTS players (id TEXT PRIMARY KEY, name TEXT NOT NULL, color TEXT NOT NULL, dir TEXT NOT NULL, pending_dir TEXT NOT NULL, alive INTEGER NOT NULL, respawn_in INTEGER NOT NULL, body TEXT NOT NULL, score INTEGER NOT NULL)",
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
        .with_file_roots([assets])
        .with_native_routes({
            files: [
                Server.static_mount({ at: "/assets", files: assets }),
            ],
            liveness: [],
            readiness: [],
        })

    Ok({ config, context: { db } })
}

respond! : Server.Request, Context => Try(Server.Outcome, [ServerErr(Str), ..])
respond! = |request, { db }| {
    path =
        match request.target() {
            Resource({ raw_path, .. }) => raw_path
            _ => ""
        }
    query =
        match request.target() {
            Resource({ raw_query, .. }) => raw_query
            _ => Absent
        }
    player_id = cookie_player(request.headers())

    match (Method.to_str(request.method()), path) {
        ("GET", "/") => {
            count = player_count!(db) ? |err| ServerErr("Failed to read players: ${Str.inspect(err)}")
            html_ok(Html.render(Snake.lobbyPage({ player_count_str: count.to_str(), full: count >= Game.max_players })))
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
        ("POST", "/api/join") =>
            join_player!(db)
        ("POST", "/api/leave") =>
            leave_player!(db, player_id)
        ("POST", "/api/dir") =>
            set_dir!(db, player_id, dir_from_query(query))
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
    rows : List({ id : Str, name : Str, color : Str, dir : Str, pending_dir : Str, alive : I64, respawn_in : I64, body : Str, score : I64 })
    rows = Sqlite.query_many!(
        {
            db,
            query: "SELECT id, name, color, dir, pending_dir, alive, respawn_in, body, score FROM players",
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
    rows : List({ id : Str, name : Str, color : Str, dir : Str, pending_dir : Str, alive : I64, respawn_in : I64, body : Str, score : I64 })
    rows = Sqlite.Transaction.query_many!(
        tx,
        {
            query: "SELECT id, name, color, dir, pending_dir, alive, respawn_in, body, score FROM players",
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
            }
            Sqlite.Transaction.execute!(
                tx,
                {
                    query: "UPDATE players SET dir = :dir, pending_dir = :pending_dir, alive = :alive, respawn_in = :respawn_in, body = :body, score = :score WHERE id = :id",
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
    }
    Sqlite.execute!(
        {
            db,
            query: "INSERT INTO players (id, name, color, dir, pending_dir, alive, respawn_in, body, score) VALUES (:id, :name, :color, :dir, :pending_dir, :alive, :respawn_in, :body, :score)",
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
    respawn_str =
        match local {
            Some(snake) if !snake.alive => "${(snake.respawn_in + 7.I64).div_trunc_by(8.I64).to_str()}s"
            _ => ""
        }
    {
        role,
        cam: "${cam.x.to_str()}, ${cam.y.to_str()}",
        respawning,
        respawn_str,
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
                    score_str: snake.score.to_str(),
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

dir_from_query = |raw_query|
    match raw_query {
        Absent => None
        Present(q) =>
            List.fold(
                Str.split_on(q, "&"),
                None,
                |acc, part|
                    match acc {
                        Some(_) => acc
                        None if part == "d=up" => Some("up")
                        None if part == "d=down" => Some("down")
                        None if part == "d=left" => Some("left")
                        None if part == "d=right" => Some("right")
                        None => acc
                    },
            )
    }
