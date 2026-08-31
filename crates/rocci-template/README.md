# rocci-template

Parse `.rocci` modules and lower explicit components to ordinary Roc.

A `.rocci` file is a Roc module: ordinary declarations stay Roc, and
`@component Name = |params| ...` bodies use a bounded HTML template
grammar. Top-level `@context`, `@init`, and verb-first `@method:role(path)`
routes declare standalone HTTP apps for `rocci run`. This crate does not
invoke the Roc compiler, type-check expressions, or spawn servers.

```sh
cargo run -p rocci-template -- build path/to/file.rocci
cargo run -p rocci-template -- ast path/to/file.rocci
cargo run -p rocci-template -- inspect --ast path/to/file.rocci
```

`build` writes generated Roc to stdout, or to a file with `-o`. `ast` prints
the parse tree as an S-expression. `inspect` prints components, fixtures, tests,
generated Roc, and source-map segments; `--ast` includes the parse tree. The
same commands exist on the workspace `rocci` CLI. Input `-` reads stdin.

Library entry points are `parse`, `lower`, `compile`, and `format_ast` in
`rocci_template`.

## File shape

A file may mix a module header, imports, types, helpers, and any number of
components:

```rocci
module CounterPage exposing [hello]

import pf.Html
import Design

Tone : [Neutral, Positive]

@component Hello = |{ name }|
    <p>Hello, {name}</p>

badgeClass = |tone| {
    match tone {
        Neutral => "badge"
        Positive => "badge badge--positive"
    }
}
```

Everything outside an `@component` body is copied into the generated Roc
module unchanged. `@component`, `@fixture`, `@test`, `@css`, `@context`, `@init`, and
`@method:role` routes are recognized only at the start of a top-level definition.

## Components

```text
@component Name = |params| html
@component Name = |params| { template }
```

A body that is one HTML tag, component call, or fragment does not need braces.
`@let`, `@css`, `@if` / `@for` / `@match`, and multiple root items still use
`{ ... }`. Nested markup inside that one tag is fine.

Component names are PascalCase, matching the tags that call them. Roc values
cannot start with an uppercase letter, so lowering emits the corresponding
camelCase function. Write `@component Hello` and `<Hello />`; the generated
module binds `hello`. Ordinary Roc in the same file—`exposing` lists, handlers,
helpers—uses that camelCase name, because those regions are Roc:

```rocci
module CounterPage exposing [hello]

@get:view("/") = |{}| {
    hello({ name: "Roc" })
}

@component Hello = |{ name }|
    <p>Hello, {name}</p>
```

Write `<HtmlShell>` / `@component HtmlShell` for a value named `htmlShell`.
Consecutive leading capitals such as `HTMLShell` are rejected as ambiguous.

`params` is a Roc parameter list. The first parameter is normally a props
record. Extra parameters are the default body:

```rocci
@component Badge = |{ tone }, content|
    <span class={badgeClass(tone)}>
        {content}
    </span>
```

`??` field defaults are allowed in `.rocci` (`|{ name ?? "Roc" }|`). Lowering
keeps a stripped pattern (`|{ name }|`) because Roc still rejects `??` in
patterns, and emits a type annotation `{ name : Str ?? "Roc" }` so omitted
fields materialize at construction. String and integer defaults infer a type;
Bool defaults stay filled at call sites (`True`/`False` in a type annotation
crashes this Roc nightly at runtime). Tag defaults need an authored type
(`tone : Tone ?? Neutral`).

```roc
hello : { name : Str ?? "Roc" } -> Html
hello = |{ name }| { … }
```

A self-closing component tag becomes a one-argument call. A paired tag
passes the nested markup as a second argument:

```rocci
<Hello name={person.name} />
<Badge tone={Positive}>Current count</Badge>
```

`rocci view --component Hello` and `--component hello` both select that
function.

```roc
hello({ name: person.name })
badge({ tone: Positive }, Html.text("Current count"))
```

There is no magic `children` field. Named regions are ordinary `Html` or
function-valued props.

## Fixtures

`@fixture` tags a Roc binding as sample input for a component. The marker is
stripped; the binding stays ordinary Roc. Use it for tests, demos, and the
component browser's fixture picker.

```text
@fixture{target: componentPath}
name = rocExpr
```

`target` is a component path: a local PascalCase name (`TodoItem`) or a
module-qualified name (`Search.Results`). The last segment is the component;
it lowers to camelCase (`todoItem`, `Search.results`). The value is one
delimiter-balanced Roc expression, usually a props record:

