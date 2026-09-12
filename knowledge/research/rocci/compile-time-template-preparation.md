---
type: Research Report
title: Compile-time template preparation can support a Roc library, but cannot replace Rocci lowering
description: "Templegen prepares template data at compile time. A typed closure fixes the reproduced context-shape gap in 16 local probes; prepared rendering wins the measured 100-row workloads but builds slower and differs from product HTML on carriage-return attributes. Full Rocci still needs Roc source lowering."
tags: [domain/rocci, integration/roc, concern/architecture, concern/syntax, concern/rendering, concern/performance]
status: draft
generated: { by: process:cursor, at: 2026-09-12T13:52:00Z }
stale_after: 2026-10-12
authority: exploratory
owners: [human:nils]
sources:
  - id: follow-up-plan
    resource: ../../plans/rocci/compile-time-template-preparation.md
    title: Bounded next investigations, compatibility requirements, and product decision criteria
  - id: scan-copy-plan
    resource: ../../plans/rocci/html-scan-copy-escape.md
    title: Follow-up to port node_scan_escape into platform Html
  - id: boolean-plan
    resource: ../../plans/rocci/html-boolean-attribute.md
    title: Follow-up to omit boolean_attribute when false
  - id: host-coverage-plan
    resource: ../../plans/rocci/html-scan-copy-host-coverage.md
    title: Exploration of HTTP origin and Linux coverage for node_scan_escape
  - id: string-theme-plan
    resource: ../../plans/rocci/html-string-theme-escape.md
    title: Exploration of split/join string Html used by theme painters
  - id: experiment
    resource: ../../../roc/template-preparation-experiment/run.py
    title: Reproducible type, diagnostics, HTML, compiler, and runtime experiment
  - id: typed-adapter
    resource: ../../../roc/template-preparation-experiment/TypedTemplate.roc
    title: Typed renderer closure over a prepared template
  - id: experiment-fixture
    resource: ../../../roc/template-preparation-experiment/Card.rocci
    title: Rocci data-path, boolean, and list comparison fixture
  - id: experiment-results
    resource: ./compile-time-template-preparation-results.json
    title: Local experiment receipt, raw diagnostics, timings, allocation counts, and source hashes
  - id: phase-0-receipt
    resource: ./compile-time-template-preparation-phase-0-results.json
    title: Phase 0 hashed baseline with named cases, cache modes, digests, and harness faults
  - id: phase-1-receipt
    resource: ./compile-time-template-preparation-phase-1-results.json
    title: Phase 1 compatibility matrix, boolean/document probes, and html5lib events
  - id: phase-2-receipt
    resource: ./compile-time-template-preparation-phase-2-results.json
    title: Phase 2 isolated escape, growth, encoding, and builder-control costs
  - id: phase-4-receipt
    resource: ./compile-time-template-preparation-phase-4-results.json
    title: Phase 4 representative fixtures, theme inspection, and host coverage limits
  - id: host-coverage-phase-2
    resource: ./html-scan-copy-host-coverage-phase-2-results.json
    title: Kept-workspace origin compare; Linux absence on Darwin
  - id: costs
    resource: ../../../roc/template-preparation-experiment/costs.py
    title: Isolated kernel and Card variants copied only into the experiment work directory
  - id: host
    resource: ../../../roc/template-preparation-experiment/host.py
    title: Phase 4 representative fixture and host-coverage runner
  - id: host-page
    resource: ../../../roc/template-preparation-experiment/HostPage.rocci
    title: Nested list plus scoped CSS page used for the product-origin smoke
  - id: allocation-counter
    resource: ../../../roc/template-preparation-experiment/allocations.c
    title: Optional macOS libc allocation-call interposer
  - id: product-html
    resource: ../../../crates/rocci-platform/platform/Html.roc
    title: Product node runtime and context-aware escaping
  - id: platform-readme
    resource: ../../../crates/rocci-platform/README.md
    title: Platform ownership, application pin, and vendor provenance
  - id: theme
    resource: ../../../crates/rocci-rocdown/src/plan/theme.rs
    title: Theme painters select Str signatures
  - id: string-html
    resource: ../../../crates/rocci-ui/runtime/Html.roc
    title: String Html runtime used in the comparison
  - id: html-preprocessing
    resource: https://html.spec.whatwg.org/multipage/parsing.html#preprocessing-the-input-stream
    title: HTML input stream newline normalization before tokenization
  - id: discussion
    resource: https://roc-lang.github.io/zulip-export/stream/304902-show-and-tell/topic/mustache.20template.20preprocessor.html#623196798
    title: Zak Kohler's Mustache preprocessor announcement and Anton's reply
  - id: templegen-main
    resource: https://github.com/y2kbugger/roc-templegen/blob/924ac031bd04af2ae04dd3def80410fce553fa3a/README.md
    title: Templegen code generation and two-branch comparison
  - id: template
    resource: https://github.com/y2kbugger/roc-templegen/blob/e13a34b63ee466a03588e1e3cba85fcb4d045f71/Template.roc
    title: Pure Roc template parsing, sample checking, operation lowering, and rendering
  - id: value
    resource: https://github.com/y2kbugger/roc-templegen/blob/e13a34b63ee466a03588e1e3cba85fcb4d045f71/Value.roc
    title: Derived context encoding into cells and offsets
  - id: formatters
    resource: https://github.com/y2kbugger/roc-templegen/blob/e13a34b63ee466a03588e1e3cba85fcb4d045f71/RuntimeFormatters.roc
    title: Explicit formatter registry and dynamic dispatch
  - id: runtime-main
    resource: https://github.com/y2kbugger/roc-templegen/blob/e13a34b63ee466a03588e1e3cba85fcb4d045f71/main.roc
    title: Embedded template files and top-level checked templates
  - id: benchmarks
    resource: https://github.com/y2kbugger/roc-templegen/blob/e13a34b63ee466a03588e1e3cba85fcb4d045f71/notes/benchmarks.md
    title: Author's September 10 side-by-side measurements and limitations
  - id: compiler-bug
    resource: https://github.com/roc-lang/roc/issues/11317
    title: Or-pattern closure regression reported on September 11
  - id: tutorial
    resource: https://github.com/roc-lang/roc/blob/62fcb65bfccb2467604635fda6747786654dab19/docs/mini-tutorial-new-compiler.md
    title: Pinned Roc tutorial on evaluating pure functions over constants
  - id: pin
    resource: ../../../docs/inventory.toml
    title: Rocci compiler pin
  - id: template-readme
    resource: ../../../crates/rocci-template/README.md
    title: Current Rocci template language contract
  - id: template-lib
    resource: ../../../crates/rocci-template/src/lib.rs
    title: Rust parse, validate, lower, metadata, and source-map output
  - id: lower-html
    resource: ../../../crates/rocci-template/src/lower/html.rs
    title: Roc expression emission and Html constructor lowering
  - id: tests
    resource: ../../../crates/rocci-template/tests/compile.rs
    title: Component composition, directives, ordinary Roc, and mapping regressions
  - id: source-map
    resource: ../../../crates/rocci-template/src/source_map.rs
    title: Rocci expression and structural origin maps
  - id: driver
    resource: ../../../crates/rocci-cli/src/driver.rs
    title: Generated Roc module staging and stylesheet extraction
  - id: hosted
    resource: ../../../crates/rocci-platform/platform/Rocci.roc
    title: Hosted compile and parse effects return source text and diagnostics
  - id: native-research
    resource: ./roc-native-template-compiler.md
    title: Existing Roc-native compiler investigation
  - id: native-postmortem
    resource: ./roc-native-template-compiler-postmortem.md
    title: Historical August compiler-port limitations
  - id: html-research
    resource: ./html-node-lowering.md
    title: Existing investigation of constructor lowering and render costs
  - id: pure-render
    resource: ../../decisions/pure-render-components.md
    title: Preserve pure render components
