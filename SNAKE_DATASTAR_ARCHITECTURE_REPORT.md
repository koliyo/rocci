# Snake input and Datastar architecture

**Investigation date:** 2026-08-14  
**Scope:** [`examples/roc-snake/Snake.rocci`](examples/roc-snake/Snake.rocci), especially `keydown_handler`  
**Status:** Design report. Extended Rocci syntax is illustrative, not a committed specification.

## Executive summary

The Snake example is not fundamentally a bad Datastar use case, and rewriting it as a conventional SPA would not fix the problem visible in `keydown_handler`. The example combines two workloads:

1. authoritative multiplayer state and server-pushed UI, which fit Datastar well; and
2. low-latency keyboard input, which belongs in a small client-side input adapter.

The current handler is ugly because a string-valued HTML attribute has become an untyped miniature program. It performs key normalization, command routing, browser-default suppression, and transport. Rocci then hides that program inside an ordinary Roc `Str`, where neither Roc nor Rocci can format, validate, or navigate it.

The recommended production shape is therefore **Datastar plus a small client island**, not a full SPA:

- keep Roc as the authoritative game model;
- keep the long-lived Datastar `GET /sse` stream for multiplayer patches;
- move keyboard and touch/gamepad normalization into a tiny `snake-input.js` module or a Datastar custom action;
- send a typed direction command to one endpoint;
- keep DOM patches for this deliberately discrete 8 Hz demo, but use a canvas/client renderer and snapshots if the goal becomes smooth animation rather than demonstrating server-driven HTML.

For Rocci itself, add lightweight syntax for **named, multiline Datastar expressions** and typed attribute/action helpers. Do not grow a second Roc-like language that transpiles arbitrary client programs to JavaScript. Complex browser logic should remain an explicit client module.

## 1. What is wrong with the current handler

The current declaration is one long string:

```roc
keydown_handler = "(evt.key.startsWith('Arrow')&&evt.preventDefault());..."
```

and is installed with:

```rocci
<body data-init="@get('/sse')" data-on:keydown__window={keydown_handler}>
```

