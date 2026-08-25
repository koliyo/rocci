# Method-role library on basic-webserver

Exploratory hybrid: **routes in Roc, markup in `Ui.rocci`**. Not a cutover
from `@method:role`, and not a change to generated dispatch.

## What the research wants

```roc
app [Context, program] {
    pf: platform "https://github.com/roc-lang/basic-webserver/releases/download/0.16.0/…",
    rocci: "Rocci",
}

program = Rocci.program({
    init!,
    routes: [
        Rocci.view("/", home!),
        Rocci.fragment(Post, "/actions/counter/increment", increment!),
        Rocci.command(Post, "/actions/live/increment", bump!),
        Rocci.live("/sse", live_slice!),
    ],
})
```

A true `package [Rocci] { pf: platform "…" }` is invalid: packages do not
take a platform, so they cannot `import pf.Server`. This example uses the
same sibling-module model as staged `Datastar.roc` / `Html.roc`:
`import Rocci`.

This Roc nightly **cannot codegen** constructors that close over handlers
in that sibling, or a `List` of those closures. Both SIGSEGV. Passing a
handler into `Rocci.view!("/", home!, request, context)` also SIGSEGVs.
Reading `request.headers()` from the sibling (command wrap) crashes at
runtime (`dispatch on a value that can never exist`).

## What this example compiles

```roc
program = Rocci.program({
    init!,
    respond!,
})
```

`Rocci.roc` owns view / fragment / events / unfold wraps. `main.roc`
`match`es `(method, path)`, calls the handlers, then the wraps. Command
empty-SSE vs 204 stays next to the handlers because it inspects the
request. Live poll stays in `main.roc` and uses `Rocci.unfold!`. Markup is
`Ui.rocci` (`@component` / `@css` only).

## Run

```sh
cargo run -q -p rocci-cli -- run experiments/method-role-lib --no-window
```

## Out of bound

No `@method:role` changes, no custom Roc platform, no `Rocci.component` I/O.
