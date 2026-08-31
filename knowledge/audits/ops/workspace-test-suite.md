---
type: Audit
title: Workspace test-suite review
description: The default Rust suite is broader than the documented sub-second budget; Roc-gated builds run whenever roc is on PATH; several kitchen-sink and CI checks overlap; generated-app HTTP, operator pytest, and the SSE target rule are the main coverage holes.
tags: [domain/rocci, domain/rocdown, concern/testing, concern/ci, concern/validation, audience/maintainer]
status: draft
generated: { by: process:cursor, at: 2026-08-30T20:30:00Z }
stale_after: 2026-11-30
authority: descriptive
owners: [human:nils]
sources:
  - id: agents
    resource: ../../../AGENTS.md
    title: Default suite budget and ignored fuzz/perf commands
    author: process:git
    last_modified: 2026-08-30
  - id: contributing
    resource: ../../../CONTRIBUTING.md
    title: cargo test --workspace as the fast crate suite
    author: process:git
    last_modified: 2026-08-26
  - id: ci-py
    resource: ../../../rocci-ops/src/rocci_ops/ci.py
    title: Canonical CI job bodies
    author: process:git
    last_modified: 2026-08-29
  - id: fuzz
    resource: ../../../crates/rocci-lsp/tests/fuzz_invariants.rs
    title: Default and ignored LSP invariant fuzz
    author: process:git
    last_modified: 2026-08-17
  - id: all-syntax-lsp
    resource: ../../../crates/rocci-lsp/tests/all_syntax.rs
    title: AllSyntax.rocci LSP kitchen-sink test
    author: process:git
    last_modified: 2026-08-25
  - id: server-rs
    resource: ../../../crates/rocci-lsp/tests/server.rs
    title: Rocci language-server integration tests
    author: process:git
    last_modified: 2026-08-26
  - id: compile-template
    resource: ../../../crates/rocci-template/tests/compile.rs
    title: Template compile and AllSyntax golden tests
    author: process:git
    last_modified: 2026-08-25
  - id: compile-rocdown
    resource: ../../../crates/rocci-rocdown/tests/compile.rs
    title: Rocdown compile and AllSyntax golden tests
    author: process:git
    last_modified: 2026-08-25
  - id: cli-e2e
    resource: ../../../crates/rocci-rocdown-cli/tests/cli_e2e.rs
    title: rocdown CLI process tests including docs check and serve
    author: process:git
    last_modified: 2026-08-20
  - id: run-rs
    resource: ../../../crates/rocci-cli/src/run.rs
    title: Roc-gated generated-app builds and HTTP smokes
    author: process:git
    last_modified: 2026-08-30
  - id: build-rs
    resource: ../../../crates/rocci-rocdown/src/build.rs
    title: Rocdown skip_without_roc probe that runs roc build
    author: process:git
    last_modified: 2026-08-22
  - id: suite-plan
    resource: ../../plans/ops/workspace-test-suite.md
    title: Fast default suite and hosted Roc smokes
    author: process:cursor
    last_modified: 2026-08-30
  - id: sse-plan
    resource: ../../plans/rocci/sse-patch-target-tests.md
    title: Planned SSE patch-target wire-format tests
    author: process:cursor
    last_modified: 2026-08-30
  - id: sse-tests
    resource: ../../../crates/rocci-datastar/tests/sse.rs
    title: Existing Datastar SSE unit tests
    author: process:git
    last_modified: 2026-08-30
  - id: python-uv-plan
    resource: ../../plans/ops/python-uv-ops-pipeline.md
    title: rocci-ops plan with pytest as a Phase 2 exit
    author: process:cursor
    last_modified: 2026-08-30
  - id: playground-pkg
    resource: ../../../playground/package.json
    title: Playground frontend scripts; build and check only
    author: process:git
    last_modified: 2026-08-18
  - id: wasm-harness
    resource: ../../../test/wasm/test-phase0-wasm.mjs
    title: Manual WASM playground harness
    author: process:git
    last_modified: 2026-08-21
  - id: roc-backend
    resource: ../../../crates/rocci-lsp/tests/roc_backend.rs
    title: Ignored live roc experimental-lsp child tests
    author: process:git
    last_modified: 2026-08-26
  - id: islands
    resource: ../../../crates/rocci-rocdown-cli/tests/islands.rs
    title: Roc-gated island preview HTTP tests
    author: process:git
    last_modified: 2026-08-22
  - id: playground-server
    resource: ../../../crates/rocci-cli/tests/playground_server.rs
    title: Playground loopback HTTP integration
    author: process:git
    last_modified: 2026-08-25
  - id: ops-tests
    resource: ../../../rocci-ops/tests/test_ci.py
    title: rocci-ops pytest for CI job shapes
    author: process:git
    last_modified: 2026-08-29
  - id: desktop
    resource: ../../../crates/rocci-desktop/src/lib.rs
    title: Thin desktop host wrapper and three unit tests
    author: process:git
    last_modified: 2026-08-26
  - id: vscode-pkg
    resource: ../../../editors/vscode/package.json
    title: VS Code npm test entry
    author: process:git
    last_modified: 2026-08-29
