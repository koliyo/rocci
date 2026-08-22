---
type: Implementation Plan
title: Implement verb-first handler declarations
description: "Clean-cut Rocci routes to mandatory @method:role headers, including path-addressed live GET routes, while keeping commands representation-free and advanced Datastar events below the high-level grammar."
tags: [domain/rocci, domain/runtime, integration/datastar, concern/language-design, concern/developer-experience]
status: draft
generated: { by: process:cursor, at: 2026-08-22T10:20:00Z }
stale_after: 2026-11-22
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../research/verb-first-handler-declarations.md
    title: Research on verb-first handler declarations
    author: process:cursor
    last_modified: 2026-08-22
  - id: bounded-research
    resource: ../research/action-handler-syntax.md
    title: Research on bounding Rocci handlers to server-rendered UI
    author: process:cursor
    last_modified: 2026-08-22
  - id: prior-plan
    resource: handler-ui-boundary.md
    title: Earlier bounded handler UI implementation plan
    author: process:cursor
    last_modified: 2026-08-22
  - id: original-plan
    resource: action-handler-syntax.md
    title: Historical semantic-handler cutover plan
    author: process:cursor
    last_modified: 2026-08-21
  - id: template-ungram
    resource: ../../crates/rocci-template/Rocci.AST.ungram
    title: Current semantic handler AST specification
    author: process:git
    last_modified: 2026-08-21
  - id: template-sidecar
    resource: ../../crates/rocci-template/Rocci.AST.toml
    title: Rocci AST generation and inspection sidecar
    author: process:git
    last_modified: 2026-08-21
  - id: template-parser
    resource: ../../crates/rocci-template/src/parser.rs
    title: Current role-first handler parser and recovery
    author: process:git
    last_modified: 2026-08-21
  - id: template-validate
    resource: ../../crates/rocci-template/src/validate.rs
    title: Rocci semantic validation
    author: process:git
    last_modified: 2026-08-21
  - id: template-lower
    resource: ../../crates/rocci-template/src/lower.rs
    title: Current handler lowering and route metadata
    author: process:git
    last_modified: 2026-08-21
  - id: template-pprint
    resource: ../../crates/rocci-template/src/pprint.rs
    title: Rocci formatting and handler inspection
    author: process:git
    last_modified: 2026-08-21
  - id: dispatch
    resource: ../../crates/rocci-cli/src/dispatch.rs
    title: Current generated document, fragment, command, and live dispatch
    author: process:git
    last_modified: 2026-08-21
  - id: handler-contract
    resource: ../../crates/rocci-template/tests/handler_contract.rs
    title: Frozen shipped handler contract
    author: process:git
    last_modified: 2026-08-21
  - id: handler-syntax
    resource: ../../crates/rocci-template/tests/handler_syntax.rs
    title: Handler parser and diagnostic tests
    author: process:git
    last_modified: 2026-08-21
  - id: compile-tests
    resource: ../../crates/rocci-template/tests/compile.rs
    title: Rocci parser and lowering integration tests
    author: process:git
    last_modified: 2026-08-21
  - id: all-syntax
    resource: ../../test/AllSyntax.rocci
    title: Comprehensive Rocci syntax fixture
    author: process:git
    last_modified: 2026-08-21
  - id: template-readme
    resource: ../../crates/rocci-template/README.md
    title: Owning crate public contract
    author: process:git
    last_modified: 2026-08-21
  - id: server-reference
    resource: ../../docs/reference/language/server.rocdown
    title: Public server declaration reference
    author: process:git
    last_modified: 2026-08-21
  - id: update-ui
    resource: ../../docs/how-to/update-the-ui.rocdown
    title: Public update-the-UI guide
    author: process:git
    last_modified: 2026-08-21
  - id: rendering-doc
    resource: ../../docs/concepts/documents-fragments-commands-streams.rocdown
    title: Public document, fragment, command, and stream model
    author: process:git
    last_modified: 2026-08-21
  - id: custom-main
    resource: ../../examples/rocci/custom/datastar/main.roc
    title: Authored GET fragment routes and Datastar response ceiling
    author: process:git
    last_modified: 2026-08-20
  - id: datastar-roc
    resource: ../../crates/rocci-datastar/src/codegen/mod.rs
    title: Generated Roc Datastar helper
    author: process:git
    last_modified: 2026-08-17
  - id: datastar-sse
    resource: ../../crates/rocci-datastar/src/sse/events.rs
    title: Rust Datastar event framing
    author: process:git
    last_modified: 2026-08-17
  - id: datastar-actions
    resource: https://data-star.dev/reference/actions
    title: Datastar actions and accepted backend responses
    author: organization:star-federation
  - id: datastar-sse-reference
    resource: https://data-star.dev/reference/sse_events
    title: Datastar SSE event reference
    author: organization:star-federation
  - id: live-path-research
    resource: ../research/path-addressed-live-streams.md
    title: Research on path-addressed live streams
    author: process:cursor
    last_modified: 2026-08-22
  - id: live-path-plan
    resource: path-addressed-live-streams.md
    title: Implementation plan for path-addressed live streams
    author: process:cursor
    last_modified: 2026-08-22
