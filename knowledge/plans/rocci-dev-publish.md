---
type: Implementation Plan
title: Deploy rocci.dev with Cloudflare, a small VPS, and CI
description: "Put rocci.dev on Cloudflare (CDN, Universal SSL, Tunnel, mail) in front of a small amd64 VPS running hybrid Caddy plus islands artifacts. Human DNS, mail, VPS, Tunnel process, bootstrap SSH, GitHub Environment, deploy user, origin docker kit, and Access-gated staging route were reported in place. CI packages site/ and deploys from staging or production; main lands PRs. Exploratory."
tags: [domain/rocci, domain/rocdown, concern/publication, concern/ci, concern/architecture, integration/datastar]
status: draft
generated: { by: process:cursor, at: 2026-08-21T09:26:00Z }
stale_after: 2026-11-20
authority: exploratory
owners: [human:nils]
sources:
  - id: human-preparation
    resource: ../log.md
    title: Maintainer-reported rocci.dev deployment preparation
    author: human:nils
    last_modified: 2026-08-20
  - id: research
    resource: ../research/rocci-dev-publish.md
    title: Publishing rocci.dev with Cloudflare, a small origin, and CI
    author: process:cursor
    last_modified: 2026-08-20
  - id: site-config
    resource: ../../site/rocdown.toml
    title: Unified rocci.dev site configuration
    author: process:git
    last_modified: 2026-08-19
  - id: docs-config
    resource: ../../docs/rocdown.toml
    title: Documentation catalog configuration
    author: process:git
    last_modified: 2026-08-19
  - id: site-home
    resource: ../../site/index.rocdown
    title: rocci.dev home page with live counter island
    author: process:git
    last_modified: 2026-08-20
  - id: docker-readme
    resource: ../../docker/README.md
    title: Static and hybrid Docker hosting
    author: process:cursor
    last_modified: 2026-08-20
  - id: compose-hybrid
    resource: ../../docker/compose.hybrid.yml
    title: Pre-built hybrid Caddy plus islands Compose file
    author: process:cursor
    last_modified: 2026-08-20
  - id: static-caddy
    resource: ../../docker/static/Caddyfile
    title: Static file_server Caddyfile
    author: process:cursor
    last_modified: 2026-08-20
  - id: cdn-caddy
    resource: ../../docker/cdn/Caddyfile
    title: Hybrid same-origin Caddy reverse proxy
    author: process:cursor
    last_modified: 2026-08-19
  - id: hybrid-guide
    resource: ../../docs/guides/hybrid-sites.rocdown
    title: Hybrid CDN plus island-service operator guide
    author: process:cursor
    last_modified: 2026-08-20
  - id: ci-workflow
    resource: ../../.github/workflows/ci.yml
    title: Rocci GitHub Actions CI workflow
    author: process:git
    last_modified: 2026-08-19
  - id: ci-local
    resource: ../../scripts/ci-local.sh
    title: Local CI job runner
    author: process:git
    last_modified: 2026-08-19
  - id: install-roc
    resource: ../../docker/install-roc.sh
    title: Pinned Roc nightly installer for Linux
    author: process:git
    last_modified: 2026-08-19
  - id: root-readme
    resource: ../../README.md
    title: Rocci workspace overview and site build command
    author: process:git
    last_modified: 2026-08-20
  - id: rocdown-readme
    resource: ../../crates/rocci-rocdown/README.md
    title: Implemented Rocdown package command
    author: process:git
    last_modified: 2026-08-20
  - id: efficient-plan
    resource: efficient-publishing.md
    title: Efficient publishing implementation plan
    author: process:cursor
    last_modified: 2026-08-20
  - id: rocci-dev-site
    resource: rocci-dev-site.md
    title: rocci.dev site architecture
    author: process:codex
    last_modified: 2026-08-18
  - id: tangled-plan
    resource: tangled-hosting.md
    title: Tangled hosting and devops with a GitHub macOS mirror
    author: process:cursor
    last_modified: 2026-08-19
  - id: publication
    resource: ../decisions/local-knowledge-publication.md
    title: Keep generated knowledge publication local
    author: process:okf-phase-5
    last_modified: 2026-08-16
  - id: preview-plan
    resource: public-preview-community.md
    title: Rocci public-preview branding and community plan
    author: process:cursor
    last_modified: 2026-08-19
  - id: cli-plan
    resource: cli-entry-points.md
    title: CLI entry points plan
    author: process:cursor
    last_modified: 2026-08-19
  - id: hosting-follow-ons
    resource: hybrid-island-hosting-follow-ons.md
    title: Hybrid island hosting follow-ons
    author: process:cursor
    last_modified: 2026-08-20
  - id: cf-universal-ssl
    resource: https://developers.cloudflare.com/ssl/edge-certificates/universal-ssl/
    title: Cloudflare Universal SSL
    author: organization:cloudflare
    last_modified: 2026-08-14
  - id: cf-tunnel
    resource: https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/
    title: Cloudflare Tunnel
    author: organization:cloudflare
  - id: cf-email-routing
    resource: https://developers.cloudflare.com/email-routing/get-started/enable-email-routing/
    title: Enable Cloudflare Email Routing
    author: organization:cloudflare
  - id: hexonet-verify
    resource: https://wiki.hexonet.net/wiki/Registrant_Verification
    title: HEXONET/EPAG registrant email verification and NS replacement
    author: organization:hexonet
  - id: get-dev
    resource: https://get.dev/
    title: Google .dev registry HTTPS requirement
    author: organization:google
  - id: hetzner-cloud
    resource: https://www.hetzner.com/cloud
    title: Hetzner Cloud product and EU locations
    author: organization:hetzner
  - id: hetzner-servers
    resource: https://docs.hetzner.com/cloud/servers/overview
    title: Hetzner Cloud server types, IPs, and OS images
    author: organization:hetzner
  - id: ovh-vps
    resource: https://www.ovhcloud.com/en/vps/cheap-vps/
    title: OVHcloud starter VPS (VPS-1)
    author: organization:ovhcloud
  - id: gh-environments
    resource: https://docs.github.com/en/actions/how-tos/writing-workflows/choosing-what-your-workflow-does/using-environments-for-deployment
    title: GitHub Actions environments for deployment secrets
    author: organization:github
