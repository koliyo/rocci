# hello-web fixture

Roc app used with `platform/` if you want to try `roc build --target=wasm32`.
This nightly's wasm32 platform header accepts a single `provides` (`roc_main`),
so the HTTP guest that tests call is `src/hello_web.wat` exporting
`roc_init_for_host` / `roc_respond_for_host` / `roc_shutdown_for_host`. The
app `respond!` HTML is the same bytes the WAT guest emits.

Do not commit `platform/targets/wasm32/host.o`. Compile `platform/host.c` locally
if you need that input.
