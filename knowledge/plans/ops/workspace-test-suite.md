---
type: Implementation Plan
title: Fast default suite and hosted Roc smokes
description: "Make cargo test --workspace the documented offline path: Roc compiles and HTTP smokes run only with ROCCI_REQUIRE_ROC=1; shrink default fuzz and cli_e2e; drop kitchen-sink and CI overlap; add hosted Roc and rocci-ops pytest. Do not re-plan SSE targets or playground WASM."
tags: [domain/rocci, domain/rocdown, concern/testing, concern/ci, concern/validation, audience/maintainer]
status: draft
generated: { by: process:cursor, at: 2026-08-30T22:40:00Z }
stale_after: 2026-11-30
authority: exploratory
owners: [human:nils]
sources:
  - id: audit
    resource: ../../audits/ops/workspace-test-suite.md
    title: Workspace test-suite review (measured 2026-08-30)
    author: process:cursor
    last_modified: 2026-08-30
  - id: agents
    resource: ../../../AGENTS.md
    title: Documented sub-second suite and ROCCI_REQUIRE_ROC
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
  - id: ci-yml
    resource: ../../../.github/workflows/ci.yml
    title: Hosted lint, test, fixtures-and-docs, editors
    author: process:git
    last_modified: 2026-08-30
  - id: test-ci
    resource: ../../../rocci-ops/tests/test_ci.py
    title: JOB_NAMES and fixtures-and-docs step asserts
    author: process:git
    last_modified: 2026-08-29
  - id: python-uv-plan
    resource: python-uv-ops-pipeline.md
    title: Phase 2 exit included pytest; CI never runs it
    author: process:cursor
    last_modified: 2026-08-30
  - id: run-rs
    resource: ../../../crates/rocci-cli/src/run.rs
    title: skip_without_roc via roc help; generated-app HTTP smokes
    author: process:git
    last_modified: 2026-08-30
  - id: dispatch-rs
    resource: ../../../crates/rocci-cli/src/dispatch.rs
    title: Duplicate skip_without_roc in dispatch tests
    author: process:git
    last_modified: 2026-08-30
  - id: build-rs
    resource: ../../../crates/rocci-rocdown/src/build.rs
    title: skip_without_roc probe that runs roc build
    author: process:git
    last_modified: 2026-08-22
  - id: islands
    resource: ../../../crates/rocci-rocdown-cli/tests/islands.rs
    title: Roc-gated island preview HTTP tests
    author: process:git
    last_modified: 2026-08-22
  - id: fuzz
    resource: ../../../crates/rocci-lsp/tests/fuzz_invariants.rs
    title: Default stride-8 and 50-iteration stress
    author: process:git
    last_modified: 2026-08-17
  - id: all-syntax-lsp
    resource: ../../../crates/rocci-lsp/tests/server.rs
    title: Kitchen-sink diagnostics, symbols, hover, tokens, inspect-regions
    author: process:git
    last_modified: 2026-08-30
  - id: server-rs
    resource: ../../../crates/rocci-lsp/tests/server.rs
    title: Richer kitchen-sink symbols and region tests
    author: process:git
    last_modified: 2026-08-26
  - id: compile-template
    resource: ../../../crates/rocci-template/tests/compile.rs
    title: Repeated AllSyntax compiles and 10s timing assert
    author: process:git
    last_modified: 2026-08-25
  - id: compile-rocdown
    resource: ../../../crates/rocci-rocdown/tests/compile.rs
    title: AllSyntax.rocdown library golden
    author: process:git
    last_modified: 2026-08-25
  - id: cli-e2e
    resource: ../../../crates/rocci-rocdown-cli/tests/cli_e2e.rs
    title: Process-level check docs, test docs, inspect, serve, RD2302
    author: process:git
    last_modified: 2026-08-20
  - id: install-roc
    resource: ../../../docker/install-roc.sh
    title: Pinned Linux Roc nightly installer
    author: process:git
    last_modified: 2026-08-25
  - id: sse-plan
    resource: ../rocci/sse-patch-target-tests.md
    title: Planned SSE patch-target wire-format tests
    author: process:cursor
    last_modified: 2026-08-30
  - id: embedded-lsp
    resource: ../rocci/embedded-roc-lsp-parity.md
    title: Live roc experimental-lsp child tests
    author: process:cursor
    last_modified: 2026-08-25
  - id: public-ci
    resource: public-ci-security.md
    title: Job bodies stay in rocci-ops; hosted /ci lane
    author: process:cursor
    last_modified: 2026-08-22
  - id: playground-pkg
    resource: ../../../playground/package.json
    title: Playground scripts are build and check only
    author: process:git
    last_modified: 2026-08-18
