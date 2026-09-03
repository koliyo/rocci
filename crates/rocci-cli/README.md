# rocci-cli

Command-line interface and development driver for `.rocci` templates, Roc applications, desktop bundling, and Datastar asset management.

## Binary

Executable name: `rocci`

## Commands

```sh
# Validate configuration
cargo run -p rocci-cli -- validate [rocci.toml]

# Compile a single .rocci template to Roc
cargo run -p rocci-cli -- build path/to/App.rocci [-o output.roc]

# Experimental WASI HTTP component (not `--host wasm`; `rocci run` stays the native Rocci platform).
# Pass the entry `.rocci`; sibling `.rocci` / `.roc` in that tree are included.
cargo run -p rocci-cli -- build --http-module \
  examples/rocci/standalone/counter/Counter.rocci -o http-module.wasm
# Multi-file: same flag, entry only
#   examples/rocci/standalone/live-counter/LiveCounter.rocci
mkdir -p .counter-data
wasmtime serve -Sp3 -Scli --env DB_PATH=./counter.db \
  --dir=.counter-data::. --dir=./http-module.assets::/assets \
  --addr 127.0.0.1:8080 http-module.wasm

# Package a Linux server binary plus assets (not a macOS .app)
cargo run -p rocci-cli -- build --release examples/rocci/custom/datastar [-o target/release/rocci-server] [--target x64musl|arm64musl|…]
cargo run -p rocci-cli -- build --release examples/rocci/standalone/counter/Counter.rocci
# Stream Roc compiler output and phase timings for a release build
cargo run -p rocci-cli -- build --release --verbose examples/rocci/custom/datastar --target x64musl
# Available backend modes: speed, size, dev, interpreter
cargo run -p rocci-cli -- build --release --opt dev examples/rocci/standalone/blocks/backend/Blocks.rocci --target x64musl

`rocci build --release` defaults to Roc's optimized `speed` backend. The
repository site packager currently passes `--opt dev` for all live examples as
a temporary workaround for an optimized-backend compiler recursion. Dev
artifacts are functional but may be larger and slower; pass `--opt speed` for
production-performance output when the compiler path is known to work.

# Run a standalone app directory (unique @init) or a named .rocci entry
cargo run -p rocci-cli -- run examples/rocci/standalone/counter
cargo run -p rocci-cli -- run examples/rocci/standalone/counter/Counter.rocci

# Print compile and wait phases to stderr
# `rocci run` always prints template/stage/roc phase lines; Roc compile of a large
# app can take minutes. `--verbose` adds per-module timings, listen heartbeats,
# and streams `roc build --opt=dev --timings --verbose` before starting the server.
cargo run -p rocci-cli -- run --verbose examples/rocci/standalone/live-counter/LiveCounter.rocci

# Pause automatic page refresh (watch/rebuild still runs)
cargo run -p rocci-cli -- run examples/rocci/standalone/counter/Counter.rocci --no-live-reload

# Headless serve: append `?reload=0` to the printed URL to pause auto-refresh
cargo run -p rocci-cli -- run examples/rocci/standalone/counter/Counter.rocci --no-window

# Listen on every interface (default is localhost only; inspector stays loopback)
cargo run -p rocci-cli -- run examples/rocci/standalone/counter/Counter.rocci --no-window --public

# Run an authored Roc application directory
cargo run -p rocci-cli -- run examples/rocci/custom/datastar

# Render a single component with mock arguments in a preview window
cargo run -p rocci-cli -- view examples/rocci/standalone/counter/Counter.rocci --component CounterCard --arg count=3

# Browse all discovered components in an application directory
cargo run -p rocci-cli -- browse examples/rocci/standalone/counter

# Open the playground with a .rocci example (also accepts .rocdown / .md)
cargo run -p rocci-cli -- playground examples/rocci/standalone/counter/Counter.rocci

# Open a Rocdown document in the same playground
cargo run -p rocci-cli -- playground examples/rocdown/pages/Guide.rocdown

# Local mode: native compile plus Html.render snapshots for .rocci
cargo run -p rocci-cli -- playground examples/rocci/standalone/counter/Counter.rocci --mode local

# Snapshot a component to HTML (add --fragment to skip the html/body wrapper)
cargo run -p rocci-cli -- render examples/rocci/standalone/counter/Counter.rocci --fragment

# Inspect AST, component signatures, and source-map segments
cargo run -p rocci-cli -- inspect --ast examples/rocci/standalone/counter/Counter.rocci

# Run @test declarations via roc test (skips files with no tests)
cargo run -p rocci-cli -- test examples/rocci/standalone/styling/Styling.rocci

# Preview the profiling panel fixture
cargo run -p rocci-cli -- view crates/rocci-cli/templates/dev/MetricsPanel.rocci --component MetricsPanel

# Bundle an ad-hoc signed macOS application (host-native server; not --target musl)
cargo run -p rocci-cli -- bundle --config rocci.toml

# Roc process `--target` matches the Linux container CPU (Apple Silicon Docker →
# arm64musl; amd64 → x64musl). See docker/README.md and `rocci build --help`.
# Not for apply or macOS .app bundles. Linux OCI: `uv run rocci-ops serve app` after
# `rocci build --release --target …`.

# Pin or update Datastar JavaScript assets
cargo run -p rocci-cli -- datastar pin 1.0.2 --app examples/rocci/custom/datastar
cargo run -p rocci-cli -- datastar update --app examples/rocci/custom/datastar
```

