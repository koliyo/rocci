---
type: Implementation Plan
title: Ungrammar AST codegen for Rocci and Rocdown
description: "Phased delivery of ungrammar as the tree spec for both languages, a shared generator of owned AST structs, and CI drift checks. Scanners and parsers stay hand-written."
tags: [domain/rocci, domain/rocdown, concern/syntax, concern/architecture, concern/tooling]
status: draft
generated: { by: process:cursor, at: 2026-08-19T16:20:00Z }
stale_after: 2026-11-19
authority: exploratory
owners: [human:nils]
sources:
  - id: follow-ons
    resource: ../ungram-follow-ons.md
    title: Ungram follow-on backends after owned-struct codegen
    author: process:cursor
    last_modified: 2026-08-19
  - id: research
    resource: ../../research/rocci/ungram-ast.md
    title: Ungrammar as AST spec for Rocci and Rocdown
    author: process:cursor
    last_modified: 2026-08-19
  - id: ungram-intro
    resource: https://rust-analyzer.github.io/blog/2020/10/24/introducing-ungrammar.html
    title: Introducing Ungrammar
    author: human:matklad
    last_modified: 2020-10-24
  - id: ungrammar-crate
    resource: https://crates.io/crates/ungrammar
    title: ungrammar crate 1.16.1
    author: organization:rust-analyzer
    last_modified: 2022-03-05
  - id: ra-arch
    resource: https://github.com/rust-lang/rust-analyzer/blob/master/docs/dev/architecture.md
    title: rust-analyzer architecture, syntax and codegen
    author: organization:rust-lang
    last_modified: 2026-08-19
  - id: rocdown-ungram
    resource: ../../../crates/rocci-rocdown/Rocdown.AST.ungram
    title: Draft Rocdown document AST ungrammar
    author: process:cursor
    last_modified: 2026-08-19
  - id: rocdown-ast
    resource: ../../../crates/rocci-rocdown/src/ast.rs
    title: Shipped Rocdown document AST
    author: process:git
    last_modified: 2026-08-19
  - id: template-ast
    resource: ../../../crates/rocci-template/src/ast.rs
    title: Shipped Rocci template AST
    author: process:git
    last_modified: 2026-08-17
  - id: template-readme
    resource: ../../../crates/rocci-template/README.md
    title: Implemented Rocci template language reference
    author: process:git
    last_modified: 2026-08-17
  - id: rocdown-readme
    resource: ../../../crates/rocci-rocdown/README.md
    title: Implemented Rocdown language reference
    author: process:git
    last_modified: 2026-08-19
  - id: ast-test
    resource: ../../../crates/rocci-rocdown/tests/ast.rs
    title: Rocdown AST inspect fixture and ungram name drift test
    author: process:git
    last_modified: 2026-08-19
  - id: block-research
    resource: ../../research/rocdown/generalized-rocdown-block-model.md
    title: Generalized Rocdown block model research
    author: process:cursor
    last_modified: 2026-08-19
  - id: block-plan
    resource: ../generalized-rocdown-block-model.md
    title: Generalized Rocdown block model implementation plan
    author: process:cursor
    last_modified: 2026-08-19
  - id: language-dev
    resource: ../../../.agents/skills/rocci-language-dev/SKILL.md
    title: Rocci and Rocdown language-development skill
    author: process:git
    last_modified: 2026-08-18
  - id: workspace-deps
    resource: ../../../tools/rocci-ops/src/rocci_ops/workspace_deps.py
    title: Workspace package class and edge checker
    author: process:git
    last_modified: 2026-08-18
  - id: cargo-workspace
    resource: ../../../Cargo.toml
    title: Cargo workspace manifest
    author: process:git
    last_modified: 2026-08-18
  - id: product-boundary
    resource: ../../decisions/consolidate-rocdown-product-boundary.md
    title: Consolidate the Rocdown format and documentation generator
    author: process:cursor
    last_modified: 2026-08-17
  - id: template-lib
    resource: ../../../crates/rocci-template/src/lib.rs
    title: rocci-template public AST exports
    author: process:git
    last_modified: 2026-08-18
  - id: rocdown-lib
    resource: ../../../crates/rocci-rocdown/src/lib.rs
    title: rocci-rocdown public AST exports
    author: process:git
    last_modified: 2026-08-19
---

# Ungrammar AST codegen for Rocci and Rocdown

## Purpose and authority

This is the implementation plan for [Ungrammar as AST spec for Rocci and
Rocdown](/research/rocci/ungram-ast.md). It is exploratory until a human
reviewer accepts a scope. It does not describe shipped behavior. Crate
READMEs remain the current language contract.[^research][^template-readme][^rocdown-readme]

