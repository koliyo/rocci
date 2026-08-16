# Rocci

A `.rocci` template language, `.rocdown` content format, and desktop runtime.
Author HTML components in `.rocci` or Markdown-first pages in `.rocdown`,
compile them to ordinary [Roc](https://www.roc-lang.org/), and serve them over
HTTP with Datastar. `rocci run` opens the app in an embedded
[tao](https://github.com/tauri-apps/tao) / [wry](https://github.com/tauri-apps/wry)
window.

The workspace is `rocci-template` (`.rocci` parse/lower), `rocci-rocdown`
(Markdown documents), `rocci-lsp`, `rocci-cli`, `rocci-core` (config), and
`rocci-wry` (preview windows).

## Run an example

Install the platform prerequisites required by Wry, plus `roc` and `cargo` on
`PATH`. Then from the repository root:

```sh
cargo run -q -p rocci-cli -- run examples/counter/Counter.rocci
cargo run -q -p rocci-cli -- run examples/styling/Styling.rocci
cargo run -q -p rocci-cli -- run examples/rocdown/Guide.rocdown
cargo run -q -p rocci-cli -- run examples/errors/Dx.rocdown
cargo run -q -p rocci-cli -- run examples/snake
cargo run -q -p rocci-cli -- run examples/datastar
```

[`examples/counter`](examples/counter) is the starting app: SQLite, `@on`, and a
Datastar patch. [`examples/styling`](examples/styling) is the same template
language with file-level and component `@css`.
[`examples/rocdown`](examples/rocdown) is a Markdown page with explicit `@roc`,
`@component`, and `@render` islands; see [`crates/rocci-rocdown`](crates/rocci-rocdown)
for the format. [`examples/errors`](examples/errors) is the 404 and parse-error
preview: a working `/dx/` page plus a broken file that still opens in the window.

`rocci run path/to/App.rocci` is a standalone app: compile that file, generate
an HTTP dispatcher from `@context` / `@init` / `@on`, and start it. `rocci run`
on a directory or `main.roc` compiles sibling `.rocci` modules and starts the
authored Roc app. Both paths stage `Html.roc` / `Datastar.roc` from the CLI
runtime and a pinned Datastar JS file in `assets/` (downloaded into
`~/.rocci/cache` on first use). The embedded window listens on a free local
TCP port and prints the URL so you (or an agent) can inspect the same HTTP
server. Pass `--no-window` to serve on port 8000 without a window. Override
the port with `--port` or `ROC_BASIC_WEBSERVER_PORT`.

On Linux, Wry requires WebKitGTK development packages. macOS and Windows use
the operating system webview. Datastar evaluates declarative expressions using
JavaScript's `Function` constructor, so the script policy permits
`unsafe-eval`; script sources remain restricted to self-hosted assets.

## Package a desktop app

`rocci bundle` compiles the Roc app, builds the `rocci` host, and assembles an
ad-hoc signed macOS `.app`. The bundled app does not need `roc` on `PATH` at
runtime. From the repository root, with `roc` and `cargo` on `PATH`:

```sh
./scripts/bundle-macos.sh
open "target/release/bundle/macos/Datastar.app"
```

Or:

```sh
cargo run -p rocci-cli -- bundle --config rocci.toml
```

The root [`rocci.toml`](rocci.toml) points at [`examples/datastar`](examples/datastar),
the custom-`main.roc` gallery. That example also has its own
`examples/datastar/rocci.toml` (`bundle.app = "."`) so you can package from the
app directory the same way.

Opening the `.app` starts the host with no arguments. It finds
`Contents/Resources/rocci.toml`, launches the compiled Roc server, and opens
the preview window.

Packaging is currently macOS-only.

## CLI

```sh
cargo run -p rocci-cli -- validate
cargo run -p rocci-cli -- bundle --config rocci.toml
cargo run -p rocci-cli -- build path/to/file.rocci
cargo run -p rocci-cli -- build examples/rocdown/Guide.rocdown
cargo run -p rocci-cli -- run examples/counter/Counter.rocci
cargo run -p rocci-cli -- run examples/rocdown/Guide.rocdown
cargo run -p rocci-cli -- view examples/counter/Counter.rocci --component CounterCard --arg count=3
cargo run -p rocci-cli -- browse examples
cargo run -p rocci-cli -- inspect --ast examples/counter/Counter.rocci
cargo run -p rocci-cli -- datastar pin 1.0.2 --app examples/datastar
cargo run -p rocci-cli -- datastar update --app examples/datastar
```

`rocci.toml` describes windows, HTTP, security, assets, development, and bundle
profiles. `[assets] datastar` pins the Datastar JS version the CLI copies into
the app; `rocci datastar update` bumps that pin. The CLI does not auto-upgrade
on `run`.

## Tests

```sh
cargo test --workspace
```

See [ROADMAP.md](ROADMAP.md) for remaining work.
