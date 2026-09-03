---
type: Implementation Plan
title: Roc rewrite of rocci-ops on a parallel branch
description: "Phased exercise: ordinary Roc under roc/rocci-ops/ matching the Python CLI names. uv rocci-ops stays the operator surface until parity and viability are recorded. Not a replacement or workflow cutover. Exploratory; do not start a phase until asked."
tags: [domain/ops, domain/rocci, integration/roc, concern/ci, concern/tooling, concern/publication]
status: draft
generated: { by: process:cursor, at: 2026-09-03T08:30:00Z }
stale_after: 2026-12-02
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../../research/ops/rocci-ops-roc.md
    title: Research for a Roc rocci-ops exercise
    author: process:cursor
    last_modified: 2026-09-03
  - id: python-research
    resource: ../../research/ops/python-uv-ops-pipeline.md
    title: Roc port deferred; POSIX remains PID 1, install-roc, ProxyCommand
    author: process:cursor
    last_modified: 2026-08-31
  - id: python-plan
    resource: python-uv-ops-pipeline.md
    title: Python pipeline; rewriting in Roc was out of bound
    author: process:cursor
    last_modified: 2026-08-31
  - id: dx-plan
    resource: rocci-ops.md
    title: Command tree the Roc app must list
    author: process:cursor
    last_modified: 2026-08-31
  - id: template-plan
    resource: ../rocci/roc-native-template-compiler.md
    title: Parallel-branch POC; production surface unchanged
    author: process:cursor
    last_modified: 2026-08-31
  - id: cli
    resource: ../../../rocci-ops/src/rocci_ops/cli.py
    title: USAGE, CHECK_USAGE, exit 2 for unknown
    author: process:git
    last_modified: 2026-08-31
  - id: ci
    resource: ../../../rocci-ops/src/rocci_ops/ci.py
    title: JOB_NAMES and Step
    author: process:git
    last_modified: 2026-08-31
  - id: deps
    resource: ../../../rocci-ops/src/rocci_ops/workspace_deps.py
    title: CLASSES and cargo metadata
    author: process:git
    last_modified: 2026-08-31
  - id: docs
    resource: ../../../rocci-ops/src/rocci_ops/docs_coverage.py
    title: Coverage TOML and heading slugs
    author: process:git
    last_modified: 2026-08-31
  - id: version
    resource: ../../../rocci-ops/src/rocci_ops/version.py
    title: Bump and lock rewrite
    author: process:git
    last_modified: 2026-08-31
  - id: origin
    resource: ../../../rocci-ops/src/rocci_ops/origin.py
    title: Origin publish/up/backup
    author: process:git
    last_modified: 2026-08-31
  - id: deploy
    resource: ../../../rocci-ops/src/rocci_ops/deploy.py
    title: SSH kit still copies Python sources
    author: process:git
    last_modified: 2026-08-31
  - id: archive
    resource: ../../../rocci-ops/src/rocci_ops/archive.py
    title: Archive naming and SHA-256
    author: process:git
    last_modified: 2026-08-31
  - id: release
    resource: ../../../rocci-ops/src/rocci_ops/release.py
    title: Operator release
    author: process:git
    last_modified: 2026-08-31
  - id: pyproject
    resource: ../../../rocci-ops/pyproject.toml
    title: Production console script
    author: process:git
    last_modified: 2026-08-31
  - id: install-roc
    resource: ../../../docker/install-roc.sh
    title: nightly-2026-08-23-fb208ba
    author: process:git
    last_modified: 2026-08-25
  - id: basic-cli-pin
    resource: ../../../crates/rocci-rocdown/src/lib.rs
    title: basic-cli 0.22.0 URL
    author: process:git
    last_modified: 2026-08-23
  - id: basic-cli-cmd
    resource: https://roc-lang.github.io/basic-cli/0.22.0/Cmd/
    title: Cmd without cwd or stdout redirect
    author: organization:roc-lang
    last_modified: 2026-08-23
  - id: author
    resource: ../../../.agents/skills/rocci-author/SKILL.md
    title: CLI is ordinary Roc, not .rocci
    author: process:git
    last_modified: 2026-08-31
---

# Roc rewrite of rocci-ops on a parallel branch

## Purpose and authority

