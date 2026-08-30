---
type: Implementation Plan
title: Deploy live example origins to staging
description: "Operator sequence to serve live-counter and datastar on Access-gated example hostnames: confirm shipped origin code, add proxied DNS and a hand-ordered Advanced certificate, promote main to staging so site.yml packages examples-live and origin publish starts hybrid plus live apps, then smoke TLS and Host isolation. Do not advertise Launch or promote production until staging health is green."
tags: [domain/rocci, concern/publication, concern/developer-experience, audience/maintainer]
status: draft
generated: { by: process:cursor, at: 2026-08-30T08:50:00Z }
stale_after: 2026-11-29
authority: exploratory
owners: [human:nils]
sources:
  - id: origins-plan
    resource: publish-example-origins.md
    title: Publish live examples on id.examples.rocci.dev
    author: process:cursor
    last_modified: 2026-08-29
  - id: play-path
    resource: live-examples-play-path.md
    title: Serve live examples at /play/id on the site host
    author: process:cursor
    last_modified: 2026-08-29
  - id: publish-plan
    resource: rocci-dev-publish.md
    title: Deploy rocci.dev with Cloudflare, a small VPS, and CI
    author: process:cursor
    last_modified: 2026-08-24
  - id: prod-readme
    resource: ../../../docker/prod/README.md
    title: Origin promote, Tunnel, and Access notes
    author: process:git
    last_modified: 2026-08-29
  - id: tunnel-ingress
    resource: ../../../docker/prod/cloudflared-ingress.yml.example
    title: Tunnel ingress including example wildcards
    author: process:git
    last_modified: 2026-08-29
  - id: site-workflow
    resource: ../../../.github/workflows/site.yml
    title: Site package and deploy from staging or production
    author: process:git
    last_modified: 2026-08-29
  - id: origin-ops
    resource: ../../../tools/rocci-ops/src/rocci_ops/origin.py
    title: Origin unpack, hybrid plus examples compose, Host health
    author: process:git
    last_modified: 2026-08-29
  - id: origin-compose
    resource: ../../../docker/compose.origin.yml
    title: Origin live-counter and datastar services
    author: process:git
    last_modified: 2026-08-29
  - id: cdn-caddy
    resource: ../../../docker/cdn/Caddyfile
    title: Hybrid Caddy Host matchers for example names
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

# Deploy live example origins to staging

Operator follow-on to [publish live examples](publish-example-origins.md).
That plan's code (catalog `site`, Launch on fixtures only, hybrid Host
routing, `examples-live` in package/deploy) is already on `main`. This
record is the remaining **deploy**: Cloudflare DNS/TLS, promote
`staging`, origin publish, and smoke. Writing it does not push
`staging` or change the dashboard.[^origins-plan][^site-workflow][^publish-plan]

Tunnel public hostnames and Cloudflare Access for
`*.examples.staging.rocci.dev` are **maintainer-reported done** as of
2026-08-29.[^tunnel-ingress][^prod-readme]

## Goal

`https://staging.rocci.dev/play/live-counter/health` and
`https://staging.rocci.dev/play/datastar/health` terminate trusted TLS
on the site host (Universal SSL; no ACM) through the existing Tunnel,
pass Access, and return **200** from that app's process. Deep example
hostnames remain an optional ACM path. `https://staging.rocci.dev/actions/`
still hits the home island. Generated `/examples/` copy stays
`planned live` until a later advertise phase.[^play-path][^origins-plan][^prod-readme][^cdn-caddy]

## Out of bound

- Product code: catalog flags, Launch injection, Caddy Host matchers,
  compose, or `site.yml` (already on `main`).
- Reconfiguring Tunnel ingress or Access policies (already done).
- [Origins plan](publish-example-origins.md) Phase 5: Launch hrefs,
  authored "reserved / not serving" copy, `docs.live-demo-hostnames`.
- `promote production` or treating public `*.examples.rocci.dev` as a
  gate.
- Flattening names to first-level hosts to avoid Advanced Certificate
  Manager.
- Enabling Total TLS as the issuance path for Tunnel hostnames.
- Running laptop `compose.examples.yml` `edge` on the VPS.
- Putting tokens, Tunnel credentials, or Access secrets in git or this
  record.

## Constraints that do not move

