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
import Blocks
import Datastar
import Game
import Html

Context : { db : Sqlite.Db }
RoomRow : { phase : Str, deadline_ms : I64, round : I64, seed : I64, revision : I64, reason : Str }
PlayerRow : {
    id : Str,
    seat : I64,
    status : Str,
    board : Str,
    piece : Str,
    bag : Str,
    seed : I64,
    board_revision : I64,
    sequence : I64,
    back_to_back : I64,
    last_lock_ms : I64,
    locks_window : I64,
    disconnect_deadline : I64,
    garbage : Str,
    cursor_seat : I64,
    last_hole : I64,
    lines_sent : I64,
}
IdParams : { id : Str }
SeatParams : {
    id : Str,
    seat : I64,
    status : Str,
    board : Str,
    piece : Str,
    bag : Str,
    seed : I64,
    garbage : Str,
    cursor_seat : I64,
    last_hole : I64,
    lines_sent : I64,
}
ReadyParams : { id : Str, status : Str }
PhaseParams : { phase : Str, deadline_ms : I64, round : I64, seed : I64, reason : Str }
LockSave : {
    id : Str,
    board : Str,
    piece : Str,
    bag : Str,
    seed : I64,
    board_revision : I64,
    sequence : I64,
    back_to_back : I64,
    status : Str,
    last_lock_ms : I64,
    locks_window : I64,
    garbage : Str,
    cursor_seat : I64,
    lines_sent : I64,
}
TargetSave : { id : Str, garbage : Str, last_hole : I64 }
AckRow : { ok : I64, error : Str, board : Str, revision : I64, piece : Str, sequence : I64, eliminated : I64 }
AckInsert : { player_id : Str, sequence : I64, ok : I64, error : Str, board : Str, revision : I64, piece : Str, eliminated : I64 }
RateParams : { id : Str, last_lock_ms : I64, locks_window : I64 }
DeadParams : { id : Str, disconnect_deadline : I64 }

program = { init!, respond!, shutdown! }

prefix = "/play/blocks"

init! : () => Try({ config : Server.Config, context : Context }, [Exit(I64), ..])
init! = || {
    db_path =
        match Env.var!("DB_PATH") {
            Ok(path) => Path.from_os_str(path)
            Err(_) => Path.utf8("./blocks.db")
        }
    db = Sqlite.open!(Sqlite.default_config(db_path)) ? |_| Exit(2)
    setup_db!(db) ? |_| Exit(2)
    interrupt_if_active!(db) ? |_| Exit(2)

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
                Server.static_mount({ at: "${prefix}/assets", files: assets }),
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
    player_id = cookie_value_in(request.headers(), "blocks")
    method = Method.to_str(request.method())
    tick_room!(db) ? |err| ServerErr("Failed to tick room: ${Str.inspect(err)}")

    match (method, path) {
        ("GET", "/play/blocks") => document!(db, player_id)
        ("GET", "/play/blocks/") => document!(db, player_id)
        ("GET", "/health") => text_ok("ok")
        ("GET", "/health/blocks") => text_ok("ok")
        ("GET", "/play/blocks/stream") => stream_room!(db, player_id)
        ("POST", "/play/blocks/join") =>
            if origin_ok(request.headers()) {
                join_player!(db, player_id)
            } else {
                json_status(403, ack_json(empty_ack("InvalidOrigin")))
            }
        ("POST", "/play/blocks/leave") =>
            if origin_ok(request.headers()) {
                leave_player!(db, player_id)
            } else {
                json_status(403, ack_json(empty_ack("InvalidOrigin")))
            }
        ("POST", "/play/blocks/command/ready") =>
            if origin_ok(request.headers()) {
                ready_player!(db, player_id)
            } else {
                json_status(403, ack_json(empty_ack("InvalidOrigin")))
            }
        ("POST", "/play/blocks/command/lock") =>
            match lock_request!(db, request, player_id) {
                Ok(out) => Ok(out)
                Err(err) => Err(ServerErr("Failed to lock: ${Str.inspect(err)}"))
            }
        _ =>
            Ok(
                Server.respond(
                    Response.from_status(404).with_body(Str.to_utf8("Not found")),
                ),
            )
    }
}

shutdown! : Server.ShutdownReason, Context => Try({}, [Exit(I64), ..])
shutdown! = |_reason, _context| Ok({})

