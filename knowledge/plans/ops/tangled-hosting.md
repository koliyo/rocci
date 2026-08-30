---
type: Implementation Plan
title: Tangled hosting and devops with a GitHub macOS mirror
description: "Deferred proposal to make Tangled the canonical git host, review surface, and Linux CI while GitHub becomes a macOS mirror. Rocci is not using Tangled at this point; GitHub remains the active repository, CI, and deployment path."
tags: [domain/rocci, concern/ci, concern/governance, concern/publication, concern/community]
status: draft
generated: { by: process:codex, at: 2026-08-20T17:49:09Z }
stale_after: 2026-11-19
authority: exploratory
owners: [human:nils]
sources:
  - id: maintainer-decision
    resource: ../../log.md
    title: Maintainer decision to defer Tangled
    author: human:nils
    last_modified: 2026-08-20
  - id: publish-plan
    resource: ../rocci-dev-publish.md
    title: rocci.dev deployment preparation status
    author: human:nils
    last_modified: 2026-08-20
  - id: research
    resource: ../../research/ops/tangled-hosting.md
    title: Tangled as canonical host with a GitHub macOS CI mirror
    author: process:cursor
    last_modified: 2026-08-19
  - id: hosting-research
    resource: ../../research/ops/repository-hosting-and-distributed-governance.md
    title: Repository hosting for Rocci's distributed governance
    author: process:codex
    last_modified: 2026-08-18
  - id: preview-plan
    resource: ../public-preview-community.md
    title: Rocci public-preview branding and community plan
    author: process:cursor
    last_modified: 2026-08-19
  - id: ci-workflow
    resource: ../../../.github/workflows/ci.yml
    title: Rocci GitHub Actions CI workflow
    author: process:git
    last_modified: 2026-08-19
  - id: knowledge-workflow
    resource: ../../../.github/workflows/knowledge.yml
    title: Rocci GitHub Actions knowledge workflow
    author: process:git
    last_modified: 2026-08-17
  - id: release-workflow
    resource: ../../../.github/workflows/release.yml
    title: Rocci GitHub Actions release workflow
    author: process:git
    last_modified: 2026-08-18
  - id: ci-local
    resource: ../../../tools/rocci-ops/src/rocci_ops/ci.py
    title: Local CI job runner
    author: process:git
    last_modified: 2026-08-19
  - id: devops-skill
    resource: ../../../.agents/skills/rocci-devops/SKILL.md
    title: Rocci GitHub Actions devops skill
    author: process:git
    last_modified: 2026-08-19
  - id: knowledge-skill
    resource: ../../../.agents/skills/manage-rocci-knowledge/SKILL.md
    title: Rocci knowledge-bundle skill
    author: process:git
    last_modified: 2026-08-18
  - id: docs-config
    resource: ../../../docs/rocdown.toml
    title: rocci.dev site configuration
    author: process:git
    last_modified: 2026-08-19
  - id: site-config
    resource: ../../../site/rocdown.toml
    title: rocci.dev unified site configuration
    author: process:git
    last_modified: 2026-08-18
  - id: root-readme
    resource: ../../../README.md
    title: Rocci workspace overview
    author: human:nils
    last_modified: 2026-08-19
  - id: tangled-spindles
    resource: https://docs.tangled.org/spindles
    title: Tangled spindle pipelines
    author: organization:tangled
  - id: tangled-docs
    resource: https://docs.tangled.org/single-page
    title: Tangled documentation
    author: organization:tangled
  - id: tangled-pages
    resource: https://docs.tangled.org/hosting-websites-on-tangled
    title: Hosting websites on Tangled
    author: organization:tangled
  - id: atproto-handle
    resource: https://atproto.com/specs/handle
    title: AT Protocol handle specification
    author: organization:bluesky
  - id: rocci-bsky
    resource: https://bsky.app/profile/rocci.bsky.social
    title: Existing unrelated rocci.bsky.social account
    author: human:helene-perndl
  - id: tangled-signup
    resource: https://tangled.org/tangled.org/core/blob/master/appview/signup/signup.go
    title: Tangled signup email uniqueness check
    author: organization:tangled
  - id: cf-email-routing
    resource: https://developers.cloudflare.com/email-routing/get-started/enable-email-routing/
    title: Enable Cloudflare Email Routing
    author: organization:cloudflare
  - id: cf-email-route
    resource: https://developers.cloudflare.com/email-routing/get-started/route-emails/
    title: Cloudflare Email Routing destination rules
    author: organization:cloudflare
  - id: hexonet-verify
    resource: https://wiki.hexonet.net/wiki/Registrant_Verification
    title: HEXONET/EPAG registrant email verification and NS replacement
    author: organization:hexonet
