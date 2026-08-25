---
type: Implementation Plan
title: Implement @test and rocci test
description: "Add root-only @test name = boolExpr with optional {fixture: ident}, lower to Roc expect outside wrap_type_module, add rocci test, then update skills, docs, AllSyntax, and example apps that already have @fixture."
tags: [domain/rocci, integration/roc, concern/syntax, concern/testing, concern/docs]
status: draft
generated: { by: process:cursor, at: 2026-08-25T12:30:00Z }
stale_after: 2026-11-25
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../../research/rocci/rocci-test-syntax.md
    title: Rocci @test syntax lowered to Roc expect
    author: process:cursor
    last_modified: 2026-08-25
  - id: ungram
    resource: ../../../crates/rocci-template/Rocci.AST.ungram
    title: Tree spec; add TestDecl beside FixtureDecl
    author: process:git
    last_modified: 2026-08-22
  - id: ast-toml
    resource: ../../../crates/rocci-template/Rocci.AST.toml
    title: Inspect tags and pprint overlays for FixtureDecl
    author: process:git
    last_modified: 2026-08-22
  - id: parser
    resource: ../../../crates/rocci-template/src/parser.rs
    title: Top-level declaration dispatch and @ keyword lists
    author: process:git
    last_modified: 2026-08-23
  - id: validate
    resource: ../../../crates/rocci-template/src/validate.rs
    title: Fixture target validation
    author: process:git
    last_modified: 2026-08-25
  - id: lower
    resource: ../../../crates/rocci-template/src/lower.rs
    title: Fixture lowering and FixtureInfo; keep expect out of roc body
    author: process:git
    last_modified: 2026-08-25
  - id: wrap
    resource: ../../../crates/rocci-template/src/roc.rs
    title: wrap_type_module used by run, view, browse, render
    author: process:git
    last_modified: 2026-08-16
  - id: compile-rs
    resource: ../../../crates/rocci-template/tests/compile.rs
    title: Template compile tests including fixtures
    author: process:git
    last_modified: 2026-08-25
  - id: language-dev
    resource: ../../../.agents/skills/rocci-language-dev/SKILL.md
    title: Ungram, parser, AllSyntax, docs order
    author: process:git
    last_modified: 2026-08-22
  - id: author-skill
    resource: ../../../.agents/skills/rocci-author/SKILL.md
    title: Authoring recognized forms and fixture checklist
    author: process:git
    last_modified: 2026-08-23
  - id: author-idioms
    resource: ../../../.agents/skills/rocci-author/idioms.md
    title: Fixture authoring idiom
    author: process:git
    last_modified: 2026-08-25
  - id: stack-skill
    resource: ../../../.agents/skills/rocci-stack/SKILL.md
    title: Widgets are pure component plus fixture
    author: process:git
    last_modified: 2026-08-24
  - id: fixtures-ref
    resource: ../../../docs/reference/language/fixtures.rocdown
    title: Public fixture page to cross-link tests
    author: process:git
    last_modified: 2026-08-22
  - id: lang-index
    resource: ../../../docs/reference/language/index.rocdown
    title: Language reference map
    author: process:git
    last_modified: 2026-08-22
  - id: cli-ref
    resource: ../../../docs/reference/cli.rocdown
    title: rocci CLI reference
    author: process:git
    last_modified: 2026-08-22
  - id: coverage
    resource: ../../../docs/coverage.toml
    title: Feature coverage manifest
    author: process:git
    last_modified: 2026-08-22
  - id: allsyntax
    resource: ../../../test/AllSyntax.rocci
    title: Comprehensive syntax fixture
    author: process:git
    last_modified: 2026-08-25
  - id: playground-html
    resource: ../../../crates/rocci-cli/src/playground_html.rs
    title: Existing Html.render staging to reuse for rocci test
    author: process:git
    last_modified: 2026-08-25
  - id: cli-main
    resource: ../../../crates/rocci-cli/src/main.rs
    title: Clap Commands enum
    author: process:git
    last_modified: 2026-08-24
  - id: highlight
    resource: ../../../crates/rocci-highlight/src/composite.rs
    title: collect_fixture highlighter
    author: process:git
    last_modified: 2026-08-22
  - id: lsp
    resource: ../../../crates/rocci-lsp/src/analysis.rs
    title: fixture_symbol and DIRECTIVES
    author: process:git
    last_modified: 2026-08-22
  - id: rocdown-scan
    resource: ../../../crates/rocci-rocdown/src/scan.rs
    title: Reserved @ names
    author: process:git
    last_modified: 2026-08-22
  - id: styling
    resource: ../../../examples/rocci/standalone/styling/Styling.rocci
    title: Catalog example with helloTest fixture
    author: process:git
    last_modified: 2026-08-23
  - id: counter
    resource: ../../../examples/rocci/standalone/counter/Counter.rocci
    title: Starter app with CounterCard fixtures
    author: process:git
    last_modified: 2026-08-23
  - id: roc-tutorial
    resource: https://github.com/roc-lang/roc/blob/main/docs/mini-tutorial-new-compiler.md
    title: Roc expect and roc test
    author: organization:roc-lang
