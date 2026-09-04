---
type: Implementation Plan
title: Publish rocci-platform as a GitHub release asset
description: "Add CI that builds native libhost, runs bundle.sh, and attaches a Roc-pinnable tar.zst to the existing tag release. Do not treat a missing v* tag as the gap. Do not switch the in-tree default pin until a macOS-capable archive exists."
tags: [domain/ops, domain/rocci, concern/ci, concern/tooling]
status: draft
generated: { by: process:cursor, at: 2026-09-04T12:00:00Z }
stale_after: 2026-12-04
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../../research/ops/rocci-platform-github-release.md
    title: No GitHub release URL for rocci-platform
    author: process:cursor
    last_modified: 2026-09-04
  - id: platform-plan
    resource: ../rocci/rocci-as-roc-platform.md
    title: Package Rocci as a Roc platform
    author: process:cursor
    last_modified: 2026-09-04
  - id: postmortem
    resource: ../../audits/rocci/rocci-as-roc-platform-postmortem.md
    title: Platform post-mortem
    author: process:cursor
    last_modified: 2026-09-04
  - id: bundle-sh
    resource: ../../../crates/rocci-platform/bundle.sh
    title: Local roc bundle
    author: process:git
    last_modified: 2026-09-04
  - id: build-sh
    resource: ../../../crates/rocci-platform/build.sh
    title: Native libhost copy
    author: process:git
    last_modified: 2026-09-04
  - id: release-yml
    resource: ../../../.github/workflows/release.yml
    title: CLI archive publish
    author: process:git
    last_modified: 2026-09-04
  - id: archive-py
    resource: ../../../rocci-ops/src/rocci_ops/archive.py
    title: archive package and publish
    author: process:git
    last_modified: 2026-09-04
  - id: ci-py
    resource: ../../../rocci-ops/src/rocci_ops/ci.py
    title: roc job steps
    author: process:git
    last_modified: 2026-09-04
  - id: dispatch-rs
    resource: ../../../crates/rocci-cli/src/dispatch/mod.rs
    title: default_platform_pin
    author: process:git
    last_modified: 2026-09-04
  - id: platform-readme
    resource: ../../../crates/rocci-platform/README.md
    title: Crate pin docs
    author: process:git
    last_modified: 2026-09-04
---

# Publish rocci-platform as a GitHub release asset

## Goal

A GitHub release of Rocci includes a **Roc-pinnable**
`rocci-platform-*.tar.zst` (or documented equivalent) built by CI. That is
the **expected prebuilt artifact** for PATH / released `rocci`. Local
`build.sh` remains an explicit developer step, not something `rocci run`
or rocci-browser invokes.[^research][^platform-plan]

## Out of bound

- Treating “push `v*`” as sufficient without workflow changes
- Committing `libhost.a` to git
- `build.sh --all` until that script is proven
- Changing `--http-module` (stays 0.16.0) or wasm apply
- rocci-browser `build.rs`, GUI cargo, a listing build button, or auto
  `build.sh` from `rocci run`
- Expanding the CLI archive matrix (Intel macOS, Windows) in this plan
- Replacing `rocci-ops release` as the only operator `v*` path

Local `build.sh` first remains the checkout path; CI still does not invoke
it today.[^postmortem]

## Constraints that do not move

- Existing `release.yml` keeps publishing CLI `.tar.gz` archives.[^release-yml][^archive-py]
- In-tree generated apps may keep a **path** pin while a checkout exists;
  a GitHub URL is for released / PATH binaries and authored `main.roc`.[^dispatch-rs]
- A URL that only contains a Linux `libhost.a` is not a macOS pin.
- Default `cargo test --workspace` stays offline (no `roc bundle` on every
  PR unless gated).

## Recommended sequence

### 1. Run `build.sh` on the hosted Roc job