---

# Deploy rocci.dev with Cloudflare, a small VPS, and CI

## Purpose and authority

This plan turns the [publishing research](../research/rocci-dev-publish.md)
into bounded delivery. It is exploratory until a human reviewer accepts the
scope. It does not describe a shipped production site.[^research]

Do not start a phase until the user asks.

Site information architecture stays in [rocci.dev site](rocci-dev-site.md).
Artifact packaging stays in [efficient publishing](efficient-publishing.md).
Git forge migration stays in [Tangled hosting](tangled-hosting.md). This plan
owns **DNS, TLS, the origin VPS, Cloudflare as CDN, and CI that publishes
`site/`**.

## Goal

Serve `https://rocci.dev` from Cloudflare in front of a Hetzner Cost-Optimized
x86 VPS (Falkenstein or Nuremberg) that runs the existing hybrid Compose stack
(Caddy plus a precompiled island binary). GitHub Actions packages `site/` with
Roc and deploys only from `staging` or `production`. `main` lands pull
requests and does not publish. Promote `main` → `staging` (Access-gated
`staging.rocci.dev`) then `staging` → `production`.

## Out of bound

- Product `rocci deploy` / Pages / Netlify adapters.[^cli-plan][^rocci-dev-site][^efficient-plan]
- Moving the hostname onto Tangled Sites.[^tangled-plan]
- Publishing the OKF knowledge bundle.[^publication]
- Cross-origin `islands.rocci.dev` (CORS unshipped).[^hosting-follow-ons][^hybrid-guide]
- Kubernetes, object-storage HTML, or a second CDN.
- Flexible SSL, grey-cloud origin IPs, or Caddy Let's Encrypt as the visitor
  certificate.
- Launch copy, visual identity, and the public-preview Phase 0 license/conduct
  gate — those can block going *public*, not standing up HTTPS.[^preview-plan]
