---
type: Research Report
title: Rocci @test syntax lowered to Roc expect
description: "Rocci has @fixture preview data and no @test. Recommend a root @test name = boolExpr form, optional {fixture: name}, lowering to Roc expect kept out of wrap_type_module, and rocci test staging that appends expects after Type := [].{ } then runs roc test on that module."
tags: [domain/rocci, integration/roc, concern/syntax, concern/testing]
status: draft
generated: { by: process:cursor, at: 2026-08-25T17:20:00Z }
stale_after: 2026-11-25
authority: exploratory
owners: [human:nils]
sources:
  - id: plan
    resource: ../../plans/rocci/rocci-test-syntax.md
    title: Implement @test and rocci test
    author: process:cursor
    last_modified: 2026-08-25
  - id: ungram
    resource: ../../../crates/rocci-template/Rocci.AST.ungram
    title: Rocci ModuleItem productions including FixtureDecl
    author: process:git
    last_modified: 2026-08-22
  - id: parser
    resource: ../../../crates/rocci-template/src/parser.rs
    title: Top-level @fixture parse and keyword recovery lists
    author: process:git
    last_modified: 2026-08-23
  - id: lower
    resource: ../../../crates/rocci-template/src/lower.rs
    title: Fixture lowering to a Roc binding plus FixtureInfo
    author: process:git
    last_modified: 2026-08-25
  - id: wrap
    resource: ../../../crates/rocci-template/src/roc.rs
    title: wrap_type_module puts the whole body in Type := [].{ … }
    author: process:git
    last_modified: 2026-08-16
  - id: compile-test
    resource: ../../../crates/rocci-template/tests/compile.rs
    title: Fixture compile, inspect, and validation tests
    author: process:git
    last_modified: 2026-08-25
  - id: template-readme
    resource: ../../../crates/rocci-template/README.md
    title: Recognized top-level forms; no @test
    author: process:git
    last_modified: 2026-08-25
  - id: fixtures-ref
    resource: ../../../docs/reference/language/fixtures.rocdown
    title: Public @fixture contract for view and browse
    author: process:git
    last_modified: 2026-08-22
  - id: components-guide
    resource: ../../../docs/templates/components.rocdown
    title: Fixtures described as experimental preview and test data
    author: process:git
    last_modified: 2026-08-25
  - id: cli-ref
    resource: ../../../docs/reference/cli.rocdown
    title: rocci CLI surface without a test command
    author: process:git
    last_modified: 2026-08-22
  - id: cli-main
    resource: ../../../crates/rocci-cli/src/main.rs
    title: rocci subcommands including view, browse, render, inspect
    author: process:git
    last_modified: 2026-08-24
  - id: playground-html
    resource: ../../../crates/rocci-cli/src/playground_html.rs
    title: Html.render snapshot staging with wrap_type_module and basic-cli
    author: process:git
    last_modified: 2026-08-25
  - id: html-roc
    resource: ../../../crates/rocci-cli/runtime/Html.roc
    title: Html.render and render_fragment wrappers
    author: process:git
    last_modified: 2026-08-15
  - id: allsyntax
    resource: ../../../test/AllSyntax.rocci
    title: Comprehensive syntax fixture with @fixture and no @test
    author: process:git
    last_modified: 2026-08-25
  - id: counter
    resource: ../../../examples/rocci/standalone/counter/Counter.rocci
    title: Standalone app with @fixture on CounterCard and CounterPage
    author: process:git
    last_modified: 2026-08-23
  - id: styling
    resource: ../../../examples/rocci/standalone/styling/Styling.rocci
    title: Cataloged styling example with Hello, FeatureCard, StylePage fixtures
    author: process:git
    last_modified: 2026-08-23
  - id: author-skill
    resource: ../../../.agents/skills/rocci-author/SKILL.md
    title: Authoring skill lists @fixture and not @test
    author: process:git
    last_modified: 2026-08-23
  - id: language-dev
    resource: ../../../.agents/skills/rocci-language-dev/SKILL.md
    title: Language-dev workflow for ungram, AllSyntax, docs
    author: process:git
    last_modified: 2026-08-22
  - id: highlight
    resource: ../../../crates/rocci-highlight/src/composite.rs
    title: Highlighter collect_fixture; no collect_test
    author: process:git
    last_modified: 2026-08-22
  - id: lsp
    resource: ../../../crates/rocci-lsp/src/analysis.rs
    title: LSP fixture symbols; DIRECTIVES omit test
    author: process:git
    last_modified: 2026-08-22
  - id: rocdown-scan
    resource: ../../../crates/rocci-rocdown/src/scan.rs
    title: Rocdown reserved @ names including fixture
    author: process:git
    last_modified: 2026-08-22
  - id: coverage
    resource: ../../../docs/coverage.toml
    title: Feature manifest with syntax.fixture and no syntax.test
    author: process:git
    last_modified: 2026-08-22
  - id: archive-jsx
    resource: ../../../archive/reports/ROC_TEMPLATE.md
    title: Historical Html.render expect example for components
    author: process:git
    last_modified: 2026-08-23
  - id: roc-tutorial
    resource: https://github.com/roc-lang/roc/blob/main/docs/mini-tutorial-new-compiler.md
    title: Roc expect and roc test on the new compiler
    author: organization:roc-lang
  - id: roc-allsyntax
    resource: https://www.roc-lang.org/examples/AllSyntax/README
    title: Official AllSyntax top-level and block expect forms
    author: organization:roc-lang
  - id: phase0-probe
    resource: https://github.com/roc-lang/roc
    title: nightly-2026-08-18-e9be50a roc test on hand-written Type-module expects
    author: process:cursor
    last_modified: 2026-08-25