---

# Tangled hosting and devops with a GitHub macOS mirror

## Current decision

Rocci is not using Tangled at this point. GitHub remains the active canonical
repository, CI surface, and deployment path. This plan is deferred: none of
its phases is a public-launch gate, and no Tangled account, remote, spindle,
or mirror work should begin without a new maintainer decision.[^maintainer-decision]

## Deferred proposal

Run Rocci's repository, review, and Linux devops on Tangled, and keep GitHub
only as a SHA-faithful mirror that provides `macos-latest` runners. This plan
does not move `rocci.dev` onto Tangled Sites, does not import GitHub issues or
pull requests, and does not treat a successful pilot as distributed
governance.[^research][^hosting-research][^tangled-pages]

Rocci is intended to become public open source shortly. The first public clone
URL, issues, and pull requests should already be Tangled; do not open on
GitHub and migrate after two release cycles. Public-preview Phase 0 (license
texts, conduct, contribution, support) remains the publication gate and should
land on the canonical Tangled surface.[^preview-plan][^research]

## Proposed future position (deferred)

- Canonical git remote, issues, and pull requests live on a Tangled knot,
  viewed at tangled.org.
- Spindle runs every job that does not need Darwin: lint, Linux workspace and
  doc tests, AST fixtures, docs check, OKF knowledge, and Linux release
  archives.
- GitHub `koliyo/rocci` (or a successor mirror under a Rocci org) receives
  fast-forward-only pushes of branches and tags. GitHub Actions there run
  Darwin workspace tests, VS Code extension-host tests, and
  `aarch64-apple-darwin` archives.
- Contributors open Tangled pull requests. GitHub pull requests are redirected
  and never merged as the source of truth.
- `uv run rocci-ops ci` remains the local job list. Spindle and the slimmed
  GitHub workflows call those commands instead of diverging.[^ci-local][^research]

## Job ownership

| Job | Tangled spindle | GitHub Actions on the mirror |
| --- | --- | --- |
| Workspace-deps, ungram `--check`, fmt, clippy | Required | Drop after spindle is green |
| `cargo test --workspace` / `--doc` on Linux | Required | Drop |
| Same tests on macOS | No public Darwin runner | Required |
| AST fixtures and `rocdown check docs` | Required | Drop |
| Zed `wasm32-wasip1` check | Optional extra | Keep inside the editors job |
| VS Code lint/compile/extension-host tests | No | Required |
| OKF tests, `check --profile base`, deterministic build | Required (Linux, full clone) | Drop; today's `macos-latest` pin is not a Darwin need |
| Linux release tarball | Preferred | Keep on GitHub only if artifact ferrying is worse |
| Darwin release tarball | No | Required |
| GitHub Release publication | No | Keep as the public binary channel until Tangled has an equivalent |
| `rocci.dev` | Out of scope | Out of scope; custom domains are unimplemented on Tangled Sites |[^ci-workflow][^knowledge-workflow][^release-workflow][^tangled-pages][^research]

If the proposal is resumed, Phases 0–4 would be launch-blocking for a future
open-source clone:
identity, dual remotes, Linux spindle, the GitHub Darwin mirror, and the
origin flip. Public-preview Phase 0 (license texts, conduct, contribution)
must land on that Tangled surface before the repository is public. Phases 5–7
may trail the first public clone if Darwin status is documented as applying
to mirrored origin refs and tags only.[^preview-plan]

## Phase 0 — identity and spindle evidence

Create a **dedicated** AT Protocol account for the project. Do not reuse a
personal Bluesky or Tangled login. Bare `rocci` is not a valid handle;
handles are DNS hostnames with at least two labels.[^atproto-handle]

| Wanted | What it actually is | Status on 2026-08-19 |
| --- | --- | --- |
| `rocci` | Invalid as an AT handle | Use `rocci.tngl.sh` or `rocci.dev` |
| `rocci.tngl.sh` | Tangled PDS username | Unresolved; try it at signup |
| `rocci.dev` | Custom-domain handle you already own for the product | Unresolved as a handle; preferred public identity |
| `rocci.bsky.social` | Unrelated existing Bluesky account | Taken; do not use |