- Changing Rocdown layout or catalog ownership.

## Constraints that do not move

| Keep | Meaning |
| --- | --- |
| Build ≠ serve | The VPS image set does not contain `rocci`, `rocdown`, `roc`, rustc, or WebKit.[^efficient-plan][^docker-readme] |
| Two artifacts stay two | CDN tree and island process stay separate. Caddy proxies `/actions/` and `/health`.[^hybrid-guide][^compose-hybrid] |
| Same-origin islands | Empty `service_origin`; browser POSTs relative `/actions/`. |
| `--cdn-only` is the wrong gate for `site/` | Home is live. Package the hybrid tree; do not publish dead buttons.[^site-home][^hybrid-guide] |
| Cloudflare is the CDN | DNS, Universal SSL, cache, Tunnel, Email Routing. Not Pages as the file host.[^research][^cf-universal-ssl] |
| `.dev` is HTTPS-only | First public hostname already has a trusted edge cert.[^get-dev][^tangled-plan] |
| amd64 origin | CI on `ubuntu-latest` emits `--target x64musl`. Default box is Hetzner Cost-Optimized x86 in Falkenstein or Nuremberg, not ARM CAX.[^docker-readme][^hetzner-cloud] |
| Three CLIs | Packaging emits files; operators (CI) upload them.[^cli-plan] |
| Secrets stay off git | Deploy keys live in a GitHub Environment; fork PRs never deploy.[^ci-workflow] |
| OKF stays local | [^publication] |

## Current evidence

`site/` is the public tree (`base_url` `https://rocci.dev`, output
`dist/rocci.dev`, docs mounted). Home is a live counter island. Local hybrid
Compose already serves that shape. CI only `check docs` and never installs
Roc.[^site-config][^site-home][^compose-hybrid][^ci-workflow][^root-readme]

On 2026-08-20, the owner reported completing registrar verification, the
Cloudflare Free zone and nameserver cutover, Always Use HTTPS (without
Flexible SSL), Email Routing with a verified personal destination and tested
`oss@rocci.dev` forwarding, the specified Hetzner VPS and firewall, a named
Tunnel with `cloudflared` installed on the VPS, bootstrap SSH, GitHub
Environment `production`, a locked-down deploy user, and the origin docker
kit at `/srv/rocci/docker`. The Tunnel has no production hostname route.
`staging.rocci.dev` is routed to the loopback Caddy origin and guarded by
Cloudflare Access: the maintainer can use a browser login and CI can use its
existing service token. Artifacts are pushed by CI, not copied by hand.[^human-preparation]

## Human preparation (before an agent continues)

Do not put tokens, SSH private keys, or Cloudflare Tunnel credentials in git
or in chat. Create them in the vendor UI, then tell the agent only the
**names** and non-secret facts below.[^ci-workflow][^gh-environments]

### Already enough for Phase 1

An agent can implement **CI package-only** (`check site`, `site.yml` that
uploads `site.tgz` + `islands`, README fix) from this repository today. That
job needs no VPS, no Cloudflare, and no GitHub Environment.[^ci-workflow][^rocdown-readme]

### Human preparation completed before Phase 2 (reported 2026-08-20)

These actions were completed by the owner in vendor UIs and on the VPS. Their
credentials and infrastructure identifiers are deliberately not recorded
here.[^human-preparation]

1. **Registrar.** ICANN registrant-email verification is complete; the
   verification placeholder is no longer the intended delegation.[^hexonet-verify][^tangled-plan]
2. **Cloudflare account.** The Free zone is added, registrar nameservers are
   pointed at Cloudflare, Always Use HTTPS is enabled, and Flexible SSL is not
   used.[^cf-universal-ssl][^get-dev]
3. **Mail.** Email Routing has a verified personal destination; `oss@rocci.dev`
   forwards there and has been tested. `security@rocci.dev` remains a later
   address.[^cf-email-routing][^tangled-plan]
4. **Hetzner Cloud.** The specified Cost-Optimized **x86** Debian 12 VPS,
   IPv6, Primary IPv4, and SSH-only provider firewall are in place.[^hetzner-cloud][^hetzner-servers]