This is legal Datastar. `data-on` exposes `evt`, `__window` installs the listener on `window`, and Datastar expressions may call backend actions such as `@post`. The official documentation even shows conditional `keydown` expressions and `evt.preventDefault()` in attributes. See the [keydown how-to](https://data-star.dev/how_tos/bind_keydown_events_to_specific_keys), [attribute reference](https://data-star.dev/reference/attributes), and [actions reference](https://data-star.dev/reference/actions).

The issue is not correctness of the primitive; it is the loss of structure:

- The same mapping is expressed as four repeated Boolean clauses.
- JavaScript/Datastar code masquerades as a Roc string.
- The entire expression is one diagnostic and formatting unit.
- The query string is assembled into four separate endpoint identities.
- Keyboard policy and HTTP transport are coupled.
- Adding touch controls or a gamepad would duplicate the transport logic.
- `startsWith('Arrow')` prevents the browser default for every arrow key, whereas the command clauses recognize a separately maintained key set.

There is also a subtle transport concern. Datastar request cancellation is keyed by HTTP method and URL by default. Because each direction is a different URL, requests for different directions do not share one ordering or cancellation lane. Concurrent HTTP requests may arrive in a different order than the key events. For an 8 Hz Snake demo this is unlikely to be disastrous, but an input sequence should ideally carry a monotonically increasing sequence number or use an ordered channel if exact ordering matters.

## 2. The idiomatic Datastar baseline

If the behavior must remain entirely in markup, the expression should at least normalize once, prevent the default only for a recognized game key, and call one endpoint with a payload. In ordinary Datastar-flavored HTML, the conceptual result is:

```html
<body
  data-init="@get('/sse')"
  data-signals:_direction="''"
  data-on:keydown__window="
    $_direction = ({
      ArrowUp: 'up', w: 'up', W: 'up',
      ArrowDown: 'down', s: 'down', S: 'down',
      ArrowLeft: 'left', a: 'left', A: 'left',
      ArrowRight: 'right', d: 'right', D: 'right'
    })[evt.key];
    $_direction && (
      evt.preventDefault(),
      @post('/api/direction', {
        payload: {direction: $_direction}
      })
    )
  "
>
```

The `_direction` signal is local: Datastar excludes underscore-prefixed signals from backend requests by default. The action reference documents both this rule and the `payload` override. The backend should validate the command exactly as it does today.

This is substantially clearer than the current handler, but it is still near the practical limit for an attribute expression. It is an acceptable self-contained demo implementation, not the preferred reusable implementation.

The `__prevent` modifier is not quite appropriate here. It would prevent every `keydown` observed on `window`, including unrelated shortcuts and keys while focus is in a control. Prevention should happen only after the key is recognized. The handler should also ignore editable targets if the page later gains inputs.

## 3. Recommended production setup: a client input island

Use Datastar for subscription and rendering, and use a small module for input:

```rocci
<head>
    ...
    <script type="module" src="/assets/datastar.js"></script>
    <script type="module" src="/assets/snake-input.js"></script>
</head>
<body data-init="@get('/sse')">
    ...
</body>
```

The module should own only browser concerns:

```js
const directions = new Map([
  ['ArrowUp', 'up'], ['w', 'up'],
  ['ArrowDown', 'down'], ['s', 'down'],
  ['ArrowLeft', 'left'], ['a', 'left'],
  ['ArrowRight', 'right'], ['d', 'right'],
])

let sequence = 0

addEventListener('keydown', (event) => {
  if (event.target instanceof HTMLInputElement ||
      event.target instanceof HTMLTextAreaElement ||
      event.target?.isContentEditable) return

  const direction = directions.get(event.key) ?? directions.get(event.key.toLowerCase())
  if (!direction || event.repeat) return

  event.preventDefault()
  void fetch('/api/direction', {
    method: 'POST',
    credentials: 'same-origin',
    headers: {'content-type': 'application/json'},
    body: JSON.stringify({direction, sequence: ++sequence}),
    keepalive: true,
  })
})
```

This module is not an SPA. It has no application state, router, component tree, virtual DOM, or duplicate game model. It is a browser-device adapter: keyboard events in, direction commands out. Touch buttons and gamepad polling can call the same `sendDirection` function.

A custom Datastar action is another reasonable spelling:

```rocci
<body
    data-init="@get('/sse')"
    data-on:keydown__window="@steer(evt)"
>
```

Datastar officially supports custom action plugins, so `@steer` is the cleanest Datastar-native abstraction. However, the [custom plugin example](https://data-star.dev/examples/custom_plugin) says that plugin API documentation is still in progress. Rocci should not make that evolving API its only client-extension mechanism. A plain ES module is the stable baseline; a bundled custom action can be offered as sugar.

## 4. Proposed Rocci ergonomics

Rocci currently has only static string attributes and `{rocExpression}` attributes. That is sufficient for small Datastar calls, but it gives embedded client expressions no distinct syntax, language identity, or formatting boundary.

### 4.1 Near-term: honest multiline client literals

A minimal extension could introduce a `client` literal whose result is still an HTML attribute string:

```rocci
steer = client {
    direction = {
        ArrowUp: "up", w: "up", W: "up",
        ArrowDown: "down", s: "down", S: "down",
        ArrowLeft: "left", a: "left", A: "left",
        ArrowRight: "right", d: "right", D: "right",
    }[evt.key]

    direction && (
        evt.preventDefault(),
        @post("/api/direction", { payload: { direction } }),
    )
}

playPage = component |{ cells, info, marks }| {
    <body
        data-init={client { @get("/sse") }}
        data-on:keydown__window={steer}
    >
        ...
    </body>
}
```

Semantics should be intentionally modest:

- preserve the block as a Datastar expression rather than pretending it is Roc;
- normalize whitespace and HTML escaping;
- provide a distinct source-map/language region for highlighting and diagnostics;
- allow Roc interpolation only through an explicit escape such as `${roc: endpoint}`;
- never silently accept an ordinary Roc `Str` where a typed `ClientExpr` is expected by Rocci's Datastar sugar.

This improves readability, but it does not type-check JavaScript and should not be marketed as doing so.

### 4.2 Better for common cases: typed Datastar builders

Common one-line actions should avoid strings entirely:

```rocci
<body ds:init={Ds.get("/sse")}> ... </body>

<button ds:on:click={Ds.post("/api/counter/increment")}>
    Increment
</button>
```

These can lower directly to today's attributes:

```text
ds:init       -> data-init
ds:on:click   -> data-on:click
Ds.get(...)   -> @get(...)
Ds.post(...)  -> @post(...)
```

The main value is contextual validation: `ds:on:*` accepts `ClientExpr`, URI quoting is centralized, modifiers can be represented structurally, and users no longer hand-build action strings. A possible modifier syntax is:

```rocci
<div ds:on:keydown.window={steer}> ... </div>
```

Rocci can then diagnose unknown modifiers before HTML is generated.

### 4.3 Optional declarative keymap helper

Snake exposes a reusable pattern that can remain declarative without becoming a general client language:

```rocci
<body
    ds:init={Ds.get("/sse")}
    ds:on:keydown.window={Ds.keymap({
        "ArrowUp" | "w" | "W" => Ds.post("/api/direction", payload: { direction: "up" }),
        "ArrowDown" | "s" | "S" => Ds.post("/api/direction", payload: { direction: "down" }),
        "ArrowLeft" | "a" | "A" => Ds.post("/api/direction", payload: { direction: "left" }),
        "ArrowRight" | "d" | "D" => Ds.post("/api/direction", payload: { direction: "right" }),
    }, preventDefault: Matched)}
>
```

`Ds.keymap` would be a compile-time expression builder, not a retained runtime component. It is worth adding only if several applications need it. It should not delay the more general `ClientExpr` boundary.

### 4.4 What not to build

Do not add arbitrary `client fn`, client-side Roc state, lifecycle hooks, or a Roc-to-JavaScript compiler merely to clean up this example. That turns Rocci from a server-template compiler into a split-runtime framework and creates much harder questions about types, effects, source maps, dependencies, hydration, and versioning.

The useful language distinction is simple:

- Roc expressions execute on the server and produce HTML;
- Datastar expressions execute in the browser and should have an explicit `ClientExpr` type/syntax;
- nontrivial browser programs live in JavaScript modules.

## 5. Is Snake a bad Datastar demo?

### What the demo shows well

- One authoritative world can update multiple spectators and players.
- A single long-lived SSE request naturally carries server-driven changes.
- Stable patch boundaries (`#board`, `#minimap`, `#hud`) make the rendering contract easy to inspect.
- Join, leave, spectate, cookies, SQLite, and reconnect behavior exercise much more of a real server application than a counter.
- Snake's movement is discrete at 8 ticks per second, so it does not require frame-perfect local animation.

Datastar explicitly positions itself for real-time collaborative applications, and its backend guide treats long-lived SSE and HTML/signal patches as core behavior. The architecture is therefore aligned with the framework rather than an abuse of it. See the [Datastar homepage](https://data-star.dev/) and [backend requests guide](https://data-star.dev/guide/backend_requests).

### What the demo shows poorly

- It visually foregrounds a large raw client expression, making Rocci look less structured than it is.
- At every revision, the server renders and sends all 31 × 21 viewport cells plus the HUD and minimap, then the browser morphs them. That is acceptable as a stress test, but it is not an efficient game-rendering lesson.
- Simulation ticking is demand-driven by connected view loads rather than owned by a dedicated game loop, which is convenient for a demo but weakens timing semantics.
- HTTP requests are adequate for occasional turns but are not a general real-time input protocol.
- The example lacks touch controls, focus policy, input ordering, and client feedback under latency.

The right label is therefore **Datastar/Rocci multiplayer stress demo**, not “recommended architecture for browser games.” Its purpose should be stated explicitly in the README.

## 6. When an SPA or richer client becomes justified

A full client architecture becomes worthwhile when one or more of these are requirements:

- animation at 30–60 frames per second;
- client-side prediction and reconciliation;
- analog or high-frequency input;
- offline play;
- a large retained scene graph;
- WebGL/canvas rendering;
- substantial local menus, editors, or game state that should not round-trip to the server.

Even then, “SPA” is too broad a prescription. A better next stage for this application would be:

1. keep normal server-rendered navigation and lobby pages;
2. keep Datastar for connection state, scores, join/leave, and other HTML UI;
3. replace `#board` with a canvas island;
4. stream compact world snapshots or deltas to the island;
5. interpolate locally and send sequenced commands over an ordered channel.

That hybrid preserves the simple hypermedia application around the game while giving the hot rendering/input loop the client runtime it needs.

## 7. Recommendation

### For the current repository

1. Keep Snake as a Datastar example.
2. Rename/reframe it in the README as a multiplayer stress demo.
3. Move key handling to `snake-input.js` now; do not wait for new Rocci syntax.
4. Change the transport to one `POST /api/direction` endpoint with a validated JSON command and optional sequence number.
5. Add touch buttons that reuse the same command sender.
6. Document that full viewport morphing at 8 Hz is intentionally demonstrative, not a performance prescription.

### For Rocci

1. Introduce a distinct `ClientExpr` concept and a multiline `client { ... }` literal.
2. Add `ds:*` attribute sugar and typed `Ds.get`/`Ds.post` builders for common actions.
3. Preserve raw `data-*="..."` attributes as an escape hatch.
4. Consider `Ds.keymap` only after the same pattern appears in more than one example.
5. Keep arbitrary browser logic in explicit modules rather than inventing a client-side Roc runtime.

## Final verdict

The handler is ugly for a real reason, but the conclusion is not “Datastar failed; use an SPA.” The better conclusion is that **server-driven UI does not eliminate client-specific device code**. Datastar remains a strong fit for the authoritative multiplayer and patch-stream portions of the example. A tiny, explicit input island is enough to repair the architectural boundary, and a `ClientExpr`/`ds:*` extension would repair Rocci's ergonomics for the smaller expressions that properly remain in markup.
