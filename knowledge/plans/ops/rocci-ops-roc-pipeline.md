---
type: Implementation Plan
title: Complete the Roc operator pipeline on rocci-ops-roc
description: Complete the Roc port while preserving the current rocci-ops CLI, step plans, and workflow structure, with an explicit comparison of roc-pandoc alternatives and their implications.
tags: [domain/ops, domain/rocci, integration/roc, concern/ci, concern/tooling, concern/publication]
status: draft
generated: { by: process:cursor, at: 2026-09-30T11:41:53Z }
stale_after: 2026-12-30
authority: exploratory
owners: [human:nils]
sources:
  - id: pandoc-modules
    resource: https://github.com/lukewilliamboswell/roc-pandoc/tree/465cc4a72e1cebc7f748f6bb37f45a0a699fcc68/scripts/src
    title: Shared Script, Project, Files, Release, and UpdatePins modules
    author: process:git
  - id: pandoc-check
    resource: https://github.com/lukewilliamboswell/roc-pandoc/blob/465cc4a72e1cebc7f748f6bb37f45a0a699fcc68/scripts/check_all.roc
    title: Executable Roc checks with separate tooling and product compilers
    author: process:git
  - id: pandoc-goldens
    resource: https://github.com/lukewilliamboswell/roc-pandoc/blob/465cc4a72e1cebc7f748f6bb37f45a0a699fcc68/scripts/test_goldens.roc
    title: Golden comparison and bounded child-process batches
    author: process:git
  - id: pandoc-bundle
    resource: https://github.com/lukewilliamboswell/roc-pandoc/blob/465cc4a72e1cebc7f748f6bb37f45a0a699fcc68/scripts/test_bundle.roc
    title: Testing the exact candidate archive through a temporary local URL
    author: process:git
  - id: pandoc-release
    resource: https://github.com/lukewilliamboswell/roc-pandoc/blob/465cc4a72e1cebc7f748f6bb37f45a0a699fcc68/.github/workflows/release.yml
    title: Build, test-bundles, aggregate validation, then protected publication
    author: process:git
  - id: pandoc-setup
    resource: https://github.com/lukewilliamboswell/roc-pandoc/blob/465cc4a72e1cebc7f748f6bb37f45a0a699fcc68/.github/actions/setup-roc/action.yml
    title: Separate compiler provisioning with Python pin extraction
    author: process:git
  - id: pandoc-ci
    resource: https://github.com/lukewilliamboswell/roc-pandoc/blob/465cc4a72e1cebc7f748f6bb37f45a0a699fcc68/.github/workflows/ci.yml
    title: Workflow directly invoking the executable check_all Roc script
    author: process:git
  - id: prototype-plan
    resource: ./rocci-ops-roc.md
    title: Historical parallel exercise and its limited parity gate
    author: process:cursor
  - id: postmortem
    resource: ../../research/ops/rocci-ops-roc-postmortem.md
    title: Compiler workarounds and incomplete operator execution
    author: process:cursor
  - id: nightly
    resource: ../../research/rocci/roc-nightly-2026-09-18.md
    title: New nightly validation, opportunities, and limits
    author: process:cursor
  - id: branch
    resource: https://github.com/koliyo/rocci/tree/26ea669272d534976312f76ce25f5efa5687ee7a/roc/rocci-ops
    title: Verified September 3 Roc branch snapshot
    author: process:git
  - id: app
    resource: ../../../roc/rocci-ops/app.roc
    title: Existing Roc effect runner, dispatch, and unimplemented commands
    author: process:git
  - id: roc-ci
    resource: ../../../roc/rocci-ops/Ci.roc
    title: Prototype job plans with Python subprocess dependencies
    author: process:git
  - id: parity
    resource: ../../../roc/rocci-ops/parity.sh
    title: Four-case prototype parity harness
    author: process:git
  - id: cli
    resource: ../../../rocci-ops/src/rocci_ops/cli.py
    title: Current complete operator command surface
    author: process:git
  - id: ci
    resource: ../../../rocci-ops/src/rocci_ops/ci.py
    title: Current job plans, process semantics, and okmate resolution
    author: process:git
  - id: tests
    resource: ../../../rocci-ops/tests
    title: Existing Python behavioral test oracles
    author: process:git
  - id: deps
    resource: ../../../rocci-ops/src/rocci_ops/workspace_deps.py
    title: Current dependency classifications and allowed reverse edge
    author: process:git
  - id: site
    resource: ../../../rocci-ops/src/rocci_ops/site.py
    title: Playground, example staging, live packaging, and dev backend policy
    author: process:git
  - id: release
    resource: ../../../rocci-ops/src/rocci_ops/release.py
    title: Release version worktree, existing-CI wait, and tag semantics
    author: process:git
  - id: promote
    resource: ../../../rocci-ops/src/rocci_ops/promote.py
    title: Implemented staging merges and production ref push
    author: process:git
  - id: readme
    resource: ../../../README.md
    title: Published operator contract and staging wording
    author: process:git
  - id: archive
    resource: ../../../rocci-ops/src/rocci_ops/archive.py
    title: Release archives, platform bundle, libhost merge, and checksums
    author: process:git
  - id: deploy
    resource: ../../../rocci-ops/src/rocci_ops/deploy.py
    title: Python origin kit and SSH publishing contract
    author: process:git
  - id: origin
    resource: ../../../rocci-ops/src/rocci_ops/origin.py
    title: Origin publish, health, rollback, and backup behavior
    author: process:git
  - id: setup
    resource: ../../../.github/actions/setup-roc/action.yml
    title: Pinned compiler provisioning from .roc-version
    author: process:git
  - id: workflows
    resource: ../../../.github/workflows
    title: Current CI, Knowledge, release, site, and comment dispatch workflows
    author: process:git
  - id: bootstrap
    resource: ../../../docker/prod/README.md
    title: Origin installation and separate deployment lanes
    author: process:git
  - id: instructions
    resource: ../../../AGENTS.md
    title: Workspace classification and validation instructions to update at cutover
    author: process:git