Do not start a phase until the user asks to implement it. Use the
`rocci-language-dev` skill for grammar, AST, and inspect-fixture work.
This plan does not change source syntax. Ungrammar specifies tree shape,
not the language of strings; that is why scanners stay
hand-written.[^language-dev][^ungram-intro][^block-research]

## Goal

Make ungrammar the tree spec for both languages and generate the owned
Rust AST data types from it, so node shapes cannot drift from the spec.

After the last phase:

- `crates/rocci-template/Rocci.AST.ungram` describes the shipped template
  tree.
- `crates/rocci-rocdown/Rocdown.AST.ungram` describes the shipped document
  tree (article + module nodes; Markdown remains `MdNode` until a later
  tree change).
- `cargo run -p rocci-ungram -- generate` writes committed
  `ast.generated.rs` files.
- `cargo run -p rocci-ungram -- check` (or the equivalent test) fails when
  generated Rust or ungrams are stale.
- Scanners, parsers, `format_ast`, and helper methods stay hand-written.

## Constraints that do not move

| Keep | Meaning for this plan |
| --- | --- |
| Hand-written scanners and parsers | Ungram is not a parser generator |
| Owned AST with byte `Span` | No `rowan` / CST |
| Product boundary | `rocci-template` does not depend on `rocci-rocdown`; Rocdown reuses Rocci types for islands |
| Comrak-owned Markdown | Do not generate a Markdown parser or replace `MdNode` in v1 |
| Ordinary Roc opaque | `RocRegion` / `RocExpr` / `BracePayload` stay leaves |
| Default tests sub-second | No ungram parse during `cargo test -p rocci-template` via `build.rs` |
| OKF Markdown-only | Do not add ungram to `knowledge/**/*.md` |

The [generalized Rocdown block model plan](/plans/rocdown/generalized-rocdown-block-model.md)
owns article-block *syntax* and renderers. This plan owns the *tree spec
and struct codegen*. If that plan still needs to reshape `Item` versus
`MdNode`, finish the runtime tree before generating those productions, or
mark them foreign in the ungram.[^block-plan][^product-boundary]

## Non-goals (all phases)

- Generating scanners, lexers, or parsers
- `rowan`, lossless CSTs, or rust-analyzer-style `SyntaxToken` accessors
- Generating `format_ast` / inspect S-expressions
- Generating LSP `SyntaxKind` or highlight token enums
- A Markdown ungram that replaces `MdNode`
- A Roc ungram
- Parser generators (pest, lalrpop, tree-sitter)
- Changing `.rocci` or `.rocdown` source spelling
- OKF YAML AST

## V1 contract

These answers freeze the research open questions for delivery. Changing
one is a plan revision.[^research]

### Files

| File | Role |
| --- | --- |
| `crates/rocci-template/Rocci.AST.ungram` | Template tree spec |
| `crates/rocci-rocdown/Rocdown.AST.ungram` | Document tree spec |
| `crates/rocci-rocdown/Rocdown.AST.toml` (or equivalent sidecar) | Foreign node map |
| `crates/rocci-ungram` | Generator CLI, classified as base-rocci |
| `src/ast.generated.rs` in each language crate | Committed output, `// @generated` |
| `src/ast.rs` | `mod ast_generated`, hand-written `impl`s, analysis types |

### Lowering rules

- Top-level `\|` of node names → enum of tuple variants matching today
  (`Item::Block(BlockCall)`).
- Sequence of labeled fields → struct. Unlabeled tokens are omitted.
- `T*` → `Vec<T>`. `T?` → `Option<T>`. Recursive struct fields → `Box`.
- Every struct gets `pub span: Span`. Enums get `fn span(&self) -> Span`.
- Leaf `Name = 'token'` uses a sidecar rust type (`Ident`, opaque
  `Span`, or `String` + `Span`). Default is not guessed silently.
- Derives: `Clone, Debug, PartialEq, Eq`.
- Public names stay the current `rocci_template` / `rocci_rocdown`
  exports.

### Generator invocation

```text
cargo run -q -p rocci-ungram -- generate
cargo run -q -p rocci-ungram -- check
```

`--check` is the CI lock. Language crates do not depend on `rocci-ungram`
or `ungrammar`. Generated files are the only compile-time artifact,
committed like rust-analyzer’s codegen output rather than produced by
`build.rs`.[^ra-arch]

## Layer map