---

# Rocci @test syntax lowered to Roc expect

## Claim

Ship a `.rocci` **`@test`** declaration that lowers to Roc **`expect`**, reuse
existing **`@fixture`** values as the usual input, and run those expects with
**`rocci test` → `roc test`**. Do not put `expect` inside
`wrap_type_module`. This record is exploratory. Implementation:
[implement `@test` and `rocci test`](/plans/rocci/rocci-test-syntax.md).[^plan]

## Current behavior

There is **no** `@test` token, AST node, CLI command, coverage row, or
AllSyntax example.[^ungram][^parser][^cli-main][^coverage][^allsyntax] A
repository-wide search for `@test` in this revision is empty.

`@fixture` **is** shipped. Grammar is root-only
`@fixture{target: Component} name = rocExpr`. The marker is stripped; the
binding stays ordinary Roc; `FixtureInfo` feeds `rocci view`, `rocci browse`,
and playground `Html.render` snapshots. Unqualified `target` must name a local
`@component`. Dotted targets are left to Roc. Inside a component body is an
error.[^parser][^lower][^compile-test][^fixtures-ref]

Public copy already treats fixtures as preview **and** test data, but nothing
consumes them as tests:[^components-guide][^author-skill]

```rocci
@fixture{target: Hello}
helloSample = { name: "Roc" }
```

Recognized top-level forms today: `@component`, `@fixture`, `@css`,
`@context`, `@init`, and `@method:role` routes.[^template-readme]

`rocci` has `build`, `run`, `view`, `browse`, `render`, `inspect`, and no
`test`. `rocdown test` is a different product (declared example commands on a
site).[^cli-ref][^cli-main]

`rocci render` / playground local mode already snapshot `Html.render` of the
first fixture or a defaultable component by wrapping generated Roc as
`TypeName := [].{ … }` and compiling a **basic-cli** `main.roc`.[^playground-html][^html-roc]

`rocci run` uses the same wrap for every `.rocci` module, then generates a
**basic-webserver** dispatcher. `wrap_type_module` strips `module … exposing`
and **indents the entire remainder** into the type body. Top-level Roc
`expect` is not a type field; putting it in that body would not be valid
Roc.[^wrap]

File `@css` injects `<style>` and `data-rocci-css="<stem>-<hash>"` onto
intrinsic HTML. `Html.render` of a component in such a file is **not** a bare
`<p>Hello, Ada</p>`. The stamp is deterministic from the file basename.[^lower]

Standalone examples already have fixtures (counter, styling, live-counter UI,
handler-matrix, blocks UI, datastar demos, snake). None have tests. Counter
handlers use SQLite `!` effects; fixtures still target pure
`@component`s.[^counter][^styling]

Highlighter, LSP outline, and Rocdown's reserved-word scanner know `fixture`
and not `test`.[^highlight][^lsp][^rocdown-scan]

## Roc `expect`