---

# Implement verb-first handler declarations

## Purpose and authority

This plan converts ordinary `.rocci` routes from the shipped role-first forms
to mandatory verb-first, role-explicit headers. It implements the recommendation
in the companion research while preserving the bounded server-rendered UI
contract established by the earlier research.[^research][^bounded-research]

The earlier [handler UI boundary plan](handler-ui-boundary.md) left the
`@patch` versus `@fragment` spelling behind a Phase 0 gate. This plan resolves
that exploratory gate by making `patch` an HTTP method and `fragment` the
response role. It supersedes that plan as the proposed implementation sequence;
the earlier plan and the original four-noun cutover remain historical context,
not descriptions of the new proposal.[^prior-plan][^original-plan]

This is an exploratory implementation plan. Writing it does not approve the
syntax, start a phase, or change shipped behavior. Do not mark any phase
complete from local source edits alone; use the listed exit evidence and, for
the final phase, green CI and Knowledge workflows.

## Current disposition

Phase 0 was jointly approved by the maintainer on 2026-08-22 together with
Phase 0 of the path-addressed live plan. The approval freezes mandatory
verb-first `@method:role(path)`, `fragment` naming, the closed matrix below,
representation-free commands, plural `@get:live(path)`, module-local singleton
injection, explicit subscriptions for multiple local streams, app-wide stream
binding and collision errors, and a clean cut with no aliases or interim
`@live(path)` spelling. This is approved implementation direction, not shipped
behavior; later phase gates remain open.

## Goal

The complete high-level route surface is:

```rocci
@get:view("/") = |{ db }| {
    page(...)
}

@get:fragment("/search") = |{ db }, request| {
    searchResults(...)
}

@post:fragment("/actions/todo/add") = |{ db }, request| {
    todoList(...)
}

@patch:fragment("/actions/todo/42") = |{ db }, request| {
    todoRow(...)
}

@post:command("/actions/counter/increment") = |{ db }| {
    increment!(db)?
}

@get:live("/sse") = |{ db }| {
    sharedCounter(...)
}
```

The accepted route matrix is closed:

| Method | Role | Successful Roc value | Generated success behavior |
| --- | --- | --- | --- |
| GET | `view` | complete `Html` document | `text/html` document |
| GET | `fragment` | stable-ID `Html` fragment | one-shot element morph response |
| GET | `live` | stable-ID `Html` fragment | long-lived polling element morph response |
| POST/PUT/PATCH/DELETE | `fragment` | stable-ID `Html` fragment | one-shot element morph response |
| POST/PUT/PATCH/DELETE | `command` | `{}` | success without a direct morph or API representation |

The path-addressed live follow-up amends the original singleton exception:
`@get:live(path)` is plural across the generated app, while retaining the
shipped poll, keepalive, patch-elements, and unambiguous singleton `data-init`
behavior. Its app-level binding and injection details are owned by the linked
subplan.[^dispatch][^template-readme][^live-path-research][^live-path-plan]

The exact command success framing is frozen in Phase 0 after testing the pinned
Datastar and host combination. The intended semantics are fixed even if the
wire differs by caller: Datastar may retain empty SSE while an ordinary caller
receives 204, but neither receives command result JSON.

Phase 0 selected that fallback wire pair: successful Datastar requests retain
the proven `200 text/event-stream` response with zero events, while successful
ordinary callers receive `204 No Content`. Neither branch serializes the
handler result, and generated Roc constrains command success to `{}`.

