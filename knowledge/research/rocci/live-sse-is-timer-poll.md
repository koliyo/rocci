---
type: Research Report
title: Generated live SSE is a timer poll, not write-triggered push
description: "HTTP SSE is a long-lived GET. Generated @get:live does not wake that GET when a command writes. Each connection re-runs the live handler on a 100ms host timer and diffs Html.render. A keepalive is a second clock (<30s idle timeout), not 100ms. Emitting an empty SSE event on every poll was a conflation, not a platform requirement."
tags: [domain/rocci, domain/runtime, integration/datastar, concern/architecture, concern/rendering]
status: draft
generated: { by: process:cursor, at: 2026-08-30T00:44:00Z }
stale_after: 2026-11-30
authority: exploratory
owners: [human:nils]
sources:
  - id: dispatch-rs
    resource: ../../../crates/rocci-cli/src/dispatch.rs
    title: Generated live unfold polls After 100ms and keepalives on a quiet count
    author: process:git
    last_modified: 2026-08-30
  - id: bws-sse-roc
    resource: https://raw.githubusercontent.com/roc-lang/basic-webserver/0.16.0/platform/Sse.roc
    title: Sse.unfold advances only when the host calls the next step
    author: organization:roc-lang
  - id: bws-http-server
    resource: https://raw.githubusercontent.com/roc-lang/basic-webserver/0.16.0/src/http_server.rs
    title: Host parks a Tokio timer for After(ms); no cross-stream notify
    author: organization:roc-lang
  - id: bws-server-roc
    resource: https://raw.githubusercontent.com/roc-lang/basic-webserver/0.16.0/platform/Server.roc
    title: default_response_idle_timeout_ms is 30_000
    author: organization:roc-lang
  - id: bws-limits
    resource: basic-webserver-sse-http.md
    title: Silent Wait dies after 30s without bytes on the wire
    author: process:cursor
    last_modified: 2026-08-21
  - id: cqrs
    resource: datastar-cqrs-action-responses.md
    title: Datastar SSE is per-request; fan-out is CQRS not one-shot POST
    author: process:cursor
    last_modified: 2026-08-21
  - id: path-plan
    resource: ../../plans/rocci/path-addressed-live-streams.md
    title: Platform pub/sub and write-triggered wake are out of bound
    author: process:cursor
    last_modified: 2026-08-30
  - id: runtime-doc
    resource: ../../../docs/reference/runtime.rocdown
    title: Public runtime page says poll stream and 100ms polling
    author: process:git
    last_modified: 2026-08-30
  - id: stack-skill
    resource: ../../../.agents/skills/rocci-stack/SKILL.md
    title: basic-webserver polls; it has no cross-request pub/sub
    author: process:git
    last_modified: 2026-08-30
  - id: live-counter
    resource: ../../../examples/rocci/standalone/live-counter/LiveCounter.rocci
    title: Increment is a representation-free command; live is a separate GET
    author: process:git
    last_modified: 2026-08-30
  - id: server-state
    resource: ../../decisions/server-owned-state.md
    title: Durable state stays on the server
    author: human:nils
    last_modified: 2026-08-16
  - id: style-incident
    resource: sse-patch-style-targets.md
    title: PatchElementsNoTargetsFound is a different live-stream defect
    author: process:cursor
    last_modified: 2026-08-30
  - id: sse-whatwg
    resource: https://html.spec.whatwg.org/multipage/server-sent-events.html
    title: SSE with no event field dispatches as type message
    author: organization:whatwg
  - id: datastar-roc
    resource: ../../../crates/rocci-cli/runtime/Datastar.roc
    title: Rocci patch events are named datastar-patch-elements
    author: process:git
    last_modified: 2026-08-30
---

# Generated live SSE is a timer poll, not write-triggered push

## Claim

**A 100ms keepalive is not required.** One hundred milliseconds is the
generated **poll** interval: how often each open live connection asks
Roc to render again. A **keepalive** only has to put bytes on the socket
before basic-webserver's **30s** response-idle timeout. Those are two
clocks. Shipping an empty SSE `message` on every poll mixed them up and
filled DevTools EventStream.[^dispatch-rs][^bws-limits][^bws-server-roc]

**SSE is a push *channel*. Generated Rocci live is not push *on
write*.** The browser holds one long GET. The server may write events
whenever it wants. Generated `@get:live` does not write when another
tab POSTs Increment. It writes when a **host timer** fires, the live
handler runs again, and `Html.render` bytes differ from last time.
Increment "shows up everywhere" because every tab's next poll reads the
same SQLite row — not because the command notified the streams.[^live-counter][^cqrs][^server-state]

