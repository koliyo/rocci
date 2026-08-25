---
type: Research Report
title: Roc nightly 2026-08-23 record defaults versus Rocci ??
description: "Nightly 2026-08-23-fb208ba typechecks defaulted and optional record fields in type position. Pattern ?? is still illegal. Rocci emits type annotations and stripped patterns; it does not copy ?? into generated parameter lists."
tags: [domain/rocci, integration/roc, concern/syntax, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-08-25T10:45:00Z }
stale_after: 2026-11-25
authority: exploratory
owners: [human:nils]
sources:
  - id: plan
    resource: ../../plans/rocci/roc-nightly-defaults.md
    title: Pin Roc and emit type-position defaults
    author: process:cursor
    last_modified: 2026-08-25
  - id: ast-strip
    resource: ../../../crates/rocci-template/src/ast.rs
    title: strip_param_defaults workaround for ?? in record patterns
    author: process:git
    last_modified: 2026-08-22
  - id: template-readme
    resource: ../../../crates/rocci-template/README.md
    title: Rocci ?? defaults and Datastar option records
    author: process:git
    last_modified: 2026-08-23
  - id: components-ref
    resource: ../../../docs/reference/language/components.rocdown
    title: Public ?? defaults stripped for current Roc nightly
    author: process:git
    last_modified: 2026-08-22
  - id: optional-request
    resource: ./optional-handler-request.md
    title: Handler request follow-ons that depend on Roc record openness
    author: process:cursor
    last_modified: 2026-08-20
  - id: block-renderers
    resource: ../rocdown/rocdown-block-renderers.md
    title: Earlier claim that Roc nightly has no optional record fields
    author: process:cursor
    last_modified: 2026-08-19
  - id: snapshot-reach
    resource: ../rocdown/island-snapshot-roc-reachability.md
    title: Roc typechecks unused bindings
    author: process:cursor
    last_modified: 2026-08-20
  - id: roc-optional-commit
    resource: https://github.com/roc-lang/roc/commit/9788fce77866ae57d641b6ccb47bb5ef080c1bb6
    title: Optional and default record fields (2026-07-27)
    author: organization:roc-lang
  - id: roc-allsyntax
    resource: https://raw.githubusercontent.com/roc-lang/examples/main/examples/AllSyntax/main.roc
    title: Official AllSyntax ServerConfig defaults and optionals
    author: organization:roc-lang
  - id: roc-nightly
    resource: https://github.com/roc-lang/roc/commit/fb208ba17ef1af6254c90a6715f423589a4bcb75
    title: nightly-2026-08-23-fb208ba merge
    author: organization:roc-lang
---

# Roc nightly 2026-08-23 record defaults versus Rocci ??

## Probe

Checked against two local compilers on 2026-08-25, using `roc check` on
`basic-cli` 0.22 apps:

| Binary | Version |
| --- | --- |
| `/Users/nils/Projects/roc/roc_nightly-macos_apple_silicon-2026-08-23-fb208ba/roc` | `nightly-2026-08-23-fb208ba` |
| `/Users/nils/Projects/roc/roc_nightly-macos_apple_silicon-2026-08-12-606470f/roc` | `nightly-2026-08-12-606470f` (PATH at probe time) |

This record is exploratory. Implementation: [pin Roc and emit type-position
defaults](/plans/rocci/roc-nightly-defaults.md). Pattern `??` is still not
copied into generated Roc.[^ast-strip][^components-ref][^plan]

## Claim

Roc gained **defaulted and optional record fields in type position**. It did
**not** gain default *function arguments*, and it still **rejects `??` in
record patterns**. Rocci's authored `|{ name ?? "Roc" }|` therefore still
cannot be copied into generated Roc.[^ast-strip][^roc-optional-commit][^roc-allsyntax]

## What works on 2026-08-23

These programs typecheck on `fb208ba` and fail to parse or typecheck on
`606470f`.[^roc-nightly]

**Defaulted field** (always present when read; omit at construction):

```roc
hello : { name : Str ?? "Roc" } -> Str
hello = |{ name }| name

_ = hello({})
_ = hello({ name: "Ada" })
```

**Optional field** (may be missing; read with `.?`):