```rocci
@fixture{target: TodoItem}
todoItemTest = { item: { id: 123, text: "Buy milk" } }

@fixture{target: Search.Results}
searchResultTest = { contacts: all_contacts, query: "Foo" }
```

This lowers to:

```roc
todoItemTest = { item: { id: 123, text: "Buy milk" } }

searchResultTest = { contacts: all_contacts, query: "Foo" }
```

`compile` reports each fixture as `FixtureInfo` (`name`, `target`, `value`).
Unqualified targets must name a `@component` in the same file. Dotted targets
are left for the Roc compiler and later project-level tools.

`@fixture` is not a template directive. Inside a component body it is an error.

## Tests

`@test` tags a boolean Roc expression. The marker is stripped from generated
Roc; `compile` reports each test as `TestInfo` (`name`, optional `fixture`,
`expr`). `rocci test` appends `expect` after the type-module wrap. Optional
`{fixture: ident}` must name a local `@fixture`.

```rocci
@test helloNamePresent = Bool.true

@test{fixture: helloSample}
helloFixturePaired = Str.contains(helloSample.name, "Ada")
```

`@test` is not a template directive. Inside a component body it is an error.

## Standalone HTTP

Top-level `@context`, `@init`, and `@method:role("literal-path")` routes
declare app state and HTTP handlers for `rocci run File.rocci`. Bodies are
Roc, not templates. Role-first forms, `@on`, trailing `json`, and pathless
`@live` are removed; old source never lowers. The generated dispatcher (not
this crate) maps handlers onto basic-webserver: authors never write
`Context`, `ServerErr`, `Exit`, or `respond!`.

```rocci
@context { db : Sqlite.Db }

@init {
    db = Sqlite.open!(Sqlite.default_config(Path.utf8("./app.db")))?
    { db: db }
}

@get:view("/") = |{ db }| {
    count = read_count!(db)?
    page({ count })
}

@get:live("/sse") = |{ db }| {
    count = read_count!(db)?
    card({ count })
}

@post:command("/actions/increment") = |{ db }| {
    increment_count!(db)?
    {}
}

@post:fragment("/actions/reset") = |{ db }| {
    count = reset_count!(db)?
    card({ count })
}
```

- `@context { ... }` lowers to `State : { ... }` on the module. Generated
  `main.roc` uses `Context : Module.State`.
- `@init { ... }` lowers to `init!`, wrapping the block so `?` works. The
  generated app maps failures to process exit.
- Every ordinary route spells one HTTP method and one response role. The
  closed matrix is: GET admits `view`, `fragment`, and `live`; POST, PUT,
  PATCH, and DELETE admit `fragment` and `command`. Illegal pairs are
  rejected. There are no default methods.
- `@get:live("literal-path")` lowers to a named live handler. Modules may
  declare several live routes; each becomes one SSE endpoint. The CLI
  dispatcher polls (`After(100)`), emits `datastar-patch-elements` when
  `Html.render` bytes change, `Wait`s on other unchanged polls, and emits
  a non-Datastar keepalive (`Sse.Event.data("")`) every 15s of quiet so
  the host 30s response-idle timeout cannot kill the stream. A document receives automatic
  `data-init=@get(path, [OpenWhenHidden(True)])` only when its module has
  exactly one live route and the body has no authored `data-init`. Multiple
  local streams require explicit subscriptions. App-wide method/path
  collisions, including sibling modules, are diagnostics.
- `@get:view(path)` returns an HTML document. `@method:fragment(path)`
  returns an HTML fragment encoded as a one-shot `datastar-patch-elements`
  event. `@method:command(path)` returns `{}` and has no success
  representation: Datastar (`Datastar-Request: true`) gets **empty SSE**
  (HTTP 200 `text/event-stream` that ends immediately); an ordinary client
  gets **204 No Content**. JSON resources and mixed responses belong in
  authored `main.roc`. `Datastar.patch_signals` is a low-level Roc helper,
  not a route role.
- Generated functions keep names such as `on_get_root!` and
  `on_post_actions_increment!`. Dispatch calls `handler!(context, request)`.
  Write `|{ db }, request|` when the handler reads the request. A one-parameter
  list such as `|{ db }|` or `|_|` lowers with an unused `_request` appended.
  Omit the parameter list to get `|state, _request|`. Generated `respond!`
  maps `?` failures to HTTP 500 (HTML overlay for Datastar; a small ordinary
  error otherwise). Bodies may call platform effects such as
  `pf.Stderr.line!`; under `rocci run` those lines are teed to the CLI and Dev
  Console. Do not print from `@component`.
