---
type: Implementation Plan
title: Pin Roc 2026-08-23 and emit type-position defaults
description: "Pin nightly-2026-08-23-fb208ba. Lower authored ?? to a Roc type annotation `{ name : Str ?? \"Roc\" }` and a stripped pattern; stop filling omitted fields at call sites. Pattern ?? stays illegal in generated Roc."
tags: [domain/rocci, integration/roc, concern/syntax, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-08-25T11:00:00Z }
stale_after: 2026-11-25
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../../research/rocci/roc-nightly-record-defaults.md
    title: Probe of type-position defaults versus pattern ??
    author: process:cursor
    last_modified: 2026-08-25
  - id: ast-strip
    resource: ../../../crates/rocci-template/src/ast.rs
    title: strip_param_defaults for pattern ??
    author: process:git
    last_modified: 2026-08-22
  - id: lower
    resource: ../../../crates/rocci-template/src/lower.rs
    title: Call-site default fill
    author: process:git
    last_modified: 2026-08-22
  - id: install-roc
    resource: ../../../docker/install-roc.sh
    title: Pinned Linux Roc nightly installer
    author: process:git
    last_modified: 2026-08-25
  - id: components-ref
    resource: ../../../docs/reference/language/components.rocdown
    title: Public ?? defaults contract
    author: process:git
    last_modified: 2026-08-22
---

# Pin Roc 2026-08-23 and emit type-position defaults

## Goal

CI, docs, and generated Roc use **nightly-2026-08-23-fb208ba**. Authored
`|{ name ?? "Roc" }|` lowers to a **type-position** default and a stripped
pattern. Call sites may omit the field. Pattern `??` is never copied into
generated Roc.[^research][^ast-strip]

## Out of bound

- Positional default arguments; unused-binding skip; handler Alternative B
- Rocci `?:` grammar; Datastar option records instead of tag lists
- Archive reports; inventing human verification

## Constraints that do not move

- Roc rejects `??` in patterns. Keep `strip_param_defaults` on `|…|`.[^research]
- Dispatch stays `handler!(context, request)`.
- Do not emit `_` for untyped required siblings, and do not put `Bool`
  defaults in a type annotation (both crash this Roc nightly at runtime).
  Emit a type annotation only when every required prop has an authored type
  and no default is `Bool`; otherwise keep call-site fill. Tag defaults
  without a type (`tone ?? Neutral`) are a diagnostic; authors must write
  `tone : Tone ?? Neutral`.

## Phase 1: Pin the nightly

**Bound:** `docker/install-roc.sh` date `2026-08-23` sha `fb208ba`;
`docs/inventory.toml`; `docs/install.rocdown`;
`docs/reference/compatibility.rocdown`; `docs/troubleshooting/install.rocdown`.

**Out of bound:** lowering.

**Exit:** Installer defaults match `nightly-2026-08-23-fb208ba`. Linux
nightly tarball exists on `roc-lang/nightlies`.

## Phase 2: Lowering

**Bound:** `crates/rocci-template` emit
`name : { fields } -> Html` (plus body `Html` params) when any first-record
field is defaulted; stop `field_defaults` fill in template and Rocdown
component/block calls; `rocci view` / playground omit default-only props.
Validate untyped tag defaults. Update `test/AllSyntax.rocci` types required
for that diagnostic. Snapshot `all_syntax.roc`.

**Tests:** `cargo test -p rocci-template`; `cargo test -p rocci-cli`;
`cargo test -p rocci-rocdown` if block-call fill tests exist;
`cargo fmt --all -- --check`.

**Exit:** Those commands pass. `<Hello />` lowers to `hello({})`.

## Phase 3: Public contract and remaining types

**Bound:** README and docs for `??`; remaining `.rocci` tag defaults that
fail the new diagnostic (`site/theme`, examples, EmbeddedLanguages).

**Exit:** `cargo test -p rocci-template`; docs still describe authored `??`
as sugar over Roc type-position defaults.

## Phase 4: Knowledge disposition

**Bound:** Research disposition (call-site fill gone; pin is 08-23); this
plan status; indexes; `knowledge/log.md`.

**Exit:** `cargo run -q -p rocci-okf -- check knowledge --profile rocci --format terminal`.

[^research]: Type-position `{ name : Str ?? "Roc" }` typechecks; pattern `??` does not.
[^ast-strip]: Pattern rewrite remains required.
[^lower]: Current fill of omitted props at calls.
[^install-roc]: Current pin was 2026-08-10 `7df8509`.
[^components-ref]: Public `??` stripping wording.