On the new compiler, a test is a top-level `expect <Bool>` (one line) or
`expect { … last-expr-is-Bool }`. `##` immediately above the `expect` is shown
on failure. `roc test file.roc` runs those expects. Inline `expect` inside
functions is a debug assertion, not the authoring form here.[^roc-tutorial][^roc-allsyntax]

Historical Rocci writing already used this pattern for components:[^archive-jsx]

```roc
expect
    Html.render(hello({ name: "Ada" })) == "<p>Hello, Ada</p>"
```

`Html` values are not the comparison type; **strings from `Html.render`**
are.[^html-roc][^archive-jsx]

## Why `@test` instead of opaque `expect`

Ordinary Roc is copied unchanged. A raw `expect` in a `.rocci` file would
enter `compiled.roc` and then `wrap_type_module`, breaking `rocci run` /
`view` / `browse` for any module that contains tests.[^wrap][^lower]

`@test` is the inspectable, validatable, wrap-safe form:

- Parser metadata (`TestInfo`) like `FixtureInfo`.
- Lowering can keep `expect` **out** of the wrapped module body.
- Optional `{fixture: name}` can require a local `@fixture` without Roc
  parsing the body.
- LSP, highlight, and `rocci inspect` can list tests.

Opaque `expect` should not be the public contract. A later diagnostic that
rewrites authors toward `@test` is optional.

## Recommended syntax

Root-only, parallel to `@fixture`, Bool body is opaque Roc (`scan_roc_expr`,
including a `{ … }` block):

```rocci
@fixture{target: Hello}
helloSample = { name: "Roc" }

## Greeting for the sample name.
@test{fixture: helloSample}
helloRenders =
    Html.render(hello(helloSample)) == "<p>Hello, Roc</p>"

@test
helloBlock = {
    actual = Html.render(hello(helloSample))
    actual == "<p>Hello, Roc</p>"
}
```

| Piece | Role |
| --- | --- |
| `name` | Inspect / `rocci test` identity. **Not** a Roc binding. |
| `= expr` | Bool expression, copied into `expect <expr>`. |
| `{fixture: ident}` | Optional. Must name a `@fixture` in this module. Does **not** rewrite the body in v1. |
| leading `##` | Copied onto the generated `expect` so `roc test` can print it. |

**Rejected for v1**

- `@test` without a name.
- HTML / template bodies (`@test … = <p>…</p>`).
- Snapshot files or `--update` goldens.
- `{target: Component}` auto-`Html.render` sugar (fixture already has
  `target`; keep `Html.render` visible).
- Testing `@init`, handlers, SQLite, or HTTP.
- `@test` as a Rocdown document declaration.

`{fixture: …}` is the pairing, not a second language. Skills and examples
always show fixture plus test together.[^plan]

## Lowering and `rocci test`

Keep `compiled.roc` free of `expect` so wrap and `rocci run` stay valid.
Record `tests: Vec<TestInfo>`. Emit:

```roc
## Greeting for the sample name.
expect Html.render(hello(helloSample)) == "<p>Hello, Roc</p>"
```

only on the **test staging** path (and in inspect text). Staging should follow
playground/render (temp dir, `Html.roc`, sibling `.roc` copies) but **must
not** indent expects into `Type := [].{ … }`.

## Chosen staging layout (Phase 0)

On pinned nightly `2026-08-18-e9be50a`, **layout (b)** is the runner:
`wrap_type_module` writes `{Type}.roc`, then the expect trailer is appended
**after** the closing `}` of `Type := [].{ … }`. `roc test {Type}.roc` runs
those expects (one test in the stand-in). Do not put `expect` in
`main.roc`.[^phase0-probe][^playground-html][^wrap][^roc-tutorial]

Probed alternatives on the same nightly:

| Layout | Result |
| --- | --- |
| (a) Flat basic-cli `app` + `main!` + top-level `expect` + `Html.render` | `roc test main.roc` reports **208** tests (platform suite plus the expect). Works, but is the wrong identity for `rocci test`. |
| (b) Expects after `Type := [].{ … }` | `roc test Widget.roc` reports **1** test and passes. Chosen. |
| (c) Expects in `main.roc` that call `Type.fn` | Same 208-test app/platform sweep as (a). |