---

# Complete the Roc operator pipeline on rocci-ops-roc

## Goal

Make ordinary Roc the working implementation of the existing `rocci-ops`
operator: preserve its command tree, `Ci` step plans, workflow job structure,
and local/release/deploy behavior while replacing Python implementation with
Roc modules behind `app.roc`. CI can execute that app as a Roc script; the
origin receives a native build of the same operator. Python remains
the temporary comparison oracle during implementation; it is removed from
the active pipeline only when the corresponding Roc behavior is proven.
The final delivery includes retirement of the Python ops package, not just
a Roc command that launches it.[^cli][^ci]

This is a successor to the parallel exercise. Its earlier “Python stays” and
“no workflow cutover” constraints describe that prototype's scope; they are
not the target of this requested plan. The post-mortem remains historical
evidence. All phases below are planned, not executed or hosted-CI complete.
No approved architecture Decision is created by this document.[^prototype-plan][^postmortem]

## Starting evidence

On September 30, GitHub and the local remote ref both identify
`rocci-ops-roc` at `26ea669272d534976312f76ce25f5efa5687ee7a`. That revision
is an ancestor of local `main`; the Roc source tree also exists in the current
checkout and has no diff against that branch snapshot. Updating the old branch
must reconcile newer product and workflow behavior, not replay the prototype
as if it were still current.[^branch][^app][^roc-ci]

The prototype's parity harness compares only help, check help, CI job names,
and dependency checking. Its `Ci.roc` still calls `uv` for ops checks and
pytest; knowledge always builds sibling okmate with Cargo; the Roc lane
unconditionally installs Roc and omits the current platform build/bundle
sequence. Passing that harness cannot establish a working replacement.
Current Python CI prefers installed okmate, conditionally provisions Roc,
and builds and packages the platform around the gated tests.[^parity][^roc-ci][^ci]

