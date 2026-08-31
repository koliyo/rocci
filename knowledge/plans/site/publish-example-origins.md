---
type: Implementation Plan
title: Publish live examples on id.examples.rocci.dev
description: "Serve catalog live apps on dedicated <id>.examples.rocci.dev hostnames, add a site Launch control, and let catalog metadata include or exclude apps from the rocci.dev site build. Do not advertise a hostname until it serves TLS."
tags: [domain/rocci, domain/rocdown, concern/publication, concern/developer-experience, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-08-29T18:50:00Z }
stale_after: 2026-11-24
authority: exploratory
owners: [human:nils]
sources:
  - id: play-path
    resource: live-examples-play-path.md
    title: Serve live examples at /play/id on the site host
    author: process:cursor
    last_modified: 2026-08-29
  - id: app-docs-plan
    resource: ../rocdown/rocci-app-docs.md
    title: Documentation generator for Rocci applications
    author: process:cursor
    last_modified: 2026-08-24
  - id: launch-audit
    resource: ../../audits/site/rocci-dev-public-launch.md
    title: rocci.dev public-launch checklist
    author: process:cursor
    last_modified: 2026-08-23
  - id: publish-plan
    resource: rocci-dev-publish.md
    title: Deploy rocci.dev with Cloudflare, a small VPS, and CI
    author: process:cursor
    last_modified: 2026-08-21
  - id: catalog
    resource: ../../../examples/rocci/apps.toml
    title: Rocci example catalog
    author: process:git
    last_modified: 2026-08-22
  - id: catalog-rs
    resource: ../../../crates/rocci-docs/src/catalog.rs
    title: Catalog schema and hosting labels
    author: process:git
    last_modified: 2026-08-23
  - id: stage-rs
    resource: ../../../crates/rocci-docs/src/stage.rs
    title: Staging tree, live URL helpers, generated index
    author: process:git
    last_modified: 2026-08-23
  - id: stage-tests
    resource: ../../../crates/rocci-docs/tests/stage.rs
    title: Staging tests that forbid unserved live hostnames
    author: process:git
    last_modified: 2026-08-23
  - id: rocci-docs-readme
    resource: ../../../crates/rocci-docs/README.md
    title: rocci-docs public contract for planned live hostnames
    author: process:git
    last_modified: 2026-08-23
  - id: site-config
    resource: ../../../site/rocdown.toml
    title: rocci.dev mounts and Examples nav
    author: process:git
    last_modified: 2026-08-22
  - id: examples-caddy
    resource: ../../../docker/examples/Caddyfile
    title: Host routing for example origins
    author: process:git
    last_modified: 2026-08-21
  - id: examples-compose
    resource: ../../../docker/compose.examples.yml
    title: Local live-example Compose stack
    author: process:git
    last_modified: 2026-08-21
  - id: hybrid-compose
    resource: ../../../docker/compose.hybrid.yml
    title: Origin hybrid Caddy plus islands
    author: process:git
    last_modified: 2026-08-22
  - id: cdn-caddy
    resource: ../../../docker/cdn/Caddyfile
    title: Hybrid site Caddy on origin port 8080
    author: process:git
    last_modified: 2026-08-23
  - id: docker-readme
    resource: ../../../docker/README.md
    title: Docker hosting notes for planned example hostnames
    author: process:cursor
    last_modified: 2026-08-22
  - id: site-workflow
    resource: ../../../.github/workflows/site.yml
    title: Site package and deploy; hardcoded live binaries
    author: process:git
    last_modified: 2026-08-23
  - id: local-ops
    resource: ../../../rocci-ops/src/rocci_ops/site.py
    title: package site stages docs and builds live apps
    author: process:git
    last_modified: 2026-08-23
  - id: deploy-ops
    resource: ../../../rocci-ops/src/rocci_ops/deploy.py
    title: Deploy push of site.tgz and islands only
    author: process:git
    last_modified: 2026-08-22
  - id: origin-ops
    resource: ../../../rocci-ops/src/rocci_ops/origin.py
    title: Origin unpack and hybrid compose publish
    author: process:git
    last_modified: 2026-08-22
  - id: tunnel-ingress
    resource: ../../../docker/prod/cloudflared-ingress.yml.example
    title: Tunnel ingress without example hostnames
    author: process:git
    last_modified: 2026-08-21
  - id: coverage
    resource: ../../../docs/coverage.toml
    title: docs.live-demo-hostnames still planned
    author: process:git
    last_modified: 2026-08-22
  - id: live-counter-docs
    resource: ../../../examples/rocci/standalone/live-counter/index.rocdown
    title: Authored live-counter page; hostname reserved not serving
    author: process:git
    last_modified: 2026-08-23
  - id: link-card
    resource: ../../../crates/rocci-rocdown/templates/DocsComponents.rocci
    title: Existing :link-card widget
    author: process:git
    last_modified: 2026-08-22