---

# Compile-time template preparation for Rocci

## Assessment

**Viable for a restricted, platform-independent template library; insufficient
as a replacement for the current `.rocci` language.** Templegen's pure Roc
branch removes an external generator by preparing a template data structure
during compilation. It still encodes application values and interprets
operations when rendering. It does not splice generated Roc expressions into
the module being compiled.[^template][^value][^runtime-main]

Rocci already generates ordinary Roc before the Roc compiler runs. Its
interpolations, conditions, patterns, helpers, and component calls become
actual Roc source that Roc can resolve and type-check. Preserving that
contract requires retaining source generation or introducing a separate
mechanism for binding executable Roc expressions. Parsing them into strings
at compile time is insufficient.[^template-lib][^lower-html][^tests]

This is research, not an approved language change or an implementation plan.
The assessment below distinguishes inspected source, local probes, the
author's measurements, and proposed follow-up experiments.

**Follow-up executed at the user's request:** the typed-closure adapter now
passes 16 targeted probes, alongside the 62 upstream tests. The larger local
workloads favor prepared rendering, while compilation and small-node rendering
favor current Rocci. A carriage-return attribute mismatch remains. See
[follow-up experiments](#follow-up-experiments) for the measured scope and
reproducible harness.[^experiment-results]

## What the linked project demonstrates

The linked September 10 Zulip message announces a **Zig source generator**.
Anton's September 11 reply suggests a standalone tool or a pure Roc package;
it does not document an official compiler build hook. The public archive
contains the exact requested message, `623196798`; it was used because the
live message API requires authentication.[^discussion]

The repository now contains two implementations. Inspected revisions on
2026-09-12:

| Branch | Revision | Compilation and rendering |
| --- | --- | --- |
| `master` | `924ac031bd04af2ae04dd3def80410fce553fa3a` | Zig reads Mustache and emits a typed Roc module; Roc compiles it; runtime calls the generated string builder. |
| `runtime-templates` | `e13a34b63ee466a03588e1e3cba85fcb4d045f71` | Roc embeds source strings, parses and checks top-level values, then renders through prepared operations and encoded context data. |

The generated version exposes a closed context record and a renderer. Its
formatter calls are direct Roc calls. This is already the broad architecture
of Rocci, although Rocci preserves arbitrary Roc and emits Html constructors
rather than Templegen's Mustache-oriented string builder.[^templegen-main][^template-readme]

The pure Roc version has a different boundary:

```text
compile time: embedded text -> parsed nodes + sample context -> checked ops
runtime:     actual context -> cells/offsets -> interpret ops -> HTML string
```

`Value` uses Roc's derived `encoder_for` to obtain field names and values
during checking. Runtime encoding omits names and uses the prepared field
positions. That is serialization into a restricted value vocabulary, not
reflection that can resolve arbitrary Roc source. The encoded scalar cases
are strings, booleans, I64, U64, and Dec, plus lists and records.[^value]

Formatter names have an explicit registry that unwraps encoded values and
calls named Roc functions. Adding a function to a Roc module does not
automatically make it callable by template text.[^formatters]

The technique is consistent with Roc's documented evaluation of pure calls
on constants. File embedding brings in **data**: the example app imports
Mustache files as `Str`. It does not import a custom language as executable
Roc.[^tutorial][^runtime-main]

## Local verification on Rocci's pin

`docs/inventory.toml` and the local `roc version` agree on
`nightly-2026-09-03-62fcb65`.[^pin] The following are original observations
from this investigation on Apple Silicon macOS, using unchanged
`Template.roc`, `Value.roc`, `RuntimeFormatters.roc`, and `Formatters.roc`
downloaded from the pinned runtime-templates revision. Files and probes were
kept in a temporary directory; no product code was changed.

| Probe | Observed result |
| --- | --- |
| `roc check smoke.roc` with a top-level checked template | Exit 0. |
| `roc smoke.roc` | Exit 0; `Hi &lt;Ada&gt;!`. |
| Check a template referencing `missing` against `{ name: "" }` | Exit 1; compile-time crash diagnostic identifies the missing field. |
| Check an item-referencing loop against a typed empty sample list | Exit 1; diagnostic requests at least one sample item. |
| `roc check wrong-shape.roc` below | Exit 0, despite a different record layout. |
| `roc wrong-shape.roc` | Exit 0; `Hi WRONG!`. |
| `roc test Template.roc` | All 62 tests passed, including imported-module tests. |

Minimal reproduction of the context-layout problem:

```roc
import Template

page = Template.compile("Hi {{ name }}!").check({ name: "" })

main! = |_| {
    echo!(page.render({ aaa: "WRONG", name: "Ada" }))
    Ok({})
}
```

For the successful escaping smoke, change the render argument to
`{ name: "<Ada>" }`. For the missing-field negative test, change the
template variable to `missing` and run `roc check`. For the empty-list
test, use:

```roc
sample : { items : List({ name : Str }) }
sample = { items: [] }
page = Template.compile("{{#items}}{{ name }}{{/items}}").check(sample)
```

Render `page` with `sample` in `main!` and run `roc check`. The compiler
rejects this even though the list's element type is annotated: checking
uses encoded sample values, not the list's static type schema.

The first sandboxed execution failed with `ShmOpenFailed`; rerunning the
reviewed probes with shared-memory access succeeded. Sandboxed checks also
reported unavailable cache writes. These were environment constraints, not
template failures. Commands had bounded timeouts; none required killing.
This initial probe round did not include an HTTP load test, optimized
application build, or Rocci integration. The follow-up below adds optimized
basic-cli comparisons. Neither round establishes whole-app compatibility.

## Type checking and sample checking differ

The concrete layout error follows from the API: `check` consumes any
encodable sample and returns an unparameterized `Template`; `render` accepts
an independently chosen encodable record. No type relation connects those
calls. A matching context shape is a caller obligation, whereas generated
`Ctx -> Str` functions express it to Roc.[^template][^templegen-main]

Because field positions are sorted by name, an added earlier field can
silently select a different value. The reproduction above demonstrates that
this is an output-correctness issue, not merely weaker diagnostic wording.
Empty list samples also need explicit representative values for bodies that
use their elements. Sample checking should not be described as equivalent
to type-checking every future renderer invocation.[^value][^template]

An API can return a renderer whose input type is tied to the sample, or hide
the template behind one explicitly typed render function. The follow-up now
tests the closure form: its shared type variable rejects the reproduced
top-level, nested-record, and list-element shape changes on this pin.
This addresses shape drift at the adapter boundary; it does not add execution
of Roc expressions embedded in strings.[^typed-adapter][^experiment-results]

## Fit against the Rocci contract

| Rocci capability | Fit with compile-time preparation |
| --- | --- |
| Literal tags/text and named data holes | Feasible in a pure parser and renderer, with Rocci's escaping and whitespace rules implemented deliberately. |
| `{count.to_str()}`, imported helpers, nested Roc expressions | Not supplied by this technique. Needs generated Roc, explicitly supplied functions, or a new expression interpreter. |
| `@if`, `@for`, `@match`, `@let` | Bool/list operations cover restricted control flow; arbitrary expressions, patterns, local bindings, and Roc semantics do not follow from Mustache operations. |
| Component calls, typed props/defaults, Html body arguments | Need typed function bindings and composition. Named partials over serialized records are a different contract. |
| `@css` scope IDs and extracted styles | Pure CSS preparation is plausible, but stable file identity, extraction metadata, and runtime injection still need integration. |
| Ordinary Roc, imports, handlers, fixtures/tests | Embedded text does not become module declarations. Current lowering and CLI consumers remain necessary. |
| Diagnostics, inspector, LSP mappings | Preparation can report syntax failures, but must separately preserve Rocci spans, diagnostic IDs, and expression mappings. |

These requirements are grounded in the current README, compiler output
record, lowering, regression tests, source maps, and CLI staging code, not
in an assumption that the earlier native-parser prototype reached
parity.[^template-readme][^template-lib][^lower-html][^tests][^source-map][^driver]

A prepared renderer could build `Html.Node` values instead of strings.
That preserves a possible composition boundary, but does not eliminate
runtime interpretation or solve expression binding. Pure render functions
remain the established component contract.[^pure-render][^template-readme]

The shipped `pf.Rocci.compile!` path is separate again: it is a hosted
effect returning Roc source and diagnostics. It is neither a pure
compile-time parser nor a facility for evaluating that returned source in
the surrounding compilation.[^hosted]

## Performance and compiler risk

Use the author's **same-run September 10 table** rather than mixing it with
the later README snapshot. On the author's six-core laptop and August 25
nightly, generated versus prepared/interpreted templates took 7–9 versus
38–41 microseconds for the small todos page, and 3.8–4.0 versus 4.5–5.3
milliseconds for the 1,000-row table. Cold builds were about 6.8 versus
12.9–14.5 seconds. The author reports substantial timing variability.
These numbers favor source generation in those workloads; they are not
Rocci benchmarks or proof of a universal ratio.[^benchmarks]

The benefit to investigate is removing a separate generator, not an assumed
speedup. Rocci's existing Html-lowering research already found little reason
to change constructor emission in its measured small-component workload;
that historical result should be remeasured if a new workload motivates
change.[^html-research]

The runtime branch's README reports a September 11 compiler regression.
Issue #11317 is open as inspected: an or-pattern arm whose closure captures
a bound variable can hang or crash, independently of whether the function
is evaluated at compile time. Its reported workaround is to avoid that
capture shape. This does not negate the successful September 3 probes, and
those probes do not show that the later regression is fixed.[^compiler-bug][^templegen-main]

The older Roc-native parser post-mortem is likewise evidence about its
August pin, not a current claim that every reported compiler bug persists.
Templegen shows that more substantial pure template preparation is practical
than a blanket reading of that historical record might suggest.[^native-postmortem]

## Recommendation

This investigation is complete locally. Compile-time template preparation is
**not** the performance opportunity: a matched string builder beat prepared
rendering. Keep Rust-owned `.rocci` parsing and source lowering.

| Evidence | Next deliverable |
| --- | --- |
| `boolean_attribute(False)` emits `disabled=""`; string omits it; `.rocci` only lowers True | [Omit the attribute when false](/plans/rocci/html-boolean-attribute.md); independent of speed. |
| `node_scan_escape` matches representative Node fixtures and is ~34% faster on the 100-row escaped Card | [Scan/copy escape in platform Html](/plans/rocci/html-scan-copy-escape.md) completed locally on `main`. Constructor lowering and `&#13;` kept. Linux, HTTP origin, and theme `Str` remain unmeasured. |
| Restricted library subset plus typed closure | Defer. No generator-free packaging from this investigation. |
| Builder-control beat prepared rendering | No fusion, unification, or second emit mode. The September 9 NavList status quo stands. |
| Theme painters use `Str`; candidate HTTP and Linux coverage absent | Explore separately: [HTTP/Linux Node coverage](/plans/rocci/html-scan-copy-host-coverage.md); [string/theme escape](/plans/rocci/html-string-theme-escape.md). Do not claim those hosts until those plans report. |

These follow-ups are exploratory plans, not approved Decisions. Scan/copy
Node escape shipped locally in platform Html; `boolean_attribute(False)`
is not started. This investigation's own phases did not ship the algorithms.
HTTP origin, Linux, and theme `Str` remain unmeasured.
[^follow-up-plan][^scan-copy-plan][^boolean-plan][^host-coverage-plan][^string-theme-plan][^phase-2-receipt][^phase-4-receipt][^html-research][^native-research]

Keep Rust-owned `.rocci` parsing and source lowering as the product path.
The existing native-compiler research remains relevant to a **Roc program
that emits `.roc`**, followed by an ordinary Roc compilation. Templegen's
runtime branch establishes a different option: **data templates prepared by
the compiler**, with application logic authored separately in Roc. It does
not bridge those two options automatically.[^native-research]

The small experiment proposed here has now been executed below. Its results
do not justify replacing `.rocci` or introducing another product grammar.
The restricted library remains deferred. Html runtime follow-ups are the
scan/copy escape plan and the boolean-helper repair, not fusion.
[^experiment-results][^scan-copy-plan][^boolean-plan]

For unchanged `.rocci` syntax, keep constructor lowering. Buffer-growth
fusion is not selected; scan/copy escaping is the measured Node runtime
change.[^html-research][^lower-html][^scan-copy-plan]

## Follow-up experiments

Executed on 2026-09-12 at the user's request. The harness lives in
`roc/template-preparation-experiment/` with reproduction instructions in its README.
It downloads hash-verified upstream sources into a temporary directory;
only the adapter, fixture, runner, and optional allocation counter are authored
in the repository. Raw observations and exact inputs are in the sibling
`compile-time-template-preparation-results.json` experiment receipt.
Phase 0 added `compile-time-template-preparation-phase-0-results.json` without
replacing that September 12 file.[^experiment][^experiment-fixture][^experiment-results][^phase-0-receipt]

### Typed renderer and diagnostics

The adapter's essential signature is:

```text
prepare : Str, Str, a -> (a -> Str)
          filename, source, sample -> renderer
```

Its full annotation constrains `a.encoder_for` to the existing `Value`
encoding. Preparation uses `Template.bag` to retain the filename in parse
errors, then `check_value` to prefix check errors with the same filename.
The result closes over the checked template and accepts only the sample's
type. There is no mutable registry or exposed untyped template in this
adapter's return value.[^typed-adapter]

All 16 focused probes met their explicit expected outcomes, and all 62
upstream tests passed:[^experiment-results]

| Area | Observed result |
| --- | --- |
| Same record type | Checks and renders correctly. |
| Reordered record fields | Accepted; output remains correct. Field order in a literal is not a distinct record shape. |
| Added or missing fields; Str changed to I64 | Rejected during `roc check`. |
| Nested record adds a field or changes a field type | Rejected during `roc check`. |
| List item adds a field or changes a field type | Rejected during `roc check`. |
| Empty runtime list, non-empty checking sample | Accepted and renders the empty branch. |
| Empty sample, body references item fields | Rejected despite an explicit element-type annotation. |
| Empty sample, constant section body | Accepted; a later two-item list renders the body twice. |
| Unknown template field and unclosed section on line 2 | Compile-time diagnostics include the template filename and line 2. |
| Unsupported U8 field or tag union | Missing encoder-method errors, sometimes followed by a generic compile-time crash diagnostic. An unused unsupported field still blocks encoding. |

The location improvement is message-level. Roc still points its main source
frame at the preparation call or imported engine; this does not provide
Rocci's byte spans, precise columns, or LSP source maps. The fixture's existing
Rocci AST and mappings were inspected and retained in the receipt.
No full source-map implementation was attempted.[^experiment-results][^source-map]

### Rendering equivalence

Three separately compiled apps consume actual argv data: prepared Mustache,
Rocci constructor output using the string runtime, and the same constructor
output using the product node runtime. The node app stages unchanged
`Html.roc` and `Attribute.roc` with the CLI wrapper's imports redirected to
local copies. All three use the same basic-cli 0.22.0 host. This measures
render libraries under a shared allocator, not HTTP serving.[^experiment]

The fixture covers literal elements, a dynamic attribute and text, a Bool
branch, and a list of records. Seven inputs cover ordinary text, empty data,
quotes/ampersands/angle brackets, Unicode, HTML-looking attacks, line breaks,
and template-looking text. Six cases agree with an independent escaped HTML
reference for every backend. No supplied script-shaped value becomes a
script element. Quotes have different byte spelling in text but equal parsed
meaning.[^experiment-fixture][^experiment-results]

The seventh case exposes a compatibility failure: a raw CR in an attribute
becomes LF when HTML is parsed. The prepared engine and Rocci string runtime
emit that raw CR; the product node runtime emits `&#13;` and preserves the
attribute value. All three normalize literal CR in text in the ordinary
HTML way. The harness preserves raw subprocess bytes and applies HTML input
newline preprocessing before entity decoding, so the recorded difference is
not Python subprocess newline conversion.[^experiment][^experiment-results][^product-html][^string-html][^html-preprocessing]

The runner records `html_equal: false` and keeps this mismatch as an explicit
negative compatibility fixture. Exit 0 means all expected experimental
outcomes, including that known incompatibility, were reproduced; it does
not mean the adapter achieved complete HTML parity.[^experiment]

### Local timings

Apple Silicon macOS, `nightly-2026-09-03-62fcb65`. Check times are medians of
three `roc check --no-cache` processes; each optimized build was measured once
with `roc build --no-cache --opt=speed`. These are cold **Roc-cache** runs,
not cold filesystem/platform-download runs.[^experiment-results]

| Backend | Check median | Optimized build | Binary bytes |
| --- | ---: | ---: | ---: |
| Prepared template | 283 ms | 1.940 s | 460,032 |
| Rocci string | 109 ms | 0.717 s | 425,120 |
| Rocci product nodes | 98 ms | 0.764 s | 425,024 |

Runtime values below are median wall milliseconds for **5,000 renders**, from
three uninstrumented processes. Each loop varies the title and boolean; the
text and row count come from argv. Checksums agree across all backends on
these workloads. Times include startup and shared context construction; the
small case is particularly sensitive to that overhead.[^experiment-results]

| Workload | Prepared | Rocci string | Rocci product nodes |
| --- | ---: | ---: | ---: |
| No rows, clean text | 12.66 ms | 17.20 ms | 9.60 ms |
| 100 rows, clean text | 98.65 ms | 715.18 ms | 373.82 ms |
| 100 rows, escaped text | 211.59 ms | 868.18 ms | 650.16 ms |

Prepared rendering is approximately 3.1–3.8 times faster than the product
node renderer on these two 100-row fixtures, while the smallest node case
is faster and the prepared app costs more to compile. These results are not
the same comparison as Templegen's own generated string builder. They compare
the current Rocci Html implementations with an engine using geometric string
growth and different escaping/encoding algorithms. The numbers do not isolate
compile-time preparation as the cause of the win.[^experiment-results][^template][^product-html][^string-html]

### Allocation-call measurements

Separate runs load a macOS interposer into only the measured child process.
It counts calls to libc `malloc`/`calloc` and `realloc`, subtracts a zero-render
baseline, and divides by 5,000. Runs with 10,000 iterations exactly double
the measured call deltas for all three workloads and backends. These are
intercepted allocation calls, not exhaustive Roc allocations or peak live
memory; allocation instrumentation is absent from the timing runs.
[^allocation-counter][^experiment-results]

| Workload/backend | malloc + calloc per render | realloc per render | Requested bytes per render, including realloc |
| --- | ---: | ---: | ---: |
| 100 clean rows / prepared | 27 | 15 | 38,560 |
| 100 clean rows / Rocci string | 2,553 | 306.5 | 248,461 |
| 100 clean rows / Rocci nodes | 918 | 306.5 | 123,951 |
| 100 escaped rows / prepared | 229 | 16 | 58,663 |
| 100 escaped rows / Rocci string | 2,756 | 211.5 | 284,972 |
| 100 escaped rows / Rocci nodes | 1,626 | 817.5 | 184,085 |

The half calls reflect alternating active/inactive renders, not fractional
allocations in one execution. Requested bytes count cumulative requests,
including reallocations; they should not be interpreted as retained memory.
The reduced allocation traffic is consistent with the large-fixture timing
advantage, but attributing the result to individual renderer algorithms needs
a separate controlled comparison.[^experiment-results][^allocation-counter]

### Phase 0 baseline

Executed on 2026-09-12 after the original receipt. Adapter, fixture, and
allocation-counter hashes match the September 12 files. The runner now
emits timestamps, input/product/generated hashes, the basic-cli platform
archive hash, OS/CPU, compiler flags, dirty tracked paths, and untracked
experimental paths. Named HTML cases replace positional mismatch indexes.
Summaries are `harness_ok`, `type_contract_ok`, and `html_compatible`.
`html_compatible` remains false because of the carriage-return attribute
case; `html_expected_findings_confirmed` is true. Invalid UTF-8 is an
error. Every timing repetition and instrumented checksum is checked.
Untimed `digest` hashes for the same dynamic inputs match across backends.
Upstream `roc test` was 34 ms on a cache-hit default run and 705 ms with
`--no-cache`. Deliberate harness faults (absent backend, same-length wrong
output, wrong expected error, corrupted source hash, timed-out child, and
git porcelain parsing) produce complete failed receipts.[^phase-0-receipt]

Apple Silicon macOS (`Apple M1 Max`), `nightly-2026-09-03-62fcb65`. Median
wall seconds for 5,000 renders, three processes:

| Workload | Prepared | Rocci string | Rocci product nodes |
| --- | ---: | ---: | ---: |
| No rows, clean text | 13.2 ms | 16.9 ms | 13.0 ms |
| 100 rows, clean text | 101.0 ms | 712.4 ms | 376.7 ms |
| 100 rows, escaped text | 214.9 ms | 860.5 ms | 642.8 ms |

The claimed ordering reproduced: prepared is fastest on both 100-row
workloads; product nodes are fastest on the smallest fixture. No-cache
optimized builds were 2.18 s prepared versus 0.80 s string and 0.77 s
nodes. Warm incremental checks drop to about 33–46 ms after the first
warm sample. Allocation-call scaling remained linear on the double-length
runs. These are still not HTTP measurements or isolated renderer latency.
[^phase-0-receipt]

### Phase 1 compatibility

Executed on 2026-09-12 with html5lib 1.1 as the pinned HTML5 parser. The Python
event helper remains a fast fixture check; literal CR/CRLF preprocessing is
recorded separately from character-reference decoding. The restricted library's
promised subset is ordinary text and complete double-quoted ordinary attribute
values. Dynamic tag/attribute names, script/style/event-handler contexts, raw
HTML, and arbitrary Roc expressions stay unsupported.[^phase-1-receipt]

Benchmarked Card cases have no unexplained differences. Quotes and some
ampersand spellings are equivalent serialization: html5lib text is
`&<>"' café 😀` for every backend. A raw CR in an attribute is a real
DOM-value split: prepared and the string runtime parse to LF (code 10);
product nodes emit `&#13;` and keep CR (code 13). That is product/string
drift plus a restricted-library limitation, not equivalent serialization.
[^phase-1-receipt][^html-preprocessing][^product-html][^string-html]

`Compat.rocci` adds void elements, a valueless boolean `input`, nested
lists, a nested component, and an Html body parameter. String versus node
markup for those extras is equivalent serialization except where CR appears.
Prepared Mustache cannot express those Rocci forms; those gaps are restricted
library limitations, not benchmark failures. Document wrappers also drift:
the string `render_document` inserts a newline after the doctype; the node
runtime concatenates `<!DOCTYPE html>` with no newline. Fragments take
different shapes (`Str` identity versus `List(Node)` -> `Fragment`).
[^phase-1-receipt]

`.rocci` valueless attributes lower only to `boolean_attribute(name, True)`.
Hand-written Roc that passes `False` shows product/string drift: string omits
the attribute (`<button>off</button>`); nodes still emit
`disabled=""` for both branches. Proposed repair, independent of speed: in
`crates/rocci-platform/platform/Html.roc` (and the CLI wrapper) make the false
branch omit the attribute so presence matches HTML boolean semantics. Tests
belong on that helper. Do not treat this as a `.rocci` syntax bug.
[^phase-1-receipt][^product-html][^lower-html]

### Phase 2 cost isolation

Executed on 2026-09-12. Isolated kernels and Card variants copy Html modules
only into the experiment work directory; generated Card Roc stays unchanged.
`--costs` times process totals (startup included). Kernels run 20,000 escapes.
Card cases use 3,000 renders except the 1,000-row sweep (500 renders).
Apple M1 Max, `nightly-2026-09-03-62fcb65`. `harness_ok` is true.
[^costs][^phase-2-receipt]

Isolated escape kernels (median process seconds):

| Kernel | Clean | Escaped |
| --- | ---: | ---: |
| Node fold/concat (current) | 5.9 ms | 9.6 ms |
| Node scan/copy | 4.0 ms | 6.2 ms |
| String split/join (current) | 6.2 ms | 8.1 ms |
| String scan/copy | 3.9 ms | 6.6 ms |

Scan/copy beats fold/concat and split/join on both clean and escaped kernels.
Geometric join growth on the current node renderer did not move the 100-row
escaped Card (401 ms vs 395 ms). Combining scan/copy with join growth after
the individual measurements was no faster than scan/copy alone (270 ms vs
265 ms).

Card 100-row escaped, 3,000 renders, median process seconds:

| Variant | Median |
| --- | ---: |
| Current product nodes | 401 ms |
| Node scan/copy escape | 265 ms (~34% faster) |
| Node join growth | 395 ms |
| Current string | 565 ms |
| String scan/copy escape | 386 ms |
| Prepared | 132 ms |
| Builder-control (scan/copy + join) | 115 ms |

The 1,000-row escaped sweep (500 renders) keeps the same ordering: nodes 651 ms,
scan/copy 463 ms, prepared 214 ms, builder-control 210 ms. Encoding 3,000
copies of a 100-row clean context took 16 ms median versus 63 ms prepared
end-to-end on the same row count, so context encoding is a minority of the
prepared process total and is kept outside the Card timings.
[^phase-2-receipt]

**Stop condition:** builder-control beat prepared rendering on the 100-row
escaped Card. Matched escaping and growth therefore remove the reason to
present compile-time template preparation as the performance opportunity.
A runtime-only scan/copy escape remains the interesting candidate.
[^phase-2-receipt]

**Selection:** `node_scan_escape` is the Phase 4 runtime candidate. The
proposed screen is at least 15% improvement on the 100-row escaped Card and
no more than 5% regression on the empty clean Card. Empty-card process totals
sit on a ~2 ms noise floor; the 1.0 ms scan-versus-node difference there is
inside that floor, recorded as a reasoned departure rather than a real
regression. `string_scan_escape` fails the screen (3.7% large-case gain and
a 34% empty-card regression). Join growth alone fails the 15% rule. Correct
output is unchanged versus the current node contract, including `&#13;` in
attributes. No product Html file was edited.
[^phase-2-receipt][^costs][^product-html]

### Phase 3 library boundaries

Phase 3 is conditional on generator-free consumption remaining a goal in its
own right. After the Phase 2 stop condition, that path is **deferred**, not
expanded. No new module-export, nominal-record, higher-order, or structured-error
probes were added. The supported/unsupported contract below is the existing
typed-adapter and Phase 1 subset, recorded as the library boundary for this
investigation.[^typed-adapter][^phase-1-receipt][^follow-up-plan]

Supported in this experiment:

- `prepare : Str, Str, a -> (a -> Str)` with one shared sample/renderer type
  variable. Added, missing, and nested-field shape changes fail `roc check`.
- Ordinary text and complete double-quoted ordinary attribute values.
- Bool sections and list sections when the checking sample contains a
  representative item.
- Empty runtime lists when the checking sample was non-empty.
- Reordered record fields in a literal (not a distinct shape).

Unsupported, and not promised:

- Dynamic tag or attribute names; script, style, and event-handler contexts;
  raw HTML; arbitrary Roc expressions inside template strings.
- Empty checking samples whose bodies read item fields.
- U8 fields, tag unions, and unused unsupported encoder fields.
- Nested Rocci components, Html body parameters, and valueless boolean
  attributes as Mustache forms.
- An untyped prepared `Template` as the public API.
- Rocci byte spans, `RCxxxx` IDs, or LSP source maps.
- Wrapping a string renderer result as a trusted Html body.

Choose **defer**. Do not carry a library candidate into Phase 4. A later
packaging experiment can reopen these API questions; this plan does not.
[^typed-adapter][^follow-up-plan]

### Phase 4 representative host

Executed on 2026-09-12. `--host` copies platform Html only into the work
directory and leaves generated Roc unchanged. Hello (small component), Card
(list), Compat (nested components, void elements, Html body), and Callout
(scoped CSS fragment) are byte-identical between current nodes and
`node_scan_escape`. Theme compilation still sets `html_type: "Str"`; the node
candidate does not apply to painters. Linux was not available (Darwin /
Apple M1 Max).[^host][^phase-4-receipt][^theme]

Candidate HTTP coverage is absent: `rocci build --output` writes a process
binary against the in-tree `rocci` pin and drops the staged workspace, so a
copied `Html.roc` cannot be substituted. `--platform` accepts only `rocci`.
A product-origin smoke of HostPage also failed to listen (`PermissionError`).
Do not treat the basic-cli fixture match as HTTP performance.
[^host][^host-page][^phase-4-receipt][^platform-readme]

Narrow any later runtime change to macOS Node Html first. Do not claim
cross-platform or webview-origin benefit from this phase.

A later [HTTP/Linux coverage](/plans/rocci/html-scan-copy-host-coverage.md)
kept the staged HostPage workspace, retargeted a copied platform, and compared
fold versus scan/copy on `http://127.0.0.1`. `GET /` and `GET /card` were
byte-identical; renderer gain was not visible on that low-load path.
Linux coverage remains absent on Darwin. Keep the product scan/copy plan
narrowed to macOS Node Html and keep repeating the Linux gap. Do not label
those origin timings as HTTP throughput.
[^host-coverage-plan][^host-coverage-phase-2]

### Disposition after the experiment

The restricted-library idea passes the first viability test: type-safe
context binding can be layered over compile-time preparation on the current
pin without a product parser change. Empty samples, the encoder vocabulary,
attribute semantics, and coarse diagnostics remain real limits. Phase 3
defers packaging that library. Compile-time preparation is not the
performance opportunity. Follow-ups are scan/copy Node escape and
`boolean_attribute(False)` omission; neither is started or approved.
Neither result makes embedded Roc expressions executable or establishes a
replacement for `.rocci`.
[^typed-adapter][^experiment-results][^phase-2-receipt][^scan-copy-plan][^boolean-plan]

[^discussion]: Exact linked announcement and archived reply; no documented platform compiler hook.
[^templegen-main]: Main-branch generator, closed Ctx, formatter calls, two strategies, and later benchmark summary.
[^template]: `check`, `render`, `check_nodes`, `lower`, and `render_ops`; sample validation and independent generic render input.
[^value]: `Value.Encoding`, `run_encoder`, cells, offsets, and sorted field positions.
[^formatters]: Explicit Id mapping and `apply` dispatch to ordinary Roc functions.
[^runtime-main]: Raw string imports and top-level template preparation in the example server.
[^benchmarks]: Author-reported September 10 same-run results; not reproduced here.
[^compiler-bug]: Public bug report, open when inspected on 2026-09-12; later compiler compatibility remains unverified.
[^tutorial]: Constant folding runs pure calls over constant inputs where possible.
[^pin]: `roc_nightly = "nightly-2026-09-03-62fcb65"`, also verified by local `roc version`.
[^template-readme]: Public template, CSS, expression, composition, and generated-Roc contracts.
[^template-lib]: Rust-owned compile entry point and CompileOutput metadata.
[^lower-html]: `lower_interpolation` emits original expression spans; structural forms lower to Roc and Html calls.
[^tests]: Existing tests for typed defaults, body arguments, directives, imports, and source remapping were read, not rerun.
[^source-map]: Expression, attribute, directive, signature, CSS, and structural origins.
[^driver]: Generated module staging and extracted stylesheet handling.
[^hosted]: Hosted effects return source/AST strings; do not interpret interpolations.
[^native-research]: Existing source-emitting Roc parser/lowerer investigation and product boundary.
[^native-postmortem]: Historical prototype results and pin-specific limitations.
[^html-research]: Existing performance investigation and exploratory fusion alternatives.
[^pure-render]: Established pure component contract, corroborated against current lowering and tests.
[^experiment]: Isolated runner, pinned source hashes, explicit positive/negative assertions, fixture staging, and measurement procedure.
[^typed-adapter]: Shared sample/renderer type variable and filename-aware error messages.
[^experiment-fixture]: Small Rocci component; no product source change.
[^experiment-results]: Recorded local outputs, expected failures, HTML comparison cases, check/build/runtime samples, and allocation counters.
[^allocation-counter]: Child-only libc interposition; counts calls and requested bytes, not peak live memory.
[^product-html]: `escape_text` versus `escape_attribute`, including newline/carriage-return references in attributes.
[^string-html]: Shared escaping and string-based Html construction used by the comparison app.
[^html-preprocessing]: CRLF and literal CR are normalized before tokenization; character references are decoded later.
[^follow-up-plan]: Separate plan for evidence quality, controlled comparisons, API boundaries, and representative host checks; not a product cutover.
[^scan-copy-plan]: Scan/copy Node escape Phases 0–1 completed locally on `main`; HTTP/Linux/theme unmeasured; not an approved Decision.
[^boolean-plan]: Follow-up to omit `boolean_attribute` when false; not started.
[^host-coverage-plan]: Phases 0–2 local: macOS origin bytes match, no visible renderer gain; Linux absent.
[^host-coverage-phase-2]: Local Darwin receipt; Docker unavailable; origin VPS not used as a compile host.
[^string-theme-plan]: Exploration of split/join string Html for painters; may close with no change.
[^phase-0-receipt]: Phase 0 local baseline receipt; September 12 file preserved; claimed speed ordering reproduced.
[^phase-1-receipt]: Phase 1 local matrix and helper probes; html5lib 1.1; no unexplained benchmarked Card differences.
[^phase-2-receipt]: Phase 2 local cost table; process totals; `node_scan_escape` selected; builder-control beat prepared rendering.
[^phase-4-receipt]: Phase 4 local host receipt; fixture bytes equal; theme `Str`; candidate HTTP and Linux absent.
[^host]: Representative Html copies in the work directory; rocci-cli cannot retarget a copied platform for HTTP.
[^host-page]: HostPage view and fragment routes; product-origin smoke only.
[^costs]: Isolated Html copies in the work directory; generated Card Roc unchanged.
[^platform-readme]: In-tree rocci-platform pin; `libhost.a` is not a kept workspace for this experiment.
[^theme]: `compile_single_module` sets `html_type: "Str"`.