try_column! = |db, query|
    match Sqlite.execute!({ db, query, params: {} }) {
        Ok({}) => Ok({})
        Err(_) => Ok({})
    }

setup_db! = |db| {
    Sqlite.execute!(
        {
            db,
            query: "CREATE TABLE IF NOT EXISTS room (id INTEGER PRIMARY KEY CHECK (id = 1), phase TEXT NOT NULL, deadline_ms INTEGER NOT NULL, round INTEGER NOT NULL, seed INTEGER NOT NULL, revision INTEGER NOT NULL, reason TEXT NOT NULL)",
            params: {},
        },
    )?
    Sqlite.execute!(
        {
            db,
            query: "INSERT OR IGNORE INTO room (id, phase, deadline_ms, round, seed, revision, reason) VALUES (1, 'lobby', 0, 0, 1, 0, '')",
            params: {},
        },
    )?
    Sqlite.execute!(
        {
            db,
            query: "CREATE TABLE IF NOT EXISTS players (id TEXT PRIMARY KEY, seat INTEGER NOT NULL, status TEXT NOT NULL, board TEXT NOT NULL, piece TEXT NOT NULL, bag TEXT NOT NULL, seed INTEGER NOT NULL, board_revision INTEGER NOT NULL, sequence INTEGER NOT NULL, back_to_back INTEGER NOT NULL, last_lock_ms INTEGER NOT NULL, locks_window INTEGER NOT NULL, disconnect_deadline INTEGER NOT NULL, garbage TEXT NOT NULL DEFAULT '', cursor_seat INTEGER NOT NULL DEFAULT 0, last_hole INTEGER NOT NULL DEFAULT -1, lines_sent INTEGER NOT NULL DEFAULT 0)",
            params: {},
        },
    )?
    try_column!(db, "ALTER TABLE players ADD COLUMN garbage TEXT NOT NULL DEFAULT ''")?
    try_column!(db, "ALTER TABLE players ADD COLUMN cursor_seat INTEGER NOT NULL DEFAULT 0")?
    try_column!(db, "ALTER TABLE players ADD COLUMN last_hole INTEGER NOT NULL DEFAULT -1")?
    try_column!(db, "ALTER TABLE players ADD COLUMN lines_sent INTEGER NOT NULL DEFAULT 0")?
    Sqlite.execute!(
        {
            db,
            query: "CREATE TABLE IF NOT EXISTS commands (player_id TEXT NOT NULL, sequence INTEGER NOT NULL, ok INTEGER NOT NULL, error TEXT NOT NULL, board TEXT NOT NULL, revision INTEGER NOT NULL, piece TEXT NOT NULL, eliminated INTEGER NOT NULL, PRIMARY KEY (player_id, sequence))",
            params: {},
        },
    )
}

interrupt_if_active! = |db| {
    room = load_room!(db)?
    if room.phase == "round" or room.phase == "countdown" {
        now = now_ms!()
        set_phase!(db, "result", now + result_ms!({}), room.round, room.seed, "Interrupted")
    } else {
        Ok({})
    }
}

document! = |db, player_id| {
    room = load_room!(db) ? |err| ServerErr("Failed to load room: ${Str.inspect(err)}")
    players = load_players!(db) ? |err| ServerErr("Failed to load players: ${Str.inspect(err)}")
    count = List.len(players).to_i64_wrap()
    if player_id != "" and player_exists(players, player_id) {
        html_ok(Html.render(Blocks.playPage({ view: build_view(room, players, player_id, now_ms!()) })))
    } else {
        html_ok(Html.render(Blocks.lobby({ player_count: count, full: count >= 8 })))
    }
}

stream_room! = |db, player_id|
    Ok(
        Server.stream(
            Sse.unfold!(
                { revision: -1.I64, last_emit: 0.I64, player_id },
                |state| {
                    match tick_room!(db) {
                        Err(_) => Ok(End)
                        Ok({}) =>
                            match load_room!(db) {
                                Err(_) => Ok(End)
                                Ok(room) => {
                                    now = now_ms!()
                                    if room.revision == state.revision and now - state.last_emit < 10000 {
                                        Ok(Wait({ state, wake: After(200) }))
                                    } else {
                                        match load_players!(db) {
                                            Err(_) => Ok(End)
                                            Ok(players) => {
                                                view = build_view(room, players, state.player_id, now)
                                                event = Datastar.patch_elements(Blocks.gamePatch({ view: view }))
                                                Ok(
                                                    Emit(
                                                        {
                                                            event,
                                                            state: {
                                                                revision: room.revision,
                                                                last_emit: now,
                                                                player_id: state.player_id,
                                                            },
                                                            wake: After(200),
                                                        },
                                                    ),
                                                )
                                            }
                                        }
                                    }
                                }
                            }
                    }
                },
            ),
        ),
    )

