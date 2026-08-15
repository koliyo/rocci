# Rocci

A `.rocci` template language and desktop runtime. Author HTML components in
`.rocci`, compile them to ordinary [Roc](https://www.roc-lang.org/), and serve
them over HTTP with Datastar. `rocci run` opens the app in an embedded
[tao](https://github.com/tauri-apps/tao) / [wry](https://github.com/tauri-apps/wry)
window.

The workspace is `rocci-template` (parse/lower), `rocci-lsp`, `rocci-cli`,
`rocci-core` (config), and `rocci-wry` (preview windows).

## Run an example

Install the platform prerequisites required by Wry, plus `roc` and `cargo` on
`PATH`. Then from the repository root:

```sh
./scripts/run-counter.sh
./scripts/run-snake.sh
./scripts/run-datastar.sh
```

Each script copies shared assets and runs `rocci run`, which compiles sibling
`.rocci` modules and starts `main.roc`. Pass `--no-window` to serve only.
Override the port with `--port` or `ROC_BASIC_WEBSERVER_PORT`.

```sh
cargo run -q -p rocci-cli -- run examples/counter/main.roc
```

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
open "target/release/bundle/macos/Counter.app"
```

Or:

```sh
cargo run -p rocci-cli -- bundle --config rocci.toml
```

The root [`rocci.toml`](rocci.toml) points at [`examples/counter`](examples/counter),
the documented bundle walkthrough. That example also has its own
`examples/counter/rocci.toml` (`bundle.app = "."`) so you can package from the
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
cargo run -p rocci-cli -- run examples/counter
cargo run -p rocci-cli -- view examples/counter/Counter.rocci --component counterCard --arg count=3
cargo run -p rocci-cli -- browse examples
cargo run -p rocci-cli -- inspect --ast examples/counter/Counter.rocci
```

`rocci.toml` describes windows, HTTP, security, assets, development, and bundle
profiles.

## Tests

```sh
cargo test --workspace
```

See [ROADMAP.md](ROADMAP.md) for remaining work.
