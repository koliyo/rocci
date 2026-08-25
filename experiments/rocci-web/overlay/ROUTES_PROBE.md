# Phase 3 routes probe

`requires { routes }` cannot name `Rocci.Route` (platform `requires` is parsed
before `import Rocci`). A `Routes : route_list` type variable stays opaque in
`respond_for_host!`, so `Rocci.dispatch!(program.routes, …)` does not typecheck.

A two-route `List` of `Rocci.view` / `Rocci.fragment` constructors plus
`Rocci.dispatch!` **did compile and link** (not a SIGSEGV). The server printed
`Listening` then crashed: `dispatch on a value that can never exist`.

This encoding is stopped. The gallery keeps Phase 2 `match` + wraps.
