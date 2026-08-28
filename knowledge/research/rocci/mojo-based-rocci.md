---
type: Research Report
title: A Mojo-based Rocci alternative after Mojo 1.0
description: "Exploratory counterfactual: Mojo 1.0 is a stable indent-based systems language with comptime parameters, ownership, traits, t-strings, and Python interop, but no custom decorators, no match/unions, and unfinished async. A Rocci-shaped product on Mojo should not use indent as an HTML or Markdown closer; the honest fits are a paren-escaped Html library, a Python+Datastar host with Mojo kernels, or compute islands inside current Rocci—not a retargeted .rocci grammar."
tags: [domain/rocci, domain/rocdown, domain/runtime, concern/syntax, concern/language-design, concern/developer-experience, concern/architecture, concern/authoring, integration/datastar, integration/roc]
status: draft
generated: { by: process:cursor, at: 2026-08-28T18:16:00Z }
stale_after: 2026-11-28
authority: exploratory
owners: [human:nils]
sources:
  - id: counter
    resource: ../../../examples/rocci/standalone/counter/Counter.rocci
    title: One-file GET view plus POST fragment counter
    author: process:git
    last_modified: 2026-08-25
  - id: live-counter
    resource: ../../../examples/rocci/standalone/live-counter/LiveCounter.rocci
    title: GET view, POST command, GET live
    author: process:git
    last_modified: 2026-08-23
  - id: template-readme
    resource: ../../../crates/rocci-template/README.md
    title: Implemented Rocci template language reference
    author: process:git
    last_modified: 2026-08-25
  - id: rocdown-readme
    resource: ../../../crates/rocci-rocdown/README.md
    title: Implemented Rocdown language reference
    author: process:git
    last_modified: 2026-08-24
  - id: file-structure
    resource: ../../../docs/reference/language/file-structure.rocdown
    title: File structure and Roc regions
    author: process:git
    last_modified: 2026-08-25
  - id: components-ref
    resource: ../../../docs/reference/language/components.rocdown
    title: Component declaration contract
    author: process:git
    last_modified: 2026-08-25
  - id: server-ref
    resource: ../../../docs/reference/language/server.rocdown
    title: Verb-first server declaration reference
    author: process:git
    last_modified: 2026-08-22
  - id: directives-ref
    resource: ../../../docs/reference/language/directives.rocdown
    title: Template @if @for @match contract
    author: process:git
    last_modified: 2026-08-22
  - id: attributes-ref
    resource: ../../../docs/reference/language/attributes.rocdown
    title: Attributes and Datastar actions
    author: process:git
    last_modified: 2026-08-25
  - id: css-ref
    resource: ../../../docs/reference/language/css.rocdown
    title: File-level and component @css
    author: process:git
    last_modified: 2026-08-25
  - id: tests-ref
    resource: ../../../docs/reference/language/tests.rocdown
    title: Root @test contract
    author: process:git
    last_modified: 2026-08-25
  - id: tags-ref
    resource: ../../../docs/reference/language/tags.rocdown
    title: Tags, fragments, and component calls
    author: process:git
    last_modified: 2026-08-25
  - id: guide-rocdown
    resource: ../../../examples/rocdown/pages/Guide.rocdown
    title: Markdown-first Rocdown guide with islands
    author: process:git
    last_modified: 2026-08-23
  - id: format-arch
    resource: ../../architecture/rocdown-format.md
    title: Rocdown format boundary
    author: process:cursor
    last_modified: 2026-08-20
  - id: markdown-first
    resource: ../../decisions/markdown-first-explicit-islands.md
    title: Markdown-first with explicit executable islands
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: pure-render
    resource: ../../decisions/pure-render-components.md
    title: Pure render components
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: server-owned
    resource: ../../decisions/server-owned-state.md
    title: Server-owned durable state
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: product-boundary
    resource: ../../decisions/consolidate-rocdown-product-boundary.md
    title: Rocci template versus Rocdown document boundary
    author: process:cursor
    last_modified: 2026-08-24
  - id: block-model
    resource: ../rocdown/generalized-rocdown-block-model.md
    title: Generalized Rocdown block model
    author: process:cursor
    last_modified: 2026-08-24
  - id: roc-library
    resource: method-role-handlers-as-roc-library.md
    title: Method-role handlers as a Roc library
    author: process:cursor
    last_modified: 2026-08-24
  - id: ecosystem
    resource: method-role-handlers-datastar-ecosystem.md
    title: Method-role matrix compared with Datastar SDKs
    author: process:cursor
    last_modified: 2026-08-24
  - id: islands
    resource: client-behavior-islands.md
    title: Client-behavior @island design blockers
    author: process:cursor
    last_modified: 2026-08-28
  - id: mojo-1-0-blog
    resource: https://www.modular.com/blog/modular-26-5-mojo-1-0-is-here
    title: Modular 26.5, Mojo 1.0 is here
    author: organization:modular
    last_modified: 2026-08-11
  - id: mojo-oss
    resource: https://www.modular.com/blog/mojo-open-source
    title: Mojo is now open source (Apache 2.0 with LLVM exceptions)
    author: organization:modular
    last_modified: 2026-08-18
  - id: mojo-basics
    resource: https://docs.modular.com/mojo/manual/basics/
    title: Mojo 1.0 language basics
    author: organization:modular
    last_modified: 2026-08-11
  - id: mojo-control
    resource: https://docs.modular.com/mojo/manual/control-flow/
    title: Mojo control flow, including no match/switch
    author: organization:modular
    last_modified: 2026-08-11
  - id: mojo-ownership
    resource: https://docs.modular.com/mojo/manual/values/ownership/
    title: Mojo ownership and argument conventions
    author: organization:modular
    last_modified: 2026-08-11
  - id: mojo-params
    resource: https://docs.modular.com/mojo/manual/parameters/
    title: Mojo compile-time parameters
    author: organization:modular
    last_modified: 2026-08-11
  - id: mojo-errors
    resource: https://docs.modular.com/mojo/manual/errors/
    title: Mojo errors, raises, try/except, and with
    author: organization:modular
    last_modified: 2026-08-11
  - id: mojo-python
    resource: https://docs.modular.com/mojo/manual/python/
    title: Mojo Python interoperability
    author: organization:modular
    last_modified: 2026-08-11
  - id: mojo-decorators
    resource: https://docs.modular.com/mojo/manual/decorators/
    title: Built-in Mojo decorators; no custom decorators
    author: organization:modular
    last_modified: 2026-08-11
  - id: mojo-parameter-deco
    resource: https://docs.modular.com/mojo/manual/decorators/parameter/
    title: "@parameter parametric closures; comptime if/for replace @parameter if/for"
    author: organization:modular
    last_modified: 2026-08-11
  - id: mojo-literals
    resource: https://mojolang.org/docs/reference/literals/
    title: Mojo t-string interpolation
    author: organization:modular
    last_modified: 2026-08-11
  - id: mojo-release
    resource: https://mojolang.org/releases/v1.0.0/
    title: Mojo v1.0.0 release notes and stability policy
    author: organization:modular
    last_modified: 2026-08-11
  - id: mojo-nightly
    resource: https://mojolang.org/releases/nightly/
    title: Nightly notes privatizing unfinished async coroutine types
    author: organization:modular
    last_modified: 2026-08-28
  - id: mojo-path
    resource: https://www.modular.com/blog/the-path-to-mojo-1-0
    title: Path to Mojo 1.0; async and private fields deferred
    author: organization:modular
    last_modified: 2026-08-11
  - id: lightbug
    resource: https://github.com/Lightbug-HQ/lightbug_http
    title: Community Mojo HTTPService framework
    author: organization:lightbug-hq
    last_modified: 2026-08-28
  - id: datastar-py
    resource: https://github.com/starfederation/datastar-python
    title: Official-line Datastar Python SDK
    author: organization:starfederation
    last_modified: 2026-06-02