---

# Workspace test-suite review

Measured 2026-08-30 on this checkout. Counts and timings below are from
`cargo test` on an already-built `target/` unless noted. Roc nightly
`2026-08-23-fb208ba` was on `PATH` for the first workspace run and removed
for the timed runs.

This is an audit of current tests. Implementation is the paired
[fast default suite plan](/plans/ops/workspace-test-suite.md).[^suite-plan]

## Inventory

About 990 Rust `#[test]` functions live in the 18 workspace members. Eight
are `#[ignore]`, all in `rocci-lsp`: two deep fuzz cases, three release
latency benches, and three live `roc experimental-lsp` child tests.[^fuzz][^roc-backend][^agents]

Outside Cargo:

- VS Code: Mocha unit tests plus `@vscode/test-electron` integration. CI
  `editors` runs `npm --prefix editors/vscode test`.[^ci-py][^vscode-pkg]
- `rocci-ops`: 95 pytest functions. No CI job runs them.[^ops-tests][^ci-py]
- `playground/`: `build` and `check` only; no test script.[^playground-pkg]
- `test/wasm/*.mjs`: six manual WASM harnesses, not wired to CI.[^wasm-harness]

`rocci-desktop` has three string and type checks around the h35 host
wrapper. That matches a thin crate, not a missing window suite.[^desktop]

## Policy versus measured time

`AGENTS.md` and `CONTRIBUTING.md` say default `cargo test`,
`cargo test -p <pkg>`, and `cargo test --workspace` are structured for
sub-second execution (under 2s), with intensive work behind
`#[ignore]`.[^agents][^contributing]

Already-built, Roc off `PATH`, this machine:

| Command | Wall time | Slowest binary |
| --- | --- | --- |
| `cargo test --workspace` | 17.5s | many crate binaries plus Cargo startup |
| `cargo test -p rocci-rocdown-cli` | 5.5s | `cli_e2e` 5.04s / 10 tests |
| `cargo test -p rocci-rocdown` | 2.1s | lib 0.92s / 205 tests |
| `cargo test -p rocci-lsp` | 1.9s | `fuzz_invariants` 1.44s / 5 default tests |
| `cargo test -p rocci-cli` | 0.8s | lib 0.16s; playground HTTP 0.14s |
| `cargo test -p rocci-template` | 0.6s | `compile` 0.01s / 90 tests |

Parser and lowering binaries are already in the intended budget. The
workspace command is not. Cargo process startup across many crates is most
of the 17s; the other two over-budget owners are `cli_e2e` and the
**default** (not ignored) fuzz binary.[^cli-e2e][^fuzz]

With Roc on `PATH`, the first `cargo test --workspace` compiled generated
apps and left
`/var/folders/.../rocci-roc-build-78908-9/server` running after the Cargo
parent had disappeared. The timed wrapper was still open at 4m+ when
killed. Those Roc-gated tests skip in GitHub Actions because CI never
installs Roc and never sets `ROCCI_REQUIRE_ROC`.[^run-rs][^ci-py]

The `<2s` sentence is design intent for the offline path, not a measured
workspace property, and it is false for any developer who has `roc` on
`PATH`.

## Longer than needed

**Roc-gated tests sit in the default suite.**
`skip_without_roc()` returns early only when `roc help` fails. If Roc is
installed, `rocci-cli` compiles live-counter, counter, handler-matrix, and
multi-page-streams, then runs HTTP smokes that spawn the generated
server. `rocci-rocdown` and `rocci-rocdown-cli` do the same for site
builds and island preview. That is useful coverage, but it is not a
sub-second default path.[^run-rs][^islands]

**The skip probe itself can compile Roc.** In `rocci-rocdown`,
`skip_without_roc` writes a `basic-cli` probe and runs `roc build` before
deciding to skip. Every Roc-gated test in that module pays a compile when
Roc is present, even if the real case would have been skipped for another
reason.[^build-rs]