| Concern | Owner |
| --- | --- |
| Ungram DSL parse | `ungrammar` crate, used only by `rocci-ungram` |
| Dialect restrictions, owned-struct emit, sidecars | `crates/rocci-ungram` |
| Template ungram and generated nodes | `crates/rocci-template` |
| Document ungram, foreign map, generated nodes | `crates/rocci-rocdown` |
| Scanners, parsers, lowering | unchanged owning files |
| `format_ast` | `pprint.rs` in each crate |
| Workspace class | `tools/rocci-ops/src/rocci_ops/workspace_deps.py` `BASE_ROCCI` |
| Language-dev loop | crate README plus ungram; still run `inspect --ast` |

## Delivery phases

Each phase is one mergeable change. Do not start one until asked.

### Phase 1 — Dialect and both ungrams against shipped trees

**Bound:** write `Rocci.AST.ungram` and tighten `Rocdown.AST.ungram` so
every *generated* production matches types the parsers construct today.
Document lowering rules in the ungram headers. Add a Rocdown sidecar that
lists foreign / opaque nodes. No generator crate yet.

**Include:**

- `Rocci.AST.ungram` covering `Document` / `ModuleItem` / template nodes
  from `rocci-template/src/ast.rs`, with Roc payloads as leaves.
- Rocdown ungram comments: which productions are generated, which are
  documentation (`Paragraph` / `Inline`), which are foreign
  (`ComponentDecl` → `rocci_template::ComponentDecl`).
- Do not invent `Item::Paragraph` in Rust.

**Done when:** a reviewer can walk from ungram production to a shipped
Rust type (or to an explicit `foreign` / `doc-only` mark) without
guessing. Existing `ungram_article_call_productions_exist_as_rust_types`
still passes.[^rocdown-ungram][^template-ast][^rocdown-ast][^ast-test]

**Status:** implemented on `ungram-ast-implementation` (ungrams + sidecars).
Not CI-complete. Generator crate is Phase 2.

### Phase 2 — `rocci-ungram` crate, snapshots only

**Bound:** add the generator. Emit owned structs into *snapshot fixtures
inside `rocci-ungram` tests*, not into the language crates. Classify the
crate as base-rocci.

**Include:**

- Workspace member + `CLASSES` entry.
- `generate` / `check` CLI.
- Tests: parse both ungrams; reject illegal rules; snapshot emitted Rust
  for a small fixture ungram and for the two real files.
- Dependency on `ungrammar`; no reverse edge into Rocdown from base Rocci
  at runtime (the CLI may *read* Rocdown’s ungram path as a file).

**Done when:** `cargo test -p rocci-ungram` is sub-second and snapshots
the would-be `ast.generated.rs` text. Language crates still compile their
hand-written `ast.rs`.[^workspace-deps][^cargo-workspace][^ungrammar-crate]

**Status:** implemented on `ungram-ast-implementation` (`rocci-ungram`
generate/check + snapshots). Not CI-complete. Language cutover is Phase 3.

### Phase 3 — Cut over `rocci-template`

**Bound:** replace hand-written template data types with generated ones.
Keep `impl` methods, `ParsedParams`, and `parse_component_params` in
`ast.rs`.

**Include:**

- `include!` or `mod ast_generated` from committed `ast.generated.rs`.
- Re-exports unchanged.
- `cargo test -p rocci-template` and
  `cargo run -q -p rocci-cli -- inspect --ast test/AllSyntax.rocci`.

**Done when:** template parsers, lowerers, and inspect tests use generated
types with no public API rename. `rocci-template` still has no
`rocci-rocdown` or `ungrammar` dependency.[^template-lib][^language-dev][^product-boundary]

**Status:** implemented on `ungram-ast-implementation` (`ast.generated.rs`
plus hand-written helpers in `ast.rs`). Not CI-complete. Rocdown cutover
is Phase 4.

### Phase 4 — Cut over Rocdown non-`MdNode` nodes

**Bound:** generate `Document` / `Item` / article-call / param types.
Leave `MdNode`, `PageMeta`, `HeadingInfo`, `LinkInfo` hand-written.
Foreign Rocci decls stay `rocci_template::*`.

**Include:**

- Move `BlockCall` helpers into `impl` on the generated struct.
- Delete the name-only drift test; `--check` replaces it.
- `cargo test -p rocci-rocdown` and
  `cargo run -q -p rocci-rocdown-cli -- inspect ast test/AllSyntax.rocdown`.

**Done when:** Rocdown compiles against generated article/module nodes
and still produces `Item::Markdown(MdNode)` for Comrak blocks.
Inspect fixtures do not change unless a field was already drifting
(treat unexpected inspect diffs as a bug).[^rocdown-lib][^ast-test][^rocdown-ast]