## Out of bound

- Bare `@get(path)`, bare `@post(path)`, or any method-dependent default role
- Role-first aliases after the clean cut
- `@get:command`, mutation `:view` / `:live`, or additional HTTP methods without a new
  contract review
- `:json`, `:signals`, `:script`, `:redirect`, `:download`, `:response`,
  `:stream`, content-type, patch-mode, selector, or event-count modifiers
- A general response ADT or direct `Server.Outcome` return from ordinary
  high-level `.rocci` handlers
- API schema, authentication, authorization, versioning, or status-code design
- Durable domain state in Datastar signals
- Changes to live polling interval, keepalive, fan-out behavior, or advanced
  client lifecycle beyond the path-addressed subplan's injection rules
- Inferring method or response role from an opaque Roc body
- Adding compatibility aliases or a deprecation window
- Executing any phase merely because this plan has been written

Datastar's method, content-type, and SSE-event capabilities remain available
below this intentionally smaller language surface; they are not evidence for a
declaration cross-product.[^datastar-actions][^datastar-sse-reference]

## Constraints that do not move

1. **Both route axes are explicit.** Every ordinary route header spells one
   HTTP method and one semantic response role.[^research]
2. **The pair matrix is semantic validation.** Parsing may recognize a
   structurally valid pair, but validation rejects method-role combinations
   outside the table.[^template-validate]
3. **Roc bodies remain opaque.** The template parser does not inspect body text
   to infer `Html`, unit, JSON, or response types.[^template-parser]
4. **Transport stays below grammar.** The language records method and role;
   generated dispatch owns HTML/SSE/204 framing and Datastar request handling.
   [^template-lower][^dispatch]
5. **HTML is the high-level UI boundary.** Views, fragments, and live regions
   render HTML. Commands mutate without creating a browser domain model.
6. **GET fragments are idempotent by contract.** Rocci cannot prove that an
   arbitrary Roc effectful body is side-effect free.
7. **Live ownership remains exclusive.** A command does not also patch the ID
   owned by its `@get:live` read stream.
8. **Public routes remain inspectable.** Method, role, path, handler name, and
   source span survive AST, lowering, inspection, LSP, logs, and dispatch.
9. **No mixed syntax release.** Parser, lowerer, runtime, tooling, examples,
   and public docs land together.
10. **Recovery always advances.** Every malformed or unclosed header path
    guarantees monotonic cursor progress and preserves later declarations.

## Frozen proposed syntax

Before implementation the contract test should enumerate exactly these forms:

```text
@get:view(path)
@get:fragment(path)
@post:fragment(path)
@put:fragment(path)
@patch:fragment(path)
@delete:fragment(path)
@post:command(path)
@put:command(path)
@patch:command(path)
@delete:command(path)
@get:live(path)
```

Representative rejected forms include:

```text
@get(path)
@post(path)
@get:command(path)
@post:view(path)
@live
@live(path)
@post:live(path)
@post:json(path)
@get:signals(path)
@fragment:get(path)
@view(path)
@patch:patch(path)
@command:delete(path)
@on:get(path)
```

The last four are removal-diagnostic inputs after the clean cut, not aliases.
The old source should never produce a valid route node.
The current syntax and diagnostic suite is the baseline for proving that every
accepted form is deliberately replaced and every retained removal case still
recovers.[^handler-syntax]

## Clean-cut migration table

| Shipped source | New source | Diagnostic guidance |
| --- | --- | --- |
| `@view(path)` | `@get:view(path)` | GET documents require explicit method and role |
| `@patch(path)` | `@post:fragment(path)` | POST was implicit; `fragment` names the result |
| `@patch:put(path)` | `@put:fragment(path)` | move method before role |
| `@patch:patch(path)` | `@patch:fragment(path)` | first `patch` becomes the HTTP method |
| `@patch:delete(path)` | `@delete:fragment(path)` | move method before role |
| `@command(path)` | `@post:command(path)` | POST becomes explicit |
| `@command:put(path)` | `@put:command(path)` | move method before role |
| `@command:patch(path)` | `@patch:command(path)` | move method before role |
| `@command:delete(path)` | `@delete:command(path)` | move method before role |
| `@live` | `@get:live("/sse")` | path and GET role become explicit |

