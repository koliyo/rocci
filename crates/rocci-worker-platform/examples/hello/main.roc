app [Context, program] {
    pf: platform "platform/main.roc",
    http: "https://github.com/roc-lang/http/releases/download/1.0.0/6ZUwqYhCS8PU9Mo6MF7oV82ET2o7KYb57CLKDq4cq4sS.tar.zst",
}

import pf.Server
import http.Response

Context : {}
program = {init!,respond!,shutdown!}
init! = || Ok({config:Server.default_config,context:{}})
respond! = |_,_| Ok(Server.respond(Response.from_status(200).with_body(Str.to_utf8("Hello from ordinary Roc"))))
shutdown! = |_,_| Ok({})