This is implemented behavior of `rocci-cli` dispatch plus pinned
basic-webserver 0.16. It is not a Datastar requirement and not an
`@get:live` grammar feature.[^dispatch-rs][^bws-sse-roc][^stack-skill]

## What SSE is, and what the host can do

Datastar's live pattern is CQRS: a long-lived GET is the read channel;
POSTs are writes with no success morph. That GET is Server-Sent Events:
the response stays open and the server *may* push `datastar-patch-elements`
when the view changes.[^cqrs]

Pinned basic-webserver exposes that as `Sse.unfold!`. The application
returns one step: `Emit` (write a frame, then wait), `Wait` (write
nothing, then wait), or `End`. The **host** owns the wait. After
`After(ms)` it resets a Tokio timer and later calls
`roc_sse_advance_for_host` on **that connection's** source only. There
is no mailbox, broadcast, or "wake stream X from request Y."
`Wait` produces no bytes; if the body is silent for 30s the host
closes it as idle.[^bws-sse-roc][^bws-http-server][^bws-limits]

So the platform can push. It cannot, in 0.16, *schedule* a push from a
different HTTP request. Generated live therefore **simulates** a shared
stream by polling.

Path-addressed live planning already listed platform pub/sub and
write-triggered wake as out of bound. Custom `main.roc` can change
cadence or mix event types; it still sits on the same unfold timer
unless the platform grows a notify API.[^path-plan]

## The two clocks

| Clock | What it is for | Bound | What DevTools shows |
| --- | --- | --- | --- |
| Poll | Re-run the live handler; emit a patch if HTML bytes changed | Latency of shared-view updates (today 100ms) | `datastar-patch-elements` when the fragment changes |
| Keepalive | Prevent the 30s idle timeout while HTML is unchanged | Must be **less than 30s**; 15s is ample | Unnamed SSE `message` with empty data |

Poll cost scales with **open connections**, not with clicks. Two
browsers on home plus one on the example host are three independent
loops, each rendering `LiveSlice` ten times a second if the poll stays
at 100ms, even when nobody increments.[^dispatch-rs][^runtime-doc]

Keepalive cost should be one empty frame per connection per tens of
seconds. Until the quiet-`Wait` change in this checkout, generated
dispatch **emitted the keepalive on every poll** (`Sse.Event.data("")`
then `After(100)`). Datastar ignores events whose names do not start
with `datastar`. Chrome EventStream still lists them. That is the
landing-page "spam" of empty `message` rows at ~100ms. It is not
`PatchElementsNoTargetsFound` (untagged `style` siblings on real
patches).[^dispatch-rs][^style-incident][^bws-limits]

This checkout's generated arm `Wait`s on quiet polls and emits a
keepalive every 150 quiet ticks (15s). Poll stays 100ms so a click still
appears on the next timer fire. Staging still runs the old every-poll
keepalive until that dispatcher is promoted.[^dispatch-rs]

## Why that shows up in the browser

The poll loop is server-side. The **client still owns the socket.**
`data-init=@get("/sse")` is an ordinary long-lived GET. Anything the
server `Emit`s is written on that response body. Chrome's Network
EventStream panel is a dump of those frames, not a Datastar debug
view.[^runtime-doc][^dispatch-rs]

`Sse.Event.data("")` frames as `data: \n\n` — a valid SSE event with
**no** `event:` name. The HTML EventSource rule is: missing name means
type `message`. Empty `data` means the Data column is blank. That is
exactly the spam row.[^bws-sse-roc][^sse-whatwg]

`Wait` never reaches the browser. Only `Emit` does. The old generated
arm used `Emit` on every 100ms poll, so the client saw 10 empty
`message` events per second even though the page did not morph.
Datastar only applies events named `datastar-…`
(`datastar-patch-elements`, `datastar-patch-signals`). An unnamed
keepalive is ignored by the library and still listed by DevTools.
The UI can look fine while the EventStream tab is full.[^datastar-roc][^bws-limits]

## Why 100ms poll exists at all

There is no write-triggered wake, so the only way tab B sees tab A's
Increment is for B's stream to **re-read** soon. 100ms is a latency
choice (at most one tenth of a second after the other tab's SQLite
commit), not a keepalive choice and not an SSE protocol constant. A
slower poll (250ms, 1s) would be a product tradeoff: fewer renders, more
visible lag. A faster poll would be more CPU for the same reason.
Neither changes the model.[^dispatch-rs][^live-counter]

