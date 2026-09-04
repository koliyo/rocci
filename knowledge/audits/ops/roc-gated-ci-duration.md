---
type: Audit
title: Hosted Roc-gated crate tests exceed ten minutes
description: The hosted roc job ran 16 minutes on 2026-09-04 because island tests waited 120–180s for /health and preview after Roc rejected the absolute in-tree platform pin; cold Rust compile and release musl libhost add several more minutes.
tags: [domain/ops, domain/rocci, domain/rocdown, concern/ci, concern/testing, concern/performance, audience/maintainer]
status: draft
generated: { by: process:cursor, at: 2026-09-04T11:20:00Z }
stale_after: 2026-12-04
authority: descriptive
owners: [human:nils]
sources:
  - id: ci-yml
    resource: ../../../.github/workflows/ci.yml
    title: Hosted roc job checkout, musl target, and platform artifact upload
    author: process:git
    last_modified: 2026-09-04
  - id: ci-py
    resource: ../../../rocci-ops/src/rocci_ops/ci.py
    title: roc job steps including build.sh, gated cargo test, and bundle.sh
    author: process:git
    last_modified: 2026-09-04
  - id: gitattributes
    resource: ../../../.gitattributes
    title: "*.a and *.o stored in Git LFS"
    author: process:git
    last_modified: 2026-09-04
  - id: build-sh
    resource: ../../../crates/rocci-platform/build.sh
    title: Release musl libhost copy
    author: process:git
    last_modified: 2026-09-04
  - id: bundle-sh
    resource: ../../../crates/rocci-platform/bundle.sh
    title: bundle.sh re-runs build.sh unless --skip-build
    author: process:git
    last_modified: 2026-09-04
  - id: dispatch-pin
    resource: ../../../crates/rocci-cli/src/dispatch/mod.rs
    title: Default generated apps pin in-tree rocci-platform via CARGO_MANIFEST_DIR
    author: process:git
    last_modified: 2026-09-04
  - id: islands
    resource: ../../../crates/rocci-rocdown-cli/tests/islands.rs
    title: Island tests with 120s health and 180s preview waits
    author: process:git
    last_modified: 2026-09-04
  - id: suite-plan
    resource: ../../plans/ops/workspace-test-suite.md
    title: Hosted roc lane intended as generated-app HTTP smokes
    author: process:cursor
    last_modified: 2026-09-04
  - id: run-sep1
    resource: https://github.com/koliyo/rocci/actions/runs/33488554463
    title: Successful roc job before platform build.sh and in-tree pin
    author: organization:github
    last_modified: 2026-09-01
  - id: run-lfs-fail
    resource: https://github.com/koliyo/rocci/actions/runs/33864045191
    title: Roc job failing on Git LFS pointer crt1.o
    author: organization:github
    last_modified: 2026-09-04
  - id: run-lfs-pull
    resource: https://github.com/koliyo/rocci/actions/runs/33865097789
    title: First roc job with lfs true; 16m wall; islands timeout
    author: organization:github
    last_modified: 2026-09-04
---

# Hosted Roc-gated crate tests exceed ten minutes

## Scope

