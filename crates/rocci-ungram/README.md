# rocci-ungram

Generate owned Rust AST data types from Rocci and Rocdown ungrammar tree specs.

This crate is a developer CLI. It does not generate scanners, parsers, or a CST.
Language crates do not depend on it. `generate` writes `crates/rocci-template/src/ast.generated.rs`
and a Rocdown snapshot (Rocdown cutover is a later phase). `--check` fails when those files are stale.

```sh
cargo run -q -p rocci-ungram -- generate
cargo run -q -p rocci-ungram -- check
```

Tree specs:

- `crates/rocci-template/Rocci.AST.ungram`
- `crates/rocci-rocdown/Rocdown.AST.ungram`

Foreign, opaque, leaf, and extra-field mappings live in the sibling `*.AST.toml`
sidecars. The `ungrammar` crate parses the DSL; this crate owns dialect restrictions
and owned-struct lowering.