**`cli_e2e` re-runs product CLI work the fixtures job already does.**
`check_docs_succeeds_in_terminal_and_json` and `test_docs_examples` spawn
`rocdown check docs` and `rocdown test docs` against the full docs tree.
CI `fixtures-and-docs` already runs `rocdown check docs` (and `check
site`). Those two tests dominate the 5s `cli_e2e` binary.[^cli-e2e][^ci-py]

**Default `fuzz_invariants` is still a stress suite.**
`test_multibyte_and_non_bmp_byte_slicing_stress` (stride 8),
`test_truncated_and_malformed_constructs_stress`,
`test_deeply_nested_structures`, and 50-iteration mutation fuzz run on
every `cargo test -p rocci-lsp`. Only the exhaustive stride-1 and
5000-iteration variants are ignored. 1.44s is acceptable for a nightly
gate; it is the whole `rocci-lsp` default budget by itself.[^fuzz][^agents]

**AllSyntax is compiled six times in one file.**
`kitchen_sink_compiles_without_errors` already asserts the full golden
Roc. Five later tests `include_str!` the same fixture and call
`compile_ok` again for subset string checks. Cheap individually; needless
repeat work and a maintenance magnet when the golden moves.[^compile-template]

**CI duplicates workspace tests.** `fixtures-and-docs` runs
`cargo test -p rocci-docs` after the `test` job already ran
`cargo test --workspace`. The four `inspect ast` CLI invocations overlap
library compile and AST tests that already load the same fixtures.[^ci-py][^compile-rocdown]

Playground loopback HTTP (0.14s, 10ms poll) is in budget and is not the
problem.[^playground-server]

## Redundant or overlapping

These are not copy-paste twins. They re-exercise the same fixture at a
second boundary. Keep one owner per claim; drop or shrink the rest.

| Overlap | Owners | What to keep |
| --- | --- | --- |
| AllSyntax.rocci has no error diagnostics and exposes Badge / Hello / CounterPage / `GET /` / `PATCH /actions/patch` | `all_syntax.rs::test_lsp_all_syntax_rocci` and `server.rs::kitchen_sink_has_no_error_diagnostics_and_component_symbols` | `server.rs` already has the richer symbol list (`POST`, `GET /sse`). `all_syntax.rs` adds hover, tokens, and inspect-regions on the same open.[^all-syntax-lsp][^server-rs] |
| AllSyntax + EmbeddedLanguages region trees and token invariants | `server.rs` region/token tests and `fuzz_invariants::test_invariants_on_all_standard_fixtures` | Keep the `server.rs` named cases. The fuzz fixture pass is a third walk of the same files.[^server-rs][^fuzz] |
| AllSyntax.rocdown compiles and inspects | `rocci-rocdown` compile/ast tests, `cli_e2e::inspect_ast_and_roc_on_syntax_fixture`, CI inspect | Library golden is the contract. CLI inspect is a thin argv check. CI inspect is a third parse of the same files.[^compile-rocdown][^cli-e2e][^ci-py] |
| Docs `check` / `test` | `cli_e2e` and CI `fixtures-and-docs` | One process-level docs check is enough if CI keeps the CLI invocation.[^cli-e2e][^ci-py] |

The two `all_syntax.roc` goldens (template vs rocdown) are not redundant.
They are expected output for two languages.[^compile-template][^compile-rocdown]

## Weak or misleading tests

`compile_output_includes_parse_validate_and_lower_timings` only asserts
each timing is under 10 seconds. That does not protect a budget and does
not fail on a hang shorter than the process timeout.[^compile-template]

`desktop::ipc_handler_type_is_send` is a compile-time type check written
as a runtime test. Harmless; it does not exercise the host.[^desktop]

Documented `ROCCI_REQUIRE_ROC=1` is not how the default suite behaves.
Without the env var, Roc-gated tests **run** when Roc is found. The env
var only **panics** when Roc is missing. Contributors who follow
`CONTRIBUTING.md` and put `roc` on `PATH` get the slow path, not the
documented skip.[^run-rs][^contributing][^agents]

## Critically missing

**Generated-app and live-handler path never runs in CI.** Handler-matrix
and multi-page HTTP smokes, island preview, and `rocci-rocdown` `roc
build` cases all skip on the hosted `test` job. A lowering or platform
regression that only fails under `roc build` can merge green.[^run-rs][^islands][^ci-py]

**SSE `datastar-patch-elements` target rule is still untested.**
`rocci-datastar` covers style stripping and selector formatting. Nothing
in the default suite rejects a patch-elements payload that has no
selector and a top-level element without `id`. That is the 2026-08-30
style-sibling incident; a plan already exists and has not started.[^sse-tests][^sse-plan]

