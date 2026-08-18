platform ""
    requires {
        main! : {} => [Ok({}), Err([Exit(I32)])]
    }
    exposes []
    packages {}
    provides { "roc_main": main_for_host! }
    targets: {
        inputs_dir: "targets/",
        wasm32: { inputs: ["host.o", app] },
    }

main_for_host! : {} => I32
main_for_host! = |{}|
    match main!({}) {
        Ok({}) => 0
        Err(Exit(code)) => code
    }
