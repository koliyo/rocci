# Rocci

A `.rocci` template language, `.rocdown` content format, and desktop runtime.
Author HTML components in `.rocci` or Markdown-first pages in `.rocdown`,
compile them to ordinary [Roc](https://www.roc-lang.org/), and serve them over
HTTP with Datastar. `rocci run` opens the app in a
[tao](https://github.com/tauri-apps/tao) / [wry](https://github.com/tauri-apps/wry)
preview window.

Rocci is an independent open-source project. It is built on Roc and is not an
official Roc language project.

The workspace is organized into focused packages with strictly enforced one-way boundaries:
- **Base Rocci:** `rocci-template` (`.rocci` parse/lower), `rocci-core` (configuration and runtime contracts), `rocci-desktop` (windowing and webview runtime), `rocci-cli` (`rocci` binary), `rocci-ui` (domain-neutral view records and presentation components).
- **Rocdown:** `rocci-rocdown` (format parser, static catalog, article rendering, site generator), `rocci-rocdown-cli` (`rocdown` binary), `rocci-theme` (document CSS theme resolver).
- **Open Knowledge Format:** inert `knowledge/` bundle in this repo; parse, check, inspect, search, build, and preview with [okmate](https://github.com/koliyo/okmate).
- **Tooling:** `rocci-lsp` (generic language-server core and Rocci analyzer), `rocci-rocdown-lsp` (shipped `rocci-language-server` for `.rocci` and `.rocdown`), `rocci-highlight` (pinned Tree-sitter highlighter library).

## Run an example

Install the platform prerequisites required by Wry, plus `roc` and `cargo` on
`PATH`. Then from the repository root:

```sh
cargo run -q -p rocci-cli -- run examples/rocci/standalone/counter
cargo run -q -p rocci-cli -- run examples/rocci/standalone/styling
cargo run -q -p rocci-cli -- run examples/rocci/custom/snake
cargo run -q -p rocci-cli -- run examples/rocci/custom/datastar
cargo run -q -p rocci-rocdown-cli -- run examples/rocdown/pages/Guide.rocdown
cargo run -q -p rocci-rocdown-cli -- run examples/rocdown/errors/ErrorDemo.rocdown
```

[`examples/rocci/standalone/counter`](examples/rocci/standalone/counter) is the starting app: SQLite and a
Datastar fragment. [`examples/rocci/standalone/styling`](examples/rocci/standalone/styling) is the same template
language with file-level and component `@css`.
[`examples/rocdown/pages`](examples/rocdown/pages) is a Markdown page with explicit `@roc`,
`@component`, and `@render` islands; see [`crates/rocci-rocdown`](crates/rocci-rocdown)
for the format. [`examples/rocdown/errors`](examples/rocdown/errors) is the 404 and parse-error
preview: a working `/error-demo/` page plus a broken file that still opens in the window.

`rocci run path/to/app` is a standalone app directory: resolve a unique
entry, generate an HTTP dispatcher from `@context` / `@init` /
`@method:role` routes, and start it. `rocci run path/to/App.rocci` names
the entry file and still loads sibling modules. At most one process
`@init` is allowed. `rocci run` on a directory that contains `main.roc`
compiles sibling `.rocci` modules and starts the authored Roc app. Both
paths stage `Html.roc` / `Datastar.roc` from the CLI
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
uv run rocci-ops package macos
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

To install the release `rocci` and `rocdown` binaries into
`~/.local/bin`, run `uv run rocci-ops install cli`.

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

### Knowledge (OKF)

The `knowledge/` tree stays in this repository. Check, inspect, search, build,
and preview it with [okmate](https://github.com/koliyo/okmate):

```sh
okmate check knowledge --profile base
okmate inspect concept architecture/system-overview knowledge
okmate inspect graph knowledge
okmate search "rendering" knowledge
okmate benchmark knowledge/retrieval-benchmark.toml knowledge
okmate view knowledge
okmate view knowledge/plans/shared/cli-entry-points.md
okmate build knowledge --output dist/knowledge
```

From a sibling checkout, `cargo run -q --no-default-features --manifest-path
../okmate/Cargo.toml -p okmate --` is the same CLI. Knowledge CI checks out
`koliyo/okmate` and runs those bundle commands; engine tests run in the okmate
repo.

Retrieval questions are measured by `okmate benchmark`; the command reports hit
rate and mean reciprocal rank and fails when the checked-in threshold is missed.

The public `rocci.dev` tree is [`site`](site), configured by
[`site/rocdown.toml`](site/rocdown.toml) and written to `dist/rocci.dev`.
[`docs`](docs) remains the mounted documentation catalog and a standalone
`check docs` / `test docs` target. With `roc` and `cargo` on `PATH`, package
the complete local site with:

```sh
uv run rocci-ops site
```

That repository-level command stages generated example documentation, checks
links and catalog policy, runs documented examples, and builds
`dist/rocci.dev`. The focused `rocci-rocdown-cli` commands remain available.
To package the hybrid site (CDN archive plus musl `islands` binary), use:

```sh
uv run rocci-ops package site --target x64musl
```

This stages example docs, builds live example servers, and packages the hybrid
site. Site packaging currently uses Roc's `dev` backend for every live server
because the pinned nightly can recurse in its optimized backend. These
artifacts are functional but are not production-performance builds: they may
be larger and slower. Use `rocci build --release --opt speed` when an optimized
binary is required. `rocci-docs` and `rocci-rocdown` remain separate crates;
`rocci-rocdown` does not import `rocci-docs`.

That writes `dist/rocci.dev`, `dist/site.tgz`, `dist/islands`, and
`publish.json`. GitHub Actions workflow `site.yml` packages on linux/amd64
and, on `staging` or `production` only, scps those artifacts to the origin
using the matching GitHub Environment. Land work on `main`; promote to
`staging` to publish behind Access, then to `production` for the public
hostname. Pull requests never deploy.

To promote the current `main` revision to staging locally, run
`uv run rocci-ops promote staging`. This rebases `staging` onto `main`,
pushes `staging` to `origin`, and restores the branch that was active when it
started. After a signed-out staging smoke,
`uv run rocci-ops promote production` pushes `origin/staging` to
`origin/production` (creates the branch on first use). That push runs hosted CI
and Knowledge, then the site package/deploy job. Do not promote production
until staging has been smoked.

To publish a GitHub release from `origin/main`, run
`uv run --no-dev rocci-ops release patch` (or `minor`, `major`, or `vX.Y.Z`,
optionally `--from BRANCH`). That is the only operator path that creates an
immutable `v*` tag. It writes the workspace version to `Cargo.toml` and
`Cargo.lock`, pushes that commit to the target branch, waits for hosted lint
and Test Workspace checks, then pushes the tag so `release.yml` can package
archives. `--dry-run` prints the resolved tag and whether those files already
match. Pass `--force` only to move an existing `v*`.
`uv run --no-dev rocci-ops release dev` force-moves the rolling `dev`
prerelease tag (no version rewrite). The same cut can run from
**Actions → Cut release** (`workflow_dispatch` on `cut-release.yml`; not
attached to the `release`, `staging`, or `production` environments).
`promote tag` is gone; `promote` is only `staging` and `production`.
A later `git pull` then reports `! [rejected] dev -> dev (would clobber
existing tag)` unless this repo force-updates that tag on fetch:

```sh
git config --local --add remote.origin.fetch '+refs/tags/dev:refs/tags/dev'
```

Do not force-fetch all tags; `v*` releases stay immutable. To replace local
`dev` once without changing config, run `git fetch origin tag dev --force`.

To test a pull request in this worktree when an agent already has the PR
branch checked out, run `uv run rocci-ops pr-checkout 39`. With no argument,
that lists open PRs via `gh`. Quote `#39` in the shell, or pass a GitHub PR
URL or branch. That fetches the tip and switches this checkout to a local
`pr/<branch>` branch.

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

`cargo test --workspace` is the offline crate suite. Roc on `PATH` does not enable generated-app builds; set `ROCCI_REQUIRE_ROC=1` for that lane. `uv run rocci-ops ci` runs the GitHub Actions validation jobs on this OS (lint, tests, fixtures-and-docs, editors, knowledge, and Linux `roc`). It does not run the ubuntu/macos matrix or release cross-platform builds. Pass job names to run a subset, for example `uv run rocci-ops ci lint test`.

GitHub Actions CI, Knowledge, Site, and Release run on GitHub-hosted runners (`ubuntu-latest` / `macos-latest`). CI and Knowledge run automatically on push to `main`, `staging`, and `production`. They do not run on every pull request. A reviewer comments `/ci` or `/CI` (conversation, review body, or inline review comment) to queue hosted CI for that PR head. Owners, members, and collaborators may do this, including on forks. Dependabot PRs need `/ci` the same way. `/ci-local` and `/cl-local` are accepted but queue the same hosted jobs. Site package and deploy use `ubuntu-latest`; deploy secrets stay on the `staging` and `production` GitHub Environments; CI and Knowledge jobs cannot read them.

## Contributing

This preview does not accept pull requests; that may change later.
[CONTRIBUTING.md](CONTRIBUTING.md) is the current contract, including crate
ownership and `/ci`. Conduct, security, support, and
governance live beside it at the repository root.

## License

Copyright 2026 Nils Hjelte.

Rocci is licensed under the [Apache License, Version 2.0](LICENSE).
Third-party components retain their own licenses; see
[THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md).
