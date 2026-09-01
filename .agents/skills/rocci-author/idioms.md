# Rocci, Rocdown, and Roc idioms

Style and best practices for code written **in** the languages. Syntax details live in
`docs/reference/rocci.rocdown`, `docs/reference/rocdown.rocdown`, and the
owning crate READMEs.

## Directory and Module Organization
In Rocci projects, structure directories according to the semantic role of each file:

| Directory | Primary Role | Examples | Key Constructs |
| --- | --- | --- | --- |
| **`components/`** | Reusable UI widgets and design primitives | `Button.rocci`, `StatusCard.rocci`, `NavList.rocci` | `@component`, `@fixture`, scoped `@css` |
| **`theme/`** or **`layouts/`** | Document frames, site chrome, and responsive shells | `SiteShell.rocci`, `Layouts.rocci`, `RocdownTheme.rocci` | `@component Layout = \|{ view }, content\|`, global CSS tokens |
| **`pages/`** or **`routes/`** (or app root) | Standalone full-page web applications and HTTP route handlers | `Counter.rocci`, `LiveCounter.rocci` | `@context`, `@init`, `@get:view`, `@method:fragment` / `@method:command`, `@get:live` |
| **`backend/`** | Handler-only module plus ordinary Roc rules | `examples/rocci/standalone/blocks/backend/` | `@context`, `@init`, routes, SQLite; no `@component` |
| **`ui/`** | Pure render for a nested standalone app | `examples/rocci/standalone/blocks/ui/` | `@component`, `@fixture`, `@css`; no I/O |
| **`docs/`** | Documentation pages and guides | `overview.rocdown`, `quickstart.rocdown` | Markdown, `@page`, `:note`, `:img` |

### Why `components/` instead of `templates/`

- **Language identity**: Rocci defines components via `@component Name = |props| ...` and instantiates them with `<Name />` tags. Calling reusable UI directories `components/` matches the syntax and frontend mental model.
- **Avoid template misconceptions**: The word "templates" often connotes untyped string interpolation (Jinja, EJS, Handlebars). Rocci `.rocci` files are type-safe, compiled Roc modules with scoped CSS and fixture metadata.
- **Distinction from layouts & pages**: Not every `.rocci` file is a widget. Using `components/` for reusable widgets, `layouts/` for chrome frames, and `pages/` for full-page apps prevents dumping unrelated concerns into a single catch-all folder.
- **Crate internal exception**: `crates/*/templates/` is strictly an internal Rust convention for static assets compiled into binaries via `include_str!`. User projects and site code should use `components/`, `theme/`, or `layouts/`.

### Co-location vs Extraction

1. **Co-locate private sub-components**: If a small sub-component is only used within a single page or parent component, declare it directly in that same `.rocci` file (e.g. `@component ItemRow` inside `TodoList.rocci`).
2. **Extract shared components to `components/`**: When a component is reused across multiple pages or designed as part of a design system, move it to `components/` (e.g. `components/Button.rocci`).
3. **Explore with `rocci browse`**: Run `cargo run -p rocci-cli -- browse components/` to discover, test, and interactively preview all components and fixtures in the component directory.

### Nested standalone apps

Flat apps (live-counter) keep `LiveCounter.rocci` and `LiveCounterUi.rocci` as
siblings. Nested apps (`examples/rocci/standalone/blocks/`) put I/O in
`backend/` and markup in `ui/`. Discovery walks up from the entry `.rocci` to
the nearest app `rocci.toml` and recurses; stop at `.git` or a Cargo workspace
root so the repository `rocci.toml` is not treated as an app.

Quoted `data-on:keydown__window="… && @post('/actions/…')"` is the exception
when the handler must inspect `evt.key`. Unquoted `@post("/path")` remains the
default for buttons.

## Prefer match over chained if/else

When the input is a tag union, a closed set of strings, or several discrete
cases, write `match` (Roc) or `@match` (markup). Chained `if` / `else if` is
for a boolean, not for a discriminator.

**Roc — do this**

```roc
status_label = |status|
    match status {
        Ready => "Ready"
        Working => "Working"
        Failed => "Failed"
    }
```

**Roc — not this**