This is the implementation plan for [A Roc port of rocci-ops is a
parallel exercise](/research/ops/rocci-ops-roc.md). It is an exploratory
**exercise** until a human accepts a scope. Python `rocci-ops` remains
the operator contract. Do not start a phase until the user asks.[^research][^python-plan][^cli]

This plan does **not** replace `rocci-ops/`. uv stays the CI, origin, and
localhost surface (`uv run --no-dev rocci-ops`). The Roc app exists to
prove **command parity** and **basic-cli viability**. Implementation
commits stay on git branch `rocci-ops-roc`, in parallel with
`main`.[^research][^template-plan][^pyproject]

Author ordinary `.roc` in the dialect this repo already compiles
(`snake_case`, `List(Str)`, `match`, `main! : … => Try({}, [..])`). This
is a CLI, not a Rocci page or component.[^author][^install-roc]

## Goal

On branch `rocci-ops-roc`, a basic-cli **app** under `roc/rocci-ops/`
implements the same command names as Python `rocci-ops`. When behavior
disagrees, **Python wins** and the Roc port changes.[^cli][^dx-plan][^research]

After the last phase:

- `roc test` covers dispatch, workspace-deps rules, version bump, and
  coverage checks against fixtures in `roc/rocci-ops/fixtures/`.
- `roc roc/rocci-ops/app.roc -- -h` lists the same commands as
  `uv run --no-dev rocci-ops -h`.
- `roc roc/rocci-ops/app.roc -- ci --list` prints the same job names in
  the same order.
- `check deps` on this workspace matches Python accept/reject.
- A recorded musl (or documented glibc) Linux build of the app prints
  `origin --help` without a Roc toolchain on that machine.
- GitHub workflows, origin bootstrap, and the README operator path still
  call Python. The POC is unused by product commands.

## Out of bound

- Replacing or idling `rocci-ops/` Python
- Switching `.github/workflows/*`, origin `uv run`, or README to the Roc
  app
- Installing a `rocci-ops` binary that shadows the uv console script
- Putting `roc` / rustc / `rocci` on the origin
- Rewriting `docker/*/entrypoint.sh`, `docker/install-roc.sh`, or
  `docker/prod/access-ssh-proxy.sh`
- Merging a product cutover onto `main`
- Changing command names or job lists to make Roc easier
- Cargo workspace membership for the Roc app
- `.rocci`, `@component`, `@context`, or `rocci.toml` for this CLI
- Waiting on a third-party Roc TOML or regex package

## Constraints that do not move

- **Parallel, not successor.** Operator CI, deploy, and origin stay
  Python until a later plan after Phase 7.
- **Parity means match Python.** Do not change pytest goldens or CLI
  help to accommodate the port.[^cli]
- Pin `nightly-2026-08-23-fb208ba` and basic-cli 0.22.0.[^install-roc][^basic-cli-pin]
- Origin still has no product toolchain. A Roc `origin` path is a
  **prebuilt binary**, not `roc run` on the VPS.[^origin][^deploy][^python-research]
- POSIX shims stay POSIX.[^python-research]
- `Cmd` has no cwd and no stdout-file redirect; sequential
  `Env.set_cwd!` and capture-then-write are the workarounds, recorded if
  they fail.[^basic-cli-cmd][^ci]
- Dual `CLASSES` copies must not drift; when a crate is added, Python is
  updated first and Roc follows in the same or next phase.[^deps]
- Prefer `match` on command tags and job names; `if` only for booleans.
- Do not `git add` unrelated work. Do not push from a phase unless asked.

## Non-goals (all phases)

- Byte-identical log timestamps and `elapsed_ms` lines
- Replacing pytest in the lint job
- Shipping Homebrew / Sparkle / a second package manager
- Porting tests that open real SSH or mutate `origin/main`

## Phase 0: Pin spike for basic-cli

**Bound:** A tiny `roc/rocci-ops/` app (or headerless file) that: reads
argv; runs `Cmd.exec_exit_code!` on `true` / `false` (or `cmd.exe`
equivalents only if the spike is Windows, which this repo does not
require); reads and writes UTF-8 via `Path`; reads one `Env.var_str!`;
`Encoding.Json.parse` of a checked-in cargo-metadata **subset** fixture;
`Env.set_cwd!` then `Cmd.exec!`. Record whether `main!` receives argv on
this pin.

