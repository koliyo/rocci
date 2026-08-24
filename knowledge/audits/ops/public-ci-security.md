---
type: Audit
title: Public-repo GitHub Actions security review
description: Current CI is comment-gated and mostly self-hosted; that is too tight for community PRs, too loose for a public self-hosted runner, and not yet the hosted main/staging/production path needed before flipping visibility.
tags: [domain/rocci, concern/ci, concern/security, concern/publication, audience/maintainer]
status: draft
generated: { by: process:cursor, at: 2026-08-22T22:50:00Z }
stale_after: 2026-11-22
authority: descriptive
owners: [human:nils]
sources:
  - id: ci-workflow
    resource: ../../../.github/workflows/ci.yml
    title: Comment-dispatched CI workflow with authorize then self-hosted jobs
    author: process:git
    last_modified: 2026-08-22
  - id: ci-command
    resource: ../../../.github/workflows/ci-command.yml
    title: /ci issue-comment dispatcher
    author: process:git
    last_modified: 2026-08-22
  - id: knowledge-workflow
    resource: ../../../.github/workflows/knowledge.yml
    title: Comment-dispatched Knowledge workflow
    author: process:git
    last_modified: 2026-08-22
  - id: site-workflow
    resource: ../../../.github/workflows/site.yml
    title: Site package and environment deploy workflow
    author: process:git
    last_modified: 2026-08-22
  - id: release-workflow
    resource: ../../../.github/workflows/release.yml
    title: Tag and workflow_dispatch release workflow
    author: process:git
    last_modified: 2026-08-22
  - id: root-readme
    resource: ../../../README.md
    title: Documented /ci and self-hosted restriction
    author: human:nils
    last_modified: 2026-08-22
  - id: prod-readme
    resource: ../../../docker/prod/README.md
    title: Environment secret names and branch restriction guidance
    author: process:git
    last_modified: 2026-08-22
  - id: preview-plan
    resource: ../../plans/site/public-preview-community.md
    title: Public-preview branding and community plan
    author: process:cursor
    last_modified: 2026-08-21
  - id: launch-audit
    resource: ../rocci-dev-public-launch.md
    title: rocci.dev public-launch checklist
    author: process:cursor
    last_modified: 2026-08-22
  - id: harden-docs
    resource: https://docs.github.com/en/actions/security-for-github-actions/security-guides/security-hardening-for-github-actions
    title: GitHub Actions security hardening
    author: organization:github
  - id: self-hosted-docs
    resource: https://docs.github.com/en/actions/hosting-your-own-runners/managing-self-hosted-runners/about-self-hosted-runners
    title: About self-hosted runners
    author: organization:github
  - id: pr-target-docs
    resource: https://docs.github.com/en/actions/reference/security/securely-using-pull_request_target
    title: Secure use of pull_request_target
    author: organization:github
  - id: dependabot-docs
    resource: https://docs.github.com/en/code-security/dependabot/dependabot-version-updates/configuration-options-for-the-dependabot.yml-file
    title: dependabot.yml configuration options
    author: organization:github
  - id: ci-security-plan
    resource: ../../plans/ops/public-ci-security.md
    title: Public-repo CI security implementation plan
    author: process:cursor
    last_modified: 2026-08-22
---

# Public-repo GitHub Actions security review

## Verdict

The current pipeline is built for a **private** repo and a **single trusted
author**. It already avoids the worst public-repo defaults: no
`pull_request` or `pull_request_target` trigger, deploy secrets live only in
the `staging` and `production` GitHub Environments, repository secrets are
empty, and `GITHUB_TOKEN` defaults to read.[^ci-workflow][^ci-command][^site-workflow][^prod-readme]

It is **not** ready to flip `koliyo/rocci` public. Self-hosted jobs still
execute checked-out PR heads on machines that also write deploy keys to
`$HOME/.ssh/deploy`. `/ci` only accepts the `koliyo` account and
`koliyo`-authored same-repo PRs, so community PRs get no hosted CI. Nothing
runs automatically on `main`, `staging`, or `production`. There is no
Dependabot config. Comment matching is lowercase-only and conversation-only,
not review comments.[^ci-command][^root-readme][^harden-docs][^self-hosted-docs]

