---
type: Implementation Plan
title: Investigate template preparation and Html runtime costs before choosing a product change
description: "Follow the September 12 typed-template results with stronger evidence capture, HTML compatibility tests, controlled runtime experiments, and conditional library/host probes. Prefer improvements that preserve Rocci source lowering; no new grammar, runtime unification, or product cutover is selected."
tags: [domain/rocci, integration/roc, concern/architecture, concern/rendering, concern/performance, concern/validation]
status: draft
generated: { by: process:cursor, at: 2026-09-12T10:40:00Z }
stale_after: 2026-10-12
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../../research/rocci/compile-time-template-preparation.md
    title: Findings, limitations, and local experiment results
  - id: receipt
    resource: ../../research/rocci/compile-time-template-preparation-results.json
    title: September 12 raw experiment receipt
  - id: phase-0-receipt
    resource: ../../research/rocci/compile-time-template-preparation-phase-0-results.json
    title: Phase 0 hashed baseline receipt with named cases and harness faults
  - id: runner
    resource: ../../../roc/template-preparation-experiment/run.py
    title: Current experiment construction, checks, and timing assertions
  - id: adapter
    resource: ../../../roc/template-preparation-experiment/TypedTemplate.roc
    title: Shared sample and renderer type variable
  - id: counter
    resource: ../../../roc/template-preparation-experiment/allocations.c
    title: macOS libc call counter
  - id: template-readme
    resource: ../../../crates/rocci-template/README.md
    title: Current template and generated-Roc contract
  - id: lower
    resource: ../../../crates/rocci-template/src/lower/html.rs
    title: Constructor lowering and expression emission
  - id: platform-html
    resource: ../../../crates/rocci-platform/platform/Html.roc
    title: Node rendering, fragments, escaping, and boolean helper
  - id: cli-html
    resource: ../../../crates/rocci-cli/runtime/Html.roc
    title: Runtime wrapper and eager fragment serialization
  - id: ui-html
    resource: ../../../crates/rocci-ui/runtime/Html.roc
    title: String constructors and repeated split/join escaping
  - id: theme
    resource: ../../../crates/rocci-rocdown/src/plan/theme.rs
    title: Theme painters select Str signatures
  - id: platform-readme
    resource: ../../../crates/rocci-platform/README.md
    title: Platform ownership, application pin, and vendor provenance
  - id: prior-html-plan
    resource: ./html-node-lowering.md
    title: Earlier NavList measurements and status-quo decision
  - id: native-plan
    resource: ./roc-native-template-compiler.md
    title: Separate source-emitting Roc compiler prototype
  - id: pure-render
    resource: ../../decisions/pure-render-components.md
    title: Pure components from explicit inputs to Html
  - id: rust-catalog
    resource: ../../decisions/rust-catalog-rocci-shell.md
    title: Rust catalog and compiled Rocci shell boundary
---

# Investigate template preparation and Html runtime costs

## Goal

Determine which part of the prepared-template experiment's advantage is
portable to Rocci, and whether a restricted pure Roc template library has a
useful role independent of that performance result. End with a supported
choice: a focused runtime improvement, continued library research, or no
product change. Do not infer that compile-time parsing itself caused the
measured speedup.[^research]

This is a **new, paired investigation plan**, not an extension of the
source-emitting Roc parser port. It also leaves the earlier HTML-lowering
plan's September 9 status-quo outcome intact: that decision used small,
one-shot workloads, while the new experiment uses repeated rendering and
100-row lists. Neither result invalidates the other. This plan does not
activate the older plan's skipped fusion or runtime-unification phases.
[^prior-html-plan][^native-plan]

**State:** draft; Phase 0 completed locally on `compile-time-template-preparation`.
The September 12 receipt is preserved. The Phase 0 receipt reproduces the
claimed 100-row prepared win and small-node win, distinguishes cache-hit
upstream tests from `roc test --no-cache`, and reports named HTML findings
plus harness faults. Not hosted-CI complete. Phases 1–5 have not started.
[^phase-0-receipt]

