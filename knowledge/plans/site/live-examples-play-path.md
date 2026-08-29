---
type: Implementation Plan
title: Serve live examples at /play/id on the site host
description: "Route live-counter and datastar at /play/<id>/ on staging.rocci.dev and later rocci.dev so Universal SSL and Access apply with no extra DNS. Strip that prefix in Caddy before islands /actions/. Keep /examples/<id>/ as docs. Do not advertise Launch until staging /play/ health is TLS 200. Exploratory."
tags: [domain/rocci, concern/publication, concern/developer-experience, audience/maintainer]
status: draft
generated: { by: process:cursor, at: 2026-08-29T18:45:00Z }
stale_after: 2026-11-29
authority: exploratory
owners: [human:nils]
sources:
  - id: background
    resource: ../../research/site/live-examples-play-path.md
    title: Front doors for live example apps without paid wildcard TLS
    author: process:cursor
    last_modified: 2026-08-29
  - id: origins-plan
    resource: publish-example-origins.md
    title: Publish live examples on id.examples.rocci.dev
    author: process:cursor
    last_modified: 2026-08-29
  - id: deploy-plan
    resource: example-origin-cloudflare-tls.md
    title: Deploy live example origins to staging
    author: process:cursor
    last_modified: 2026-08-29
  - id: cdn-caddy
    resource: ../../../docker/cdn/Caddyfile
    title: Hybrid Caddy Host matchers and site /actions/
    author: process:git
    last_modified: 2026-08-29
  - id: origin-ops
    resource: ../../../tools/rocci-ops/src/rocci_ops/origin.py
    title: Origin health checks by Host examples.localhost
    author: process:git
    last_modified: 2026-08-29
  - id: origin-compose
    resource: ../../../docker/compose.origin.yml
    title: Origin live-counter and datastar services
    author: process:git
    last_modified: 2026-08-29
  - id: example-tests
    resource: ../../../tools/rocci-ops/tests/test_example_origins.py
    title: Caddy Host isolation tests
    author: process:git
    last_modified: 2026-08-29
  - id: prod-readme
    resource: ../../../docker/prod/README.md
    title: Promote staging and origin smoke
    author: process:git
    last_modified: 2026-08-29
  - id: site-workflow
    resource: ../../../.github/workflows/site.yml
    title: Site package and deploy from staging or production
    author: process:git
    last_modified: 2026-08-29
  - id: stage-rs
    resource: ../../../crates/rocci-docs/src/stage.rs
    title: live_demo_url and Launch hrefs
    author: process:git
    last_modified: 2026-08-29
---

# Serve live examples at `/play/<id>/` on the site host

Follow-on to [publish live examples](publish-example-origins.md) (code on
`main`) and the TLS comparison in
[front doors without paid wildcard TLS](/research/site/live-examples-play-path.md).
Writing this plan does not edit Caddy or promote `staging`.[^background][^origins-plan]

## Goal

`https://staging.rocci.dev/play/live-counter/` and
`https://staging.rocci.dev/play/datastar/` reverse-proxy to those origin
containers. Datastar `/actions/` and `/sse` under that prefix hit the
app, not the home island. `/examples/<id>/` stays docs. Later the same
paths work on `https://rocci.dev/play/<id>/`. No ACM and no extra
DNS.[^background][^cdn-caddy]

## Out of bound

- Advanced Certificate Manager, wildcard example DNS, or Cloudflare API
  sync from CI (optional later; see [ACM deploy plan](example-origin-cloudflare-tls.md)).[^deploy-plan]
- Mounting live apps at `/examples/<id>/`.
- Gallery hosts (`examples.staging.rocci.dev`, `examples-staging.rocci.dev`).
- Per-app first-level hosts (`live-counter-staging.rocci.dev`).
- [Origins](publish-example-origins.md) Phase 5 Launch advertising until
  Phase 3 of **this** plan is green.
- `promote production`.
- Changing handler syntax, SQLite volumes, or dropping Host matchers
  for `*.examples.localhost` in the first Caddy change (keep them for
  laptop compose).
- Running `compose.examples.yml` `edge` on the VPS.[^origin-compose][^prod-readme]

## Constraints that do not move

| Constraint | Required behavior |
| --- | --- |
| Advertise last | No public Launch href to `/play/` until staging play health is TLS 200 through Access.[^origins-plan][^stage-rs] |
| Docs vs play | `/examples/<id>/` is Rocdown. `/play/<id>/` is the live process.[^background] |
| Islands isolation | Bare `/actions/*` and `/sse` on the site Host stay islands. Play-prefixed paths must not use that handle.[^cdn-caddy] |
| Strip prefix | After `handle_path /play/<id>/*`, the app sees `/`, `/actions/`, `/sse`, `/health` as on a dedicated Host.[^cdn-caddy] |
| Deploy lane | `site.yml` only from `staging` / `production`. Shared `/srv/rocci`.[^site-workflow][^prod-readme] |
| Publish health | Failed live health must not flip `current`; rollback hybrid + examples together.[^origin-ops] |
| Tests | Caddy and origin-health contracts stay in rocci-ops tests; no Roc required.[^example-tests] |

