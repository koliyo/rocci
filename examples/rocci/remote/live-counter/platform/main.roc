platform "private-counter-values"
    requires { call : Str -> Str }
    exposes []
    packages {}
    provides { "roc_call": call_for_host }
    targets: {
        inputs_dir: "targets/",
        wasm32: { inputs: ["host.o", app], exports: ["invoke", "input_ptr", "output_ptr", "output_len", "arena_used"] },
        arm64mac: { inputs: ["host.o", app] },
        x64glibc: { inputs: ["host.o", app] },
    }

call_for_host : Str -> Str
call_for_host = |input| call(input)