`app.roc` has placeholders for installation, several packaging and serve
paths, `check zed`, most archive operations, non-dry-run release, and deploy
and origin execution. Its release dry run does not resolve/check release
files as the current Python implementation does. The new plan must audit
every command rather than treating parsing or help as implementation.
Staging promotion has an existing documentation discrepancy: Python and
the Roc prototype use merges, while the root README describes a rebase.
Preserve implemented semantics for parity and reconcile the documentation
explicitly; changing promotion policy is not an incidental port fix.
[^app][^cli][^release][^promote][^readme]

The September 18 compiler was validated against Rocci, not this operator
app. Empty nominal defaults, annotated polymorphic defaults, indexed
iterators, and automatic dev object caching are promising. Fully inferred
defaults still have limits. The August compiler-isolation workarounds must
be retested individually; none is declared fixed merely because the
compiler is newer.[^nightly][^postmortem]

## Compare approaches: existing rocci-ops and roc-pandoc

Read the requested `scripts/src` tree and its callers at immutable revision
`465cc4a72e1cebc7f748f6bb37f45a0a699fcc68` on September 30. These are
observed source patterns; its workflows were not executed here. The migration
baseline remains existing rocci-ops, as requested. The comparison distinguishes
implementation techniques we can borrow from changes to the operator model.
[^pandoc-modules][^pandoc-ci][^cli][^ci]

| Existing rocci-ops baseline | roc-pandoc approach | Difference, implication, and plan choice |
| --- | --- | --- |
| One CLI dispatches command families; the Roc prototype already has `app.roc` and modules. | Multiple executable apps such as `check_all.roc`, `build_bundle.roc`, and `update_example_pins.roc` import shared `src` modules. | Per-task apps make each workflow operation visible but add entry apps, package headers, and inputs to maintain. Keep one CLI and its command modules; no required scripts directory or second dispatch surface. |
| `ci.py` and `Ci.roc` plan named jobs as ordered steps; Actions retains six validation jobs. | CI calls one aggregate `check_all.roc`, whose driver orders tooling, goldens, package, examples, artifacts, and docs. | A procedural aggregate is simple for a small package; Rocci needs its existing job selection, step inspection, fail-fast/keep-going behavior, and job/check names. Keep steps as data and current job boundaries. |
| Python's runner handles argv, cwd, environment, redirects, and failures; the Roc prototype's effects are incomplete. | `Script.Command` centralizes argv, child cwd, capture, spawn/wait, environment helpers, and output labels. | Borrow the effects boundary and probe these APIs. Its capture helper returns stdout on success and reports stderr on failure; Rocci still needs successful stderr, exact exits, streaming redirects, and reliable cleanup. |
| Existing modules encode repository paths, discovery, version and archive rules. | `Project` validates a workspace; `Files` sorts paths; `Release` validates values; `UpdatePins` computes edits before writing. | Borrow validated context and pure edit planning within existing owners. Preserve Rocci's path/Unicode and `dev`/`v*` contracts; do not import its ASCII restriction or app-source substring heuristic. |
| `.roc-version` and setup select one product/compiler pin; the prototype uses basic-cli 0.22.0. | Apps use basic-cli 0.23.0-rc1; `ROC_STABLE` runs automation and `ROC_NIGHTLY` tests the product. | Separate pins isolate automation from candidate compiler failures but add provisioning, compatibility, cache, and maintenance cost. Keep one pin for initial parity; document a split as a later option if candidate testing demonstrates a need. Platform upgrade is a separate capability decision. |
| Current Python tests, CI plans, and packaging assertions are the behavioral oracle. | Pure expects, golden bytes, temporary copies, and bounded spawn/wait batches validate scripts and outputs. | Expand Roc pure/effect parity tests and temporary fixture use. Parallel batches can reduce time but introduce child ownership and early-failure cleanup; preserve current ordering initially. |
| Existing archive and release commands run inside established workflow gates. | Release builds assets, tests exact downloaded bundles, aggregates validation, then publishes in a protected job. | Borrow exact-asset verification inside current packaging/release stages. Preserve Rocci's graph and gate behavior during the port; a new matrix or aggregate job would be a separately reviewed workflow change. |
| Origin bootstrap currently ships Python ops and uv metadata. | Reference scripts execute on provisioned compiler runners; they do not demonstrate Rocci's origin deployment. | Rocci needs a native Linux origin operator without a compiler. This packaging difference is necessary for Python retirement and must be tested independently. |

