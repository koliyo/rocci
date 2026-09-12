# rocci-platform

In-tree Roc platform for Rocci apps. Apps pin this crate as `pf`, not
basic-webserver plus a Rocci package. Generated `rocci run` / `rocci build`
apps and the custom datastar/snake examples use this pin. `--http-module`
and wasm apply (`crates/rocci-roc-host`) are **not** this platform.

The platform exposes `Datastar`, `Html`, and `Rocci` (`import pf.Datastar` /
`import pf.Html` / `import pf.Rocci`). `Html` is the basic-webserver module,
including tag helpers such as `Html.div`. Rocci compiler helpers live on
the same module: `element` / `void_element` / `attribute` /
`boolean_attribute` / `fragment` (sibling nodes as one `Node`) / `empty`.
`Html.render_fragment` remains the nominal rendered-string type.
`0.16.0` pins still get staged wrapper copies.

Node text and attribute escaping scans for whether a replacement is needed,
then either returns the original string or copies into a pre-sized byte
buffer. Attribute CR/LF stay `&#13;` / `&#10;`. This is not a second Html
type or a language change. A macOS `127.0.0.1` HostPage origin compared fold
versus scan/copy: responses were byte-identical; renderer gain was not
visible on that low-load path. Linux remains unmeasured (keep repeating).
Theme `Str` painters are unmeasured.

`pf.Rocci` is hosted glue onto `crates/rocci-template`: `compile!` returns
generated Roc source plus diagnostics; `parse!` returns a `format_ast`
S-expression plus diagnostics. File wrappers (`compile_file!` /
`parse_file!`) use `Path.read_utf8!`. This is **not** `rocci run` / `rocci
build`, **not** a Roc-native rewrite of the parser, and **not** apply or
HTML render (interpolations stay compiled Roc). Proof apps:
`examples/hello-compile.roc` and `examples/hello-parse.roc`.

## Host origin

HTTP/1.1 client-disconnect logs are classified in this host, not by
generated `main`.

The Rust host and most `platform/` Roc modules are a vendored snapshot of
[roc-lang/basic-webserver](https://github.com/roc-lang/basic-webserver)
at `50e064cdd1c4562c293598c61f6ce7a895d99bcf` (0.16 line). Copyright
© 2023 Richard Feldman and subsequent Roc authors. The full UPL text is
`LICENSE-UPL`. This is not a git submodule. `*.a`, `*.o`, `*.lib`, and
`*.tbd` are Git LFS. `libhost.a` is rebuilt by `build.sh` and is not
committed.

Rocci-original modules in this crate (`platform/Datastar.roc`,
`platform/Rocci.roc`, the compiler helpers on `platform/Html.roc`, and
the `hosted_rocci_*` symbols) are Apache-2.0, same as the rest of Rocci.
A later vendor snapshot of basic-webserver **must keep** those Rocci
hosted symbols, `platform/Rocci.roc`, and the `rocci-template` Cargo
dependency. Crate SPDX is `Apache-2.0 AND UPL-1.0`.

Host crate versions are workspace-compatible (not the upstream `=` pins)
so one Cargo.lock can resolve; `libsqlite3-sys` stays on the `0.30` line
that `rocci-wasi-http` already links.

## App pin (dev)

```roc
app [Context, program] {
    pf: platform "crates/rocci-platform/platform/main.roc",
}
```

Roc rejects absolute platform specs. Generated apps in a checkout use that
repo-relative pin in docs and inspect output. Staged `rocci` workspaces
rewrite it to a `../…/platform/main.roc` path from the temp app directory.
`examples/hello-web.roc` pins `../platform/main.roc`. Default listen is
`127.0.0.1:8000`
(`Server.default_config`). `hello-web.roc` also honors
`ROC_BASIC_WEBSERVER_PORT` and `ROC_BASIC_WEBSERVER_HOST`.

Hosted CI uploads an Actions artifact named `rocci-platform` containing
`rocci-platform.tar.zst` plus `rocci-platform.tar.zst.sha256`. Tag
releases (`dev` and `v*`) attach the same files:

`https://github.com/koliyo/rocci/releases/download/<tag>/rocci-platform.tar.zst`

The release archive includes `arm64mac` and `x64musl` `libhost.a`. A PATH
`rocci` that cannot see in-tree `platform/main.roc` pins this URL when
generating apps. Checkout `rocci` still pins the path. Apple Silicon macOS
and x64 Linux only until more triples ship. Do not treat the URL as the
default pin while a git checkout exists.

## Build the native host

```sh
crates/rocci-platform/build.sh
```

That writes `platform/targets/<native>/libhost.a`. `build.sh --all` is
not proven. Release bundles currently include `arm64mac` and `x64musl`.
Missing triples: `x64mac`, `arm64musl`, `x64win`, `arm64win`. wasm32 is
out of bound (apply stays `rocci-roc-host`).

## Bundle

```sh
crates/rocci-platform/bundle.sh
```

Writes a hashed `.tar.zst` next to this README (`platform/*.roc` plus
whatever `libhost.a` files exist under `platform/targets/`).
`--skip-build` bundles already-staged triples without calling
`build.sh`. CI copies the result to `rocci-platform.tar.zst`. Pin a
local archive as `pf` when you want a package instead of a path; do not
treat the GitHub URL as the default `rocci` pin.

## Regenerating glue

When hosted exports in `platform/main.roc` change, regenerate from
`crates/rocci-platform` with a matching compiler and `RustGlue.roc`
(Zig-compiler layout):

```sh
roc glue --no-cache /path/to/roc/src/glue/src/RustGlue.roc ./src/ platform/main.roc
```

That overwrites `src/roc_platform_abi.rs`. Then alias new generated
record types in `src/abi/mod.rs` (numbered `AnonStruct*` names shift).
`--http-module` and wasm apply stay unchanged: they are not this
platform.