| Constraint | Required behavior |
| --- | --- |
| Advertise last | No generated `examples.rocci.dev` Launch href until staging example hosts have served TLS.[^origins-plan][^launch-audit] |
| Deploy lane | `site.yml` packages and deploys only from `staging` or `production`. Pushes to `main` are no-ops.[^site-workflow][^prod-readme] |
| Separate lanes | `staging` publishes `/srv/rocci/staging` on `:8081` (hybrid plus live apps). `production` publishes `/srv/rocci/prod` on `:8080` (hybrid only). Origin deploys are serialized.[^prod-readme] |
| Two origin ports | Production Tunnel targets `http://127.0.0.1:8080`. Staging and live-example hosts target `:8081`. Do not run `compose.examples.yml` `edge` on the VPS.[^tunnel-ingress][^cdn-caddy][^origin-compose] |
| Publish health | `origin publish` GETs site `/health`, each live id at `/play/<id>/health`, and `Host` for `<id>-example-staging.rocci.dev`, `<id>-example.rocci.dev`, and `<id>.examples.localhost`. Failure restores the previous release (hybrid + examples together).[^origin-ops][^play-path] |
| Proxy orange | Example DNS stays **Proxied**. Grey-cloud skips Cloudflare edge TLS. |
| Universal SSL is not enough | Full-setup Universal SSL covers `rocci.dev` and `*.rocci.dev` only.[^cf-universal-ssl][^origins-plan][^launch-audit] |
| One wildcard, one label | Staging apps need `*.examples.staging.rocci.dev`, not only `*.examples.rocci.dev`.[^cf-acm] |
| ACM, not Total TLS | Total TLS does not issue certificates for Cloudflare Tunnel hostnames. Order **Advanced** certificates by hand.[^cf-total-tls][^cf-acm] |
| Access stays on | Staging example hosts stay gated like `staging.rocci.dev`. Signed-out curls 302 after TLS succeeds.[^prod-readme] |

## What already shipped (do not redo)

