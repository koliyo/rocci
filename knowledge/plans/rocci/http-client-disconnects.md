---
type: Implementation Plan
title: Classify HTTP client disconnects as expected cancel, not server failure
description: "In rocci-platform, classify HTTP/1.1 incomplete requests and client-aborted responses as beginner-facing disconnects. Document the three phases for authors. Do not raise idle timeouts, generate fake handlers, or PR roc-lang unless asked."
tags: [domain/rocci, domain/runtime, integration/datastar, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-09-09T09:09:00Z }
stale_after: 2026-12-09
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../../research/rocci/http-client-disconnects.md
    title: Three-phase disconnect taxonomy
    author: process:cursor
    last_modified: 2026-09-09
  - id: bws-sse
    resource: ../../research/rocci/basic-webserver-sse-http.md
    title: Idle timeout and noisy Body-stream logs
    author: process:cursor
    last_modified: 2026-09-09
  - id: http-server
    resource: ../../../crates/rocci-platform/src/http_server.rs
    title: http1_connection_diagnostic and incomplete_message test
    author: process:git
    last_modified: 2026-09-09
  - id: request-body
    resource: ../../../crates/rocci-platform/src/request_body.rs
    title: PumpError::ClientDisconnected mapping
    author: process:git
    last_modified: 2026-09-03
  - id: serve-rs
    resource: ../../../crates/rocci-cli/src/serve.rs
    title: level_for_stderr_line for inspector Console
    author: process:git
    last_modified: 2026-08-25
  - id: standalone-doc
    resource: ../../../docs/applications/standalone.rocdown
    title: Public Counter standalone tutorial
    author: process:git
    last_modified: 2026-09-09
  - id: author-skill
    resource: ../../../.agents/skills/rocci-author/SKILL.md
    title: Rocci authoring skill
    author: process:git
    last_modified: 2026-09-09
  - id: platform-readme
    resource: ../../../crates/rocci-platform/README.md
    title: In-tree platform pin; vendored host snapshot
    author: process:git
    last_modified: 2026-09-09
  - id: cqrs
    resource: ../../research/rocci/datastar-cqrs-action-responses.md
    title: requestCancellation versus live GET path
    author: process:cursor
    last_modified: 2026-08-31
---

# Classify HTTP client disconnects as expected cancel, not server failure

Exploratory. Do not start a phase until the user asks. Research:
[HTTP client disconnects](/research/rocci/http-client-disconnects.md).[^research]

## Goal

Preview and `rocci run` treat client cancel as **expected transport
noise**: incomplete HTTP/1.1 requests and client-aborted responses print
the same class of beginner-facing line, not `Could not serve an HTTP/1.1
connection`. Public standalone docs and the author skill state the three
phases so operators do not debug SQLite after a navigation cancel.[^research][^http-server]

## Out of bound

- Raising `response_idle_timeout_ms` or treating `Sse.Wait` as idle
  progress (that is a hung-response detector).[^bws-sse]
- Generated `match` arms on `@post:fragment` / `@command` handlers that
  never read `request.body`.
- Changing empty-SSE versus 204, live keepalives, or
  `requestCancellation` defaults.[^cqrs]
- HTTP/2 or TLS on plaintext `rocci run`.
- A pull request to `roc-lang/basic-webserver` unless a later phase is
  explicitly started for that.
- Editing sibling `../roc-basic-webserver` in the same change as
  `crates/rocci-platform` unless the user names that checkout.
- Inspector Console HTTP ingest, `@log`, or new handler logging APIs.

## Constraints that do not move

1. **Incomplete requests never reach Roc.** Do not invent a Roc error
   value for a message Hyper did not finish parsing.[^research][^http-server]
2. **Work in `rocci-platform`.** Product apps pin this host. Do not
   fork 0.16.0 staging copies for diagnostics.[^platform-readme]
3. **Keep a beginner-facing sentence.** Do not delete the incomplete-request
   log without a human gate. Classify more client-abort errors into that
   class; do not make unknown Hyper failures quiet.
4. **Body `ClientDisconnected` stays typed.** Authors who read or sink a
   body still match `Server.Body.Err`; the host does not turn that into
   500.[^request-body]
5. **Default tests stay in-process.** Diagnostic tests use duplex I/O
   like `incomplete_http1_request_has_a_beginner_facing_diagnostic`, not
   a browser.[^http-server]

