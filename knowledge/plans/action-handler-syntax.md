---
type: Implementation Plan
title: Semantic view, patch, command, and live handlers
description: "Replace @on with @view, @patch, and @command beside @live; default mutations to POST; encode command result data with Roc; convert every existing source without compatibility aliases; and build a complete syntax matrix with the pinned Roc compiler."
tags: [domain/rocci, domain/runtime, integration/datastar, concern/language-design, concern/developer-experience]
status: draft
generated: { by: process:cursor, at: 2026-08-22T09:13:16Z }
stale_after: 2026-11-22
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../research/action-handler-syntax.md
    title: Semantic handler syntax and response architecture research
    author: process:cursor
    last_modified: 2026-08-21
  - id: ungram
    resource: ../../crates/rocci-template/Rocci.AST.ungram
    title: Current Rocci declaration AST
    author: process:git
    last_modified: 2026-08-21
  - id: parser
    resource: ../../crates/rocci-template/src/parser.rs
    title: Current declaration parser and recovery
    author: process:git
    last_modified: 2026-08-21
  - id: validate
    resource: ../../crates/rocci-template/src/validate.rs
    title: Current route and response validation
    author: process:git
    last_modified: 2026-08-21
  - id: lower
    resource: ../../crates/rocci-template/src/lower.rs
    title: Current handler lowering and RouteInfo metadata
    author: process:git
    last_modified: 2026-08-21
  - id: pprint
    resource: ../../crates/rocci-template/src/pprint.rs
    title: Rocci formatter for declarations
    author: process:git
    last_modified: 2026-08-21
  - id: compile-tests
    resource: ../../crates/rocci-template/tests/compile.rs
    title: Handler parsing, lowering, diagnostics, and Roc compilation tests
    author: process:git
    last_modified: 2026-08-21
  - id: dispatch
    resource: ../../crates/rocci-cli/src/dispatch.rs
    title: Generated standalone HTTP dispatch
    author: process:git
    last_modified: 2026-08-21
  - id: lsp
    resource: ../../crates/rocci-lsp/src/analysis.rs
    title: Rocci symbols and diagnostics analysis
    author: process:git
    last_modified: 2026-08-21
  - id: all-syntax
    resource: ../../test/AllSyntax.rocci
    title: Rocci all-syntax fixture
    author: process:git
    last_modified: 2026-08-21
  - id: counter
    resource: ../../examples/rocci/standalone/counter/Counter.rocci
    title: One-shot patch counter
    author: process:git
    last_modified: 2026-08-21
  - id: live-counter
    resource: ../../examples/rocci/standalone/live-counter/LiveCounter.rocci
    title: Live command and stream counter
    author: process:git
    last_modified: 2026-08-21
  - id: hybrid-counter
    resource: ../../examples/rocdown/counter/index.rocdown
    title: Rocdown live counter island
    author: process:git
    last_modified: 2026-08-21
  - id: template-readme
    resource: ../../crates/rocci-template/README.md
    title: Public template and handler contract
    author: process:git
    last_modified: 2026-08-21
  - id: server-actions
    resource: ../../docs/guides/server-actions.rocdown
    title: Public server-actions guide
    author: process:git
    last_modified: 2026-08-21
  - id: roc-json
    resource: https://www.roc-lang.org/docs/main/
    title: Roc standard-library JSON encoding API
    author: organization:roc-lang
  - id: follow-up
    resource: handler-ui-boundary.md
    title: Follow-up plan for a bounded Rocci UI handler surface
    author: process:cursor
    last_modified: 2026-08-22
---

# Semantic view, patch, command, and live handlers

## Current disposition

The clean cut from `@on` to `@view`, `@patch`, `@command`, and `@live` is
implemented in the current tree. This record preserves the original cutover
design and phase structure; it is no longer the active recommendation for the
command/API boundary. The [handler UI boundary follow-up](handler-ui-boundary.md)
reopens negotiated command JSON, adds the missing GET fragment case, and keeps
patch-signals below declaration syntax.[^follow-up]