## Evidence and open questions

| Established local evidence | What remains unproven |
| --- | --- |
| A closure sharing the sample/input type passes 16 focused probes; added, missing, and nested fields are rejected. | Exported, inferred, nominal, and higher-order uses across modules; typed composition; behavior on a later compiler. |
| The receipt reports 62 upstream tests passing. | The final receipt's upstream invocation used cached results; a fresh checkpoint should distinguish cached and uncached runs. |
| Prepared rendering was 3.1–3.8 times faster than the product node runtime on two 100-row fixtures, with fewer intercepted allocation calls. | Attribution to escaping, growth, node construction, encoding, or interpretation; other workloads, platforms, and actual HTTP serving. |
| Product nodes won the smallest runtime fixture, and prepared templates cost more to check/build. | Warm incremental behavior, many-template scaling, first-use latency, and practical authoring cost. |
| A CR-in-attribute mismatch is explicit; quote entity spellings can differ without changing parsed meaning. | A broader HTML contract including fragments, void elements, boolean attributes, raw-text contexts, and browser parsing. |
| Filename and line appear in preparation diagnostics. | Structured offsets, precise columns, partial origins, stable error codes, and LSP navigation. |

These statements come from the recorded results and inspected adapter/runner.
The experiment files' stored hashes match the current files when this plan
was authored; the receipt does not hash every staged Rocci runtime input.
Current performance evidence therefore needs stronger input capture before
it serves as a product-selection baseline.[^receipt][^runner][^adapter]

## Out of bound

- Replacing `.rocci` parsing/lowering, evaluating arbitrary Roc expressions
  from strings, or changing component parameters, defaults, or handler syntax.
- Product runtime unification, static-chunk fusion, a second emit mode, or a
  compiler-pin upgrade as a prerequisite for these investigations.
- Moving static article/catalog work from Rust to Roc or interpreting a
  Rocci theme in Rust to bypass its compilation.[^rust-catalog]
- Publishing a template package, vendoring new upstream code, deploying a
  site, or sending upstream issues/PRs. Prepare reproducible evidence if such
  follow-up is warranted; external communication is not part of this plan.
- A general HTML sanitizer, URL-policy engine, Roc interpreter, or full
  Mustache compatibility project. Escaping tests do not establish those claims.

## Constraints that do not move

1. `@component` remains a pure function from explicit values to Html;
   arbitrary Roc holes still become Roc source checked by Roc.[^pure-render][^lower]
2. Preserve the current product parser, public Html constructor API, expression
   source maps, and CSS ownership while testing implementation alternatives.
   A pure runtime optimization should not rewrite generated Roc goldens.
   [^template-readme][^lower]
3. Keep experiments under `roc/template-preparation-experiment/`, findings and
   receipts in `knowledge/`. Temporary runtime copies may vary one algorithm
   at a time; no product file changes in this investigation's phases.
4. Correctness is independent of performance. Record byte serialization,
   parsed HTML, unsupported cases, expected incompatibilities, and harness
   execution failures as separate results. A known failing fixture must not
   become a production acceptance result merely because it is expected.
5. Preserve the September 12 receipt. Write a new receipt for each meaningful
   experimental revision; do not replace historical measurements with reruns.
6. Run bounded subprocesses. Treat timeouts, signals, and compiler crashes as
   failures to investigate, never as successful negative type tests. Terminate
   timed-out children and descendants before another run; retain diagnostics.

## Order and ownership

```text
0 evidence capture -> 1 correctness contract -> 2 controlled cost experiments
                                             -> 3 library boundaries (conditional)
2 promising candidates ----------------------> 4 representative host checks
1 + 2 + any applicable 3/4 -------------------> 5 recommendation and handoff
```

