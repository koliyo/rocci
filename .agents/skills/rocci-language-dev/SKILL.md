---
name: rocci-language-dev
description: Develop and review Rocci's `.rocci` template language and `.rocdown` document language. Use for grammar, scanning, parsing, AST, validation, lowering, generated Roc, diagnostics, recovery, source maps, syntax fixtures, or language-reference changes in `rocci-template` and `rocci-rocdown`. Do not use for pure runtime, desktop, Rocdown-site, or knowledge-bundle work unless the language change directly requires it.
---

# Rocci Language Development

Change Rocci and Rocdown as source languages while preserving their boundary
with ordinary Roc, Markdown, runtime hosting, and Rocdown static-site behavior.
Use existing compiler entry points and fixtures instead of creating wrapper
scripts.

## Establish the contract

1. Work from the repository root and inspect `git status --short`. Preserve
   unrelated changes, especially modified language documentation and fixtures.
2. Read `crates/rocci-template/README.md` for `.rocci` behavior and
   `crates/rocci-rocdown/README.md` for `.rocdown` behavior.
3. Read `crates/rocci-template/Rocci.AST.ungram` or
   `crates/rocci-rocdown/Rocdown.AST.ungram` for the owned tree spec. Those
   files generate `src/ast.generated.rs`; they do not generate the scanner or
   parser.
4. Read the relevant existing tests before editing implementation code:
   `crates/rocci-template/tests/compile.rs` or
   `crates/rocci-rocdown/tests/compile.rs`.
5. Consult `knowledge/architecture/rocdown-format.md` and the applicable
   decision record when semantics or language boundaries may change. Invoke
   `$manage-rocci-knowledge` as well if the task requires editing canonical
   knowledge.
6. State whether the change is implemented behavior, an approved direction,
   or an experiment. Do not present deferred syntax as shipped.

## Choose the owning layer

- Put shared Rocci template syntax, AST nodes, validation, lowering, source-map
  kinds, and diagnostics in `crates/rocci-template`.
- Put Rocdown document-boundary scanning in `crates/rocci-rocdown/src/scan.rs`,
  document parsing in `parse.rs`, Markdown behavior in `markdown.rs`, page
  metadata in `page.rs`, page-link behavior in `links.rs`, and document
  lowering in `lower.rs`.
- Reuse exported `rocci-template` parsing and lowering for Rocci regions inside
  Rocdown. Do not implement a second template grammar in `rocci-rocdown`.
- Keep Roc compilation, type checking, servers, desktop hosting, and static-site
  catalog policy outside the two language crates. Change consumers only when
  their public integration contract is affected.

## Preserve language invariants

- Keep ordinary Roc outside recognized `.rocci` declarations unchanged.
- Keep `@component` as a pure render abstraction that lowers to an ordinary Roc
  function. Do not add hidden persistence, request lifecycle, or client state.
- Keep Rocdown Markdown-first. Recognize executable declarations only at the
  documented root boundary; keep prose, email addresses, lists, quotations,
  and fenced examples non-executable.
- Keep raw Markdown HTML disabled by default and keep `@island` unimplemented
  unless the task explicitly approves and implements that language feature.
- Keep parser and lowering tests independent of servers. Do not invoke Roc from
  compiler-core tests unless the test explicitly verifies a generated-Roc
  compatibility contract.
- Ensure all lexical scanners, cursors, and token skippers guarantee monotonic
  forward progress (`cur.pos > before` or `cur.bump()`) on every branch, even on
  malformed, multiline, or unclosed delimiters, to prevent CPU-spinning infinite loops.
- Preserve byte spans through scanning and parsing. Update source-map segments
  whenever generated text or origin ownership changes.
- Emit actionable diagnostics at the narrowest source span and preserve parser
  recovery so one malformed declaration does not hide unrelated later errors.

## Implement the change

1. Add or update a focused regression test that demonstrates the source input,
   AST or diagnostic outcome, generated Roc, and source-map behavior relevant
   to the change.
2. When the syntax introduces a new semantic shape, edit the language ungram
   and sidecar first (`Rocci.AST.ungram` / `Rocdown.AST.ungram` and the sibling
   `*.AST.toml`), then `cargo run -q -p rocci-ungram -- generate`. Do not
   hand-edit `ast.generated.rs`. Then add parser branches. Avoid encoding new
   syntax as unrelated existing nodes.
3. Update scanning and parsing boundary cases together. Test the accepted form,
   malformed near-misses, nesting restrictions, indentation, and literal or
   fenced forms that must remain inert.
4. Update validation separately from parsing when the source is structurally
   valid but semantically forbidden.
5. Update lowering and source-map emission together. Inspect generated Roc for
   both valid and recovery paths.
6. Update `format_ast` output when the AST changes so `inspect --ast` remains a
   useful debugging contract.
7. Update `test/AllSyntax.rocci` or `test/AllSyntax.rocdown` and the matching
   generated fixture only when the feature belongs in the comprehensive syntax
   example. Review generated fixture changes rather than accepting them blindly.
8. Update the owning crate README and the corresponding
   `docs/reference/rocci.rocdown` or `docs/reference/rocdown.rocdown` page when
   the public language contract changes.

## Validate progressively

Run one exact integration test while iterating:

```sh
cargo test -p rocci-template --test compile TEST_NAME -- --exact
cargo test -p rocci-rocdown --test compile TEST_NAME -- --exact
```

Run the full owning packages after the focused test passes:

```sh
cargo test -p rocci-template
cargo test -p rocci-rocdown
```

Inspect the comprehensive source, AST, diagnostics, generated Roc, and maps:

```sh
cargo run -q -p rocci-cli -- inspect --ast test/AllSyntax.rocci
cargo run -q -p rocci-rocdown-cli -- inspect ast test/AllSyntax.rocdown
```

Also run:

```sh
cargo run -q -p rocci-ungram -- check
cargo fmt --all -- --check
```

Run `cargo test -p rocci-lsp` when diagnostics, positions, symbols, completion,
or semantic tokens may change (default tests complete in <2s; use
`cargo test -p rocci-lsp --test fuzz_invariants -- --ignored` for deep mutation
fuzzing and `cargo test -p rocci-lsp --test perf --release -- --nocapture --ignored`
for latency benchmarks). Run `cargo test -p rocci-cli` when compilation
metadata or CLI inspection output changes. Run `cargo test --workspace` for a
cross-format or cross-consumer change. Set `ROCCI_REQUIRE_ROC=1` only when the
task must prove compatibility with the pinned Roc toolchain.

If a test or build task takes unexpectedly long (>5s for focused unit tests),
check `manage_task` with `Action: 'list'` and terminate stuck or hanging tasks
immediately with `manage_task(Action: 'kill')` before retrying.

Build `docs` with `cargo run -q -p rocci-rocdown-cli -- build docs` when public language
documentation changes and inspect the affected generated page.

## Report results

- Describe the accepted and rejected syntax before summarizing implementation.
- Name the owning parser, validator, lowerer, and consumers changed.
- Call out generated-Roc or source-map changes explicitly.
- List focused, package, consumer, and workspace validation separately.
- Identify deferred integrations instead of silently expanding the task into
  runtime, LSP, Rocdown, or knowledge work.
