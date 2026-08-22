# Frozen Rocci app-docs inventory

This file freezes include/exclude rules and the staging-tree shape for
`rocci-docs`. Tests that enforce the tree land with the crate (Phase 1).
Published example source uses the completed verb-first handler cutover
(`@method:view`, `@method:fragment`, `@method:command`, and `@get:live`).

Discovery is the catalog `apps.toml`, not every directory under `examples/rocci`.
`handler-matrix` is included as the exhaustive method/role reference.

## Include

- `*.rocci`
- authored `*.roc` (not under excluded directories)
- `rocci.toml`
- `assets/**` files that are not empty keepers
- explicit extra file paths listed on a catalog row (`files`)

## Exclude

- directories named `generated/`, `target/`, `dist/`, `.git/`
- `*.db` and SQLite sidecars
- `.gitkeep` and empty keeper files
- editor swap files (`*~`, `*.swp`, `*.swo`, `.#*`)
- authored `*.rocdown` (pages, not source listings)
- `README.md` (local run notes)

A catalog app without `index.rocdown` is an error.

## Staging tree

```text
example-docs/
  index.rocdown                 # generated app index
  <id>/
    index.rocdown               # copy of authored docs
    extra.rocdown               # extra authored pages, if any
    source/
      index.rocdown             # generated file list
      Counter.rocci.rocdown     # :include of staged source (no `..`)
    snippets/
      Counter.rocci             # copy used by :include
```

Canonical routes after mount: `/examples/<id>/` and
`/examples/<id>/source/<file>/`. Include paths are relative, never `..`,
NUL, or absolute.
