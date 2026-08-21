# rocci-browser

Product-blind project browser: a registry of directories, a two-stage fuzzy
picker, and out-of-process adapters.

The host never sniffs file formats. Adapters on `PATH` claim paths, list
documents, and `open` an HTTP origin. Direct product `run` commands keep their
one-shot preview windows.

## Build

The host binary is not enough. This workspace's `.rocci/browser.toml` execs
`target/debug/rocdown`, `target/debug/rocci-okf`, and `target/debug/rocci` with
`argv = ["browser-adapter"]`. `cargo run -p rocci-browser` does not rebuild
those plugins.

```sh
cargo build -p rocci-browser -p rocci-cli -p rocci-rocdown-cli -p rocci-okf
```

A stale plugin prints `unrecognized subcommand 'browser-adapter'` and
`adapter … closed stdout during initialize`. The host still opens targets that
a remaining up-to-date adapter claims. Rebuild any plugin whose `browser-adapter`
command is missing.

Installed `~/.local/bin` copies are unused while plugin `bin` paths contain a
slash: those resolve against the directory that owns `.rocci/`, not `PATH`.

## Commands

```sh
cargo run -q -p rocci-browser -- add ./my-project
cargo run -q -p rocci-browser -- list
cargo run -q -p rocci-browser -- open my-project --no-window --json
cargo run -q -p rocci-browser -- open my-project --document about --no-window --json
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

Switching targets reuses a still-warm adapter origin for the same root. The
previous child is grace-stopped after about 30s. The Dev iframe follows the
session inspector URL.

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
`.rocci/`. A missing binary is a warning next to the plugin id. See [Build](#build) for
the four-package command this repo's plugin rows expect.

Illustrative manifest:

```toml
id = "fixture"
bin = "python3"
argv = ["-u", "crates/rocci-browser/tests/fixtures/adapter.py"]
```

`bin` without a slash is looked up on `PATH`. First-party product CLIs expose a
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
`tests/fixtures/`. It does not start product CLIs, compile other formats, or
call `codesign`.

## macOS app

On macOS, wrap the same graphical `preview()` binary in an ad-hoc **Rocci
Browser.app**:

```sh
uv run --project tools/rocci-ops rocci-ops bundle browser-macos
open "target/release/bundle/macos/Rocci Browser.app"
```

Or `cargo build --release -p rocci-browser` then
`cargo run --release -p rocci-browser -- package`. This does not call
`rocci bundle` (that still packages authored Roc apps) and does not copy
product CLIs into the bundle. Adapters stay on `PATH` or as absolute plugin
`bin` paths.

Finder / Dock launch repairs a sanitized GUI `PATH` by prepending existing
`/opt/homebrew/bin`, `/usr/local/bin`, `$HOME/.local/bin`, and
`$HOME/.cargo/bin`. It does not read `.rocci/browser.toml` from cwd `/`.
`--root` still selects a repo file. After a graphical quit with a real repo
root, the host writes `last-root` under the browser directory and restores it
on the next bundled launch.

There is no `tui` command. Agents and machines without a display use
`open --no-window`. Production signing and notarization are **planned**, not
shipped; the `.app` is ad-hoc signed only.