5. **Tunnel and private staging.** A named Tunnel exists and `cloudflared` is
   installed on the VPS. `staging.rocci.dev` routes to loopback Caddy behind a
   Cloudflare Access application. The maintainer's email is allowed for
   browser testing, and the GitHub deployment service token has a Service
   Auth policy for CI tests. `rocci.dev` and `www.rocci.dev` remain unrouted
   until a public launch decision.[^cf-tunnel][^human-preparation]
6. **Bootstrap SSH.** SSH as a sudo-capable user is confirmed. A locked-down
   `deploy` user is in place for CI artifact push.

### Phase 3 access preparation completed (reported 2026-08-20)

7. **GitHub Environment `production`.** It is configured for this repository;
   its secret values remain only in GitHub, never in the repository or this
   record.[^gh-environments][^ci-workflow]

   | Secret | Value |
   | --- | --- |
   | `DEPLOY_HOST` | Access SSH hostname (`ssh.rocci.dev`) |
   | `DEPLOY_USER` | SSH user that may write the release directory |
   | `DEPLOY_SSH_KEY` | Private key for that user (ed25519) |
   | `CF_ACCESS_CLIENT_ID` | Cloudflare Access service token id |
   | `CF_ACCESS_CLIENT_SECRET` | Cloudflare Access service token secret |

   The `deploy` user's matching public key is on the VPS. Optional later: a
   Cloudflare API token used only to purge cache, as `CLOUDFLARE_PURGE_TOKEN`.

8. **Smoke identity.** After CI deploy, origin health is
   `http://127.0.0.1:8080/health` on the VPS. `https://rocci.dev/` is the
   exit for the Tunnel-routing phase, not for the first artifact push.

### Hand-off to the agent (paste facts, not secrets)

When the items above exist, send a message like this (fill the brackets):

```text
Human prep for rocci-dev-publish:
- Phase 1: start whenever
- Cloudflare zone rocci.dev: active
- NS at Cloudflare: yes
- oss@rocci.dev: receives mail
- VPS: Hetzner fsn1 (or nbg1), Debian 12, amd64, IPv4 [x.x.x.x]
- SSH: locked-down deploy user and public key installed (private key is a GitHub secret, not chat)
- Tunnel: name [name], cloudflared running, hostname not yet on production
- GitHub Environments staging and production: configured
- Decision gates 2–4: keep live home island; deploy from GitHub staging/production; private staging is Access-gated
```

### What the agent still must not invent

- Registrar or Cloudflare logins
- Hetzner/OVH billing
- Secret values
- A public knowledge deploy
- Product `rocci deploy` adapters

## Delivery phases

Each phase is one mergeable change or one operator session. Start only when
asked.

### Phase 0 — Cloudflare zone, mail, and HTTPS-ready DNS

**Bound:** registrar and Cloudflare dashboard. No application code. Shares
the mailbox steps with Tangled Phase 0; this phase also prepares the website
hostname.[^tangled-plan][^cf-email-routing]

**Does:**

- Complete registrant-email verification so NS are real.[^hexonet-verify]
- Add `rocci.dev` to Cloudflare (Free plan) and switch registrar NS.
- Always Use HTTPS. SSL/TLS mode unused for Tunnel visitors; do not set
  Flexible.
- Email Routing: `oss@rocci.dev`, later `security@rocci.dev`.
- Create a Tunnel; do **not** yet route production traffic if origin Caddy
  is not up. A parked "coming soon" origin is allowed.
- Reserve `www.rocci.dev` → `https://rocci.dev`.
- Leave `_atproto.rocci.dev` for Tangled; do not collide with MX.[^tangled-plan]

**Does not:** provision the VPS; write GitHub workflows; publish `site/`.

**Exit:** `dig NS rocci.dev` shows Cloudflare; a test message reaches
`oss@rocci.dev`; Universal SSL is issued for the apex (or Cloudflare shows
the zone active and waiting on the Tunnel CNAME). HTTP to the apex cannot
succeed in a normal browser.[^cf-universal-ssl][^get-dev]

