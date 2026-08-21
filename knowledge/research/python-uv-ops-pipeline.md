---
type: Research Report
title: Findings after migrating operator scripts to Python and uv
description: "After implementing tools/rocci-ops, POSIX remains only where the process cannot be uv (container PID 1, image Roc install, OpenSSH ProxyCommand). A later Roc port should reuse the same CLI surface; basic-cli is not yet a substitute for uv on CI, laptops, and the origin."
tags: [domain/rocci, concern/ci, concern/tooling, concern/publication]
status: draft
generated: { by: process:cursor, at: 2026-08-21T10:00:00Z }
stale_after: 2026-11-21
authority: exploratory
owners: [human:nils]
sources:
  - id: plan
    resource: ../plans/python-uv-ops-pipeline.md
    title: Python and uv operator pipeline plan
    author: process:cursor
    last_modified: 2026-08-21
  - id: pyproject
    resource: ../../tools/rocci-ops/pyproject.toml
    title: rocci-ops uv project metadata
    author: process:cursor
    last_modified: 2026-08-21
  - id: ci-yml
    resource: ../../.github/workflows/ci.yml
    title: Thin GitHub Actions CI workflow
    author: process:cursor
    last_modified: 2026-08-21
  - id: site-yml
    resource: ../../.github/workflows/site.yml
    title: Site package and deploy workflow
    author: process:cursor
    last_modified: 2026-08-21
  - id: origin
    resource: ../../tools/rocci-ops/src/rocci_ops/origin.py
    title: Origin publish implemented in Python
    author: process:cursor
    last_modified: 2026-08-21
  - id: proxy
    resource: ../../docker/prod/access-ssh-proxy.sh
    title: OpenSSH ProxyCommand
    author: process:git
    last_modified: 2026-08-21
  - id: app-entry
    resource: ../../docker/app/entrypoint.sh
    title: App container PID 1
    author: process:git
    last_modified: 2026-08-20
  - id: install-roc
    resource: ../../docker/install-roc.sh
    title: Pinned Roc nightly installer
    author: process:git
    last_modified: 2026-08-19
---

# Findings after migrating operator scripts to Python and uv

## What moved

CI job bodies, release packaging and `gh` gating, deploy SSH/SCP, origin
publish/up/backup, and localhost maintainer helpers now live in
`tools/rocci-ops` (Python 3.12, stdlib plus pytest, committed `uv.lock`).
GitHub Actions YAML is checkout, toolchains, cache, `astral-sh/setup-uv`,
secrets, and artifacts; sequences are `uv run --project tools/rocci-ops --no-dev rocci-ops …`.[^plan][^pyproject][^ci-yml][^site-yml][^origin]

## What stayed POSIX

These processes cannot be `uv run` without changing a foreign contract:

- Container PID 1 (`docker/app/entrypoint.sh`, `docker/cdn/entrypoint.sh`) execs the image binary.[^app-entry]
- `docker/install-roc.sh` runs before Roc exists in builder images.[^install-roc]
- `docker/prod/access-ssh-proxy.sh` is OpenSSH `ProxyCommand` and must `exec cloudflared`.[^proxy]

## Origin versus product toolchain

The VPS is assumed to have Python and uv. It still must not gain `rocci`,
`rocdown`, `roc`, rustc, or WebKit. Bootstrap copies Compose/Caddy plus the
ops package to `/srv/rocci`; remote publish is `uv run rocci-ops origin publish`.[^origin][^site-yml]

## Roc port (separate branch)

Do not block operator work on Roc. A 1:1 port of `rocci-ops` would need:

- Reliable subprocess, env, and non-zero exit mapping comparable to CPython
- A pinned Roc on every GitHub job, laptop, and the origin (site packaging already installs Roc; lint/test currently do not)
- Either a static binary usable as `ProxyCommand` or leaving that shim POSIX
- PID 1 in Debian slim images remaining a tiny shell unless Roc emits a static `exec`

`basic-cli` is the likely host; it is not yet a drop-in replacement for uv on
all three machines. Keep the Python CLI stable so the Roc branch can match
command names.[^plan]

[^plan]: Phased Bound/Exit for the Python migration.
[^pyproject]: `requires-python >= 3.12`, console script `rocci-ops`.
[^ci-yml]: `rocci-ops ci` per job after rust/node setup.
[^site-yml]: Deploy job writes the SSH key then `rocci-ops deploy`.
[^origin]: Health check, compose up, rollback, prune.
[^proxy]: Tokens stay in the proxy process environment.
[^app-entry]: `exec server`.
[^install-roc]: curl/tar of a pinned nightly.