The reference setup itself uses Python to extract compiler pins, and release
steps delegate some work to external actions. Copying that setup would not
meet Python retirement or implement Rocci's policy. Keep the current simple
pin/bootstrap model, use authoritative subprocess argument lists, and compare
0.22.0 versus 0.23.0-rc1 only where an effect capability is missing. Every
adopted difference belongs in the coverage ledger with its rationale, tests,
and bootstrap/runtime implications; unrelated architecture changes stay out
of the parity migration.[^pandoc-check][^pandoc-goldens][^pandoc-bundle][^pandoc-release][^pandoc-setup][^app]

## Out of bound

- Changing Rocci/Rocdown grammar, parser ownership, or product behavior to
  make the operator port easier.
- Replacing Cargo, npm, Git, gh, Docker Compose, curl, SSH, or okmate with
  newly implemented equivalents.
- Installing Roc, Rust, or product build toolchains on the origin VPS.
- Removing necessary container PID 1, compiler-install, or SSH ProxyCommand
  shims; embedding pipeline business logic in those shims.
- Promoting production, pushing release tags, deploying to a live origin,
  rewriting remote branch history, or merging this plan as an implementation
  action. These belong to later execution and its existing authorization.
- Requiring a general-purpose new Roc TOML/regex package or a Rust/Python
  orchestration host as a prerequisite.
- Removing the site dev-backend workaround without Linux target evidence.

## Constraints that do not move

1. Work on the existing `rocci-ops-roc` line in an isolated checkout during
   execution. Preserve current unrelated edits. Fetch the branch and current
   main explicitly, then advance/reconcile locally; do not force-push an
   already published line as a convenience.
2. Keep `.roc-version` as the compiler source of truth, initially
   `nightly-2026-09-18-1d982dc`, and validate the imported operator app on it.
   Start with basic-cli 0.22.0; compare the reference's 0.23.0-rc1 only for
   demonstrated effect capabilities. Any platform upgrade is a separate
   measured change. Explicit executable paths prevent accidental old-PATH
   use. A separate automation compiler pin is a documented option, not a
   prerequisite or an automatic part of this port.[^setup][^app][^nightly][^pandoc-check][^pandoc-setup]
3. Pipeline planning, dispatch, policy, parsing, retries, and exit decisions
   live in ordinary `.roc`. External tools are child commands with argument
   lists. Minimal bootstrap commands may provision and compile the operator
   before it can execute itself; that necessary bootstrap is documented.
   Actions retains scheduling, credentials, artifacts, and Environment gates;
   Roc owns the repository operation each job performs.[^pandoc-ci][^pandoc-release]
4. Preserve child exit codes, stderr, output files, cwd, environment, and
   ordering. Unknown commands fail with the existing usage semantics;
   unsupported migrated commands never report success.[^cli][^ci]
5. Preserve hosted PR snapshot-SHA checkout and credentials policy,
   `/ci` authorization, deployment Environment boundaries, staging/production
   separation, rolling `dev` behavior, and immutable `v*` defaults. Keep the
   release's existing-CI wait separate from Knowledge validation; do not
   introduce a release/bootstrap dependency cycle.[^workflows][^release][^bootstrap]
6. The workspace test suite remains offline with respect to generated Roc
   apps. Bootstrap compilation of the pipeline binary is an explicit earlier
   step or direct script launch, not an implicit compiler invocation inside
   `cargo test --workspace`.
   Only the dedicated Roc lane sets `ROCCI_REQUIRE_ROC=1`.[^ci]
7. During migration, Python tests and command plans are fixed oracles for
   established behavior. Intentional corrections receive their own rationale
   and regression; do not adjust both implementations to conceal differences.
   After retirement, Roc tests and public references own the contract.[^tests]

## Target execution model

Keep `roc/rocci-ops/app.roc` as the single CLI entry and the existing
command-family modules as the owners. `Ci` still returns ordered step data;
a tested effects module executes it. Extract effects and common validated
context where they are shared, without reorganizing the source tree or
adding a parallel collection of per-task apps as a migration requirement.
Local use, Actions, and the native origin build call the same dispatch and
policy.[^app][^roc-ci][^cli][^ci]