Diagnostics must preserve the original path and point at the smallest header
span. They may suggest a new header but must not attempt to rewrite opaque Roc
bodies automatically.

## Proposed internal model

Use one route-declaration family with typed semantic variants rather than one
generic bag of strings:

```text
RouteDecl = ViewDecl | FragmentDecl | CommandDecl | LiveDecl

ViewDecl     = method + path + params + body
FragmentDecl = method + path + params + body
CommandDecl  = method + path + params + body
LiveDecl     = GET + path + params + body
```

The parser reads `method` before `role`, then constructs the typed variant from
the role token. The method remains explicit for `ViewDecl` and `LiveDecl`;
validation proves that both are GET. This preserves role-specific AST traversal
while making the common source grammar uniform. Plural live metadata,
subscription injection, and app-level binding follow the dedicated stream
plan.[^live-path-plan]

Normalize every route to runtime metadata with exhaustive role information:

```text
RouteInfo {
    method,
    path,
    fn_name,
    respond: Document | Fragment | Command,
    span,
}
```

Do not infer `Document` from GET. `GET + Fragment` must reach the fragment
branch, and illegal pairings must have been rejected before lowering.
[^template-lower][^dispatch]

## Phase 0 — Approve the contract with complete examples

**Status:** Approved 2026-08-22 as part of the joint verb-first/live gate.

The maintainer approved the final syntax and clean cut in the combined program
request. The contract-only `handler_contract.rs` fixture records all eleven
accepted method-role pairs, representative rejected forms, complete source,
the injection table, and the linear polling cost model. The existing host was
probed through its actual origin: Datastar command success was empty SSE and
the ordinary branch was JSON before this cutover. Two concurrent `/sse`
connections, observed for 0.55 seconds on macOS Apple Silicon with the pinned
Roc nightly and Datastar 1.0.2, each received one initial multi-ID patch and
five idle keepalives from the 100 ms loop. This local observation supports the
coarse-stream recommendation but is not a universal performance claim.

No newcomer study was performed or claimed. Complete-example review found no
method/role inversion or `view`/`fragment` ambiguity, and implementation may
proceed under the maintainer's explicit approval.

**Bound**

- No production parser, AST, lowerer, runtime, example, or public-doc change.
- Contract fixtures may be proposed or updated only as non-shipping evidence.
- Use the pinned Datastar asset and actual standalone host for wire probes.

**Work**

1. Replace the current role-first contract matrix with the exact accepted and
   rejected forms above in a review fixture derived from
   `handler_contract.rs`.[^handler-contract]
2. Prepare two complete, behavior-equivalent source examples: one shipped
   role-first and one proposed verb-first. Include document navigation, GET
   search, POST validation fragment, HTTP PATCH fragment, command/live, and an
   illegal pair.
3. Run the research evaluation tasks with maintainers and at least one user who
   did not design the syntax. Record observations; do not invent successful
   newcomer evidence.
4. HTTP-smoke a no-content response and the current empty-SSE response through
   the pinned Datastar action path and the same origin used by the webview.
5. Freeze one command policy:
   - preferred if proven: 204 for both Datastar and ordinary callers;
   - fallback if required by the pinned stack: empty SSE for Datastar, 204 for
     ordinary callers.
6. Confirm the clean-cut migration and reject role-first compatibility aliases.
7. Record maintainer approval or stop without modifying production syntax.

**Exit**

- The maintainer approves the exact method-role matrix and clean cut.
- Complete-example evaluation does not reveal a systematic method/role
  inversion or `view`/`fragment` ambiguity.
- The command success framing is proven against the pinned browser/runtime and
  recorded without JSON representation.
- No shipped behavior has changed.

## Phase 1 — Change the AST, parser, validation, and recovery

**Bound**

- Owning crate: `crates/rocci-template`.
- Do not change generated HTTP response behavior until Phase 2.
- Do not hand-edit generated AST files.

**Work**

1. Update `Rocci.AST.ungram` to replace top-level `ViewDecl`, `PatchDecl`,
   `CommandDecl`, and pathless `LiveDecl` membership with the typed `RouteDecl`
   family described above; rename `PatchDecl` to `FragmentDecl` and make
   `LiveDecl` a path-addressed GET variant.[^template-ungram][^live-path-plan]
