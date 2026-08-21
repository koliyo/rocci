---
name: rocci-stack
description: >-
  Keep Roc, Datastar, Rocci, Rocdown, Markdown, HTML, and CSS in their
  idiomatic roles. Use when choosing one-shot Datastar patches versus a live
  SSE stream, where durable state lives, whether work belongs in grammar versus
  a template versus Markdown versus CSS, hybrid islands, server actions, or when
  an agent might treat Datastar as a client framework or put domain data in
  signals. Do not use for parser or ungram work (rocci-language-dev) or for
  ordinary isolated component authoring once the stack choice is clear
  (rocci-author).
---

# Rocci stack

Datastar is the **browser transport** for Rocci apps, not a client framework
and not a language feature. Roc owns types and effects. Rocci owns markup
components and handlers. Rocdown owns Markdown-first documents. HTML is the
shared render boundary. CSS is colocated and scoped.

Write **in** the stack with `$rocci-author`. Change **the languages** with
`$rocci-language-dev`. Facts live in `knowledge/` and `docs/`; this skill is
the composition rules.

## Establish context

1. Inspect `git status --short`. Preserve unrelated work.
2. Read `docs/concepts/rendering-model.rocdown`.
3. For durable state, `knowledge/decisions/server-owned-state.md`. For
   `@component`, `knowledge/decisions/pure-render-components.md`.
4. For shared-view SSE vs one-shot POST, start at
   `knowledge/research/datastar-cqrs-action-responses.md`. Generated `@live`
   and `json` commands are on `datastar-cqrs-action-responses` (not CI-complete
   on `main`). Snake remains the hand-written ceiling.
5. Prefer nearby `examples/` over inventing a new layering.

## Who owns what

| Layer | Owns | Does not own |
| --- | --- | --- |
| **Markdown** (Rocdown) | Prose, headings, links, fences, `:kind` | Client models, Datastar, grammar of `.rocci` |
| **Rocdown** | Catalog, routes, static HTML, islands splice | Interpreting templates in Rust to skip a theme |
| **Rocci** | `@component`, `@css`, `@on`, `@live`, `@context` | Parser/ungram; browser domain stores |
| **Roc** | Types, helpers, SQLite, `!` effects | Markup in interpolations; Datastar event framing |
| **HTML** | Documents (`GET`) and patch fragments (stable `id`) | A second copy of the domain in JS |
| **CSS** | Colocated `@css` / `@scope`; document chrome on `body` | Riding `<style>` on SSE patches |
| **Datastar** | `@get`/`@post`/…, morph by `id`, optional long-lived GET | Canonical count/todos/game state |

Hand off:

- Syntax, AST, lowering, diagnostics → `$rocci-language-dev`
- A page, component, or handler **after** the pattern is chosen → `$rocci-author`
- Knowledge records → `$manage-rocci-knowledge`

## Datastar is fundamental, not optional glue

Interactive Rocci UIs are **hypermedia**: the server renders HTML; Datastar
sends intent and morphs what comes back. Do not add a client domain model,
signals for the source of truth, or JSON that you expect to update `#counter`.

Shipped response types Datastar understands:

| Server sends | Datastar does |
| --- | --- |
| `text/event-stream` | 0–n patch-elements / patch-signals events |
| `text/html` | Morph top-level elements by `id` |
| `application/json` | **Patch signals**, not the DOM |
| `204` | Success, no morph |

`application/json` `{ "count": 5 }` sets `$count`. It does not morph
`<output>` inside `#counter`. Do not bind the visible count to `$count` to
“reuse” JSON. Signals are for ephemeral UI (open/closed, loading), not SQLite.

`Accept` cannot tell Datastar from `curl` (Datastar lists event-stream, html,
and json). Discriminate on `Datastar-Request: true`.

## Two update patterns (pick one per flow)

**Direct patch (shipped default).** POST returns a fragment; generated
dispatch wraps it in one `datastar-patch-elements` and closes. Only the acting
tab updates. Correct for the first-app counter, search, click-to-edit,
validation. Stable `id` on the fragment. Do not also stream the same `id`
without versions.

**CQRS / live (Datastar Tao; generated `@live` plus Snake).**
A long-lived `GET /sse` is the read channel. Authors write `@live` returning
Html; Rocci generates the poll unfold and injects `data-init` on `<body>` when
absent. POSTs marked `json` are commands: **204** for Datastar, JSON for API
clients. Do not also patch the same `id` from the command. Put an authored
`data-init=@get("/sse", [OpenWhenHidden(Bool.true)])` only when injection
cannot see a `<body>` (island fragments). `basic-webserver` polls
(`Wait` + `After`); it has no cross-request pub/sub. Copy Snake’s unfold only
when you need a custom ceiling the generator does not cover.

Do not treat “Datastar architecture” as “every POST is a broadcast bus.” Do
not infer live mode from “there is a POST.”

Examples: `examples/rocci/standalone/counter` is one-shot;
`examples/rocci/standalone/live-counter` and hybrid `examples/rocdown/counter`
are `@live` + `json`; Snake is authored CQRS.

## File and language fit

- **Reusable widgets:** `components/*.rocci` — pure `@component`, scoped CSS,
  `@fixture`. No `@on`.
- **HTTP app:** `pages/*.rocci` or app-root `.rocci` — `@context` / `@init` /
  `@on` / optional `@live`. GET returns `<html>`; unmarked mutations return a
  fragment; `json` mutations return a `Str` (204 vs JSON).
- **Docs / prose:** `.rocdown` — Markdown first. Executable `@` / `:` only at
  document root. Static `rocdown build` rejects `@use` and live handlers;
  islands are a separate live origin.
- **Ordinary Roc:** `*.roc` for types and effectful helpers. Import from
  `.rocci`. Do not put markup in Roc strings to avoid the template language.
- **Knowledge:** `knowledge/**/*.md` stays inert Markdown. No Rocdown, no
  Datastar.

CSS: authors write `class="card"`; lowering scopes it. Patch fragments should
not carry a sibling `<style>`. `html { ... }` is the wrong home for document
chrome (`body` or `:scope`).

Navigation: real `<a href>` and redirects. Do not invent client routing.

## Consistency checks

Before implementing, answer:

1. Is this a **language** change or a **use** of the language?
2. Is durable state staying on the server (SQLite / `@init`), with the
   browser holding only the morph target and ephemeral signals?
3. One-shot patch or live stream — not both on the same `id`?
4. If returning JSON, is the client an API/`curl`, not Datastar morph?
5. Is CSS colocated with the component that owns those classes?
6. Is prose Markdown (Rocdown) rather than HTML-in-Roc or JS templates?

## Do not

- Encode Datastar SSE policy, CQRS, or 204/JSON negotiation in the `.rocci`
  parser. That is dispatch/runtime (`rocci-cli`). `@live` and `json` are
  shipped; further declarations still need `$rocci-stack` and a plan.
- Mirror the domain in Datastar signals or a client store.
- Put `@component` I/O, logging, or request lifecycle in the render function.
- Use raw Markdown HTML; use `:note`, `:img`, a document-root tag, or
  `@render`.
- Copy Snake’s `Sse.unfold!` into ordinary apps; generated `@live` already
  emits the poll loop.

## Validate

Match the layer: `rocci-author` inspect/run for templates; `rocci-language-dev`
crate tests for grammar; `docs/guides/server-actions.rocdown` after action
behavior changes. Two-browser fan-out is not proven by “SQLite is shared.”