**Status:** human preparation complete; production hostname route waits on
origin Caddy from CI deploy.

### Phase 1 — CI packages `site/` without deploying

**Bound:** GitHub Actions, `scripts/ci-local.sh`, README. No VPS, no secrets
beyond the existing checkout.

**Does:**

- Add `rocdown check site` next to `check docs` in `fixtures-and-docs` and
  `ci-local.sh`.[^ci-workflow][^ci-local]
- Add `.github/workflows/site.yml`:
  - `on: pull_request` and `push` (or path filters on `site/`, `docs/`,
    theme crates, `docker/`).
  - `ubuntu-latest`; rust cache; Linux GTK/WebKit deps (today's CLI still
    links desktop).
  - Install the pinned Roc nightly (reuse `docker/install-roc.sh` or a
    `$HOME`-writable variant).[^install-roc]
  - `cargo run -q -p rocci-rocdown-cli -- package site --target x64musl`
    (hybrid `package` writes `publish.json` / `site.tgz` and the musl
    `islands` binary).[^rocdown-readme]
  - Upload `site.tgz` (or `dist/rocci.dev`) and the `islands` binary as
    artifacts. Do not SSH.
- Correct the root README: public tree is `site/` → `dist/rocci.dev`;
  `docs/` remains the mounted catalog and the standalone docs
  check.[^root-readme][^site-config][^docs-config]

**Does not:** deploy; change Compose; reopen OKF.

**Exit:**

```text
cargo run -q -p rocci-rocdown-cli -- check site
cargo run -q -p rocci-rocdown-cli -- check docs
./scripts/ci-local.sh fixtures-and-docs
```

A GitHub `site.yml` run on the same revision uploads linux/amd64 artifacts.
`package` is not required in `ci-local.sh` unless Roc is on `PATH`.

**Status:** implemented in this revision (`check site`, `site.yml` package-only
artifacts, README). Not logged complete until CI and Knowledge succeed on the
revision.

### Phase 2 — Origin VPS and docker kit

**Bound:** one Hetzner Cloud **Cost-Optimized x86** VM (2 vCPU / 4 GB, Debian
12) in **Falkenstein (`fsn1`)** or **Nuremberg (`nbg1`)**, Docker Engine, the
hybrid Compose files, `cloudflared` **process** (no production hostname yet).
OVHcloud VPS-1 in Germany is the fallback if Hetzner is unavailable. Do not
copy `site.tgz` or `islands` by hand.[^docker-readme][^compose-hybrid][^cf-tunnel][^hetzner-cloud][^hetzner-servers][^ovh-vps]

**Does:**

- Create the VM. Add IPv6 plus a Primary IPv4 for bootstrap. No inbound
  22/80/443 on the provider firewall. CI and operators SSH through Cloudflare
  Access (`ssh.rocci.dev` + service token), not grey-cloud port 22.
- Install Docker. Scp `docker/compose.hybrid.yml`, `docker/cdn/`,
  `docker/islands/Dockerfile`, and `docker/prod/` (`bootstrap-scp.sh`).
- Keep `cloudflared` installed; do not attach `rocci.dev` until origin
  health exists.
- Persist SQLite on a named volume. Document a copy-out backup.

**Does not:** unpack site artifacts; GitHub deploy job (Phase 3); production
Tunnel hostname or cache bypass (Phase 4); rate limits (Phase 5).

**Exit:** `/srv/rocci/docker` contains Compose, Caddy, islands Dockerfile, and
`prod/` scripts. `deploy` can write `/srv/rocci` and run `docker compose`.

**Status:** complete. Origin kit is on the VPS; no artifacts yet.

### Phase 3 — CI pushes artifacts from `staging` and `production`

**Bound:** `site.yml` deploy job, GitHub Environments `staging` and
`production`, SSH as a locked-down deploy user. No Kubernetes. No public
hostname requirement. No second origin stack.

**Does:**