For Actions, preserve each current job and replace its Python ops invocation
with the corresponding invocation of this Roc app. Minimal setup provisions
the checked-in compiler/platform before the app can run. The initial shape
is direct execution through an explicit compiler executable:

```sh
# Setup resolves ROC to an absolute executable from .roc-version.
"$ROC" roc/rocci-ops/app.roc -- ci lint
"$ROC" roc/rocci-ops/app.roc -- ci --list
```

Verify this syntax and exact CLI arguments in Phase 0 against the chosen
compiler. Workflow YAML continues to own scheduling, permissions, artifacts,
runner selection, and Environment gates; Roc owns the same repository
operations currently implemented in Python. Nested ops calls use shared
modules/internal dispatch, not a second set of script-specific policies.
No Python or uv is needed solely to launch migrated ops. Provisioning Roc
for automation is an explicit setup cost even in a job whose Rust tests
remain offline with respect to generated Roc applications.[^setup][^workflows]

Direct invocation is the simple launch baseline, not a requirement to compile
on every command forever. Measure cold/warm script startup and a locally
compiled CLI before selecting caching or shared build artifacts. A native
build is the same `app.roc`, not a new orchestration layer. Do not require a
new bootstrap job/artifact dependency graph for the first working port.
If reuse is justified later, key all imported sources, compiler/platform,
target and optimization inputs; match OS/architecture explicitly and accept
artifacts only from the authorized workflow and source revision. Start with
dev; optimize only after behavioral parity.[^nightly][^workflows]

The origin receives a separately tested Linux `x64musl` native operator and
manifest with source revision, compiler/platform identity, target, and hash.
It runs without Python, uv, or Roc on the VPS. Verify the exact candidate
archives and their installed runtime within the existing packaging/release
stages. This required origin delivery change does not imply a redesign of
hosted CI.[^bootstrap][^pandoc-bundle]

A tooling/product compiler split remains a compared alternative. It allows
a validated automation compiler to report a failing product candidate, at
the cost of two pins/download paths and compatibility gates. With one pin,
a broken compiler may prevent the driver itself from starting; fail clearly
at setup/driver compilation and use the last validated pin to diagnose it.
Only add a split after recording that practical need and how overrides,
cache keys, and pin updates work.[^pandoc-check][^pandoc-setup]

## Phase 0: Reconcile the branch and establish the coverage ledger

**Bound:** Update the branch against current main and bring the validated
nightly pin/required compatibility repair into that revision. Inventory
every current Python command, option, job plan, workflow invocation,
origin-kit file, and test. Classify the existing Roc implementation as
complete, partial, stub, or drifted. Record the branch/base SHAs and tool
versions in new implementation evidence.

**Tests:** Run the current Python ops tests, then Roc module/app tests,
the four-case historical harness, native build/help, and an initial Linux
binary smoke. Probe old `Ok`/`Err`, multiple-union, Version import, reserved
field, and self-alias failure cases on the new compiler. Capture errors
and minimal reproducers rather than broadly rewriting around them.

**Exit:** A command-by-command ledger identifies all missing execution and
behavior drift, plus a differences ledger separating parity repairs, borrowed
effects/testing techniques, and deferred architecture alternatives. Each
difference records CLI/workflow/bootstrap/runtime implications. New-nightly
app compilation has an explained outcome on macOS and Linux; unresolved blockers have bounded fixes. Prototype parity
is explicitly separated from full pipeline acceptance.

## Phase 1: Make the effects runner correct

**Bound:** Extract child-command execution, capture, output-file handling,
per-child cwd, environment overrides, temporary paths, and cleanup into one
module. Probe the reference Script APIs against 0.22.0 and, if needed,
0.23.0-rc1 under the selected compiler before choosing the final pin.
Reject empty argv instead of running `true`. Preserve stderr on
successful capture and failing redirects. Restore cwd on every exit,
including spawn, mkdir, write, and child errors. If basic-cli still lacks
per-child cwd or file streaming, retain serialized execution and implement
only the smallest fixed, safely parameterized process shim needed for those
effects; do not put job lists or decisions in shell. The reference capture
helper returns stdout on success and reports stderr on nonzero/signaled
status; adapt it to Rocci's required successful-stderr and exact exit-code
semantics rather than copying its return type unchanged.[^app][^ci][^pandoc-modules]

