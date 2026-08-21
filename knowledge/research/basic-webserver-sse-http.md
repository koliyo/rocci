---
type: Research Report
title: basic-webserver 0.16 SSE and HTTP limits for Rocci live streams
description: "Pinned basic-webserver 0.16 fails silent SSE Wait after a 30s response idle timeout, logs hyper Body errors without the inner detail, and serves browsers on HTTP/1.1 for plaintext rocci run. Rocci workarounds keepalives and empty SSE commands; it does not fork the platform."
tags: [domain/rocci, domain/runtime, integration/datastar, concern/architecture, concern/rendering]
status: draft
generated: { by: process:cursor, at: 2026-08-21T13:08:32Z }
stale_after: 2026-11-19
authority: exploratory
owners: [human:nils]
sources:
  - id: dispatch-rs
    resource: ../../crates/rocci-cli/src/dispatch.rs
    title: Generated empty_sse and live keepalive Emit
    author: process:git
    last_modified: 2026-08-21
  - id: bws-http-server
    resource: https://raw.githubusercontent.com/roc-lang/basic-webserver/0.16.0/src/http_server.rs
    title: basic-webserver 0.16 response idle timeout and HTTP/1.1 Body diagnostics
    author: organization:roc-lang
  - id: bws-transport
    resource: https://raw.githubusercontent.com/roc-lang/basic-webserver/0.16.0/src/server_transport.rs
    title: basic-webserver 0.16 HTTP/2 prior-knowledge preface detection
    author: organization:roc-lang
  - id: bws-server-roc
    resource: https://raw.githubusercontent.com/roc-lang/basic-webserver/0.16.0/platform/Server.roc
    title: default_response_idle_timeout_ms is 30_000
    author: organization:roc-lang
  - id: bws-sse-roc
    resource: https://raw.githubusercontent.com/roc-lang/basic-webserver/0.16.0/platform/Sse.roc
    title: Sse.Wait emits no framed bytes
    author: organization:roc-lang
  - id: ds-sdk-adr
    resource: https://github.com/starfederation/datastar/blob/v1.0.2/sdk/ADR.md
    title: Datastar SSE headers; Connection keep-alive is HTTP/1.1 only
    author: organization:star-federation
  - id: cqrs-research
    resource: datastar-cqrs-action-responses.md
    title: Generated CQRS empty SSE commands and live keepalives
    author: process:cursor
    last_modified: 2026-08-21
  - id: live-counter
    resource: ../../examples/rocci/standalone/live-counter/LiveCounter.rocci
    title: Live counter @live plus @command
    author: process:git
    last_modified: 2026-08-21
---

# basic-webserver 0.16 SSE and HTTP limits for Rocci live streams

## Claim

Pinned **basic-webserver 0.16** is a capable SSE host, but several of its
defaults and diagnostics interact badly with Datastar long-lived `GET /sse`
and short write commands. Rocci works around them in generated dispatch. It
does **not** fork or vendor the platform for these issues.[^dispatch-rs][^cqrs-research]

## Silent `Wait` versus response idle timeout

`Sse.unfold!` may return `Wait({ wake: After(ms) })`. That parks the host
timer; it does **not** write SSE framing to the socket.[^bws-sse-roc]

The host default `response_idle_timeout_ms` is **30_000**. When the response
body makes no progress for that long, `TrackedResponseBody` fails with
`response body made no progress before its deadline`. Hyper then logs
`Could not serve an HTTP/1.1 connection: error from user's Body stream` and
**does not print the inner `io::Error` string** in that diagnostic.[^bws-server-roc][^bws-http-server]

A generated `@live` loop that only `Wait`s while HTML is unchanged therefore
dies after ~30s of idle (or sooner if a poll stalls). Datastar reconnects;
Safari shows red `sse` rows. The acting `@command` can still succeed against
SQLite, so the UI looks fine while the CLI and inspector look broken.[^live-counter]

**Rocci workaround:** on the unchanged poll path, emit a non-Datastar
keepalive (`Sse.Event.data("")`, frames as `data: \n\n`). Datastar ignores
events whose names do not start with `datastar`. Bytes on the wire reset the
idle timer.[^dispatch-rs]

## Empty command bodies and inspector noise

Datastar accepts **204** as success with no morph. Safari Web Inspector often
shows blank bodies as “An error occurred trying to load the resource.” Rapid
clicks also abort the previous POST to the same URL under Datastar
`requestCancellation: auto`.

**Rocci workaround:** Datastar `@command` success returns **empty SSE**
(`Sse.unfold!(0, |_| Ok(End))`), matching Snake: HTTP 200
`text/event-stream`, zero events. Ordinary clients still get JSON.[^dispatch-rs][^cqrs-research]

## HTTP/1.1 on `rocci run` is expected

Datastar’s contract is fetch plus response encoding (`text/event-stream`,
HTML, JSON, or empty success). It does **not** require HTTP/2. The SDK ADR
special-cases `Connection: keep-alive` as **HTTP/1.1 only**.[^ds-sdk-adr]

`rocci run` serves plaintext `http://127.0.0.1:…`. Browsers negotiate HTTP/2
via TLS ALPN (`h2`), not cleartext `h2c`. basic-webserver accepts HTTP/2
**prior knowledge** (`PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n`) and has a separate
`serve_http2` path; browsers never send that preface on `http://`.[^bws-transport]

HTTP/2 would help many concurrent SSE streams later (multiplexing vs ~6
HTTP/1.1 connections per host). One live-counter tab is one stream plus short
POSTs. Forcing HTTP/2 or HTTP/3 on localhost without TLS is out of bound.
HTTP/3 is not in basic-webserver 0.16.

## Shared SQLite and blocking handlers

Each request (and each SSE transition) runs on a blocking handler path. A
long-lived `/sse` poll that reads SQLite on every wake shares one
`Sqlite.Db` with concurrent `@command` writes. Contention can stall a
transition; combined with a silent `Wait` policy that would also look like a
Body-stream error. Keepalives do not remove SQLite contention; they only keep
the transport alive when polls complete without HTML changes.[^live-counter]

## Client disconnect remains noisy

Closing a tab aborts an open SSE body. Hyper may still log a connection /
Body error. Rocci cannot silence that without a platform change. Incomplete
request heads already get a clearer message; incomplete **responses** do
not.[^bws-http-server]

## Out of bound for Rocci

- Forking or vendoring basic-webserver to change idle defaults or log text
- Teaching Safari Preview to render infinite SSE
- Dual-patching the same `id` from `@command` and `@live`

Upstream improvements worth tracking: treat long-lived SSE `Wait` as progress
for idle deadlines, or surface the inner Body error in the HTTP/1.1
diagnostic.

[^dispatch-rs]: Generated `empty_sse!` and live keepalive `Emit`.
[^bws-http-server]: Response idle Body error; HTTP/1.1 connection diagnostic.
[^bws-transport]: Prior-knowledge HTTP/2 preface detection.
[^bws-server-roc]: `default_response_idle_timeout_ms = 30_000`.
[^bws-sse-roc]: `Wait` advances without framed bytes.
[^ds-sdk-adr]: Datastar SSE headers; keep-alive is HTTP/1.1 only.
[^cqrs-research]: Empty SSE commands and live keepalives as shipped CQRS policy.
[^live-counter]: Live-counter `@live` plus `@command` against one SQLite handle.