Prefer `rocci.dev` as the public handle. Register first on Tangled’s PDS so
the project account is not hosted on Bluesky. `rocci.bsky.social` is already
someone else’s Bluesky account.[^rocci-bsky][^atproto-handle]

Steps:

1. Use a **Rocci-specific mailbox** on `rocci.dev` (see Mail at rocci.dev
   below). Tangled signup rejects an email that already exists. Do not use
   the personal address already on another Tangled account.[^tangled-signup]
2. Open [tangled.org/signup](https://tangled.org/signup) and create an
   account with username `rocci`. That yields `rocci.tngl.sh` on Tangled’s
   PDS (`tngl.sh`).[^tangled-docs]
3. Add an SSH key under Settings → Keys.
4. To switch the handle to `rocci.dev`, add a DNS TXT record and then tell
   the PDS the new handle:
   - name `_atproto.rocci.dev`
   - value `did=did:plc:…` (the DID from Settings after signup)
   - then update the handle on the PDS (`com.atproto.identity.updateHandle`,
     or the account/handle control in Settings → Profile if Tangled exposes
     it). The DID document must list `at://rocci.dev` in `alsoKnownAs`.[^atproto-handle]
5. Create the `rocci` repository on a managed knot. Clone URLs become
   `git@tangled.org:rocci.dev/rocci` once the handle is `rocci.dev`.

The `rocci.dev` website can keep serving at the apex. The TXT record lives
on the `_atproto` subdomain and does not replace the site. Tangled Sites
still cannot use `rocci.dev` as a Pages hostname.[^tangled-pages]

### Mail at rocci.dev

Need is inbound only: Tangled (and later GitHub, security reports) must be
able to deliver to `oss@rocci.dev`. Do not buy Google Workspace or a
mailbox host for that. Cloudflare Email Routing is free on the Cloudflare
Free plan, adds MX/SPF/DKIM itself, and forwards to an existing inbox.
Outbound SMTP is a later, separate product if the project must *send* as
`@rocci.dev`.[^cf-email-routing][^cf-email-route]

The former registrar-verification DNS state is historical: on 2026-08-20, the
maintainer reported that ICANN registrant-email verification, the Cloudflare
nameserver cutover, and tested `oss@rocci.dev` forwarding were complete. The
personal destination inbox remains the forwarding target; `security@rocci.dev`
is intentionally deferred.[^publish-plan][^hexonet-verify]

Recommended path (zero extra monthly cost):

1. Complete registrar verification so real nameservers are restored.
2. Add `rocci.dev` to Cloudflare and switch the domain’s NS to Cloudflare
   (keep the current website origin as an A/AAAA or CNAME; `.dev` is
   HSTS-preloaded, so the site must stay on HTTPS).
3. Email → Email Routing → Get started. Cloudflare writes MX, SPF, and
   routing DKIM. Unlock those records only if you later leave Cloudflare
   mail.[^cf-email-routing]
4. Verify a destination inbox (the personal mailbox is fine as the
   *forward target*, not as the Tangled signup address).
5. Create custom addresses, not a catch-all at first (less spam):
   - `oss@rocci.dev` — Tangled account, forge recovery, GitHub/org mail
   - `security@rocci.dev` — later public security contact
6. Confirm with a test message to `oss@rocci.dev`, then use that address
   on tangled.org/signup.
7. Add `_dmarc.rocci.dev` TXT `v=DMARC1; p=none;` once routing works. Do
   not set `p=quarantine`/`reject` until sending is intentional.
8. Keep `_atproto.rocci.dev` as a separate TXT; it does not conflict with
   MX.

A plus-alias on a personal mailbox remains a last-resort signup workaround
if DNS is still blocked. It is not the project identity.

Then push all branches and tags, and run a smoke `.tangled/workflows`
pipeline that compiles and tests a thin crate subset.

**Done when:**

- AT handle, knot hostname, and repo name are recorded.
- A spindle run on that repo has a pipeline ID, log URL, and measured wall
  time.
- Timeout, Nix/GTK/`webkitgtk`, and `rustc` availability are known. If the
  hosted 5-minute default cannot finish even the smoke job, a self-hosted
  spindle with a raised `SPINDLE_*_PIPELINES_WORKFLOW_TIMEOUT` is in scope
  for Phase 2 rather than a later surprise.[^tangled-spindles][^research]

**Status:** not started.

## Phase 1 — dual remotes, GitHub still origin

Add a `tangled` remote beside `origin`. Maintainers can push both. Do not
flip `origin`, README clone URLs, or the knowledge completion gate.

**Done when:** every branch and tag that GitHub has also exists on Tangled,
and a documented push recipe exists for the current maintainer clones.[^tangled-docs]

**Status:** not started.

## Phase 2 — Linux jobs on spindle

Add `.tangled/workflows` for `lint`, Linux `test`, `fixtures-and-docs`, and
`knowledge`. Prefer the `microvm` NixOS engine so `pkg-config`, GTK, and
WebKit match today's `apt-get` Linux extras. Set clone depth so knowledge
provenance sees full history. Keep GitHub Linux jobs running in parallel
until spindle is boringly green.[^tangled-spindles][^ci-workflow][^knowledge-workflow][^ci-local]

**Done when:** the same revision is green on spindle Linux jobs and on
current GitHub Linux jobs, and the spindle timeout policy is written down
(hosted vs self-hosted).

**Status:** not started.

## Phase 3 — GitHub becomes the Darwin slice plus mirror pusher

Slim `.github/workflows/ci.yml` to `macos-latest` workspace tests and the
editors job. Move or delete `knowledge.yml` from GitHub once spindle
knowledge is required. Add a Tangled-side mirror workflow (deploy key,
fast-forward only, never `--force`) that pushes refs to GitHub so Actions
fire on the same SHAs.[^ci-workflow][^research]

Pull-request SHAs from Tangled forks need an explicit ref on GitHub (for
example `refs/ci/pr/<id>`) or Darwin coverage is maintainer-branch-only.
Choose that policy here; do not leave it implicit.

**Done when:** a push to Tangled `main` appears on GitHub at the same SHA
and starts Darwin Actions; GitHub Linux duplicate jobs are gone; the PR
mirror policy is written in README.

**Status:** not started.

## Phase 4 — flip canonical origin

Set `origin` to Tangled. Point `docs/rocdown.toml` and `site/rocdown.toml`
`repository` at the Tangled URL. README clone instructions, contribution
text, and editor `repository` fields that are not marketplace-constrained
follow. Keep GitHub URLs in VS Code / Zed marketplace manifests if those
stores require them.[^docs-config][^site-config][^root-readme][^preview-plan]

Protect GitHub `main` as a mirror: no GitHub-native merges. A short issue
template or README section tells GitHub visitors to open Tangled pull
requests.

**Done when:** a new clone from the documented URL lands on Tangled, and a
GitHub-only pull request is rejected or redirected by policy.

**Status:** not started.

## Phase 5 — split merge gate

Linux spindle status is required to merge on Tangled. Darwin GitHub Actions
status is required on mirrored SHAs that land on `main` and on `v*` tags.
If Phase 3 mirrored PR refs, Darwin is also required on those Tangled PRs;
otherwise README states that macOS is post-push-to-origin.

Do not claim a single forge UI shows both results unless a poller or webhook
comment is implemented. A spindle step that waits on `gh run` for
`${TANGLED_SHA}` is enough if a GitHub token is stored as a spindle
secret.[^research][^devops-skill][^tangled-spindles]

**Done when:** merging to canonical `main` is impossible when spindle Linux
fails, and tagging `v*` is impossible when Darwin Actions on the mirrored
SHA fail.

**Status:** not started.

## Phase 6 — releases

Keep GitHub Releases as the download channel. On `v*` tags:

1. Tangled records the tag and mirrors it.
2. Spindle builds the Linux archive, or GitHub `ubuntu-latest` still does if
   uploading from spindle is more operational risk than it is worth.
3. GitHub `macos-latest` builds the Darwin archive.
4. Replace `ci-gate`'s hard-coded GitHub check names with: spindle Linux
   (or remaining GitHub Linux) plus Darwin tests.

`workflow_dispatch` dev releases stay GitHub-side until Tangled manual
triggers plus artifact upload are proven.[^release-workflow]

**Done when:** a tagged revision publishes both platform archives to one
GitHub Release, and the gate waited for Linux and Darwin rather than the
pre-split check names.

**Status:** not started.

## Phase 7 — devops docs and knowledge gate

Update `rocci-devops` to inspect spindle pipelines and GitHub Darwin runs.
Change the knowledge completion rule from "GitHub CI and Knowledge
workflows" to "required Tangled Linux pipelines and GitHub Darwin jobs on
that revision," citing pipeline IDs and GitHub run IDs. README's
Local CI docs should mention both forges.[^devops-skill][^knowledge-skill][^root-readme]

**Done when:** an agent following the skills can find the blocking Linux
pipeline and the blocking macOS run without reading this plan.

**Status:** not started.

## Out of scope

- Moving `rocci.dev` to Tangled Sites (no custom domains).
- Importing GitHub issues, pull requests, or Actions history.
- Native macOS spindle workers (still a Tangled proposal).
- Making Tangled equal to distributed governance; knot operators and
  release signing remain separate decisions.
- Splitting the Cargo workspace into separate git remotes. Tangled can host
  sibling repos under one handle; it has no org or nested group namespace.
  Keep one `rocci` git repository unless a later product decision splits
  crates.[^tangled-pages][^hosting-research][^research]

## Validation

Phase 0 is a live spindle smoke run, not a Rocci unit test. Later phases
still run `uv run rocci-ops ci` locally. After Phase 2, do not log a
phase complete in `knowledge/log.md` until the then-required Tangled Linux
pipelines and, once Darwin is mirrored, GitHub macOS jobs are green on that
revision. Do not set `ROCCI_REQUIRE_ROC=1`.

## Open product questions

1. Handle, knot, and whether the GitHub mirror stays `koliyo/rocci`. Prefer
   `rocci.dev` as the AT handle; `rocci.tngl.sh` is the signup fallback.
2. Hosted versus self-hosted spindle after Phase 0 timings.
3. Darwin required on every Tangled PR, or only on mirrored origin refs.
4. Linux release artifacts from spindle or leftover GitHub Ubuntu.
5. Date at which GitHub issues are closed with a pointer to Tangled.
6. Calendar date for the public open-source clone; only “shortly” is decided.
7. Dedicated `rocci` AT handle with sibling repos versus the current single
   workspace remote. Tangled grouping stops at handle-owned siblings.

[^research]: Operational constraints for the inverse topology, timeouts, pages, and split status.
[^maintainer-decision]: The maintainer deferred Tangled adoption in the
2026-08-20 Codex task; GitHub remains active for repository, CI, and deploy.
[^publish-plan]: Maintainer-reported registrar, Cloudflare DNS, and inbound-mail completion; the deployment plan retains the remaining origin work.
[^hosting-research]: Prior comparison and the governance warning that hosting is not governance.
[^preview-plan]: Near-term public open-source intent; Phase 0 (license texts, conduct, contribution) remains the publication gate.
[^ci-workflow]: Current four-job GitHub CI matrix including Darwin tests and macOS editors.
[^knowledge-workflow]: OKF job currently on `macos-latest` with full-history checkout.
[^release-workflow]: Tag builds, GitHub check-run gate, and GitHub Release upload.
[^ci-local]: Shared lint, test, fixture, editor, and knowledge commands.
[^devops-skill]: Agent CI inspection is GitHub-only today.
[^knowledge-skill]: Bundle skill currently names GitHub CI and Knowledge as the completion gate.
[^docs-config]: Documentation site `repository` URL.
[^site-config]: Unified site `repository` URL.
[^root-readme]: Clone and local-CI instructions.
[^tangled-spindles]: Workflow YAML, engines, clone depth, timeouts, secrets, PR SHAs.
[^tangled-docs]: Remote migration, dual remotes, webhooks, no GitHub metadata import.
[^tangled-pages]: Static sites without custom domains.
[^atproto-handle]: Handles are DNS names; `rocci` alone is invalid. Custom domains use `_atproto` TXT plus DID `alsoKnownAs`.
[^rocci-bsky]: `rocci.bsky.social` is already a personal Bluesky account, not this project.
[^tangled-signup]: Signup looks up the email string in Tangled's emails table and returns "Email already exists" on a hit.
[^cf-email-routing]: Free inbound forwarding; Cloudflare DNS required; MX/SPF/DKIM added on enable.
[^cf-email-route]: Custom addresses forward to a verified destination mailbox; catch-all is optional.
[^hexonet-verify]: Unverified registrant email causes nameserver replacement and the domain stops resolving.
