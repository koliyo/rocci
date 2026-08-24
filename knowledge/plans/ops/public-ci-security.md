---
type: Implementation Plan
title: Public-repo CI security and Dependabot
description: Before flipping koliyo/rocci public, split hosted /ci from koliyo-only /ci-local, auto-run hosted CI on main/staging/production, keep environment secrets off every PR lane, and add Dependabot.
tags: [domain/rocci, concern/ci, concern/security, concern/publication, audience/maintainer]
status: draft
generated: { by: process:cursor, at: 2026-08-22T22:50:00Z }
stale_after: 2026-11-22
authority: exploratory
owners: [human:nils]
sources:
  - id: audit
    resource: ../../audits/ops/public-ci-security.md
    title: Public-repo GitHub Actions security review
    author: process:cursor
    last_modified: 2026-08-22
  - id: ci-workflow
    resource: ../../../.github/workflows/ci.yml
    title: Current PR-only self-hosted CI workflow
    author: process:git
    last_modified: 2026-08-22
  - id: ci-command
    resource: ../../../.github/workflows/ci-command.yml
    title: Current /ci comment dispatcher
    author: process:git
    last_modified: 2026-08-22
  - id: knowledge-workflow
    resource: ../../../.github/workflows/knowledge.yml
    title: Current PR-only self-hosted Knowledge workflow
    author: process:git
    last_modified: 2026-08-22
  - id: site-workflow
    resource: ../../../.github/workflows/site.yml
    title: Site package and environment deploy workflow
    author: process:git
    last_modified: 2026-08-22
  - id: release-workflow
    resource: ../../../.github/workflows/release.yml
    title: Release workflow
    author: process:git
    last_modified: 2026-08-22
  - id: ops-ci
    resource: ../../../tools/rocci-ops/src/rocci_ops/ci.py
    title: Shared CI job bodies
    author: process:cursor
    last_modified: 2026-08-21
  - id: root-readme
    resource: ../../../README.md
    title: Documented /ci and self-hosted restriction
    author: human:nils
    last_modified: 2026-08-22
  - id: devops-skill
    resource: ../../../.agents/skills/rocci-devops/SKILL.md
    title: Rocci DevOps skill CI trigger notes
    author: process:cursor
    last_modified: 2026-08-22
  - id: prod-readme
    resource: ../../../docker/prod/README.md
    title: Environment secret and branch restriction guidance
    author: process:git
    last_modified: 2026-08-22
  - id: preview-plan
    resource: ../public-preview-community.md
    title: Public-preview branding and community plan
    author: process:cursor
    last_modified: 2026-08-21
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
  - id: env-docs
    resource: https://docs.github.com/en/actions/deployment/targeting-different-environments/using-environments-for-deployment
    title: Using environments for deployment
    author: organization:github
  - id: dependabot-docs
    resource: https://docs.github.com/en/code-security/dependabot/dependabot-version-updates/configuration-options-for-the-dependabot.yml-file
    title: dependabot.yml configuration options
    author: organization:github
---

# Public-repo CI security and Dependabot

## Goal

Make GitHub Actions safe to expose when `koliyo/rocci` becomes public:
reviewers opt in to **hosted** CI with `/ci` or `/CI`; only `koliyo` may
request **local** runners with `/ci-local` (optional alias `/cl-local`);
`main`, `staging`, and `production` run hosted CI on push; environment
secrets stay unreachable from every PR path; Dependabot opens
version-update PRs.[^audit][^preview-plan]

Writing this plan is not executing it and does not flip repository
visibility.

## Current contract (must change)

`ci.yml` and `knowledge.yml` have no `push` or `pull_request` trigger. A
default-branch `issue_comment` workflow runs only when the commenter is
`koliyo` and the body is `/ci` or starts with `/ci `, then authorizes a
same-repo `koliyo` PR and executes jobs on `rocci-linux` / `rocci-macos`.
Site deploy already uses Environments `staging` and `production`; those
secrets are not repository secrets.[^ci-workflow][^ci-command][^knowledge-workflow][^site-workflow][^audit]

Findings F1–F7 in the [security review](/audits/ops/public-ci-security.md)
are the gap this plan closes.[^audit]

## Out of bound

- Flipping the repository from private to public.
- Adding `pull_request` or `pull_request_target` to CI, Knowledge, Site, or
  Release.
- Running `/ci-local` (or any self-hosted job) on a fork, Dependabot, or
  non-`koliyo` PR, even if `koliyo` comments.
- Auto-merge of Dependabot PRs.
- Replacing Dependabot with Renovate in this sequence.
- Moving deploy off Cloudflare Access / the existing Environments.
- Making CI a required status check on every PR (that would fight opt-in
  `/ci`).