---

# Implement @test and rocci test

## Goal

Authors write **`@test`** next to **`@fixture`** in `.rocci` files. Lowering
emits Roc **`expect`** that **`rocci test`** runs via **`roc test`**.
`rocci run` / `view` / `browse` keep working because expects never enter
`wrap_type_module`.[^research][^wrap]

## Out of bound

- HTML or template bodies on `@test`
- Snapshot files, `--update`, or golden HTML trees
- `{target: …}` auto-`Html.render` sugar
- Handler, `@init`, SQLite, HTTP, or Datastar tests
- `@test` as a Rocdown executable declaration (reserve the word only)
- Testing `rocci` itself by putting `@test` in compiler crates
- Replacing `rocdown test`
- Bare `expect` as the documented authoring form
- Inventing `Html` equality

## Constraints that do not move

- Root-only `@test`, same recovery rules as `@fixture`.[^parser]
- Name is inspect/CLI identity, not a Roc binding.
- Body is a Bool Roc expression (`scan_roc_expr`, blocks allowed).
- Optional `{fixture: ident}` must name a `@fixture` in this module; unknown
  attributes error like `@fixture`.[^validate]
- `compiled.roc` used for wrap **must not** contain `expect`.[^wrap][^lower]
- `@component` stays pure. Test bodies must not require `!`.
- Parser/lowering tests do not invoke Roc. Native `roc test` proofs are
  `ROCCI_REQUIRE_ROC` or `#[ignore]`.[^language-dev]
- File `@css` stamps belong in expected `Html.render` strings, or tests avoid
  full-render equality.[^research]
- Ungram + sidecar first, then `rocci-ungram generate`; do not hand-edit
  `ast.generated.rs`.[^language-dev]

## Phase 0: Probe `roc test` staging

**Bound:** Record which generated layout this Roc nightly accepts: (a) flat
module + basic-cli `main!` + top-level `expect` + `Html.render`; (b) expects
after `Type := [].{ … }`; (c) expects in `main.roc` that call `Type.fn`.
Note platform choice when the source imports `pf.Sqlite`. Amend
[the research](/research/rocci/rocci-test-syntax.md) with the chosen layout.
No grammar change.

**Out of bound:** `@test` parser.

**Exit:** Research names one staging layout. `roc test` on a hand-written
stand-in of that layout passes on the pinned nightly.[^roc-tutorial][^playground-html]

## Phase 1: Tree, parse, validate

**Bound:** `TestDecl` on `ModuleItem` in `Rocci.AST.ungram` /
`Rocci.AST.toml` (inspect tag `test`, pprint overlay).
`cargo run -q -p rocci-ungram -- generate`. Parser
`try_parse_test` beside fixture; add `test` to keyword and
`at_column_zero_def` lists. Validate: missing name/`=`, unknown attrs,
duplicate `fixture`, unknown fixture name, `@test` inside a component body.
`CompileOutput` / `LoweredModule` may grow `tests: Vec<TestInfo>` as empty.
Tests in `compile.rs` for accept/reject/AST only.

**Out of bound:** emitting `expect`; CLI.

**Tests:** `cargo test -p rocci-template --test compile`;
`cargo run -q -p rocci-ungram -- check`; `cargo fmt --all -- --check`.

**Exit:** Those commands pass. `format_ast` prints
`(test helloRenders fixture:helloSample (roc "…"))`.

## Phase 2: Lowering, inspect, AllSyntax

**Bound:** `lower_test` records `TestInfo` and **does not** write `expect`
into `compiled.roc`. Helper that formats the expect trailer (leading `##`,
`expect <expr>`). `rocci inspect` lists `# tests`. AllSyntax: one Bool
`@test` and one `{fixture: …}` that does not snapshot scoped CSS (fixture
field or `Str.contains`). Update `all_syntax.roc` only if the trailer is
part of that golden; prefer keeping the golden wrap-safe (no `expect` in
the module body).

**Tests:** `cargo test -p rocci-template`;
`cargo run -q -p rocci-cli -- inspect --ast test/AllSyntax.rocci`.

**Exit:** Those commands pass. Generated module body still has no `expect`.

## Phase 3: `rocci test`