---

# A Mojo-based Rocci alternative after Mojo 1.0

## Status

Exploratory counterfactual. Mojo 1.0 is a real, source-stable language as of Modular 26.5 (2026-08-11), and the compiler is Apache-2.0 as of 2026-08-18. Nothing here is a Rocci language change, a cutover plan, or an approved architecture. Sketches are invented against published Mojo 1.0 docs and shipped Rocci/Rocdown contracts; they are not compiled.[^mojo-1-0-blog][^mojo-oss][^mojo-release][^template-readme]

This is the same kind of document as [handlers as a Roc library](method-role-handlers-as-roc-library.md): a host-language thought experiment that keeps Rocci's product shape (pure HTML components, server-owned state, Datastar morph) in view so the syntax and DX claims stay comparable.[^roc-library][^pure-render][^server-owned]

## What is being analogized

Rocci is not "a web framework in Roc." It is a **template language that is also a Roc module**. A `.rocci` file copies ordinary Roc through unchanged and recognizes `@component`, `@css`, `@fixture`, `@test`, `@context`, `@init`, and verb-first `@method:role(path)` only at the start of a top-level definition. Component bodies are a bounded HTML grammar. Handlers are Roc, not templates. Lowering emits ordinary Roc `Html` functions. The parser does not type-check Roc regions.[^template-readme][^file-structure][^components-ref][^server-ref]

The one-file counter is the DX ceiling of that bet: SQLite `@init`, `@get:view("/")`, two `@post:fragment` actions, colocated `@css`, PascalCase tags, `data-on:click=@post("…")`, `@fixture` / `@test`. The live counter adds `@post:command` plus `@get:live("/sse")`. Reader UX is HTML plus Datastar morph, not a client store.[^counter][^live-counter][^attributes-ref][^server-owned]

Rocdown is a **different product**: Markdown-first documents, executable only at document-root line-start `@` / `:kind` / `<Tag>` islands, fences never execute. Indentation is not an article-block closer; that alternative was already scored as hostile to lists, fences, and copy-paste.[^rocdown-readme][^markdown-first][^format-arch][^block-model][^product-boundary]

A "Mojo-based Rocci" therefore has to pick which of those bets it keeps:

1. HTML as a typed value, not a string.
2. One-file apps with inspectable route headers.
3. Pure render components and server-owned durable state.
4. Datastar as browser transport.
5. Markdown-first documents with explicit islands.
6. A custom grammar plus source maps, versus one host language plus a library.

Mojo 1.0 can host (1), (3), and (4) in principle. It fights (2) and (5) on syntax, and it is missing language features that (1) and live (4) currently lean on in Roc.

## What Mojo 1.0 actually is

Mojo 1.0 is presented as a production-ready, source-stable general-purpose language: Python-like indent, static types, MLIR compilation, CPU/GPU targeting, and a small but growing stable stdlib. 1.x changes should be mostly additive; a later 2.0 is expected to break for memory-safety features such as private fields.[^mojo-1-0-blog][^mojo-release][^mojo-path]

Load-bearing 1.0 facts for a web/template analog:

| Mojo 1.0 fact | Why it matters here |
| --- | --- |
| Blocks are `header:` plus a deeper indented body; statements end at newline; `()`, `[]`, `{}` allow continuation | Template syntax must either *use* indent as the tree, or *escape* it inside parens |
| `var`, structs, traits, `def` | Components become functions or structs, not a runtime registry |
| Parameters in `[]` are compile-time; arguments in `()` are runtime; `comptime if` / `comptime for` | The distinctive Mojo tool: specialize CSS, parse HTML, unroll loops at compile time |
| Default args are immutable refs; `mut`, `var`+`^` ownership, `ref`, `out`, `deinit` | `Html` trees need an ownership story; mutating a fragment in place fights purity |
| `raises` / `try`/`except`; errors are extra return values, not stack unwind | Not Roc `Result` / `?`; no structural `match` on tags |
| **No** Python `match` / C `switch` | `@match` and Roc tags have no 1:1 lowering |
| T-strings: `t"Hello, {name}!"`, including triple quotes | Interpolation without inventing `{expr}` grammar; HTML-in-string temptation |
| Python interop both ways via unmodified CPython 3.10–3.14 | Datastar, Starlette, sqlite3, Jinja exist *today* on the Python side |
| Built-in decorators only (`@fieldwise_init`, `@parameter`, …); **no custom decorators** | FastAPI-style `@get` / `@component` is not Mojo-native in 1.0 |
| `@parameter` on nested `def` builds a capturing compile-time closure | Useful for `List.map`-shaped `@for`, not for HTTP routing |
| Async: `async def` exists; coroutine types were removed from the public prelude because the runtime is unfinished | `@get:live` and SSE generators are not a 1.0 strength |
| Official LSP, indent-based editor folding | DX win versus Rocci source maps — if there is no second grammar |
| Compiler and stdlib are open source; Modular/Qualcomm still own the design process | A forkable host, not a tiny language you control like `.rocci` |

[^mojo-basics][^mojo-control][^mojo-ownership][^mojo-params][^mojo-errors][^mojo-python][^mojo-decorators][^mojo-parameter-deco][^mojo-literals][^mojo-nightly][^mojo-path]

Community HTTP exists (Lightbug `HTTPService` with one `func(mut self, req) raises -> HTTPResponse`). Official Datastar Python (`datastar-py`) already speaks patch-elements, patch-signals, and async SSE generators. There is no first-party Mojo Datastar SDK in this survey.[^lightbug][^datastar-py][^ecosystem]

## Fit at a glance

Scores are exploratory, for "could this host a Rocci-shaped hypermedia app in 2026," not "is Mojo a good language."