- Tangled / spindle CI.
- Adding `cargo-audit` as a merge gate.

## Constraints that do not move

- Job bodies stay in `rocci-ops ci`. Workflow YAML chooses triggers,
  runners, and authorization only.[^ops-ci]
- `/ci` and `/CI` always mean **GitHub-hosted** runners. They never
  schedule `self-hosted`.
- `/ci-local` (and the `/cl-local` alias) is accepted only when the
  **commenter** login is `koliyo` and the PR is an exact same-repository
  PR whose **author** is `koliyo`. Snapshot the head SHA on a hosted job
  before any local job starts. Match the **first whitespace-separated
  token**, case-insensitive, never `startsWith('/ci')`.
- Deploy secrets remain Environment-only (`staging`, `production`). No
  CI/Knowledge job may use `environment:`, `secrets: inherit`, or those
  secret names.[^prod-readme][^env-docs]
- Trusted workflow text always comes from the default branch
  (`issue_comment` / review events and `workflow_call`). Untrusted heads
  are checked out by immutable SHA only.[^pr-target-docs]
- `GITHUB_TOKEN` for CI and Knowledge stays read-only for contents.
- Self-hosted machines are treated as persistently compromised if they
  ever run untrusted code; the YAML must make that impossible, not merely
  unlikely.[^self-hosted-docs][^harden-docs]

## Target trigger matrix

| Event | Runner | Secrets | Who |
| --- | --- | --- | --- |
| Push to `main`, `staging`, `production` | Hosted `ubuntu-latest` / `macos-latest` | None | Automatic |
| `/ci` or `/CI` on a PR (conversation, review body, or inline review comment) | Hosted | None | Commenter `author_association` is `OWNER`, `MEMBER`, or `COLLABORATOR` |
| `/ci-local` on a PR (`/cl-local` alias) | Current self-hosted labels | None | Commenter `koliyo` **and** PR author `koliyo` **and** same-repo head |
| Push to `staging` / `production` matching `site.yml` paths | Package may stay self-hosted; deploy uses the named Environment | Environment secrets on deploy only | Branch policy |
| Tag `v*` / trusted release dispatch | Existing matrix | `GITHUB_TOKEN` write for the publish job only | Tag / owner |

`/ci-local` is acceptable. The earlier `/cl-local`-only spelling was a
workaround for naive prefix matching, not a security requirement. The
dispatcher must take the first whitespace-separated token, lowercase it,
and compare **exactly**:

- `/ci` → hosted
- `/ci-local` or `/cl-local` → local
- `/circle`, `/ci-local-please` as a single token, or a body whose first
  token is anything else → no-op

`/ci please` is hosted because the first token is `/ci`. `/ci-local`
must not be treated as `/ci`. Do not use
`startsWith(body, '/ci')`. Today's job `if` uses `== '/ci'` or
`startsWith(..., '/ci ')` (note the space); that space form does **not**
match `/ci-local`, but a space-less prefix would.

## Phase 1 — Comment dispatcher: `/ci` hosted, `/ci-local` local

**Bound:** `.github/workflows/ci-command.yml` only. Do not change runner
labels inside `ci.yml` / `knowledge.yml` yet; the dispatcher must pass a
`lane` (or equivalent) input that those files will honor in Phase 2, or
call new `workflow_call` entry points added in the same change set as
Phase 2 if that is smaller. Prefer one PR that lands Phase 1 and Phase 2
together if splitting would leave a window where `/ci` still hits
self-hosted.

Parse the first whitespace-separated token of the comment or review body,
trimmed, case-insensitive:

- `/ci` → `lane=hosted`
- `/ci-local` or `/cl-local` → `lane=local`
- anything else → no-op

Listen to `issue_comment` (created, and only when the issue is a pull
request), `pull_request_review` (submitted), and
`pull_request_review_comment` (created). Never interpolate the body into
a shell command; put it in `env` and branch in bash or a tiny
`rocci-ops` helper.

Authorization on a **hosted** `ubuntu-latest` job, before any
`workflow_call`:

1. Resolve PR number from the event. Reject non-PRs.
2. Fetch `pulls/{n}`; record `head.sha` only if it matches
   `^[0-9a-f]{40}$`.
3. Hosted `/ci`: commenter association in
   `{OWNER, MEMBER, COLLABORATOR}`. Any head repo is allowed (forks
   included).
4. Local `/ci-local` or `/cl-local`: commenter login `koliyo`, PR author
   `koliyo`, `head.repo.full_name == github.repository`. Otherwise comment
   that local CI was denied and exit 0.
5. Post a PR comment naming the lane and the exact SHA.