This audit explains why the GitHub Actions job **Roc-gated crate tests**
took **16 minutes** on 2026-09-04 (run
[33865097789](https://github.com/koliyo/rocci/actions/runs/33865097789)).
It compares that run with a 6.5-minute green job from 2026-09-01 and a
same-day 6-minute failure on LFS pointer objects. It does not change CI
or tests.[^ci-yml][^run-sep1][^run-lfs-fail][^run-lfs-pull]

## What the job runs

Workflow YAML installs desktop deps and a musl Rust target, then
`uv run --no-dev rocci-ops ci roc` runs:[^ci-yml][^ci-py][^suite-plan]

1. `docker/install-roc.sh` (pinned nightly).
2. `rustup target add x86_64-unknown-linux-musl`.
3. `crates/rocci-platform/build.sh` — release musl `libhost`.
4. `ROCCI_REQUIRE_ROC=1 cargo test -p rocci-cli -p rocci-rocdown -p rocci-rocdown-cli` (entire default suites).
5. `crates/rocci-platform/bundle.sh` (calls `build.sh` again unless `--skip-build`).
6. `rocci-ops archive package-platform`.

Linux jobs do not use `Swatinem/rust-cache`.[^ci-yml]

## Measured wall time

| Run | LFS objects | Job wall | Cargo-test internals | Outcome |
| --- | --- | --- | --- | --- |
| [33488554463](https://github.com/koliyo/rocci/actions/runs/33488554463) | no | ~6.5 min | compile 3m 33s; `rocci-cli` lib 65s; islands 16s | success |
| [33864045191](https://github.com/koliyo/rocci/actions/runs/33864045191) | pointers | ~6 min | `build.sh` 1m 13s; compile 2m 56s; lib tests fail in 45s on `crt1.o` | 9 `roc build` failures |
| [33865097789](https://github.com/koliyo/rocci/actions/runs/33865097789) | `lfs: true` | **16 min 4s** (`10:50:31Z`–`11:06:35Z`) | see breakdown below | islands failed; bundle skipped |

Run 33865097789, step `Run Roc-gated tests` (`10:51:55Z`–`11:06:34Z`):[^run-lfs-pull][^build-sh]

| Substep | Duration |
| --- | --- |
| `install-roc.sh` | ~2s |
| `build.sh` release musl `libhost` | 1m 32s |
| `cargo test` compile (debug) | 3m 30s |
| `rocci-cli` lib tests (230 passed) | 56s |
| remaining `rocci-cli` / `rocci-rocdown` binaries | ~30s |
| `rocci-rocdown-cli` `islands` (0 passed, 5 failed) | **481s** |

The 10+ minute complaint is this last binary plus the cold compile and
host build. `bundle.sh` did not run because cargo test failed.[^bundle-sh][^run-lfs-pull]

## Dominant cost: island waits after an absolute platform pin

Generated apps pin
`crates/rocci-platform/platform/main.roc` through
`CARGO_MANIFEST_DIR`, which is an **absolute** path on CI
(`/home/runner/work/rocci/rocci/crates/rocci-platform/platform/main.roc`).
The pinned Roc nightly rejects that:[^dispatch-pin][^run-lfs-pull]

```text
── ✗ absolute platform path ──
Absolute paths are not allowed for platform specifications.
    /home/runner/work/rocci/rocci/crates/rocci-platform/platform/main.roc
Tip: Use a relative path like ../path/to/platform or a URL.
```

`rocci-cli` lib smokes still passed in 56s (they stage a workspace and
build from that directory, so the pin can be relative). The
`islands` tests spawn `rocdown serve-islands` / `rocdown run` from the
repo root and then poll:[^islands][^run-lfs-pull]

- `wait_for_health`: 120s, panic `timed out waiting for island service`
- `wait_for_preview` / AllSyntax: 180s, panic `timed out waiting for … preview`

The preview server often **does** bind (`preview_ready`) and serves a
build-error HTML page, but `/health` never becomes `ok` and the tests
keep polling. `ROC_LOCK` serializes the five cases, so the waits
**add**: ~120s (`hybrid_cdn_html_and_island_post_morph`) + ~180s
(`all_syntax_run_serves_the_kitchen_sink`) + ~180s
(`docs_run_previews_the_site`) ≈ 480s, matching the 481.47s suite
time. Two other tests failed faster on the build-error body.[^islands][^run-lfs-pull]

On 2026-09-01 the same five tests finished in 16s because generated
apps still used the `basic-webserver` 0.16.0 URL, which Roc
accepts.[^run-sep1]

## Secondary costs (not the 10-minute jump)

**Git LFS.** `*.a` and `*.o` are LFS, including `crt1.o`. Without
smudge, `roc build` dies in seconds on `version https://git-lfs.github.com/spec/v1`.
That is why 33864045191 stayed near 6 minutes: the expensive island
path never ran. `lfs: true` (commit `53f150de`) made `rocci-cli`
builds succeed; it did not make islands fast.[^gitattributes][^run-lfs-fail][^ci-yml]

**Release `libhost`.** `build.sh` adds ~1.5 minutes that the September
1 job did not pay. `bundle.sh` would add another host compile on a
green cargo test because `ci.py` does not pass `--skip-build`.[^build-sh][^bundle-sh][^ci-py]

**Cold debug compile** of the three packages is ~3.5 minutes with or
without Roc, same order as September 1. Linux has no rust-cache on this
job.[^run-sep1][^ci-yml]

## Implications (not implemented)

The 16-minute wall is mostly **serialized 2–3 minute timeouts** after a
deterministic Roc error (absolute `pf` path), not a slow successful
`roc build` of every fixture. A relative pin (or URL) should make
islands fail in seconds or pass like September 1. Independently, fail
fast when `/health` never returns `ok` instead of waiting 120–180s, and
pass `--skip-build` to `bundle.sh` after `build.sh`. This record does
not choose among those.

[^ci-yml]: Hosted `roc` job in `.github/workflows/ci.yml`.
[^ci-py]: `rocci-ops` `ci roc` step list.
[^gitattributes]: LFS patterns for `*.a` and `*.o`.
[^build-sh]: Native musl `cargo build -p rocci-platform --release`.
[^bundle-sh]: `bundle.sh` invokes `build.sh` unless `--skip-build`.
[^dispatch-pin]: `default_platform_pin` / `rocci_platform_main_roc`.
[^islands]: `wait_for_health` 120s; `wait_for_preview` 180s.
[^suite-plan]: Hosted roc lane in the workspace test-suite plan.
[^run-sep1]: CI run 33488554463 logs.
[^run-lfs-fail]: CI run 33864045191 logs.
[^run-lfs-pull]: CI run 33865097789 logs.