- `rocci view` / `rocci browse` ignore these directives and render fixtures.

| Kind | Declaration | Body | Response |
| --- | --- | --- | --- |
| HTML document | `@get:view(path)` | Full `<html>` | `text/html` |
| One-shot fragment | `@get:fragment` or mutation `@method:fragment` | Fragment with a stable `id` | `datastar-patch-elements` |
| Command | mutation `@method:command` | `{}` | **Empty SSE** for Datastar; **204** otherwise |
| Live stream | `@get:live(path)` | Fragment with a stable `id` | Long-lived SSE at that path |

Former headers are a removal, not a deprecation. Diagnostics rewrite:

- `@view(path)` or `@on:get(path)` → `@get:view(path)`
- `@patch(path)` or `@on:post(path)` → `@post:fragment(path)`
- `@patch:put(path)` → `@put:fragment(path)`
- `@patch:patch(path)` → `@patch:fragment(path)`
- `@patch:delete(path)` → `@delete:fragment(path)`
- `@command(path)` → `@post:command(path)`
- `@command:put(path)` → `@put:command(path)`
- `@command:patch(path)` → `@patch:command(path)`
- `@command:delete(path)` or `@on:delete(path) json` → `@delete:command(path)`
- `@live` / `@live(path)` → `@get:live("/sse")` or another authored path

Handler return alternatives not taken: returning `Server.Outcome` from the
body, or branching on `Html` vs `Sse.Event`. A custom unfold can still live in
an authored `main.roc`.

`@context` / `@init` / `@method:role` routes are module-level only.

## Tags

| Form | Meaning |
| --- | --- |
| `<div>`, `<output>` | HTML element (lowercase) |
| `<Hello />`, `<CounterCard>` | Component call; resolves to `hello`, `counterCard` |
| `<Design.Button />` | Qualified call; resolves to `Design.button` |
| `<>...</>` | Fragment; lowers to `Html.fragment(...)` |
| `<br>`, `<img />` | Void HTML elements use `Html.void_element` |

Write `<HtmlShell>` / `@component HtmlShell` for a value named `htmlShell`.
Consecutive leading capitals such as `<HTMLShell>` are rejected as ambiguous.

Unknown PascalCase tags still lower to a call. The Roc compiler reports
missing values later.

## Attributes

```rocci
<section id="counter" class="counter-card">
<a href={person.url} class={if selected { "selected" } else { "" }}>
<button disabled>
```

- `name="..."` is a static string.
- `name={expr}` is a Roc expression.
- `name=@post("/path")` is a Datastar backend action. Arguments are Roc; this lowers to `Datastar.post("/path")`.
- A valueless name is a boolean attribute (`Html.boolean_attribute`).

On HTML elements these become `Html.attribute` / `Html.boolean_attribute`.
On component tags they become a props record. `count={count}` is emitted as
`{ count: count }`.

Attribute names may include hyphens (`aria-current`, `data-on-click`).
Dynamic attribute names are not supported.

### Datastar actions

`@get`, `@post`, `@put`, `@patch`, and `@delete` are Rocci keywords in
attribute position. They generate Datastar backend actions. Arguments are Roc
(`"/path"`, not `'/path'`):

```rocci
<button data-on:click=@post("/actions/quiz")>
    Submit answer
</button>

<button data-on:click=@delete("/actions/todos/${item.id}")>
    Delete
</button>

<body data-init=@get("/sse", [OpenWhenHidden(True)])>
```

This is not Datastar JS. `@post('/x')` with single quotes is a parse error;
write `@post("/x")`, or quote a literal Datastar expression:

```rocci
<form data-on:submit__prevent="$input.trim() && @patch('/actions/todos') && ($input = '')">
```

Quoted `"@post('...')"` stays an opaque client string. Custom actions such as
`@peek` stay quoted. Using an action injects `import Datastar` when the module
does not already import it. The `Datastar` helpers return the HTML attribute
string, including JS quoting of the URI.

A second argument is a `List` of option tags (`OpenWhenHidden`, `ContentType`,
`Header`, `Retry`, `RequestCancellation`) and lowers to `Datastar.get_with`.
That list is the authored option surface; it is not a JS-style options object.

## Interpolation