Concurrency group is `ci-command-{pr_number}-{lane}`, not `github.ref`
(comment workflows often resolve `ref` to `main` and would cancel other
PRs).

**Exit:** The default-branch dispatcher accepts `/CI` in a review body
from `koliyo`; treats `/ci-local` as local (not hosted); accepts
`/cl-local` as the same local lane; ignores `/ci` from a
`FIRST_TIME_CONTRIBUTOR` or `NONE` association; ignores `/circle`; and
refuses local CI on a non-`koliyo` or fork PR. No `pull_request` trigger
exists.

## Phase 2 — Hosted automatic CI on protected branches

**Bound:** `.github/workflows/ci.yml` and `.github/workflows/knowledge.yml`.
Add a `lane` input (`hosted` \| `local`) and optional `pr_number` /
`pr_sha`. Keep a single job list; select `runs-on` from `lane`.[^ci-workflow][^knowledge-workflow]

Triggers:

```yaml
on:
  push:
    branches: [main, staging, production]
  workflow_call:
    inputs: { lane, pr_number, pr_sha }
  workflow_dispatch:
    inputs: { lane, pr_number, pr_sha }
```

Rules:

- `push` and `workflow_dispatch` with `lane=hosted` (default) run on
  GitHub-hosted runners at `github.sha` / the selected ref. No authorize
  job.
- `workflow_call` with `lane=hosted` checks out `pr_sha` when present,
  else `github.sha`. `persist-credentials: false` whenever `pr_sha` is
  set.
- `lane=local` keeps today's authorize job (malformed inputs, author
  `koliyo`, same-repo head, SHA match) on `ubuntu-latest`, then today's
  self-hosted labels. Hosted jobs in that run are skipped.
- Never attach an `environment:` or `secrets:` block.
- Disable `Swatinem/rust-cache` (or key it `pr-${{ inputs.pr_number }}`
  and never share with branch builds) whenever `pr_sha` is set, so a
  comment-triggered hosted run cannot poison `main`'s cache.[^harden-docs]
- Concurrency: `ci-{lane}-{pr_number or github.ref}`.

Hosted matrix: `lint`, `fixtures-and-docs` on `ubuntu-latest`; `test` on
`ubuntu-latest` and `macos-latest`; `editors` on `macos-latest` (Node +
WASI). Knowledge hosted on `macos-latest` or `ubuntu-latest` (pick one;
document it). Local lane may keep the current self-hosted OS split.

**Exit:** `rg -n "pull_request" .github/workflows` is empty for these
files. A dry read of `ci.yml` shows `push.branches` includes `main`,
`staging`, and `production`, and every self-hosted `runs-on` is behind
`lane == local` plus `needs.authorize.outputs.allowed == 'true'`.
`uv run rocci-ops ci --list` is unchanged.[^ops-ci]

## Phase 3 — Secret and runner isolation

**Bound:** `.github/workflows/site.yml`, `.github/workflows/release.yml`,
checkout/cache/pinning on all workflows, and the operator checklist in
`docker/prod/README.md`.[^site-workflow][^release-workflow][^prod-readme]

YAML:

- Site **package** runs only when
  `github.ref` is `refs/heads/staging` or `refs/heads/production`. Drop
  the "package only on other refs" path. `workflow_dispatch` on a feature
  branch becomes a no-op.
- Deploy stays `environment: ${{ github.ref_name }}` with the same
  `if:`. After writing `$HOME/.ssh/deploy`, delete that file in a
  `always()` step (`shred`/`rm`). Prefer a distinct runner label
  `rocci-deploy` so CI local jobs and deploy never share a host; if the
  label does not exist yet, document it as an operator step and still
  wipe the key.
- Release `workflow_dispatch` documents that it is owner-only; once the
  repo is public, a ruleset should restrict `v*` tags. Do not add
  environment secrets to Release.
- Pin third-party actions to full commit SHAs (leave `actions/*` on
  official tags or pin them too). Set
  `persist-credentials: false` on every checkout of `pr_sha`.
- Add `.github/CODEOWNERS` with `.github/ @koliyo`.

Operator (GitHub UI, record completion in the README checklist; do not
invent a verification event):

