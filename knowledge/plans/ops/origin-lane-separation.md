---
type: Implementation Plan
title: Separate staging and production origins on one VPS
description: "Two Compose projects on the rocci.dev VPS: production keeps /srv/rocci on :8080 without live-example origins; staging is /srv/rocci-staging on :8081 with its own SQLite volumes. Cloudflare routes by hostname. Exploratory; code in this revision; cutover is operator work."
tags: [domain/rocci, concern/publication, concern/ci, audience/maintainer]
status: draft
generated: { by: process:cursor, at: 2026-08-30T08:50:00Z }
stale_after: 2026-11-30
authority: exploratory
owners: [human:nils]
sources:
  - id: lanes
    resource: ../../../tools/rocci-ops/src/rocci_ops/lanes.py
    title: ROCCI_LANE presets and publish-live flag
    author: process:cursor
    last_modified: 2026-08-30
  - id: origin-ops
    resource: ../../../tools/rocci-ops/src/rocci_ops/origin.py
    title: Origin publish, compose, and health
    author: process:git
    last_modified: 2026-08-30
  - id: deploy-ops
    resource: ../../../tools/rocci-ops/src/rocci_ops/deploy.py
    title: SSH bootstrap and origin_publish_cmd
    author: process:git
    last_modified: 2026-08-30
  - id: site-workflow
    resource: ../../../.github/workflows/site.yml
    title: Site package and deploy
    author: process:git
    last_modified: 2026-08-30
  - id: prod-readme
    resource: ../../../docker/prod/README.md
    title: Origin layout, lanes, and cutover
    author: process:git
    last_modified: 2026-08-30
  - id: ingress
    resource: ../../../docker/prod/cloudflared-ingress.yml.example
    title: Tunnel hostname to loopback port map
    author: process:git
    last_modified: 2026-08-30
---

# Separate staging and production origins on one VPS

## Goal

`staging` and `production` no longer publish the same `/srv/rocci` stack.
Production is the current tree on `:8080` without live-example containers.
Staging is a second tree on `:8081` with its own SQLite volumes and live
apps.[^lanes][^prod-readme]

## Out of bound

- A second VPS, Kubernetes, or Cloudflare API from CI.
- Advertising Launch or publishing `*-example.rocci.dev`.
- ACM / deep `*.examples.rocci.dev`.
- Migrating live-counter rows off `rocci-prod_*` volumes.
- Opening provider 80/443.

## Constraints that do not move

| Constraint | Required behavior |
| --- | --- |
| Same box | Both lanes stay on the existing VPS.[^prod-readme] |
| Production quality | Current `/srv/rocci` stays production if live-example endpoints are unpublished.[^prod-readme] |
| Image isolation | Staging `--build` must not retag production images.[^lanes] |
| Serialized compose | `site.yml` deploy concurrency stays `rocci-dev-origin`.[^site-workflow] |
| Access | Staging hostnames stay Access-gated.[^prod-readme] |

## Phase 0 — Lane table and publish flag

**Bound:** Lane presets, origin compose/health, deploy publish command.[^lanes][^origin-ops][^deploy-ops]

**Status:** implemented in this revision.

## Phase 1 — Caddy snippet and image tags

**Bound:** Site Caddyfile imports an examples snippet; hybrid mounts a stub;
origin compose remounts the snippet and live apps. Image tag env on islands,
cdn, and live-app images.[^origin-ops][^prod-readme]

**Status:** implemented in this revision.

## Phase 2 — Site workflow lane env

**Bound:** Deploy job sets `ROCCI_LANE` from the branch name.[^site-workflow]

**Status:** implemented in this revision.

## Phase 3 — Docs and ingress example

**Bound:** Origin README and Tunnel ingress example.[^prod-readme][^ingress]

**Status:** implemented in this revision.

## Phase 4 — Operator cutover

**Bound:** VPS directories for the staging root, promote staging, retarget
Tunnel staging hosts to `:8081`, promote production without live
origins.[^prod-readme]

**Status:** not started. Do not log complete until CI and Knowledge succeed
and the VPS/Tunnel steps are done.

## Exit

From `tools/rocci-ops`, `uv run --group dev pytest` on the origin, deploy,
example-origins, and workflow-branches tests. `okmate check knowledge --profile rocci`.

[^lanes]: `ROCCI_LANE` table: production `/srv/rocci` `:8080` no live apps; staging `/srv/rocci-staging` `:8081` with live apps.
[^origin-ops]: `compose up` merges origin compose only when live publish is on; `--remove-orphans`; health Hosts are lane-aware.
[^deploy-ops]: Remote `origin publish` exports lane env; bootstrap copies both Caddy snippets.
[^site-workflow]: `site.yml` deploy `ROCCI_LANE` is `github.ref_name`.
[^prod-readme]: Two origin roots, cutover order, smoke curls on `:8080` and `:8081`.
[^ingress]: Staging hostnames to `:8081`; `rocci.dev` / `www` to `:8080`; no production example hosts.