**Tests:** Native black-box tests with controlled fake tools exercise exit
0/1/2, missing executable, paths/argv containing spaces and quotes, missing
and non-UTF-8 data, environment isolation, cwd restoration, partial output,
and stderr retention. Large redirected output must not require unbounded
in-memory capture. Cancellation must terminate children or propagate a
documented limitation before production use. Existing child failure takes
precedence over cleanup failure while both remain visible.

**Exit:** Process observations and resulting files match the Python oracle.
Pure expects alone are insufficient; the compiled runner has passed real
child-process tests on Linux and macOS.

## Phase 2: Complete checks, parsers, and module integration

**Bound:** Port current dependency/coverage/Zed checks and release-version
logic, with Version imported and usable from the real dispatcher. Reconcile
classification/allowlist data. Replace fragile JSON-key text substitution
with a demonstrated codec mapping or token-aware transformation if the
reserved identifier still requires it. Keep TOML support bounded to the
actual config schemas and reject unsupported constructs explicitly.

**Tests:** Import the command modules together in the application; isolated
module tests cannot detect the historical cross-module failures. Cover
malformed/truncated inputs, unknown keys where policy requires rejection,
strings containing reserved key text, numeric overflow, record width,
semver/prerelease rules, and classified-but-missing/unclassified crates.
Scanner loops must make forward progress. Reuse Python fixtures before
adding independent cases.[^postmortem][^deps][^tests]

**Exit:** All `check` commands and version operations have executable parity.
Each historical compiler workaround is retained or removed with a named
regression. Defaults/iterators simplify helpers only when they pass the
actual imported app; they are not prerequisites for the port.[^nightly]

## Phase 3: Run all validation jobs through Roc

**Bound:** Preserve `app.roc ci` and its step-data runner; bring every plan
to the current Python plan, including
installed-okmate preference with local sibling fallback, editor targets,
example staging, conditional Roc provisioning, platform build before gated
tests, and platform bundle/archive after tests. Replace nested Python ops
calls with native Roc dispatch or internal command-family execution.
Retain Rust, Node and okmate validation tools. Keep Python parity tests as
a separately visible transitional check until their coverage is migrated.

**Tests:** Compare normalized job plans: argv, order, cwd, stdout path, and
extra env. Then execute each real job on its supported host, testing
fail-fast and `--keep-going` behavior with injected failures. Prove the
`knowledge` job uses the installed pinned okmate binary on hosted runners.
Prove the `roc` lane orders platform build, gated tests, bundle, and
archive correctly.[^roc-ci][^ci][^tests]

**Exit:** `rocci-ops ci` runs the real six-job surface through Roc. No
production step delegates policy to `uv run rocci-ops` or Python; the
temporary parity suite is the only remaining ops-specific Python test use.

## Phase 4: Complete local, site, and archive execution

**Bound:** Finish install/package/serve/clean commands and current site
behavior, including playground build, generated docs, live example
enumeration, assets, target selection, and publish reports. Complete archive
package, platform-package, libhost merge, version/params, CI wait, and
publication command construction. Preserve the site's existing dev choice
and platform triples/checksums.[^cli][^site][^archive]

**Tests:** Compare commands using fake external tools, then build real site
and release-layout artifacts in temporary output roots. Inspect filenames,
archive layout, non-empty platform payload, required libhost triples,
checksums, and GitHub output append semantics. Test the exact candidate
archive through extraction/installation; for the platform bundle, serve that
archive on an ephemeral local URL to compile a temporary consumer. Compare
against the required behavior and verify archive/hash identity, following
roc-pandoc's bundle test rather than validating only the source tree.
[^pandoc-bundle]