- Environment secrets: `DEPLOY_HOST` (`ssh.rocci.dev`), `DEPLOY_USER`, SSH
  private key, Cloudflare Access service token id/secret. CI uses
  `cloudflared access ssh` as `ProxyCommand`; public 22 stays closed.
- `deploy` job: `if: github.event_name == 'push' && (github.ref ==
  'refs/heads/staging' || github.ref == 'refs/heads/production')`. Needs
  the package job. Environment name is the branch. Never runs on
  `pull_request` or on `main`.
- Scp the origin kit plus `site.tgz` and `islands`. Atomic publish:
  unpack into `releases/<sha>/`, `docker compose up -d --build`, curl
  `http://127.0.0.1:8080/health` on the VPS, then point `current` at that
  release. Failed health restores the previous release. Both branches
  publish `/srv/rocci`; deploys serialize.
- Document the Environments, that `main` is the landing branch, and that
  fork PRs cannot deploy. Promote with `git push origin
  origin/main:staging` then `origin/staging:production`.

**Does not:** `workflow_dispatch` to production without the same origin
health; deploy from contributor forks; store the SSH key in the repo;
Cloudflare Tunnel hostname routing; the public `https://rocci.dev/` curls;
a separate staging Compose stack or SQLite volume.

**Exit:** A push to `staging` or `production` that runs `site.yml` publishes
artifacts in that Actions run. A failing `package` job does not touch the
VPS. On the origin:

```text
curl -sf http://127.0.0.1:8080/health
```

README or `docker/README.md` names the workflow and the Environment.

**Status:** implementing (`push-release.sh`, `publish.sh`, `site.yml` deploy
job). Not logged complete until that workflow succeeds on `staging`.

### Phase 4 — Tunnel hostname and public smoke

**Bound:** Cloudflare Tunnel public hostname, Cache Rule. Origin Caddy from
Phase 3 must already answer on loopback.

**Does:**

- Map `rocci.dev` (and `www`) to `http://127.0.0.1:8080`.
- Cloudflare Cache Rule: bypass `/actions/` and `/health`. Origin headers
  already mark `/assets/` immutable and HTML `no-cache`.[^static-caddy][^cdn-caddy]

**Does not:** GitHub deploy changes; rate limits (Phase 5).

**Exit:** From a machine that is not the VPS:

```text
curl -sfI https://rocci.dev/
curl -sf https://rocci.dev/health
curl -sf -X POST https://rocci.dev/actions/counter/increment \
  -H 'datastar-request: true' -H 'content-type: application/json' \
  -d '{"tz":"Europe/Stockholm","handle":"Coral Lynx"}'
```

`https://rocci.dev/` is the branded home (not the docs-only tree). Hashed
`/assets/` responses include long cache headers. `docker run --rm --entrypoint
/bin/sh rocci-islands:local -c 'which roc'` still fails.

**Status:** not started. Origin kit is ready; wait for Phase 3 health.

### Phase 5 — Public-OSS hardening

**Bound:** Cloudflare rules, SQLite backup, operator notes. Code changes
only if the public counter needs a documented rate-limit header or a
robots/security.txt page that Rocdown already supports.

**Does:**

- Rate-limit `/actions/` (Cloudflare Rate limiting or WAF custom rule).
- Weekly (or image-based) SQLite copy off the box.
- Confirm `security@rocci.dev` forwards before the repository lists it.
- After the repo is public: verify a fork PR cannot read `production`
  secrets and cannot trigger deploy.
- Keep `staging.rocci.dev` protected by Cloudflare Access. Its human policy
  admits only the maintainer; its Service Auth policy admits the GitHub CI
  service token for non-interactive tests.[^human-preparation]

**Does not:** OKF public deploy; product hosting adapters; stripping the
home island unless the reviewer chose `--cdn-only` launch.

**Exit:** Written runbook: restore from backup, roll back a symlink, rotate
the deploy key. One fork-PR Actions run shows deploy skipped.

**Status:** not started.

### Phase 6 — Tangled handoff (after forge Phase 2)

**Bound:** who builds Linux `package`. Origin and Cloudflare do not move.

**Does:**

