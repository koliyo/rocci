# `rocci-rocdown-lsp`

Product composition crate for the shipped `rocci-language-server` binary. It
registers `RocciAnalyzer` from `rocci-lsp` and `RocdownAnalyzer` from
`rocci-rocdown` without adding Rocdown types to the generic server library.

```sh
cargo build -p rocci-rocdown-lsp
cargo test -p rocci-rocdown-lsp
```

Editors and release archives still invoke the `rocci-language-server` binary
name. Build this package, not `rocci-lsp`, to produce that binary.

On startup the binary tries to spawn `roc experimental-lsp --stdio` (from
`ROCCI_ROC_PATH` or `roc` on `PATH`). If that fails, host hover, tokens, and
diagnostics still work. Display-only fences are never sent to Roc.
