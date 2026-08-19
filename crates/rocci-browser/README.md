# rocci-browser

Product-blind project browser: a registry of directories, a two-stage fuzzy
picker, and out-of-process adapters.

The host never sniffs file formats. Adapters on `PATH` claim paths, list
documents, and `open` an HTTP origin. Direct product `run` commands keep their
one-shot preview windows.

## Commands

```sh
cargo run -q -p rocci-browser -- add ./my-project
cargo run -q -p rocci-browser -- list
cargo run -q -p rocci-browser -- open my-project --no-window --json
cargo run -q -p rocci-browser -- open my-project --document about --no-window --json
cargo run -q -p rocci-browser -- tui --no-window --json
cargo run -q -p rocci-browser -- remove my-project
```

`--root` selects the directory that contains `.rocci/browser.toml` (defaults to
the current working directory). That file is data: plugin rows and project ids
live there, not in host source.

`open --no-window --json` prints `{ "url", "title" }` and keeps the adapter
origin up until stdin closes (or the process is signaled).

With no arguments, `rocci-browser` opens a persistent preview window: a
host-owned launcher, Cmd-P (Ctrl-P) picker overlay, and `load_url` of adapter
origins. Tab in the picker input does not move focus. Cmd-K remains in-page
Go to File. Overlay back/home/reload still apply to the visible origin.

TUI keys: type to filter, Enter opens a target home, Tab lists adapter
documents then Enter opens one, Shift-Tab / Escape returns to targets.

## Registry and plugins

| Override | Browser directory |
| --- | --- |
| `ROCCI_BROWSER_DIR` | that directory |
| `ROCCI_HOME` | `$ROCCI_HOME/.rocci/browser` |
| default | `$HOME/.rocci/browser` |

Files: `projects.json` and `plugins/*.toml`. Window geometry stays in the
existing `state/windows.json` key `browser` once a preview window exists.

Plugin discovery order: `plugins/*.toml`, then repo-local `.rocci/browser.toml`
`[[plugin]]` rows, then `ROCCI_BROWSER_PLUGINS` (`id=bin` or executable names).
Repo-local `[[project]]` rows are unioned with `projects.json`; relative paths
and plugin bins that contain a slash resolve against the directory that owns
`.rocci/`. A missing binary is a warning next to the plugin id.

During workspace development, `cargo build -p rocci-browser -p rocci-cli -p
rocci-rocdown-cli -p rocci-okf` then either put `target/debug` on `PATH` or
keep relative `target/debug/<bin>` plugin rows in `.rocci/browser.toml`.

Illustrative manifest:

```toml
id = "fixture"
bin = "python3"
argv = ["-u", "crates/rocci-browser/tests/fixtures/adapter.py"]
```

`bin` is looked up on `PATH`. During workspace development, put `target/debug`
on `PATH` (or pass absolute bins in tests). First-party product CLIs expose a
`browser-adapter` stdio command; plugin rows pass that as `argv`. Direct
product `run` still opens a one-shot preview window.

Host tests never spawn those product adapters. They use the fixture under
`tests/fixtures/` only.

## Protocol

Newline-delimited JSON-RPC 2.0, `protocolVersion` `1`. Methods: `initialize`,
`probe`, `listDocuments`, `open`, `shutdown`. The host sends `initialize`
first. Unknown methods are ignored.

## Tests

`cargo test -p rocci-browser` uses the fixture adapter under
`tests/fixtures/`. It does not start product CLIs or compile other formats.