Verify failed packaging does not publish partial output. Exercise native server startup/stop and
same-origin HTTP/SSE for representative packaged artifacts.

**Exit:** No command in this phase is a help-only or placeholder path.
Representative Linux packages run without Python/uv; platform artifacts
retain current contents. A docs/site failure is diagnosed as product drift
or port regression rather than bypassed to obtain a green pipeline.

## Phase 5: Complete Git, release, deploy, and origin behavior

**Bound:** Implement current promotion semantics (including the documented
code/prose discrepancy), PR checkout/worktree
handling, release resolution and version-worktree cleanup, existing-CI
waiting, tag creation rules, and archive publication. Implement SSH
probe/bootstrap/push and origin publish/up/backup, including per-lane paths,
compose settings, health retries, current-pointer rollback, and retention.
The origin kit stages a versioned Roc binary and manifest and invokes it
directly; it stops shipping Python sources and uv metadata.[^release][^deploy][^origin][^bootstrap]

**Tests:** Use local bare Git remotes, fake gh/SSH/curl/Compose tools, and
temporary origin trees. Assert no release push after CI failure, no
accidental workflow dispatch by the CI waiter, correct dry-run version
resolution, `dev` movement versus default immutable version tags, branch
restoration, quoting, one-connection transfer behavior, no-proxy health
requests, staging/production isolation, and rollback after failed health.
Keep all real remote mutations and live deploys outside these tests.[^tests]

**Exit:** Every mutating path is implemented and covered by observable
effects. A Linux origin container without Roc, uv or Python can run its
operator through a successful staged publish and a failed-health rollback,
not merely print `origin --help`.

## Phase 6: Replace workflow ops invocations while preserving structure

**Bound:** Reuse the current compiler setup and job graph, then replace
Python ops calls in `ci.yml` and `knowledge.yml` with the equivalent Roc CLI
calls, followed by site package/deploy, release packaging, and Cut release.
Preserve job/check names, triggers, environment controls, PR snapshot
selection, permissions, artifact handoff, and existing release gate behavior.
Add exact-candidate verification within the owning packaging/test stages;
do not require roc-pandoc's release matrix or new aggregate job. Update
path filters to include `roc/rocci-ops/**`, `.roc-version`, setup inputs, and
affected workflow files. Native origin packaging is required; separate
compiler pins and shared CI binary artifacts remain alternatives unless
justified by recorded evidence.[^workflows][^setup][^pandoc-release]

**Tests:** Run read-only CI and Knowledge at the exact branch SHA with the
same fork/PR checkout paths as the current dispatcher. Exercise the Roc CLI
on Linux/macOS and the native Linux origin binary. Compare workflow job and
invocation behavior before/after, including failure propagation. Test setup
and operator compile failure so a broken compiler cannot yield a successful
job. Validate release/deployment with fixtures, package-only runs, and dry
runs; keep actual publishing separate. Measure cold/warm compiler setup,
app launch/build, and full pipeline time against Python. Dev object caching
is an opportunity, not a timing result. Record the cost and rationale for
any departure from the baseline.[^nightly][^pandoc-ci]

**Exit:** Hosted CI and Knowledge pass on the implementation revision;
record run IDs. Required package artifacts and simulated publication paths
are correct. Migrated jobs call the same Roc CLI with Python/uv absent from the ops
execution environment, retain their existing workflow boundaries, and consume
the exact assets that passed validation. Bootstrap failures are visible job failures, not
automatic fallback to the old Python pipeline.

## Phase 7: Retire Python and finish the operator contract

**Bound:** Map every remaining Python test to a Roc pure or native-effect
test, then remove active `rocci-ops/` Python packaging, pytest/setup-uv
dependencies, and obsolete origin installation requirements. Audit root
uv metadata before deleting it so independent Python tooling is preserved.
Give workspace classifications one Roc owner after the temporary dual-copy
parity gate. Update root/contributor/CLI/origin documentation, repository
agent instructions, and affected workflow skills to the new commands.
Keep historical knowledge explicitly historical.[^tests][^deps][^instructions][^bootstrap]