**Out of bound:** Full CLI tree; TOML; git; SSH; HTTP; `roc build --target`.

**Tests:** `roc test` / `roc roc/rocci-ops/app.roc -- …` against the pin.

**Exit:** Draft revision of the research noting: argv shape; non-zero
`Cmd` mapping; JSON subset decode; cwd workaround. If JSON parse of the
metadata subset fails, stop and do not invent a Rust JSON helper.

**Recorded pin:** `main!` receives argv (`UnixBytes` on macOS).
`exec_exit_code!` is `Ok(0)` / `Ok(1)` for `true` / `false`. JSON subset
decode works after renaming reserved key `packages` to `pkgs`; extra
fields including `null` are ignored. `Env.set_cwd!` then `Cmd` works.
Details: [research Phase 0 pin](/research/ops/rocci-ops-roc.md).

## Phase 1: App skeleton and dispatch

**Bound:** `app.roc` plus `Cli.roc`. Top-level usage text matches
`USAGE` (same command names and one-line blurbs). `--help` / `-h` exit
0; no args and unknown command exit 2 to stderr. `check` usage matches
`CHECK_USAGE`. Subcommands other than help may return a distinct
"not implemented" exit that tests treat as **not** parity-success.

**Out of bound:** Running cargo/git; implementing job bodies.

**Tests:** `roc test` compares usage strings to the Python constants (copy
the Python `USAGE` into a Roc fixture and keep them in lockstep).

**Exit:** `roc roc/rocci-ops/app.roc -- -h` lists `ci`, `check`,
`release`, `archive`, `origin`. `verify-zed` as a top-level command
exits 2.[^cli][^dx-plan]

## Phase 2: Pure ports (deps, version, docs)

**Bound:** `WorkspaceDeps.roc` — same `CLASSES` / forbidden-edge rules;
input is JSON (live `cargo metadata` via `Cmd.exec_output!` in the app,
fixtures in tests). `Version.roc` — `patch|minor|major`, `vX.Y.Z`,
package-version line replace and lock crate rewrite **without** a regex
engine (explicit scanners). `DocsCoverage.roc` — hand-rolled parser for
the `[[feature]]` / search-query / first-use tables actually used, plus
heading slugify. Python remains the oracle: run Python on the same
fixture and compare.[^deps][^version][^docs]

**Out of bound:** CI job runner; deploy; mutating the repo `Cargo.toml`
from tests.

**Tests:** `roc test` on fixtures copied from `rocci-ops/tests/` data
needs (inline TOML/JSON under `roc/rocci-ops/fixtures/`). Side-by-side:
`uv run --no-dev rocci-ops check deps` vs the Roc app on this workspace.

**Exit:** Both `check deps` tools exit 0 on this workspace, or both fail
with the same unclassified-package names. Version bump of `1.2.3` patch
is `1.2.4` in Roc tests.

## Phase 3: CI runner

**Bound:** `Ci.roc` — `JOB_NAMES` in the same order; `steps_for` as data
(`argv`, optional `cwd`, `extra_env`, `stdout_path`). `ci --list` prints
one name per line. `run_job` uses `Cmd`; `stdout_path` is
`exec_output!` then write; `cwd` is `Env.set_cwd!` restored after the
step. Do not change `.github/workflows/ci.yml`.

**Out of bound:** Replacing the lint pytest step with `roc test`;
installing Roc on lint/test jobs.

**Tests:** `roc test` that `--list` equals Python `JOB_NAMES`; knowledge
steps still redirect the three JSON paths; fixtures-and-docs still calls
`check site` / `check docs`.[^ci]

**Exit:** `roc roc/rocci-ops/app.roc -- ci --list` matches
`uv run --no-dev rocci-ops ci --list`. Running a full `ci lint` from Roc
is optional and must not be required to land the phase.

## Phase 4: Local maintainer wrappers

**Bound:** `build`, `install`, `package`, `site`, `serve` as `Cmd`
sequences matching Python argv. Darwin gate via `Env.platform!`. Icons
and playground stay subprocesses to the same tools Python shells out to.

**Out of bound:** Changing package artifact names; macOS signing.

