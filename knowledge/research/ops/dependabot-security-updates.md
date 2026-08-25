---
type: Research Report
title: Dependabot security updates for Rocci lockfiles
description: GitHub Dependabot alerts on koliyo/rocci should be closed with in-range lockfile pins; glib 0.18 cannot move to 0.20 without a wry/tao GTK4 generation bump.
tags: [domain/rocci, concern/ci, concern/security, concern/tooling]
status: draft
generated: { by: process:cursor, at: 2026-08-25T00:20:00Z }
stale_after: 2026-11-25
authority: exploratory
owners: [human:nils]
sources:
  - id: dependabot-yml
    resource: ../../../.github/dependabot.yml
    title: Dependabot version-update ecosystems
    author: process:git
    last_modified: 2026-08-25
  - id: cargo-lock
    resource: ../../../Cargo.lock
    title: Workspace Cargo.lock
    author: process:git
    last_modified: 2026-08-25
  - id: vscode-pkg
    resource: ../../../editors/vscode/package.json
    title: VS Code extension package.json including npm overrides
    author: process:cursor
    last_modified: 2026-08-25
  - id: desktop-toml
    resource: ../../../crates/rocci-desktop/Cargo.toml
    title: rocci-desktop tao wry muda pins
    author: process:git
    last_modified: 2026-08-25
  - id: workspace-toml
    resource: ../../../Cargo.toml
    title: Workspace tao wry muda versions
    author: process:git
    last_modified: 2026-08-25
  - id: skill
    resource: ../../../.agents/skills/rocci-dependabot/SKILL.md
    title: Agent procedure for Dependabot alerts
    author: process:cursor
    last_modified: 2026-08-25
  - id: public-ci-audit
    resource: ../../audits/ops/public-ci-security.md
    title: Public-repo GitHub Actions security review
    author: process:cursor
    last_modified: 2026-08-23
  - id: ghsa-ring
    resource: https://github.com/advisories/GHSA-4p46-pwfr-66x6
    title: ring AES panic when overflow checking is enabled
    author: organization:github
  - id: ghsa-glib
    resource: https://github.com/advisories/GHSA-wrw7-89jp-8q8g
    title: glib VariantStrIter unsoundness
    author: organization:github
  - id: rustsec-glib
    resource: https://rustsec.org/advisories/RUSTSEC-2024-0429.html
    title: RUSTSEC-2024-0429 glib VariantStrIter
    author: organization:rustsec
  - id: ghsa-serialize-rce
    resource: https://github.com/advisories/GHSA-5c6j-r48x-rmvq
    title: serialize-javascript RCE via RegExp.flags
    author: organization:github
  - id: ghsa-serialize-dos
    resource: https://github.com/advisories/GHSA-qj8w-gfj5-8c6v
    title: serialize-javascript CPU exhaustion DoS
    author: organization:github
  - id: crates-glib
    resource: https://crates.io/crates/glib/versions
    title: Published glib crate versions
    author: organization:crates-io
---

# Dependabot security updates for Rocci lockfiles

## Practice

List open alerts with the GitHub Dependabot API before changing lockfiles. The
push hook that names a count and
`https://github.com/koliyo/rocci/security/dependabot` is that list, not a
guess about Wasmtime or npm `glob`.[^skill]

`.github/dependabot.yml` already schedules weekly version updates for Cargo
(workspace and `editors/zed`), uv, npm under `editors/vscode`, GitHub Actions,
and Docker images. Version-update PRs and security alerts are separate
surfaces. The [public-repo CI security audit](/audits/ops/public-ci-security.md)
still describes a repo with no `dependabot.yml`; that claim is historical
relative to the checked-in config.[^dependabot-yml][^public-ci-audit]

Prefer `cargo update -p <crate> --precise <patched>` or an npm `overrides` pin
on the **same generation** already in the lockfile. Do not refresh the entire
`Cargo.lock` to clear one GHSA.[^skill][^cargo-lock]

## August 2026 default-branch alerts

Open alerts on `koliyo/rocci` at the time of this note (1 high, 3 moderate):

| Alert | Package | Manifest | Patched | Treatment |
| --- | --- | --- | --- | --- |
| 4, 5 | `serialize-javascript` | `editors/vscode/package-lock.json` | 7.0.5 | mocha still declares `^6.0.2`; npm `overrides` forces 7.0.5. Dev-only (mocha worker serialization), not the VSIX runtime.[^vscode-pkg][^ghsa-serialize-rce][^ghsa-serialize-dos] |
| 2 | `ring` | `Cargo.lock` | >= 0.17.12 | Transitive via `rustls` / `ureq` in `rocci-cli`. Precise bump on the 0.17 line.[^cargo-lock][^ghsa-ring] |
| 1 | `glib` | `Cargo.lock` | >= 0.20.0 | Locked at 0.18.5 with gtk-rs 0.18 (`gtk`, `webkit2gtk`) through `tao`, `wry`, and `muda`. No 0.18.6 was published. 0.20 is a gtk-rs generation cut, not an in-range patch.[^cargo-lock][^desktop-toml][^workspace-toml][^ghsa-glib][^crates-glib] |

The glib advisory is unsoundness in `VariantStrIter` (`RUSTSEC-2024-0429`).
Rocci does not call that iterator; Linux desktop still compiles the 0.18
bindings. Closing the GitHub alert requires `glib` >= 0.20, which needs wry/tao
on GTK4 and WebKitGTK 6 (`webkit6`), plus matching system packages. That is a
desktop-stack migration, not a Dependabot pin.[^rustsec-glib][^ghsa-glib][^desktop-toml]

## Agent workflow

Use the repository skill `.agents/skills/rocci-dependabot/SKILL.md`. Alerts
show as open on GitHub until the **default branch** contains the patched
lockfile.[^skill]

[^dependabot-yml]: Weekly Cargo, uv, npm, Actions, and Docker ecosystems.
[^cargo-lock]: Workspace lockfile versions for `ring` and `glib`.
[^vscode-pkg]: npm `overrides` for `serialize-javascript`.
[^desktop-toml]: Direct `tao` / `wry` / `muda` dependency of `rocci-desktop`.
[^workspace-toml]: Workspace versions for those crates.
[^skill]: Agent procedure for listing and patching alerts.
[^public-ci-audit]: Audit text still says Dependabot config is absent.
[^ghsa-ring]: Advisory patched in `ring` 0.17.12.
[^ghsa-glib]: GitHub patched range starts at `glib` 0.20.0.
[^rustsec-glib]: Unsound `VariantStrIter` in glib 0.15–0.19.
[^ghsa-serialize-rce]: High-severity RCE, patched in 7.0.3.
[^ghsa-serialize-dos]: Moderate DoS, patched in 7.0.5.
[^crates-glib]: No 0.18.6; 0.18 line ends at 0.18.5.