join_player! = |db, player_id| {
    players = load_players!(db) ? |err| ServerErr("Failed to join: ${Str.inspect(err)}")
    if player_exists(players, player_id) {
        redirect(303, "${prefix}/", [])
    } else if List.len(players).to_i64_wrap() >= 8 {
        redirect(303, "${prefix}/", [])
    } else {
        room = load_room!(db) ? |err| ServerErr("Failed to join: ${Str.inspect(err)}")
        seat = next_seat(players)
        id = new_id!(seat)
        status = if room.phase == "lobby" or room.phase == "countdown" {
            "seated"
        } else {
            "queued"
        }
        opened = Game.draw_piece([], room.seed)
        params : SeatParams
        params = {
            id,
            seat,
            status,
            board: Game.empty_board,
            piece: opened.piece,
            bag: Game.encode_bag(opened.bag),
            seed: opened.seed,
            garbage: "",
            cursor_seat: (seat + 1).rem_by(8),
            last_hole: -1,
            lines_sent: 0,
        }
        Sqlite.execute!(
            {
                db,
                query: "INSERT INTO players (id, seat, status, board, piece, bag, seed, board_revision, sequence, back_to_back, last_lock_ms, locks_window, disconnect_deadline, garbage, cursor_seat, last_hole, lines_sent) VALUES (:id, :seat, :status, :board, :piece, :bag, :seed, 0, 0, 0, 0, 0, 0, :garbage, :cursor_seat, :last_hole, :lines_sent)",
                params,
            },
        )
            ? |err| ServerErr("Failed to join: ${Str.inspect(err)}")
        bump_revision!(db) ? |err| ServerErr("Failed to join: ${Str.inspect(err)}")
        redirect(
            303,
            "${prefix}/",
            [{ name: "Set-Cookie", value: "blocks=${id}; Path=/play/blocks; HttpOnly; SameSite=Lax" }],
        )
    }
}

leave_player! = |db, player_id| {
    if player_id != "" {
        params : IdParams
        params = { id: player_id }
        Sqlite.execute!({ db, query: "DELETE FROM players WHERE id = :id", params })
            ? |err| ServerErr("Failed to leave: ${Str.inspect(err)}")
        bump_revision!(db) ? |err| ServerErr("Failed to leave: ${Str.inspect(err)}")
    } else {
        {}
    }
    redirect(303, "${prefix}/", [{ name: "Set-Cookie", value: "blocks=; Path=/play/blocks; HttpOnly; SameSite=Lax; Max-Age=0" }])
}

ready_player! = |db, player_id| {
    room = load_room!(db) ? |err| ServerErr("Failed to ready: ${Str.inspect(err)}")
    if player_id == "" or room.phase != "lobby" {
        redirect(303, "${prefix}/", [])
    } else {
        params : ReadyParams
        params = { id: player_id, status: "ready" }
        Sqlite.execute!({ db, query: "UPDATE players SET status = :status WHERE id = :id", params })
            ? |err| ServerErr("Failed to ready: ${Str.inspect(err)}")
        bump_revision!(db) ? |err| ServerErr("Failed to ready: ${Str.inspect(err)}")
        redirect(303, "${prefix}/", [])
    }
}

lock_request! = |db, request, player_id| {
    if origin_ok(request.headers()) == False {
        json_status(403, ack_json(empty_ack("InvalidOrigin")))
    } else if player_id == "" {
        json_status(401, ack_json(empty_ack("Unauthenticated")))
    } else {
        match request.body().with_limit(4 * 1024).read_all!() {
            Err(_) => json_status(413, ack_json(empty_ack("OversizedBody")))
            Ok(bytes) => {
                json = Str.from_utf8_lossy(bytes)
                apply_lock!(db, player_id, json)
            }
        }
    }
}