## Purpose and authority

The research recommends four semantic declarations—`@view`, `@patch`,
`@command`, and `@live`—instead of a single `@action` with a response selector.
The distinction is architectural: patches return `Html` to the acting request;
commands return data, produce 204 for Datastar, and produce encoded JSON for an
ordinary HTTP client.[^research]

This is an exploratory implementation plan. Writing it does not approve or
ship the language change. Each phase has its own exit gate; no phase should be
recorded complete from source edits alone.

## Goal

An author can write:

```rocci
@view("/") = |{ db }| { page(...) }

@patch("/actions/todo/add") = |{ db }, request| { todoList(...) }

@command:delete("/actions/todo/42") = |{ db }| {
    remaining = delete_todo!(db, 42)?
    { remaining }
}

@live = |{ db }| { liveTodoList(...) }
```

with these stable rules:

| Declaration | Default method | Allowed override | Successful Roc value | Generated success response |
| --- | --- | --- | --- | --- |
| `@view(path)` | GET | none | `Html` | document HTML |
| `@patch(path)` | POST | PUT, PATCH, DELETE | `Html` | one-shot patch-elements SSE |
| `@command(path)` | POST | PUT, PATCH, DELETE | JSON-encodable data | Datastar 204; otherwise encoded JSON |
| `@live` | generated GET `/sse` | none | `Html` | long-lived patch-elements SSE |

POST has one canonical spelling: it is omitted. `@patch:post` and
`@command:post` are rejected with a fix. GET is rejected on patch and command;
`@view` is the supported GET document contract and `@live` owns the generated
stream. A future non-document GET resource requires a new explicit contract;
the cutover does not retain `@on:get` as an escape hatch. The words describe
handler roles, while Datastar request attributes continue to express the actual
HTTP verb.[^research]

## Decision: four nouns, with one naming gate

Choose four nouns over the bracketed alternatives:

| Candidate | Decision | Reason |
| --- | --- | --- |
| `@action[patch](path)` / `@action[json](path)` | Do not choose | Locally explicit, but hides the central patch-versus-command architecture inside punctuation. |
| `@action:delete[patch](path)` | Best bracket order if reconsidered | Reads declaration, method, response, path and preserves the current method-suffix position. |
| `@action[patch]:delete(path)` | Reject | Reverses modifier order and delays the method until after response policy. |
| `@action(path) -> patch` | Reject | Resembles a Roc return annotation although `patch` is host response policy, not the full handler type. |
| `@view` / `@patch` / `@command` / `@live` | Choose | Each word maps to one expected value and one generated response policy. |

There is one explicit naming risk: `@patch` is a response role while PATCH is
also an HTTP method. `@patch:patch(path)` is legal and means “HTTP PATCH whose
success value is an HTML patch.” Keep this honest doubled spelling for the
implementation trial. If complete-example testing shows repeated method
confusion, substitute `@fragment` everywhere before declaring the syntax
stable. Do not change behavior to paper over the name.[^research]

## Out of bound

- Changing `@live` polling, SSE fan-out, element morph policy, or automatic
  `data-init` injection.
- Putting durable domain data in Datastar signals or making command JSON morph
  server-rendered HTML.
- Generating opaque action URLs, route names, form schemas, authentication, or
  authorization policy.
- A general response ADT, redirects, downloads, custom status values, or
  unconditional JSON resources.
- Inferring response policy from the Roc body or from the presence of `@live`.
- Retaining `@on` aliases or pre-encoded JSON response handling.
- Changing Datastar action syntax or the JavaScript dependency.

## Constraints that do not move