## Decision

Chosen front door is **C** from the background record. Staging already
has Universal SSL and Access. Path `/play/<id>/` avoids the docs tree
and avoids paid multi-level certs.[^background]

## Phase 0 — Caddy play routes

**Bound:** [docker/cdn/Caddyfile](../../../docker/cdn/Caddyfile) and
[tools/rocci-ops/tests/test_example_origins.py](../../../tools/rocci-ops/tests/test_example_origins.py).
No origin.py, no Launch hrefs, no promote.

**Work**

1. Before `handle /actions/*`, add `handle_path` for each `--print-live`
   id (`live-counter`, `datastar`) to that container `:8000` with
   `flush_interval -1`.
2. Redirect `/play/<id>` (no slash) to `/play/<id>/`.
3. Keep existing example **Host** matchers above those handles.
4. Tests: play path appears before islands `/actions/`;
   `handle_path /play/live-counter` proxies `live-counter:8000`; the
   islands `/actions/` block still does not mention live-counter or
   datastar; Host matchers still present.[^example-tests][^cdn-caddy]

**Exit**

```sh
uv run --no-dev rocci-ops test example-origins
```

- A reviewer can see `/play/live-counter/` in the Caddyfile above
  islands `/actions/`.

## Phase 1 — Origin health on play paths

**Bound:** [tools/rocci-ops/src/rocci_ops/origin.py](../../../tools/rocci-ops/src/rocci_ops/origin.py)
and its tests. No Launch. No promote.

**Work**

1. For each live id, health-check
   `http://127.0.0.1:{port}/play/<id>/health` (no special Host
   required).
2. Keep site `GET /health` (islands). Optionally keep
   `Host: <id>.examples.localhost` checks while Host matchers remain.
3. [docker/prod/README.md](../../../docker/prod/README.md) origin smoke
   lists the play `/health` curls.[^origin-ops][^prod-readme]

**Exit**

```sh
uv run --no-dev rocci-ops test example-origins
```

- `health_checks` includes `/play/live-counter/health` and
  `/play/datastar/health`.

## Phase 2 — Operator copy only

**Bound:** knowledge + docker README pointers. No `live_demo_url`
change. No Phase 5 advertise.

**Work**

1. Point [deploy live example origins](example-origin-cloudflare-tls.md)
   staging smoke at `/play/<id>/health` as the gate that does not need
   ACM.
2. Note in [origins plan](publish-example-origins.md) that the public
   live URL is `/play/<id>/` until or unless Host-per-app returns.

**Exit**

```sh
cargo run -q --no-default-features --manifest-path ../okmate/Cargo.toml -p okmate -- check knowledge --profile strict --format terminal
```

- Those two records cite this plan. No generated Launch `/play/` on
  repo staging fixtures yet.[^stage-rs][^origins-plan]

## Phase 3 — Promote staging and smoke

**Bound:** git promote and Site workflow. No production. No Launch
advertise.

**Work**

1. `git fetch origin` and
   `uv run --no-dev rocci-ops promote staging` (or
   `git push origin origin/main:staging`).[^prod-readme]
2. Watch Site on `staging`: package `examples-live`, `origin publish`,
   play-path health.[^site-workflow][^origin-ops][^origin-compose]
3. Signed-out:
   `curl -sI https://staging.rocci.dev/play/live-counter/health` is
   Access **302**.
4. With Service Auth or a signed-in browser: both play `/health` URLs
   are **200**.
5. `https://staging.rocci.dev/actions/` still home island.
   `https://staging.rocci.dev/examples/live-counter/` still docs.

**Exit**

- Site workflow succeeded on the promoted SHA.
- Play health 200 through Access; docs and islands unchanged.

Then [origins](publish-example-origins.md) Phase 5 may set Launch to
`/play/<id>/`. Do not start that in this phase.[^origins-plan]

## Roll-forward and rollback

Caddy-only Phase 0 can land on `main` before promote; play paths 404
until origin compose is on the box. Failed publish still restores
`current`. Revert the Caddyfile to drop `/play/` without touching
Cloudflare.[^origin-ops]

[^background]: Why ACM, Total TLS, and `/examples/` collision; decision C.
[^origins-plan]: Code on main; advertise after a live URL serves TLS.
[^deploy-plan]: ACM/DNS operator path; not required for `/play/`.
[^cdn-caddy]: Host matchers then islands `/actions/` and `/sse`.
[^origin-ops]: Health and rollback of hybrid plus examples.
[^origin-compose]: live-counter and datastar services; no examples edge.
[^example-tests]: Isolation assertions on the hybrid Caddyfile.
[^prod-readme]: Promote staging; shared origin; Access on staging.rocci.dev.
[^site-workflow]: Package and deploy only on staging or production.
[^stage-rs]: Launch hrefs unused on repo catalog until advertise.