`templates/dev/MetricsPanel.rocci` is the preview-origin Dev inspector. It has
tabs for Performance, Source, and Console. Source is a GET form (`tab`, `route`,
`view`) for original source, formatted AST, generated Roc, and generated HTML.
Long Source bodies scroll inside `.code-pane` without moving the preview page;
use the **Wrap** checkbox to toggle line wrapping (on by default). Original Rocci, Rocdown, and
Markdown, plus generated Roc and HTML, highlight with `rocci-highlight` `tok-*`
classes (playground token colors). AST stays escaped plaintext. OKF records
show Markdown source and built HTML; the Source dropdown omits AST and Generated Roc.
Console lists runtime messages teed from the session, including Roc process
stderr for `rocci run` (`GET /__rocci/logs`, SSE `GET /__rocci/logs/events`).
Handler and `@init` code can target that stream with `import pf.Stderr` and
`Stderr.line!` (see `examples/rocci/standalone/counter`; discard `StderrErr` when mixing with
other `?` errors). There is still no Rocci `@log` and no
logging from `@component` render functions. Console does not capture page
`console.*` (native Web Inspector remains the page console; that overlay wrap
is original inspector Phase 5 and is not shipped). Static
dev servers serve the panel at `GET /__rocci/dev?tab=&route=&view=` from the
current inspect snapshot; JSON for source views is
`GET /__rocci/inspect?route=&view=`. When a site rebuild fails, HTML responses
still serve the last output on disk (or a minimal shell when none exists) with a
native `<dialog>` listing the error. Close the dialog to read the page behind
it; the next failed rebuild shows the dialog again after live reload.
`rocci run` hosts the same panel on a
sibling loopback port, including `--no-window`. Overlay chrome docks that URL
right or bottom with DevTools-style dock icons over the panel corner (tabs pad
right); prefs persist in `~/.rocci/state/inspector.json`, and Open as page loads
the inspector as main content. The overlay does not embed compiler output. The panel is not a playground:
there is no editor and no WASM compile.

## Architectural Boundary

`rocci-cli` owns execution and orchestration for `.rocci` templates and Roc applications. It does not parse or execute `.rocdown` documents (which are owned by `rocci-rocdown-cli` / `rocdown`) or OKF bundles. `rocci run` on an OKF-looking `.md` file or bundle directory hints at `okmate view` by extension and leading-byte inspection only.

## Internal modules

`lib.rs` is the product barrel used by the `rocci` binary and by Rocdown
for shared host pieces. New orchestration belongs in the owning file or
subdirectory below, not a second public driver crate.

| Path | Role |
| --- | --- |
| `lib.rs` | Library barrel |
| `main.rs` | `rocci` clap dispatch |
| `src/run/` | `rocci run` orchestration and standalone plans |
| `src/dev_server/` | Shared preview and live-reload server used by Rocdown |
| `src/browse/` | Gallery compiler; not a Rocdown dependency |
| `src/dispatch/` | Generated HTTP dispatcher |
| `driver.rs` / `serve.rs` / `view.rs` / `inspect.rs` / `inspector.rs` | Compile driver, static serve, component preview, inspect, Dev overlay |
| `playground.rs` / `playground_html.rs` / `playground_compile.rs` | Playground host, HTML, local compile |
| `bundle.rs` / `http_module.rs` / `datastar_asset.rs` | macOS bundle, WASI HTTP module, Datastar pin |
| `rocci_test.rs` / `logs.rs` / `profile.rs` / `error_page.rs` / `path_hint.rs` | `rocci test`, log tee, metrics fixture, failed-rebuild dialog, `okmate` hint |
| `style.rs` / `native_target.rs` / `runtime_assets.rs` / `roc_module.rs` | Highlight CSS, host triple, staged assets, Roc module helpers |

`rocci_cli::` stays `pub` for modules the `rocci` binary, Rocdown, or
integration tests import. `dispatch`, `inspector`, `playground_compile`,
`playground_html`, `roc_module`, and `runtime_assets` are `pub(crate)`
(`render_file` is re-exported). Do not add CLI flags.

The split sequence and no-feature contract are in the
[implementation-structure plan](../../knowledge/plans/rocci/implementation-structure.md)
and the
[structure audit](../../knowledge/audits/rocci/implementation-structure.md).
Site-planning code still belongs in `rocci-rocdown` (`src/plan/`, `src/docs/`,
`src/lower/`, `src/catalog/`, `src/build/`), not here.
