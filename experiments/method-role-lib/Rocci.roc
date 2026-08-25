import pf.Server
import pf.Sse
import http.Response
import Datastar
import Html

## Wire wraps for the closed method-role matrix. Constructors that take
## handlers (`Rocci.view("/", home!)`) currently SIGSEGV this Roc compiler
## when the capture lives in a sibling module or a `List`.

Rocci := [].{
    program = make_program
    view! = wrap_view
    get_fragment! = wrap_fragment!
    fragment! = wrap_fragment!
    events! = wrap_events!
    get_events! = wrap_events!
    unfold! = wrap_unfold
    slash_alternate = alt_slash
    prefix_remainder = remainder_after_prefix
}

make_program = |{ init!, respond! }| {
    init!,
    respond!,
    shutdown!,
}

shutdown! = |_reason, _context| Ok({})

wrap_view = |html|
    html_ok(Html.render(html))

wrap_fragment! = |html|
    Ok(patch_html!(html))

wrap_events! = |sse_list|
    Ok(emit_events!(sse_list))

wrap_unfold = |stream|
    Ok(Server.stream(stream))

alt_slash = |path|
    if path == "/" {
        Err({})
    } else if Str.ends_with(path, "/") {
        Ok(Str.from_utf8_lossy(List.drop_last(Str.to_utf8(path), 1)))
    } else {
        Ok("${path}/")
    }

remainder_after_prefix = |path, prefix|
    if Str.starts_with(path, prefix) {
        Ok(Str.drop_prefix(path, prefix))
    } else {
        Err({})
    }

patch_html! = |node| {
    event = Datastar.patch_elements(node)
    Server.stream(
        Sse.unfold!(
            0,
            |state|
                match state {
                    0 => Ok(Emit({ event, state: 1, wake: Immediately }))
                    _ => Ok(End)
                },
        ),
    )
}

emit_events! = |sse_list|
    Server.stream(
        Sse.unfold!(
            sse_list,
            |pending|
                match pending {
                    [event, .. as rest] => Ok(Emit({ event, state: rest, wake: Immediately }))
                    [] => Ok(End)
                },
        ),
    )

html_ok = |body|
    Ok(
        Server.respond(
            Response.from_status(200)
            .with_headers([{ name: "Content-Type", value: "text/html; charset=utf-8" }])
            .with_body(Str.to_utf8(body)),
        ),
    )
