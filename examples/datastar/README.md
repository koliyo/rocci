# Rocci Datastar gallery

Ports of official [Datastar examples](https://data-star.dev/examples/) that exercise Rocci control flow: `@if`, `@for`, `@match`, and `@let`. Datastar is the transport. The server still owns HTML.

Pinned together:

- Roc nightly **2026-08-08**
- `basic-webserver` **0.16.0**
- Datastar **1.0.2** (CLI cache; override with `[assets] datastar` in `rocci.toml`)

## Run

From the repository root, with `roc` and `cargo` on `PATH`:

```sh
cargo run -q -p rocci-cli -- run examples/datastar
```

This opens an embedded window on a free local TCP port and prints the URL. Pass `--no-window` to serve on [http://127.0.0.1:8000](http://127.0.0.1:8000) without a window (then open that URL yourself, or curl it). Override the port with `--port` or `ROC_BASIC_WEBSERVER_PORT`. SQLite state lives in `examples/datastar/gallery.db`. Set `DB_PATH` to use another file.

`rocci run` compiles each `*.rocci` module to a gitignored Roc type module, stages `Html.roc` / `Datastar.roc` from the CLI runtime, and copies the pinned Datastar JS into `examples/datastar/assets/`. This example uses an authored `main.roc` for custom HTTP routing.

Package a desktop app from this directory with [`rocci.toml`](rocci.toml), or from the repository root where [`rocci.toml`](../../rocci.toml) points here.

## Pages

| Path | Datastar original | Rocci features |
| --- | --- | --- |
| `/search` | [Active Search](https://data-star.dev/examples/active_search) | `@let`, `@if` empty, `@for` rows |
| `/edit` | [Click to Edit](https://data-star.dev/examples/click_to_edit) | `@match Viewing \| Editing` |
| `/todos` | [TodoMVC](https://data-star.dev/examples/todomvc) | `@match` filter empty copy, `@for` items, `@if` completed |
| `/tabs` | [Lazy Tabs](https://data-star.dev/examples/lazy_tabs) | `@for` tablist, `@if` selected |
| `/validate` | [Inline Validation](https://data-star.dev/examples/inline_validation) | `@match Empty \| Valid \| Invalid`, then Form vs SignedUp |

## Syntax traps this gallery hits

Per-row Datastar URLs use Rocci actions, so the URI is a Roc string:

```rocci
<button data-on:click=@delete("/actions/todos/${item.id}")>Delete</button>
```

A static quoted attribute `data-on:click="@delete('/actions/todos/${item.id}')"` would send the literal `${item.id}` to the browser. `@delete("/actions/todos/${item.id}")` interpolates on the server, then `Datastar.delete` quotes the result for Datastar.

Quoted Datastar objects also cannot use unquoted `{ ... }` attribute values. In Rocci, `{expr}` is a Roc interpolation, so client objects stay inside `"..."`.

TodoMVC adds items with a short `data-on:submit` expression rather than a large inline keydown program. Mixed Datastar/JS such as `$input.trim() && @patch('/actions/todos')` stays a quoted attribute. Anything bigger belongs in a JavaScript module.

## Smoke checks

With the server running (`--no-window` if you do not want an embedded window):

```sh
curl -s http://127.0.0.1:8000/health
# ok

curl -s http://127.0.0.1:8000/ | grep -E 'Active Search|Click to Edit|TodoMVC|Lazy Tabs|Inline Validation'

curl -s http://127.0.0.1:8000/search | grep -E 'data-bind:search|Carli'

curl -s http://127.0.0.1:8000/edit | grep -E 'id="contact"|First Name'

curl -s http://127.0.0.1:8000/todos | grep -E 'id="todo-main"|Learn Roc'

curl -s http://127.0.0.1:8000/tabs | grep -E 'role="tablist"|Tab 0'

curl -s http://127.0.0.1:8000/validate | grep -E 'id="signup"|data-bind:email'
```
