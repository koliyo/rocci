# Method-role library on an opinionated BWS overlay

Exploratory hybrid: **routes in Roc, markup in `Ui.rocci`**. Not a cutover
from `@method:role`, and not a change to generated dispatch.

The app uses the local overlay in `../rocci-web` (fetch first):

```roc
app [Context, program] {
    pf: platform "../rocci-web/vendor/main.roc",
    http: "https://github.com/roc-lang/http/releases/download/1.0.0/…",
}

program = Rocci.program({
    init!,
    respond!,
})
```

`pf.Rocci` owns view / fragment / events / unfold wraps and command-from-header-list.
`main.roc` still `match`es `(method, path)`. Call `Rocci.command!(request.headers())`
from the app; `request.headers()` inside another module crashes this nightly.

A `List` of `Rocci.view("/", home!)` constructors **linked**, then crashed at
runtime. `requires { routes }` does not typecheck. Hosted register needs a
rebuilt `libhost.a`. Notes: `../rocci-web/overlay/ROUTES_PROBE.md`.

## Run

```sh
./experiments/rocci-web/fetch-bws.sh
cargo run -q -p rocci-cli -- run experiments/method-role-lib --no-window
```

## Out of bound

No `@method:role` changes, no generated dispatch, no `Rocci.component` I/O.
