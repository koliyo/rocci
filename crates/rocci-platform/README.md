# rocci-platform

In-tree Roc platform for Rocci apps. Apps pin this crate as `pf`, not
basic-webserver plus a Rocci package. Generated `rocci run` / `rocci build`
apps and the custom datastar/snake examples use this pin. `--http-module`
and wasm apply (`crates/rocci-roc-host`) are **not** this platform.

The platform exposes `Datastar` and `Html` (`import pf.Datastar` /
`import pf.Html`). `Html` is the basic-webserver module, including tag
helpers such as `Html.div`. Rocci compiler helpers live on the same
module: `element` / `void_element` / `attribute` / `boolean_attribute` /
`fragment` (sibling nodes as one `Node`) / `empty`.
`Html.render_fragment` remains the nominal rendered-string type.
`0.16.0` pins still get staged wrapper copies.

## Host origin

The Rust host and most `platform/` Roc modules are a vendored snapshot of
[roc-lang/basic-webserver](https://github.com/roc-lang/basic-webserver)
at `50e064cdd1c4562c293598c61f6ce7a895d99bcf` (0.16 line). Copyright
© 2023 Richard Feldman and subsequent Roc authors. The full UPL text is
`LICENSE-UPL`. This is not a git submodule. Musl libc/crt/unwind and the
macOS `libSystem.tbd` under `platform/targets/` are stored with Git LFS.
`libhost.a` is rebuilt by `build.sh` and is not committed.

Rocci-original modules in this crate (`platform/Datastar.roc` and the
compiler helpers on `platform/Html.roc`) are Apache-2.0, same as the rest
of Rocci. Crate SPDX is `Apache-2.0 AND UPL-1.0`.

Host crate versions are workspace-compatible (not the upstream `=` pins)
so one Cargo.lock can resolve; `libsqlite3-sys` stays on the `0.30` line
that `rocci-wasi-http` already links.

## App pin (dev)

```roc
app [Context, program] {
    pf: platform "crates/rocci-platform/platform/main.roc",
}
```

Use an absolute path from staged `rocci` workspaces. `examples/hello-web.roc`
pins `../platform/main.roc`. Default listen is `127.0.0.1:8000`
(`Server.default_config`). `hello-web.roc` also honors
`ROC_BASIC_WEBSERVER_PORT` and `ROC_BASIC_WEBSERVER_HOST`.

There is no GitHub release URL in this crate. Apps pin the path (or a
local `.tar.zst` from `bundle.sh`).

## Build the native host

```sh
crates/rocci-platform/build.sh
```

That writes `platform/targets/<native>/libhost.a`. `build.sh --all` is
not proven. Missing triples: `x64mac`, `x64musl`, `arm64musl`, `x64win`,
`arm64win`. wasm32 is out of bound (apply stays `rocci-roc-host`).

## Bundle

```sh
crates/rocci-platform/bundle.sh
```

Writes a `.tar.zst` next to this README: `platform/*.roc` plus the native
`libhost.a`. Pin that archive as `pf` when you want a package instead of
a path.

## Regenerating glue

When hosted exports in `platform/main.roc` change:

```sh
roc glue /path/to/roc/crates/compiler/glue/src/RustGlue.roc ./src/ platform/main.roc
```

That overwrites `src/roc_platform_abi.rs`. Needs a Roc compiler source
checkout for `RustGlue.roc`.