apply_lock! = |db, player_id, json| {
    now = now_ms!()
    player = load_player!(db, player_id)?
    room = load_room!(db)?
    sequence = json_i64(json, "sequence")
    match load_ack!(db, player_id, sequence) {
        Ok(ack) => json_status(200, ack_json(ack))
        Err(_) if sequence != 0 and sequence <= player.sequence =>
            json_status(409, ack_json(player_ack(player, "DuplicateSequence")))
        Err(_) if sequence != player.sequence + 1 =>
            json_status(409, ack_json(player_ack(player, "DuplicateSequence")))
        Err(_) if room.phase != "round" or player.status != "playing" =>
            json_status(409, ack_json(player_ack(player, "WrongPhase")))
        Err(_) => {
            window = if now - player.last_lock_ms >= 1000 { 1 } else { player.locks_window + 1 }
            if window > 10 {
                json_status(429, ack_json(player_ack(player, "RateLimited")))
            } else if json_i64(json, "board_revision") != player.board_revision {
                json_status(409, ack_json(player_ack(player, "StaleRevision")))
            } else {
                piece = json_str(json, "piece")
                if piece != player.piece {
                    json_status(409, ack_json(player_ack(player, "UnknownPiece")))
                } else {
                    rot = json_i64(json, "rotation")
                    x = json_i64(json, "x")
                    y = json_i64(json, "y")
                    b2b = player.back_to_back != 0
                    match Game.lock(player.board, piece, rot, x, y, b2b) {
                        Err(UnknownPiece) => json_status(409, ack_json(player_ack(player, "UnknownPiece")))
                        Err(BadRotation) => json_status(409, ack_json(player_ack(player, "BadRotation")))
                        Err(InvalidGeometry) => json_status(409, ack_json(player_ack(player, "InvalidGeometry")))
                        Ok(placed) => {
                            players = load_players!(db)?
                            cancelled = Game.cancel_incoming(Game.decode_queue(player.garbage), placed.attack)
                            living =
                                List.map(
                                    List.keep_if(players, |row| row.status == "playing" and row.id != player.id),
                                    |row| row.seat,
                                )
                            resolved = Game.resolve_residual(cancelled, player.seat, player.cursor_seat, living)
                            ready = Game.apply_ready(resolved.incoming, now)
                            boarded = Game.insert_garbage(placed.board, ready.applied)
                            drawn = Game.draw_piece(Game.decode_bag(player.bag), player.seed)
                            eliminated = if Game.spawn_ok(boarded, drawn.piece) { 0 } else { 1 }
                            status = if eliminated != 0 { "eliminated" } else { "playing" }
                            next = {
                                id: player.id,
                                board: boarded,
                                piece: drawn.piece,
                                bag: Game.encode_bag(drawn.bag),
                                seed: drawn.seed,
                                board_revision: player.board_revision + 1,
                                sequence,
                                back_to_back: if placed.back_to_back { 1 } else { 0 },
                                status,
                                last_lock_ms: now,
                                locks_window: window,
                                garbage: Game.encode_queue(ready.remaining),
                                cursor_seat: resolved.cursor,
                                lines_sent: player.lines_sent + resolved.residual,
                            }
                            save_lock!(db, next)?
                            victims = List.keep_if(players, |row| row.seat == resolved.target)
                            match victims {
                                [victim, ..] if resolved.writes == 1 => {
                                    victim_queue = Game.decode_queue(victim.garbage)
                                    hole = Game.next_hole(victim.last_hole)
                                    packet = {
                                        rows: resolved.residual,
                                        ready_at_ms: now + Game.garbage_delay_ms,
                                        hole,
                                        order: Game.next_order(victim_queue),
                                    }
                                    save_target!(
                                        db,
                                        {
                                            id: victim.id,
                                            garbage: Game.encode_queue(List.append(victim_queue, packet)),
                                            last_hole: hole,
                                        },
                                    )?
                                    {}
                                }
                                _ => {}
                            }
                            ack = {
                                ok: 1.I64,
                                error: "",
                                board: next.board,
                                revision: next.board_revision,
                                piece: next.piece,
                                sequence,
                                eliminated,
                            }
                            store_ack!(db, player_id, ack)?
                            bump_revision!(db)?
                            json_status(200, ack_json(ack))
                        }
                    }
                }
            }
        }
    }
}

