# rocci-platform

In-tree Roc platform for Rocci apps. Apps pin this crate as `pf`, not
basic-webserver plus a Rocci package.

## Host origin

The Rust host and Roc modules are a vendored snapshot of sibling
[`roc-basic-webserver`](https://github.com/koliyo/roc-basic-webserver) at
`241061577473444a11777abc2f9376cc224e0e5f` (0.16 line). That host is UPL-1.0;
see `LICENSE-UPL`. This is not a git submodule. Host crate versions are
workspace-compatible (not the upstream `=` pins) so one Cargo.lock can
resolve; `libsqlite3-sys` stays on the `0.30` line that `rocci-wasi-http`
already links.

The wasm apply platform (`crates/rocci-roc-host`) and the WASI HTTP adapter
are not this crate.

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

## Build the native host

```sh
crates/rocci-platform/build.sh
```

That writes `platform/targets/<native>/libhost.a`. Glue regeneration
(`roc glue`) is documented in a later packaging phase.