| Constraint | Required behavior |
| --- | --- |
| Public route | Method and path remain authored, inspectable HTTP contracts. |
| Body opacity | The template parser treats ordinary Roc bodies as opaque; syntax selects response policy.[^parser][^ungram] |
| Handler arity | Existing context-only and context-plus-request adapters keep working.[^lower][^compile-tests] |
| Server-owned UI | Commands do not return domain JSON to Datastar for DOM rendering. |
| Negotiation | A successful Datastar command gets 204; a caller without `Datastar-Request` gets JSON.[^dispatch] |
| Errors | Datastar keeps the HTML developer overlay; API clients keep status 500 plus a JSON error object.[^dispatch] |
| Duplicate routes | Collisions are detected across all semantic declarations.[^validate] |
| Formatter | Formatting is idempotent and preserves declaration kind and method.[^pprint] |
| Forward progress | Every scanner/parser recovery path advances on malformed or unclosed input.[^parser] |

## Clean-cut internal model

There is no compatibility surface. Remove `OnDecl`, its optional `json`
response identifier, and the pre-encoded JSON dispatch branch. Convert every
existing handler in the repository in the same change:

```rocci
@on:post("/actions/counter/increment") json = |{ db }| {
    count = increment_count!(db)?
    Json.to_str({ count })
}
```

becomes:

```rocci
@command("/actions/counter/increment") = |{ db }| {
    count = increment_count!(db)?
    { count }
}
```

The normalized route model has only the semantic response roles needed by the
new language: document HTML, one-shot patch HTML, and command data. Rename
internal `RespondKind::Json` to `RespondKind::Command` (or an equally explicit
name) so no generated path suggests that handlers return JSON text. A returned
`Str` is ordinary command data and therefore serializes as a JSON string.

## Phase 0 — Freeze contract and prove Roc encoding

**Bound**

- No grammar changes.
- Use the pinned Roc nightly and the same platform imports as generated apps.

**Work**

1. Add a focused generated-Roc probe that returns a record containing `Str`,
   `Bool`, integer, optional/tagged data, and a list.
2. Compile both `Json.to_str` and `Json.to_str_try` paths against the pinned
   nightly. Prefer `to_str_try` in host dispatch so fallible encoders have an
   explicit error path.[^roc-json]
3. Freeze the four-declaration table, method rules, clean-cut removal policy,
   and `@patch`/`@fragment` naming gate in tests or design fixtures.

**Exit**

- The probe compiles through the actual Rocci build platform.
- The plan records the exact encoder API used by generated dispatch.
- No author-written JSON interpolation remains in the proposed examples.

**Encoder API (frozen)**

The Phase 0 probe compiled both builtins through the same `basic-webserver`
0.16.0 / `http` 1.0.0 platform imports as generated apps:

| Call | Result | Use in generated dispatch |
| --- | --- | --- |
| `Encoding.Json.to_str(data)` | `Str` | total encoders only |
| `Encoding.Json.to_str_try(data)` | `Try(Str, err)` | ordinary-client command success and host API error objects |

Prefer `Encoding.Json.to_str_try(data) ?? fallback` (or an explicit `match` on
the `Try`) so fallible encoders have an error path. No `import` is required;
the names are Roc builtins. A returned `Str` encodes as a JSON string. Do not
author `"{\"count\":...}"` interpolation in command bodies.[^roc-json]

## Phase 1 — AST, parser, recovery, and validation

**Bound**

- Owning crate: `crates/rocci-template`.
- Do not change generated HTTP behavior in this phase.

**Work**

1. Replace `OnDecl` in the ungrammar with `ViewDecl`, `PatchDecl`, and
   `CommandDecl`; keep the existing `LiveDecl`.[^ungram]
2. Parse the canonical headers:

   ```text
   @view(path)
   @patch(path)
   @patch:put(path)
   @patch:patch(path)
   @patch:delete(path)
   @command(path)
   @command:put(path)
   @command:patch(path)
   @command:delete(path)
   @live
   ```

3. Preserve zero-, one-, and two-parameter source shapes wherever the existing
   declaration grammar permits them.