tick_room! = |db| {
    room = load_room!(db)?
    now = now_ms!()
    players = load_players!(db)?
    ready = List.len(List.keep_if(players, |p| p.status == "ready" or p.status == "playing")).to_i64_wrap()
    living = List.len(List.keep_if(players, |p| p.status == "playing")).to_i64_wrap()
    match room.phase {
        "lobby" if ready >= 2 and room.deadline_ms == 0 =>
            set_phase!(db, "countdown", now + countdown_ms!({}), room.round, room.seed, "")
        "countdown" if now >= room.deadline_ms =>
            if ready >= 2 {
                start_round!(db, room, players, now)
            } else {
                set_phase!(db, "lobby", 0, room.round, room.seed, "")
            }
        "round" if living <= 1 or (room.deadline_ms != 0 and now >= room.deadline_ms) => {
            reason = if living <= 1 { "Last player standing" } else { "Timeout" }
            set_phase!(db, "result", now + result_ms!({}), room.round, room.seed, reason)
        }
        "result" if now >= room.deadline_ms =>
            reset_lobby!(db, room)
        _ => Ok({})
    }
}

start_round! = |db, room, players, now| {
    seated = List.keep_if(players, |player| player.status != "queued")
    deal_players!(db, seated, room.seed)?
    set_phase!(db, "round", now + round_ms!({}), room.round + 1, room.seed, "")
}

deal_players! = |db, players, seed|
    match players {
        [] => Ok({})
        [player, .. as rest] => {
            opened = Game.draw_piece([], seed + player.seat)
            params : SeatParams
            params = {
                id: player.id,
                seat: player.seat,
                status: "playing",
                board: Game.empty_board,
                piece: opened.piece,
                bag: Game.encode_bag(opened.bag),
                seed: opened.seed,
                garbage: "",
                cursor_seat: (player.seat + 1).rem_by(8),
                last_hole: -1,
                lines_sent: 0,
            }
            Sqlite.execute!(
                {
                    db,
                    query: "UPDATE players SET seat = :seat, status = :status, board = :board, piece = :piece, bag = :bag, seed = :seed, board_revision = 0, sequence = 0, back_to_back = 0, garbage = :garbage, cursor_seat = :cursor_seat, last_hole = :last_hole, lines_sent = :lines_sent WHERE id = :id",
                    params,
                },
            )?
            deal_players!(db, rest, seed)
        }
    }

reset_lobby! = |db, room| {
    Sqlite.execute!({ db, query: "UPDATE players SET status = 'seated' WHERE status != 'queued'", params: {} })?
    Sqlite.execute!({ db, query: "UPDATE players SET status = 'seated' WHERE status = 'queued'", params: {} })?
    set_phase!(db, "lobby", 0, room.round, room.seed, "")
}

set_phase! = |db, phase, deadline_ms, round, seed, reason| {
    params : PhaseParams
    params = { phase, deadline_ms, round, seed, reason }
    Sqlite.execute!(
        {
            db,
            query: "UPDATE room SET phase = :phase, deadline_ms = :deadline_ms, round = :round, seed = :seed, reason = :reason, revision = revision + 1 WHERE id = 1",
            params,
        },
    )
}

save_lock! = |db, row| {
    params : LockSave
    params = row
    Sqlite.execute!(
        {
            db,
            query: "UPDATE players SET board = :board, piece = :piece, bag = :bag, seed = :seed, board_revision = :board_revision, sequence = :sequence, back_to_back = :back_to_back, status = :status, last_lock_ms = :last_lock_ms, locks_window = :locks_window, garbage = :garbage, cursor_seat = :cursor_seat, lines_sent = :lines_sent WHERE id = :id",
            params,
        },
    )
}

save_target! = |db, row| {
    params : TargetSave
    params = row
    Sqlite.execute!(
        {
            db,
            query: "UPDATE players SET garbage = :garbage, last_hole = :last_hole WHERE id = :id",
            params,
        },
    )
}

store_ack! = |db, player_id, ack| {
    params : AckInsert
    params = {
        player_id,
        sequence: ack.sequence,
        ok: ack.ok,
        error: ack.error,
        board: ack.board,
        revision: ack.revision,
        piece: ack.piece,
        eliminated: ack.eliminated,
    }
    Sqlite.execute!(
        {
            db,
            query: "INSERT OR REPLACE INTO commands (player_id, sequence, ok, error, board, revision, piece, eliminated) VALUES (:player_id, :sequence, :ok, :error, :board, :revision, :piece, :eliminated)",
            params,
        },
    )
}