---

# Fast default suite and hosted Roc smokes

The [workspace test-suite review](/audits/ops/workspace-test-suite.md)
measured an already-built `cargo test --workspace` at 17.5s without Roc,
and a hang past 4 minutes with Roc on `PATH`. Documented
`ROCCI_REQUIRE_ROC=1` only panics when Roc is missing; it does not keep
Roc compiles out of the default suite. Hosted CI never installs Roc and
never runs `rocci-ops` pytest.[^audit][^run-rs][^ci-py]

Exploratory. Do not start a phase until the user asks.

## Goal

Default `cargo test`, `cargo test -p <pkg>`, and
`cargo test --workspace` stay **offline**: no `roc build`, no generated
server, no docs-tree CLI, no LSP stress. Parser and lowering crates stay
in the documented per-binary budget. One hosted Linux job installs the
pinned Roc nightly and sets `ROCCI_REQUIRE_ROC=1` so handler-matrix,
islands, and site `roc build` cannot merge green on a skip. `rocci-ops`
pytest runs from CI `lint`.[^agents][^audit]

## Current contract (must change)

`skip_without_roc` in `rocci-cli`, `rocci-rocdown`, and
`rocci-rocdown-cli` returns early only when `roc help` fails. If Roc is
on `PATH`, generated-app HTTP smokes and island preview **run**. The
`rocci-rocdown` helper also writes a `basic-cli` probe and runs
`roc build` before every gated test in that module.[^run-rs][^build-rs][^islands]

`ROCCI_REQUIRE_ROC=1` only changes the missing-Roc path from skip to
panic. Contributors who follow the contributing install steps and put
`roc` on `PATH` get the slow path, not the documented skip.[^contributing][^agents]

Hosted `test` is `cargo test --workspace` plus `--doc` with no Roc.
`fixtures-and-docs` re-runs `cargo test -p rocci-docs` and four
`inspect ast` CLIs after the workspace job. `lint` never invokes
pytest.[^ci-py][^ci-yml][^python-uv-plan]

## Out of bound

- Implementing [SSE patch-target tests](/plans/rocci/sse-patch-target-tests.md).
  That plan owns the wire-format rule. Cite it; do not copy its phases.
- Playground CodeMirror / worker tests, `playground/package.json` `test`
  script, and wiring `test/wasm/*.mjs` into CI.[^playground-pkg]
- Un-ignoring `roc_backend` live `roc experimental-lsp` child tests.
  That path belongs to [embedded Roc LSP parity](/plans/rocci/embedded-roc-lsp-parity.md).[^embedded-lsp]
- A desktop window or IPC runtime suite for `rocci-desktop`.
- Making `cargo test --workspace` wall time itself sub-second. Cargo
  process startup across 18 members is most of the measured 17.5s; this
  plan does not merge test binaries or adopt nextest.[^audit]
- Changing hosted `/ci` authorization, runners, or Environment secrets.
  New job bodies stay in `rocci-ops ci`.[^public-ci]
- Installing Roc on `macos-latest`. `install-roc.sh` is Linux-only.[^install-roc]
- Tangled / spindle CI, Dependabot, or required status checks on every PR.
- Rewriting the audit's measured timings as if they were this plan's
  exit.

