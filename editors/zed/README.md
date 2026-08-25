# Rocci Zed Extension

Language support for `.rocci` template modules and `.rocdown` documents. Analysis is implemented by `rocci-language-server`; this extension is a thin LSP client.

Zed does not bundle the language server binary into the extension WASM. When settings, `PATH`, and a worktree debug build miss, the extension downloads `rocci-language-server` from [koliyo/rocci](https://github.com/koliyo/rocci) GitHub releases (`aarch64-apple-darwin` and `x86_64-unknown-linux-gnu`). Preview still uses the native window and still needs `rocci` / `rocdown` on `PATH` or in settings.

## Features

- **Semantic Highlighting**: Full semantic token support for host declarations, Roc, CSS, HTML elements, and Markdown structure.
- **Embedded Languages**: Highlighting for `@css`, `@roc`, inline Roc expressions, and display-only code fences.
- **Diagnostics**: Compiler diagnostics and recovery on syntax errors. Executable Roc also receives remapped `roc experimental-lsp` diagnostics when `roc` is available (`ROCCI_ROC_PATH` or `settings.rocPath`).

## Preview

Zed has no extension-owned embedded browser beside the buffer. Until the host provides a webview or HTML pane that can load a loopback origin, preview uses **tasks** that start the product CLI and open the native Tao/Wry preview window:

| Task | Command |
| --- | --- |
| **Preview Rocci file** | `rocci run $ZED_FILE` |
| **Preview Rocdown file** | `rocdown view $ZED_FILE` |

This extension ships those templates in `tasks.json`. Run them from the task palette (**task: spawn**). Optional **Serve … (no window)** tasks start `--no-window --port auto` so you can open `http://127.0.0.1:<port>/` in a system browser; they do not create an in-editor pane. There is no Zed webview toolbar or Dev dock; those stay in the native preview window.

If a checkout does not load extension tasks, copy `editors/zed/tasks.json` into the project `.zed/tasks.json`.

The Zed manifest remains an LSP adapter. Do not expect Simple Browser–style beside-buffer preview here.

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
4. A GitHub-release extract for the current Zed platform (`/releases/latest` by default)

Install the rolling `dev` tag/release instead of latest:

```json
{
  "lsp": {
    "rocci-language-server": {
      "settings": {
        "channel": "dev"
      }
    }
  }
}
```

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

Point the optional `roc experimental-lsp` child at a specific compiler (`ROCCI_ROC_PATH`), either through binary env or `settings.rocPath`. Host hover still works if `roc` is missing. Restart the language server after changing this. Set `settings.verbose` or `ROCCI_LSP_VERBOSE=1` for child-spawn and mapped-hover logs.

```json
{
  "lsp": {
    "rocci-language-server": {
      "settings": {
        "rocPath": "/absolute/path/to/roc"
      },
      "binary": {
        "env": {
          "ROCCI_ROC_PATH": "/absolute/path/to/roc"
        }
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
uv run rocci-ops check zed
```

Package the release WASM extension artifact:

```sh
uv run rocci-ops package zed
```