load_ack! = |db, player_id, sequence| {
    params : { player_id : Str, sequence : I64 }
    params = { player_id, sequence }
    row : AckRow
    row = Sqlite.query!(
        {
            db,
            query: "SELECT ok, error, board, revision, piece, sequence, eliminated FROM commands WHERE player_id = :player_id AND sequence = :sequence",
            params,
            limits: Sqlite.default_query_limits,
        },
    )?
    Ok(row)
}

load_room! = |db| {
    row : RoomRow
    row = Sqlite.query!(
        {
            db,
            query: "SELECT phase, deadline_ms, round, seed, revision, reason FROM room WHERE id = 1",
            params: {},
            limits: Sqlite.default_query_limits,
        },
    )?
    Ok(row)
}

load_players! = |db| {
    rows : List({
        id : Str,
        seat : I64,
        status : Str,
        board : Str,
        piece : Str,
        bag : Str,
        seed : I64,
        board_revision : I64,
        sequence : I64,
        back_to_back : I64,
        last_lock_ms : I64,
        locks_window : I64,
        disconnect_deadline : I64,
        garbage : Str,
        cursor_seat : I64,
        last_hole : I64,
        lines_sent : I64,
    })
    rows = Sqlite.query_many!(
        {
            db,
            query: "SELECT id, seat, status, board, piece, bag, seed, board_revision, sequence, back_to_back, last_lock_ms, locks_window, disconnect_deadline, garbage, cursor_seat, last_hole, lines_sent FROM players ORDER BY seat",
            params: {},
            limits: Sqlite.default_query_limits,
        },
    )?
    Ok(rows)
}

load_player! = |db, player_id| {
    params : IdParams
    params = { id: player_id }
    row : PlayerRow
    row = Sqlite.query!(
        {
            db,
            query: "SELECT id, seat, status, board, piece, bag, seed, board_revision, sequence, back_to_back, last_lock_ms, locks_window, disconnect_deadline, garbage, cursor_seat, last_hole, lines_sent FROM players WHERE id = :id",
            params,
            limits: Sqlite.default_query_limits,
        },
    )?
    Ok(row)
}

bump_revision! = |db|
    Sqlite.execute!({ db, query: "UPDATE room SET revision = revision + 1 WHERE id = 1", params: {} })

build_view = |room, players, player_id, now| {
    seats = List.map(
        players,
        |player| {
            status =
                match player.status {
                    "eliminated" => "eliminated"
                    "playing" => "alive"
                    "ready" => "ready"
                    "queued" => "queued"
                    _ => "seated"
                }
            living = List.map(List.keep_if(players, |row| row.status == "playing"), |row| row.seat)
            target = Game.select_target(living, player.seat, player.cursor_seat)
            {
                seat: player.seat,
                status,
                board: player.board,
                target: if target < 0 { player.cursor_seat } else { target },
                queue: Game.queue_rows(player.garbage),
                ready: Game.ready_rows_now(player.garbage, now),
                piece: player.piece,
                you: if player.id == player_id { 1.I64 } else { 0.I64 },
            }
        },
    )
    winner = List.fold(
        players,
        "",
        |acc, player|
            if acc != "" {
                acc
            } else if player.status == "playing" {
                "Seat ${player.seat.to_str()}"
            } else {
                acc
            },
    )
    {
        phase: room.phase,
        revision: room.revision,
        deadline_ms: room.deadline_ms,
        round: room.round,
        piece: "",
        winner,
        reason: room.reason,
        seats,
    }
}

player_exists = |players, player_id|
    List.any(players, |player| player.id == player_id)

next_seat = |players| {
    used = List.map(players, |player| player.seat)
    List.fold(
        [0, 1, 2, 3, 4, 5, 6, 7],
        -1,
        |acc, seat|
            if acc >= 0 {
                acc
            } else if List.contains(used, seat) {
                acc
            } else {
                seat
            },
    )
}

player_ack = |player, error|
    {
        ok: 0.I64,
        error,
        board: player.board,
        revision: player.board_revision,
        piece: player.piece,
        sequence: player.sequence,
        eliminated: if player.status == "eliminated" { 1.I64 } else { 0.I64 },
    }

empty_ack = |error|
    {
        ok: 0.I64,
        error,
        board: Game.empty_board,
        revision: 0.I64,
        piece: "",
        sequence: 0.I64,
        eliminated: 0.I64,
    }