## Constraints that do not move

1. **Opt-in Roc, not PATH Roc.** After Phase 1, gated tests skip unless
   `ROCCI_REQUIRE_ROC=1`. The env var is the require switch, not a
   panic-only overlay. `#[ignore]` is reserved for exhaustive fuzz, perf,
   and live `experimental-lsp`.[^agents][^run-rs]
2. **One owner per claim.** Library golden owns AllSyntax compile.
   `server.rs` owns kitchen-sink diagnostics and symbols. CI owns
   process-level `rocdown check docs` / `check site`. Do not keep a
   second process that only repeats those asserts.[^compile-template][^server-rs][^cli-e2e]
3. **Parser and catalog tests stay in-process.** Do not boot a server or
   call `roc` to prove parse, lower, or catalog.[^compile-template][^compile-rocdown]
4. **Job bodies stay in `rocci-ops`.** Workflow YAML chooses triggers and
   runners only. `JOB_NAMES` and `test_ci.py` change in the same
   commit as the job list.[^ci-py][^test-ci][^public-ci]
5. **Pinned nightly.** Hosted Roc uses `docker/install-roc.sh`
   (`nightly-2026-08-26-b29bef3`). Do not float `roc` from the runner
   image.[^install-roc]
6. **HTTP smokes reap children.** A generated server must not outlive
   the test process. Phase 6 fails if the leftover-server hang from the
   audit can still happen.[^audit][^run-rs]
7. **Keepalives and Datastar policy stay out of this plan.** Do not
   grow SSE awareness in the parser to "make tests faster."[^sse-plan]

## Target suite matrix

| Lane | Command | Roc | What it proves |
| --- | --- | --- | --- |
| Default / hosted `test` | `cargo test --workspace` and `--doc` | Off | In-process crates, playground loopback HTTP, serve-without-roc, RD2302 |
| Hosted `lint` | current lint steps plus `uv run --group dev pytest` under `rocci-ops` | Off | Operator job shapes and deploy-client units |
| Hosted `fixtures-and-docs` | `check docs`, `check site`, example-docs stage; no workspace re-test, no inspect AST | Off | Product CLI on the docs tree |
| Hosted `roc` (new, Linux) | `ROCCI_REQUIRE_ROC=1` cargo test on the gated packages | Pinned nightly | Generated-app HTTP, islands, `rocci-rocdown` `roc build` |
| On demand | `cargo test -p rocci-lsp --test fuzz_invariants -- --ignored` | Off | Exhaustive stride-1 and 5000-iteration fuzz |

## Phase 1 — Opt-in Roc gate

**Bound:** Change every `skip_without_roc` (and the `rocci-cli`
`native_target` / `playground_html` / `rocci_test` twins) so the first
check is `ROCCI_REQUIRE_ROC=1`. If the var is unset or not `1`, skip
even when `roc help` succeeds. If it is `1` and `roc help` fails,
panic as today.

In `rocci-rocdown` `build.rs`, drop the `basic-cli` `roc build` probe
from the skip helper. A require-roc failure is a real test failure, not
a skip. Do not add a new shared crate for the helper; keep the copies
in the owning modules and make the contract identical.[^dispatch-rs]

Do not add a CI Roc job. Do not `#[ignore]` the HTTP smokes (the env
var is the gate).

**Exit:** With Roc on `PATH` and `ROCCI_REQUIRE_ROC` unset,
`cargo test -p rocci-cli -p rocci-rocdown -p rocci-rocdown-cli` does
not invoke `roc build` and does not spawn a generated server. With
`ROCCI_REQUIRE_ROC=1` and Roc missing, those packages panic on a gated
test. `cargo fmt --all -- --check`.

## Phase 2 — Default fuzz off the hot path

