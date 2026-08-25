# Phase 4 hosted-register fallback

Phase 3's `List` of constructors linked, then crashed at runtime
(`dispatch on a value that can never exist`). The remaining encoding is
init-time `pf` effects that store callbacks on the **host**.

The 0.16.0 release tarball ships a prebuilt `libhost.a`. New `hosted { }`
symbols require rebuilding that archive from
[basic-webserver 0.16.0](https://github.com/roc-lang/basic-webserver/tree/0.16.0)
source. This overlay does not vendor that Rust crate or add I/O.

Until a host rebuild exists, the gallery keeps Phase 2 `match` + `pf.Rocci`
wraps. Do not add hosted names that cannot link.
