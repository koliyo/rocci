# Rocci

A `.rocci` template language, `.rocdown` content format, and desktop runtime.
Author HTML components in `.rocci` or Markdown-first pages in `.rocdown`,
compile them to ordinary [Roc](https://www.roc-lang.org/), and serve them over
HTTP with Datastar. `rocci run` opens the app in a
[tao](https://github.com/tauri-apps/tao) / [wry](https://github.com/tauri-apps/wry)
preview window.

The workspace is organized into focused packages with strictly enforced one-way boundaries:
- **Base Rocci:** `rocci-template` (`.rocci` parse/lower), `rocci-core` (configuration and runtime contracts), `rocci-desktop` (windowing and webview runtime), `rocci-cli` (`rocci` binary), `rocci-ui` (domain-neutral view records and presentation components).
- **Rocdown:** `rocci-rocdown` (format parser, static catalog, article rendering, site generator), `rocci-rocdown-cli` (`rocdown` binary), `rocci-theme` (document CSS theme resolver).
- **Open Knowledge Format:** `okf` (portable, UI-neutral knowledge engine), `rocci-okf` (`rocci-okf` application binary and review server).
- **Tooling:** `rocci-lsp` (generic language-server core and Rocci analyzer), `rocci-rocdown-lsp` (shipped `rocci-language-server` for `.rocci` and `.rocdown`), `rocci-highlight` (pinned Tree-sitter highlighter library).

## Run an example

Install the platform prerequisites required by Wry, plus `roc` and `cargo` on
`PATH`. Then from the repository root:

```sh
cargo run -q -p rocci-cli -- run examples/rocci/standalone/counter/Counter.rocci
cargo run -q -p rocci-cli -- run examples/rocci/standalone/styling/Styling.rocci
cargo run -q -p rocci-cli -- run examples/rocci/custom/snake
cargo run -q -p rocci-cli -- run examples/rocci/custom/datastar
cargo run -q -p rocci-rocdown-cli -- run examples/rocdown/pages/Guide.rocdown
cargo run -q -p rocci-rocdown-cli -- run examples/rocdown/errors/ErrorDemo.rocdown
```

[`examples/rocci/standalone/counter`](examples/rocci/standalone/counter) is the starting app: SQLite, `@on`, and a
Datastar patch. [`examples/rocci/standalone/styling`](examples/rocci/standalone/styling) is the same template
language with file-level and component `@css`.
[`examples/rocdown/pages`](examples/rocdown/pages) is a Markdown page with explicit `@roc`,
`@component`, and `@render` islands; see [`crates/rocci-rocdown`](crates/rocci-rocdown)
for the format. [`examples/rocdown/errors`](examples/rocdown/errors) is the 404 and parse-error
preview: a working `/error-demo/` page plus a broken file that still opens in the window.

`rocci run path/to/App.rocci` is a standalone app: compile that file, generate
an HTTP dispatcher from `@context` / `@init` / `@on`, and start it. `rocci run`
on a directory or `main.roc` compiles sibling `.rocci` modules and starts the
authored Roc app. Both paths stage `Html.roc` / `Datastar.roc` from the CLI
runtime and a pinned Datastar JS file in `assets/` (downloaded into
`~/.rocci/cache` on first use). The preview window listens on a free local
TCP port and prints the URL so you (or an agent) can inspect the same HTTP
server. Pass `--no-window` to serve on port 8000 without a preview window. Override
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
uv run rocci-ops bundle macos
open "target/release/bundle/macos/Datastar.app"
```

Or:

```sh
cargo run -p rocci-cli -- bundle --config rocci.toml
```

The root [`rocci.toml`](rocci.toml) points at [`examples/rocci/custom/datastar`](examples/rocci/custom/datastar),
the custom-`main.roc` gallery. That example also has its own
`examples/rocci/custom/datastar/rocci.toml` (`bundle.app = "."`) so you can package from the
app directory the same way.

Opening the `.app` starts the host with no arguments. It finds
`Contents/Resources/rocci.toml`, launches the compiled Roc server, and opens
the preview window.

Packaging is currently macOS-only.

## CLI

### Rocci

```sh
cargo run -p rocci-cli -- validate
cargo run -p rocci-cli -- bundle --config rocci.toml
cargo run -p rocci-cli -- build path/to/file.rocci
cargo run -p rocci-cli -- run examples/rocci/standalone/counter/Counter.rocci
cargo run -p rocci-cli -- view examples/rocci/standalone/counter/Counter.rocci --component CounterCard --arg count=3
cargo run -p rocci-cli -- browse examples
cargo run -p rocci-cli -- inspect --ast examples/rocci/standalone/counter/Counter.rocci
cargo run -p rocci-cli -- datastar pin 1.0.2 --app examples/rocci/custom/datastar
cargo run -p rocci-cli -- datastar update --app examples/rocci/custom/datastar
```

### Rocdown

```sh
cargo run -p rocci-docs -- --catalog examples/rocci/apps.toml --output dist/example-docs
cargo run -p rocci-rocdown-cli -- run examples/rocdown/pages/Guide.rocdown
cargo run -p rocci-rocdown-cli -- build examples/rocdown/site --output dist
cargo run -p rocci-rocdown-cli -- check site
cargo run -p rocci-rocdown-cli -- check docs
cargo run -p rocci-rocdown-cli -- test docs
cargo run -p rocci-rocdown-cli -- inspect ast test/AllSyntax.rocdown
```

Rocdown discovers `.rocdown` files, resolves routes in Rust, renders article HTML
from the Markdown AST, and wraps each page in [`RocdownTheme.rocci`](crates/rocci-rocdown/templates/RocdownTheme.rocci).
Content edits do not recompile Markdown as Roc.

### Rocci OKF

```sh
cargo run -p rocci-okf -- check knowledge --profile rocci
cargo run -p rocci-okf -- inspect concept architecture/system-overview knowledge
cargo run -p rocci-okf -- inspect graph knowledge
cargo run -p rocci-okf -- search "rendering" knowledge
cargo run -p rocci-okf -- benchmark knowledge/retrieval-benchmark.toml knowledge
cargo run -p rocci-okf -- run knowledge
cargo run -p rocci-okf -- run knowledge/plans/cli-entry-points.md
cargo run -p rocci-okf -- build knowledge --output dist/knowledge
```

The separate OKF knowledge path validates, inspects, searches, benchmarks, and renders
`knowledge/`. Its fixed lexical retrieval questions are measured by
`rocci-okf benchmark`; the command reports hit rate and mean reciprocal
rank and fails when the checked-in threshold is missed.

The public `rocci.dev` tree is [`site`](site), configured by
[`site/rocdown.toml`](site/rocdown.toml) and written to `dist/rocci.dev`.
[`docs`](docs) remains the mounted documentation catalog and a standalone
`check docs` / `test docs` target. With `roc` and `cargo` on `PATH`, package
the hybrid site (CDN archive plus musl `islands` binary) with:

```sh
cargo run -q -p rocci-docs -- --catalog examples/rocci/apps.toml --output dist/example-docs
cargo run -p rocci-rocdown-cli -- package site --target x64musl
```

`rocci-docs` must run first so `site/rocdown.toml` can mount `dist/example-docs`
at `/examples/`. `rocci-rocdown` does not import `rocci-docs`.

That writes `dist/rocci.dev`, `dist/site.tgz`, `dist/islands`, and
`publish.json`. GitHub Actions workflow `site.yml` packages on linux/amd64
and, on `staging` or `production` only, scps those artifacts to the origin
using the matching GitHub Environment. Land work on `main`; promote to
`staging` to publish behind Access, then to `production` for the public
hostname. Pull requests never deploy.

`rocci.toml` describes windows, HTTP, security, assets, development, and bundle
profiles. `[http] redirect_trailing_slash` (default `true`) sends GET `/page` to
`/page/` or the reverse with **308**, matching the registered `@page` route;
set it `false` to 404 with a hint instead. Custom `main.roc` apps own their
routing. `[assets] datastar` pins the Datastar JS version the CLI copies into
the app; `rocci datastar update` bumps that pin. The CLI does not auto-upgrade
on `run`.

## Tests

```sh
cargo test --workspace
uv run rocci-ops ci
```

`cargo test --workspace` is the fast crate suite. `uv run rocci-ops ci` runs the GitHub Actions validation jobs on this OS (lint, tests, AST fixtures, editors, and knowledge checks). It does not run the ubuntu/macos matrix or release cross-platform builds. Pass job names to run a subset, for example `uv run rocci-ops ci lint test`. GitHub Actions CI and Knowledge do not run on every push or PR; comment `/ci` on a pull request or use **Run workflow**.

See [ROADMAP.md](ROADMAP.md) for remaining work.

## License

Copyright 2026 Nils Hjelte.

Rocci is licensed under the [Apache License, Version 2.0](LICENSE).
Third-party components retain their own licenses; see
[THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md).