```roc
status_label = |status|
    if status == Ready {
        "Ready"
    } else if status == Working {
        "Working"
    } else {
        "Failed"
    }
```

**Rocci markup — do this**

```rocci
@match mode {
    Viewing => <ContactCard contact={contact} />
    Editing => <ContactForm contact={contact} />
}

@match view.layout {
    "home" => <Layouts.Home view={view}>{content}</Layouts.Home>
    "product" => <Layouts.Product view={view}>{content}</Layouts.Product>
    "section" => <Layouts.Section view={view}>{content}</Layouts.Section>
    _ => <Layouts.Docs view={view}>{content}</Layouts.Docs>
}
```

Each `@match` arm is one self-delimiting value: an element, component call,
fragment, interpolation, or nested directive. Wrap bare text in an element or
`<>...</>`.

**Boolean conditions still use if**

```rocci
@if List.is_empty(items) {
    <EmptyState />
} @else {
    <ul>
        @for item in items {
            <TodoRow item={item} />
        }
    </ul>
}

<a class={if selected { "selected" } else { "" }}>
```

```roc
db_path =
    match Env.var!("DB_PATH") {
        Ok(path) => Path.from_os_str(path)
        Err(_) => Path.utf8("./app.db")
    }
```

`Result`, `List.get`, and similar APIs are tag unions — match them. A missing
optional string (`view.description != ""`) is a boolean — use `@if`.

Combine when useful: `@match` on state, `@if` inside an arm for emptiness.

```rocci
@match state {
    Loading => <Spinner />
    Failed({ message }) => <ErrorNotice message={message} />
    Ready(items) if !List.is_empty(items) => <ItemList items={items} />
    Ready(_) => <EmptyState />
}
```

Parenthesize a record scrutinee so `{` does not open the match body:

```rocci
@match ({ status, items }) {
    { status: Loading } => <Spinner />
    { status: Ready } => <ItemList items={items} />
}
```

## Naming

| Kind | Convention | Example |
| --- | --- | --- |
| `@component` / tag | PascalCase | `StatusCard`, `<StatusCard />` |
| Lowered component value | camelCase | `statusCard` in `@get:view` and `exposing` |
| Ordinary Roc helper / field | snake_case | `read_count!`, `has_completed` |
| Type / tag union payload | PascalCase | `Status : [Ready, Working, Failed]` |
| Effectful function | `snake_case!` | `write_page!`, `from_request!` |

Call the component from Roc with the lowered name:

```rocci
@get:view("/") = |{ db }| {
    count = read_count!(db)?
    counterPage({ count })
}

@component CounterPage = |{ count }|
    <html>...</html>
```

Do not copy `test/AllSyntax.rocci` identifiers such as `List.isEmpty` or
`Num.toStr`. Current examples and the pinned nightly use `List.is_empty`,
`count.to_str()`, `Str.split_on`, `I64.from_str`.

Booleans: `True` / `False` in Roc expressions, `@page`, and `:kind[params]`.
Typed integers: `3.I64`.

## Types on the pinned nightly

Product Roc is `nightly-2026-08-23-fb208ba` (`docs/inventory.toml`,
`docker/install-roc.sh`). Write types the way that compiler parses them,
not the way older Roc or Elm juxtaposition looked.

| Form | Use |
| --- | --- |
| `Foo : { title : Str }` | Structural alias. Default for view records. |
| `Foo := { …, children : List(Foo) }` | Nominal. **Required** when the type refers to itself. |
| `Foo :: { key : Str }.{ … }` | Opaque. Hide fields; do not start here. |
| `List(Str)`, `Page(a)`, `Page(_)` | Type application. Parentheses are required. |
| `Type.{ field: value }` | Nominal constructor. A matching `{ … }` also unifies when the expected type is known. |
| `->` | Pure function (components, wasm render). |
| `=>` | Effectful function (`write_page!`, `@init`). |
| `=> Try({}, [..])` | Effect that returns `Ok({})`. Matches custom app `main!`. |
| `{ name : Str ?? "Roc" }` | Defaulted field on a **type**. Pattern `|{ name ?? "Roc" }|` is still illegal. |

**Do this**