2. Update `Rocci.AST.toml` with item variants, inspect tags, omissions, or
   fallbacks for every generated production, then run the generator.
   [^template-sidecar]
3. Replace role-first top-level recognition with a route parser that recognizes
   only `get`, `post`, `put`, `patch`, and `delete`, requires `:`, parses one
   role, then scans path, optional params, and opaque body.
4. Preserve the distinction between top-level method declarations and
   Datastar action expressions inside attributes.
5. Validate the closed pairing matrix separately from structural parsing:
   - GET accepts `view`, `fragment`, and path-addressed `live`;
   - mutation methods accept `fragment` and `command`;
   - all other roles and pairs are rejected.
6. Preserve path literal checks, duplicate method/path detection, handler
   arity, leading docs, and source spans; replace fixed `/sse` reservation with
   app-level live-route collision validation from the stream subplan.
7. Add narrow diagnostics for missing colon, missing role, unknown method,
   unknown role, illegal pair, missing/unclosed path, missing body, stray
   selectors, and trailing response experiments.
8. Add exact removal diagnostics for every shipped header in the migration
   table and retained `@on` removal inputs. Old forms recover to the next
   top-level declaration but do not lower.
9. Add monotonic-progress cases with malformed headers before valid routes,
   components, CSS, ordinary Roc, multiline bodies, and EOF.
10. Regenerate owned AST code and inspect the diff rather than accepting it
    mechanically.

**Exit**

- Every accepted header produces the intended typed route node and exact span.
- Every illegal pair has one actionable diagnostic and preserves later nodes.
- Old role-first forms are rejected, not aliases.
- Parser loops terminate on malformed, multiline, and unclosed inputs.
- `cargo run -q -p rocci-ungram -- check` passes.
- Focused parser/validator tests and `cargo test -p rocci-template` pass before
  runtime behavior changes.

## Phase 2 — Normalize response roles and change generated dispatch

**Bound**

- Owning layers: `rocci-template` lowering and `rocci-cli` dispatch generation.
- Preserve handler function naming, context/request adaptation, path matching,
  error overlays, request logs, and live-stream behavior.

**Work**

1. Replace `RespondKind::Patch` with exhaustive finite-response `Document`,
   `Fragment`, and `Command` variants, and lower GET-live declarations into
   plural route-like `LiveInfo` metadata.[^template-lower][^live-path-plan]
2. Lower each typed route variant with its explicit method and response role.
3. Remove the generated dispatch shortcut that treats every GET as a document.
   Match method/path first, then dispatch exhaustively by response role.
   [^dispatch]
4. Generate GET fragments through the same one-shot element-patch path as
   mutation fragments; preserve stable-ID requirements and Datastar framing.
5. Constrain command success to `{}` in generated Roc. Remove generated command
   JSON encoder wrappers, `Encoding.Json` success branches, and the ordinary
   `application/json` success response.
6. Implement the Phase 0 command success framing. Preserve Datastar developer
   error overlays. Return a small ordinary HTTP error without describing it as
   a versioned JSON API contract.
7. Apply the path-addressed stream subplan while preserving current behavior
   per connection: authored paths and plural app binding change, while polling,
   changed-HTML patches, keepalives, and error overlays do not.[^live-path-plan]
8. Update generated-Roc snapshots and source-map segments for new headers and
   removed command adapter code.[^compile-tests]

**Exit**

- `@get:view` reaches document HTML.
- `@get:fragment` reaches one-shot element morph behavior, never document HTML.
- Every mutation fragment uses its authored HTTP method.
- Commands compile only with `{}` success and return no JSON representation.
- No dispatch branch infers response role from method.
- Focused lowering/generated-dispatch tests and
  `cargo test -p rocci-template -p rocci-cli` pass.

## Phase 3 — Formatter, inspect output, LSP, and source maps

**Bound**

- No new method-role pairs or runtime response types.
- Source, AST inspection, symbols, diagnostics, and formatting must expose the
  same method and role vocabulary.

**Work**

1. Format every accepted header canonically as lowercase
   `@method:role(path)` and prove two-pass idempotence.[^template-pprint]
