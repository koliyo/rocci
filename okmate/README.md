# Okmate

Okmate (open knowledge mate) is a standalone knowledge application for Open
Knowledge Format (OKF) bundles. The binary is `okmate`.

## Stack

- **Engine:** the portable [`okf`](../crates/okf) crate (the only in-repo Rust
  dependency)
- **HTML:** Askama 0.16 (planned)
- **HTTP:** Axum (planned)
- **Morph / SSE:** official Datastar Rust SDK (planned)
- **Desktop:** tao / wry / rfd, in this crate (planned)

This directory is shaped so it can become its own git repository: keep the crate
layout, change `okf` from a path dependency to a git or crates.io dependency,
and drop the workspace member line in the Rocci `Cargo.toml`.

## Depends on `okf` only

Okmate must not depend on any `rocci-*` crate. It does not interpret `.rocci`
templates. `cargo test -p okmate` does not require Roc.

`okmate check` is the knowledge application CLI. `rocci-okf check` remains the
Rocci tool until an explicit cutover.

## Usage

```sh
okmate check knowledge --profile rocci --format json
okmate inspect catalog knowledge
okmate inspect concept architecture/system-overview knowledge
okmate inspect graph knowledge
okmate search "system overview" knowledge --profile rocci
okmate benchmark knowledge/retrieval-benchmark.toml knowledge
```

JSON shapes match `rocci-okf` for these engine commands so agents can switch the binary name without a new schema. There is no `--host` flag.
