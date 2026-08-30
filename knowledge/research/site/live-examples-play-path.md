---
type: Research Report
title: Front doors for live example apps without paid wildcard TLS
description: "Universal SSL covers only rocci.dev and one label. Deep example hosts need ACM; Total TLS skips Tunnel. /play/<id>/ on the site host 404s /assets/datastar.js and sends /actions/ to islands. The TLS-free Host front door is <id>-example-staging.rocci.dev. Exploratory."
tags: [domain/rocci, concern/publication, concern/architecture, audience/maintainer]
status: draft
generated: { by: process:cursor, at: 2026-08-30T08:50:00Z }
stale_after: 2026-11-29
authority: exploratory
owners: [human:nils]
sources:
  - id: origins-plan
    resource: ../../plans/site/publish-example-origins.md
    title: Publish live examples on id.examples.rocci.dev
    author: process:cursor
    last_modified: 2026-08-29
  - id: deploy-plan
    resource: ../../plans/site/example-origin-cloudflare-tls.md
    title: Deploy live example origins to staging
    author: process:cursor
    last_modified: 2026-08-29
  - id: play-plan
    resource: ../../plans/site/live-examples-play-path.md
    title: Serve live examples at /play/id on the site host
    author: process:cursor
    last_modified: 2026-08-29
  - id: prod-readme
    resource: ../../../docker/prod/README.md
    title: Origin promote, Tunnel, Access, wildcard cert note
    author: process:git
    last_modified: 2026-08-29
  - id: cdn-caddy
    resource: ../../../docker/cdn/Caddyfile
    title: Hybrid Caddy Host matchers and site /actions/
    author: process:git
    last_modified: 2026-08-29
  - id: launch-audit
    resource: ../../audits/site/rocci-dev-public-launch.md
    title: rocci.dev public-launch checklist
    author: process:cursor
    last_modified: 2026-08-25
  - id: cf-universal-ssl
    resource: https://developers.cloudflare.com/ssl/edge-certificates/universal-ssl/
    title: Cloudflare Universal SSL
    author: organization:cloudflare
    last_modified: 2026-08-14
  - id: cf-acm
    resource: https://developers.cloudflare.com/ssl/edge-certificates/advanced-certificate-manager/
    title: Cloudflare Advanced certificates
    author: organization:cloudflare
    last_modified: 2026-08-14
  - id: cf-total-tls
    resource: https://developers.cloudflare.com/ssl/edge-certificates/additional-options/total-tls/
    title: Cloudflare Total TLS
    author: organization:cloudflare
    last_modified: 2026-04-16
---

# Front doors for live example apps without paid wildcard TLS

Pair: [serve live examples at `/play/<id>/`](/plans/site/live-examples-play-path.md).
Not an approved hosting contract. Does not change Caddy or promote
`staging`.[^play-plan]

## What is already true

[Publish live examples](/plans/site/publish-example-origins.md) Phases 0–4
are on `main`: catalog `site`, fixture-only Launch, hybrid Host matchers
for `<id>.examples.{rocci.dev,staging.rocci.dev,localhost}`, origin
compose for live-counter and datastar, `examples-live` in `site.yml`.
Repo `/examples/` still says `planned live`. Advertise is Phase 5, after
a live URL has served TLS.[^origins-plan][^cdn-caddy]

`staging.rocci.dev` resolves, is Tunnel-published, and `/health` returns
Access 302. Universal SSL covers that first-level name.[^prod-readme][^cf-universal-ssl]

A Tunnel public hostname for `*.examples.staging.rocci.dev` did **not**
create a DNS row. `dig` for `live-counter.examples.staging.rocci.dev`
was empty on 2026-08-29. Concrete Tunnel DNS rows (apex, `www`,
`staging`) were created automatically; the wildcard was not.[^deploy-plan]

`origin/staging` last packaged 2026-08-23 and does not include origin
live-app compose. Promoting `staging` still required for the VPS to run
those containers. `staging` is `/srv/rocci/staging` on `:8081`;
`production` is `/srv/rocci/prod` on `:8080` without live-example origins.[^prod-readme]

## Why dedicated example hosts cost money