- Staging environment: custom branch policy `staging` only (match
  production's `production` allow-list). `protected_branches` is not
  enough on a free private repo.
- Keep deploy secrets Environment-only. Do not copy them to repository
  secrets.
- Self-hosted runners: this repository only, not org-wide.
- After public: enable a `main` / `staging` / `production` ruleset
  (available on public repos without Pro). Do not require the CI check
  on PRs.
- Leave Actions default token read-only; do not grant Actions the right
  to approve reviews.
- Fork-PR approval: require approval for all outside collaborators
  (defense in depth; this plan still must not add `pull_request`).

**Exit:** `site.yml` package and deploy both require staging/production
refs. No workflow other than `site.yml` deploy references
`DEPLOY_*` or `CF_ACCESS_*`. `secrets: inherit` is absent.
`docker/prod/README.md` lists the staging custom-branch policy and the
key-wipe / runner-label notes.

## Phase 4 — Dependabot

**Bound:** new `.github/dependabot.yml` only. No automerge. No
self-hosted execution of Dependabot heads (Phase 1 already forbids
`/ci-local` on non-`koliyo` authors; `dependabot[bot]` is not
`koliyo`).[^dependabot-docs]

Weekly version updates, `open-pull-requests-limit: 10`, label
`dependencies`, grouped where the ecosystem allows:

| `package-ecosystem` | `directory` |
| --- | --- |
| `cargo` | `/` |
| `cargo` | `/editors/zed` |
| `uv` | `/` |
| `npm` | `/editors/vscode` |
| `github-actions` | `/` |
| `docker` | `/docker/runtime`, `/docker/app`, `/docker/cdn`, `/docker/islands` (or `directories:`) |

Do not enable a pip ecosystem alongside `uv`. Commit-message prefixes:
`chore(deps)`, `ci` for Actions. Group rust crates as `cargo-workspace`,
Actions as `github-actions`.

In the repo UI (not the YAML): turn on Dependabot security updates and
Dependabot alerts. Reviewers run `/ci` on those PRs; they do not run
automatically.

**Exit:** The file exists on the default branch. A maintainer can open
the Dependabot tab and see the configured ecosystems. Docs in Phase 5
state that Dependabot PRs need `/ci` and must never get `/ci-local`.

## Phase 5 — Contributor and operator docs

**Bound:** root `README.md` Tests section, `.agents/skills/rocci-devops/SKILL.md`,
`site/project/contributing.rocdown` (CI subsection only). Do not expand
the public-preview plan's Phase 0 into this work.[^root-readme][^devops-skill]

State the trigger matrix from this record. Mention that `/ci` is hosted
and reviewer-gated; `/ci-local` is `koliyo` plus same-repo `koliyo` PRs;
protected branches are automatic hosted CI; secrets stay on Environments.

**Exit:** `rg -n "/ci" README.md .agents/skills/rocci-devops/SKILL.md`
describes both commands and hosted automatic branches. The contributing
page tells external authors that a maintainer comments `/ci` after
review.

## Hand-off after Exit

Land Phases 1–5 on `main` while the repo is still private, then exercise
`/CI` and `/ci-local` on a same-repo `koliyo` PR and `/ci` denial on a
throwaway fork if one exists. Flip visibility only after the [public
preview](/plans/site/public-preview-community.md) launch gate and the
[public-launch checklist](/audits/site/rocci-dev-public-launch.md) are also
satisfied. This plan does not order that flip.

[^audit]: Current-behavior findings F1–F7 against the 2026-08-22 workflows and live Environment settings.
[^ci-workflow]: `.github/workflows/ci.yml` currently requires `pr_number`/`pr_sha` and runs self-hosted after authorize.
[^ci-command]: `.github/workflows/ci-command.yml` is `issue_comment` only, lowercase `/ci`, commenter and author `koliyo`.
[^knowledge-workflow]: `.github/workflows/knowledge.yml` mirrors the CI authorize-then-self-hosted shape.
[^site-workflow]: `.github/workflows/site.yml` deploy job uses `environment: ${{ github.ref_name }}` on staging/production only.
[^release-workflow]: `.github/workflows/release.yml` is tag- and dispatch-triggered with `contents: write`.
[^ops-ci]: `rocci-ops ci` owns lint, test, fixtures-and-docs, editors, and knowledge job bodies.
[^root-readme]: README Tests section is the current contributor-facing CI contract.
[^devops-skill]: DevOps skill documents `/ci` plus `ci.yml` / `knowledge.yml` / `ci-command.yml`.
[^prod-readme]: Operator README already requires Environment secrets and matching-branch restriction.
[^preview-plan]: Public-preview plan does not itself flip visibility or define CI policy.
[^harden-docs]: GitHub hardening guidance for secrets, caches, and self-hosted isolation.
[^self-hosted-docs]: Self-hosted runners persist state and are unsafe for untrusted public-repo heads.
[^pr-target-docs]: Privileged events must not execute untrusted checkouts; this plan never adds `pull_request_target`.
[^env-docs]: Environment secrets are available only to jobs that declare that environment and pass its protection rules.
[^dependabot-docs]: Official `dependabot.yml` ecosystems used in Phase 4.