On `origin/main` (PR #61 plus later CI pins):

- Catalog `site`, `--print-live`, fixture Launch only; repo `/examples/`
  still says `planned live`.[^origins-plan]
- `docker/cdn/Caddyfile` Host matchers for live-counter and datastar on
  production, staging, and localhost names, before site `/actions/`.[^cdn-caddy]
- `docker/compose.origin.yml` live-counter and datastar services; merge
  with hybrid; no examples `edge`.[^origin-compose]
- `package site` / `site.yml` upload `dist/examples-live/**`.
  `deploy push` + `origin publish` unpack those contexts, `compose up`
  hybrid **and** live apps, health-check, flip `current`.[^site-workflow][^origin-ops]

Hosted CI and Knowledge on `main` are green as of 2026-08-29. `site.yml`
on `main` is the expected 0s no-op.

`origin/staging` last packaged 2026-08-23 and does **not** include this
origin path. Until Phase 3, the VPS still serves the old hybrid-only
release.

## Current evidence (2026-08-29)

Maintainer-reported: Tunnel ingress matches the checked-in
`docker/prod/cloudflared-ingress.yml.example`; Access Allow + Service
Auth already apply to `*.examples.staging.rocci.dev`.[^tunnel-ingress]

Public resolver:

- `staging.rocci.dev` resolves; `/health` is Access **302**.
- `live-counter.examples.staging.rocci.dev` and
  `datastar.examples.staging.rocci.dev` have **no DNS**.

## Decision: DNS and cert first, then one staging promote

Keep `<id>.examples.{rocci.dev,staging.rocci.dev}`. Do not rename to
first-level labels for Universal SSL.[^origins-plan]

1. Proxied wildcard CNAMEs to the same Tunnel as `staging.rocci.dev`.
2. One Advanced certificate: apex + both example wildcards.[^cf-acm]
3. Promote `origin/main` → `staging` so `site.yml` is the only publisher.
   Do not scp artifacts by hand.[^prod-readme]
4. Smoke Access + Host isolation. Stop. Production and Launch wait.

DNS/TLS can overlap with waiting for the Site run; do not treat a 502
after a good handshake as a certificate bug.

## Phase 0 — Confirm preconditions

**Bound:** read-only git, Actions, Zero Trust. No DNS, cert, or promote.

**Work**

1. `origin/main` contains `docker/compose.origin.yml` and Caddy example
   Host matchers. Hosted CI on that SHA is green.
2. Zero Trust → Tunnels: published hostnames include
   `*.examples.staging.rocci.dev` and first-level
   `<id>-example-staging.rocci.dev`, service
   `http://127.0.0.1:8081`.[^tunnel-ingress]
3. Access: Self-hosted app for `*.examples.staging.rocci.dev` with the
   same Allow (maintainer email) and Service Auth as
   `staging.rocci.dev`.[^prod-readme]
4. Copy the Tunnel CNAME target from the `staging.rocci.dev` DNS row
   (typically `<uuid>.cfargotunnel.com`).
5. Remember: promoting `staging` publishes `/srv/rocci/staging` on
   `:8081`. Production stays on `/srv/rocci/prod` `:8080`. Do not start Phase
   3 during an unrelated production publish.[^prod-readme]

**Exit**

- Written down: Tunnel CNAME target; Access already covers staging
  example hosts; `main` SHA you will promote.
- No dashboard or git writes in this phase.

## Phase 1 — Proxied wildcard DNS

**Bound:** Cloudflare DNS for zone `rocci.dev`. No certificate order,
no promote.

**Work**

**DNS → Records**, **Proxied** (orange cloud):

| Type | Name | Content | Proxy |
| --- | --- | --- | --- |
| CNAME | `*.examples.staging` | same as `staging.rocci.dev` | Proxied |
| CNAME | `*.examples` | same as `staging.rocci.dev` | Proxied |

Do not create an A/AAAA to the VPS. Do not grey-cloud. Names
`examples.staging` / `examples` are not required for health checks.

**Exit**

```sh
dig +short CNAME live-counter.examples.staging.rocci.dev
dig +short CNAME datastar.examples.staging.rocci.dev
```

Both return the Tunnel CNAME (or Cloudflare flattening). Empty `dig`
means this phase is incomplete. TLS may still fail until Phase 2.

## Phase 2 — Advanced Certificate Manager

**Bound:** SSL/TLS → Edge Certificates. No Total TLS. No promote.

**Work**

1. If the zone lacks Advanced Certificate Manager, add the paid
   add-on. Universal SSL stays for `staging.rocci.dev`.[^cf-acm][^cf-universal-ssl]
2. **Order Advanced Certificate** with at least:

   - `rocci.dev` (apex must be on the cert)
   - `*.examples.rocci.dev`
   - `*.examples.staging.rocci.dev`

   One wildcard SAN is one label.[^cf-acm]
3. HTTP or TXT DCV. Wait until the certificate is **Active**.
4. Do **not** use Total TLS for these names (Tunnel hostnames are
   skipped).[^cf-total-tls]

**Exit**

```sh
echo | openssl s_client -servername live-counter.examples.staging.rocci.dev \
  -connect live-counter.examples.staging.rocci.dev:443 2>/dev/null \
  | openssl x509 -noout -subject -ext subjectAltName
```

The cert lists `*.examples.staging.rocci.dev` (or the specific
hostname). A private window must not show
NET::ERR_CERT_COMMON_NAME_INVALID.

Signed-out:

```sh
curl -sI https://live-counter.examples.staging.rocci.dev/health | head
```

Expect **HTTP/2 302** to Access (TLS ok). Handshake errors are this
phase. 502/521 after a good handshake is Phase 3 (origin not published
yet).

## Phase 3 — Promote staging and origin publish

**Bound:** git promote and `site.yml`. No Phase 5 copy. No production
promote. No laptop `compose.examples.yml` on the VPS.

**Work**

1. Fetch so local `main` matches `origin/main`.
2. Promote (rebases local `staging` onto local `main` and pushes):

   ```sh
   git fetch origin
   uv run --no-dev rocci-ops promote staging
   ```

   Equivalent: `git push origin origin/main:staging`.[^prod-readme]
3. Watch **Site** on branch `staging` (not `main`):

   - Package: `rocci-ops package site --target x64musl`; artifact
     includes `dist/site.tgz`, `dist/islands`,
     `dist/examples-live/**`.[^site-workflow]
   - Deploy (Environment `staging`): Access SSH probe, `deploy push`
     of those artifacts, remote `origin publish SHA`.
4. On the box, publish unpacks to `releases/<sha>/`, copies
   `examples-live/<id>/` (server, assets, Dockerfile),
   `compose up -d --build` of hybrid **plus**
   `docker/compose.origin.yml`, then loopback health. Failure leaves
   `current` on the previous release.[^origin-ops][^origin-compose][^prod-readme]

**Exit**

- Site workflow on `staging` succeeded for the promoted SHA.
- Deploy log shows `examples-live` files and origin health **200** for
  the site and `Host: live-counter.examples.localhost` /
  `Host: datastar.examples.localhost`.[^origin-ops]

If package or health fails, do not start Phase 4 public curls as a
success gate. Inspect the failed job; do not run
`compose.examples.yml` `edge` on the VPS.

## Phase 4 — Staging smoke through Access

**Bound:** HTTPS to staging example hosts and `staging.rocci.dev`. No
production promote. No Launch advertising.

**Work**

The Access-gated gate that does **not** need ACM is the first-level
example Hosts.[^play-path]

Signed-out (TLS + Access only):

```sh
curl -sI https://live-counter-example-staging.rocci.dev/health
curl -sI https://datastar-example-staging.rocci.dev/health
```

Expect **302** to Cloudflare Access (or NXDOMAIN until DNS exists).

With Service Auth (`CF-Access-Client-Id` /
`CF-Access-Client-Secret`) or a signed-in browser, expect **200** on
both `/health` URLs, and `/assets/datastar.js` must not 404.

Deep example hosts (`https://live-counter.examples.staging.rocci.dev/health`)
still need ACM. They are not this plan's advertise gate.

Confirm isolation:

- `https://staging.rocci.dev/actions/` still serves the **home
  island**, not a gallery app.[^origins-plan][^cdn-caddy]
- Optional VPS loopback if public 502 after a green Site job:

  ```sh
  curl -sf http://127.0.0.1:8081/health
  curl -sf http://127.0.0.1:8081/play/live-counter/health
  curl -sf http://127.0.0.1:8081/play/datastar/health
  curl -sf -H 'Host: live-counter-example-staging.rocci.dev' http://127.0.0.1:8081/health
  curl -sf -H 'Host: datastar-example-staging.rocci.dev' http://127.0.0.1:8081/health
  curl -sf -H 'Host: live-counter.examples.localhost' http://127.0.0.1:8081/health
  curl -sf -H 'Host: datastar.examples.localhost' http://127.0.0.1:8081/health
  ```

  Those are the same checks `origin publish` already ran.[^prod-readme][^origin-ops]

Do not require two-tab live-counter updates as a Phase 4 exit (that is
origins-plan Phase 5 browser work).

**Exit**

- Both `https://<id>-example-staging.rocci.dev/health` URLs are TLS 200
  through Access. That is the gate that does not need ACM.[^play-path]
- Home-island `/actions/` still works on `staging.rocci.dev`.
- Production example names may still fail; ignore them.
- Do not start [origins plan](publish-example-origins.md) Phase 5.

## After staging is green (not this plan)

When Phase 4 exit is met, a **separate** change can advertise Launch
and promote `production`. That work lives in
[publish-example-origins](publish-example-origins.md) Phase 5 and
[public-launch operator](public-launch-operator.md). This record stops
at Access-gated staging health.

## Roll-forward and rollback

DNS and certificates can land before origin publish. If ACM lags, keep
`planned live` and do not promote production.

Failed `origin publish` restores the previous `current` symlink
(hybrid + examples together). Removing the two CNAME wildcards restores
"no DNS"; leave the Advanced certificate unless abandoning the
hostname contract.[^origin-ops][^prod-readme]

[^origins-plan]: Code Phases 0–4 on main; advertise is Phase 5; `*.rocci.dev` is not enough.
[^play-path]: `/play/<id>/` leftover; ACM-free smoke is `<id>-example-staging.rocci.dev`.
[^publish-plan]: Cloudflare is DNS and edge TLS; Tunnel to loopback Caddy; Access on staging.
[^prod-readme]: Promote `main` → `staging`; separate `/srv/rocci/prod` and `/srv/rocci/staging`; example wildcards need their own certs.
[^tunnel-ingress]: Sample ingress lists staging hosts to `:8081` and `rocci.dev` to `:8080`.
[^site-workflow]: Package and deploy only when `ref` is `staging` or `production`; uploads `examples-live/**`.
[^origin-ops]: Unpack examples-live; compose hybrid plus origin examples; Host health; rollback previous release.
[^origin-compose]: live-counter and datastar services; cdn waits until they are healthy.
[^cdn-caddy]: Example Host matchers before site `/actions/` and `/sse`.
[^launch-audit]: Reserved example hosts failed TLS and must not be linked as live demos.
[^cf-universal-ssl]: Full setup Universal SSL is apex plus first-level subdomains only.
[^cf-acm]: Advanced certificates cover multi-level names; each wildcard is one label; apex required on the cert.
[^cf-total-tls]: Total TLS does not issue for Cloudflare Tunnel hostnames; order Advanced certificates instead.
