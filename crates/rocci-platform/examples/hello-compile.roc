app [Context, program] {
    pf: platform "../platform/main.roc",
    http: "https://github.com/roc-lang/http/releases/download/1.0.0/6ZUwqYhCS8PU9Mo6MF7oV82ET2o7KYb57CLKDq4cq4sS.tar.zst",
}

import pf.Env
import pf.Rocci
import pf.Server
import http.Response

Context : {}

program = { init!, respond!, shutdown! }

success_source : Str
success_source = "@component Hello = |{}| {\n    <p>ok</p>\n}\n"

error_source : Str
error_source = "@component hello = |{}| {\n    <p>ok</p>\n}\n"

init! : () => Try({ config : Server.Config, context : Context }, [Exit(I64), ..])
init! = || {
    config = Server.default_config.with_listen({
        host: listen_host!({}),
        port: listen_port!({}),
    })
    Ok({ config, context: {} })
}

respond! : Server.Request, Context => Try(Server.Outcome, [ServerErr(Str), ..])
respond! = |request, _context| {
    path =
        match request.target() {
            Resource({ raw_path, .. }) => raw_path
            _ => ""
        }
    match path {
        "/bad" => {
            result = Rocci.compile!({ name: "Bad.rocci", source: error_source })
            code =
                match List.get(result.diagnostics, 0) {
                    Ok(diag) => diag.code
                    Err(_) => ""
                }
            status = if Str.starts_with(code, "RC") {
                200
            } else {
                500
            }
            Ok(
                Server.respond(
                    Response.from_status(status)
                    .with_headers([{ name: "Content-Type", value: "text/plain; charset=utf-8" }])
                    .with_body(Str.to_utf8(code)),
                ),
            )
        }
        _ => {
            result = Rocci.compile!({ name: "Hello.rocci", source: success_source })
            Ok(
                Server.respond(
                    Response.from_status(200)
                    .with_headers([{ name: "Content-Type", value: "text/plain; charset=utf-8" }])
                    .with_body(Str.to_utf8(result.roc)),
                ),
            )
        }
    }
}

shutdown! : Server.ShutdownReason, Context => Try({}, [Exit(I64), ..])
shutdown! = |_reason, _context| Ok({})

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

listen_host! : {} => Str
listen_host! = |_| {
    match Env.var_str!("ROC_BASIC_WEBSERVER_HOST") {
        Ok("") => "127.0.0.1"
        Ok(value) => value
        Err(_) => "127.0.0.1"
    }
}