```roc
Cfg : { host : Str, timeout_ms ?: U64 }

describe = |cfg| match cfg.?timeout_ms {
    Ok(ms) => ms.to_str()
    Err(MissingField) => "none"
}

_ = describe({ host: "localhost" })
```

That matches the current AllSyntax `ServerConfig` sketch: `port : U16 ?? 8080`
and `timeout_ms ?: U64`.[^roc-allsyntax][^roc-optional-commit]

`roc` (not only `check`) of the defaulted-field and optional-field apps
exited 0 with empty stdout.

## What still fails (both nightlies)

| Shape | Result on `fb208ba` | Rocci implication |
| --- | --- | --- |
| `hello = \|{ name ?? "Roc" }\|` | Parse: expected `:` after field name | Keep `strip_param_defaults` and call-site fill.[^ast-strip] |
| `\|{ name: name ?? "Roc" }\|` or `\|{ name : Str ?? "Roc" }\|` | Parse: `??` is not a pattern | Same. |
| `hello = \|name ?? "Roc"\|` | Parse: `??` cannot start a pattern | No positional defaults. |
| `pair = \|a, b ?? 0\|` then `pair(1)` | Same | Generated handlers still cannot omit a trailing argument.[^optional-request] |
| `use_db = \|{ db }\|` called with `{ db, request }` | Type mismatch: extra field | Closed records. Extra fields need an explicit rest. |
| `handle : HandlerIn -> Str` with `HandlerIn : { db, request }` and `\|{ db }\|` | Missing field `request` | Alternative B (omit unused fields) is still blocked.[^optional-request] |
| `use_db : { db : Str, .. } -> Str` with body `\|{ db }\|` | Pattern is a closed `{ db }` | Openness must appear on the pattern (`\|{ db, .. }\|`), not only the annotation. |
| Unused `helper("wrong")` | Type mismatch | Unused values are still typechecked.[^snapshot-reach] |

`|{ db, .. }|` accepting `{ db: "ok", request: "http" }` typechecks on both
nightlies.

## What this is not

Roc still has no optional *positional* parameters. Research that said
“Roc has no optional trailing arguments” remains true for `|a, b|`
functions.[^optional-request]

Research that said “Roc nightly cannot express optional *record fields*”
is **stale as of this nightly**. Optional and defaulted fields exist on
types and constructions. Rocci templates still cannot rely on pattern
`??`.[^block-renderers][^template-readme]

Emitting a generated type annotation `{ name : Str ?? "Roc" }` and a
pattern `|{ name }|` would be a **new lowering design**, not a drop-in
replacement for stripping. Call sites that currently fill omitted props
could instead construct `{}` and let Roc materialize the default. That
is not implemented and is not recommended until the pinned CI Roc is
this nightly or later.

## Disposition

- Keep `strip_param_defaults` on generated patterns.
- Emit `{ name : Str ?? "Roc" }` (or an authored type) and stop filling omitted
  fields at call sites. That is the [implementation plan](/plans/rocci/roc-nightly-defaults.md).
- Datastar option records stay a tag `List` by product choice.
- Revisit handler `HandlerIn` flattening only if authors write `|{ db, .. }|`.
- Snapshot reachability still cannot wait on Roc skipping unused bindings.

[^ast-strip]: Pattern rewrite remains required; types live on the generated annotation.
[^plan]: Cutover: pin 2026-08-23-fb208ba and emit type-position defaults.
[^template-readme]: Documents Rocci `??` stripping and that Datastar options are not a JS-style optional record.
[^components-ref]: Public contract: authored `??`; generated type-position defaults.
[^optional-request]: Dispatch is always `handler!(context, request)` because Roc has no optional trailing arguments; Alternative B asked whether unused record fields can be omitted.
[^block-renderers]: Stated Roc nightly cannot express optional record fields and that `??` cannot live on a Roc type.
[^snapshot-reach]: Authoring-optimal “skip unused bindings” is unavailable; confirmed again with a type-mismatched unused call.
[^roc-optional-commit]: End-to-end defaulted (`name : Type ?? default`) and optional (`name :? Type` / `?:`) record fields.
[^roc-allsyntax]: `ServerConfig : { host : Str, port : U16 ?? 8080, timeout_ms ?: U64 }` and `.?timeout_ms`.
[^roc-nightly]: Compiler tag of the probed binary.
