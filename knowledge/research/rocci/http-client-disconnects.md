---
type: Research Report
title: HTTP client disconnects never reach Roc until a complete request exists
description: "The beginner-facing incomplete-request log is Hyper HTTP/1.1 parse failure before handle_req. Body.ClientDisconnected and SSE abort are later phases; generated apps should treat them as expected client cancel, not 500s."
tags: [domain/rocci, domain/runtime, integration/datastar, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-09-09T09:09:00Z }
stale_after: 2026-12-09
authority: exploratory
owners: [human:nils]
sources:
  - id: http-diag
    resource: ../../../crates/rocci-platform/src/http_server.rs
    title: HTTP/1.1 incomplete_message diagnostic; request_body_error; SSE cancel
    author: process:git
    last_modified: 2026-09-09
  - id: request-body
    resource: ../../../crates/rocci-platform/src/request_body.rs
    title: Body pump maps PumpError::ClientDisconnected to BodyError
    author: process:git
    last_modified: 2026-09-03
  - id: server-roc
    resource: ../../../crates/rocci-platform/platform/Server.roc
    title: Server.Body.Err and WriteFileErr include ClientDisconnected
    author: process:git
    last_modified: 2026-09-03
  - id: datastar-roc
    resource: ../../../crates/rocci-platform/platform/Datastar.roc
    title: requestCancellation Auto Cleanup Disabled
    author: process:git
    last_modified: 2026-09-03
  - id: live-reload
    resource: ../../../crates/rocci-cli/src/dev_server/mod.rs
    title: Preview EventSource /__rocci/events reconnect then location.reload
    author: process:git
    last_modified: 2026-08-31
  - id: dispatch
    resource: ../../../crates/rocci-cli/src/dispatch/mod.rs
    title: Generated empty_sse and one-shot patch_html
    author: process:git
    last_modified: 2026-09-04
  - id: counter
    resource: ../../../examples/rocci/standalone/counter/Counter.rocci
    title: @post:fragment increment writes SQLite then returns a patch
    author: process:git
    last_modified: 2026-08-25
  - id: bws-sse
    resource: basic-webserver-sse-http.md
    title: Idle timeout, HTTP/1.1, empty SSE, noisy response abort
    author: process:cursor
    last_modified: 2026-09-09
  - id: cqrs
    resource: datastar-cqrs-action-responses.md
    title: requestCancellation versus live GET; empty SSE commands
    author: process:cursor
    last_modified: 2026-08-31
  - id: plan
    resource: ../../plans/rocci/http-client-disconnects.md
    title: Classify disconnects in rocci-platform; document three phases
    author: process:cursor
    last_modified: 2026-09-09
---

# HTTP client disconnects never reach Roc until a complete request exists

## Claim

The CLI line `Client disconnected before finishing an HTTP request` is a
**host parse failure** on plaintext HTTP/1.1. Hyper never finished the
inbound message, so `handle_req` did not run and the Roc application did not
see the request. Later disconnects (body read, response/SSE abort) are
different phases with different APIs. Treat all of them as expected client
cancel unless the app already committed a write.[^http-diag]

This is not a Rocci fork of basic-webserver. Product apps pin
`crates/rocci-platform`, which classifies incomplete requests and client-aborted
responses as beginner-facing disconnects. Idle-timeout hung bodies stay a
loud Hyper Body-stream failure. Related transport limits:
[basic-webserver SSE and HTTP](basic-webserver-sse-http.md).
Implementation: [classify HTTP client disconnects](/plans/rocci/http-client-disconnects.md).[^bws-sse][^plan]

## Three phases

```text
TCP bytes  ->  complete HTTP request  ->  Roc handler  ->  response / SSE
     |                 |                        |                  |
 incomplete_message    Body.ClientDisconnected   handler ran      Body-stream /
 (this log; no Roc)    if handler reads body         then client      client abort
                                                  aborts read
```

### 1. Incomplete request (the observed log)

`serve_connection` fails with `hyper::Error::is_incomplete_message()`. The
host prints the beginner-facing sentence and continues. The unit test
reproduces it by writing `GET / HTTP/1.1\r\nHost: localhost\r\n` (no
terminating blank line) and dropping the client.[^http-diag]

That covers **truncated headers** and **a declared body that never
finished** (`Content-Length` / chunked) **before** Hyper assembled a
`Request` for the service function. If Hyper already handed a request to
`handle_req`, this log is the wrong bucket; body errors go through
`request_body_error` instead.[^http-diag][^request-body]

Typical preview causes (any one is enough; the log does not name which):

- Browser or webview **cancels navigation** (new URL, Back, Cmd-K HTML
  swap, closing the tab) while a keep-alive connection had started the next
  request.
- Datastar **`requestCancellation: auto`**: a later `@post` to the same URL
  aborts the previous fetch. If abort hits while the POST is still being
  written, Hyper can see an incomplete message; if the POST already
  completed, the next line is more often a response/SSE abort, not this
  sentence.[^datastar-roc][^cqrs]
