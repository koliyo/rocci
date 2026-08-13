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
import Counter
import Datastar
import Html

Context : { db : Sqlite.Db }

program = { init!, respond!, shutdown! }

init! : () => Try({ config : Server.Config, context : Context }, [Exit(I64), ..])
init! = || {
    db_path =
        match Env.var!("DB_PATH") {
            Ok(path) => Path.from_os_str(path)
            Err(_) => Path.utf8("./counter.db")
        }
    db = Sqlite.open!(Sqlite.default_config(db_path)) ? |_| Exit(2)
    Sqlite.execute!(
        {
            db,
            query: "CREATE TABLE IF NOT EXISTS counter (id INTEGER PRIMARY KEY CHECK (id = 1), value INTEGER NOT NULL)",
            params: {},
        },
    )
        ? |_| Exit(2)
    Sqlite.execute!(
        {
            db,
            query: "INSERT OR IGNORE INTO counter (id, value) VALUES (1, 0)",
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

    match (Method.to_str(request.method()), path) {
        ("GET", "/") => {
            count = read_count!(db) ? |err| ServerErr("Failed to read counter: ${Str.inspect(err)}")
            html_ok(Html.render(Counter.counterPage({ count: count })))
        }
        ("GET", "/health") =>
            Ok(
                Server.respond(
                    Response.from_status(200)
                    .with_headers([{ name: "Content-Type", value: "text/plain; charset=utf-8" }])
                    .with_body(Str.to_utf8("ok")),
                ),
            )
        ("POST", "/api/counter/increment") => {
            count = increment_count!(db) ? |err| ServerErr("Failed to increment counter: ${Str.inspect(err)}")
            Ok(patch_counter!(count))
        }
        ("POST", "/api/counter/reset") => {
            count = reset_count!(db) ? |err| ServerErr("Failed to reset counter: ${Str.inspect(err)}")
            Ok(patch_counter!(count))
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

patch_counter! = |count| {
    event = Datastar.patch_elements(Counter.counterCard({ count: count }))
    Server.stream(
        Sse.unfold!(0, |state|
            match state {
                0 => Ok(Emit({ event, state: 1, wake: Immediately }))
                _ => Ok(End)
            }
        ),
    )
}

read_count! = |db| {
    row : { value : I64 }
    row = Sqlite.query!(
        {
            db,
            query: "SELECT value FROM counter WHERE id = 1",
            params: {},
            limits: Sqlite.default_query_limits,
        },
    )?
    Ok(row.value)
}

increment_count! = |db| {
    Sqlite.execute!(
        {
            db,
            query: "UPDATE counter SET value = value + 1 WHERE id = 1",
            params: {},
        },
    )?
    read_count!(db)
}

reset_count! = |db| {
    Sqlite.execute!(
        {
            db,
            query: "UPDATE counter SET value = 0 WHERE id = 1",
            params: {},
        },
    )?
    read_count!(db)
}
