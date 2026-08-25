# rocci-web overlay

Exploratory local platform: **basic-webserver 0.16.0 plus `pf.Rocci` wraps**.
Not a shipped Rocci host. Does not fork the Rust host for SSE idle timeout.

## Fetch

```sh
./experiments/rocci-web/fetch-bws.sh
```

Unpacks the same tarball as `crates/rocci-cli/src/dispatch.rs` `PLATFORM` into
gitignored `vendor/`, then copies `overlay/Rocci.roc` and exposes `Rocci`.

## Gallery

```sh
cargo run -q -p rocci-cli -- run experiments/method-role-lib --no-window
```

The app header is `pf: platform "../rocci-web/vendor/main.roc"` (Roc requires
the `.roc` suffix). See `overlay/ROUTES_PROBE.md` and
`overlay/HOSTED_REGISTER.md` for constructor encodings that stopped.

## Out of bound

No `@method:role` changes, no generated dispatch, no `libhost.a` rebuild.