| Rocci / Rocdown job | Mojo 1.0 | Tension |
| --- | --- | --- |
| Indent as code structure | Native | Fights HTML visual indent, CSS braces, Markdown 4-space code |
| HTML as a value | Possible (builder, JSX-like grammar, or comptime parse) | Not in the language; you add a library or a parser |
| One-file starter | Possible if markup is Mojo | Loses `.rocci` header grep unless you keep a preprocessor |
| Verb-first closed matrix | Library constructors, or a preprocessor | Custom `@get:view` decorators are **not** 1.0 |
| Pure components | `def … -> Html` with no I/O | `mut self` HTTP services invite hiding state in the service struct |
| `@match` / Roc tags | Missing | Rewrite as `if/elif` or wait for Phase-2 ADTs |
| Live SSE | Weak | Unfinished async; Python host is the honest live path |
| Markdown-first docs | Hostile if islands are indent-Mojo | Need fences or `:begin`/`:end`, which Rocdown already chose |
| Editor | Strong if one language | Collapses if you add `.mojoci` |
| CPU/GPU kernels | Strong | Almost orthogonal to morphing `#counter` |
| Python ecosystem | Strong | Pulls the product toward Python templates unless Mojo owns Html |

## The indent problem

This is the syntax question. Everything else is a library or a compiler.

Mojo compound statements are header-plus-indent: the header ends in `:`, the body must be indented more than the header, and the first body line sets the indent for the rest of that block. Continuation is allowed inside matching `()`, `[]`, or `{}`.[^mojo-basics][^mojo-control]

Rocci made the opposite choice for templates: the body-opening `{` is on the same logical header line; `@if` / `@for` / `@match` / `@let` are brace groups; a single-root component may omit braces because the **tag** is the closer. Indent in `.rocci` markup is cosmetic. Roc regions use Roc's own braces and `|params|`.[^directives-ref][^components-ref][^template-readme]

Rocdown already rejected indent as a closer for article blocks: no closer token, hostile to lists, fences, and copy-paste. Shipped spelling is `:kind[params]` with line content, `{{ }}`, or `:kind.begin` … `:kind.end`.[^block-model][^format-arch]

So a Mojo-based analog has three structural regimes in one file if it tries to keep today's product:

- **Mojo** — indent is meaning.
- **HTML** — tags are meaning; indent is taste.
- **CSS** — braces are meaning.
- **Markdown** (Rocdown) — indent is sometimes a code block.

Those four do not compose without an explicit escape. The rest of the syntax section is the catalog of escapes.

### How authors already escape Mojo indent

Mojo, like Python, treats indent inside parentheses as free:

```mojo
def greet(name: String, title: String) -> String:
    return (
        "Hello, "
        + title
        + " "
        + name
    )
```

A Rocci alternative that wants HTML trees without a new grammar should put the tree in a paren, bracket, or call-argument position so **indent becomes cosmetic again**. That is the Mojo-native analog of Rocci's "tags close themselves." Inventing indent-HTML (Pug, Slim, Haml) throws away the escape the language already has, and re-imports a decade of Django/Jinja copy-paste pain.

### The `@` collision

Rocci `@` is a declaration sigil: `@component`, `@css`, `@get:view`, `@if`, and attribute-position `@post("/path")`.[^file-structure][^attributes-ref]

Mojo `@` is a **compiler decorator** on the previous line (`@fieldwise_init`, `@parameter`). User-defined decorators are not supported in 1.0.[^mojo-decorators][^mojo-parameter-deco]

A Mojo-based product that keeps Rocci's `@` grammar fights the host lexer, the official LSP, and every Mojo tutorial. A Mojo-based product that uses `@get` as a user decorator is not expressible on 1.0 without a preprocessor (which is a second language again).

Datastar's `data-on:click=@post(…)` in HTML attributes is a third `@`. In a builder it becomes an ordinary call. In a t-string it is text. In JSX-in-Mojo it needs a dedicated attribute grammar, as Rocci has today.

## Syntax alternatives (markup)

All sketches render the counter card from `Counter.rocci`: a `<section id="counter">` with a label and an `<output>`. They are not Mojo that compiles.[^counter]

### S1 — Dual-mode: Mojo indent, HTML tags, indent as the HTML closer

```mojo
def counter_card(count: Int) -> Html:
    html:
        <section id="counter" class="counter-card">
            <p>Shared count</p>
            <output>{count}</output>
        </section>
```

**Idea:** `html:` opens a template region; dedent closes it; inside, tags work like Rocci.

**DX:** Looks like Rocci nested in Python. **Fails** on the first MDN paste whose indent is shallower than the `html:` body, on CSS `{}` mixed into the same indent ladder, and on `@if` that is Mojo (`if count > 0:`) versus template (`@if` with its own indent). Error recovery is "unexpected indent" rather than "missing `</section>`." This is the alternative Rocdown already rejected for documents, now applied to apps.[^block-model]

**Verdict:** Do not. Indent cannot be both Mojo structure and HTML structure.

### S2 — Indent-HTML (Pug / Slim / Haml)

```mojo
def counter_card(count: Int) -> Html:
    section#counter.counter-card:
        p: Shared count
        output: String(count)
        button data-on-click=post("/actions/counter/increment"): Increment
```

**Idea:** The tree *is* indent. No closing tags. Very Mojo-looking.