`Html.render` in the stand-in used the same sibling `Html.roc` as
`rocci render`. A `pf.Sqlite` import on the type module still passed
`roc test Widget.roc` without an app header. A basic-webserver `main.roc`
without `Context` failed `roc test main.roc` (`missing platform required
type`). Files that import `pf.Sqlite` therefore still use layout (b) and
must not be tested through a dummy `main.roc`. `rocci run` keeps
basic-webserver; test staging does not switch platforms to execute
expects.[^counter][^playground-html]

Default crate tests stay Roc-free. An end-to-end `rocci test` proof uses
`ROCCI_REQUIRE_ROC=1` or `#[ignore]`, consistent with other native-Roc
gates.[^language-dev]

## CSS and expected strings

Equality against `Html.render` includes injected scoped CSS when the module
has `@css`. Authors should take expected bytes from `rocci render --fragment`
or assert with `Str.contains` / a field of the fixture, not a guessed bare
tag. AllSyntax should include a non-HTML `@test` so the keyword is covered
without locking the CSS stamp into a tutorial-shaped string.[^lower][^allsyntax]

## Skills, docs, examples

| Surface | Today | After |
| --- | --- | --- |
| `docs/reference/language/` | Fixtures page; no tests page | New tests page; file-structure, fixtures, generated-roc, CLI, coverage |
| `rocci-author` | Add fixtures for `view` | Also add `@test{fixture: …}` for components |
| `rocci-language-dev` | AllSyntax + ungram | `TestDecl` production and inspect tag |
| `rocci-stack` | Widgets: component + fixture | Tests stay pure; no Datastar/handler I/O |
| Examples | Fixtures only | `@test` next to existing fixtures on styling, counter cards, and at least one UI-only module (live-counter UI or handler-matrix UI) |

Rocdown should **reserve** `test` so `@test` in a `.rocdown` file is a clear
error (“tests belong in `.rocci`”), not opaque prose. Do not parse tests in
the document compiler.[^rocdown-scan]

## Alternatives not chosen

| Idea | Why not v1 |
| --- | --- |
| JSX-style HTML as Roc, tests are just `expect` | Historical alternative; `@component` is the shipped form.[^archive-jsx] |
| Auto-snapshot `@test{fixture: x}` with no body | Hidden `Html.render` and CSS-stamp surprises. |
| Separate `Foo.test.rocci` | Fixtures already live beside components; split files fight `view`. |
| HTTP handler tests | Requires a server, `!`, and a different runner. |

[^plan]: Paired implementation sequence; not started.
[^ungram]: `ModuleItem` includes `FixtureDecl`, not a test production.
[^parser]: `try_parse_fixture` and `@` recovery keywords omit `test`.
[^lower]: Fixtures emit `name = value`; CSS scoping stamps `data-rocci-css`.
[^wrap]: `wrap_type_module` indents every non-import line into the type.
[^compile-test]: Fixture metadata and rejection tests; no test-declaration tests.
[^template-readme]: Documented top-level forms.
[^fixtures-ref]: View/browse consumers; no test runner.
[^components-guide]: “supply test data” without a test form.
[^cli-ref]: Documented rocci commands.
[^cli-main]: Clap `Commands` enum.
[^playground-html]: Snapshot `main.roc` calls `Html.render` after wrap.
[^html-roc]: `render` / `render_fragment` / `to_str` on fragments.
[^allsyntax]: `@fixture{target: Hello}`; no `@test`.
[^counter]: Fixtures for cards; SQLite in `@init` / fragments.
[^styling]: Three fixtures including `helloTest`.
[^author-skill]: Author checklist step 5 is fixtures only.
[^language-dev]: Ungram then parser; AllSyntax; no Roc in template tests.
[^highlight]: `collect_fixture` only.
[^lsp]: `fixture_symbol`; `DIRECTIVES` has no `test`.
[^rocdown-scan]: `Reserved::Fixture`; no `Test`.
[^coverage]: `syntax.fixture` current; no `syntax.test` / `cli.test`.
[^archive-jsx]: `Html.render(hello(…)) == "<p>…"`.
[^roc-tutorial]: `expect` and `roc test` on the new compiler.
[^roc-allsyntax]: One-line and block `expect`.
[^phase0-probe]: Hand-written `/tmp` stand-in on nightly-2026-08-18-e9be50a; layout (b) is 1 test.
