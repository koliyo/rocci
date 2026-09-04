---
type: Research Report
title: No GitHub release URL for rocci-platform
description: "Missing platform tar.zst on GitHub is deferred CI, not an unpushed v* tag. The product should expect that prebuilt artifact; checkout build.sh stays a human step. A released rocci binary cannot use CARGO_MANIFEST_DIR as pf."
tags: [domain/ops, domain/rocci, concern/ci, concern/tooling]
status: draft
generated: { by: process:cursor, at: 2026-09-04T10:05:00Z }
stale_after: 2026-12-04
authority: exploratory
owners: [human:nils]
sources:
  - id: plan
    resource: ../../plans/ops/rocci-platform-github-release.md
    title: Publish rocci-platform as a GitHub release asset
    author: process:cursor
    last_modified: 2026-09-04
  - id: platform-plan
    resource: ../../plans/rocci/rocci-as-roc-platform.md
    title: Package Rocci as a Roc platform
    author: process:cursor
    last_modified: 2026-09-04
  - id: postmortem
    resource: ../../audits/rocci/rocci-as-roc-platform-postmortem.md
    title: Platform post-mortem; no GitHub release URL yet
    author: process:cursor
    last_modified: 2026-09-04
  - id: platform-readme
    resource: ../../../crates/rocci-platform/README.md
    title: No GitHub release URL; local bundle.sh
    author: process:git
    last_modified: 2026-09-04
  - id: bundle-sh
    resource: ../../../crates/rocci-platform/bundle.sh
    title: roc bundle of platform Roc plus native libhost
    author: process:git
    last_modified: 2026-09-04
  - id: build-sh
    resource: ../../../crates/rocci-platform/build.sh
    title: Native libhost only; --all exits 1
    author: process:git
    last_modified: 2026-09-04
  - id: release-yml
    resource: ../../../.github/workflows/release.yml
    title: Tag-triggered CLI archive matrix
    author: process:git
    last_modified: 2026-09-04
  - id: archive-py
    resource: ../../../rocci-ops/src/rocci_ops/archive.py
    title: RELEASE_BINARIES rocci rocdown rocci-language-server
    author: process:git
    last_modified: 2026-09-04
  - id: ci-py
    resource: ../../../rocci-ops/src/rocci_ops/ci.py
    title: Hosted roc job does not run build.sh
    author: process:git
    last_modified: 2026-09-04
  - id: dispatch-rs
    resource: ../../../crates/rocci-cli/src/dispatch/mod.rs
    title: default_platform_pin from CARGO_MANIFEST_DIR
    author: process:git
    last_modified: 2026-09-04
  - id: rocci-readme
    resource: ../../../README.md
    title: rocci-ops release patch creates v* tags
    author: process:git
    last_modified: 2026-09-04
  - id: host-lib
    resource: ../../plans/ops/workspace-test-suite.md
    title: Hosted Roc lane
    author: process:cursor
    last_modified: 2026-09-04
---

# No GitHub release URL for rocci-platform

## Scope and authority

This record asks whether a **Roc-pinnable** `rocci-platform` `.tar.zst` is
missing because a GitHub **tag was never pushed**, or because **CI never
builds or uploads** that artifact. It is **exploratory**.[^plan]

It is not the same problem as a local checkout missing `libhost.a`
(`build.sh` not run). That is a source-tree host archive the **developer**
builds. A GitHub platform release is the **prebuilt** distribution pin,
the way basic-webserver `0.16.0` is a URL.[^platform-readme][^postmortem]

## What exists today

The platform plan froze **App pin (release)** as a `.tar.zst` from
`bundle.sh`, and said **no GitHub release URL unless a later phase adds
the workflow**. Phase 6 implemented local `build.sh` / `bundle.sh` and
**did not** add that workflow.[^platform-plan][^bundle-sh] The crate README
states there is no GitHub release URL; pin a path or a local
archive.[^platform-readme] The postmortem repeats: do not treat a GitHub
URL as shipped; operator CI should call `build.sh` when jobs need
`libhost.a`.[^postmortem]

`uv run rocci-ops release patch` (or Cut release) is the operator path
that creates an immutable `v*` tag so `release.yml` can publish.[^rocci-readme]
That workflow **does run** on `v*` and `dev`. It is not unused. It
packages **CLI binaries** (`rocci`, `rocdown`, `rocci-language-server`) as
`rocci-{version}-{target}.tar.gz`. It does not run `bundle.sh`, does not
install Roc for `roc bundle`, and does not attach a platform
`.tar.zst`.[^release-yml][^archive-py]

As of 2026-09-04 the public GitHub releases list for `koliyo/rocci` is the
rolling **`dev` prerelease** only. There is still no `v*` CLI release
either. That is a **separate** “no immutable tag yet” fact. Pushing `v*`
tomorrow would still **not** publish `rocci-platform`.

The hosted `roc` job installs Roc and sets `ROCCI_REQUIRE_ROC=1`. It does
not run `build.sh`.[^ci-py][^host-lib]

`bundle.sh` builds the **native** triple only (`build.sh --all` exits
1).[^build-sh] A GitHub URL that only contains `x64musl` `libhost.a` would
not serve macOS `rocci run`.

## Why a URL matters

`rocci-cli` default pin is
`env!("CARGO_MANIFEST_DIR")/../rocci-platform/platform/main.roc`.[^dispatch-rs]
That works for a binary compiled in a checkout **after** `build.sh`. A
binary from GitHub Actions was compiled under
`/home/runner/work/rocci/…`. That path is not on a user’s machine. A
released `rocci` therefore **cannot** use the in-tree pin unless it
downloads or embeds the platform. Catalog start from rocci-browser using
PATH `rocci` from a GitHub CLI archive hits the same hole.

## Conclusion

| Hypothesis | Verdict |
| --- | --- |
| Tag not pushed, CI would publish the platform | **False.** `release.yml` would still omit it. |
| CI setup for a Roc pin URL is missing | **True.** Deferred on purpose in Phase 6; not an accident. |
| Rolling `dev` has no `v*` | **True but orthogonal.** CLI archives exist on `dev`; platform still absent. |

Implementation: [platform GitHub release plan](/plans/ops/rocci-platform-github-release.md).[^plan]

[^plan]: Paired implementation plan.
[^platform-plan]: Phase 0 release pin; Phase 6 no GitHub URL unless the workflow is added.
[^postmortem]: Local `.tar.zst` only; CI does not invoke `build.sh`.
[^platform-readme]: Path or local bundle; no release URL.
[^bundle-sh]: `roc bundle` of `*.roc` plus whatever `targets/*/*.a` exist.
[^build-sh]: Native triple; `--all` not proven.
[^release-yml]: Matrix macos aarch64 + linux x86_64; `rocci-ops archive package`.
[^archive-py]: Three CLI names; README; `.tar.gz` + sha256.
[^ci-py]: `roc` job is install-roc then cargo test.
[^dispatch-rs]: Compile-time workspace path.
[^rocci-readme]: Operator `release patch` creates `v*`.
[^host-lib]: Hosted Roc lane is crate tests, not platform packaging.
