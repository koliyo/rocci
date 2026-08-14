# Rocci Datastar gallery

Ports of official [Datastar examples](https://data-star.dev/examples/) that exercise Rocci control flow: `@if`, `@for`, `@match`, and `@let`. Datastar is the transport. The server still owns HTML.

Pinned together:

- Roc nightly **2026-08-08**
- `basic-webserver` **0.16.0**
- Datastar **1.0.2** from `assets/datastar.js`

## Run

From the repository root, with `roc` and `cargo` on `PATH`:

```sh
./scripts/run-roc-datastar.sh
```

Then open [http://127.0.0.1:8000](http://127.0.0.1:8000). Override the port with `ROC_BASIC_WEBSERVER_PORT`. SQLite state lives in `examples/roc-datastar/gallery.db`. Set `DB_PATH` to use another file.

The script copies `datastar.js` into `examples/roc-datastar/assets/` and runs `rocci run`, which compiles each `*.rocci` module to a gitignored Roc type module.

## Pages

| Path | Datastar original | Rocci features |
| --- | --- | --- |
| `/search` | [Active Search](https://data-star.dev/examples/active_search) | `@let`, `@if` empty, `@for` rows |
| `/edit` | [Click to Edit](https://data-star.dev/examples/click_to_edit) | `@match Viewing \| Editing` |
| `/todos` | [TodoMVC](https://data-star.dev/examples/todomvc) | `@match` filter empty copy, `@for` items, `@if` completed |
| `/tabs` | [Lazy Tabs](https://data-star.dev/examples/lazy_tabs) | `@for` tablist, `@if` selected |
| `/validate` | [Inline Validation](https://data-star.dev/examples/inline_validation) | `@match Empty \| Valid \| Invalid`, then Form vs SignedUp |

## Syntax traps this gallery hits

Per-row Datastar URLs must be Roc expressions:

```rocci
<button data-on:click={"@delete('/todos/${item.id}')"}>Delete</button>
```

A static attribute `data-on:click="@delete('/todos/${item.id}')"` would send the literal `${item.id}` to the browser. TodoMVC and Lazy Tabs use the Roc form on purpose.

Quoted Datastar objects also cannot use unquoted `{ ... }` attribute values. In Rocci, `{expr}` is a Roc interpolation, so client objects stay inside `"..."`.

TodoMVC adds items with a short `data-on:submit` expression rather than a large inline keydown program. Anything bigger than that belongs in a JavaScript module, not a Roc string.

## Smoke checks

With the server running:

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