`Html.render` equality is the change detector. The live arm compares the
**whole** `LiveSlice` (count + feed). A second-resolution "N seconds
ago" string changes those bytes every second after a click, so both
windows get a real `datastar-patch-elements` (first node is `#counter`)
with no further Increment. That is render identity, not a broken
keepalive. The 2026-08-30 EventStream after a quiet-until-click session
was this: one patch on Increment, then 1 Hz patches while any feed row
was in the seconds bucket, then a later unnamed `message` keepalive
once the HTML stopped changing.[^live-counter]

The demo now buckets `< 60s` as "just now", then minutes. The stream
stays quiet for a minute after a click. A live fragment that prints
wall-clock or per-second relative time will always be this chatty; do
not put a ticking clock in the morph HTML if the channel should stay
quiet.[^live-counter]

## What true push would take

A command handler would need to signal "revision N changed" to every
parked live source sharing that view. basic-webserver 0.16 has no such
hook. Options, none of them generated `@get:live` today:

- Platform work: a notify/mailbox that `Wait` can observe besides the
  timer.
- Process-local side channel outside the platform (custom `main.roc`,
  still not portable).
- Accept poll, and keep the two clocks honest (do not keepalive at poll
  rate).

Do not treat Datastar or `@get:live` syntax as the missing notify.
Do not put pub/sub in the `.rocci` parser.[^stack-skill][^path-plan]

## Is the 30s idle timeout too short?

**No, as a general HTTP default. Yes, as a description of how long an
SSE view may be quiet.** Those are different jobs.[^bws-server-roc]

`response_idle_ms` is “maximum time without **outbound transport
progress**.” It applies to every response, not only live GET. Sibling
defaults match that story: 30s between request-body chunks, 60s between
keep-alive requests, 5s waiting for a Roc handler. The clock starts
when the response starts writing and resets when the socket makes
progress.[^bws-server-roc][^bws-limits]

For a normal request that is sound. If a handler has opened a body and
writes nothing for 30s, something is wedged (deadlock, infinite `Wait`,
half-open TCP). Without a bound, leaked connections accumulate. 30s is
a conventional “this is stuck” window.

For SSE, silence is a legal state. “No events yet” is not failure. The
protocol even has comment pings for liveness. A 30s “must emit or die”
rule means every quiet live GET is *by definition* dying unless the app
writes dummy bytes. That is a category mismatch: the timeout treats
idle as a bug; SSE treats idle as “nothing to say.”

So:

- Too short as “how long a live view may be quiet”? Yes, if you wanted
  true push with no client-visible traffic. Minutes of quiet should be
  normal.
- Too short as “detect a hung response”? No. 30s of zero outbound
  progress is still a reasonable wedged-stream detector.
- The 100ms client spam was **not** implied by 30s. Beating that bound
  needs a frame every ~15–25s, not every poll.[^dispatch-rs]

Raising the platform default to minutes would be more SSE-friendly and
worse for stuck file or HTML responses. A cleaner platform split (not
shipped) is a longer idle for `text/event-stream` only, or a notify so
`Wait` is not the only wake. Rocci can raise `response_idle_ms` via
`Server.with_timeouts`; that is optional if keepalives stay honest.
Do not fork the platform for this.[^bws-limits]

## Disposition

Exploratory investigation of shipped (and this-checkout) dispatch.
Public runtime docs should say "timer poll" and "separate keepalive
clock" in the same sentence as "100ms." This record is not a plan to
build write-triggered wake.

[^dispatch-rs]: Generated live unfold; quiet `Wait` versus keepalive `Emit`.
[^bws-sse-roc]: `unfold!` steps are Emit, Wait, or End; `Event.data` writes `data:` lines.
[^bws-http-server]: Host parks a Tokio timer per connection; no cross-request wake.
[^bws-server-roc]: Default response idle timeout is 30 seconds.
[^bws-limits]: Silent `Wait` dies at that timeout; keepalives exist to write bytes.
[^cqrs]: Datastar live is a long GET; commands are other requests.
[^path-plan]: Pub/sub and write-triggered wake stay out of generated live.
[^runtime-doc]: Public runtime page already says the live route is a poll stream.
[^stack-skill]: Transport policy stays out of the parser; the host has no pub/sub.
[^live-counter]: Increment writes SQLite and returns no representation; `/sse` is separate.
[^server-state]: Durable count stays on the server; tabs reconcile by re-reading.
[^style-incident]: Style-sibling patch errors are a different EventStream failure.
[^sse-whatwg]: An SSE event with no `event:` field is dispatched as `message`.
[^datastar-roc]: Rocci patch helper names the event `datastar-patch-elements`.
