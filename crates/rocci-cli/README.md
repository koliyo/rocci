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

# Package a Linux server binary plus assets (not a macOS .app)
cargo run -p rocci-cli -- build --release examples/datastar [-o target/release/rocci-server] [--target x64musl|arm64musl|…]
cargo run -p rocci-cli -- build --release examples/counter/Counter.rocci

# Run a standalone template application with live reload and embedded preview
cargo run -p rocci-cli -- run examples/counter/Counter.rocci

# Pause automatic page refresh (watch/rebuild still runs)
cargo run -p rocci-cli -- run examples/counter/Counter.rocci --no-live-reload

# Headless serve: append `?reload=0` to the printed URL to pause auto-refresh
cargo run -p rocci-cli -- run examples/counter/Counter.rocci --no-window

# Run an authored Roc application directory
cargo run -p rocci-cli -- run examples/datastar

# Render a single component with mock arguments in a preview window
cargo run -p rocci-cli -- view examples/counter/Counter.rocci --component CounterCard --arg count=3

# Browse all discovered components in an application directory
cargo run -p rocci-cli -- browse examples/counter

# Open the playground with a .rocci example (also accepts .rocdown / .md)
cargo run -p rocci-cli -- playground examples/counter/Counter.rocci

# Open a Rocdown document in the same playground
cargo run -p rocci-cli -- playground examples/rocdown/Guide.rocdown

# Local mode: native compile plus Html.render snapshots for .rocci
cargo run -p rocci-cli -- playground examples/counter/Counter.rocci --mode local

# Snapshot a component to HTML (add --fragment to skip the html/body wrapper)
cargo run -p rocci-cli -- render examples/counter/Counter.rocci --fragment

# Inspect AST, component signatures, and source-map segments
cargo run -p rocci-cli -- inspect --ast examples/counter/Counter.rocci

# Preview the profiling panel fixture
cargo run -p rocci-cli -- view crates/rocci-cli/templates/dev/MetricsPanel.rocci --component MetricsPanel

# Bundle an ad-hoc signed macOS application (host-native server; not --target musl)
cargo run -p rocci-cli -- bundle --config rocci.toml

# Roc process `--target` matches the Linux container CPU (Apple Silicon Docker →
# arm64musl; amd64 → x64musl). See docker/README.md and `rocci build --help`.
# Not for apply or macOS .app bundles. Linux OCI: `./docker/run-app.sh` after
# `rocci build --release --target …`.

# Pin or update Datastar JavaScript assets
cargo run -p rocci-cli -- datastar pin 1.0.2 --app examples/datastar
cargo run -p rocci-cli -- datastar update --app examples/datastar

# Speak the rocci-browser adapter protocol on stdio (probe / listDocuments / open)
cargo run -p rocci-cli -- browser-adapter
```

`templates/dev/MetricsPanel.rocci` is the preview-origin Dev inspector. It has
tabs for Performance, Source, and Console. Source is a GET form (`tab`, `route`,
`view`) for original source, formatted AST, generated Roc, and generated HTML.
Long Source bodies scroll inside `.code-pane`. Original Rocci, Rocdown, and
Markdown, plus generated Roc and HTML, highlight with `rocci-highlight` `tok-*`
classes (playground token colors). AST stays escaped plaintext. OKF records
show Markdown source and built HTML; the Source dropdown omits AST and Generated Roc.
Console lists runtime messages teed from the session, including Roc process
stderr for `rocci run` (`GET /__rocci/logs`, SSE `GET /__rocci/logs/events`).
Handler and `@init` code can target that stream with `import pf.Stderr` and
`Stderr.line!` (see `examples/counter`; discard `StderrErr` when mixing with
other `?` errors). There is still no Rocci `@log` and no
logging from `@component` render functions. Console does not capture page
`console.*` (native Web Inspector remains the page console; that overlay wrap
is original inspector Phase 5 and is not shipped). Static
dev servers serve the panel at `GET /__rocci/dev?tab=&route=&view=` from the
current inspect snapshot; JSON for source views is
`GET /__rocci/inspect?route=&view=`. `rocci run` hosts the same panel on a
sibling loopback port, including `--no-window`. Overlay chrome docks that URL
right or bottom with DevTools-style dock icons over the panel corner (tabs pad
right); prefs persist in `localStorage`, and Open as page loads the inspector as
main content. The overlay does not embed compiler output. The panel is not a playground:
there is no editor and no WASM compile.

## Architectural Boundary

`rocci-cli` owns execution and orchestration for `.rocci` templates and Roc applications. It does not parse or execute `.rocdown` documents (which are owned by `rocci-rocdown-cli` / `rocdown`) or OKF bundles. `rocci run` on an OKF-looking `.md` file or bundle directory hints at `rocci-okf run` by extension and leading-byte inspection only.
