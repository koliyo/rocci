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
import Edit
import Gallery
import Html
import Search
import Signals
import Tabs
import Todos
import Validate

Context : { db : Sqlite.Db }
ContactRow : { first_name : Str, last_name : Str, email : Str, orig_first : Str, orig_last : Str, orig_email : Str, editing : I64 }
IdParams : { id : I64 }
TextParams : { id : I64, text : Str }
FilterParams : { filter : Str }
ContactParams : { first_name : Str, last_name : Str, email : Str }
EditParams : { editing : I64 }
Filter : [All, Pending, Completed]
FieldState : [Empty, Valid, Invalid({ message : Str })]
Field : { value : Str, hint : Str, state : FieldState }

program = { init!, respond!, shutdown! }

init! : () => Try({ config : Server.Config, context : Context }, [Exit(I64), ..])
init! = || {
    db_path =
        match Env.var!("DB_PATH") {
            Ok(path) => Path.from_os_str(path)
            Err(_) => Path.utf8("./gallery.db")
        }
    db = Sqlite.open!(Sqlite.default_config(db_path)) ? |_| Exit(2)
    setup_db!(db) ? |_| Exit(2)

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
    method = Method.to_str(request.method())
    path =
        match request.target() {
            Resource({ raw_path, .. }) => raw_path
            _ => ""
        }

    if method == "GET" and Str.starts_with(path, "/tabs/") {
        tabs_patch!(Str.drop_prefix(path, "/tabs/"))
    } else if method == "POST" and Str.starts_with(path, "/todos/") {
        todo_post!(db, Str.drop_prefix(path, "/todos/"))
    } else if method == "DELETE" and Str.starts_with(path, "/todos/") and path != "/todos/completed" {
        delete_todo!(db, Str.drop_prefix(path, "/todos/"))
    } else {
        match (method, path) {
            ("GET", "/") => html_ok(Html.render(Gallery.home({})))
            ("GET", "/health") => text_ok("ok")
            ("GET", "/search") =>
                html_ok(Html.render(search_page("")))
            ("GET", "/search/results") => {
                json = Signals.from_request!(request) ? |err| ServerErr("Failed to read search: ${Str.inspect(err)}")
                Ok(patch!(Search.results({ contacts: all_contacts, query: Signals.str(json, "search") })))
            }
            ("GET", "/edit") => {
                contact = load_contact!(db) ? |err| ServerErr("Failed to read contact: ${Str.inspect(err)}")
                html_ok(Html.render(edit_page(contact)))
            }
            ("GET", "/edit/contact") => contact_patch!(db)
            ("GET", "/edit/contact/edit") => set_editing!(db, 1)
            ("GET", "/edit/contact/cancel") => set_editing!(db, 0)
            ("PUT", "/edit/contact") => save_contact!(db, request)
            ("PATCH", "/edit/contact/reset") => reset_contact!(db)
            ("GET", "/todos") => {
                view = load_todos!(db) ? |err| ServerErr("Failed to read todos: ${Str.inspect(err)}")
                html_ok(Html.render(todos_page(view)))
            }
            ("PATCH", "/todos") => add_todo!(db, request)
            ("PUT", "/todos/reset") => reset_todos!(db)
            ("PUT", "/todos/mode/all") => set_todo_filter!(db, "all")
            ("PUT", "/todos/mode/pending") => set_todo_filter!(db, "pending")
            ("PUT", "/todos/mode/completed") => set_todo_filter!(db, "completed")
            ("DELETE", "/todos/completed") => clear_completed!(db)
            ("GET", "/tabs") => html_ok(Html.render(tabs_page("0")))
            ("GET", "/validate") => html_ok(Html.render(validate_page(empty_form())))
            ("POST", "/validate/check") => validate_check!(request)
            ("POST", "/validate") => validate_submit!(request)
            _ =>
                Ok(
                    Server.respond(
                        Response.from_status(404)
                        .with_body(Str.to_utf8("Not found")),
                    ),
                )
        }
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

text_ok = |body|
    Ok(
        Server.respond(
            Response.from_status(200)
            .with_headers([{ name: "Content-Type", value: "text/plain; charset=utf-8" }])
            .with_body(Str.to_utf8(body)),
        ),
    )

patch! = |node| {
    event = Datastar.patch_elements(node)
    Server.stream(
        Sse.unfold!(0, |state|
            match state {
                0 => Ok(Emit({ event, state: 1, wake: Immediately }))
                _ => Ok(End)
            }
        ),
    )
}

search_page = |query|
    Gallery.page(
        {
            title: "Active Search",
            eyebrow: "@let, @if, @for",
            lede: "The server filters contacts. Datastar only binds the search box and asks for a new table after you pause typing.",
            current: "search",
        },
        Search.demo({ contacts: all_contacts, query: query }),
    )

edit_page = |contact|
    Gallery.page(
        {
            title: "Click to Edit",
            eyebrow: "@match Viewing | Editing",
            lede: "There is no client form model. Edit and view are two server-rendered trees behind one patch boundary.",
            current: "edit",
        },
        Edit.panel({ mode: contact_mode(contact), contact: sanitize_contact(contact) }),
    )

todos_page = |view|
    Gallery.page(
        {
            title: "TodoMVC",
            eyebrow: "@match, @for, @if",
            lede: "Filters, empty states, and completed rows are Rocci directives. Item URLs are Roc strings interpolated into Datastar actions.",
            current: "todos",
        },
        Todos.demo(view),
    )

tabs_page = |selected|
    Gallery.page(
        {
            title: "Lazy Tabs",
            eyebrow: "HATEOAS + @for",
            lede: "The selected tab is part of the URL. The patch returns the whole tablist and panel.",
            current: "tabs",
        },
        Tabs.demo(tabs_view(selected)),
    )

validate_page = |page|
    Gallery.page(
        {
            title: "Inline Validation",
            eyebrow: "@match Empty | Valid | Invalid",
            lede: "Each field is a Roc tag. Submitting either re-renders errors or replaces the form with a welcome message.",
            current: "validate",
        },
        Validate.demo({ page: page }),
    )

all_contacts = [
    { first: "Carli", last: "Stoltenberg" },
    { first: "Kristina", last: "Yundt" },
    { first: "Kirstin", last: "Okuneva" },
    { first: "Brycen", last: "Cronin" },
    { first: "Philip", last: "Zieme" },
    { first: "Paula", last: "Nikolaus" },
    { first: "Addie", last: "Kshlerin" },
    { first: "Alexandre", last: "Rodriguez" },
    { first: "Maegan", last: "Hudson" },
    { first: "Leta", last: "Welch" },
]

setup_db! = |db| {
    Sqlite.execute!(
        {
            db,
            query: "CREATE TABLE IF NOT EXISTS contact (id INTEGER PRIMARY KEY CHECK (id = 1), first_name TEXT NOT NULL, last_name TEXT NOT NULL, email TEXT NOT NULL, orig_first TEXT NOT NULL, orig_last TEXT NOT NULL, orig_email TEXT NOT NULL, editing INTEGER NOT NULL)",
            params: {},
        },
    )?
    Sqlite.execute!(
        {
            db,
            query: "INSERT OR IGNORE INTO contact (id, first_name, last_name, email, orig_first, orig_last, orig_email, editing) VALUES (1, 'John', 'Doe', 'john@roc-lang.org', 'John', 'Doe', 'john@roc-lang.org', 0)",
            params: {},
        },
    )?
    Sqlite.execute!(
        {
            db,
            query: "CREATE TABLE IF NOT EXISTS todos (id INTEGER PRIMARY KEY, text TEXT NOT NULL, done INTEGER NOT NULL)",
            params: {},
        },
    )?
    Sqlite.execute!(
        {
            db,
            query: "CREATE TABLE IF NOT EXISTS todo_meta (id INTEGER PRIMARY KEY CHECK (id = 1), filter TEXT NOT NULL)",
            params: {},
        },
    )?
    Sqlite.execute!(
        {
            db,
            query: "INSERT OR IGNORE INTO todo_meta (id, filter) VALUES (1, 'all')",
            params: {},
        },
    )?
    seed_todos!(db)
}

seed_todos! = |db| {
    count : { n : I64 }
    count = Sqlite.query!(
        {
            db,
            query: "SELECT COUNT(*) AS n FROM todos",
            params: {},
            limits: Sqlite.default_query_limits,
        },
    )?
    if count.n == 0 {
        insert_todo_row!(db, 1, "Learn Roc", 0)?
        insert_todo_row!(db, 2, "Learn Datastar", 0)?
        insert_todo_row!(db, 3, "Write .rocci views", 0)?
        insert_todo_row!(db, 4, "Profit", 1)
    } else {
        Ok({})
    }
}

insert_todo_row! = |db, id, text, done| {
    params : TextParams
    params = { id: id, text: text }
    done_params : { id : I64, done : I64 }
    done_params = { id: id, done: done }
    Sqlite.execute!(
        {
            db,
            query: "INSERT OR IGNORE INTO todos (id, text, done) VALUES (:id, :text, 0)",
            params,
        },
    )?
    Sqlite.execute!(
        {
            db,
            query: "UPDATE todos SET done = :done WHERE id = :id",
            params: done_params,
        },
    )
}

load_contact! = |db| {
    row : ContactRow
    row = Sqlite.query!(
        {
            db,
            query: "SELECT first_name, last_name, email, orig_first, orig_last, orig_email, editing FROM contact WHERE id = 1",
            params: {},
            limits: Sqlite.default_query_limits,
        },
    )?
    Ok(
        {
            first: row.first_name,
            last: row.last_name,
            email: row.email,
            orig_first: row.orig_first,
            orig_last: row.orig_last,
            orig_email: row.orig_email,
            editing: row.editing,
        },
    )
}

contact_mode = |contact|
    if contact.editing != 0 {
        Editing
    } else {
        Viewing
    }

sanitize_contact = |contact| {
    {
        first: sanitize(contact.first),
        last: sanitize(contact.last),
        email: contact.email,
    }
}

sanitize = |text|
    if Str.contains(text, "heck") {
        "****"
    } else {
        text
    }

contact_patch! = |db| {
    contact = load_contact!(db) ? |err| ServerErr("Failed to read contact: ${Str.inspect(err)}")
    Ok(patch!(Edit.panel({ mode: contact_mode(contact), contact: sanitize_contact(contact) })))
}

set_editing! = |db, editing| {
    params : EditParams
    params = { editing: editing }
    Sqlite.execute!(
        {
            db,
            query: "UPDATE contact SET editing = :editing WHERE id = 1",
            params,
        },
    )
        ? |err| ServerErr("Failed to update contact: ${Str.inspect(err)}")
    contact_patch!(db)
}

save_contact! = |db, request| {
    json = Signals.from_request!(request) ? |err| ServerErr("Failed to read contact: ${Str.inspect(err)}")
    params : ContactParams
    params = {
        first_name: Signals.str(json, "firstName"),
        last_name: Signals.str(json, "lastName"),
        email: Signals.str(json, "email"),
    }
    Sqlite.execute!(
        {
            db,
            query: "UPDATE contact SET first_name = :first_name, last_name = :last_name, email = :email, editing = 0 WHERE id = 1",
            params,
        },
    )
        ? |err| ServerErr("Failed to save contact: ${Str.inspect(err)}")
    contact_patch!(db)
}

reset_contact! = |db| {
    Sqlite.execute!(
        {
            db,
            query: "UPDATE contact SET first_name = orig_first, last_name = orig_last, email = orig_email, editing = 0 WHERE id = 1",
            params: {},
        },
    )
        ? |err| ServerErr("Failed to reset contact: ${Str.inspect(err)}")
    contact_patch!(db)
}

load_todos! = |db| {
    meta : { filter : Str }
    meta = Sqlite.query!(
        {
            db,
            query: "SELECT filter FROM todo_meta WHERE id = 1",
            params: {},
            limits: Sqlite.default_query_limits,
        },
    )?
    rows : List({ id : I64, text : Str, done : I64 })
    rows = Sqlite.query_many!(
        {
            db,
            query: "SELECT id, text, done FROM todos ORDER BY id",
            params: {},
            limits: Sqlite.default_query_limits,
        },
    )?
    filter = parse_filter(meta.filter)
    visible = List.keep_if(rows, |row| todo_visible(row, filter))
    pending = List.len(List.keep_if(rows, |row| row.done == 0)).to_i64_wrap()
    has_completed = List.any(rows, |row| row.done != 0)
    Ok(
        {
            items: List.map(visible, todo_item),
            filter: filter,
            pending_str: pending.to_str(),
            has_completed: has_completed,
        },
    )
}

todo_item = |row| {
    {
        id: row.id.to_str(),
        text: row.text,
        done: row.done != 0,
        row_class: if row.done != 0 {
            "done"
        } else {
            ""
        },
    }
}

todo_visible = |row, filter|
    match filter {
        All => True
        Pending => row.done == 0
        Completed => row.done != 0
    }

parse_filter = |text|
    if text == "pending" {
        Pending
    } else if text == "completed" {
        Completed
    } else {
        All
    }

todos_patch! = |db| {
    view = load_todos!(db) ? |err| ServerErr("Failed to read todos: ${Str.inspect(err)}")
    Ok(patch!(Todos.todosPatch(view)))
}

add_todo! = |db, request| {
    json = Signals.from_request!(request) ? |err| ServerErr("Failed to read todo: ${Str.inspect(err)}")
    text = Str.trim(Signals.str(json, "input"))
    if text == "" {
        todos_patch!(db)
    } else {
        next : { n : I64 }
        next = Sqlite.query!(
            {
                db,
                query: "SELECT COALESCE(MAX(id), 0) + 1 AS n FROM todos",
                params: {},
                limits: Sqlite.default_query_limits,
            },
        )
            ? |err| ServerErr("Failed to add todo: ${Str.inspect(err)}")
        insert_todo_row!(db, next.n, text, 0) ? |err| ServerErr("Failed to add todo: ${Str.inspect(err)}")
        todos_patch!(db)
    }
}

todo_post! = |db, rest| {
    parts = Str.split_on(rest, "/")
    match (List.get(parts, 0), List.get(parts, 1)) {
        (Ok(id), Ok("toggle")) => toggle_todo!(db, id)
        _ =>
            Ok(
                Server.respond(
                    Response.from_status(404)
                    .with_body(Str.to_utf8("Not found")),
                ),
            )
    }
}

toggle_todo! = |db, id_str| {
    match I64.from_str(id_str) {
        Ok(id) => {
            params : IdParams
            params = { id: id }
            Sqlite.execute!(
                {
                    db,
                    query: "UPDATE todos SET done = CASE done WHEN 0 THEN 1 ELSE 0 END WHERE id = :id",
                    params,
                },
            )
                ? |err| ServerErr("Failed to toggle todo: ${Str.inspect(err)}")
            todos_patch!(db)
        }
        Err(_) => todos_patch!(db)
    }
}

delete_todo! = |db, id_str| {
    match I64.from_str(id_str) {
        Ok(id) => {
            params : IdParams
            params = { id: id }
            Sqlite.execute!(
                {
                    db,
                    query: "DELETE FROM todos WHERE id = :id",
                    params,
                },
            )
                ? |err| ServerErr("Failed to delete todo: ${Str.inspect(err)}")
            todos_patch!(db)
        }
        Err(_) => todos_patch!(db)
    }
}

set_todo_filter! = |db, filter| {
    params : FilterParams
    params = { filter: filter }
    Sqlite.execute!(
        {
            db,
            query: "UPDATE todo_meta SET filter = :filter WHERE id = 1",
            params,
        },
    )
        ? |err| ServerErr("Failed to set filter: ${Str.inspect(err)}")
    todos_patch!(db)
}

clear_completed! = |db| {
    Sqlite.execute!(
        {
            db,
            query: "DELETE FROM todos WHERE done != 0",
            params: {},
        },
    )
        ? |err| ServerErr("Failed to clear todos: ${Str.inspect(err)}")
    todos_patch!(db)
}

reset_todos! = |db| {
    Sqlite.execute!({ db, query: "DELETE FROM todos", params: {} }) ? |err| ServerErr("Failed to reset todos: ${Str.inspect(err)}")
    Sqlite.execute!({ db, query: "UPDATE todo_meta SET filter = 'all' WHERE id = 1", params: {} }) ? |err| ServerErr("Failed to reset todos: ${Str.inspect(err)}")
    seed_todos!(db) ? |err| ServerErr("Failed to reset todos: ${Str.inspect(err)}")
    todos_patch!(db)
}

tabs_view = |selected| {
    ids = ["0", "1", "2", "3", "4"]
    tabs = List.map(
        ids,
        |id| {
            {
                id: id,
                label: "Tab ${id}",
                selected: id == selected,
            }
        },
    )
    panel =
        if selected == "0" {
            "Rocci lowers @for to List.map. The selected tab is just another field on that list."
        } else if selected == "1" {
            "Clicking a tab GETs a new fragment. Hypertext carries the selected index; the client does not store it."
        } else if selected == "2" {
            "aria-selected is a Roc string. There is no client tab widget, only buttons that request HTML."
        } else if selected == "3" {
            "Stable id=\"tabs\" is the morph boundary. Datastar replaces the tablist and the panel together."
        } else if selected == "4" {
            "This is the same HATEOAS idea as Click to Edit: the server decides which pane is current."
        } else {
            "Unknown tab."
        }
    { tabs: tabs, panel: panel }
}

tabs_patch! = |selected|
    Ok(patch!(Tabs.demo(tabs_view(selected))))

empty_form = || {
    Form(
        {
            email: field("", "The server checks this on every pause in typing."),
            first: field("", "Any non-empty first name is enough."),
            last: field("", "Any non-empty last name is enough."),
        },
    )
}

field = |value, hint| {
    { value: value, hint: hint, state: field_state(value, hint) }
}

field_state = |value, hint| {
    trimmed = Str.trim(value)
    if trimmed == "" {
        Empty
    } else if hint == "The server checks this on every pause in typing." {
        if Str.contains(trimmed, "@") and Str.contains(trimmed, ".") {
            Valid
        } else {
            Invalid({ message: "Use a full email like ada@roc-lang.org." })
        }
    } else if List.len(Str.to_utf8(trimmed)).to_i64_wrap() < 2 {
        Invalid({ message: "Enter at least two characters." })
    } else {
        Valid
    }
}

read_form! = |request| {
    json = Signals.from_request!(request) ? |err| ServerErr("Failed to read form: ${Str.inspect(err)}")
    Ok(
        Form(
            {
                email: field(Signals.str(json, "email"), "The server checks this on every pause in typing."),
                first: field(Signals.str(json, "firstName"), "Any non-empty first name is enough."),
                last: field(Signals.str(json, "lastName"), "Any non-empty last name is enough."),
            },
        ),
    )
}

validate_check! = |request| {
    page = read_form!(request)?
    Ok(patch!(Validate.demo({ page: page })))
}

validate_submit! = |request| {
    page = read_form!(request)?
    match page {
        Form({ email, first, last }) =>
            match (email.state, first.state, last.state) {
                (Valid, Valid, Valid) =>
                    Ok(patch!(Validate.demo({ page: SignedUp({ name: "${Str.trim(first.value)} ${Str.trim(last.value)}" }) })))
                _ => Ok(patch!(Validate.demo({ page: page })))
            }
        SignedUp(_) => Ok(patch!(Validate.demo({ page: page })))
    }
}
