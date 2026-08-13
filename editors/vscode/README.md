# Rocci Visual Studio Code Extension

Language support for `.rocci` template modules. Analysis is implemented by `rocci-language-server`; this extension is a thin LSP client.

## Development

From the rocci repository:

1. `cargo build -p rocci-lsp`
2. Run **Run Rocci Extension** from the repo root, or open `editors/vscode` and run **Run Extension**

Override the server with `rocci.lsp.serverPath` if needed.

## Packaging

```sh
./scripts/package-vscode.sh
```

Install the resulting `.vsix`:

```sh
code --install-extension editors/vscode/rocci-*.vsix
```
