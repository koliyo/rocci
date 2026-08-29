# hello-web fixture

Roc app used with `platform/` (`roc build --target=wasm32`). This nightly's
wasm32 platform header accepts a single `provides` (`roc_main`), so the HTTP
guest that tests call is the linked object in `src/hello_web.wat` exporting
`roc_init_for_host` / `roc_respond_for_host` / `roc_shutdown_for_host`. The
app `respond!` HTML is the same bytes the WAT guest emits.

```sh
roc build --target=wasm32 --opt=dev --output=hello-web.wasm fixtures/hello-web/main.roc
```