---

# Publish live examples on `<id>.examples.rocci.dev`

## Purpose and authority

The [app-docs plan](/plans/rocdown/rocci-app-docs.md) already staged `/examples/<id>/` and
reserved live origins. That work is in tree: catalog `hosting = "docs" |
"live"`, `rocci-docs --print-live`, musl binaries under `dist/examples-live/`,
and a laptop Compose file that routes by `Host`. Public copy still says the
names are reserved. Tests forbid emitting `examples.rocci.dev` URLs. The
origin publish path unpacks `site.tgz` and `islands` only. Cloudflare ingress
has no example hostnames. Probes of the reserved names fail TLS.[^app-docs-plan][^catalog][^stage-tests][^rocci-docs-readme][^origin-ops][^tunnel-ingress][^launch-audit]

This follow-on finishes serving, adds a site **Launch** control, and splits
"in the catalog" from "on the public site." It is exploratory. Writing it
does not attach DNS, issue certificates, or advertise a hostname.

## Goal

A catalog row can opt into the rocci.dev `/examples/` tree, opt out of it, or
opt into a live origin. The TLS-free public live URL is
`https://<id>-example-staging.rocci.dev` (later
`https://<id>-example.rocci.dev`). Deep `<id>.examples.*` names still
need ACM. `/play/<id>/` on the site host is leftover and does not own
`/assets/` or `/actions/`.[^play-path] Live apps that are on the site get a Launch control
after advertise; default hostname helpers may still say
`https://<id>.examples.rocci.dev`. Docs-only apps have no Launch control.
The hybrid site keeps `/actions/` and `/sse` on `rocci.dev`.[^stage-rs][^examples-caddy][^cdn-caddy]

## Out of bound

- Path-prefix mounting (`/examples/<id>/app/`) or rewriting Datastar URLs.
- Hosting every catalog app, or hosting snake as a public origin tenant.
- A generator for Rocdown examples (`examples/rocdown/**`).
- A new Rocdown block kind, Datastar launch island, or theme rewrite.
- Changing handler syntax, SQLite, or live SSE policy.
- Making rocci.dev itself a dynamic Rocci application.
- Apex DNS, repository visibility, or the rest of the public-launch operator
  sequence.[^launch-audit]
- Committing `dist/` trees.

## Constraints that do not move

| Constraint | Required behavior |
| --- | --- |
| Catalog | `examples/rocci/apps.toml` remains the inventory. Discovery is not "every directory under `examples/rocci`".[^catalog] |
| Site inclusion | A row is staged into `/examples/` only when `site = true`. Default `true` so today's catalog does not vanish. |
| Live isolation | A live example is its own process and hostname. It must not steal `rocci.dev` `/actions/` or `/sse`.[^cdn-caddy][^examples-caddy] |
| Advertise last | Do not emit `https://<id>.examples.rocci.dev` from generated or authored public pages until a staging deploy has served that name with TLS.[^stage-tests][^launch-audit] |
| Invalid combo | `hosting = "live"` requires `site = true` (nowhere to put Launch). |
| Artifacts | Failed example health must not replace a previously valid hybrid origin. Failed docs staging must not replace `dist/example-docs`.[^origin-ops][^app-docs-plan] |
| Ownership | `rocci-docs` inventories and writes Rocdown. Rocdown mounts and paints. Origin compose and Caddy stay in `docker/` and `rocci-ops`. |
| One origin port | Production Tunnel targets loopback `8080`. Example Host matchers belong on that Caddy, not a second `:8080` edge.[^tunnel-ingress][^hybrid-compose] |
| Wildcard TLS | Cloudflare `*.rocci.dev` does not cover `*.examples.rocci.dev`. Example names need their own certificate coverage. That gap is why reserved hosts fail TLS today.[^launch-audit] |
| Tests | Catalog, staging, and nav-contract checks do not require Roc or a server. |

## Current evidence