2. Keep handler inspection fields explicit: normalized uppercase method, path,
   source kind, and semantic role. Replace the old `patch` role with
   `fragment`; report each path-addressed stream with role `live`.
3. Update declaration symbols so methods do not erase semantic role names.
   Decide one stable symbol label such as `POST /path — command` and snapshot
   it.
4. Add two-stage completions:
   - top-level `@` offers route methods plus non-route declarations;
   - after `@method:` offer only legal roles for that method.
5. Update hover and diagnostics with the value/result contract and canonical
   migration forms.
6. Ensure semantic highlighting distinguishes top-level method declarations
   from Datastar action calls in attributes and distinguishes the role suffix.
7. Update source-map tests for moved tokens, renamed fragment nodes, and
   generated handler bodies.
8. Replace handler coverage in `test/AllSyntax.rocci`; regenerate and review
   the matching AST/reference fixture through the supported generator path.
   [^all-syntax]

**Exit**

- Formatter output is idempotent for all accepted forms and recovery inputs.
- `inspect --ast` exposes correct methods, roles, paths, and spans.
- LSP completion never proposes an illegal method-role pair.
- Symbols, hover, highlighting, and removal diagnostics pass focused tests.
- `cargo test -p rocci-lsp` passes.

## Phase 4 — Convert examples and prove HTTP behavior

**Bound**

- Convert active examples in one clean cut; do not preserve two syntax styles.
- A live stream and a one-shot fragment must not own the same stable ID.
- General JSON and advanced Datastar responses remain in custom `main.roc`.

**Work**

1. Convert the standalone counter to `@get:view` plus `@post:fragment`; retain
   acting-tab one-shot behavior.
2. Convert live-counter to `@get:view`, `@post:command`, and
   `@get:live("/sse")`; return `{}` from commands and remove result records
   used only by generated JSON.
3. Convert the Rocdown counter island through the shared generated route path.
4. Update or add the handler-matrix example with every accepted route pair and
   one `@get:live("/sse")`. Give each fragment/live result an independent
   stable ID. The multi-page and plural stream example belongs to the stream
   subplan.[^live-path-plan]
5. Add generated standalone search and tabs examples based on the existing
   authored GET fragment routes.[^custom-main]
6. Add HTTP smoke coverage for method/path, status, content type, and body:
   - GET document;
   - GET fragment;
   - each mutation fragment method;
   - each command method with and without `Datastar-Request`;
   - generated live stream.
7. Verify a Datastar call site and its route share the leading method in every
   example.
8. Build representative examples through the pinned Roc compiler, not only
   Rust parser tests.

**Exit**

- Counter, live-counter, handler matrix, GET-fragment example, and Rocdown
  island build with the pinned Roc toolchain.
- HTTP smoke tests prove the complete matrix and command no-representation
  behavior.
- Browser testing proves GET and mutation fragments morph their stable IDs.
- Two live-counter tabs update through `@get:live("/sse")` after commands without command
  JSON or a competing direct patch.
- Repository search finds no active role-first syntax in examples or fixtures.

## Phase 5 — Add the low-level patch-signals ceiling

**Bound**

- Owner: generated Roc helper in `rocci-datastar`, with protocol parity tests.
- No `.rocci` route role, parser branch, or generated standalone response mode.
- No durable signal-backed domain example.

**Work**

1. Audit Rust patch-signals framing against the pinned Datastar version before
   exposing the same subset to Roc.[^datastar-sse]
2. Add a typed `Datastar.patch_signals` Roc event builder to the generated
   helper; include options such as `onlyIfMissing` only when supported by the
   pin and represented consistently.[^datastar-roc]
3. Add byte-for-byte or semantic parity fixtures between Rust and Roc builders
   for the supported subset.
4. Add an authored `main.roc` fixture that can compose element and signal
   events in one SSE response without manual framing.
5. Keep script execution, redirects, arbitrary event streams, and every patch
   option outside this phase.

**Exit**

- Authored Roc can emit a valid patch-signals event through a typed helper.
- Rust and Roc fixtures agree on framing.
- No new high-level route role or browser source-of-truth signal store exists.
- `cargo test -p rocci-datastar` and the focused authored-Roc fixture pass.

## Phase 6 — Update public contracts, skills, and inventories

**Bound**

- Documentation changes only after the implementation and real-build gates are
  green on the working branch.
