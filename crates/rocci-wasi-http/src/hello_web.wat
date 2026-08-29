(module
  (import "host" "hosted_emit_ordinary" (func $emit (param i32 i32 i32)))
  (memory (export "memory") 1)
  (data (i32.const 16) "<!doctype html><html><body>hello-web</body></html>")
  (func (export "roc_init_for_host") (result i32)
    i32.const 0)
  (func (export "roc_respond_for_host") (result i32)
    i32.const 200
    i32.const 16
    i32.const 50
    call $emit
    i32.const 0)
  (func (export "roc_shutdown_for_host") (result i32)
    i32.const 0)
)
