# Rocci Zed Extension

Language support for `.rocci` template modules and `.rocdown` documents. Analysis is implemented by `rocci-language-server`; this extension is a thin LSP client.

Zed does not bundle the language server binary into the extension WASM. Build or install `rocci-language-server` separately.

## Features

- **Semantic Highlighting**: Full semantic token support for host declarations, Roc, CSS, HTML elements, and Markdown structure.
- **Embedded Languages**: Highlighting for `@css`, `@roc`, inline Roc expressions, and display-only code fences.
- **Diagnostics**: Compiler diagnostics and recovery on syntax errors.

## File icons

This extension also ships a **Rocci** icon theme. Select it in
**icon theme selector: toggle** to show the folded-R document mark on `.rocci`
and `.rocdown` files. The theme is intentionally small: it only adds those two
suffixes plus simple folder/default glyphs. To keep another icon pack, copy the
`rocci` / `rocdown` suffix mappings from `icon_themes/rocci.json` into that pack.

## Development

From the rocci repository:

1. `cargo build -p rocci-rocdown-lsp`
2. In Zed, run **zed: install dev extension** (from Command Palette) and choose `editors/zed`.

The extension looks up the server in this order:

1. `lsp.rocci-language-server.binary.path` in Zed settings
2. `rocci-language-server` on `PATH`
3. `{worktree}/target/debug/rocci-language-server` when the worktree is this repo (after `cargo build -p rocci-rocdown-lsp`)

Override the binary if needed:

```json
{
  "lsp": {
    "rocci-language-server": {
      "binary": {
        "path": "/absolute/path/to/rocci-language-server"
      }
    }
  }
}
```

## Highlighting Configuration

Highlighting comes from LSP semantic tokens. Enable them for Rocci and Rocdown (this repository already enables this in `.zed/settings.json`):

```json
{
  "languages": {
    "Rocci": {
      "semantic_tokens": "full"
    },
    "Rocdown": {
      "semantic_tokens": "full"
    }
  }
}
```

## Testing & Packaging

Verify the Zed extension build and configuration:

```sh
uv run rocci-ops verify-zed
```

Package the release WASM extension artifact:

```sh
uv run rocci-ops package zed
```