**Bound:** In `fuzz_invariants.rs`, mark
`test_multibyte_and_non_bmp_byte_slicing_stress`,
`test_truncated_and_malformed_constructs_stress`,
`test_deeply_nested_structures`, and
`test_deterministic_mutation_fuzzing` with `#[ignore]` and the same
on-demand command already used for the exhaustive variants.[^fuzz] Keep
`test_invariants_on_all_standard_fixtures` as the default cheap walk,
or shrink it to a single fixture if it still dominates. Do not change
`perf` benches.

**Exit:** `cargo test -p rocci-lsp --test fuzz_invariants` finishes
well under 1s on an already-built tree. The four stress cases still
compile and run under `--ignored`. `cargo fmt --all -- --check`.

## Phase 3 — Shrink `cli_e2e`

**Bound:** Remove `check_docs_succeeds_in_terminal_and_json` and
`test_docs_examples` from `cli_e2e.rs`. Keep
`serve_built_tree_returns_html_without_roc`, both RD2302 cases,
`inspect_config_reads_rocdown_toml`, catalog/graph/artifacts inspect,
and `build_single_rocdown_file_writes_roc`. Keep
`inspect_ast_and_roc_on_syntax_fixture` as the one process-level argv
check; do not add another docs-tree spawn.

**Exit:** `cargo test -p rocci-rocdown-cli --test cli_e2e` no longer
runs `rocdown check docs` or `rocdown test docs`.
`cargo fmt --all -- --check`.

## Phase 4 — One AllSyntax compile and one LSP kitchen-sink

**Bound:** In `rocci-template` `compile.rs`, compile `AllSyntax.rocci`
once (a `OnceLock` / helper that returns `&CompileOutput`) and reuse
it for the golden and the subset string checks. Do not merge unrelated
small `compile_ok` cases into that helper.

Replace `compile_output_includes_parse_validate_and_lower_timings`
10-second asserts with "fields are present and finite" (or drop the
wall-clock asserts and keep the field reads).

Fold `all_syntax.rs` hover, tokens, and inspect-regions into
`server.rs` next to
`kitchen_sink_has_no_error_diagnostics_and_component_symbols`.[^all-syntax-lsp]
Delete `all_syntax.rs`. Do not drop `POST` / `GET /sse` from the richer
symbol list.

**Exit:** `cargo test -p rocci-template --test compile` and
`cargo test -p rocci-lsp --test server`. `crates/rocci-lsp/tests/all_syntax.rs`
is gone. `cargo fmt --all -- --check`.

## Phase 5 — Pytest in lint; drop CI overlap

**Bound:** Add `uv run --group dev pytest` with cwd
`rocci-ops` to the `lint` job in `ci.py`. Do not put pytest on
`--no-dev`. Remove from `fixtures-and-docs`: the four `inspect ast`
CLI steps and `cargo test -p rocci-docs`. Keep `check docs`,
`check site`, `rocci-ops check docs`, and the example-docs stage.

Update `test_ci.py` so `JOB_NAMES` is unchanged and the fixtures
asserts no longer require `cargo test -p rocci-docs` or inspect AST.[^test-ci]

**Exit:** `uv run --group dev pytest` under `rocci-ops`.
`uv run --no-dev rocci-ops ci --list` still prints the five current
job names. `cargo fmt` is not required unless Rust changed.

## Phase 6 — Hosted Linux Roc job

**Bound:** Add `roc` to `JOB_NAMES`. Steps: run
`docker/install-roc.sh`, then
`ROCCI_REQUIRE_ROC=1 cargo test -p rocci-cli -p rocci-rocdown -p rocci-rocdown-cli`
(plus any other package whose default suite still contains a
`skip_without_roc` call after Phase 1). Do not set the flag on the
existing `test` job. Do not run `--workspace` under the flag.

Add a `roc` job to `.github/workflows/ci.yml` on `ubuntu-latest` only,
same checkout/pin pattern as `lint`. Invoke
`uv run --no-dev rocci-ops ci roc`.

Gated HTTP tests must kill the child server and wait; a leftover
`server` process after the Cargo parent exits is a Phase 6 bug.