4. Validate path literals, supported methods, duplicate routes, and the
   reserved generated `/sse` route.
5. Add targeted diagnostics and recovery for `:post`, `:get`, unknown methods,
   missing path, missing `=`, malformed/unclosed parentheses, stray selector
   brackets, `@action[patch]:delete`, arrow-response experiments, and removed
   `@on` declarations. The `@on` diagnostic points to the appropriate semantic
   noun but does not parse or lower the old declaration.
6. Add monotonic-progress regression inputs for malformed declarations nested
   among components, styles, Roc blocks, and valid following handlers.[^parser]

**Exit**

- `cargo run -q -p rocci-ungram -- check` passes.
- Parser/validator snapshots cover every accepted header and every rejected
  near miss.
- Malformed-input tests terminate and retain later valid declarations.
- `cargo test -p rocci-template` passes before lowering behavior changes.

## Phase 2 — Normalize declarations and encode command data

**Bound**

- Owning layers: `rocci-template` lowering and `rocci-cli` generated dispatch.
- Keep public paths, handler function naming, request arity, and error UI.

**Work**

1. Normalize semantic nodes into existing route metadata rather than creating
   a parallel router:

   - `ViewDecl` → GET document;
   - `PatchDecl` → method plus patch response;
   - `CommandDecl` → method plus command-data response.

2. Replace the JSON-text response metadata with a command-data response kind;
   delete raw JSON success handling.[^lower]
3. Generate command dispatch in this order:

   ```text
   handler success
       +-- Datastar-Request: true --> 204, no serialization
       `-- ordinary client --------> Encoding.Json.to_str_try(data)
                                      +-- Ok --> 200 application/json
                                      `-- Err -> 500 JSON error
   ```

4. Preserve the existing consumer-aware handler-error branches. Encode the
   host's own API error object with Roc's JSON encoder as well, so inspected
   error text is escaped correctly.[^dispatch][^roc-json]
5. Ensure a returned `Str` is encoded as a JSON string, never treated as raw
   JSON.
6. Retain compile-time encoder constraints by compiling the ordinary-client
   branch even though Datastar success skips runtime encoding.

**Exit**

- Lowering snapshots show stable generated handler names and the correct
  response kind for all semantic declarations.
- Dispatch tests prove command 204, record/list JSON, JSON-string encoding,
  encoder failure, and handler failure. No raw JSON-text success branch remains.
- A real Roc build fails usefully when a command returns a value without a JSON
  encoder.
- `cargo test -p rocci-template -p rocci-cli` passes.

## Phase 3 — Formatter, inspect output, LSP, and syntax fixtures

**Bound**

- No new runtime behavior.
- Keep semantic declaration names visible throughout formatting and inspect
  output.

**Work**

1. Format every canonical declaration, including `@patch:patch`, and prove
   two-pass idempotence.[^pprint]
2. Extend inspect output with declaration kind, normalized method, path, and
   response role.
3. Add document symbols, semantic highlighting, completion, hover, and
   diagnostics for `view`, `patch`, and `command`.[^lsp]
4. Replace all `@on` entries in `test/AllSyntax.rocci` with the complete
   accepted matrix, parameter variants, and `@live`.[^all-syntax]
5. Add editor fixtures that place malformed semantic declarations before valid
   components to verify recovery and source ranges.

**Exit**

- `cargo run -q -p rocci-cli -- inspect --ast test/AllSyntax.rocci` reports all
  constructs with correct spans.
- Formatter snapshots and LSP tests pass.
- Highlighting distinguishes declaration nouns from the `:patch` HTTP method.

## Phase 4 — Complete examples and repository-wide cutover

**Bound**

- Examples must be runnable applications, not parser-only snippets.
- A live stream and a one-shot patch must not both own the same element id.

**Work**

1. Convert the standalone counter to `@view` plus `@patch`; its one-shot
   behavior remains unchanged.[^counter]