- Historical knowledge may retain old syntax when clearly labeled historical.
- Generated `dist/` output remains derived.

**Work**

1. Rewrite the owning crate README route table and removal guidance.
   [^template-readme]
2. Update the public server declaration reference with grammar, legal matrix,
   value contracts, errors, and clean-cut rewrites.[^server-reference]
3. Rewrite the update-the-UI guide around call-site/server symmetry, GET
   fragments, mutation fragments, and representation-free commands.
   [^update-ui]
4. Update the document/fragment/command/live concept with path-addressed
   GET-live routes, coarse stream boundaries, and explicit multi-stream
   subscriptions.[^rendering-doc][^live-path-research]
5. Update tutorials, how-to pages, examples catalog prose, app documentation,
   template/stack/author/language skills, and active repository instructions.
6. Explain that JSON resources and mixed/custom responses belong in authored
   Roc servers, while patch-signals is a low-level transport helper.
7. Run a repository-wide inventory over `crates`, `examples`, `site`, `docs`,
   `test`, `.agents`, and active `knowledge`. Classify every old spelling as a
   removal test, historical record, or defect.
8. Build docs and inspect the server reference, server-actions guide, rendering
   concept, and affected example pages.

**Exit**

- Every accepted pair appears in the public reference and complete example.
- Every rejected near miss has one canonical rewrite.
- No active public page says POST is implicit or that high-level commands
  return ordinary-client JSON.
- Public docs distinguish HTTP PATCH, HTML fragments, Datastar element patches,
  signal patches, and live streams.
- Skills route custom API/event work to authored Roc rather than growing the
  grammar.

## Phase 7 — Integrated validation and release gate

**Bound**

- Use temporary output directories for builds and smoke fixtures.
- Failed builds must preserve previous valid output trees.
- Do not update phase status to complete before the required GitHub workflows
  succeed on the landed revision.

**Work**

1. Language and code generation:

   ```sh
   cargo run -q -p rocci-ungram -- check
   cargo test -p rocci-template
   cargo run -q -p rocci-cli -- inspect --ast test/AllSyntax.rocci
   ```

2. Runtime and tooling consumers:

   ```sh
   cargo test -p rocci-cli
   cargo test -p rocci-lsp
   cargo test -p rocci-datastar
   cargo test -p rocci-rocdown -p rocci-rocdown-cli
   ```

3. Formatting and workspace integration:

   ```sh
   cargo fmt --all -- --check
   cargo test --workspace
   ```

4. Build representative standalone and Rocdown examples through Roc with
   `ROCCI_REQUIRE_ROC=1` where the existing test gate supports it. Use fresh
   temporary output paths.
5. Run the complete HTTP smoke matrix through the same origin used by the
   webview and assert method, status, content type, and body/event framing.
6. Perform browser tests for acting-tab fragments and two-tab command/live
   ownership.
7. Build and inspect public docs:

   ```sh
   cargo run -q -p rocci-rocdown-cli -- build docs
   ```

8. Validate canonical knowledge and report lifecycle/provenance warnings
   separately from errors:

   ```sh
   cargo run -q -p rocci-okf -- check knowledge --profile rocci --format terminal
   ```

9. Run the final old-syntax and JSON-command inventory. Every occurrence must
   be an intentional removal test or clearly historical record.
10. After landing, require green CI and Knowledge workflows for the exact
    revision. Record run IDs before claiming phase or plan completion.

**Exit**

- Focused, package, consumer, workspace, docs, and OKF checks pass.
- Every accepted route parses, formats, inspects, lowers, compiles through Roc,
  starts, and answers the correct method with the correct response role.
- Every illegal pair and old header fails with an actionable diagnostic.
- Browser behavior proves document, fragment, command, and live ownership
  without command JSON or durable signal state.
- Required CI and Knowledge workflows are green on the landed revision.

## Expected ownership

