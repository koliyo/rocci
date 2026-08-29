# rocci-docs

Workspace tool that inventories cataloged Rocci apps (`examples/rocci/apps.toml`)
and writes a Rocdown staging tree. It does not render HTML, compile Roc, or
package servers. Binary name: `rocci-docs` (like `rocci-ungram`, not a fourth
product CLI).

```sh
cargo run -q -p rocci-docs -- --catalog examples/rocci/apps.toml --output dist/example-docs
cargo run -q -p rocci-docs -- --catalog examples/rocci/apps.toml --print-live
cargo run -q -p rocci-docs -- --catalog examples/rocci/apps.toml --output dist/example-docs --all
```

Each catalog row may set `site` (bool, default `true`) to include the app in
the rocci.dev `/examples/` tree. `site = false` keeps the row for local
inventory, coverage, and path checks without staging it. `hosting = "live"`
requires `site = true`. Unknown keys are rejected so a misspelling of `site`
cannot silently default to included.

`--print-live` lists `id`, catalog-relative `path`, and `entry` for
`hosting = "live"` and `site = true` rows. Docs-only and `site = false` ids
are omitted. Staging writes only `site = true` apps unless `--all` is set
(local preview of excluded rows). `package site` and `build site` must not
pass `--all`.

For each published `.rocci` file, `rocci-docs` parses attached `## ` doc
comments (same attachment rules as Roc: no blank line before `@`) and writes
them onto the generated source page above `:include`. Tutorial prose stays in
colocated `.rocdown`. This crate depends on `rocci-template` for parse only; it
does not compile Roc or render HTML.

`site/rocdown.toml` mounts `../dist/example-docs` at prefix `examples`. Run
this tool before `rocdown check site`, `rocdown view site`, or
`rocdown package site`. `rocci-rocdown` does not depend on this crate.

Staging writes a complete sibling tree and replaces the previous output only
after success. A failed catalog copy or write leaves the previous tree in
place.

The generated `/examples/` index lists `site = true` cataloged Rocci apps only.
Rocdown examples belong on the Rocdown product lane.

Reserved live hostnames use `<id>-example-staging.rocci.dev` (Universal
SSL) and `<id>.examples.rocci.dev` (needs ACM). Those names are **not
advertised**; the generated examples table labels them `planned live` and
does not emit the URLs.
