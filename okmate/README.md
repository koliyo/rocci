# Okmate

Okmate (open knowledge mate) is a standalone knowledge application for Open
Knowledge Format (OKF) bundles. The binary is `okmate`.

## Stack

- **Engine:** the portable [`okf`](../crates/okf) crate (the only in-repo Rust
  dependency)
- **HTML:** Askama 0.16
- **HTTP:** Axum 0.8
- **Morph / SSE:** official Datastar Rust SDK 0.4 (`axum` feature) plus a
  pinned `assets/datastar.js`
- **Desktop:** tao / wry / rfd, in this crate (`okmate/src/desktop.rs`)

This directory is shaped so it can become its own git repository. It does not
interpret `.rocci` templates. `cargo test -p okmate` does not require Roc.

## Depends on `okf` only

Okmate must not depend on any `rocci-*` crate. Workspace class `okmate` may
depend on `okf-engine` only.

## Coexistence with `rocci-okf`

`okmate check` is the knowledge **application** CLI. `rocci-okf check` remains
the Rocci tool (CI, `manage-rocci-knowledge`) until an explicit cutover. Engine
JSON for `check` / `inspect` / `search` / `benchmark` matches `rocci-okf` so
agents can switch the binary name without a new schema. There is no `--host`
flag.

Settings live under `~/.okmate/` (`OKMATE_CONFIG`, `OKMATE_CACHE`,
`OKMATE_STATE`). If `~/.okmate/config.toml` is missing, Okmate may import
`~/.rocci/okf.toml` once. Do not treat `~/.rocci/` as the long-term path.

## Extract as a standalone repository

1. Copy this `okmate/` directory to its own git root.
2. Change the `okf` path dependency in `Cargo.toml` to a git (or crates.io)
   dependency, for example:
   `okf = { git = "https://github.com/koliyo/rocci", package = "okf" }`
3. Drop the `"okmate"` workspace member line from Rocci’s root `Cargo.toml`
   and the `okmate` class from `workspace_deps.py`.
4. Keep Askama, Axum, `datastar`, clap, tokio, notify, tao, wry, and rfd as
   crates.io dependencies. Do not add `rocci-*`.

## CLI

| Command | Purpose |
| --- | --- |
| `okmate check [root]` | Validate a bundle (`--format terminal\|json`, `--profile`) |
| `okmate inspect catalog\|concept\|graph` | Engine JSON inspect |
| `okmate search <query> [root]` | Metadata and heading search JSON |
| `okmate benchmark <toml> [root]` | Retrieval benchmark |
| `okmate build [root] -o <dir>` | Engine artifacts plus Askama HTML |
| `okmate view [path]` | Live preview; omit `--no-window` to open tao/wry |
| `okmate roots` | Print resolved root paths (`--format json\|paths`, `--sync` / `--no-sync`) |
| `okmate sync [id]` | Fetch configured git roots |

```sh
okmate check knowledge --profile rocci --format json
okmate inspect catalog knowledge
okmate inspect concept architecture/system-overview knowledge
okmate inspect graph knowledge
okmate search "system overview" knowledge --profile rocci
okmate benchmark knowledge/retrieval-benchmark.toml knowledge
okmate build knowledge -o dist/knowledge
okmate view knowledge --no-window
okmate roots --format json --no-sync
okmate sync
```

`check`, `inspect`, `search`, and `build` stay single-root. Agents list
resolved folders first:

```sh
okmate roots --format paths | while IFS= read -r root; do
  okmate inspect catalog "$root"
done
```

`--format json` emits `{ id, kind, path, revision, incoming, enabled, error }`
and never includes tokens or resolved secrets. If the config is missing or
`roots` is empty, `./knowledge` is printed when that directory exists.

`view` serves the live HTML tree on localhost (pass `--public` to bind every
interface). Settings POST is `/__okmate/settings` and loopback-only. In the
desktop window, **Choose folder…** uses `rfd` via wry IPC (`pick-folder`), not
an HTTP pick-folder route. Without a window, paste the folder path.

JSON shapes match `rocci-okf` for engine commands. Git cache is
`OKMATE_CACHE` (default `~/.okmate/cache`).
