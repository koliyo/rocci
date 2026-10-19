app [Context, program] {
    pf: platform "platform/main.roc",
    http: "https://github.com/roc-lang/http/releases/download/1.0.0/6ZUwqYhCS8PU9Mo6MF7oV82ET2o7KYb57CLKDq4cq4sS.tar.zst",
}

import pf.Server
import http.Response
import pf.Http
import pf.Env
import http.Request

Context : {}
program = {init!,respond!,shutdown!}
init! = || Ok({config:Server.default_config,context:{}})
respond! = |_,_| {
    uri = Env.var_str!("UPSTREAM_URL") ?? "https://example.com/"
    config = Http.default_config.with_max_response_bytes(256 * 1024).with_timeout_millis(5000)
    response = Http.send_with!(Request.from_method(GET).with_uri(uri),config) ? |error| ServerErr(Str.inspect(error))
    Ok(Server.respond(response))
}
shutdown! = |_,_| Ok({})