**Exit:** `test_ci.py` lists `roc` and asserts the install script plus
`ROCCI_REQUIRE_ROC=1`.[^test-ci] `uv run --group dev pytest` under
`rocci-ops`. Hosted `/ci` on this revision runs the new job (do
not log the phase complete until that CI run is green).

## Phase 7 — Document the two lanes

**Bound:** Rewrite the AGENTS "Validate proportionally" and
CONTRIBUTING "Getting started" sentences so they match Phase 1: default
workspace is offline; `ROCCI_REQUIRE_ROC=1` is the hosted/on-demand Roc
lane; ignored fuzz/perf commands stay as they are after Phase 2. Add
the `roc` job to the devops skill job list if that list names `lint` /
`test` / `fixtures-and-docs`. One sentence on the crate README that
owns `skip_without_roc` if it still says "runs when roc is on PATH."

Point this plan and the audit at each other if any remaining
"recommendations" line is still open. Do not restate the 17.5s
measurement as a new audit.

**Exit:** `okmate check knowledge --profile base --format terminal`.
`cargo fmt --all -- --check` if Rust comments changed.

## Tests

Phases 1–4 are crate tests and `cargo fmt`. Phase 5 is pytest plus the
lint job body. Phase 6 is pytest plus a hosted CI run. Phase 7 is
knowledge check. Do not set `ROCCI_REQUIRE_ROC=1` on default workspace
exits.

## Related plans

- [SSE patch-target tests](/plans/rocci/sse-patch-target-tests.md) —
  still the owner of the untested `datastar-patch-elements` target
  rule. Execute it separately; this plan does not start those phases.
- [Python and uv operator pipeline](/plans/ops/python-uv-ops-pipeline.md)
  — Phase 2 already required pytest locally; Phase 5 of this plan puts
  it on hosted `lint`.
- [Embedded Roc LSP parity](/plans/rocci/embedded-roc-lsp-parity.md) —
  owns live `experimental-lsp` child tests. Leave them `#[ignore]`.

[^audit]: Measured 17.5s workspace without Roc; Roc-on-PATH hang; skip-on-PATH semantics; CI holes.
[^agents]: Default suites claimed under 2s; `ROCCI_REQUIRE_ROC=1` documented as the require switch.
[^contributing]: `cargo test --workspace` as the fast crate suite; install steps put `roc` on `PATH`.
[^ci-py]: Hosted jobs are lint, test, fixtures-and-docs, editors, knowledge; no Roc; no pytest.
[^ci-yml]: Hosted workflow invokes `rocci-ops ci` per job; test is ubuntu and macos.
[^test-ci]: `JOB_NAMES` and fixtures-and-docs step contract; pytest is local-only until Phase 5.
[^python-uv-plan]: Phase 2 exit included pytest; hosted lint never gained that step.
[^run-rs]: `skip_without_roc` uses `roc help`; smokes run when Roc exists.
[^dispatch-rs]: Same skip helper copied into dispatch tests.
[^build-rs]: Skip helper compiles a basic-cli probe before deciding.
[^islands]: Five island tests skip only when `roc help` fails.
[^fuzz]: Default fuzz binary still runs stride-8, malformed stress, nesting, and 50 mutations.
[^all-syntax-lsp]: AllSyntax open plus hover, tokens, inspect-regions.
[^server-rs]: Kitchen-sink diagnostics and the richer symbol list.
[^compile-template]: Six AllSyntax compiles; 10s timing assert.
[^compile-rocdown]: AllSyntax.rocdown library golden is the compile contract.
[^cli-e2e]: Process-level docs check/test dominate the binary; serve and RD2302 stay.
[^install-roc]: Linux pinned nightly installer for the hosted Roc job.
[^sse-plan]: Target-rule plan; not started; out of bound here.
[^embedded-lsp]: Live child-hover path; keep ignored in the default suite.
[^public-ci]: Job bodies in `rocci-ops`; YAML is triggers and runners.
[^playground-pkg]: No frontend test script; out of bound.
