# rocci-ungram

Generate owned Rust AST data types from Rocci and Rocdown ungrammar tree specs.

This crate is a developer CLI. It does not generate scanners, parsers, or a CST.
Language crates do not depend on it. `generate` writes committed `ast.generated.rs`, `pprint.generated.rs`, and `node_kind.generated.rs` in
`rocci-template` and `rocci-rocdown`, plus `md.generated.rs` from `Rocdown.Markdown.ungram`. `--check` fails when those files are stale
or a generated production has no inspect mapping.
The CI lint job runs `--check`.

```sh
cargo run -q -p rocci-ungram -- generate
cargo run -q -p rocci-ungram -- check
```

Tree specs:

- `crates/rocci-template/Rocci.AST.ungram`
- `crates/rocci-rocdown/Rocdown.AST.ungram`
- `crates/rocci-rocdown/Rocdown.Markdown.ungram`

Foreign, opaque, leaf, extra-field, and inspect-tag mappings live in the sibling `*.toml`
sidecars (`Rocci.AST.toml`, `Rocdown.AST.toml`, `Rocdown.Markdown.toml`). `[inspect.tags]`, `[inspect.omit]`, and `[inspect.fallback]` freeze `format_ast`
heads; `--check` fails when a generated production has no inspect mapping, or when the
public tree appendix pages under `docs/reference/` are stale. The `ungrammar`
crate parses the DSL; this crate owns dialect restrictions and owned-struct lowering.