Prioritize Phases 0–2. Phase 3 is useful only if consuming templates with
ordinary `roc` remains a goal in its own right; it is not required to improve
the existing Html runtime. Phase 4 evaluates only surviving candidates, not
every experimental variant.

| Work | Owner if a later implementation is chosen |
| --- | --- |
| Prepared-template adapter and benchmark machinery | Isolated experiment first; no product crate selected for a library yet. |
| Product Html serialization/escaping | `crates/rocci-platform/platform/Html.roc`; account for its documented vendor provenance. |
| Legacy wrapper behavior | `crates/rocci-cli/runtime/Html.roc`, only where still consumed. |
| String Html behavior | `crates/rocci-ui/runtime/Html.roc` and other actual consumers identified by source inspection. |
| Theme integration | `crates/rocci-rocdown`; painters currently select `Str`. |
| Template grammar/source emit | `crates/rocci-template`, excluded from the first candidate changes. |

The ownership table follows current runtime code and platform/theme
documentation, rather than assuming all applications use the same wrapper.
[^platform-readme][^platform-html][^cli-html][^ui-html][^theme]

## Phase 0 — Make the evidence reproducible and failures explicit

**Bound**

- Retain the original receipt, engine revision, and compiler pin. Capture
  hashes for the runner, adapter, fixtures, generated app sources, every staged
  Html/Attribute module, and platform artifact; record OS/architecture, CPU,
  compiler flags, dirty tracked inputs, and untracked experimental inputs.
  A Git HEAD alone cannot describe an uncommitted experiment.
- Emit timestamps and hashes directly from the runner. Distinguish cache-hit
  tests, warm incremental checks/builds, and `--no-cache` runs. Repeat the
  initial probes once without cached test results using supported Roc options.
- Replace positional expected-mismatch indexes with named cases and separate
  `harness_ok`, `type_contract_ok`, and `html_compatible` summaries. Missing
  variants, failed builds, missing allocation records, and exceptions must
  produce a complete failed receipt, never an incomplete success summary.
- Check **every** runtime repetition and instrumented output, not just the
  first repetition's checksum. The current checksum sums lengths; add untimed
  full-output comparisons or content digests for the same dynamic inputs.
  Do not insert expensive hashing into the timed renderer loop.
- Preserve raw output bytes. Treat invalid UTF-8 from these valid-input HTML
  programs as an error instead of silently replacing bytes. Add small harness
  fault probes: absent backend, wrong output of the same length, wrong expected
  error, corrupted source hash, and a controlled timed-out child.

The current runner compares only the first timing sample's length checksum,
recognizes the expected CR mismatch by array position, and records allocator
linearity without making it an acceptance condition. These are sufficient
for the initial observation, but too weak for choosing a product change.
[^runner][^counter]

**Exit:** one new reproducible baseline receipt, all original probes accounted
for, and deliberate harness faults correctly reported. If the claimed speed
ordering does not reproduce, record that result and investigate the discrepancy
before expanding candidates; do not average incompatible runs together.

## Phase 1 — Define compatibility before optimizing

**Bound**

- Create an explicit fixture contract with expected authored data, emitted
  bytes where exact bytes matter, and parsed element/text/attribute values.
  Cover quotes, ampersands, CR/LF/CRLF, tabs, Unicode, empty strings, long
  strings, nested lists, empty lists, conditional branches, void elements,
  document versus fragment output, and nested components/Html body values.
- Confirm CR and quote cases using a browser HTML parser or a pinned HTML5
  parser. Retain the small Python event comparator as a fast fixture helper,
  not a general DOM-conformance oracle. Keep literal-newline preprocessing
  distinct from character-reference decoding.
- Define the restricted library's supported interpolation positions. Begin
  with ordinary text and complete, quoted ordinary attribute values. Explicitly
  reject unsupported dynamic tag/attribute names and script/style/event-handler
  contexts in any HTML-aware prototype; leave arbitrary expression execution
  and raw HTML outside its promised safe subset.
