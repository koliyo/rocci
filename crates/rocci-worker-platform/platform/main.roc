platform "rocci-worker-qualification"
    requires {
        [Context : context] for program : {
            init! : () => context,
            call! : Str, context => Str,
        }
    }
    exposes [Effect]
    packages {}
    provides {
        "roc_worker_init": init_for_host!,
        "roc_worker_call": call_for_host!,
        "roc_worker_drop_context": drop_context,
        "roc_worker_drop_string": drop_string,
    }
    hosted {
        "hosted_worker_echo": Effect.echo!,
        "hosted_worker_delay": Effect.delay!,
        "hosted_worker_resource": Effect.resource!,
        "hosted_worker_resource_value": Effect.resource_value!,
    }
    targets: {
        inputs_dir: "targets/",
        wasm32: {
            inputs: ["host.o", app],
            exports: ["worker_init", "worker_invoke", "worker_close", "input_ptr", "input_capacity", "output_ptr", "output_len", "live_bytes", "live_allocations", "heap_high_water", "worker_abi_version"],
        },
    }

import Effect

init_for_host! : () => Box(Context)
init_for_host! = || Box.box((program.init!)())

call_for_host! : Str, Box(Context) => Str
call_for_host! = |input, context| (program.call!)(input, Box.unbox(context))

drop_context : Box(Context) -> U64
drop_context = |_context| 0

drop_string : Str -> U64
drop_string = |_string| 0