**`rocci-ops` pytest is local-only.** Phase 2 of the uv pipeline treated
pytest as an exit criterion. CI `lint` / `test` never invoke
`uv run --group dev pytest` under `rocci-ops`. Operator job-shape
and deploy-client regressions are unguarded on GitHub.[^python-uv-plan][^ops-tests][^ci-py]

**Playground UI and WASM harnesses are manual.** `playground/package.json`
has no `test` script. `test/wasm/*.mjs` require a prebuilt module and are
not a CI job. `rocci-playground` Rust compile/parity tests do not cover
the browser worker or CodeMirror shell.[^playground-pkg][^wasm-harness]

Live `roc experimental-lsp` hovers stay ignored. That is the right
default-suite choice; it is still an unguarded product path if no nightly
or manual job runs `--ignored`.[^roc-backend][^agents]

## What is in good shape

- Parser, lowering, and catalog tests stay in-process and do not boot a
  server. That boundary is holding for `rocci-template` and most of
  `rocci-rocdown`.[^compile-template][^compile-rocdown]
- Heavy fuzz and perf are named and documented; only the shallow fuzz
  leaked into the default `rocci-lsp` run.[^fuzz][^agents]
- VS Code `npm test` is in CI. Zed is `cargo check` plus a manifest
  check, which matches an extension with almost no logic.[^ci-py]
- Knowledge validation is a separate workflow, not mixed into crate
  tests.[^ci-py]
- Kitchen-sink fixtures are shared from `test/AllSyntax.*` rather than
  forked per crate.

## Recommended follow-ups

Highest leverage first. Phases live on the paired plan:[^suite-plan]

1. Gate Roc compile and HTTP smokes on `ROCCI_REQUIRE_ROC=1` (or
   `#[ignore]`), so `cargo test --workspace` matches the documented
   offline budget. Add one CI or nightly job that installs Roc and sets
   the flag.
2. Move default `fuzz_invariants` stress (stride-8 slicing, 50-iteration
   mutation) behind `--ignored` or a smaller default iteration count.
3. Drop or shrink `cli_e2e` docs `check` / `test` if CI keeps those CLI
   invocations; keep the serve-without-roc and RD2302 cases.
4. Fold `all_syntax.rs` into `server.rs` or drop the duplicated symbol
   and diagnostic asserts.
5. Compile AllSyntax once in `rocci-template` tests and share the
   `CompileOutput`.
6. Implement the existing SSE patch-target plan.
7. Run `rocci-ops` pytest from CI `lint` or a small ops job.

[^suite-plan]: Implementation plan for the opt-in Roc gate, hosted Roc job, fuzz/cli_e2e shrink, and pytest.
[^agents]: Default suites claimed under 2s; ignored fuzz and perf commands.
[^contributing]: `cargo test --workspace` documented as the fast crate suite; install steps put `roc` on `PATH`.
[^ci-py]: Hosted test job is `cargo test --workspace` plus `--doc`; fixtures-and-docs re-tests `rocci-docs` and inspects AllSyntax; no Roc and no pytest.
[^fuzz]: Default fuzz binary still runs stride-8 slicing, malformed stress, nesting, and 50 mutations; exhaustive cases are ignored.
[^all-syntax-lsp]: AllSyntax open, no-error diagnostics, symbols, hover, tokens, inspect-regions.
[^server-rs]: Kitchen-sink diagnostics and symbols, plus region and token invariant tests on the same fixtures.
[^compile-template]: Six AllSyntax compiles; golden Roc equality; 10s timing assert.
[^compile-rocdown]: AllSyntax.rocdown compile golden and AST coverage.
[^cli-e2e]: Process-level `check docs`, `test docs`, inspect, serve, and RD2302.
[^run-rs]: `skip_without_roc` uses `roc help`; generated-app builds and HTTP smokes run when Roc exists.
[^build-rs]: Skip helper runs `roc build` on a basic-cli probe.
[^sse-plan]: Planned selector-or-id rule; no phase started.
[^sse-tests]: Style-strip and selector formatting only.
[^python-uv-plan]: Phase 2 exit included pytest for `rocci-ops`.
[^playground-pkg]: Scripts are `build` and `check` only.
[^wasm-harness]: Manual WASM phase script; requires a prebuilt module.
[^roc-backend]: Three ignored live child-hover tests.
[^islands]: Five island tests skip without Roc.
[^playground-server]: Loopback playground HTTP; 10ms poll, 50 attempts.
[^ops-tests]: pytest exists for CI job shapes and other operator modules.
[^desktop]: Three unit tests; no window or IPC runtime.
[^vscode-pkg]: `npm test` compiles, runs unit Mocha, then Electron integration.