- Probe boolean attributes and fragment/document semantics independently of
  the performance fixture. The product `boolean_attribute` helper currently
  constructs the same empty attribute in both branches, while the string
  helper omits it for false. This is code evidence of a discrepancy, not proof
  of its reachability through current `.rocci` syntax. Trace the caller and
  classify the contract before proposing a correction.[^platform-html][^ui-html]
- Classify each mismatch as equivalent serialization, product/string drift,
  restricted-library limitation, or unresolved contract. Prefer the documented
  public contract and browser behavior over declaring either current backend
  universally correct. Do not normalize away a DOM-value difference.

**Exit:** a compatibility matrix with no unexplained differences in benchmarked
cases, explicit unsupported cases, and reproductions for real discrepancies.
A candidate must pass its promised subset; an expected failure remains a
failure of wider parity. Any independent product correctness bug gets a
focused proposed repair with its own scope, even if no optimization survives.

## Phase 2 — Identify the source of the speed and allocation difference

**Bound**

Freeze compiler, host, inputs, and correct output within each comparison.
Keep current generated Roc unchanged. First measure isolated escape and
append/serialization kernels; then test only promising changes in the full
fixture. Limit the initial sweep to these factors:

| Comparison | Question answered |
| --- | --- |
| Current node escape versus a context-correct scan/copy implementation | How much comes from building the escaped byte buffer, including the no-escape fast path? |
| Current string escape versus the same context-correct strategy | How much comes from repeated split/join passes? |
| Same renderer with current versus geometric/pre-sized output growth | How much comes from buffer growth rather than template representation? |
| Prepared rendering with encoding measured separately and with an already encoded value | How much is context serialization versus operation execution? Keep pre-encoding outside normal end-to-end numbers. |
| A small direct Roc string-builder control with matched escaping/growth | Does prepared interpretation still win when the source-generated alternative uses comparable algorithms? This is a control, not a second product lowerer. |

The node runtime currently folds over UTF-8 bytes and appends/concatenates;
the string runtime escapes through repeated split/join replacements. These
are concrete candidates for measurement, not proof of quadratic behavior or
a compiler allocation bug. Ownership and compiler optimization may change
their actual costs.[^platform-html][^ui-html]

- Change one factor per variant; combine winners only after recording the
  individual effects. Attribute speedups cautiously where a factor cannot be
  isolated without changing representation.
- Vary row counts (0, 1, 10, 100, 1,000), short/long text, clean/escape-heavy
  data, heterogeneous rows, nesting, and optional content. Sweep sizes first;
  run expensive measurements on selected representative cases, not a large
  Cartesian product. Keep input data dependent on runtime arguments.
- Use a timing source supported by the pinned platform to separate setup,
  rendering, and output, or clearly report process totals where separation
  is unavailable. Warm up, alternate backend order, record distributions,
  and repeat on at least two independent batches. Run long enough for signal
  to exceed startup/timer noise; retain the smallest real workloads as guards.
- Keep instrumented allocation runs separate. Validate the interposer with a
  known allocation control and double-length runs; distinguish call count,
  requested bytes, retained memory, and resident memory. Missing coverage on
  a platform is a limitation, not a zero-allocation result.
- Measure cold Roc-cache and warm incremental builds separately. Vary template
  counts and nesting to expose preparation scaling. Measure syntax/name error
  latency as well as successful builds. Keep a later compiler in a separate
  comparison column; do not replace the product pin during attribution.

**Exit:** a cost table identifying at least one supported explanation, or an
explicit inconclusive result. Select at most one runtime candidate for Phase 4.
As a proposed screening rule, require a repeatable improvement of at least
15% in a relevant render-heavy case, outside run-to-run spread, and no
repeatable regression above 5% in small cases after accounting for measurement
noise. Also report absolute time and build/binary/memory costs. These are
investigation filters, not established project SLAs; record any reasoned
departure before selecting a candidate. Correctness repairs need no speedup.