**Tests:** Search the repository for Python/uv ops calls and distinguish
historical citations from executable paths. From clean Linux/macOS
checkouts, bootstrap and run the complete Roc pipeline; run the origin
fixture without language toolchains. Repeat hosted CI and Knowledge after
removing the oracle. Check knowledge, classifications, docs/site, and
artifact manifests with the final layout.

**Exit:** There is one active Roc operator implementation, with no Python
ops execution or unsupported command path. Final revision CI and Knowledge
run IDs, package/runtime evidence, command coverage, and measured costs are
recorded. Update the log as complete only after those workflows pass. A
real staging smoke and production promotion remain separately authorized
operator actions.

## Acceptance and recovery

Completion requires full command/option coverage, native effect tests,
all six validation jobs, hosted Linux/macOS validation, real package
artifacts, release/deploy fixture behavior, and origin publish/rollback
without Python/uv/Roc. Help parity and a compiler-success summary are not
sufficient.

If a phase uncovers a compiler/platform bug, preserve a minimal reproducer
and stop only that dependent phase while completing independent plan work.
Prefer a bounded explicit workaround; do not silently remove a test or
route production back through Python. During rollout, the prior proven
operator binary/workflow revision is the explicit rollback target.
Production remains on its existing revision until the replacement has
passed its applicable gates.

For implementation evidence, pair this plan with an updated report in
`knowledge/research/ops/`; keep approved direction, local results, and
hosted completion distinct. This planning change itself requires only OKF
base validation and a clean diff check. No implementation phase or workflow
dispatch is performed by writing this document.

[^prototype-plan]: Earlier prototype scope, phases, and later-cutover requirement.
[^postmortem]: Historical isolation/runtime problems and partial command behavior.
[^nightly]: September 18 compiler results; operator app and Linux packaging were outside that validation.
[^branch]: Remote branch head verified read-only on September 30.
[^app]: Existing dispatch, effect handling, stubs, limited dry run, and staging merge sequence.
[^roc-ci]: Prototype job plans still invoke uv and omit newer platform steps.
[^parity]: Four observed command surfaces, not full execution parity.
[^cli]: Current names/options are the initial behavior contract.
[^ci]: Current child semantics, six jobs, tool discovery, and dedicated Roc lane.
[^tests]: Behavioral oracles for CLI, CI, archive, release, deploy, origin, and local work.
[^deps]: Temporary dual-copy rules and final classification owner.
[^site]: Current playground/live packaging and dev optimization choice.
[^release]: Existing-CI gate, isolated version edits, dry-run and tag behavior.
[^promote]: Current staging code merges origin/main and restores the starting branch; production pushes origin/staging.
[^readme]: Current operator prose describes staging as a rebase, which differs from implemented behavior.
[^archive]: Payload, triple, output, and checksum contracts.
[^deploy]: Remote command and bootstrap-kit contents currently depend on Python/uv.
[^origin]: Health, lane, publish, rollback, and backup behavior to retain.
[^setup]: Exact toolchain selection through the repository pin.
[^workflows]: Current workflow triggers, credentials, artifacts, environments, and operator invocations.
[^bootstrap]: Origin prerequisites and separate staging/production contracts.
[^instructions]: Current Python classification path and operator commands must change alongside retirement.

[^pandoc-modules]: Shared effects, validated project/release values, deterministic discovery, and pure edit planning, read at pinned revision 465cc4a7.
[^pandoc-check]: Tooling compiler is separate from package compiler; scripts pin basic-cli 0.23.0-rc1 and validate generated artifacts in temporary copies.
[^pandoc-goldens]: Golden-byte comparisons and bounded spawn/wait batches; Rocci must additionally prove early-failure child cleanup.
[^pandoc-bundle]: Candidate bundle is served locally and consumed by temporary test apps; no upstream test execution claimed here.
[^pandoc-release]: Observed workflow graph tests built assets before protected publication and supports validation without publishing.
[^pandoc-setup]: Reference setup extracts script/product pins with Python; it is inspiration, not a Python-free bootstrap to copy unchanged.
[^pandoc-ci]: Actions invokes scripts/check_all.roc with explicit tooling and product compiler environment paths.