## Phase 1 — Classify HTTP/1.1 client abort in the host

Bound:

- Extend `http1_connection_diagnostic` (and the HTTP/2 eprintln paths
  only if they already print the same Body-stream noise on localhost)
  so **client closed / canceled / incomplete** connections share one
  beginner-facing class. Incomplete request keeps the current “not
  passed to the Roc application” clause.
- Client abort **after** a complete request (open SSE, cancelled fetch
  of a finished POST) must not use `Could not serve an HTTP/1.1
  connection: error from user's Body stream` when the inner error is
  closed, canceled, incomplete, broken pipe, or connection reset.
  Idle-timeout hung bodies stay a distinct diagnostic (do not relabel
  them as navigation cancel).[^bws-sse][^http-server]
- Add tests next to the existing incomplete-request test: (a) incomplete
  headers still maps to the request sentence; (b) a classified
  client-abort error does not contain `Could not serve`; (c) a
  non-disconnect Hyper error still uses the loud form.
- Do not change admission, Roc dispatch, or SSE keepalive timing.

**Exit:** `cargo test -p rocci-platform` and `cargo fmt --all -- --check`.

## Phase 2 — Public standalone troubleshooting

Bound:

- One short subsection on `docs/applications/standalone.rocdown` (or the
  streams/runtime page if that is where CLI output is already described):
  the incomplete-request line is navigation cancel; increment stderr means
  the write ran; do not treat the line as a 500.[^standalone-doc][^research]
- One sentence in `crates/rocci-platform/README.md` that host disconnect
  logs are classified in this crate, not by generated `main`.[^platform-readme]
- Do not paste the research taxonomy tables into the tutorial.

**Exit:** `okmate check knowledge --profile base`. Public doc change is
in the same commit as the sentence.

## Phase 3 — Author skill

Bound:

- In `.agents/skills/rocci-author/SKILL.md`, a short gotcha: do not match
  incomplete-request logs in handlers; when reading a body, treat
  `ClientDisconnected` / `Cancelled` as stop; keep mutation routes
  idempotent; live GET and acting POST stay on different paths.[^author-skill][^cqrs]
- Point at the public standalone sentence, not the knowledge research
  record, as the author-facing fact.

**Exit:** skill file updated; `okmate check knowledge --profile base`
if this phase also touches knowledge.

## Phase 4 — Review gate: quiet log and upstream

Bound: **human decision only.** Record the choice in the research or this
plan; do not implement until chosen.

**Recorded 2026-09-09 (durable product policy):**

1. **Keep the one-line `eprintln`.** Classified disconnects stay beginner-facing
   stderr. A debug/trace channel would hide the sentence from `rocci run` and
   preview, which violates keeping a human-readable cancel line. The host has
   no separate log framework; do not add one to quiet expected cancel.
2. **Do not special-case `Could not serve` in `level_for_stderr_line`.** After
   Phase 1 those strings are leftover Hyper/idle failures. Inspector already
   maps them to Info unless they look like Roc failure; demoting them further
   would hide hung-response detector noise that operators should still see.
   Inspector HTTP ingest stays out of bound.[^serve-rs]
3. **Keep the classifier in `rocci-platform`.** Product apps pin this host.
   Copy the diagnostic on a later vendor snapshot of basic-webserver if it is
   still missing there. No `roc-lang` PR in this phase.

**Exit:** written decision in this plan or the paired research. No code
required.

## Tests

Phase 1 is the required floor. Phases 2–3 are documentation. Phase 4 is
a review gate.

[^research]: Incomplete request vs body `ClientDisconnected` vs SSE/response abort.
[^bws-sse]: Do not retune idle timeout to hide disconnect logs.
[^http-server]: Current incomplete_message diagnostic and `Could not serve` fallback.
[^request-body]: Complete requests that abort mid-body already surface as `ClientDisconnected`.
[^serve-rs]: Inspector maps stderr; disconnect text is Info unless it matches failure heuristics.
[^standalone-doc]: Counter tutorial is where operators see island increment plus CLI.
[^author-skill]: Handler authoring, not platform grammar.
[^platform-readme]: Product host is `crates/rocci-platform`.
[^cqrs]: Same-URL POST cancel versus live GET; empty SSE commands already shipped.
