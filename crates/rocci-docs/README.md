# rocci-docs

Workspace tool that inventories cataloged Rocci apps and writes a Rocdown
staging tree. It does not render HTML, compile Roc, or package servers.

```sh
cargo run -q -p rocci-docs -- --catalog examples/rocci/apps.toml --output dist/example-docs
```

`rocdown package site` and local site preview expect this tree to exist before
the `examples` mount is checked. See the root `AGENTS.md` owning-layer table.