```roc
Page(a) : {
    output_path : Str,
    segments : List(a),
    view : PageView,
}

NavGroupView := {
    title : Str,
    href : Str,
    open : Bool,
    items : List(NavItemView),
    children : List(NavGroupView),
}

write_page! : Str, Views.Page(_) => Try({}, [..])
```

**Not this**

```roc
Page a : { segments : List a }          # juxtaposition does not parse
write_page! : Str, Views.Page _ => _    # same, plus a hole at the export
NavGroup : { children : List(NavGroup) } # recursive alias is illegal
note : [Some Str, None]                 # tag payloads need parentheses
```

Leave `|group|` / `|item|` inferred inside `NavList` and theme shells.
Name the contract once at the module edge (`Views.roc`, generated
`pages : List(Views.Page(_))`, `write_page!`). Apply chrome field names
match `crates/rocci-ui/src/view.rs`.

## Markup vs Roc expressions

Use directives when **branches are markup**. Use Roc inside text and
attributes.

```rocci
# Markup branches
@if active {
    <ActiveIcon />
} @else {
    <IdleIcon />
}

# Roc in an attribute or interpolation
<span class={badge_class(if active { Positive } else { Neutral })}>
<p>{count.to_str()}</p>
```

Invalid: `{if active { <Icon /> }}`. Markup does not nest inside `{...}`.

`@let` binds a Roc value before render items in the current block:

```rocci
@let visible = List.keep_if(items, |item| matches(item, query))
```

## Components

```rocci
@component Badge = |{ tone : Tone ?? Neutral }, content|
    <span class={badge_class(tone)}>{content}</span>

@component Dashboard = |{}|
    <main>
        <Badge tone={Positive}>Current count</Badge>
        <Hello name={person.name} />
    </main>
```

- Self-closing tags are one-argument calls. Paired tags pass nested markup as
  the extra `Html` argument.
- Layouts and shells take the article as that body argument:

```rocci
@component Docs = |{ view }, content| {
    <main id="main-content">
        <article class="article">{content}</article>
    </main>
}
```

Putting `content` in the props record escapes the article as text.

- `@css` is preamble: only before render-producing items. File-level rules
  share one file id; component rules use a per-component id. Patch fragments
  should not carry `<style>` — prefer file-level or shared CSS there.
- `#` at the start of a template item is a comment. `@@` emits a literal `@`.

## Rocdown pages

Start with Markdown and `@page`. Add a declaration only where the page needs
structure.

````rocdown
@page {
    meta: {
        title: "Hello",
        description: "A small Markdown-first page",
    },
}

# Hello

Ordinary paragraph. Fenced code is never executed:

```roc
answer = 42
```

:note[title: "Static sites"] {{
    `rocdown build` accepts `@page` and `:note` beside Markdown.
}}
````

- Document-root `<Byline date={published} />` is an HTML island, not raw HTML.
- `@render MyComponent({ ... })` splices a PascalCase component call into the Markdown stream.
- Links: `[[Page]]`, `[label](Page.rocdown)`, or stable `/route/` on sites.
- `:include` should prefer a named region over line numbers.
- Knowledge records stay inert Markdown. Do not add Rocdown declarations there.

## Server handlers and purity

Keep rendering pure. Put I/O in `@init` and `@method:role` handlers (or an authored `main.roc`).

```rocci
@post:fragment("/actions/counter/increment") = |{ db }| {
    count = increment_count!(db)?
    counterCard({ count })
}

@component CounterCard = |{ count }|
    <section id="counter" class="counter-card">
        <output>{count.to_str()}</output>
    </section>
```

`@get:view` returns a document. Mutation fragments return the HTML Datastar
will patch; the target element needs a stable `id`. That POST updates **this
tab only**. Shared live views and representation-free commands are
`$rocci-stack`. Use `@post("/actions/...")` (Roc strings), not single-quoted
JS, unless the attribute is intentionally an opaque Datastar expression
(window keydown that branches on `evt.key`).

## Fixtures

```rocci
@fixture{target: StatusCard}
failed_card = {
    title: "Documentation",
    status: Failed,
}
```

Unqualified `target` must name a local `@component`. The binding stays
ordinary Roc for `rocci view`. Pair a boolean check with
`@test{fixture: failed_card}`.
