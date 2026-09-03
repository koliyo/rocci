app [Context, program] {
    pf: platform "../platform/main.roc",
    http: "https://github.com/roc-lang/http/releases/download/1.0.0/6ZUwqYhCS8PU9Mo6MF7oV82ET2o7KYb57CLKDq4cq4sS.tar.zst",
}

import pf.Datastar
import pf.Html
import pf.Server
import pf.Sse
import http.Response

Context : {}

program = { init!, respond!, shutdown! }

init! : () => Try({ config : Server.Config, context : Context }, [Exit(I64), ..])
init! = || Ok({ config: Server.default_config, context: {} })

respond! : Server.Request, Context => Try(Server.Outcome, [ServerErr(Str), ..])
respond! = |_request, _context| {
    event = Datastar.patch_elements(Html.p([], [Html.text("hello-datastar")]))
    Ok(
        Server.stream(
            Sse.unfold!(0, |state|
                match state {
                    0 => Ok(Emit({ event, state: 1, wake: Immediately }))
                    _ => Ok(End)
                }
            ),
        ),
    )
}

shutdown! : Server.ShutdownReason, Context => Try({}, [Exit(I64), ..])
shutdown! = |_reason, _context| Ok({})
