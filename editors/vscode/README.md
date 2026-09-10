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
- **Tools**: **Rocci: Update tools** (`rocci.updateTools`) checks GitHub releases and, on `dev`, a local Cargo build. Supported archives are `rocci-{version}-aarch64-apple-darwin.tar.gz` and `rocci-{version}-x86_64-unknown-linux-gnu.tar.gz`.

The preview is the product HTTP origin, not a second renderer. Saving a Rocdown file in the same site reloads the webview after the CLI rebuilds. Saving a Rocci file restarts `rocci run` (that command does not watch). Preview requires a saved file; untitled buffers cannot be served.

## Tool resolution

The VSIX does not ship binaries. `rocci-language-server`, `rocci`, and `rocdown` are chosen in this order:

1. An explicit path (`rocci.lsp.serverPath`, `rocci.preview.rocciPath`, `rocci.preview.rocdownPath`).
2. **F5** (`VSCODE_DEBUG_MODE`): a local Cargo binary if one exists, so extension-host debugging is not pulled onto GitHub mid-session.
3. **`rocci.tools.channel`**:
   - `stable` (default): GitHub `/releases/latest`. A locally installed rolling `dev` extract is kept only when it is newer than that versioned release.
   - `dev`: newest of three candidates by timestamp:
     1. Local Cargo binary mtime (`target/debug/<exe>` or `target/release/<exe>`; later file wins).
     2. Rolling GitHub tag/release `dev` (`publishedAt`).
     3. Versioned GitHub `/releases/latest` (`publishedAt`). A newer versioned release always beats a stale GitHub `dev`.
4. A previously extracted archive under VS Code global storage, then `PATH`.

Local Cargo roots are the repo implied by the extension checkout (`extensionPath/../../target/…`) and each workspace folder’s `target/…`. Auto-update still fetches both GitHub remotes on `dev`. If the local Cargo mtime is newer than the selected remote, the download is skipped (`Skip install: local Cargo build newer than …`). The **Rocci** output channel logs `Extension: koliyo.rocci <version> git <hash> (installed|F5)` and `Language server: local Cargo …` or `Language server: GitHub v…`.

### Local dev LSP (no GitHub tag)

Use this to test a language-server change without pushing `dev` or cutting a `v*` release.

1. In the rocci workspace (this repo), set `"rocci.tools.channel": "dev"` (see `.vscode/settings.json`).
2. Build from the repository root:

   ```sh
   uv run rocci-ops build lsp
   ```

   That is `cargo build -p rocci-rocdown-lsp` and writes `target/debug/rocci-language-server`. For a release-profile binary:

   ```sh
   uv run rocci-ops build lsp --release
   ```

   (`target/release/rocci-language-server`). The extension looks in both `target/debug` and `target/release` and uses the file with the newer mtime.

   Preview CLIs, if you need those from the same tree:

   ```sh
   cargo build -p rocci-cli -p rocci-rocdown-cli
   ```

3. **Rocci: Restart LSP server** (or reload the window). The installed extension then uses the local binary when its mtime is newer than GitHub.

F5 still works: `uv run rocci-ops build lsp`, then **Run Rocci Extension**. That session always prefers the local binary when present.

## Configuration

| Setting | Type | Default | Description |
| --- | --- | --- | --- |
| `rocci.lsp.serverPath` | `string` | `""` | Path to `rocci-language-server`. Empty follows [Tool resolution](#tool-resolution). |
| `rocci.roc.path` | `string` | `""` | Path to the `roc` compiler for executable Roc LSP features. Empty uses `ROCCI_ROC_PATH`, then vscode-roc `roc.path`, then `roc` on `PATH`. |
| `rocci.lsp.verbose` | `boolean` | `false` | Write child-spawn, projection, and mapped-hover logs to the **Rocci** output channel. Also enabled when `rocci.lsp.trace.server` is `verbose`. |
| `rocci.lsp.trace.server` | `string` | `"off"` | Traces communication between VS Code and the language server (`"off"`, `"messages"`, `"verbose"`) |
| `rocci.preview.rocciPath` | `string` | `""` | Path to `rocci`. Empty follows [Tool resolution](#tool-resolution). |
| `rocci.preview.rocdownPath` | `string` | `""` | Path to `rocdown`. Empty follows [Tool resolution](#tool-resolution). |
| `rocci.tools.channel` | `string` | `"stable"` | See [Tool resolution](#tool-resolution). `stable` is GitHub `/releases/latest`. `dev` is local Cargo vs GitHub `dev` vs versioned, newest date wins. |
| `rocci.tools.autoUpdate` | `boolean` | `true` | Check GitHub releases on activate when not debugging. |

Semantic highlighting is enabled by default in VS Code (`editor.semanticHighlighting.enabled: true`).

## Development

See [Local dev LSP](#local-dev-lsp-no-github-tag) for the installed-extension `dev` channel. For a debug Extension Host:

1. `uv run rocci-ops build lsp` from the repository root.
2. Press **F5** (or **Run Rocci Extension**).

## Testing

Run automated extension-host integration tests against the live `rocci-language-server`:

```sh
cd editors/vscode
npm test
```

## Packaging

Package the extension into a standalone `.vsix`. The VSIX does not contain Rocci binaries; first non-debug launch (or **Rocci: Update tools**) follows [Tool resolution](#tool-resolution) and downloads from GitHub when that candidate wins.

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

Path settings, F5, and `rocci.tools.channel` override the download as described in [Tool resolution](#tool-resolution).