**Status:** implemented on `ungram-ast-implementation` (`ast.generated.rs`
plus hand-written `MdNode`, analysis types, and `BlockCall` helpers).
Not CI-complete. CI `--check` is Phase 5.

### Phase 5 — CI `--check` and docs

**Bound:** wire `rocci-ungram --check` into the lint or test job; mention
the ungrams in both crate READMEs as the tree spec (not the language
reference). Update `rocci-language-dev` to edit the ungram before AST
fields.

**Include:**

- README sentences: generate nodes, do not generate the scanner.
- Optional: a short pointer from `docs/reference/*.rocdown` that the
  developer tree spec lives in the ungram files. Do not paste the ungram
  into knowledge records.

**Done when:** a stale `ast.generated.rs` fails CI, and language-dev
instructions name the ungram as the first edit for a new node.
Architecture records stay descriptive and are updated only after this
phase ships, not in the plan commit.

**Status:** implemented on `ungram-ast-implementation` (CI lint `--check`,
crate README tree-spec notes, language-dev ungram-first). Not CI-complete
until required GitHub workflows succeed on this revision.

## Follow-ons (not v1)

Delivery is the [ungram follow-on backends plan](ungram-follow-ons.md).
Sequence: freeze inspect tags → generate exhaustive `format_ast`
walkers → `NodeKind` highlighter coverage → `Rocdown.Markdown.ungram`
for shipped `MdNode` → public tree appendix. Do not generate
rust-analyzer `SyntaxKind`, highlight collectors, scanners, or a
CST.[^follow-ons]

## Validation

Per phase:

```text
cargo test -p rocci-ungram
cargo test -p rocci-template
cargo test -p rocci-rocdown
cargo fmt --all -- --check
cargo run -q -p rocci-ungram -- check
```

After Phase 3:

```text
cargo run -q -p rocci-cli -- inspect --ast test/AllSyntax.rocci
```

After Phase 4:

```text
cargo run -q -p rocci-rocdown-cli -- inspect ast test/AllSyntax.rocdown
```

After knowledge edits:

```text
cargo run -q -p rocci-okf -- check knowledge --profile rocci --format terminal
```

Cross-cutting (workspace member + CI): `cargo test --workspace`. Do not
set `ROCCI_REQUIRE_ROC=1`. Do not log a phase complete in
`knowledge/log.md` until required GitHub workflows have succeeded on
that revision.

## Open questions that would still change the plan

1. Sidecar as TOML versus generate-command flags for foreign / leaf Rust
   types.
2. Whether `Element.self_closing` is a generated `BoolLit` field or stays
   in a hand-written `impl`.
3. Whether public docs should link the ungrams in Phase 5 or stay
   developer-only until a Markdown ungram exists. Moved to the
   [follow-on plan](ungram-follow-ons.md): generated name/tag appendix,
   not a raw ungram paste.

[^follow-ons]: Inspect exhaustiveness first; NodeKind not SyntaxKind; Markdown ungram generates MdNode; CST deferred.
[^research]: Recommendation: ungram as tree spec; generate owned structs; no scanner, parser, or CST.
[^ungram-intro]: Ungrammar describes trees and pairs with a hand-written parser.
[^ungrammar-crate]: Parser-of-ungram library; not a parser generator.
[^ra-arch]: Committed codegen, no bootstrap, syntax crate independent of the rest of the tool.
[^rocdown-ungram]: Existing draft; generate nodes not scanner; mixed generated / opaque / doc-only productions.
[^rocdown-ast]: Hybrid Markdown + module + `BlockCall` tree and analysis types.
[^template-ast]: Template nodes and param helpers in one hand-written file.
[^template-readme]: Shipped `.rocci` contract.
[^rocdown-readme]: Shipped `.rocdown` contract.
[^ast-test]: Name-only drift test to replace with `--check`.
[^block-research]: Left Rocci ungram optional and AST codegen as later work.
[^block-plan]: Follow-on “generate AST types from the ungram”; this plan owns that follow-on.
[^language-dev]: Inspect fixtures, monotonic scanners, owning crates.
[^workspace-deps]: New members must be classified; base Rocci must not depend on Rocdown packages.
[^cargo-workspace]: Member list to extend with `rocci-ungram`.
[^product-boundary]: Template language stays in `rocci-template`; Rocdown reuses it.
[^template-lib]: Public template AST exports.
[^rocdown-lib]: Public document AST exports including re-exported template types.