This audit does not authorize publication. Implementation sequence:
[public-repo CI security plan](/plans/ops/public-ci-security.md).[^ci-security-plan][^preview-plan][^launch-audit]

Inspected 2026-08-22 against the workflow files and the live GitHub
repository settings. The repository was still `private`.

## What is already safe

- **No PR-auto CI.** `ci.yml` and `knowledge.yml` expose only `workflow_call`
  and `workflow_dispatch`. Forks cannot start those workflows by opening a
  pull request.[^ci-workflow][^knowledge-workflow]
- **Trusted workflow text.** `issue_comment` runs the dispatcher from the
  default branch, then calls reusable workflows from that same trusted
  definition. The PR head is checked out by SHA after authorization, not
  used as the workflow source.[^ci-command][^pr-target-docs]
- **Authorize before self-hosted.** Both CI and Knowledge run an
  `ubuntu-latest` job that requires a numeric PR, a 40-character SHA, author
  `koliyo`, and `head.repo.full_name == github.repository`. Failure sets
  `allowed=false` and skips runner jobs.[^ci-workflow][^knowledge-workflow]
- **Least-privilege tokens.** CI and Knowledge request `contents: read` and
  `pull-requests: read`. The dispatcher may write a PR comment. Site stays
  `contents: read`. Repo setting `default_workflow_permissions` is `read`;
  Actions cannot approve pull-request reviews.[^ci-workflow][^ci-command][^site-workflow]
- **Deploy secrets are environment-only.** Live inspection: repository
  Actions secrets `[]`. Environments `staging` and `production` each hold
  `DEPLOY_HOST`, `DEPLOY_USER`, `DEPLOY_SSH_KEY`, `CF_ACCESS_CLIENT_ID`, and
  `CF_ACCESS_CLIENT_SECRET`. The deploy job references
  `environment: ${{ github.ref_name }}` and runs only on
  `refs/heads/staging` or `refs/heads/production`.[^site-workflow][^prod-readme]
- **Production environment branch policy** is a custom allow-list containing
  only the `production` branch. A `workflow_dispatch` on another ref cannot
  see production secrets.

## Findings

### F1 — Persistent self-hosted runners will be a public-repo incident

GitHub's guidance is that self-hosted runners should almost never serve
public repositories: they are not ephemeral VMs, and untrusted workflow code
can persist on the machine and later read secrets or tokens.[^self-hosted-docs][^harden-docs]

Today almost every expensive job uses `[self-hosted, rocci-linux]` or
`[self-hosted, rocci-macos]`, with Linux caches at
`/home/nils/.cache/rocci-target` and `/home/nils/.cache/uv`. The site deploy
job writes `DEPLOY_SSH_KEY` to `$HOME/.ssh/deploy` on that same
`rocci-linux` label and does not delete it.[^ci-workflow][^site-workflow]

If any untrusted head ever reaches those labels after the repo is public,
the attacker inherits the cache directory, any leftover deploy key, and the
runner network. `/ci` today is the only path onto those labels; after a
public flip the command surface must split so community validation never
touches them.

### F2 — `/ci` is both too narrow and the wrong runner class

The dispatcher requires the **commenter** and the **PR author** to be
`koliyo`, the head repo to be this repository, and the body to be exactly
`/ci` or to start with `/ci `. It does not accept `/CI`, review bodies, or
inline review comments. It then calls the self-hosted workflows.[^ci-command][^root-readme]

For a public repo that is the opposite of the intended contract: reviewers
need an explicit hosted `/ci` on **any** PR, including forks, and only
`koliyo` may request local runners (`/ci-local`, with `/cl-local` as an
alias).

Today's `startsWith(..., '/ci ')` (space required) does **not** match
`/ci-local`. A later dispatcher that used `startsWith(body, '/ci')`
without a token boundary would. Exact first-token compare is the
required parse.

### F3 — Protected branches never get CI

`ci.yml` and `knowledge.yml` require `pr_number` and `pr_sha` on every
dispatch. Pushes to `main`, `staging`, and `production` do not run
validation. After merge, the only way to learn the branch is still green is
another manual dispatch that still assumes a PR.[^ci-workflow][^knowledge-workflow]

