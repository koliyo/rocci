# rocci-ungram

Generate owned Rust AST data types from Rocci and Rocdown ungrammar tree specs.

This crate is a developer CLI. It does not generate scanners, parsers, or a CST.
Language crates do not depend on it. `generate` writes committed `ast.generated.rs` files in
`rocci-template` and `rocci-rocdown`. `--check` fails when those files are stale.
The CI lint job runs `--check`.

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