**DX:** Fast to type for tiny widgets. Hostile to HTML literacy (Rocci's current on-ramp: paste a `<section>`). Hostile to browser DevTools copy. Component slots and fragments become indent puzzles. CSS cannot be pasted. Markdown cannot embed these islands without a fence.

**Verdict:** Attractive in screenshots, poor as a hypermedia product language. Rocci's bet is that authors already know HTML.[^tags-ref]

### S3 — JSX-in-Mojo (tags are the closer; indent is cosmetic)

```mojo
def counter_card(count: Int) -> Html:
    return (
        <section id="counter" class="counter-card">
            <p>Shared count</p>
            <output>{count}</output>
        </section>
    )
```

Control flow stays Mojo, because the JSX value is just an expression:

```mojo
def todo_item(item: Todo) -> Html:
    return (
        <li>
            {item.text}
            {<s>{item.text}</s> if item.done else <span>{item.text}</span>}
        </li>
    )
```

Or, more readably, Mojo `if` **around** JSX, not inside a template directive:

```mojo
def badge(tone: String) -> Html:
    if tone == "positive":
        return <span class="badge badge--positive">OK</span>
    return <span class="badge">OK</span>
```

**Idea:** This is what Rocci already almost is, transplanted into an indent host. `<` starts a tag expression that ends at the matching closer; wrapping parens make indent free, matching Mojo continuation rules.[^mojo-basics][^tags-ref]

**Cost:** A second grammar. `<` is comparison in Mojo (`if x < y`). Rocci avoids that by only scanning tags **inside** component bodies. A Mojo-host JSX needs either the same region rule (`html:` then tags) or a full expression-level JSX parser in the Mojo compiler (not a 1.0 feature). Source maps return. Official Mojo LSP will not understand `<Hello />` unless you own the grammar.

**DX:** Best *template* hybrid if you insist on tags. Worst *tooling* hybrid unless Modular adds JSX (they will not; this is GPU/Python land).

**Verdict:** The right *shape* if a preprocessor exists; do not pretend it is "just Mojo."

### S4 — T-string HTML (Lit / htmx / Jinja-in-quotes)

```mojo
def counter_card(count: Int) -> Html:
    return html(t"""
        <section id="counter" class="counter-card">
            <p>Shared count</p>
            <output>{count}</output>
        </section>
    """)
```

T-strings interpolate expressions in `{}`, join adjacent literals, and exist as a typed `TString` that defers formatting. Triple quotes and a raw prefix are legal. Literal braces use `{{` / `}}`.[^mojo-literals]

**Idea:** Zero new grammar. Mojo LSP works. Indent inside the string is *data*; you want `std.String.dedent` or a convention that the first newline is stripped.

**DX / UX risks:**

- HTML is a string until `html()` parses it. Typos become runtime or a comptime parse if you are disciplined.
- `{count}` in a t-string is Mojo interpolation, not Rocci `{expr}` in an HTML AST. Nested `{` in CSS or JS in the string is a footgun (`{{` escaping).
- XSS: `t"<div>{user}</div>"` is the classic hole unless `html()` typed-escapes interpolated `String` and only allows `Html` to splice unescaped.
- Components: `<CounterCard />` in a string is not a typed call unless `html()` is a compiler plugin.
- Scoped CSS: you lose `@css` as an AST preamble unless CSS is a second string.

**Verdict:** Honest 1.0 baseline. Fine for prototypes. Weaker than Rocci for the one-file starter's whole point (typed tags, scoped CSS, fixtures).[^components-ref][^css-ref]

### S5 — Comptime-parsed HTML (the interesting Mojo-native language idea)

Mojo parameters are compile-time values in `[]`. `comptime for` unrolls. A function can take a string parameter and produce specialized code.[^mojo-params][^mojo-basics]

Sketch:

```mojo
fn counter_card(count: Int) -> Html:
    return html[
        """
        <section id="counter" class="counter-card">
            <p>Shared count</p>
            <output>{{count}}</output>
        </section>
        """
    ](count)
```

Or split static structure from runtime holes with a typed hole list:

```mojo
alias CARD = HtmlTemplate["section#counter", Holes["count: Int"]]

def counter_card(count: Int) -> Html:
    return CARD.fill(count=count)
```

**Idea:** Parse HTML **at compile time**, emit an `Html` constructor tree, type-check hole names, fail the build on unclosed tags. Interpolation is not a runtime format string. This is closer to Rocci lowering than S4, implemented with Mojo's actual distinctive feature instead of a Rust parser.

**DX:** Author still writes HTML in a string or a `.html` file passed as a parameter. Editor highlighting inside the string is weak unless the LSP is taught about `html[…]`. Diagnostics can be excellent if the comptime parser reports positions.

**Verdict:** The one *language* experiment worth stealing from Mojo even if Rocci stays on Roc: comptime HTML/CSS, not indent-HTML.

### S6 — Builder DSL in paren position (most 1.0-native)

```mojo
def counter_card(count: Int) -> Html:
    return section(
        id="counter",
        class="counter-card",
        p("Shared count"),
        output(String(count)),
    )

def counter_page(count: Int) -> Html:
    return html(
        lang="en",
        head(
            meta(charset="utf-8"),
            title("Welcome"),
            script(type="module", src="/assets/datastar.js"),
        ),
        body(
            counter_card(count),
            div(
                class="actions",
                button("Increment", data_on_click=post("/actions/counter/increment")),
                button("Reset", class="secondary", data_on_click=post("/actions/counter/reset")),
            ),
        ),
    )
```

`class` is a Mojo keyword, so the HTML attribute becomes `class_` or a `**attrs` convention. That naming friction is real DX.

**Idea:** Same family as [handlers as a Roc library](method-role-handlers-as-roc-library.md): delete the template language; HTML is functions. Indent is free because the tree lives in `()`.[^roc-library][^mojo-basics]

**DX:** One language, one LSP, one formatter (`mojo format` indent). Verbose on documents. Excellent on small widgets. Tests are ordinary `def`. No `@fixture` sugar unless you write a convention. Loses "grep `@post:fragment`" unless routes are also constructors (see handlers below).

**Verdict:** Best *pure Mojo 1.0* analog of Rocci components. The one-file starter gets worse; the gallery/custom-main cliff gets better. Same trade as the Roc-library paper, with indent making builders *more* palatable than Roc `Html.element("section", […], …)` because calls nest with trailing commas instead of `}` noise.[^roc-library][^counter]

### S7 — `with` as the tree (context-manager HTML)

Mojo `with` runs `__enter__` / `__exit__` and scopes `as` bindings to the block.[^mojo-errors]

```mojo
def counter_card(count: Int) -> Html:
    with Html.builder() as h:
        with h.tag("section", id="counter", class="counter-card"):
            h.tag("p", text="Shared count")
            h.tag("output", text=String(count))
        return h.finish()
```

**Idea:** Indent *is* the tree, but through a legal Mojo compound statement rather than a new grammar.

**DX:** Reads like a tutorial. Nested `with` is noisy. Early `return` and exceptions must still produce well-formed HTML (the builder's `__exit__` has to be airtight). Mixing `if` and `with` is natural:

```mojo
def maybe_banner(show: Bool) -> Html:
    with Html.builder() as h:
        if show:
            with h.tag("div", class="banner"):
                h.text("Hello")
        return h.finish()
```

That last snippet is the rare case where **indent-as-HTML helps**: the `if` and the element share one indent language. S6 does this with ordinary `if` around a `div(...)` call, which is clearer.

**Verdict:** Cute, not the default. Keep `with` for SQLite connections and request-scoped files, not for `<div>`.

### S8 — Two files: Mojo + Jinja/Mustache/Askama-class templates

```mojo
def counter_card(count: Int) -> Html:
    return render("counter_card.html", count=count)
```

```html
<section id="counter" class="counter-card">
  <p>Shared count</p>
  <output>{{ count }}</output>
</section>
```

**Idea:** Python web's actual practice. Mojo does not have to parse HTML.

**DX:** HTML highlighting works. Split navigation hurts the one-file starter. Scoped CSS and fixtures become a convention across files. This is Askama/okmate's world, not Rocci's.[^roc-library]

**Verdict:** If the goal is "ship a Mojo backend," this is boring and correct. If the goal is "Rocci's colocated markup DX," this is a downgrade.

### Directive mapping (`@if` / `@for` / `@match` / `@let`)

Rocci template directives exist because the body is not Roc: you cannot write Roc `if` in the middle of a tag tree without leaving the HTML grammar. Bodies open `{` on the header line; `@for` lowers to `List.map`; `@match` arms are elements or nested directives, not bare text.[^directives-ref]

In S3/S6, directives **disappear**. They become Mojo `if` / `for`. That is a DX win *if* there is no second grammar.

`@match` does not disappear. Mojo 1.0 has no `match` / `switch`. The docs say so. Error handling suggests `Variant` and enumerated error structs, not general tagged unions.[^mojo-control][^mojo-errors]

Rocci/Roc authors write:

```rocci
@match tone {
    Neutral => <span class="badge">OK</span>
    Positive => <span class="badge badge--positive">OK</span>
}
```

Mojo 1.0 authors write:

```mojo
if tone == Tone.neutral:
    return span(class="badge", "OK")
elif tone == Tone.positive:
    return span(class="badge badge--positive", "OK")
else:
    raise "unreachable"
```

Exhaustiveness is on you until Phase-2 ADTs. For a UI language that uses tags as domain modeling, that is a **regression**, not a skin.

`@let` becomes `var` before the return. Fine.

`@for` becomes `for item in items:` collecting a `List[Html]`, or a list comprehension if/when those exist. Ownership: appending `Html` into a list wants `Movable` and often `^`. Easy to get wrong; Rocci hides it behind `List.map`.[^mojo-ownership][^directives-ref]

### CSS mapping

Rocci `@css { … }` is raw CSS, no interpolations, concatenated in source order; component CSS is a preamble; lowering stamps `data-rocci-css` and wraps `@scope`. Authors keep `class="card"`.[^css-ref]

Mojo options:

| CSS form | Fit |
| --- | --- |
| String next to the function | Works; no scoping unless a helper hashes the source |
| Comptime `css[".card { … }"]` | Can stamp a scope id at compile time — actually *better* than runtime hashing |
| Indent-CSS (Sass-like) | Third indent language; do not |
| Separate `.css` file | Fine; breaks colocation |
| `with` / builder `h.css(...)` | Odd; CSS is not a DOM tree |

Do not put interpolations in CSS by accident via t-strings. Rocci forbids them for a reason.[^css-ref]

### Component naming

Rocci: PascalCase `@component Hello` / `<Hello />`, generated camelCase `hello`, because Roc values cannot start uppercase.[^components-ref][^tags-ref]

Mojo structs are PascalCase, functions are snake_case by convention. A builder analog should **call** `counter_card(...)` and never invent `<CounterCard />` unless there is JSX. That removes a whole class of Rocci diagnostics (consecutive leading capitals, `HtmlShell` vs `HTMLShell`) and also removes the "tag looks like HTML but is a function" onboarding trick.

### Tests and fixtures

Rocci `@fixture` / `@test` are root-only markers; `rocci test` stages `expect`. Invalid in Rocdown.[^tests-ref]

Mojo: ordinary `def test_counter_card():` and whatever test runner Modular ships. No grammar. Less discoverable in a component browser unless you keep a naming convention (`counter_card_fixture`).

## Host-architecture alternatives

Markup syntax is independent of who talks HTTP. These are product shapes.

### A — Retarget lowering: keep `.rocci`, emit Mojo

The current crate copies Roc through and lowers templates to Roc `Html`.[^template-readme]

Emitting Mojo instead means:

- Roc `|state| { … }` handlers become `def` + indent; every authored Roc region is now a **second** translation.
- Source maps target Mojo, not Roc.
- You still have `@` vs Mojo `@`.
- You lose Roc's tags/`?` and do not gain Mojo parameters unless the lowerer invents them.

**Verdict:** Worst of both worlds. The whole point of `.rocci` is that non-template regions are *already* the host language. Roc and Mojo are not close enough for that copy-through trick.

### B — New file type `.moji` / `.mojoci` with a Rocci-like grammar redesigned for indent

A preprocessor owned like `rocci-template`, hosted on Mojo.

If you do this, the grammar lesson from S1–S3 is: **tags or builders, not indent-HTML**; **braces or parens for template regions, not dedent**; **do not reuse `@`**. A plausible header style is keyword-based:

```text
component CounterCard(count: Int):
    section(id="counter"):
        ...
```

which slides into S2 and should be rejected, or:

```text
component CounterCard(count: Int) = html(
    section(id="counter", ...),
)
```

which is S6 with a keyword prefix the compiler strips — i.e. a library with extra sugar.

**Verdict:** Only justified if the sugar is the inspectable verb-first matrix. That sugar is ~50 lines of constructors in a library. See C.

### C — Library, not language (Mojo analog of the Roc-library paper)

Routes are a `List[Route]`. Roles are constructors. Components are S6 functions. Program wrap (empty SSE vs 204, live poll) stays in `program()`, not in author bodies.[^roc-library][^ecosystem]

```mojo
fn routes() -> List[Route]:
    return [
        view("/", get_view),
        fragment("POST", "/actions/counter/increment", increment),
        fragment("POST", "/actions/counter/reset", reset),
    ]

def get_view(state: State, request: Request) raises -> Html:
    return counter_page(read_count(state.db))

def increment(state: State, request: Request) raises -> Html:
    return counter_card(increment_count(state.db))
```

Lightbug's actual 1.0-shaped API is one `HTTPService.func` that switches on the request — a manual dispatch table, not a matrix:[^lightbug]

```mojo
struct CounterService(HTTPService):
    var db: SqliteDb

    fn func(mut self, req: HTTPRequest) raises -> HTTPResponse:
        var path = req.uri()
        if req.method() == "GET" and path == "/":
            return OK(render(counter_page(read_count(self.db))))
        if req.method() == "POST" and path == "/actions/counter/increment":
            return patch_elements(counter_card(increment_count(self.db)))
        return NotFound()
```

**DX:** Matches Datastar SDKs (register helpers, speak HTTP). Loses header inspectability and pre-host illegal-pair diagnostics unless constructors encode the eleven-pair matrix. Gains prefix routes and mixed events as ordinary code. Custom `mut self` on the service is a purity footgun: it is easy to keep "current count" on the struct instead of SQLite.[^roc-library][^pure-render][^server-owned][^ecosystem]

**Verdict:** The only honest *pure Mojo* Rocci. Wait for async before promising live. Encode roles as constructors, not as `if path ==`.

### D — Python host, Mojo kernels (the 1.0-shaped split)

Python already has `datastar-py` with `@datastar_response`, `SSE.patch_elements`, and async generators for long-lived streams. Mojo can import Python modules and can be imported from Python as a compiled extension.[^datastar-py][^mojo-python]

```python
from datastar_py import ServerSentEventGenerator as SSE
from counter_mojo import render_card, increment  # Mojo module

@app.post("/actions/counter/increment")
def increment_route():
    count = increment()
    return DatastarResponse(SSE.patch_elements(render_card(count)))

@app.get("/sse")
async def live():
    async def events():
        while True:
            yield SSE.patch_elements(render_card(read_count()))
            await asyncio.sleep(0.1)
    return DatastarResponse(events())
```

**Idea:** Give HTTP, async, SSE, and the Datastar SDK to Python. Give inner loops (render a huge table, OKF search, Blocks gravity, image) to Mojo.

**DX:** Two languages, but both are "Python-shaped." Templates might stay Jinja or a Python Html builder. This is **not Rocci**. It is a Modular-shaped web app. Live UX is actually *easier* than Mojo-only because async exists.

**UX:** End users still see Datastar morph. Latency depends on the CPython boundary: if every click round-trips `PythonObject`, you have lost; if the request handler stays in Python and only heavy renders enter Mojo, it can win.

**Verdict:** Best *practical* use of Mojo 1.0 near this product. Does not replace `.rocci`. Could replace a custom `main.roc` compute hotspot.

### E — Keep Rocci; Mojo as a compute island

Analogous to reserved client-behavior `@island`, but for **server** CPU/GPU work, not browser JS. No new template grammar. Rocci still owns HTML and routes. A native library (or Python sidecar) runs a kernel.[^islands][^pure-render]

Examples that would actually use Mojo's differentiator:

- OKF retrieval / embedding search over a large bundle.
- Falling-block collision or replay at high tick rates (today's Blocks is SQLite + HTML).
- Image / syntax highlighting / PDF raster.

**Verdict:** Smallest product risk. Orthogonal to indent syntax. Only do it with a measured hotspot; do not introduce Mojo to make `#counter` faster.

### F — Comptime site generator (Mojo as Rocdown's static half)

`comptime` walks a content directory, parses Markdown with a Python library via interop, emits HTML. Runtime is a file server.

**Idea:** Modular-shaped Rocs. Fights the approved split (Rust catalog, Rocci shell) and the OKF inert-Markdown boundary. Python already does this (mkdocs, sphinx). Mojo adds little until Markdown parse is a GPU joke.

**Verdict:** Non-goal unless the catalog itself is the hotspot.

## Handler and runtime alternatives

Rocci's closed matrix is `@get:view|fragment|live` and mutation `:fragment|:command` on literal paths. Dispatch calls `handler!(context, request)`. Commands are empty SSE for Datastar and 204 otherwise. Live is path-addressed poll, not push.[^server-ref][^ecosystem]

### H1 — Constructor matrix in Mojo (recommended if C)

Same as the Roc-library paper: `view`, `fragment`, `command`, `live` constructors. Illegal pairs cannot be spelled. Wrap stays in `program()`.[^roc-library]

### H2 — FastAPI-style decorators

```mojo
@get_view("/")
def home(state: State) raises -> Html:
    return counter_page(read_count(state.db))
```

**Not 1.0.** Custom decorators are unsupported.[^mojo-decorators]

A preprocessor that *rewrites* this into H1 is a third language. Drop it.

### H3 — Trait per role

```mojo
trait View:
    fn render(self, state: State, req: Request) raises -> Html

trait Command:
    fn run(self, state: State, req: Request) raises
```

Mojo traits are compile-time, not Python classes; no dynamic dispatch in 1.0 structs.[^mojo-basics]

Useful for `program[routes: Tuple[Route]]()` specializing dispatch at comptime (a real Mojo-shaped win: the eleven-pair matrix becomes a `comptime for` over a parameter tuple). Too cute for v1 of a library.

### H4 — Lightbug single `func`

See C. Lowest ecosystem friction, worst inspectability, easy to accidentally return JSON that Datastar treats as patch-signals (the bug Rocci's `:command` exists to make unrepresentable).[^ecosystem][^lightbug]

### H5 — Python ASGI + Mojo render (D)

The live-capable path. `datastar-py` already documents one-event, multi-event, and infinite SSE. That matches Rocci's one-shot versus live split better than Lightbug's single response.[^datastar-py]

### Errors versus Roc `?`

Rocci handlers use `?` on `Result`. Mojo uses `raises` and `try`/`except`. Typed errors and `Variant` exist; they are not Roc tags.[^mojo-errors]

`@init` failure exits the process in Rocci.[^server-ref] In Mojo, an uncaught raise already terminates with a non-zero code — same UX if you do not `except` in `main`. Do not catch-and-ignore around SQLite setup.

### Ownership versus pure render

Approved Rocci: `@component` is a function from explicit values to `Html`; no instance lifecycle.[^pure-render]

Mojo default args are immutable refs (no copy). Returning a new `Html` tree is natural if `Html` is `Movable`. `mut self` on a component struct is how you accidentally invent React-on-the-server.

Rule for a Mojo analog: **components take immutable args and return owned `Html`**; only handlers may `mut` a `db` handle. Transfer large trees with `^` at the HTTP boundary so the service does not keep a stale DOM.[^mojo-ownership][^pure-render]

### Live streams

Rocci live is generated poll plus keepalives because basic-webserver has no pub/sub and async Roc is not the model.[^live-counter][^ecosystem]

Mojo 1.0's public async story is weaker than that: coroutine types were hidden so people would not build on them; a robust async model is explicitly post-1.0.[^mojo-nightly][^mojo-path]

A Mojo-only live counter would likely be **blocking poll in the handler**, which is worse than Rocci's generated unfold, or a hand-rolled thread — do not advertise Datastar Tao CQRS on that.

## Rocdown in a Mojo world

### Why indent islands are a non-starter

Document-root recognition today: optional indent, then `@` + reserved name, or `:` kind, or `<` tag. Indented `@roc {` at root is a real declaration; CommonMark indented code does **not** win for reserved names. Fences never execute. Ordinary `@` in prose is not special.[^rocdown-readme][^format-arch][^markdown-first]

Mojo *is* indent. A naive island:

```markdown
# Guide

    def feature_count() -> Int:
        return 3
```

is a CommonMark indented code block. It would display, not run — which is today's correct fence policy — and authors would think they wrote a program.[^guide-rocdown]

A slightly less naive island:

```markdown
@mojo:
    def feature_count() -> Int:
        return 3
```

Dedent is the closer. Nested Markdown lists inside a mixed document, copy-paste, and "is this still the island?" are exactly the failure mode the block-model research scored as **Indentation: No closer token | Hostile**. Prefix `@` is also the language-island sigil; putting Mojo's only block syntax behind it doubles the confusion.[^block-model]

### R1 — Keep Markdown-first; Mojo islands use an explicit closer

Reuse shipped closers:

```markdown
@roc.begin
def feature_count() -> Int:
    return 3
@roc.end
```

or `:mojo.begin` … `:mojo.end`, or `{{ }}` for short bodies. Indent inside the island is Mojo. The island itself is line-start pair, not dedent. This is the only document-safe embedding.[^block-model][^format-arch]

`@roc { … }` braces are *also* an explicit closer and already exist. A Mojo body inside braces is legal but ugly (Mojo does not want braces for `def`). Prefer begin/end for indent languages.

### R2 — Invert "fences never execute" for a labeled fence

````markdown
```mojo live
def feature_count() -> Int:
    return 3
```
````

Familiar from mdBook / MyST / Jupyter. **Breaks** Rocdown's security and review story: fences are the thing reviewers trust as dead. The format decision is that executable regions must be visible declarations, not code fences.[^markdown-first][^rocdown-readme]

**Verdict:** Do not, unless the product stops being Rocdown.

### R3 — Documents stay Rocdown/Roc; apps become Mojo

Two ecosystems, one Datastar UX. Matches the approved product split (Rocci apps vs Rocdown docs) taken to the host-language level.[^product-boundary]

**DX:** Docs authors never see Mojo indent. App authors never see Markdown indent traps. The cost is two toolchains and no shared `@component` between `Guide.rocdown` and `Counter.moji` unless you invent an FFI Html.

### R4 — Mojo-down (the document *is* indent)

Python-markdown processors, some wiki engines, and "literal programming" layouts. Every heading is a `def`. Unreadable as prose. Destroys Markdown tool reuse (GitHub preview, OKF inert records).

**Verdict:** Reject. Knowledge stays inert Markdown.[^markdown-first]

### R5 — T-string Markdown (not a document format)

```mojo
def page(published: String) -> Html:
    return markdown(t"# Rocdown\n\nPublished {published}.")
```

Fine as a helper. Not a site catalog, not routes, not `[[wiki]]` links, not `:note`. Do not confuse a function with Rocdown.[^guide-rocdown]

### Interpolation

Rocdown's exploratory hole is `@{expr}` in Markdown; Rocci keeps `{expr}` in template mode. Bare `{expr}` in Markdown is prose.[^guide-rocdown]

Mojo t-strings use `{expr}` always. A Markdown-in-t-string document would make `{published}` live, which is the MDX problem Rocdown refused. If a Mojo analog of Rocdown exists, keep a **visible** hole (`@{expr}` or `t-string` only inside an island), never silent interpolation in prose.[^markdown-first]

### What "Rocdown" even means on Mojo

Today Rocdown means: Markdown AST in Rust, optional Roc/Rocci islands, Rocci theme shell.[^product-boundary][^format-arch]

On Mojo it would mean one of:

- The same Rust catalog, with islands lowered to Mojo instead of Roc (A, but for documents — still a bad copy-through).
- Python-Markdown + a Mojo theme function (D).
- No Rocdown; docs are MkDocs.

The indent influence is decisive: **Rocdown's job is to keep prose in a language where indent is not control flow.** Hosting documents on Mojo inverts that job. Keep Rocdown on Markdown regardless of whether apps move.

## DX (authors, agents, tooling)

### One-file starter

Rocci wins on the counter: markup, CSS, routes, fixtures, tests in one file with grepable headers.[^counter][^roc-library]

Mojo S6+H1 is two conceptual layers in one `.mojo` file (functions + a `routes()` list). That is still one file, but the contract is not on the def line unless you accept H2's illegal decorators.

Mojo S4 t-strings look like one file and behave like PHP.

### Inspectability without types

Rocci headers exist because the template parser cannot see Roc return types.[^roc-library][^server-ref]

Mojo *has* types on `def`. A library can put `-> Html` vs `raises` (command) in the signature. That **removes** the original reason for `@method:role` — in Mojo, H1 constructors are extra safety, not a parser workaround. That is a real DX argument for C over B.

### Completions and diagnostics

- Pure Mojo: official LSP, indent folding, no source maps.[^mojo-release]
- `.mojoci` + JSX: you rebuild `rocci-lsp`.
- T-strings: LSP sees a string; HTML errors are inside a literal.

Agent/LLM authoring: models emit messy HTML indent. S3/S6 forgive it. S1/S2/S7 punish it. Rocci's tag grammar is also forgiving of indent. **Do not pick indent-HTML if agents author UI.**

### Formatter

`mojo format` will happily destroy visual HTML indent inside S1 regions if those regions are not strings. T-strings and paren-builders format as Mojo. JSX needs a custom formatter.

### `@` and keywords

Never teach `@component` as a Mojo decorator in 1.0 docs; it will not compile.[^mojo-decorators]

`class=` vs `class`, `match` missing, `async` unfinished: these will dominate forum questions, not SIMD.

### Purity and `mut`

The failure mode to design against: a `CounterService` with `var count: Int` updated in `func`. That duplicates domain state in memory, breaks multi-worker, and fights the server-owned decision. SQLite (or Python sqlite3 via interop) stays the source of truth; `Html` is a snapshot.[^server-owned][^pure-render]

### Toolchain and desktop

Rocci's preview window is Rust/Wry regardless of app language. A Mojo app is still an HTTP origin. Distribution becomes "Modular toolchain + host" instead of `roc` + `rocci`. Open source helps; it does not give you `cargo install` simplicity yet.[^mojo-oss]

WASI/wasm for Mojo web is not this paper; GPU/MLIR is the published target, not WASI HTTP.

## UX (end users of apps and docs)

If the wire stays HTML + Datastar, **reader UX can be identical**. Morph, stable `id`, one-shot vs live — those are protocol choices, not host-language choices.[^server-owned][^ecosystem][^datastar-py]

Where the host language leaks into UX:

| Surface | Mojo 1.0 risk |
| --- | --- |
| Live counter fairness across tabs | No honest push/async; worse than generated Rocci poll if someone fakes it with blocking I/O |
| Time-to-first-byte / patch size | Native render can be faster for huge trees; CPython boundary can be slower for tiny patches |
| Error pages | Uncaught `raises` vs Roc `?` — need a wrap that returns HTML 500, not a process abort after `@init` |
| Docs readability | Indent islands and t-string Markdown make source harder to review than `.rocdown` |
| Trust | Fences that execute, or HTML t-strings without escaping, are XSS and "did this run?" UX for reviewers |

Do not expect GPU to change a button click. Users feel Datastar morph time and network, not MLIR.

## Worked alternatives (same app, different bets)

The following are sketches of the **same** counter product. None are shipped.

### Alt-1 — Mojo library, paren Html, constructor routes (recommended Mojo-only shape)

Host: Lightbug or a thin ASGI later. Markup: S6. Handlers: H1. No second grammar. No live until async exists; ship one-shot only, like the non-live counter.[^counter][^lightbug]

This preserves: typed Html, server-owned SQLite, Datastar one-shot, pure functions.

This drops: tag syntax, `@css` AST, header grep, `@match`, live, Rocdown islands sharing components.

### Alt-2 — Python+Datastar host, Mojo `render_card` (recommended 1.0-shaped split)

Host: Starlette/Flask + `datastar-py`. Markup: Python builder or Jinja, **or** Mojo S6 imported as `render_card`. Live: Python async generator. Mojo: increment + render, or only a hotspot.[^datastar-py][^mojo-python]

This preserves: live UX, Datastar SDK literacy, Python editor tooling.

This drops: Rocci as a language; one-file `.rocci`; Rocdown-as-app.

### Alt-3 — JSX preprocessor `.mojoci` (only if the tag on-ramp is non-negotiable)

S3 + H1 generated into Alt-1. You have re-invented `rocci-template` on a worse host (indent + `<` ambiguity + no match). Custom `@` forbidden; use `component`/`css` keywords or attributes in JSX.

This preserves: HTML paste, colocated CSS if the preprocessor keeps S5 comptime CSS.

This drops: official LSP-for-free. You are in the compiler business again, now against an open-source MLIR stack you do not control.

### Alt-4 — T-strings everywhere (prototype only)

S4 + Lightbug `func`. Fast to demo. XSS and untyped paths. Fine for a spike that answers "can Mojo emit Datastar SSE at all," not for a product.

### Alt-5 — Compute island inside current Rocci (recommended incremental path)

No syntax. Measure a hotspot. Call Mojo (or a C ABI) from Roc/Rust. Keep `.rocci` and `.rocdown`.[^islands]

## What Mojo's programming model is actually good for here

Use these if *any* alternative ships; they are the reason to touch Mojo at all.

1. **Parameters / comptime** — specialize scoped CSS, unroll static nav, parse HTML templates, encode the route matrix as a parameter tuple. This is the analog of Rocci lowering, *in* the language.[^mojo-params]
2. **Ownership** — `Html` as `Movable`, transfer at the response boundary, immutable component args. Makes purity *checkable* rather than conventional, if you resist `mut self` state.[^mojo-ownership][^pure-render]
3. **Traits** — `Writable` already powers t-strings; an `IntoHtml` / `Node` trait is the component interface without a runtime registry.[^mojo-basics][^mojo-literals]
4. **Python interop** — Datastar, HTTP, Markdown, sqlite, pygments. The ecosystem Rocci currently rebuilds in Roc+Rust.[^mojo-python][^datastar-py]
5. **T-strings** — attribute values and debug dumps; not the DOM tree.
6. **`with`** — DB connections and files, not tags.[^mojo-errors]
7. **`@parameter` closures** — compile-time `map` for static lists of nav links.[^mojo-parameter-deco]
8. **SIMD / GPU** — search, parsers, games physics; **not** `#counter`.[^mojo-1-0-blog][^mojo-path]

Do not use: indent as HTML/Markdown closer; custom `@` route decorators; `match`-shaped UI; Mojo-only live SSE; fences as the executable island.

## Exploratory recommendation

Treat Mojo 1.0 as a **stable compute-and-Python language**, not as a hypermedia template language.

- **Do not** retarget `.rocci` to Mojo (A).
- **Do not** use indent as an HTML or Rocdown closer (S1, S2, R2-as-indent, R4). Rocdown research already called that hostility; Mojo makes it worse, not better.[^block-model]
- **Do not** promise FastAPI decorators or `@get:live` on Mojo 1.0 (no custom decorators; unfinished async).[^mojo-decorators][^mojo-nightly]
- **If** the question is "what would a Mojo Rocci look like as syntax?": **S6 paren builders + H1 constructors**, with optional **S5 comptime HTML/CSS** later. That is Rocci's semantics without Rocci's grammar, and it uses indent the way Mojo already does.
- **If** the question is "what would actually work this year?": **Alt-2** (Python Datastar host + Mojo kernels) or **Alt-5** (islands). Reader UX can stay Datastar; author DX becomes Python-shaped.
- **If** the question is "should Rocdown become Mojo-down?": **No.** Keep Markdown-first; if Mojo appears, it is a begin/end island, never a dedent island.[^markdown-first]

A future Mojo 2.x with ADTs, match, custom decorators, and real async would reopen B and H2. That language is not 1.0. Writing a grammar plan against it now would be the same mistake as planning `@island` before a morph spike.[^islands][^mojo-path]

[^counter]: One-file counter: `@context` / `@init`, `@get:view`, `@post:fragment`, `@component`, `@css`, `@fixture`, `@test`.
[^live-counter]: Live counter: `@post:command` plus `@get:live("/sse")`.
[^template-readme]: `.rocci` is a Roc module with a bounded HTML grammar; copy-through of non-`@` regions; no typecheck in the template crate.
[^rocdown-readme]: Markdown-first; document-root declarations; fences never execute.
[^file-structure]: Top-level recognition of `@` forms; Roc regions copied through.
[^components-ref]: `@component` params, PascalCase, optional braces for one root.
[^server-ref]: Closed `@method:role` matrix, `handler!(context, request)`.
[^directives-ref]: `@if` / `@for` / `@match` / `@let`; brace on the header line.
[^attributes-ref]: `data-on:click=@post("…")`; Datastar actions inject import.
[^css-ref]: Raw `@css`, `@scope` stamping, no interpolations.
[^tests-ref]: Root `@test`; invalid in Rocdown.
[^tags-ref]: Intrinsic tags vs PascalCase component calls; no dynamic tag names.
[^guide-rocdown]: Guide page: `@page`, `@roc`, `@css`, `<FeatureCount />`, Markdown `@{published}`.
[^format-arch]: Implemented Rocdown boundary versus historical report.
[^markdown-first]: Ordinary prose is Markdown; mode switch only at visible root declarations.
[^pure-render]: `@component` lowers to a pure function to `Html`.
[^server-owned]: Durable state is server-owned; Datastar transports HTML.
[^product-boundary]: Rocci owns templates; Rocdown owns documents and the static generator.
[^block-model]: Indentation rejected as an article-block closer; `:kind[params]` plus `{{ }}` or begin/end.
[^roc-library]: Counterfactual Roc library for the same matrix; one-file HTML vs constructor inspectability.
[^ecosystem]: Shipped matrix versus Datastar SDKs that leave roles in handler bodies.
[^islands]: Client-behavior `@island` is designed, not approved; do not plan grammar first.
[^mojo-1-0-blog]: Modular 26.5 announces Mojo 1.0 as a stable foundation.
[^mojo-oss]: Compiler open-sourced Apache 2.0 with LLVM exceptions on 2026-08-18.
[^mojo-basics]: Indent blocks, `var`, structs, traits, parameters vs arguments, Python import example.
[^mojo-control]: `if`/`elif`/`else`, `for`, `while`; no `match`/`switch`.
[^mojo-ownership]: Default immutable ref; `mut`; `var` and `^` transfer; exclusivity.
[^mojo-params]: `[]` compile-time parameters; inference; `comptime` control flow.
[^mojo-errors]: `raises`, `try`/`except`, typed errors, `with` context managers.
[^mojo-python]: CPython interop both directions; Python 3.10–3.14 for interop.
[^mojo-decorators]: Decorators are compiler-built-in; no custom decorators.
[^mojo-parameter-deco]: `@parameter` capturing closures; `comptime if/for` replace `@parameter if/for`.
[^mojo-literals]: T-string `{expr}` interpolation, triple quotes, `{{` escaping.
[^mojo-release]: 1.x stability policy; lambda; stdlib stability begins small.
[^mojo-nightly]: Unfinished async: coroutine types removed from the prelude.
[^mojo-path]: Async and private fields called out as not-1.0; Phase 2 systems-programming features.
[^lightbug]: `HTTPService.func(mut self, req) raises -> HTTPResponse`.
[^datastar-py]: Python SDK: patch-elements, patch-signals, async SSE generators.
