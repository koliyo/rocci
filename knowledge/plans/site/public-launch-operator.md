---
type: Implementation Plan
title: rocci.dev public-launch operator sequence
description: Maintainer sequence after the 2026-08-23 Should pass — promote current main to staging and smoke it, flip koliyo/rocci public, then promote production and route DNS. Hosted CI already lists production; the branch is created by the first promote-production push.
tags: [domain/rocci, domain/rocdown, concern/publication, concern/community, concern/ci]
status: draft
generated: { by: process:cursor, at: 2026-08-23T00:30:00Z }
stale_after: 2026-11-22
authority: exploratory
owners: [human:nils]
sources:
  - id: launch-audit
    resource: ../../audits/site/rocci-dev-public-launch.md
    title: rocci.dev public-launch checklist
    author: process:cursor
    last_modified: 2026-08-23
  - id: ci-security-audit
    resource: ../../audits/ops/public-ci-security.md
    title: Public-repo GitHub Actions security review
    author: process:cursor
    last_modified: 2026-08-22
  - id: ci-security-plan
    resource: ../public-ci-security.md
    title: Public-repo CI security and Dependabot
    author: process:cursor
    last_modified: 2026-08-22
  - id: ci-workflow
    resource: ../../../.github/workflows/ci.yml
    title: Hosted CI on push to main, staging, and production
    author: process:cursor
    last_modified: 2026-08-23
  - id: knowledge-workflow
    resource: ../../../.github/workflows/knowledge.yml
    title: Knowledge validation on push to main, staging, and production
    author: process:cursor
    last_modified: 2026-08-23
  - id: site-workflow
    resource: ../../../.github/workflows/site.yml
    title: Site package and deploy on staging and production
    author: process:git
    last_modified: 2026-08-22
  - id: prod-readme
    resource: ../../../docker/prod/README.md
    title: Origin promote and Environment policy
    author: process:cursor
    last_modified: 2026-08-23
  - id: root-readme
    resource: ../../../README.md
    title: Documented promote-staging and promote-production
    author: process:cursor
    last_modified: 2026-08-23
  - id: publish-plan
    resource: ../rocci-dev-publish.md
    title: rocci.dev Cloudflare and VPS deploy plan
    author: process:cursor
    last_modified: 2026-08-21
---

# rocci.dev public-launch operator sequence

## Goal

Give the maintainer a single ordered list for the remaining public-launch
gates: smoke current copy on staging, flip `koliyo/rocci` public, then
promote `production` and route DNS. Hosted CI and Knowledge already trigger
on `production`; the missing piece was an operator command that creates that
branch from smoked `staging`.[^launch-audit][^ci-workflow][^knowledge-workflow][^root-readme]

Writing this plan does not flip visibility or route apex DNS.

## Out of bound

- Visual identity, logo comparison, and trademark clearance.
- Enabling GitHub Discussions.
- A current install tag as a Must (it stays a Should).
- Publishing the OKF knowledge bundle.
- Adding `pull_request` or `pull_request_target` to any workflow.
- Creating `origin/production` before the signed-out staging smoke.

## Constraints that do not move

- Land on `main`. Promote `main` → `staging` (Access-gated), then
  `staging` → `production` (public hostname once the Tunnel route
  exists). Never deploy from `main` or a pull request.[^prod-readme][^site-workflow]
- First push to `production` creates the branch, runs hosted CI and
  Knowledge, and runs site package/deploy. Both lanes currently publish
  the same `/srv/rocci` origin.[^prod-readme][^publish-plan]
- Deploy secrets stay Environment-only. Repository Actions secrets stay
  empty. CI, Knowledge, Site, and Release are GitHub-hosted only;
  `/ci-local` queues the same hosted jobs.[^ci-security-plan]
- Signed-out `https://staging.rocci.dev/` 302s to Cloudflare Access until
  the maintainer signs in or adds a temporary bypass.[^launch-audit][^publish-plan]
- Apex and `www` 502 until the Tunnel public hostname is attached. Do not
  treat that as a live site.[^launch-audit][^publish-plan]

## Current evidence (2026-08-23)

- `main` is `98c4eac`. `origin/staging` is `4c8a725` (four commits
  behind). `origin/production` does not exist. GitHub Environments
  `staging` and `production` already use custom branch allow-lists for
  those exact names. Repo secrets are `[]`.[^ci-security-audit][^prod-readme]
- `ci.yml` and `knowledge.yml` push to `main`, `staging`, and
  `production` on GitHub-hosted runners. `site.yml` packages and deploys
  only on `staging` and `production`. No `pull_request` trigger.[^ci-workflow][^knowledge-workflow][^site-workflow]
