app [Context, program] {
    pf: platform "platform/main.roc",
    http: "https://github.com/roc-lang/http/releases/download/1.0.0/6ZUwqYhCS8PU9Mo6MF7oV82ET2o7KYb57CLKDq4cq4sS.tar.zst",
}
import pf.Server
import pf.Http
import pf.Env
import pf.Sleep
import http.Request
import http.Response
Context : {}
program = {init!,respond!,shutdown!}
init! = || Ok({config:Server.default_config,context:{}})
respond! = |request,_| {
    path = match request.target() { Resource({raw_path,..}) => raw_path, _ => "/" }
    match path {
        "/method" => Ok(Server.respond(Response.from_status(200).with_body(Str.to_utf8(Str.inspect(request.method())))))
        "/body" => {
            body = request.body().read_all!() ? |error| ServerErr(Str.inspect(error))
            Ok(Server.respond(Response.from_status(200).with_body(body)))
        }
        "/sleep" => {
            Sleep.millis!(100)
            Ok(Server.respond(Response.from_status(204)))
        }
        _ => {
            uri = Env.var_str!("UPSTREAM_URL") ?? "http://127.0.0.1:8799"
            config = Http.default_config.with_max_response_bytes(256 * 1024).with_timeout_millis(100)
            response = Http.send_with!(Request.from_method(GET).with_uri(uri.concat(path)),config) ? |error| ServerErr(Str.inspect(error))
            Ok(Server.respond(response))
        }
    }
}
shutdown! = |_,_| Ok({})
