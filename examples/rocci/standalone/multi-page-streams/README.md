# Multi-page streams

Run the primary module; Rocci discovers the sibling page and shared stream modules:

```sh
cargo run -q -p rocci-cli -- run examples/rocci/standalone/multi-page-streams/Dashboard.rocci
```

The dashboard opens `/streams/dashboard` and `/streams/notifications`. The
admin page opens `/streams/admin` and `/streams/notifications`. Each stream is
one independent HTTP connection and one independent poll/render loop, so each
page deliberately stops at two streams. The dashboard stream returns two
stable-ID regions to demonstrate the preferred coarse coherence boundary.

`/streams/admin` checks `X-Rocci-Admin: demo` inside the authored Roc body.
The generated route makes that policy observable but does not provide it.
An unauthorized request fails before any admin HTML is returned.

The page-specific subscription lives on the unpatched `<body>` shell and uses
Datastar's default hidden-tab lifecycle. The shared notifications subscription
lives on a second stable shell and opts into `OpenWhenHidden(True)` so it stays
connected while the tab is hidden. These explicit shell attributes also prevent
module-local singleton injection from creating a duplicate subscription.

## Behavioral evidence

The `multi_page_streams_http_smoke` test builds this directory with the pinned
Roc compiler and exercises the generated binary. On 2026-08-22 on macOS Apple
Silicon, two simultaneous 650 ms samples each received one initial
`datastar-patch-elements` event plus at least one keepalive. The dashboard event
contained both `#dashboard-summary` and `#dashboard-activity`. Unauthorized
admin traffic contained no admin fragment; the demo header enabled only the
admin path. Unknown paths returned 404 without leaking another page's HTML.

An in-app browser run confirmed exactly two `data-init` subscriptions per
page, navigation cancellation of the previous requests, authorized admin
rendering, shared notifications, and the same patches in two browser tabs.
The browser kept controlled tabs in the visible lifecycle state, so the hidden
boundary is asserted at the emitted options: the page stream uses Datastar's
default close/reopen behavior, while notifications alone emits
`openWhenHidden: true`.

Generated polling remains one 100 ms render loop per open stream. The expected
idle cost is therefore linear: one page stream is about 10 polls/renders per
second; this two-stream example is about 20, with two HTTP connections. Prefer
one coarse stream that patches several IDs unless lifecycle or authorization
boundaries justify another connection.
