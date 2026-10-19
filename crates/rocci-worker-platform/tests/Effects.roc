app [Context, program] { pf: platform "../platform/main.roc" }

import pf.Effect

Context : {
    prefix : Str,
    resource : Effect.Resource,
    render : Str -> Str,
}

program = { init!, call! }

init! = || {
    prefix = "retained Roc context → 🌍 with a heap string"
    { prefix, resource: Effect.resource!(73), render: |value| "[${prefix}] ${value}" }
}

call! = |input, context| {
    first = Effect.echo!(input)
    result = Effect.delay!(41)
    second = Effect.echo!(first)
    resource = Effect.resource!(19)
    value = Effect.resource_value!(resource)
    retained = Effect.resource_value!(context.resource)
    match result {
        Ok(number) => "${(context.render)(second)}:${number.to_str()}:${value.to_str()}:${retained.to_str()}"
        Err(Rejected) => "rejected:${(context.render)(second)}"
    }
}
