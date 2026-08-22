app [Context, program] {
    pf: platform "https://github.com/roc-lang/basic-webserver/releases/download/0.16.0/42jC1JT3auhHSmv2Ah8mW5F2MXiAakq1UQQ4NQceQjXw.tar.zst",
    http: "https://github.com/roc-lang/http/releases/download/1.0.0/6ZUwqYhCS8PU9Mo6MF7oV82ET2o7KYb57CLKDq4cq4sS.tar.zst",
}

import pf.Env
import pf.Path
import pf.Server
import pf.Sqlite
import pf.Sse
import http.Method
import http.Response
import Datastar
import Html
import Notes
import Signals

Context : { db : Sqlite.Db }
Note : { id : I64, body : Str }
BodyParams : { body : Str }

program = { init!, respond!, shutdown! }

init! : () => Try({ config : Server.Config, context : Context }, [Exit(I64), ..])
init! = || {
    db_path =
        match Env.var!("DB_PATH") {
            Ok(path) => Path.from_os_str(path)
            Err(_) => Path.utf8("./notes.db")
        }
    db = Sqlite.open!(Sqlite.default_config(db_path)) ? |_| Exit(2)
    Sqlite.execute!(
        {
            db,
            query: "CREATE TABLE IF NOT EXISTS notes (id INTEGER PRIMARY KEY, body TEXT NOT NULL)",
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
            files: [Server.static_mount({ at: "/assets", files: assets })],
            liveness: [],
            readiness: [],
        })

    Ok({ config, context: { db } })
}

respond! : Server.Request, Context => Try(Server.Outcome, [ServerErr(Str), ..])
respond! = |request, { db }| {
    method = Method.to_str(request.method())
    path =
        match request.target() {
            Resource({ raw_path, .. }) => raw_path
            _ => ""
        }
    match (method, path) {
        ("GET", "/") => {
            notes = load_notes!(db) ? |err| ServerErr("Failed to read notes: ${Str.inspect(err)}")
            html_ok(Html.render(Notes.page({ notes })))
        }
        ("GET", "/health") => text_ok("ok")
        ("POST", "/actions/notes/add") => add_note!(db, request)
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

load_notes! = |db| {
    rows : List(Note)
    rows = Sqlite.query_many!(
        {
            db,
            query: "SELECT id, body FROM notes ORDER BY id DESC",
            params: {},
            limits: Sqlite.default_query_limits,
        },
    )?
    Ok(rows)
}

add_note! = |db, request| {
    json = Signals.from_request!(request) ? |err| ServerErr("Failed to read note: ${Str.inspect(err)}")
    body = Str.trim(Signals.str(json, "body"))
    if body != "" {
        params : BodyParams
        params = { body }
        Sqlite.execute!(
            {
                db,
                query: "INSERT INTO notes (body) VALUES (:body)",
                params,
            },
        )
            ? |err| ServerErr("Failed to add note: ${Str.inspect(err)}")
    } else {
        Ok({})
    }?
    notes = load_notes!(db) ? |err| ServerErr("Failed to read notes: ${Str.inspect(err)}")
    Ok(patch!(Notes.notesBoard({ notes })))
}

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

patch! = |node| {
    events!([Datastar.patch_elements(node)])
}

events! = |events| {
    Server.stream(
        Sse.unfold!(events, |pending|
            match pending {
                [event, .. as rest] => Ok(Emit({ event, state: rest, wake: Immediately }))
                [] => Ok(End)
            }
        ),
    )
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