2. Convert live-counter to `@view`, `@command`, and `@live`. Return `{ count }`
   from increment/reset instead of an encoded string.[^live-counter]
3. Convert the Rocdown counter island to the same command-data contract and
   verify it still uses shared generated dispatch.[^hybrid-counter]
4. Add `examples/rocci/standalone/handler-matrix/HandlerMatrix.rocci` containing
   every accepted construct in one build:

   ```text
   @view
   @patch, @patch:put, @patch:patch, @patch:delete
   @command, @command:put, @command:patch, @command:delete
   @live
   ```

   Patch handlers return uniquely identified `Html` fragments. Command
   handlers return records/lists. The live handler owns a separate id so the
   example does not demonstrate racing render channels.
5. Give the matrix a README with `curl` commands for every method, a Datastar
   header example expecting 204, and ordinary-client examples expecting JSON.
6. Inventory and convert every active `.rocci`, `.rocdown`, generated fixture,
   test input, documentation snippet, and skill. The current inventory spans
   `crates/rocci-template`, `rocci-cli`, `rocci-lsp`, `rocci-rocdown`, and
   `rocci-rocdown-cli`; standalone and Rocdown examples; `site/`; both
   `AllSyntax` fixtures; public docs; and repository skills. Keep old spelling
   only inside rejection-diagnostic tests and explicitly historical knowledge
   records.
7. Remove every manual `Json.to_str` used solely to construct a command success
   body; retain unrelated JSON encoding where it is genuinely application work.

**Exit**

- Release builds succeed with the pinned Roc compiler for counter,
  live-counter, handler-matrix, and the Rocdown island.
- HTTP smoke tests verify method routing, response content types, 204
  negotiation, JSON shapes, and one-shot SSE patches.
- Two live-counter tabs update from the stream after a command.
- The matrix README has no manual JSON string construction.
- Repository searches find no active `@on:` declarations or pre-encoded
  command-result strings.

## Phase 5 — Public contract and removal diagnostics

**Bound**

- Documentation describes semantic syntax as proposed until the implementation
  and real-build gates are green.
- Document the cutover as a removal, not a deprecation period.

**Work**

1. Update the owning crate README with the four-value/response table and state
   that `@on` and the trailing `json` marker are removed.[^template-readme]
2. Rewrite the server-actions guide around one-shot patch versus command/live,
   including the ordinary JSON API branch and the fact that JSON does not morph
   server-rendered HTML.[^server-actions]
3. Update the public Rocci reference, rendering model, examples index, and
   relevant author/stack/language skills.
4. Add removal diagnostics with exact rewrites:

   - `@on:get(path)` → `@view(path)` when it is a document;
   - `@on:post(path)` → `@patch(path)`;
   - `@on:delete(path) json` → `@command:delete(path)` plus removal of explicit
     `Json.to_str` from the body.

   These are diagnostics, not supported aliases: the old source never lowers.
5. Explain that a command returning `Str` produces a JSON string. Authors who
   want a JSON object return a record.
6. Run the naming gate on complete examples. If `@patch` is repeatedly read as
   the HTTP method, rename it to `@fragment` across grammar, tests, examples,
   docs, and skills before stability—not as a compatibility alias.

**Exit**

- A new author can predict body value and browser/API response from each
  declaration without consulting generated Roc.
- Searches for `@command` inventory negotiated JSON endpoints.
- Documentation never says that command JSON renders durable UI in Datastar.
- The `@patch`/`@fragment` decision is explicit before syntax stabilization.

## Phase 6 — Integrated validation

**Bound**

- Use temporary output directories; do not treat `dist/` as source.
- Failed example builds must not replace previously valid output.

**Work and exit commands**

1. Language and generated dispatch:

   ```sh
   cargo run -q -p rocci-ungram -- check
   cargo test -p rocci-template
   cargo test -p rocci-cli
   cargo test -p rocci-lsp
   cargo run -q -p rocci-cli -- inspect --ast test/AllSyntax.rocci
   ```