- When spindle Linux jobs are required, run the same `package site --target
  x64musl` command there (or keep packaging on the GitHub mirror of
  canonical `main`).
- Point `site/rocdown.toml` `repository` at the Tangled URL in the same
  change as Tangled Phase 4, not earlier.[^tangled-plan][^site-config]
- Keep GitHub Environment deploy until a spindle secret can SSH with the
  same atomic publish.

**Does not:** Tangled Sites; changing Tunnel or VPS size.

**Exit:** A documented sentence: which forge produces `site.tgz` and which
job SSHs. `rocci.dev` still resolves through Cloudflare.

**Status:** deferred. GitHub remains the active repository, CI, and deploy
path; do not begin this handoff without a new maintainer decision.

## Decision gates

Human approval is required before treating these as normative:

1. Cloudflare + Hetzner Cost-Optimized x86 in Falkenstein/Nuremberg (OVH
   VPS-1 Germany is the fallback). Not Pages, not a second CDN, not ARM.
2. Keep the live home island on first publish (versus a `--cdn-only` launch).
3. Deploy from GitHub `main`; Tangled adoption is deferred and is not a
   deployment or public-launch gate at this point.
4. Keep `staging.rocci.dev` private behind Cloudflare Access before the first
   public publish; allow the maintainer's browser identity and the GitHub CI
   service token only.[^human-preparation]

## Validation

Per phase as listed. Knowledge after record edits:

```text
cargo run -q -p rocci-okf -- check knowledge --profile rocci --format terminal
```

Do not log a phase complete until CI and Knowledge workflows succeed on that
revision. Phase 0 is operator DNS and has no crate test.

## Dependency order

```text
1 (CI package) can start now
2 origin docker kit (human) before 3
3 (CI artifact push) before 4 (Tunnel hostname)
5 after a public origin exists
```

[^research]: Cloudflare as CDN; Tunnel TLS; hybrid origin; GitHub Actions until Tangled.
[^site-config]: Public tree `site/` → `dist/rocci.dev`.
[^docs-config]: Mounted and standalone docs catalog.
[^site-home]: Live counter on `/`.
[^docker-readme]: Hybrid Compose; musl `--target` matches container CPU.
[^compose-hybrid]: Caddy plus islands; healthcheck.
[^static-caddy]: Asset and HTML cache headers.
[^cdn-caddy]: Same-origin `/actions/` proxy.
[^hybrid-guide]: Two-artifact production sketch; CORS unshipped.
[^ci-workflow]: Docs check only; `contents: read`.
[^ci-local]: Mirrors fixtures-and-docs.
[^install-roc]: Pinned Linux Roc nightly.
[^root-readme]: Stale `build docs` / `dist/rocci.dev` pairing.
[^rocdown-readme]: `package` hybrid artifacts.
[^efficient-plan]: No product CDN adapters; build ≠ serve.
[^rocci-dev-site]: No deploy-plugin product.
[^tangled-plan]: Mail at Cloudflare; Sites out of scope; later `repository` URL flip.
[^publication]: Knowledge stays local.
[^preview-plan]: Public-preview Phase 0 is the open-source gate.
[^cli-plan]: No plugin host.
[^hosting-follow-ons]: CORS not shipped.
[^cf-universal-ssl]: Free edge certificates, auto-renew.
[^cf-tunnel]: Outbound-only origin.
[^cf-email-routing]: Inbound forwarding needs Cloudflare DNS.
[^hexonet-verify]: Unverified NS replacement.
[^get-dev]: `.dev` requires HTTPS.
[^hetzner-cloud]: Falkenstein/Nuremberg/Helsinki; cost-optimized shared x86.
[^hetzner-servers]: Debian images; IPv6 free; IPv4 extra; firewall.
[^ovh-vps]: VPS-1 2 vCores / 4 GB / 40 GB NVMe; unlimited EU traffic.
[^gh-environments]: Deployment jobs should use an Environment so secrets are not available to ordinary PR jobs.
[^human-preparation]: Maintainer report in the 2026-08-20 Codex task; no
credentials, IP addresses, account identifiers, or tunnel token were recorded.
