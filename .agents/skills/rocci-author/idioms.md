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
| **`pages/`** or **`routes/`** (or app root) | Standalone full-page web applications and HTTP route handlers | `Counter.rocci`, `Todos.rocci`, `Edit.rocci` | `@context`, `@init`, `@view`, `@patch` |
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
| Lowered component value | camelCase | `statusCard` in `@on` and `exposing` |
| Ordinary Roc helper / field | snake_case | `read_count!`, `has_completed` |
| Type / tag union payload | PascalCase | `Status : [Ready, Working, Failed]` |
| Effectful function | `snake_case!` | `write_page!`, `from_request!` |

Call the component from Roc with the lowered name:

```rocci
@view("/") = |{ db }| {
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
@component Badge = |{ tone ?? Neutral }, content|
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

Keep rendering pure. Put I/O in `@init` and `@on` (or an authored `main.roc`).

```rocci
@patch("/actions/counter/increment") = |{ db }| {
    count = increment_count!(db)?
    counterCard({ count })
}

@component CounterCard = |{ count }|
    <section id="counter" class="counter-card">
        <output>{count.to_str()}</output>
    </section>
```

GET returns a document. Mutations return the fragment Datastar will patch; the
target element needs a stable `id`. That POST updates **this tab only**. Shared
live views and JSON-vs-HTML Datastar responses are `$rocci-stack`. Use
`@post("/actions/...")` (Roc strings), not single-quoted JS, unless the
attribute is intentionally an opaque Datastar expression.

## Fixtures

```rocci
@fixture{target: StatusCard}
failed_card = {
    title: "Documentation",
    status: Failed,
}
```

Unqualified `target` must name a local `@component`. The binding stays
ordinary Roc for tests and `rocci view`.