| Piece | State |
| --- | --- |
| Catalog `hosting` | `docs` or `live` only. Live rows: `live-counter`, `datastar`.[^catalog] |
| Generated index | Labels live rows `planned live`. No `examples.rocci.dev` strings.[^catalog-rs][^stage-tests] |
| `live_demo_url` | Helper returns `https://{id}.examples.rocci.dev`; unused by the index.[^stage-rs] |
| Authored pages | "hostname is reserved and is not serving yet."[^live-counter-docs] |
| Package site | Builds live musl servers into `dist/examples-live/<id>/`. Hardcoded docs-only exclusion list. Workflow uploads those two servers.[^local-ops][^site-workflow] |
| Deploy / origin | scp `site.tgz` + `islands`. Unpack ignores example binaries. Compose is hybrid only.[^deploy-ops][^origin-ops] |
| Tunnel | `staging.rocci.dev`, `rocci.dev`, `www.rocci.dev` → `:8080`. No example names.[^tunnel-ingress] |
| Site nav | Hand list of eight example indexes; must stay in lockstep with staged apps.[^site-config] |
| Coverage | `docs.live-demo-hostnames` status `planned`.[^coverage] |

## Decision: `site` flag, Launch, and origin Host

### Catalog

```toml
[[app]]
id = "live-counter"
path = "standalone/live-counter"
title = "Live counter"
# ...existing fields...
site = true          # include in rocci.dev /examples/ (default true)
hosting = "live"     # package a live origin (docs | live)
```

| `site` | `hosting` | Staging | Live package | Launch |
| --- | --- | --- | --- | --- |
| `true` (default) | `docs` | `/examples/<id>/` | no | no |
| `true` | `live` | `/examples/<id>/` | yes | after advertise phase |
| `false` | `docs` | omitted | no | no |
| `false` | `live` | **error** | — | — |

`site = false` keeps the app in the catalog for local inventory, coverage, and
CI path checks. `rocci-docs` skips it when writing `dist/example-docs`. An
optional `--all` flag stages excluded apps for local preview and must not be
what `package site` uses.

This plan does not pick exclusions. Current rows stay included until a
maintainer sets `site = false`.

`live_url` stays an optional override of the default hostname. It does not
replace `hosting` or `site`.

`--print-live` lists rows with `hosting = "live"` and `site = true` only.

### Launch control

`rocci-docs` injects Launch into the **staged** app `index.rocdown` and the
generated `/examples/` index. Do not edit checked-in example prose for the
button itself.

Use the existing `:link-card` with an external `href` (`title: "Launch"`). Do
not add a block kind. Same-tab navigation is enough; do not wait on
`target` support.[^link-card]

Until the advertise phase, live rows keep the `planned live` label and no
hostname href, matching today's tests.[^stage-tests]

### Origin

Laptop: keep `docker/compose.examples.yml` with its own edge for Host-header
demos. Do not run that edge on the VPS (`8080` is already hybrid Caddy).[^examples-compose][^hybrid-compose][^docker-readme]

VPS: add live-app services (no second Caddy) to the origin compose project.
Add Host matchers to `docker/cdn/Caddyfile` **before** the site handles:

- `<id>-example.rocci.dev` and `<id>-example-staging.rocci.dev` (Universal SSL)
- `<id>.examples.rocci.dev` and `<id>.examples.staging.rocci.dev` (ACM)

those reverse-proxy to that app. Default host (`rocci.dev`,
`staging.rocci.dev`) keeps today's island `/actions/` and `/sse`.[^cdn-caddy][^examples-caddy]

Operator work (same class as [rocci.dev publish](rocci-dev-publish.md), not a
product CLI) is sequenced in
[deploy live example origins to staging](example-origin-cloudflare-tls.md)
(first-level DNS and Access; optional ACM for deep names).
[Play path](live-examples-play-path.md) is leftover on the site host.
Tunnel ingress and Access for `staging.rocci.dev` are assumed already
configured.[^play-path][^publish-plan][^tunnel-ingress]

## Phase 0 — Catalog `site` field

**Bound**

- `examples/rocci/apps.toml` and `crates/rocci-docs` catalog parse only.
- No staging filter yet, no Launch hrefs, no origin or DNS.

**Work**

1. Add `site: bool` to `AppEntry`, serde default `true`.
2. Reject `hosting = "live"` when `site = false`.
3. Reject unknown keys that would hide a misspelling of `site`.
4. Document the field in `crates/rocci-docs/README.md` and a short comment
   above the catalog. Leave current rows implied-included.

**Exit**

```sh
cargo test -p rocci-docs
cargo fmt --all -- --check
```

- Fixture with `site = false` plus `hosting = "live"` fails catalog load.
- Fixture with `site = false` plus `hosting = "docs"` loads.
- Repo catalog still loads; default keeps every current app included.

