app [Context, program] {
    pf: platform "../rocci-web/vendor/main.roc",
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
import pf.Rocci
import Ui

Context : { db : Sqlite.Db }
NameParam : { name : Str }
CountRow : { value : I64 }
NowRow : { now : I64 }

catalog = [
    "view",
    "get_fragment",
    "fragment",
    "command",
    "live",
    "prefix_fragment",
    "get_events",
    "unfold",
]

listed_routes = "<li><code>GET /</code></li><li><code>GET /counter</code></li><li><code>GET /live</code></li><li><code>GET /tabs</code></li><li><code>GET /compose</code></li><li><code>GET /search</code></li><li><code>GET /clock</code></li><li><code>POST /actions/counter/increment</code></li><li><code>POST /actions/live/increment</code></li><li><code>GET /sse</code></li><li><code>GET /actions/tabs/*</code></li><li><code>GET /actions/signals/compose</code></li><li><code>GET /actions/search/results</code></li><li><code>GET /actions/clock/ticks</code></li><li><code>GET /health</code></li>"

program = Rocci.program({
    init!,
    respond!,
})

respond! : Server.Request, Context => Try(Server.Outcome, [ServerErr(Str), ..])
respond! = |request, context| {
    method = Method.to_str(request.method())
    path =
        match request.target() {
            Resource({ raw_path: raw, .. }) => raw
            _ => ""
        }
    match (method, path) {
        ("GET", "/") => {
            html = home!(context, request) ? |err| ServerErr(Str.inspect(err))
            Rocci.view!(html)
        }
        ("GET", "/counter") => {
            html = counter_page!(context, request) ? |err| ServerErr(Str.inspect(err))
            Rocci.view!(html)
        }
        ("GET", "/live") => {
            html = live_page!(context, request) ? |err| ServerErr(Str.inspect(err))
            Rocci.view!(html)
        }
        ("GET", "/tabs") => {
            html = tabs_page!(context, request) ? |err| ServerErr(Str.inspect(err))
            Rocci.view!(html)
        }
        ("GET", "/compose") => {
            html = compose_page!(context, request) ? |err| ServerErr(Str.inspect(err))
            Rocci.view!(html)
        }
        ("GET", "/search") => {
            html = search_page!(context, request) ? |err| ServerErr(Str.inspect(err))
            Rocci.view!(html)
        }
        ("GET", "/clock") => {
            html = clock_page!(context, request) ? |err| ServerErr(Str.inspect(err))
            Rocci.view!(html)
        }
        ("POST", "/actions/counter/increment") => {
            html = increment_fragment!(context, request) ? |err| ServerErr(Str.inspect(err))
            Rocci.fragment!(html)
        }
        ("POST", "/actions/counter/reset") => {
            html = reset_fragment!(context, request) ? |err| ServerErr(Str.inspect(err))
            Rocci.fragment!(html)
        }
        ("POST", "/actions/live/increment") => {
            _ = increment_live!(context, request) ? |err| ServerErr(Str.inspect(err))
            Rocci.command!(request.headers())
        }
        ("POST", "/actions/live/reset") => {
            _ = reset_live!(context, request) ? |err| ServerErr(Str.inspect(err))
            Rocci.command!(request.headers())
        }
        ("GET", "/sse") => {
            stream = Sse.unfold!(
                "",
                |prev|
                    match live_slice!(context, request) {
                        Ok(html) => {
                            rendered = Html.render(html)
                            if rendered == prev {
                                Ok(Emit({ event: Sse.Event.data(""), state: prev, wake: After(100) }))
                            } else {
                                Ok(Emit({ event: Datastar.patch_elements(html), state: rendered, wake: After(100) }))
                            }
                        }
                        Err(_) => Ok(Emit({ event: Sse.Event.data(""), state: prev, wake: After(100) }))
                    },
            )
            Rocci.unfold!(stream)
        }
        ("GET", "/actions/signals/compose") => {
            events = compose_events!(context, request) ? |err| ServerErr(Str.inspect(err))
            Rocci.events!(events)
        }
        ("GET", "/actions/search/results") => {
            html = search_results!(context, request) ? |err| ServerErr(Str.inspect(err))
            Rocci.get_fragment!(html)
        }
        ("GET", "/actions/clock/ticks") => Rocci.unfold!(clock_ticks!(context, request))
        _ =>
            if method == "GET" and path == "/health" {
                health!({})
            } else if method == "GET" {
                match Rocci.prefix_remainder(path, "/actions/tabs/") {
                    Ok(id) => {
                        html = tabs_patch!(id, context, request)
                            ? |err| ServerErr(Str.inspect(err))
                        Rocci.fragment!(html)
                    }
                    Err(_) =>
                        match Rocci.slash_alternate(path) {
                            Ok("/") => redirect_slash!("/")
                            Ok("/counter") => redirect_slash!("/counter")
                            Ok("/live") => redirect_slash!("/live")
                            Ok("/tabs") => redirect_slash!("/tabs")
                            Ok("/compose") => redirect_slash!("/compose")
                            Ok("/search") => redirect_slash!("/search")
                            Ok("/clock") => redirect_slash!("/clock")
                            Ok("/health") => redirect_slash!("/health")
                            _ => not_found!(method, path, listed_routes)
                        }
                }
            } else {
                not_found!(method, path, listed_routes)
            }
    }
}

init! : () => Try({ config : Server.Config, context : Context }, [Exit(I64), ..])
init! = || {
    unique_keys(
        [
            "GET /",
            "GET /counter",
            "GET /live",
            "GET /tabs",
            "GET /compose",
            "GET /search",
            "GET /clock",
            "POST /actions/counter/increment",
            "POST /actions/counter/reset",
            "POST /actions/live/increment",
            "POST /actions/live/reset",
            "GET /sse",
            "GET /actions/signals/compose",
            "GET /actions/search/results",
            "GET /actions/clock/ticks",
            "GET /actions/tabs/",
        ],
    )?
    db_path =
        match Env.var!("DB_PATH") {
            Ok(path) => Path.from_os_str(path)
            Err(_) => Path.utf8("./method-role.db")
        }
    db = Sqlite.open!(Sqlite.default_config(db_path)) ? |_| Exit(2)
    setup_db!(db) ? |_| Exit(2)
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
    Ok({ config: config, context: { db: db } })
}

setup_db! = |db| {
    Sqlite.execute!(
        {
            db,
            query: "CREATE TABLE IF NOT EXISTS kv (name TEXT PRIMARY KEY, value INTEGER NOT NULL)",
            params: {},
        },
    )?
    ensure_key!(db, "fragment")?
    ensure_key!(db, "live")
}

ensure_key! = |db, name| {
    params : NameParam
    params = { name: name }
    Sqlite.execute!(
        {
            db,
            query: "INSERT OR IGNORE INTO kv (name, value) VALUES (:name, 0)",
            params,
        },
    )
}

home! = |_ctx, _request|
    Ok(Ui.home({}))

counter_page! = |{ db }, _request| {
    count = read_kv!(db, "fragment")?
    Ok(Ui.counterPage({ count: count }))
}

increment_fragment! = |{ db }, _request| {
    count = bump_kv!(db, "fragment")?
    Ok(Ui.counterCard({ count: count }))
}

reset_fragment! = |{ db }, _request| {
    count = set_kv!(db, "fragment", 0)?
    Ok(Ui.counterCard({ count: count }))
}

live_page! = |{ db }, _request| {
    count = read_kv!(db, "live")?
    Ok(Ui.livePage({ count: count }))
}

increment_live! = |{ db }, _request| {
    _ = bump_kv!(db, "live")?
    Ok({})
}

reset_live! = |{ db }, _request| {
    _ = set_kv!(db, "live", 0)?
    Ok({})
}

live_slice! = |{ db }, _request| {
    count = read_kv!(db, "live")?
    Ok(Ui.liveCard({ count: count }))
}

tabs_page! = |_ctx, _request|
    Ok(Ui.tabsPage({ selected: "0", panel: tab_copy("0") }))

tabs_patch! = |id, _ctx, _request|
    Ok(Ui.tabsPanel({ selected: id, panel: tab_copy(id) }))

tab_copy = |id|
    match id {
        "0" => "Alpha is the default tab. The remainder after /actions/tabs/ is the id."
        "1" => "Beta is reached by prefix_fragment. Illegal GET+command stays unrepresentable."
        "2" => "Gamma is still one Html fragment. Dispatch wraps the patch."
        _ => "Unknown tab. Prefix routes do not validate the remainder."
    }

compose_page! = |_ctx, _request|
    Ok(Ui.composePage({ notice: "idle" }))

compose_events! = |_ctx, _request|
    Ok(
        [
            Datastar.patch_elements(Ui.composeStatus({ notice: "ready" })),
            Datastar.patch_signals_with("{\"notice\":\"ready\"}", [OnlyIfMissing(True)]),
        ],
    )

search_page! = |_ctx, _request|
    Ok(Ui.searchPage({ query: "", hits: catalog }))

search_results! = |_ctx, request| {
    query = request_query(request, "q")
    Ok(Ui.searchResults({ hits: filter_catalog(query) }))
}

clock_page! = |{ db }, _request| {
    now = read_now!(db)?
    Ok(Ui.clockPage({ now: now }))
}

clock_ticks! = |{ db }, _request|
    Sse.unfold!(
        0.I64,
        |_prev| {
            now = read_now!(db) ? |err| ServerErr("clock: ${Str.inspect(err)}")
            event = Datastar.patch_elements(Ui.clockCard({ now: now }))
            Ok(Emit({ event, state: now, wake: After(1000) }))
        },
    )

filter_catalog = |query| {
    if query == "" {
        catalog
    } else {
        List.keep_if(catalog, |name| Str.contains(name, query))
    }
}

request_query = |request, key|
    match request.target() {
        Resource({ raw_query: Present(query), .. }) => query_value(query, key)
        _ => ""
    }

query_value = |query, key| {
    from_pair = pair_value(query, key)
    if from_pair != "" {
        from_pair
    } else {
        json_string_value(query, key)
    }
}

pair_value = |query, key| {
    needle = "${key}="
    parts = Str.split_on(query, needle)
    match List.get(parts, 1) {
        Ok(after) => {
            end = Str.split_on(after, "&")
            match List.get(end, 0) {
                Ok(value) => percent_decode(value)
                Err(_) => ""
            }
        }
        Err(_) => ""
    }
}

json_string_value = |text, key| {
    needle = "\"${key}\":\""
    parts = Str.split_on(text, needle)
    match List.get(parts, 1) {
        Ok(after) =>
            match List.get(Str.split_on(after, "\""), 0) {
                Ok(value) => percent_decode(value)
                Err(_) => ""
            }
        Err(_) => ""
    }
}

percent_decode = |text|
    Str.join_with(Str.split_on(text, "%20"), " ")

read_kv! = |db, name| {
    params : NameParam
    params = { name: name }
    row : CountRow
    row = Sqlite.query!(
        {
            db,
            query: "SELECT value FROM kv WHERE name = :name",
            params,
            limits: Sqlite.default_query_limits,
        },
    )?
    Ok(row.value)
}

bump_kv! = |db, name| {
    params : NameParam
    params = { name: name }
    Sqlite.execute!(
        {
            db,
            query: "UPDATE kv SET value = value + 1 WHERE name = :name",
            params,
        },
    )?
    read_kv!(db, name)
}

set_kv! = |db, name, value| {
    params : { name : Str, value : I64 }
    params = { name, value }
    Sqlite.execute!(
        {
            db,
            query: "UPDATE kv SET value = :value WHERE name = :name",
            params,
        },
    )?
    read_kv!(db, name)
}

read_now! = |db| {
    row : NowRow
    row = Sqlite.query!(
        {
            db,
            query: "SELECT CAST(strftime('%s', 'now') AS INTEGER) AS now",
            params: {},
            limits: Sqlite.default_query_limits,
        },
    )?
    Ok(row.now)
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

health! = |{}| text_ok("ok")

not_found! = |method, path, listed|
    html_status(
        404,
        "<!doctype html><html><head><meta charset=\"utf-8\" /><title>Not found</title></head><body><h1>Not found</h1><p><code>${method} ${path}</code></p><ul>${listed}</ul></body></html>",
    )

redirect_slash! = |location|
    Ok(
        Server.respond(
            Response.from_status(308)
            .with_headers([{ name: "Location", value: location }])
            .with_body([]),
        ),
    )

unique_keys = |keys|
    match List.fold(
        keys,
        Ok([]),
        |acc, key|
            match acc {
                Err(err) => Err(err)
                Ok(seen) =>
                    if List.contains(seen, key) {
                        Err(Exit(2))
                    } else {
                        Ok(List.append(seen, key))
                    }
            },
    ) {
        Ok(_) => Ok({})
        Err(err) => Err(err)
    }

text_ok = |body|
    Ok(
        Server.respond(
            Response.from_status(200)
            .with_headers([{ name: "Content-Type", value: "text/plain; charset=utf-8" }])
            .with_body(Str.to_utf8(body)),
        ),
    )

html_status = |status, body|
    Ok(
        Server.respond(
            Response.from_status(status)
            .with_headers([{ name: "Content-Type", value: "text/html; charset=utf-8" }])
            .with_body(Str.to_utf8(body)),
        ),
    )