- Dependabot version-update YAML is on `main`. Vulnerability alerts are
  still disabled in the GitHub UI.[^ci-security-plan]

## Phase 1 — Production CI contract and promote command

**Bound:** `.github/workflows/ci.yml`, `.github/workflows/knowledge.yml`,
`.github/workflows/site.yml` (read-only unless a branch list is wrong),
`tools/rocci-ops` promote command, root `README.md`, and
`docker/prod/README.md`. Do not push `production`.

**Exit:** Push branch lists are `main`/`staging`/`production` for CI and
Knowledge, `staging`/`production` for Site. `uv run rocci-ops
promote-production` fetches and pushes
`origin/staging:refs/heads/production`. A rocci-ops test fails if those
branch lists regress.

Phase 1 is in this revision.

## Phase 2 — GitHub UI before the flip

**Bound:** GitHub repository settings only. No git push.

1. Enable Dependabot alerts and Dependabot security updates.
2. Require approval for workflows from all outside collaborators.
3. Leave Discussions off. Leave default `GITHUB_TOKEN` read-only. Do not
   grant Actions the right to approve reviews.
4. Optional: clear the repository homepage until apex is routed, or accept
   that `https://rocci.dev` 502s the moment the repo is public.

**Exit:** Vulnerability alerts are enabled. Fork-workflow approval is on.
Environment allow-lists remain `staging` only and `production` only.

## Phase 3 — Promote staging and smoke signed-out

**Bound:** `uv run rocci-ops promote-staging`, then a maintainer browser
session (or a temporary Access bypass) on `https://staging.rocci.dev/`.

Walk `/`, `/docs/`, `/docs/install/`, `/docs/five-minutes/`,
`/docs/the-stack/`, `/docs/applications/standalone/`, `/docs/rocdown/`,
`/examples/`, `/playground/`, `/faq/`, and `/project/status/`. Confirm
Home `GET /sse` still increments. Confirm `/docs/start/install/`,
`/rocdown/`, and `/docs/tutorials/ship/` 404; `/news/` and
`/news/feed.xml` 410; former News article paths 404, not 308.[^launch-audit]

**Exit:** Staging serves `main`'s tip (not `4c8a725`). The walk passed.
Site workflow on `staging` is green.

## Phase 4 — Flip the repository public

**Bound:** GitHub visibility. Do not route DNS in this phase.

1. Set `koliyo/rocci` to public.
2. Open `https://github.com/koliyo/rocci` and
   `https://github.com/koliyo/rocci/issues` signed out.
3. Confirm issue templates appear and blank issues stay disabled.

**Exit:** The clone URL in Install works without authentication.

## Phase 5 — Ruleset after the flip

**Bound:** GitHub rulesets. Public repos can use them without Pro.

Add a ruleset for `main`, `staging`, and `production`. Restrict direct
pushes as the maintainer prefers. Do **not** require the CI check on pull
requests (that fights opt-in `/ci`). After `production` exists, the
existing Environment allow-list already binds that name.[^ci-security-plan]

**Exit:** `main` is no longer an unprotected default branch.

## Phase 6 — Promote production and route DNS

**Bound:** `uv run rocci-ops promote-production`, then Cloudflare Tunnel
public hostnames for `rocci.dev` and `www.rocci.dev`.

Run this only after Phase 3. The first push creates `origin/production`
at the smoked staging SHA, runs hosted CI and Knowledge, and deploys
through the `production` Environment. Then attach the Tunnel route. Do
not announce Roc or Datastar while apex still 502s.[^prod-readme][^publish-plan]

**Exit:** Launch-day `curl -I` from the [public-launch
checklist](/audits/site/rocci-dev-public-launch.md): 200 on the listed
pages, 410 on `/news/` and `/news/feed.xml`, 404 on retired routes. Home
still increments. GitHub still opens signed-out.

A current install tag remains a Should after this sequence, not a Must.

[^launch-audit]: Remaining Musts are staging smoke, the visibility flip, and production DNS.
[^ci-security-audit]: Residual operator checks: Dependabot alerts, fork-workflow approval, post-public ruleset.
[^ci-security-plan]: Hosted `/ci`, `koliyo`-only `/ci-local`, automatic hosted CI on protected branches, Environment-only secrets.
[^ci-workflow]: `on.push.branches` is `main`, `staging`, `production`.
[^knowledge-workflow]: Same push branches as CI; hosted validate on `ubuntu-latest`.
[^site-workflow]: Package and deploy only when `github.ref` is `staging` or `production`.
[^prod-readme]: Promote commands and custom-branch Environment policy.
[^root-readme]: `promote-staging` then `promote-production` after a staging smoke.
[^publish-plan]: Staging Access-gated; apex unrouted until a launch decision. Both lanes share one origin today.
