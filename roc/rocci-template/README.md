# Roc-native template compiler (POC)

Exploratory second implementation of `.rocci` template parse and Html
lowering. **Rust `crates/rocci-template` stays the product compiler.**
`rocci` does not invoke this package.

Pinned Roc: `nightly-2026-08-23-fb208ba`.

## Package tests

```sh
roc test roc/rocci-template/main.roc
```

## Compiler driver

From the repository root:

```sh
roc roc/rocci-template/app.roc -- roc/rocci-template/fixtures/hello.rocci
```

Stdout is ordinary Roc and matches

```sh
cargo run -q -p rocci-template -- build roc/rocci-template/fixtures/hello.rocci
```

Read stdin with `-`. Write a file with `-o path` (or `-o -` for stdout).

This `app.roc` is a proof-of-concept driver only. Do not install it as
`rocci`. Do not change public docs for the product CLI.

## `roc check` of generated Roc

Stub `Html` hosts live under `check/`:

```sh
roc check roc/rocci-template/check/hello.roc
roc check roc/rocci-template/check/branch.roc
roc check roc/rocci-template/check/css.roc
```