**Bound:** `rocci test [PATH]` in `rocci-cli`. File or directory of `.rocci`.
Stage with Phase 0 layout; copy `Html.roc` and siblings like render.
Skip files with no tests (exit 0). Non-zero if Rocci diagnostics or `roc test`
fails. Clap/unit tests without Roc: argv, empty-dir, trailer text. One
`#[ignore]` or `ROCCI_REQUIRE_ROC` test that runs `rocci test` on a tiny
fixture file.

**Out of bound:** docs site, example apps.

**Tests:** `cargo test -p rocci-cli`; `cargo fmt --all -- --check`.

**Exit:** Those commands pass. `rocci test` on the tiny fixture is green when
Roc is required.

## Phase 4: Highlight, LSP, Rocdown reserve

**Bound:** `collect_test` in `rocci-highlight`. LSP `test_symbol`, outline,
`DIRECTIVES` / completion include `test`. Rocdown `Reserved::Test`: diagnose
that tests belong in `.rocci` (docs pipeline and interactive). No Rocdown
`TestDecl` lowering.

**Tests:** `cargo test -p rocci-highlight`; `cargo test -p rocci-lsp`;
`cargo test -p rocci-rocdown`; `cargo fmt --all -- --check`.

**Exit:** Those commands pass. `@test` highlights and outlines in `.rocci`.

## Phase 5: Docs, coverage, skills

**Bound:** New `docs/reference/language/tests.rocdown`. Link from language
index, file-structure, fixtures, generated-roc, grammar, CLI, glossary,
`docs/templates/components.rocdown`, `docs/install.rocdown` if it still says
fixtures are “possibly for tests”. `docs/coverage.toml`: `syntax.test` and
`cli.test`. Crate READMEs: `rocci-template`, `rocci-cli`. Skills:
`rocci-author` (recognized forms, checklist, idioms), `rocci-language-dev`
(AllSyntax mentions tests), `rocci-stack` (tests stay pure). Coverage
contract includes `test/AllSyntax.rocci`.

**Out of bound:** rewriting example apps (Phase 6).

**Exit:** `cargo run -q -p rocci-rocdown-cli -- build docs` succeeds for the
new page. Coverage lists the new features as current/experimental as labeled
on the page.

## Phase 6: Example applications

**Bound:** Add `@test{fixture: …}` beside existing fixtures in:

- `examples/rocci/standalone/styling/Styling.rocci` (`helloTest` at minimum)
- `examples/rocci/standalone/counter/Counter.rocci` (`counterCardTest`; not
  SQLite handlers)
- One UI-only module: `LiveCounterUi.rocci` or `HandlerMatrixUi.rocci`

Take expected `Html.render` bytes from `rocci render --fragment` when the
file has `@css`, or assert a stable substring / fixture field. Mention tests
in that app’s `index.rocdown` in one sentence. Do not add tests to snake
lock logic or `@init`.

**Tests:** `cargo test -p rocci-template`; with Roc,
`rocci test` on those three files.

**Exit:** Those examples compile. `rocci test` on them is green when Roc is
available. `rocci view` still previews the same fixtures.

## Phase 7: Knowledge disposition

**Bound:** Research disposition (chosen staging, shipped syntax). This plan
status. Indexes. `knowledge/log.md` only after CI and Knowledge workflows
succeed if a phase-complete claim is made.

**Exit:** `cargo run -q -p rocci-okf -- check knowledge --profile rocci --format terminal`.

[^research]: Syntax, wrap hazard, fixture pairing, CSS stamps.
[^wrap]: Type-module wrap used by run/view/browse/render.
[^parser]: Fixture parse and `@` keyword lists.
[^validate]: Unknown fixture target pattern to copy.
[^lower]: Fixture bindings stay in the Roc body; tests must not.
[^language-dev]: Ungram-first; no Roc in template tests.
[^roc-tutorial]: `expect` / `roc test`.
[^playground-html]: Existing staging to copy, not to indent expects into.
[^ast-toml]: Inspect tag `fixture` pattern for `test`.
[^compile-rs]: Fixture tests as the template for `@test` cases.
[^author-skill]: Add `@test` to recognized forms.
[^author-idioms]: Fixture samples.
[^stack-skill]: Keep tests off the handler/Datastar path.
[^fixtures-ref]: Cross-link.
[^lang-index]: New row.
[^cli-ref]: Document `rocci test`.
[^coverage]: New feature ids.
[^allsyntax]: Keyword coverage without CSS-golden HTML.
[^cli-main]: New subcommand.
[^highlight]: Parallel `collect_fixture`.
[^lsp]: Parallel `fixture_symbol`.
[^rocdown-scan]: Reserve `test`.
[^styling]: `helloTest`.
[^counter]: `counterCardTest`.
