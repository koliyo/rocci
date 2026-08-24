# Rocci Visual Studio Code Extension

Language support for `.rocci` template modules and `.rocdown` documents. Analysis is implemented by `rocci-language-server`; this extension is a thin LSP client.

## Features

- **Full Syntax and Embedded Language Highlighting**: Semantic highlighting out-of-the-box for Rocci/Rocdown declarations, template HTML tags/attributes, component invocations, and embedded Roc, CSS, HTML, and Markdown constructs via LSP semantic tokens.
- **Embedded Language Backends**: In-process Tree-sitter backends highlight executable `@roc`, `@css`, `{expression}` interpolations, and display-only code fences (`roc`, `html`, `css`, etc.) with zero boundary leaks.
- **Document Symbols & Outline**: Outline view and breadcrumbs for components, handlers, fixtures, styles, page metadata, and Rocdown headings.
- **Diagnostics & Error Recovery**: Push diagnostics for parser syntax errors with parser recovery that preserves partial highlighting on incomplete documents.
- **File icons**: Explorer icons for `.rocci` and `.rocdown` use the folded-R document mark.
- **Navigation & Definition**: Go-to-definition for same-file component declarations (`<UserCard />` -> `@component UserCard`).
- **Completion & Hover**: Autocomplete for directives (`@if`, `@for`, `@match`, `@let`, `@component`, `@css`, `@page`, `@roc`, `:note`), handlers (`@get:view`, `@post:fragment`), HTML elements, and components; hover documentation for template elements.
- **Preview**: **Rocci: Preview** (`rocci.preview`) saves the active `.rocci` or `.rocdown` file, runs `rocci run` or `rocdown view` with `--no-window --port auto`, and opens that loopback origin in Simple Browser **beside** the editor. **Rocci: Stop Preview** stops the process. The play / open-preview control is on the editor title bar and in the Run menu for Rocci and Rocdown files.

The preview is the product HTTP origin, not a second renderer. Datastar, live reload, and the Dev inspector overlay stay on that origin / the desktop host. Simple Browser does not show the Tao/Wry inspector chrome. Preview requires a saved file; untitled buffers cannot be served.

## Configuration

| Setting | Type | Default | Description |
| --- | --- | --- | --- |
| `rocci.lsp.serverPath` | `string` | `""` | Path to the `rocci-language-server` executable (defaults to packaged binary or `target/debug/rocci-language-server`) |
| `rocci.lsp.trace.server` | `string` | `"off"` | Traces communication between VS Code and the language server (`"off"`, `"messages"`, `"verbose"`) |
| `rocci.preview.rocciPath` | `string` | `""` | Path to the `rocci` binary (empty uses bundled `dist/bin`, `PATH`, or workspace `target/debug`) |
| `rocci.preview.rocdownPath` | `string` | `""` | Path to the `rocdown` binary (empty uses bundled `dist/bin`, `PATH`, or workspace `target/debug`) |

Semantic highlighting is enabled by default in VS Code (`editor.semanticHighlighting.enabled: true`).

## Development

From the repository root:

1. Build the language server:
   ```sh
   cargo build -p rocci-rocdown-lsp
   ```
2. Press **F5** in VS Code (or run **Run Rocci Extension** from the Run & Debug panel).

## Testing

Run automated extension-host integration tests against the live `rocci-language-server`:

```sh
cd editors/vscode
npm test
```

## Packaging

Package the extension into a standalone `.vsix` bundle containing the compiled `rocci-language-server` release binary:

```sh
uv run rocci-ops package vscode
```

Install the resulting `.vsix` into VS Code or Cursor:

```sh
uv run rocci-ops install vscode
uv run rocci-ops install cursor
```

`install vscode` runs `code --install-extension` on the newest
`editors/vscode/rocci-*.vsix`. `install cursor` uses the same `code` CLI with
`--extensions-dir` pointed at `~/.cursor/extensions`.

Preview needs `rocci` and `rocdown` on `PATH`, in the packaged `dist/bin`, in workspace `target/debug`, or via the path settings above. The language server binary is separate from those CLIs.