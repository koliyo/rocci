# Rocci Visual Studio Code Extension

Language support for `.rocci` template modules and `.rocdown` documents. Analysis is implemented by `rocci-language-server`; this extension is a thin LSP client.

## Features

- **Full Syntax and Embedded Language Highlighting**: Highlighting for Rocci/Rocdown declarations, template HTML tags/attributes, component invocations, and embedded Roc, CSS, and Markdown constructs via LSP semantic tokens.
- **Embedded Region Support**: Accurate highlighting across executable `@roc`, `@css`, `{expression}`, and Markdown display-only code fences (`roc`, `html`, `css`, etc.).
- **Diagnostics & Error Recovery**: Direct compiler syntax diagnostics and graceful partial highlighting on incomplete code.
- **Same-File Navigation**: Go-to-definition for component declarations (`<UserCard>` -> `@component UserCard`).

## Development

From the rocci repository:

1. `cargo build -p rocci-lsp`
2. Run **Run Rocci Extension** from the repo root, or open `editors/vscode` and run **Run Extension** (F5).

Override the server path with `rocci.lsp.serverPath` in settings if needed.

## Testing

Run the automated Extension Host integration tests against the live language server:

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
