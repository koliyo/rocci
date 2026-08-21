# rocci-docs

Workspace tool that inventories cataloged Rocci apps (`examples/rocci/apps.toml`)
and writes a Rocdown staging tree. It does not render HTML, compile Roc, or
package servers. Binary name: `rocci-docs` (like `rocci-ungram`, not a fourth
product CLI).

```sh
cargo run -q -p rocci-docs -- --catalog examples/rocci/apps.toml --output dist/example-docs
cargo run -q -p rocci-docs -- --catalog examples/rocci/apps.toml --print-live
```

`--print-live` lists `id`, catalog-relative `path`, and `entry` for
`hosting = "live"` rows. Docs-only ids are omitted.

`site/rocdown.toml` mounts `../dist/example-docs` at prefix `examples`. Run
this tool before `rocdown check site`, `rocdown run site`, or
`rocdown package site`. `rocci-rocdown` does not depend on this crate.

Live demo links use `<id>.examples.rocci.dev` (staging:
`<id>.examples.staging.rocci.dev`). Those hostnames are **planned** until a
staging origin deploy has served them.