**Tests:** `roc test` of parsed argv / Darwin rejection on a fake
`Env.platform!` if the pin allows injecting platform; otherwise a
documented manual `--help` check per subcommand.

**Exit:** Each subcommand `--help` (or usage on missing args) matches
Python's usage exit. No workflow YAML changes.

## Phase 5: Git and gh (dry-run)

**Bound:** `pr-checkout`, `promote`, `push-worktrees`, `release` with
`--dry-run`, `archive version|params`. `gh` and `git` via `Cmd`.
`wait_for_check` may exist but tests use a fake `gh` fixture, not the
network. Do not push tags.

**Out of bound:** Hosted Cut release YAML; force-moving `dev` from the
Roc app.

**Tests:** `roc test` of version-from-ref and archive stem; release
`--dry-run` argument parse.

**Exit:** `archive version` / `params` write the same GitHub output keys
as Python when `GITHUB_REF_*` and `GITHUB_SHA` are set in the test
env.[^archive][^release]

## Phase 6: Deploy and origin logic; musl binary spike

**Bound:** Port `origin publish|up|backup` and `deploy probe|bootstrap|push`
**logic** (lane env, health URL, compose argv, keep-releases). SSH and
`tar` via `Cmd`. Health via `Http.get_utf8!` or `Cmd` to `curl` if Http
timeouts cannot match Python's 5s no-proxy opener — record the choice.
Spike `roc build` of `app.roc` for Linux (`--target` as supported by the
pin). Run the artifact's `origin --help` in Docker or on a Linux runner
**without** installing Roc in that image.

**Out of bound:** Changing `site.yml`; copying Roc sources onto the VPS;
a live deploy.

**Tests:** `roc test` of health URL construction and compose argv against
the Python tests' expectations (`test_origin.py` / `test_deploy.py` as
oracles). Binary spike documented in the research (command + target
triple).

**Exit:** Research notes whether a no-Roc Linux binary can print origin
usage. If `--target` cannot produce that binary on the pin, that is a
**viability fail** for origin, not a reason to install Roc on the VPS.

## Phase 7: Parity harness and viability gate

**Bound:** A small comparison script or `roc test` plus documented shell
that diffs: `-h`, `check -h`, `ci --list`, `check deps` (exit +
unclassified names). Revise the research with a go / no-go table
(Cmd mapping, JSON, TOML subset, origin binary, CI toolchain cost).
Still do not switch workflows.

**Out of bound:** Deleting Python; approving a cutover Decision.

**Tests:** The harness runs on this repo; `okmate check knowledge --profile base`.

**Exit:** Research and this plan record the table. Knowledge log is not
marked complete until CI and Knowledge workflows succeed on the revision
that contains the Roc sources. A later cutover is a **new** plan.

## Tests (whole plan)

```sh
roc test roc/rocci-ops/app.roc
roc roc/rocci-ops/app.roc -- -h
roc roc/rocci-ops/app.roc -- ci --list
uv run --no-dev rocci-ops -h
uv run --no-dev rocci-ops ci --list
uv run --no-dev rocci-ops check deps
uv run --directory rocci-ops --group dev pytest
okmate check knowledge --profile base --format terminal
```

`uv run --no-dev rocci-ops` stays the operator CLI. `cargo test
--workspace` must not spawn this Roc app unless `ROCCI_REQUIRE_ROC=1`.
`cargo fmt` is required only if a phase touches Rust.

[^research]: Dual implementation; Python oracle; origin needs a binary.
[^python-research]: uv on three machines; POSIX leftovers.
[^python-plan]: Roc rewrite was explicitly out of bound there.
[^dx-plan]: Names to list in `-h`.
[^template-plan]: Same parallel-branch rule.
[^cli]: Usage and exit 2.
[^ci]: Job list and Step fields.
[^deps]: Boundary classes.
[^docs]: TOML coverage.
[^version]: Semver and lock text.
[^origin]: No Roc on the VPS.
[^deploy]: Bootstrap still ships Python.
[^archive]: Archive params.
[^release]: Dry-run only in this plan.
[^pyproject]: Console script remains Python.
[^install-roc]: Compiler pin.
[^basic-cli-pin]: Platform URL.
[^basic-cli-cmd]: Exec API limits.
[^author]: Ordinary Roc for this CLI.