## Phase 1 — Filter site staging and nav

**Bound**

- Staging output and Examples nav contract. No live URL advertising. No
  origin.

**Work**

1. `stage` writes only `site = true` apps. Generated index omits the rest.
2. `--print-live` omits `site = false` (already invalid if live).
3. `package site` uses `--print-live`; drop the hardcoded docs-only id
   list in `site.py`.
4. Optional `rocci-docs --all` stages excluded apps; `package site` and
   `build site` must not pass it.
5. Contract test: `site/rocdown.toml` Examples `items` (except
   `examples/index`) match `site = true` catalog ids. A `site = false` row
   must not remain in that nav list.

**Exit**

```sh
cargo test -p rocci-docs
uv run --no-dev rocci-ops test example-origins
cargo fmt --all -- --check
```

- Fixture catalog with one excluded app: staging tree and index omit it;
  `--print-live` omits it.
- Repo `package site` still stages today's eight apps until a row is
  excluded.

## Phase 2 — Launch injection (fixtures only)

**Bound**

- Generator and fixture tests. Repo catalog index must still omit
  `examples.rocci.dev` and keep `planned live`.[^stage-tests]

**Work**

1. For fixture apps with `hosting = "live"`, staged `index.rocdown` gains a
   `:link-card` titled Launch whose `href` is `app_play_url` (default
   `https://<id>.examples.rocci.dev`, or `live_url` when set).
2. Fixture generated catalog index includes that href in a Launch column.
3. Docs-only fixtures have no Launch card and no hostname href.
4. Repo staging path stays on `planned live` until Phase 5. Keep
   `catalog_index_does_not_advertise_unserved_live_hostnames` green.

**Exit**

```sh
cargo test -p rocci-docs
cargo fmt --all -- --check
```

- Fixture live app staged page contains `:link-card` and the default
  hostname.
- Repo `examples/rocci/apps.toml` staging still has no `examples.rocci.dev`.

## Phase 3 — Origin Host routing (local)

**Bound**

- Docker Caddy and Compose on a developer machine. No Cloudflare, no
  public advertising, no workflow advertise flip.

**Work**

1. Add Host matchers to `docker/cdn/Caddyfile` for each `--print-live` id
   (start with `live-counter` and `datastar`) on both production and
   staging example names. Matchers run before site `/actions/` handles.
2. Origin compose grows live-app services (health on `/health`, own
   SQLite volume, `ROC_BASIC_WEBSERVER_HOST=0.0.0.0`). No `edge` service
   from `compose.examples.yml`.
3. Keep `compose.examples.yml` as a laptop-only stack; README states it
   must not share the VPS `:8080` with hybrid Caddy.
4. Tests: example Caddyfile/cdn Caddyfile must not proxy `/actions/*` for
   the default site host; must proxy example hosts to app services;
   `snake` absent.

**Exit**

```sh
uv run --no-dev rocci-ops test example-origins
```

- `curl -H 'Host: live-counter.examples.localhost' http://127.0.0.1:8080/health`
  against the local hybrid+examples fixture returns 200.
- `curl -H 'Host: staging.rocci.dev' http://127.0.0.1:8080/actions/counter/...`
  still hits the home island, not the gallery (documented fixture or
  compose test).

## Phase 4 — Deploy, Tunnel, certificates

**Bound**

- Origin publish and Cloudflare operator steps. Do not flip generated
  Launch hrefs yet.

**Work**

1. `package site` / `site.yml` upload `dist/examples-live/*/server` (glob,
   not two hardcoded paths) plus app Docker context files the origin
   needs.
2. `deploy push` copies those artifacts. `origin publish` unpacks them
   beside islands, `compose up` hybrid **and** live apps, health-checks
   site `8080/health` **and** each live app. Failure rolls back the
   previous release (hybrid + examples together).
3. Maintainer: wildcard DNS, certificate for `*.examples.rocci.dev` and
   `*.examples.staging.rocci.dev`, Tunnel ingress for those names to
   `:8080`, Access on staging example hosts.[^publish-plan]
4. Do not treat laptop `compose.examples.yml` edge as production.

**Exit**

- Staging: `curl -I https://live-counter.examples.staging.rocci.dev/health`
  and `https://datastar.examples.staging.rocci.dev/health` are TLS 200
  through Access (or a Service Auth token).
- `https://staging.rocci.dev/actions/` still serves the home island.
- Production example names may still fail until Phase 5 and production
  DNS; do not advertise them.

