import Server
import Sse
import Html
import http.Method
import http.Response

Mutation : [Post, Put, Patch, Delete]
Decision : [Hit(Server.Outcome), Miss]

Rocci := [].{
    program = make_program
    view = make_view
    fragment = make_fragment
    dispatch! = dispatch_routes!
    view! = wrap_view
    get_fragment! = wrap_fragment!
    fragment! = wrap_fragment!
    command! = wrap_command!
    events! = wrap_events!
    get_events! = wrap_events!
    unfold! = wrap_unfold
    slash_alternate = alt_slash
    prefix_remainder = remainder_after_prefix
}

make_view = |path, handle!| View({ path: path, handle!: handle! })

make_fragment = |verb, path, handle!| Fragment({ verb: verb, path: path, handle!: handle! })

dispatch_routes! = |routes, request, context| {
    method = Method.to_str(request.method())
    path =
        match request.target() {
            Resource({ raw_path: raw, .. }) => raw
            _ => ""
        }
    match first_hit!(routes, method, path, request, context)? {
        Hit(outcome) => Ok(outcome)
        Miss => html_status(404, "not found")
    }
}

first_hit! = |routes, method, path, request, context|
    match routes {
        [route, .. as rest] =>
            match apply_route!(route, method, path, request, context)? {
                Hit(outcome) => Ok(Hit(outcome))
                Miss => first_hit!(rest, method, path, request, context)
            }
        [] => Ok(Miss)
    }

apply_route! = |route, method, path, request, context|
    match route {
        View({ path: want, handle!, .. }) =>
            if method == "GET" and path == want {
                html = handle!(context, request) ? |err| ServerErr(Str.inspect(err))
                to_hit(wrap_view(html))
            } else {
                Ok(Miss)
            }
        Fragment({ verb, path: want, handle!, .. }) =>
            if method == mutation_str(verb) and path == want {
                html = handle!(context, request) ? |err| ServerErr(Str.inspect(err))
                to_hit(wrap_fragment!(html))
            } else {
                Ok(Miss)
            }
    }

to_hit = |result|
    match result {
        Ok(outcome) => Ok(Hit(outcome))
        Err(err) => Err(err)
    }

mutation_str = |verb|
    match verb {
        Post => "POST"
        Put => "PUT"
        Patch => "PATCH"
        Delete => "DELETE"
    }

html_status = |status, body|
    Ok(
        Server.respond(
            Response.from_status(status)
            .with_headers([{ name: "Content-Type", value: "text/html; charset=utf-8" }])
            .with_body(Str.to_utf8(body)),
        ),
    )

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

wrap_command! = |headers|
    if datastar_request(headers) {
        empty_sse!()
    } else {
        no_content()
    }

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

patch_elements = |node|
    Sse.Event.keyed(
        "datastar-patch-elements",
        "elements",
        Html.render_without_doc_type(node),
    )

patch_html! = |node| {
    event = patch_elements(node)
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

empty_sse! = ||
    Ok(Server.stream(Sse.unfold!(0, |_state| Ok(End))))

no_content = ||
    Ok(Server.respond(Response.from_status(204)))

datastar_request = |headers|
    List.any(
        headers,
        |header|
            (
                header.name == "datastar-request"
                or header.name == "Datastar-Request"
                or header.name == "DATASTAR-REQUEST"
            )
            and (
                header.value == "true"
                or header.value == "True"
                or header.value == "TRUE"
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