ack_json = |ack|
    "{\"ok\":${ack.ok.to_str()},\"error\":\"${ack.error}\",\"board\":\"${ack.board}\",\"revision\":${ack.revision.to_str()},\"piece\":\"${ack.piece}\",\"sequence\":${ack.sequence.to_str()},\"eliminated\":${ack.eliminated.to_str()}}"

html_ok = |body|
    Ok(
        Server.respond(
            Response.from_status(200)
            .with_headers([{ name: "Content-Type", value: "text/html; charset=utf-8" }])
            .with_body(Str.to_utf8(body)),
        ),
    )

text_ok = |body|
    Ok(
        Server.respond(
            Response.from_status(200)
            .with_headers([{ name: "Content-Type", value: "text/plain; charset=utf-8" }])
            .with_body(Str.to_utf8(body)),
        ),
    )

json_status = |status, body|
    Ok(
        Server.respond(
            Response.from_status(status)
            .with_headers([{ name: "Content-Type", value: "application/json" }])
            .with_body(Str.to_utf8(body)),
        ),
    )

redirect = |status, location, cookie|
    Ok(
        Server.respond(
            Response.from_status(status)
            .with_headers(List.concat([{ name: "Location", value: location }], cookie))
            .with_body([]),
        ),
    )

origin_ok = |headers| {
    origin = header_value(headers, "origin")
    referer = header_value(headers, "referer")
    host = header_value(headers, "host")
    if origin != "" {
        Str.contains(origin, host) or Str.contains(origin, "127.0.0.1") or Str.contains(origin, "localhost")
    } else if referer != "" {
        Str.contains(referer, host) or Str.contains(referer, "127.0.0.1")
    } else {
        True
    }
}

header_value = |headers, name|
    List.fold(
        headers,
        "",
        |acc, header|
            if acc != "" {
                acc
            } else if header.name == name or header.name == capitalize(name) {
                header.value
            } else {
                acc
            },
    )

capitalize = |name|
    if name == "origin" {
        "Origin"
    } else if name == "referer" {
        "Referer"
    } else if name == "host" {
        "Host"
    } else if name == "cookie" {
        "Cookie"
    } else {
        name
    }

cookie_value_in = |headers, key|
    cookie_value(header_value(headers, "cookie"), key)

cookie_value = |raw, key|
    List.fold(
        Str.split_on(raw, ";"),
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

json_str = |json, key| {
    needle = "\"${key}\":"
    parts = Str.split_on(json, needle)
    match List.get(parts, 1) {
        Ok(after) => json_string_value(skip_ws(after))
        Err(_) => ""
    }
}

json_i64 = |json, key| {
    needle = "\"${key}\":"
    parts = Str.split_on(json, needle)
    match List.get(parts, 1) {
        Ok(after) =>
            match I64.from_str(take_num(skip_ws(after))) {
                Ok(n) => n
                Err(_) => 0
            }
        Err(_) => 0
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
        Ok(34) if escape == False => Str.from_utf8_lossy(acc)
        Ok(92) if escape == False => read_json_string(bytes, index + 1, acc, True)
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

take_num = |text| take_num_bytes(Str.to_utf8(text), [])

take_num_bytes = |bytes, acc|
    match List.get(bytes, 0) {
        Ok(head) if (head >= 48 and head <= 57) or (head == 45 and List.is_empty(acc)) =>
            take_num_bytes(List.drop_first(bytes, 1), List.append(acc, head))
        _ => Str.from_utf8_lossy(acc)
    }

now_ms! = || {
    ts = UnixTime.now!()
    secs = UnixTime.Timestamp.seconds_since_epoch(ts)
    nanos = UnixTime.Timestamp.subsecond_nanoseconds(ts)
    secs * 1000.I64 + nanos.to_i64().div_trunc_by(1_000_000.I64)
}

new_id! = |seat| {
    ts = UnixTime.now!()
    secs = UnixTime.Timestamp.seconds_since_epoch(ts)
    "${secs.to_str()}-${seat.to_str()}"
}

env_ms! = |name, fallback|
    match Env.var_str!(name) {
        Ok(value) =>
            match I64.from_str(value) {
                Ok(n) if n > 0 => n
                _ => fallback
            }
        Err(_) => fallback
    }

countdown_ms! = |_| env_ms!("BLOCKS_COUNTDOWN_MS", 10000)
result_ms! = |_| env_ms!("BLOCKS_RESULT_MS", 10000)
round_ms! = |_| env_ms!("BLOCKS_ROUND_MS", 300000)

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