## Phase 5 — Advertise Launch and public copy

**Bound**

- Generator, authored reserved-hostname sentences, READMEs, coverage,
  and the public-launch audit. Only after Phase 4 staging health is
  green.

**Work**

1. Generated `/examples/` index: live rows show Launch via
   `:link-card` / column using `app_play_url`. Public href is
   `https://<id>-example.rocci.dev` (or staging twin). Label `live`, not
   `planned live`.[^play-path]
2. Staged live app indexes get the Launch card. Remove "reserved and is
   not serving" from authored live-counter and datastar pages (replace
   with Launch plus local-run).
3. Invert `catalog_index_does_not_advertise_unserved_live_hostnames`:
   repo index **must** contain `https://live-counter.examples.rocci.dev`
   and **must not** say `planned live` for those rows.
4. `crates/rocci-docs/README.md`, `docker/README.md`,
   `docs/coverage.toml` (`docs.live-demo-hostnames` → `current`),
   [app-docs plan](/plans/rocdown/rocci-app-docs.md) live-origins note, and
   [public-launch audit](/audits/site/rocci-dev-public-launch.md) record that
   advertised hosts serve.

**Exit**

```sh
cargo test -p rocci-docs
uv run --no-dev rocci-ops test example-origins
cargo run -q -p rocci-okf -- check knowledge --profile base --format terminal
```

- Local `rocci-docs` then `rocdown check site` shows Launch on
  `/examples/live-counter/` and `/examples/datastar/`, omitted on
  `/examples/counter/`.
- Staging browser: Launch opens the example hostname; two-tab
  live-counter still updates.

## Roll-forward and rollback

Land `site` metadata and staging filters before origin routing. Land
origin TLS before advertising. If certificates or Tunnel lag, keep
`planned live` and the no-URL tests; a live-container failure must not
take down `rocci.dev` HTML.

Rollback advertising by reverting Phase 5 copy and restoring the
"reserved / not serving" sentences plus the no-URL staging test. Origin
rollback is the existing `current` symlink to the previous release.

## Suggested first exclusion (optional, not a phase gate)

If local or CI site builds need a smaller tree, `handler-matrix`,
`snake`, or `blocks` are the first `site = false` candidates (docs-only,
heavy, or not a public demo). Choosing them is a maintainer edit in
Phase 1, not a requirement of this plan.

[^play-path]: TLS-free live URL is `<id>-example-staging.rocci.dev`; `/play/<id>/` leftover.
[^app-docs-plan]: Phases 0–6 staged `/examples/`; live hostnames stayed planned until a staging deploy served them.
[^launch-audit]: 2026-08-23 Should pass: reserved example hosts fail TLS and must not be linked as live demos.
[^publish-plan]: Cloudflare Tunnel, Access-gated staging, VPS origin on loopback Caddy.
[^catalog]: Catalog rows include `hosting` plus audience metadata; no `site` field yet.
[^catalog-rs]: `Hosting::Live` public label is `planned live`.
[^stage-rs]: `live_demo_url` builds `https://{id}.examples.rocci.dev`; catalog index does not call it.
[^stage-tests]: Repo staging must not contain `examples.rocci.dev`; live ids are `live-counter` and `datastar`.
[^rocci-docs-readme]: Public contract: reserved names are not serving; table says `planned live`.
[^site-config]: Examples nav lists eight hardcoded app indexes plus `examples/index`.
[^examples-caddy]: Host matchers for live-counter and datastar; no site `/actions/` proxy.
[^examples-compose]: Separate example edge on `${ROCCI_HTTP_PORT:-8080}` for laptop demos.
[^hybrid-compose]: Origin hybrid Caddy plus islands already bind host `8080`.
[^cdn-caddy]: Site island `/actions/` and `/sse` live on the default host Caddyfile.
[^docker-readme]: Documents planned `<id>.examples.rocci.dev` Host routing.
[^site-workflow]: Uploads `dist/examples-live/live-counter/server` and `datastar/server` only.
[^local-ops]: `package site` stages example-docs, `--print-live`, then musl builds.
[^deploy-ops]: scp `site.tgz` and `islands` only.
[^origin-ops]: Unpack and hybrid compose; no example app context.
[^tunnel-ingress]: Example hostnames absent from Tunnel ingress sample.
[^coverage]: `docs.live-demo-hostnames` remains `planned`.
[^live-counter-docs]: Authored page states the dedicated hostname is not serving.
[^link-card]: `:link-card` already paints an `<a href>`.