### F4 — Comment-triggered workflows run in the privileged repo context

`issue_comment` is not a `pull_request` event. The job gets the base
repository token and can write Actions caches. Checking out an untrusted
SHA and then using `Swatinem/rust-cache@v2` can poison caches later reused
by `main`. Checkout also keeps the default credentials unless
`persist-credentials: false`.[^ci-workflow][^harden-docs]

First-time-contributor approval settings do **not** apply to comment
triggers. Authorization must stay in the dispatcher (`author_association`
plus command parsing), not in GitHub's fork-PR approval toggle.

### F5 — Site package can still run untrusted refs on the deploy runner

`site.yml` triggers on `workflow_dispatch` with no ref restriction. The
package job always runs on `rocci-linux`. Only deploy is gated to
`staging`/`production`. The documented "package only" dispatch on other
refs is a way to execute that ref's tree on the same host that stores
deploy keys.[^site-workflow][^prod-readme]

`staging`'s environment policy is `protected_branches: true` rather than a
custom allow-list of the `staging` branch. While the repo is private on a
free plan, branch protection and rulesets are unavailable; that policy may
not bind as intended. Production's custom `production` allow-list is the
stronger pattern.

### F6 — Release and Actions supply chain are unsigned

`release.yml` uses `workflow_dispatch` on any ref, `contents: write`, and
the Linux self-hosted label. Tag protection / rulesets are unavailable
until the repo is public or the account is Pro.[^release-workflow]

Third-party actions are floating major tags (`actions/checkout@v7`,
`dtolnay/rust-toolchain@stable`, `astral-sh/setup-uv@v6`,
`Swatinem/rust-cache@v2`). Repo setting `allowed_actions` is `all` and
`sha_pinning_required` is false.

### F7 — No dependency automation

There is no `.github/dependabot.yml` (or Renovate). Cargo, uv, npm
(`editors/vscode`), Dockerfiles under `docker/`, the excluded Zed crate,
and Actions versions are unmonitored.[^dependabot-docs]

## Non-findings

- Deploy jobs do not use `pull_request`. Fork PRs cannot read environment
  secrets through the current YAML.[^site-workflow]
- Reusable CI calls do not pass `secrets: inherit`.
- Comment bodies are not interpolated into the authorize shell; the `/ci`
  test lives in the job `if:`.[^ci-command]

## Residual operator checks (not in git)

Confirm before flipping visibility: staging environment custom-branch
allow-list is `staging` only; self-hosted runners are repository-scoped
(not org-wide); Actions "fork pull request workflows" require approval
even though this plan does not add `pull_request`; Dependabot security
updates are enabled in the repo UI.

[^ci-workflow]: `.github/workflows/ci.yml` authorize job and self-hosted lint/test/fixtures/editors.
[^ci-command]: `.github/workflows/ci-command.yml` `koliyo`-only lowercase `/ci` dispatcher.
[^knowledge-workflow]: `.github/workflows/knowledge.yml` authorize-then-self-hosted validate job.
[^site-workflow]: `.github/workflows/site.yml` package on self-hosted; deploy uses `environment` and staging/production refs only.
[^release-workflow]: `.github/workflows/release.yml` tag plus `workflow_dispatch`, `contents: write`, Linux self-hosted.
[^root-readme]: README documents `/ci` from `koliyo` only and the same-repo self-hosted restriction.
[^prod-readme]: `docker/prod/README.md` names Environment secrets and says to restrict each Environment to its matching branch.
[^preview-plan]: Public-preview plan treats repository publication as a separate launch gate.
[^launch-audit]: Public-launch checklist records that the repository stays private until the maintainer flips it.
[^harden-docs]: GitHub Actions hardening: self-hosted runners are not ephemeral; restrict secret access with Environments.
[^self-hosted-docs]: GitHub documents that self-hosted runners should almost never be used for public repositories.
[^pr-target-docs]: `pull_request_target` and other privileged events run in the base-repository context.
[^dependabot-docs]: `dependabot.yml` ecosystems include cargo, uv, npm, github-actions, and docker.
[^ci-security-plan]: Implementation sequence for hosted `/ci`, `/ci-local`, protected-branch CI, isolation, and Dependabot.