**Stop condition:** if the gain disappears with matched escaping and growth,
stop presenting template preparation as the performance opportunity. A
runtime-only improvement may still proceed. If no result is stable, retain
the product status quo rather than adding more implementation variants.

## Phase 3 — Test library boundaries, only if generator-free consumption matters

**Bound**

- Retain the existing `a -> Str` binding. Test exported renderers across
  modules, explicit versus inferred sample types, nominal view records,
  higher-order storage/passing, and simultaneous renderers of different shapes.
  Include negative calls at public module boundaries; never expose an untyped
  prepared template as the normal safe API.[^adapter]
- Test nested empty samples and distinguish structural checking from executing
  formatters on sample values. Probe sample-sensitive formatter failures.
  Document all accepted scalar/container types, including the rejection of
  unsupported fields that the template never uses.
- Compare a representative non-empty witness with an explicit schema approach
  **only if needed**. First prove what the pinned encoder/derivation API can
  express; do not assume reflection can derive a schema without values.
  Choose one approach or retain the sample limitation, rather than maintaining
  two public APIs by default.
- Exercise partials and typed Roc-to-renderer composition on one bounded
  fixture: a nested list with reusable row markup and an explicit body value.
  Define its output trust boundary. Do not turn a string-returning template
  into an Html body by blindly wrapping arbitrary user strings as trusted HTML.
- In a temporary upstream-derived variant, explore structured errors carrying
  filename, byte offsets, line/column, and partial origin. Offer a result-valued
  preparation path plus a deliberate top-level fail-fast adapter where feasible.
  Do not assign product `RCxxxx` IDs to an unrelated Mustache grammar.
- Check runtime-supplied template text separately: failure timing differs from
  a top-level constant. Add bounded malformed/nested-input probes and verify
  monotonic scanner progress for any scanner code changed in the experiment.
- Test HTML-context-aware escaping only within the Phase 1 subset. If meeting
  it requires a new language implementation, record that cost and stop this
  spike before building a second product grammar.

**Exit:** one explicit supported/unsupported API contract and evidence for
type binding, sample policy, composition, diagnostics, and HTML contexts.
Choose continued experiment or defer. The absence of arbitrary Roc expression
execution remains a hard boundary, not a deferred small feature.

## Phase 4 — Verify representative uses and the actual host

**Bound**

- For the winning runtime candidate, use one small standalone component, one
  larger list, and one actual theme painter. Include nested components,
  fragments, and scoped CSS where applicable. Use identical original and
  candidate inputs and preserve template-generated Roc. Large data-only lists
  cannot stand in for all Rocci applications.[^template-readme][^theme]
- Validate the candidate through the in-tree Rocci platform and the same HTTP
  origin used by the preview webview. Compare full-page and fragment responses;
  include a realistic low-load request path before any throughput experiment.
  If build/host limitations prevent this, retain the basic-cli result as such.
  Do not label it HTTP performance.[^platform-readme]
- Separate serialization CPU from routing, storage, compression, network, and
  concurrent allocator effects. One bounded load sweep is enough to establish
  whether renderer gains matter to the chosen application; avoid turning this
  into a general server benchmark.
- Confirm the selected candidate on the deployment-relevant Linux target as
  well as macOS before recommending a cross-platform runtime change. Record
  target/allocator differences and unsupported profiling tools. If only one
  target is available, narrow the recommendation explicitly.
- Carry a library candidate into this phase only if Phase 3 passed and a
  concrete generator-free use case exists. Do not retrofit a full Rocci
  application into the restricted subset merely to make the comparison pass.

**Exit:** representative correctness plus measured host benefit, or a documented
absence of benefit/coverage. No candidate proceeds based solely on the initial
100-row benchmark ratio.

## Phase 5 — Record the decision and prepare the smallest follow-up

**Bound**