| Change | Primary owner |
| --- | --- |
| Ungram, generated AST, parser, validation, lowering, diagnostics, source maps | `crates/rocci-template` |
| Generated response policy, command framing, standalone HTTP smoke | `crates/rocci-cli` |
| Low-level Datastar event metadata and Roc helper generation | `crates/rocci-datastar` |
| Editor completion, symbols, hover, highlighting | `crates/rocci-lsp` and `crates/rocci-rocdown-lsp` |
| Rocdown live-handler reuse and island consumer tests | `crates/rocci-rocdown` and `crates/rocci-rocdown-cli` |
| Public language and rendering contract | `crates/rocci-template/README.md` and `docs/` |
| Example migration and behavioral fixtures | `examples/` and `test/` |
| Rationale, phase evidence, and lifecycle | `knowledge/` |

## Roll-forward and rollback

Develop the phases on one branch but land the public AST/parser, lowering,
runtime, tooling, examples, and documentation as one clean cut. There is no
supported mixed source state.

Before release, rollback means reverting the complete cutover rather than
leaving aliases or a partially converted repository. After release, roll
forward on `@method:role`; do not restore hidden method defaults, role-first
aliases, generated command JSON, or MIME/event suffixes to repair an isolated
consumer.

If Phase 0 finds that ordinary JSON APIs are a first-class `.rocci`
requirement, stop this plan. Research a single typed low-level response
facility and its API lifecycle instead of adding `:json` to the matrix.
Path-selectable multi-stream lifecycle is now designed by the dedicated
subplan and must pass its own Phase 0 gate before implementation.
[^live-path-plan]

## Final approval gate

Implementation may start only after a maintainer approves:

- the exact mandatory `@method:role` syntax;
- `fragment` as the HTML fragment role;
- the closed method-role matrix;
- the role-first clean cut with no aliases;
- the representation-free command contract and tested wire framing;
- path-addressed `@get:live(path)` as the GET-only live role; and
- patch-signals remaining a low-level Roc helper.

Approval of this plan would approve a bounded UI language change, not a generic
HTTP or Datastar response framework.[^research]

[^research]: Companion research compares backend conventions, server-driven UI counterexamples, Datastar symmetry, syntax candidates, and the bounded matrix.
[^bounded-research]: Earlier research separates GET fragments, JSON resources, signal patches, commands, and live streams.
[^prior-plan]: Earlier follow-up plan retains the UI boundary but leaves source order and fragment spelling unresolved.
[^original-plan]: Historical clean cut from `@on` to the shipped role-first declarations.
[^template-ungram]: Current tree has separate role-first view, patch, command, and live declaration nodes.
[^template-sidecar]: Generated AST and inspect behavior require matching sidecar classifications.
[^template-parser]: Current hand-written parser owns top-level recognition, method suffixes, opaque bodies, recovery, and diagnostics.
[^template-validate]: Validation owns semantic route restrictions and duplicate-route checks.
[^template-lower]: Current lowering records insufficient response-role detail for GET fragments.
[^template-pprint]: Current formatter and handler inspection expose declaration kind and normalized route metadata.
[^dispatch]: Current dispatch infers document from GET and negotiates empty SSE versus JSON commands.
[^handler-contract]: Current frozen matrix records hidden method defaults and the `@patch:patch` naming problem.
[^handler-syntax]: Existing tests cover accepted suffixes, rejected GET forms, recovery, and removal diagnostics.
[^compile-tests]: Owning integration suite covers parser, lowering, generated Roc, and source maps.
[^all-syntax]: Comprehensive syntax fixture is the inspectable repository-wide language corpus.
[^template-readme]: Owning crate README defines current public standalone handler and live behavior.
[^server-reference]: Public language reference must change with the grammar.
[^update-ui]: Public guide owns common one-shot versus command/live authoring guidance.
[^rendering-doc]: Public concept owns the semantic distinction among documents, fragments, commands, and streams.
[^custom-main]: Existing authored server demonstrates GET fragments and the low-level response ceiling.
[^datastar-roc]: Generated Roc helper is the correct low-level place to expose supported signal-event framing.
[^datastar-sse]: Rust protocol layer provides framing evidence that must be reconciled with the pinned Datastar version.
[^datastar-actions]: Datastar methods and backend response types are orthogonal.
[^datastar-sse-reference]: Patch-elements and patch-signals are event types, not route roles.
[^live-path-research]: Follow-up finding that multi-page streams require authored paths, multiplicity, app-level binding, and coarse coherence boundaries.
[^live-path-plan]: Dedicated implementation sequence for plural live metadata, injection, dispatch, tooling, examples, measurements, and validation.