`{expr}` in text or attribute position is a Roc expression. Markup is not
allowed inside it:

```rocci
<p>Hello, {name}</p>
<span class={badgeClass(if active { Positive } else { Neutral })}>
```

Text interpolations lower to `Html.text(expr)` and must be `Str`. Convert
numbers at the interpolation site (`{count.to_str()}`), not at the caller.
A bare identifier that matches a body parameter is inserted as `Html` as-is
(`{content}` above).

Use a directive when the branches themselves are markup:

```rocci
@if active {
    <ActiveIcon />
} @else {
    <IdleIcon />
}
```

## Directives

Template structure uses `@` prefixes. The first `{` at Roc delimiter depth
zero opens the body and is not part of the header expression.

### `@if`

```rocci
@if user.isSignedIn {
    <LogoutButton />
} @else if user.canRegister {
    <RegisterButton />
} @else {
    <LoginButton />
}
```

`@else` is optional. A missing else branch lowers to `Html.empty`.

### `@for`

```rocci
@for item in items {
    <TodoRow item={item} />
}
```

The binder is one lowercase identifier. This lowers to `List.map`. As the
only child of an element, the map is used directly as the children list.

### `@match`

```rocci
@match state {
    Loading => <Spinner />
    Failed({ message }) => <ErrorNotice message={message} />
    Ready(items) if !List.isEmpty(items) => <ItemList items={items} />
    Ready(_) => <>
        <Heading text="Ready" />
        <Design.Button tone={if active { Positive } else { Neutral }} />
    </>
}
```

Each arm is `Pattern =>` one self-delimiting value: an element, component
call, fragment, interpolation, or nested directive. Bare text is not an arm
result; wrap it in an element or fragment. Patterns, guards, and `|`
alternatives are captured as Roc tokens and checked by Roc.

### `@let`

```rocci
@let visible = List.keepIf(items, |item| matches(item, query))
```

The binder is one identifier. The expression ends at a depth-zero newline
(continuation is allowed inside `(...)` or `[...]`). `@let` must appear
before render-producing items in the current block.

### `@css`

Colocate isolated CSS at file scope or at the start of a component body:

```rocci
@css {
    .card { padding: 1rem; }
}

@component Hello = |{ name }| {
    @css {
        .greeting { color: navy; }
        p { margin: 0; }
    }
    <p class="greeting">Hello, {name}</p>
}
```

The block body is raw CSS, not the template grammar. No interpolations. Multiple
blocks at the same scope concatenate in source order. `@css` is a preamble item
like `@let`: only before render-producing items, and only at the component
body's top level.

Isolation is Vue-style. Authors keep writing `class="card"`. Lowering stamps
`data-rocci-css` on intrinsic HTML elements authored in that component (not on
`<Child />` calls) and wraps each block in `@scope ([data-rocci-css~="id"])`.
File-level rules share one file id across every component in the module.
Component-level rules use a per-component id. Both ids hash the file basename
so a snapshot stylesheet compiled as `LiveCounterUi.rocci` still matches
`@get:live` HTML compiled from an absolute path to the same file. Descendant selectors can still
match child-component internals; there is no `:deep` yet.

v1 injects a `<style>` element so `view` / `browse` work immediately, and also
returns scoped CSS on `CompileOutput.styles`. When the component root
is an `<html>` document, that style is placed in `<head>` so the browser applies
page chrome. Other components wrap a sibling `<style>` around the markup. SSE
patches should not carry `<style>`. The intended delivery is: keep `@css`
colocated, extract once, and link or inline the stylesheet in the document
`<head>`. For extracted simple rules, lowering retains native `@scope` and
also emits equivalent attribute-prefixed selectors for linked-stylesheet
compatibility; rules containing nested at-rules keep their native form. Until
that follow-up, prefer file-level `@css` on page modules or
shared `app.css` for patch components. `@scope` does not restyle the document
element from an `html { ... }` rule; put document chrome on `body` or `:scope`.

## Roc regions

Directive headers, `{...}` interpolations, `@match` patterns, `@let`
expressions, and component parameter lists are Roc token ranges. Nested
records, lists, closures, strings, comments, `if`, `match`, and comparisons
are legal there. The template parser does not type-check them.

A top-level `{` ends a directive header, so unparenthesized records are
invalid:

```rocci
# Invalid: `{` opens the match body.
@match { status, items } { ... }

# Valid: the record is inside parentheses or a call.
@match ({ status, items }) { ... }
@if isVisible({ user, permissions }) { ... }
```

The body `{` must be on the same logical header line. A depth-zero newline
before it is an error.

Markup does not nest inside Roc expressions. `{if active { <Icon /> }}` is
rejected by this grammar; use `@if` or call a named render function.

## Comments and escaping

Inside a template body:

- `#` at the start of an item is a line comment.
- `<!-- ... -->` is an HTML comment and is dropped.
- `@@` emits a literal `@` (`@@if` is the text `@if`, not a directive).

Immediately above a top-level `@` declaration, Roc `#` line comments and `## `
doc comments attach to that declaration and lower onto the generated Roc
assignment (no blank line between the last `## ` line and `@`). A blank line, or
a non-doc `#` line between `## ` lines and `@`, leaves those comments in the
opaque Roc region instead. Ordinary Roc `#` comments also remain valid inside
Roc regions.

## Whitespace

Indentation-only text (newlines plus leading spaces) around tags and
directive blocks is discarded. Inline spaces such as `Hello, {name}` are
kept. Adjacent tags do not get synthesized extra spaces.

## Generated Roc

Lowering targets a small `Html` constructor set:

| Template | Generated |
| --- | --- |
| `<p>...</p>` | `Html.element("p", attrs, children)` |
| void element | `Html.void_element("br", attrs)` |
| static / dynamic attr | `Html.attribute("class", ...)` |
| valueless attr | `Html.boolean_attribute("disabled", True)` |
| text | `Html.text(...)` |
| `<>...</>` | `Html.fragment(...)` |
| missing `@else` | `Html.empty` |
| `@for` | `List.map` |
| `<Hello name={x} />` | `hello({ name: x })` |

Constructors are emitted on the `Html` module name (configurable via
`LowerOptions::html_module`). Generated names and formatting are
deterministic for a given source. Segment maps record which generated
ranges came from ordinary Roc, signatures, tags, interpolations, directives,
or scaffolding.

See [`../../test/AllSyntax.rocci`](../../test/AllSyntax.rocci) and the matching
[`tests/fixtures/all_syntax.roc`](tests/fixtures/all_syntax.roc) for a complete
example.

## Tree spec

The owned parse-tree shape lives in [`Rocci.AST.ungram`](Rocci.AST.ungram).
`cargo run -q -p rocci-ungram -- generate` writes
[`src/ast.generated.rs`](src/ast.generated.rs) and exhaustive inspect walkers in
[`src/pprint.generated.rs`](src/pprint.generated.rs). The generator emits node types
and `format_ast` matches only; it does not produce the scanner or parser. Those stay
hand-written in this crate. `pprint.rs` owns `Writer` and atom policy. `cargo run -q -p rocci-ungram -- check` fails when the committed generated
file is stale or a generated production has no inspect mapping. Inspect tags live
in [`Rocci.AST.toml`](Rocci.AST.toml) and the public
[`docs/reference/contributor/rocci-tree.rocdown`](../../docs/reference/contributor/rocci-tree.rocdown)
appendix. This README remains the language contract;
the ungram is the developer tree spec, not a substitute for the syntax above.

## AST dump

`format_ast` (and the `ast` command) prints the parse tree as indented
S-expressions. Bare atoms are identifiers and dotted paths; everything else
is a double-quoted string. Roc snippets that are not a simple path are
quoted. Ordinary Roc between components is shown as `(roc ...)` lines.

```rocci
@component Hello = |{ name }|
    <p class="greeting">Hello, {name}</p>
```

```lisp
(module
  (component Hello
    (params "|{ name }|")
    (element p
      (attr class "greeting")
      (text "Hello, ")
      (interp name))))
```

Heads are `module`, `roc`, `component`, `params`, `element`, `call`,
`fragment`, `text`, `interp`, `attr`, `if`, `else-if`, `else`, `for`,
`match`, `arm`, `let`, `css`, `context`, `init`, `view`, `command`, and
`live`. Self-closing tags include the `self-closing` atom after the tag name.

## Not in this crate

- Invoking `roc check` / `roc build`, or remapping Roc type errors
- Full JSX (markup as an arbitrary Roc expression)
- Tagged `` html`...` `` literals
- Dynamic tags, prop spreading, or a runtime component registry
- Spawning HTTP servers, Datastar JS, file watching, or process management

Route metadata from `@method:role` declarations is emitted for the CLI. Dispatch lives in generated
`main.roc`, not this crate.

This README is the current language contract.