Update the paired research with new receipts, explanations, uncertainties, and
the recommendation. Keep the outcomes independent:

| Evidence | Recommended next deliverable |
| --- | --- |
| Correctness discrepancy has a clear public contract | A focused repair plan in its owning runtime, independent of speed. |
| Runtime algorithm preserves semantics and wins representative measurements | A small implementation plan/PR scope for that algorithm; retain constructor lowering. |
| Prepared library offers useful generator-free consumption and passes its declared subset | A separately scoped library experiment or packaging proposal; no `.rocci` replacement claim. |
| Only a serialization control wins, or attribution remains unclear | Further bounded measurement or status quo; no automatic fusion/unification phase. |
| Gains vanish or compile/authoring costs dominate the intended use | Close this investigation with a reasoned no-change recommendation. |

For each proposed implementation, name exact source owners, accepted behavior,
required tests, documentation changes, migration consequences, and a revert
path. Preserve the older HTML plan's recorded outcome; link any new decision
to the new evidence. Do not mark an exploratory recommendation as an approved
Decision or publish a library from this phase.[^prior-html-plan]

**Exit:** a self-contained recommendation and, only where warranted, a concrete
scoped follow-up ready for implementation review. Otherwise a recorded stop
is a valid result. This plan's investigative phases do not themselves ship
the candidate algorithms.

## Validation and completion evidence

- Planning-only edits: `okmate check knowledge --profile base` and
  `git diff --check`. Report errors separately from existing citation/link and
  lifecycle/provenance warnings.
- Experiment changes: focused harness fault tests, positive/negative Roc
  probes, fresh upstream tests where the pinned CLI permits, and `roc fmt
  --check` on authored Roc. Keep expected-error programs outside passing
  compiler test suites. Use source inspection/AST output for `.rocci` fixtures.
- Later runtime changes: run owning Roc module expects plus the selected
  generated-component comparisons and crate tests. Run `cargo fmt --all --
  --check` for Rust changes; run `cargo test --workspace` before handing off
  multi-crate product changes. Use `ROCCI_REQUIRE_ROC=1` only for explicit
  generated-Roc integration checks, not parser-only unit tests.
- If a later change affects documentation-site behavior, build `docs` and
  inspect its output. Failed static builds must retain the previous output.
- Record local exits and receipts separately from hosted CI. Do not log a
  phase as CI-complete without required CI and Knowledge run IDs for the
  revision; no such hosted completion is claimed by this plan.

[^phase-0-receipt]: Local Phase 0 baseline on Apple M1 Max, `nightly-2026-09-03-62fcb65`; original September 12 receipt unchanged.
[^research]: Paired findings distinguish prepared data from source generation and document the current experiment's limits.
[^receipt]: Raw results, cached upstream test output, binary/timing samples, source hashes, and explicit compatibility failure.
[^runner]: Named cases, cache-mode checks, untimed output digests, complete failed receipts, and harness fault probes.
[^adapter]: Returned closure ties checking and rendering to one type variable.
[^counter]: Intercepted libc request counts; not peak memory or exhaustive Roc allocator events.
[^template-readme]: Public templates, interpolation, CSS, and constructor lowering contract.
[^lower]: Source-emitted Roc expressions and Html calls stay the product path.
[^platform-html]: Node escape buffer, fragment handling, and identical true/false boolean-helper branches.
[^cli-html]: Wrapper imports and eager Raw fragment conversion.
[^ui-html]: String escape/constructor implementations and false-attribute omission.
[^theme]: Theme painters use Str signatures; runtime changes may have separate consumers.
[^platform-readme]: Actual application platform, supported targets, and vendor ownership.
[^prior-html-plan]: Prior status-quo decision remains a historical outcome for its measured scope.
[^native-plan]: Source-emitting Roc compiler work is a separate project from prepared-template interpretation.
[^pure-render]: Existing pure component contract.
[^rust-catalog]: Static content ownership must not change as a renderer shortcut.
