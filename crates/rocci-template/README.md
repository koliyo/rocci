# rocci-template

Parse `.rocci` modules and lower explicit components to ordinary Roc.

A `.rocci` file is a Roc module: ordinary declarations stay Roc, and
`@component name = |params| { ... }` bodies use a bounded HTML template
grammar. This crate does not invoke the Roc compiler, type-check
expressions, or own HTTP/runtime behavior.

```sh
cargo run -p rocci-template -- build path/to/file.rocci
cargo run -p rocci-template -- ast path/to/file.rocci
cargo run -p rocci-template -- inspect --ast path/to/file.rocci
```

`build` writes generated Roc to stdout, or to a file with `-o`. `ast` prints
the parse tree as an S-expression. `inspect` prints components, generated Roc,
and source-map segments; `--ast` includes the parse tree. The same commands
exist on the workspace `rocci` CLI. Input `-` reads stdin.

Library entry points are `parse`, `lower`, `compile`, and `format_ast` in
`rocci_template`.

## File shape

A file may mix a module header, imports, types, helpers, and any number of
components:

```rocci
module CounterPage exposing [counterPage]

import pf.Html
import Design

Tone : [Neutral, Positive]

@component hello = |{ name }| {
    <p>Hello, {name}</p>
}

badgeClass = |tone| {
    match tone {
        Neutral => "badge"
        Positive => "badge badge--positive"
    }
}
```

Everything outside an `@component` body is copied into the generated Roc
module unchanged. `@component` is recognized only at the start of a
top-level definition.

## Components

```text
@component name = |params| { template }
```

`params` is a Roc parameter list. The first parameter is normally a props
record. Extra parameters are the default body:

```rocci
@component badge = |{ tone }, content| {
    <span class={badgeClass(tone)}>
        {content}
    </span>
}
```

`??` field defaults are allowed in `.rocci` (`|{ name ?? "Roc" }|`). Roc
nightly-2026-08-08 rejects that syntax in patterns, so lowering strips `??`
from the generated function and inserts omitted values at call sites. Remove
this rewrite once `|{ name ?? "Roc" }|` typechecks.

This lowers to an ordinary function. The `@component` marker is removed:

```roc
badge = |{ tone }, content| {
    Html.element(
        "span",
        [Html.attribute("class", badgeClass(tone))],
        [content],
    )
}
```

A self-closing component tag becomes a one-argument call. A paired tag
passes the nested markup as a second argument:

```rocci
<Hello name={person.name} />
<Badge tone={Positive}>Current count</Badge>
```

```roc
hello({ name: person.name })
badge({ tone: Positive }, Html.text("Current count"))
```

There is no magic `children` field. Named regions are ordinary `Html` or
function-valued props.

## Tags

| Form | Meaning |
| --- | --- |
| `<div>`, `<output>` | HTML element (lowercase) |
| `<Hello />`, `<CounterCard>` | Component call; resolves to `hello`, `counterCard` |
| `<Design.Button />` | Qualified call; resolves to `Design.button` |
| `<>...</>` | Fragment; lowers to `Html.fragment(...)` |
| `<br>`, `<img />` | Void HTML elements use `Html.void_element` |

Write `<HtmlShell>` for a value named `htmlShell`. Consecutive leading
capitals such as `<HTMLShell>` are rejected as ambiguous.

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
- A valueless name is a boolean attribute (`Html.boolean_attribute`).

On HTML elements these become `Html.attribute` / `Html.boolean_attribute`.
On component tags they become a props record. `count={count}` is emitted as
`{ count: count }`.

Attribute names may include hyphens (`aria-current`, `data-on-click`).
Dynamic attribute names are not supported.

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

Ordinary Roc `#` comments remain valid outside `@component` bodies and inside Roc
regions.

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
| valueless attr | `Html.boolean_attribute("disabled", Bool.true)` |
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

See `tests/fixtures/kitchen_sink.rocci` and the matching
`tests/fixtures/kitchen_sink.roc` for a complete example.

## AST dump

`format_ast` (and the `ast` command) prints the parse tree as indented
S-expressions. Bare atoms are identifiers and dotted paths; everything else
is a double-quoted string. Roc snippets that are not a simple path are
quoted. Ordinary Roc between components is shown as `(roc ...)` lines.

```rocci
@component hello = |{ name }| {
    <p class="greeting">Hello, {name}</p>
}
```

```lisp
(module
  (component hello
    (params "|{ name }|")
    (element p
      (attr class "greeting")
      (text "Hello, ")
      (interp name))))
```

Heads are `module`, `roc`, `component`, `params`, `element`, `call`,
`fragment`, `text`, `interp`, `attr`, `if`, `else-if`, `else`, `for`,
`match`, `arm`, and `let`. Self-closing tags include the `self-closing`
atom after the tag name.

## Not in this crate

- Invoking `roc check` / `roc build`, or remapping Roc type errors
- Full JSX (markup as an arbitrary Roc expression)
- Tagged `` html`...` `` literals
- Dynamic tags, prop spreading, or a runtime component registry
- HTTP, Datastar, routes, file watching, or process management

The language design and open questions live in [`ROC_TEMPLATE.md`](../../ROC_TEMPLATE.md).