On a **full** Cloudflare zone, Universal SSL is the apex plus
`*.rocci.dev` (one label). It covers `staging.rocci.dev` and
`www.rocci.dev`. It does not cover:[^cf-universal-ssl][^launch-audit]

- `live-counter.examples.staging.rocci.dev` (three labels)
- `live-counter.staging.rocci.dev` (two labels)
- `examples.staging.rocci.dev` (two labels)

Each wildcard SAN is also one label. `*.rocci.dev` never equals
`*.examples.staging.rocci.dev`.[^cf-acm]

Total TLS can mint per-hostname certs for deep names, but **not** for
Cloudflare Tunnel hostnames. These apps are reached only through
Tunnel.[^cf-total-tls]

Advanced Certificate Manager can put `*.examples.staging.rocci.dev` (or
an explicit SAN list) on an Advanced certificate. That add-on is
paid.[^cf-acm][^prod-readme]

The DNS add-record UI has no Tunnel type. Wildcard routes need a
**CNAME** to `<tunnel-id>.cfargotunnel.com`, Proxied — or a first-level
name that Universal SSL already covers.[^deploy-plan]

CI could upsert an explicit `--print-live` DNS list. That fixes
resolution. It does **not** make three-label Tunnel names free.

## Options compared (2026-08-29)

| Id | Front door | TLS | Extra DNS | Product work |
| --- | --- | --- | --- | --- |
| A | `<id>.examples.staging.rocci.dev` | ACM | Per app or wildcard CNAME | Host matchers already in Caddy |
| B | `<id>-example-staging.rocci.dev` (later `<id>-example.rocci.dev`) | Universal | Per app CNAME | Host list, Launch URLs |
| C | `staging.rocci.dev/play/<id>/` | Universal (existing) | None | `handle_path` before islands `/actions/` |
| D deep | `examples.staging.rocci.dev/<id>` | ACM | One CNAME | Path + Host; `examples` reads as docs |
| D flat | `examples-staging.rocci.dev/<id>` | Universal | One CNAME | Path + extra Host; cookies shared |

[Origins plan](/plans/site/publish-example-origins.md) listed path-prefix
mounting as out of bound because a live app must not steal site
`/actions/` or `/sse`. Hybrid Caddy sends unmatched `/actions/*` to
islands. A play prefix works only if Caddy strips `/play/<id>` so the
app still sees `/` and `/actions/`, **before** the islands
handles.[^origins-plan][^cdn-caddy]

`/examples/<id>/` is the staged docs tree. Mounting the live process
there collides with the article. `/play/<id>/` does not.[^origins-plan]

On one site Host, cookies default to the whole host. Path-scoped
cookies under `/play/<id>` are safer; v1 may accept collision for
demos.

## Decision recorded here

**C** (`/play/<id>/`) shipped and proved the isolation hole: the HTML
still loads `/assets/datastar.js` and posts `/actions/…` on the site
Host. Staging play GET can be 200 while Increment does nothing.

Maintainer then chose **B**:
`https://live-counter-example-staging.rocci.dev` and
`https://datastar-example-staging.rocci.dev` (later
`https://<id>-example.rocci.dev`). First-level Universal SSL. One
proxied CNAME and Access app per name. Caddy Host matchers own `/`,
`/assets/`, and `/actions/`. Do not advertise Launch until those
staging hosts serve TLS 200 through Access.

**A** (deep `*.examples.staging`) stays optional ACM. `/play/` may
remain in Caddy; it is not the public front door.[^play-plan][^deploy-plan]

[^origins-plan]: Host-per-app contract; path prefix was out of bound; advertise after TLS.
[^deploy-plan]: Wildcard DNS gap; ACM vs Total TLS; promote staging for origin compose.
[^play-plan]: Chosen `/play/<id>/` implementation phases.
[^prod-readme]: Separate origin lanes; Universal SSL on first-level staging; ACM note for example wildcards.
[^cdn-caddy]: Host matchers then islands `/actions/` and `/sse`.
[^launch-audit]: Reserved example hosts failed TLS; do not advertise until they serve.
[^cf-universal-ssl]: Full setup Universal SSL is apex plus first-level subdomains.
[^cf-acm]: Multi-level names and extra wildcards need Advanced certificates; one wildcard is one label.
[^cf-total-tls]: Total TLS skips Cloudflare Tunnel hostnames.
