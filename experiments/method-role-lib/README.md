# Method-role library on basic-webserver

Exploratory encoding of Rocci's closed `@method:role` matrix as Roc
helpers on basic-webserver 0.16. This is **not** a product cutover and does
not change the `.rocci` grammar or generated dispatch.

The interesting fork from the research: **routes are Roc, markup stays
`.rocci`**. Authors write `match` on `(method, path)` (the same shape as
generated dispatch) and call role wraps. Illegal pairs stay unrepresentable
because each wrap is a separate function (`view!` is GET HTML, `command!`
does not return a fragment, and so on).

## Compiler limits (this experiment)

A `List` of route tags that **hold handler closures** SIGSEGVs the current
Roc compiler, so `Rocci.program({ routes })` is not representable yet.

An authored sibling module that `import pf.Server` also SIGSEGVs for this
app (staged `Html.roc` / `Datastar.roc` are fine). So:

| Lives in | What |
| --- | --- |
| `Rocci.roc` | Platform-free path helpers (`prefix_remainder`, `slash_alternate`) |
| `main.roc` | HTTP/SSE wraps, uniqueness, Datastar-Request, live poll, `match` dispatch |

`program = { init!, respond!, shutdown! }` uses field punning, as in the
Datastar gallery. Do not put an inline lambda in that record.

## Run

From the repository root of this worktree:

```sh
cargo run -q -p rocci-cli -- run experiments/method-role-lib --no-window
```

Then open the printed URL, or (replace `PORT`):

```sh
curl -s http://127.0.0.1:PORT/
curl -s -X POST -H 'Datastar-Request: true' http://127.0.0.1:PORT/actions/counter/increment
curl -s -o /dev/null -w '%{http_code}\n' -X POST http://127.0.0.1:PORT/actions/live/increment
curl -s -D - -H 'Datastar-Request: true' -X POST http://127.0.0.1:PORT/actions/live/increment | head
curl -s http://127.0.0.1:PORT/actions/tabs/1
curl -s http://127.0.0.1:PORT/actions/signals/compose | grep '^event:'
curl -s 'http://127.0.0.1:PORT/actions/search/results?q=live'
curl -s http://127.0.0.1:PORT/health
```

`rocci run` stages `Html.roc`, `Datastar.roc`, and `datastar.js`. Do not commit
those files (root `.gitignore` ignores them outside `crates/**/runtime/`).

Duplicate exact keys in the `unique_keys` list passed from `init!` exit the
process with code 2 (`Exit(2)`).

## Constructors / wraps

| Helper | Wire wrap |
| --- | --- |
| `view!` | `200 text/html` |
| `get_fragment!` | one `Datastar.patch_elements` event |
| `fragment!` | one `Datastar.patch_elements` event |
| `command!` | empty SSE if `Datastar-Request`, else `204` |
| live poll | `Sse.unfold!` in the route; patch when `Html.render` changes |
| `events!` | author `List` of SSE events |
| prefix | `Rocci.prefix_remainder` then `fragment!` |
| `unfold!` | author `Sse.Stream` |

Also:

- `GET /health` in the catch-all when that path is not an exact arm
- 308 trailing-slash redirects for known GET pages via `Rocci.slash_alternate`
- exact `match` arms win over the prefix / slash catch-all

The live page authors `data-init=@get("/sse", [OpenWhenHidden(True)])` on
`<body>` (the lowering analogue). Runtime injection of that attribute is not
in the sibling module because it needs `Datastar` + HTML strings in `main.roc`.

## Packaging

`Rocci.roc` is an **app-local sibling module**. `rocci run` already stages
`Html.roc` / `Datastar.roc` into the app directory.

A Roc `package [Rocci] { pf: platform "…" }` is not a valid package header
(packages do not take a platform). `package [Rocci] {}` plus `import pf.Server`
is also invalid, because packages are not apps and have no `pf`. Even as a
sibling, `import pf.Server` currently crashes this compiler; keep `Rocci.roc`
platform-free.

Do not add a Cargo workspace member for this experiment.

## Out of bound

No `@method:role` changes, no custom Roc platform, no `Rocci.component` I/O,
no knowledge-record promotion to shipped.
