# Rocci Zed Extension

Language support for `.rocci` template modules and `.rocdown` documents. Analysis is implemented by `rocci-language-server`; this extension is a thin LSP client.

Zed does not bundle the language server. Build or install `rocci-language-server` separately.

## Development

From the rocci repository:

1. `cargo build -p rocci-lsp`
2. In Zed, run **zed: install dev extension** and choose `editors/zed`

The extension looks up the server in this order:

1. `lsp.rocci-language-server.binary.path` in Zed settings
2. `rocci-language-server` on `PATH`
3. `{worktree}/target/debug/rocci-language-server` when the worktree is this repo (after `cargo build -p rocci-lsp`)

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

## Highlighting

Highlighting comes from LSP semantic tokens. Enable them for Rocci and Rocdown (this repository already does via `.zed/settings.json`):

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