2. Rocdown consumer boundary:

   ```sh
   cargo test -p rocci-rocdown -p rocci-rocdown-cli
   ```

   Add or use a focused `ROCCI_REQUIRE_ROC=1` island compilation test so this
   gate proves generated Roc rather than only Rust metadata.

3. Real Roc release builds, each targeting a fresh temporary directory:

   ```sh
   cargo run -q -p rocci-cli -- build --release examples/rocci/standalone/counter/Counter.rocci -o <tmp>/counter
   cargo run -q -p rocci-cli -- build --release examples/rocci/standalone/live-counter/LiveCounter.rocci -o <tmp>/live-counter
   cargo run -q -p rocci-cli -- build --release examples/rocci/standalone/handler-matrix/HandlerMatrix.rocci -o <tmp>/handler-matrix
   ```

4. Repository gates:

   ```sh
   cargo fmt --all -- --check
   cargo test --workspace
   cargo run -q -p rocci-rocdown-cli -- build docs
   cargo run -q -p rocci-okf -- check knowledge --profile rocci
   ```

5. Run matrix HTTP smoke tests against the same origin used by the webview.
   Assert response status, `Content-Type`, and body for all eight mutation
   combinations, plus document and live routes.
6. Run a final `rg` inventory over `crates`, `examples`, `site`, `test`, `docs`,
   and `.agents`. Every remaining `@on:` occurrence must be a deliberate
   removal-diagnostic input; every active handler and documented example must
   use the semantic declarations.

The phase exits only when every accepted construct parses, formats, lowers,
compiles through Roc, starts, and answers its expected HTTP method. Report OKF
lifecycle/provenance warnings separately from errors.

## Roll-forward and rollback

Land the AST/parser, normalized metadata, generated dispatch, tooling, examples,
and documentation together. There is no mixed-language release state. If
integration fails before release, revert the entire cutover as one change. Once
released, roll forward on the four semantic declarations; do not restore a raw
JSON-text branch or reinterpret `@command` data as wire bytes.

[^research]: The companion report compares response-selector spellings, four nouns, Datastar JSON semantics, and pure API behavior.
[^ungram]: The current AST specifies `OnDecl` and `LiveDecl`; semantic declarations need explicit nodes and spans.
[^parser]: The current hand-written parser owns declaration recognition and recovery around opaque Roc bodies.
[^validate]: Current validation owns method restrictions, response identifiers, and duplicate routes.
[^lower]: Lowering currently maps routes to document, patch, or JSON response metadata and adapts handler arity.
[^pprint]: The template formatter currently prints the shipped declaration surface and must preserve new semantic nouns.
[^compile-tests]: Existing tests cover current handler arities, response markers, duplicates, live injection, and example compilation.
[^dispatch]: Generated dispatch currently serves documents, one-shot SSE patches, and negotiated 204/JSON with consumer-aware errors.
[^lsp]: Rocci analysis derives symbols and diagnostics from parsed declarations and must expose the semantic surface.
[^all-syntax]: The all-syntax fixture is the repository's inspectable grammar integration corpus.
[^counter]: The standalone counter is the canonical one-shot patch example and must remain one-shot.
[^live-counter]: The live counter is the canonical command plus shared-stream example.
[^hybrid-counter]: The Rocdown counter proves that islands consume the same generated handler dispatcher.
[^template-readme]: The owning crate README documents the public standalone handler contract.
[^server-actions]: The public guide teaches direct patches separately from live commands and is the primary DX migration surface.
[^roc-json]: Roc builtins `Encoding.Json.to_str` (total) and `Encoding.Json.to_str_try` (fallible `Try(Str, err)`) compiled against the pinned nightly and the Rocci generated-app platform.
[^follow-up]: The follow-up re-evaluates command JSON, GET fragments, and low-level patch-signals after the four-noun cutover shipped.