- Preview **live reload**: `EventSource("/__rocci/events")` reconnects on
  error, then `location.reload()` on `reload`. Reload tears down in-flight
  fetches and the live-reload SSE itself.[^live-reload]
- `rocdown view site` **two origins** (docs origin plus island Rocci
  origin). Leaving an embedded counter, rebuilding the site, or swapping
  the iframe aborts connections on the island port.
- Tools (`curl -N`, interrupted proxies) closing the socket mid-write.

Successful `@post:fragment` lines such as `counter increment count=2` mean
those requests **did** reach Roc and committed SQLite. A later incomplete
request is a **different connection**, not a rollback of those
increments.[^counter]

**Handle:** do nothing in the application. Do not retry. Do not map this
log to a 500. Operators can ignore it during preview navigation. Rocci
should not try to silence it without a platform logging-level change.[^http-diag][^bws-sse]

### 2. Complete request, incomplete body (Roc is running)

If headers finished and Roc starts, an inbound body abort becomes
`PumpError::ClientDisconnected` then `Server.Body.Err.ClientDisconnected` (or
the matching `WriteFileErr`).[^request-body][^server-roc]

**Handle in authored body readers:**

- Match `ClientDisconnected`, `Cancelled`, and `RequestFinished` as
  **stop**; do not persist a partial upload.
- Host-managed `write_file` that fails with `ClientDisconnected` before
  publish does not expose the destination; do not assume a file appeared.
- Generated `@post:fragment` / `@command` handlers that never read
  `request.body` will not see this tag. Their mutations run once the
  request is admitted.[^counter][^dispatch]

Do not convert `ClientDisconnected` into `InternalServerError`. The client
already left.

### 3. Complete request, aborted response or SSE

The handler already returned a body or `Sse.unfold!`. Closing the tab,
Datastar reconnect, or live reload RST the socket. The host prints the
beginner-facing response-abort sentence when the inner error is closed,
canceled, incomplete, broken pipe, or connection reset. Idle-timeout hung
bodies still use the loud `Could not serve` form. The SSE source `cancel`
path runs (including on drop); shutdown also cancels parked `Wait`.[^http-diag][^bws-sse]

**Handle:** generated live keepalives and empty-SSE commands already exist
so idle timeout and Safari 204 noise are separate issues. Authors should
not write extra cleanup in `@get:live` for tab close. Make writes
**idempotent** so a cancelled POST plus a completed retry does not double
apply if the first handler already ran.[^dispatch][^cqrs]

Default Datastar `requestCancellation: auto` aborts an in-flight request
to the **same URL**. Put long-lived `GET` live streams on a different path
than acting POSTs so a click does not cancel the stream.[^cqrs][^datastar-roc]

## What not to do

- Treat the incomplete-request line as evidence the increment handler
  failed. Stderr from `@post:fragment` already ran if you saw `counter
  increment`.[^counter]
- Add Roc `match` arms for a request the host never dispatched.
- Fork the platform solely to drop `eprintln` on `is_incomplete_message`.
- Raise global `response_idle_timeout` to hide SSE abort logs; that
  timeout is a hung-response detector, not a disconnect
  classifier.[^bws-sse]

## Preview incident shape (`rocdown view site`)

Observed: island Rocci listening, two increment logs, then the
incomplete-request sentence. That matches phase 1 after successful phase-2
writes: keep-alive or a new TCP connection closed before the next HTTP
message finished, most often because the webview navigated or reloaded.
Confirm by whether increment stderr advanced; if it did not, the cancelled
request never mutated SQLite.

## Logging and upstream (Phase 4)

Keep classified disconnects as one-line stderr. Do not add a debug/trace
channel that would hide them from `rocci run`. Do not special-case leftover
`Could not serve` in inspector stderr mapping: those are idle-timeout or
unknown Hyper failures. Copy the classifier into a later basic-webserver
vendor snapshot if it is still missing; do not open a `roc-lang` PR for
this. Product source of truth stays `crates/rocci-platform`.[^plan]

[^http-diag]: `http1_connection_diagnostic` on `is_incomplete_message`; `request_body_error`; SSE `cancel` / shutdown cancel test.
[^request-body]: Pump maps client closed / incomplete / canceled body frames to `BodyError::ClientDisconnected`.
[^server-roc]: `Server.Body.Err` and `WriteFileErr` include `ClientDisconnected`.
[^datastar-roc]: `RequestCancellation(Auto|Cleanup|Disabled)` emitted into Datastar options.
[^live-reload]: Injected `EventSource("/__rocci/events")`; `onerror` reconnect; `reload` event calls `location.reload()`.
[^dispatch]: `empty_sse!` and `patch_html!` after the handler returns; no body-error match in generated dispatch.
[^counter]: Increment writes SQLite then logs; disconnect after that log is not an uncommitted increment.
[^bws-sse]: Incomplete **responses** stay noisy; keepalives and empty SSE are workarounds, not disconnect handlers.
[^cqrs]: Same-URL POST cancel versus live GET path; empty SSE for Datastar commands.
[^plan]: Classify HTTP/1.1 client aborts in `rocci-platform`; keep eprintln; no inspector special-case; no roc-lang PR.
