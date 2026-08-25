# Rocci Visual Studio Code Extension

Language support for `.rocci` template modules and `.rocdown` documents. Analysis is implemented by `rocci-language-server`; this extension is a thin LSP client.

## Features

- **Full Syntax and Embedded Language Highlighting**: Semantic highlighting out-of-the-box for Rocci/Rocdown declarations, template HTML tags/attributes, component invocations, and embedded Roc, CSS, HTML, and Markdown constructs via LSP semantic tokens.
- **Embedded Language Backends**: In-process Tree-sitter backends highlight executable `@roc`, `@css`, `{expression}` interpolations, and display-only code fences (`roc`, `html`, `css`, etc.) with zero boundary leaks.
- **Document Symbols & Outline**: Outline view and breadcrumbs for components, handlers, fixtures, styles, page metadata, and Rocdown headings.
- **Diagnostics & Error Recovery**: Push diagnostics for parser syntax errors with parser recovery that preserves partial highlighting on incomplete documents.
- **File icons**: Explorer icons for `.rocci` and `.rocdown` use the folded-R document mark.
- **Navigation & Definition**: Go-to-definition for same-file component declarations (`<UserCard />` -> `@component UserCard`). Executable Roc regions also forward compiler definition, completion, and references when `roc experimental-lsp` is available.
- **Completion & Hover**: Autocomplete for directives (`@if`, `@for`, `@match`, `@let`, `@component`, `@css`, `@page`, `@roc`, `:note`), handlers (`@get:view`, `@post:fragment`), HTML elements, and components; hover documentation for template elements. In executable Roc (including `{expr}` / `@{expr}`), hover prefers compiler types from `roc experimental-lsp` when that binary is on `PATH` or `rocci.roc.path`. Host hover remains when Roc is missing.
- **Restart**: **Rocci: Restart LSP server** (`rocci.restartLspServer`) respawns `rocci-language-server` and its optional `roc experimental-lsp` child. Use it after changing `rocci.roc.path`.
- **Preview**: **Rocci: Preview** (`rocci.preview`) saves the active `.rocci` or `.rocdown` file, runs `rocci run` or `rocdown view` with `--no-window --port auto --verbose`, and opens that loopback origin in a beside-editor webview host. The host owns the Rocci toolbar (back, forward, home, reload, live-reload, path, and the CLI serving name). **Rocci: Reload Preview** refreshes the page iframe. **Rocci: Stop Preview** stops the process. Watch, rebuild, and reload lines are written to the **Rocci Preview** output channel.
- **Dev inspector**: When the CLI prints `inspector_ready <url>` (piped `--no-window` stdout), **Dev** iframes `/__rocci/dev` or the sibling inspector and docks it right or bottom. Inspector UX defects (scroll, overlay overlap, OKF snapshots, `tok-*` highlighting) stay on the [preview inspector repair](../../knowledge/plans/rocci/preview-inspector-repair.md) plan, not this extension.
- **Tools**: **Rocci: Update tools** (`rocci.updateTools`) checks GitHub releases. Supported archives are `rocci-{version}-aarch64-apple-darwin.tar.gz` and `rocci-{version}-x86_64-unknown-linux-gnu.tar.gz`.

The preview is the product HTTP origin, not a second renderer. Saving a Rocdown file in the same site reloads the webview after the CLI rebuilds. Saving a Rocci file restarts `rocci run` (that command does not watch). Preview requires a saved file; untitled buffers cannot be served.

## Configuration

| Setting | Type | Default | Description |
| --- | --- | --- | --- |
| `rocci.lsp.serverPath` | `string` | `""` | Path to `rocci-language-server`. Empty uses F5 `target/debug`, a verified GitHub extract, or `PATH`. |
| `rocci.roc.path` | `string` | `""` | Path to the `roc` compiler for executable Roc LSP features. Empty uses `ROCCI_ROC_PATH` or `roc` on `PATH`. Restart the language server after changing this. |
| `rocci.lsp.trace.server` | `string` | `"off"` | Traces communication between VS Code and the language server (`"off"`, `"messages"`, `"verbose"`) |
| `rocci.preview.rocciPath` | `string` | `""` | Path to `rocci`. Empty uses F5 `target/debug`, a verified GitHub extract, or `PATH`. |
| `rocci.preview.rocdownPath` | `string` | `""` | Path to `rocdown`. Empty uses F5 `target/debug`, a verified GitHub extract, or `PATH`. |
| `rocci.tools.channel` | `string` | `"stable"` | `stable` uses `/releases/latest`. `dev` installs from the rolling GitHub tag/release `dev` (`rocci-dev-<sha>-<triple>.tar.gz`). |
| `rocci.tools.autoUpdate` | `boolean` | `true` | Check GitHub releases on activate when not debugging. |

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

Package the extension into a standalone `.vsix`. The VSIX does not contain Rocci binaries; first non-debug launch (or **Rocci: Update tools**) downloads `rocci`, `rocdown`, and `rocci-language-server` from GitHub releases after sha256 verify.

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

Path settings and F5 `target/debug` builds override the download. Preview and the language server resolve in that order, then a verified extract under global storage, then `PATH`.