Bound: `rocci-ops` `ci roc` steps (and thus `.github/workflows/ci.yml` roc
job). After installing Roc, run `crates/rocci-platform/build.sh` then the
existing `ROCCI_REQUIRE_ROC=1` cargo tests.[^ci-py][^build-sh]

**Tests:** pytest or step-list assertion that `build.sh` appears before the
gated cargo test.

**Exit:** `uv run --group dev pytest` under `rocci-ops` for the touched
test; hosted roc job definition includes `build.sh`. Do not log complete
until that workflow succeeds.

### 2. CI `bundle.sh` artifact (Linux native first)

Bound: a release or roc-adjacent job with Roc on PATH that runs
`crates/rocci-platform/bundle.sh` and uploads the resulting `.tar.zst`
(plus sha256). Document the asset name. Fail if no `libhost.a`.[^bundle-sh]

**Tests:** dry-run or fixture that `bundle.sh` is invoked with Roc mocked
only if that stays honest; otherwise a hosted job is the proof.

**Exit:** an Actions artifact exists on a `dev` or `v*` run. Crate README
names the asset as **not yet** the default `rocci` pin.[^platform-readme]

### 3. Attach the archive on `release.yml` publish

Bound: `rocci-ops archive` / `release.yml` so `archive publish` uploads the
platform `.tar.zst` next to CLI `.tar.gz` files. Same tag, same
`GITHUB_TOKEN`. Do not invent a second tag namespace.

**Tests:** unit or pytest on archive file glob; workflow string test that
`bundle` or `tar.zst` is referenced.

**Exit:** the `dev` prerelease (or next `v*`) lists the platform asset.
Document the URL shape in the crate README as **available**, still not
the generated-app default.

### 4. Multi-triple libhost in one bundle

Bound: produce `arm64mac` (and existing Linux) `libhost.a` on the release
matrix, copy into `platform/targets/<triple>/` on one runner, then one
`roc bundle`. Do not claim `--all`. Missing triples stay listed in the
README.[^build-sh]

**Tests:** archive listing includes more than one `targets/` triple, or
documented skip if a runner cannot copy artifacts between jobs.

**Exit:** a macOS `rocci run` of Counter with `pf` set to the release URL
links. Decision gate before flipping `default_platform_pin`.

### 5. Released `rocci` pin fallback (decision gate)

Bound: `rocci-cli` pin resolution. If the compile-time path does not exist,
use the GitHub `.tar.zst` URL from this release line (or a file next to
the executable). Do not network on every `rocci run` when the in-tree path
exists.[^dispatch-rs]

**Tests:** unit test path-exists vs URL fallback with a fake pin; no live
GitHub.

**Exit:** `cargo test -p rocci-cli`; docs in crate and root README. Human
review before this becomes the default generated pin.

## Decision gates

- Do not merge Phase 5 without Phase 4 (macOS host in the archive) unless
  the fallback is Linux-only and documented as such.
- Do not remove in-tree path pins for developer checkouts.

## Status

Phases 1–4 implemented on branch `rocci-platform-github-release` (hosted
`build.sh`, Linux CI artifact, tag publish, `arm64mac`+`x64musl` merge).
Not logged complete until CI and Knowledge succeed. Phase 5 remains a
human decision gate. Evidence:
[no GitHub platform URL](/research/ops/rocci-platform-github-release.md).[^research]
Follow-on to [package Rocci as a Roc platform](/plans/rocci/rocci-as-roc-platform.md)
Phase 6.[^platform-plan]

[^research]: Tag vs missing workflow.
[^platform-plan]: Release URL deferred until a workflow exists.
[^postmortem]: CI does not invoke `build.sh`.
[^bundle-sh]: Local `roc bundle`.
[^build-sh]: Native triple only.
[^release-yml]: CLI matrix and publish job.
[^archive-py]: CLI names only.
[^ci-py]: Roc job steps.
[^dispatch-rs]: Compile-time path pin.
[^platform-readme]: No URL today.
