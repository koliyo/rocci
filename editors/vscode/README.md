# Rocci Visual Studio Code Extension

Language support for `.rocci` template modules and `.rocdown` documents. Analysis is implemented by `rocci-language-server`; this extension is a thin LSP client.

## Features

- **Full Syntax and Embedded Language Highlighting**: Semantic highlighting out-of-the-box for Rocci/Rocdown declarations, template HTML tags/attributes, component invocations, and embedded Roc, CSS, HTML, and Markdown constructs via LSP semantic tokens.
- **Embedded Language Backends**: In-process Tree-sitter backends highlight executable `@roc`, `@css`, `{expression}` interpolations, and display-only code fences (`roc`, `html`, `css`, etc.) with zero boundary leaks.
- **Document Symbols & Outline**: Outline view and breadcrumbs for components, handlers, fixtures, styles, page metadata, and Rocdown headings.
- **Diagnostics & Error Recovery**: Push diagnostics for parser syntax errors with parser recovery that preserves partial highlighting on incomplete documents.
- **Navigation & Definition**: Go-to-definition for same-file component declarations (`<UserCard />` -> `@component UserCard`).
- **Completion & Hover**: Autocomplete for directives (`@if`, `@for`, `@match`, `@let`, `@component`, `@css`, `@on`, `@page`, `@roc`, `@docs`), HTML elements, and components; hover documentation for template elements.

## Configuration

| Setting | Type | Default | Description |
| --- | --- | --- | --- |
| `rocci.lsp.serverPath` | `string` | `""` | Path to the `rocci-language-server` executable (defaults to packaged binary or `target/debug/rocci-language-server`) |
| `rocci.trace.server` | `string` | `"off"` | Traces communication between VS Code and the language server (`"off"`, `"messages"`, `"verbose"`) |

Semantic highlighting is enabled by default in VS Code (`editor.semanticHighlighting.enabled: true`).

## Development

From the repository root:

1. Build the language server:
   ```sh
   cargo build -p rocci-lsp
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
./scripts/package-vscode.sh
```

Install the resulting `.vsix`:

```sh
code --install-extension editors/vscode/rocci-*.vsix
```
