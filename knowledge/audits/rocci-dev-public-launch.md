---
type: Audit
title: rocci.dev public-launch checklist
description: Current rocci.dev is close to a public preview, but GitHub is still private, News 308s still land on retired academy URLs, an unpublished first-use page is in sitemap and the page finder, and repository community-health files are missing.
tags: [domain/rocci, domain/rocdown, concern/publication, concern/community, concern/ux, concern/navigation]
status: draft
generated: { by: process:cursor, at: 2026-08-22T14:45:00Z }
stale_after: 2026-11-22
authority: descriptive
owners: [human:nils]
sources:
  - id: site-config
    resource: ../../site/rocdown.toml
    title: Current rocci.dev catalog, mounts, and navigation
    author: human:nils
    last_modified: 2026-08-22
  - id: landing
    resource: ../../site/index.rocdown
    title: Current rocci.dev landing page
    author: human:nils
    last_modified: 2026-08-22
  - id: faq
    resource: ../../site/faq/index.rocdown
    title: Current rocci.dev FAQ
    author: process:git
    last_modified: 2026-08-22
  - id: project-status
    resource: ../../site/project/status.rocdown
    title: Current public project status page
    author: process:git
    last_modified: 2026-08-22
  - id: contributing-page
    resource: ../../site/project/contributing.rocdown
    title: Current public contributing page
    author: process:git
    last_modified: 2026-08-18
  - id: install
    resource: ../../docs/install.rocdown
    title: Public install and clone instructions
    author: process:git
    last_modified: 2026-08-22
  - id: inventory
    resource: ../../docs/inventory.toml
    title: Stack-first retired-route list and launch vocabulary
    author: process:git
    last_modified: 2026-08-22
  - id: first-use
    resource: ../../docs/reference/contributor/first-use.rocdown
    title: Unpublished first-use protocol page
    author: process:git
    last_modified: 2026-08-22
  - id: caddy
    resource: ../../docker/cdn/Caddyfile
    title: Production hybrid Caddy News redirects and 410s
    author: process:git
    last_modified: 2026-08-22
  - id: caddy-test
    resource: ../../tools/rocci-ops/tests/test_example_origins.py
    title: Origin tests that pin the current News redirect targets
    author: process:git
    last_modified: 2026-08-22
  - id: ux-contract
    resource: ../../site/tests/rocci-dev-site-ux-contract.toml
    title: Phase 0 chrome contract still listing academy News targets
    author: process:codex
    last_modified: 2026-08-22
  - id: planner
    resource: ../../crates/rocci-rocdown/src/plan.rs
    title: Sitemap, pages.json, and llms.txt include every non-draft page
    author: process:git
    last_modified: 2026-08-22
  - id: site-shell
    resource: ../../site/theme/SiteShell.rocci
    title: Current rocci.dev document shell and experimental banner
    author: process:git
    last_modified: 2026-08-22
  - id: layouts
    resource: ../../site/theme/Layouts.rocci
    title: Current navigated frame used by non-home layouts
    author: process:git
    last_modified: 2026-08-22
  - id: root-readme
    resource: ../../README.md
    title: Current workspace overview and first-run copy
    author: human:nils
    last_modified: 2026-08-22
  - id: roadmap
    resource: ../../ROADMAP.md
    title: Root implementation roadmap still naming @on
    author: human:nils
    last_modified: 2026-08-17
  - id: preview-plan
    resource: ../plans/public-preview-community.md
    title: Public-preview branding and community plan
    author: process:cursor
    last_modified: 2026-08-21
  - id: site-plan
    resource: ../plans/rocci-dev-site.md
    title: rocci.dev UX and authoring improvement plan
    author: process:cursor
    last_modified: 2026-08-22
  - id: publish-plan
    resource: ../plans/rocci-dev-publish.md
    title: rocci.dev Cloudflare and VPS deploy plan
    author: process:cursor
    last_modified: 2026-08-21
  - id: ux-audit
    resource: rocci-dev-site-ux-dx.md
    title: Prior rocci.dev UX and authoring DX review
    author: process:cursor
    last_modified: 2026-08-22
  - id: github-repo
    resource: https://github.com/koliyo/rocci
    title: koliyo/rocci GitHub repository metadata
    author: organization:github
  - id: github-health
    resource: https://docs.github.com/en/communities/setting-up-your-project-for-healthy-contributions/creating-a-default-community-health-file
    title: GitHub community health files
    author: organization:github
  - id: gitattributes
    resource: ../../.gitattributes
    title: Current Git attributes without linguist vendoring
    author: process:git
    last_modified: 2026-08-17
  - id: vscode-readme
    resource: ../../editors/vscode/README.md
    title: VS Code extension README listing @on completions
    author: process:git
    last_modified: 2026-08-21
  - id: example-caddy
    resource: ../../docker/examples/Caddyfile
    title: Planned live example hostnames
    author: process:git
    last_modified: 2026-08-21
  - id: apps-catalog
    resource: ../../examples/rocci/apps.toml
    title: Cataloged example apps and live hosting flags
    author: process:git
    last_modified: 2026-08-22
  - id: known-limits
    resource: ../status/known-limitations.md
    title: Canonical known-limitations snapshot
    author: process:cursor
    last_modified: 2026-08-22
---

# rocci.dev public-launch checklist

## Executive verdict

The public site is close enough to show as an experimental preview. Home,
Docs, FAQ, Project status, and the live-counter island already tell a bounded
stack story, mark maturity, and keep News out of navigation.[^landing][^site-config][^project-status][^ux-audit]

Do not flip the hostname or the repository public until the items in
**Must before public** are done. The remaining work is mostly origin redirects,
repository visibility, first-contact copy, and GitHub community files — not
another chrome redesign.[^preview-plan][^caddy][^github-repo]

This audit does not authorize publication.

## Scope and method

Reviewed on 2026-08-22 against `main` (`2e399ea`). Sources were the current
`site/`, mounted `docs/`, origin Caddy files, root README, and the existing
publication plans. A local `rocdown view site` preview at
`http://127.0.0.1:58195/` was used for Home: path cards, experimental banner,
and the live island were visible and the increment control updated the shared
count.

This pass did not rebuild `dist/rocci.dev`, did not hit production `rocci.dev`
(still unrouted per the deploy plan), and did not repeat the prior
keyboard/zoom/forced-colors matrix.[^publish-plan][^ux-audit]

## What is already launch-shaped

- Global lanes are Docs, Examples, FAQ, and Project. There is no News lane
  and no top-level `/rocdown/` product lane.[^site-config]
- Home leads with a stack sentence, five path cards, a hybrid proof, and an
  explicit maturity boundary. The live island rendered and accepted
  increments in the local preview.[^landing]
- Docs nav is stack-first: Start, Templates, Applications, Rocdown,
  Reference, Troubleshooting. Install tells visitors to clone and build from
  source, with GitHub archives marked experimental.[^site-config][^install][^inventory]
- FAQ answers are short and point at canonical owners. Project status lists
  shipped work and deliberate limits without `@island` as a current
  syntax.[^faq][^project-status]
- Non-home layouts share `NavigatedFrame`. Home is the sidebar-free
  landing layout.[^layouts][^site-plan]
- License text is Apache-2.0 at the repository root. Site footer and Home
  banner say the software is experimental and not for production
  use.[^root-readme][^preview-plan][^site-shell]
- Staging deploy exists behind Cloudflare Access; production hostname
  routing remains a launch decision.[^publish-plan]

## Must before public

These are launch blockers. A signed-out visitor hitting `rocci.dev` or the
clone URL will feel them immediately.

### 1. Make the advertised GitHub repository actually public

**Severity:** P0. The site, install page, and header all point at
`https://github.com/koliyo/rocci`. That repository is still
**private**.[^site-config][^install][^github-repo]

Until it is public, Install's `git clone` fails for strangers, the header
GitHub icon 404s or asks for login, and the public-preview plan's "unfamiliar
developer can install without private instructions" exit is unmet.[^preview-plan]

Do this as an explicit maintainer action, not as a site copy workaround.

### 2. Retarget News 308s to the current canonical pages

**Severity:** P0 for anyone following old `/news/*` URLs, including crawlers
after the hostname is public.

Production Caddy still redirects:

| From | Current 308 target | Canonical owner after the stack cut |
| --- | --- | --- |
| `/news/introducing-rocci/` | `/docs/start/what-is-rocci/` | `/docs/the-stack/` |
| `/news/rocdown-static-collections/` | `/rocdown/site-config/` | `/docs/rocdown/sites/` |
| `/news/rocci-desktop-apps/` | `/docs/tutorials/ship/` | `/docs/applications/package/` |

Those targets are in the retired-route list and are supposed to 404. The
origin test pins the stale strings, and the chrome-contract fixture still
lists the academy targets. `/news/` and `/news/feed.xml` already 410, which
is correct.[^caddy][^caddy-test][^ux-contract][^inventory][^site-plan]

Update Caddy, the origin test, and the contract fixture together.

### 3. Stop publishing the "unpublished" first-use page

**Severity:** P1 for public catalog honesty.

`docs/reference/contributor/first-use.rocdown` says it is unpublished and not
in nav. Local `rocdown view site` still warns `RD2202` for that page. Non-draft
pages, listed or not, go into `pages.json`, `sitemap.xml`, and `llms.txt`, so
the page finder and crawlers can still surface "First-use measurement
protocol".[^first-use][^planner][^site-config]

Pick one: mark it `draft`, move it out of the mounted docs tree, or give
unlisted authored pages a discoverability policy that excludes them from
sitemap and the finder.

### 4. Add the GitHub community-health files

**Severity:** P1. Public-preview Phase 0 still requires conduct, contribution,
security, support, and governance documents plus focused issue or discussion
forms. The root has `LICENSE` and `THIRD_PARTY_LICENSES.md` only. There is no
`CODE_OF_CONDUCT.md`, `CONTRIBUTING.md`, `SECURITY.md`, `SUPPORT.md`,
`GOVERNANCE.md`, or `.github/ISSUE_TEMPLATE`. Discussions are
disabled.[^preview-plan][^github-health][^github-repo]

The in-site Contributing page is three short lists and predates the
stack-first docs and `rocci-docs` ownership. It is not a substitute for
GitHub's default community files.[^contributing-page]

`security@rocci.dev` is still recorded as a later mailbox; `oss@rocci.dev`
already forwards.[^publish-plan]

### 5. Stop teaching `@on` as current in first-contact repo files

**Severity:** P1 for trust. Public docs and Project status already use
verb-first `@method:role`. The root README still describes the starting app
as "SQLite, `@on`, and a Datastar patch" and says `rocci run` generates a
dispatcher from `@on`. The root `ROADMAP.md` repeats that spelling. The VS
Code extension README lists `@on` in current completions.[^root-readme][^roadmap][^vscode-readme][^project-status]

A visitor who clones after the site will hit README first. Align those files
with the current language before the repo is public.

## Should before public

Do these unless a maintainer explicitly defers them. They are not as sharp as
clone-404s, but they show up on day one.

- [ ] Run `uv run rocci-ops site` on the revision that will be promoted and
  confirm `check site` has no unexpected `RD2202` once first-use is handled.
- [ ] Walk `/`, `/docs/`, `/docs/install/`, `/docs/five-minutes/`,
  `/docs/the-stack/`, `/docs/applications/standalone/`, `/docs/rocdown/`,
  `/examples/`, `/faq/`, `/project/status/`, and a generated example source
  page on the **staging** hostname, signed out.
- [ ] Confirm `/docs/start/install/`, `/rocdown/`, and `/docs/tutorials/ship/`
  404 (clean cut), while the retargeted `/news/*` 308s land on live pages.
- [ ] Confirm `/news/` and `/news/feed.xml` still 410.
- [ ] Prove one clean clone-and-install from a tagged revision using only
  public Install copy. Record the Roc nightly, Datastar pin, and OS in one
  support matrix (Install plus Project status is enough if they agree).
- [ ] Rewrite the public Contributing page to match AGENTS ownership layers
  and point at the new GitHub `CONTRIBUTING.md`.
- [ ] Decide whether the experimental banner stays Home+Docs-index only.
  Footer copy already repeats the warning on other layouts.[^site-shell]
- [ ] Mark Tree-sitter grammar C as linguist-vendored so GitHub does not
  classify the public repo as C. `.gitattributes` currently only lists image
  LFS rules.[^gitattributes][^github-repo]
- [ ] Sweep root leftovers that will become the GitHub file list:
  `AGENT_SKILLS_PLAN.md`, `ROCCI_LANGUAGE_SERVER_IMPLEMENTATION_PLAN.md`,
  `ROCCI_PLAYGROUND_IMPLEMENTATION_PLAN.md`, `ROC_TEMPLATE.md`,
  `archive/reports/`, and `reports/branding/`. Keep, relocate under
  `knowledge/`/`archive/`, or add a short README that they are historical.
- [ ] Confirm live example hostnames
  (`live-counter.examples.rocci.dev`, `datastar.examples.rocci.dev`) are
  either serving or not linked as if they were. Catalog `hosting = "live"`
  for those two apps; origin Caddy already names the hosts.[^apps-catalog][^example-caddy]
- [ ] Route production `rocci.dev` / `www.rocci.dev` only after staging smoke
  of hybrid Home (`GET /sse`) and the News 308/410 matrix.[^publish-plan]
- [ ] Enable GitHub Discussions or publish a single public feedback URL
  before any Roc/Datastar announcement.[^preview-plan]

## Explicitly not a launch gate

Leave these on their existing plans. They must not delay a labeled
experimental preview.

- Visual identity, logo comparison, and trademark clearance.[^preview-plan]
- Full-text search. The finder is titles and paths only.[^known-limits]
- Windows/Linux installers, notarization, crates.io, Homebrew.[^project-status][^install]
- Reserved `@island` syntax and client-owned islands.[^project-status][^known-limits]
- Human first-use timing sessions (Phase 7 of the documentation plan).
- Tangled as canonical git.[^publish-plan]
- Publishing the OKF knowledge bundle.[^publish-plan]
- A broader accessibility audit beyond the 2026-08-22 chrome matrix already
  recorded in the UX audit.[^ux-audit]

## Suggested order

1. Fix News 308s and the tests that pin them.
2. Unpublish first-use from sitemap/finder (or accept it as public maintainer
   notes and drop the "unpublished" sentence).
3. Align README, ROADMAP, and editor READMEs with verb-first handlers.
4. Add community-health files and a real contributing path.
5. Flip the GitHub repository to public.
6. Smoke staging, then route production DNS.

## Launch-day smoke (after DNS)

From a signed-out browser and `curl -I`:

```text
200  / /docs/ /docs/install/ /docs/five-minutes/ /faq/ /project/status/ /examples/
200  /robots.txt /sitemap.xml /llms.txt
410  /news/ /news/feed.xml
308  /news/introducing-rocci/            -> /docs/the-stack/
308  /news/rocdown-static-collections/   -> /docs/rocdown/sites/
308  /news/rocci-desktop-apps/           -> /docs/applications/package/
404  /docs/start/install/ /rocdown/ /docs/tutorials/ship/
```

Home must still show the five path cards and a working increment on the live
island. GitHub must open without authentication.

[^site-config]: Current lanes, mounts, clone URL, experimental footer, and live-counter island service.
[^landing]: Current home proposition, path cards, hybrid island, and maturity copy.
[^faq]: Current FAQ questions and canonical follow-up links.
[^project-status]: Current shipped inventory and deliberate limits, including verb-first handlers.
[^contributing-page]: Short in-site contributing notes last revised before the stack-first docs cut.
[^install]: Public clone URL, source-build path, and experimental GitHub archives.
[^inventory]: Retired academy and `/rocdown/` routes that must 404; approved stack nav labels.
[^first-use]: Page body states it is unpublished and not the documentation IA.
[^caddy]: Origin 308 targets still use `/docs/start/what-is-rocci/`, `/rocdown/site-config/`, and `/docs/tutorials/ship/`.
[^caddy-test]: Origin tests assert those exact stale redirect strings.
[^ux-contract]: Chrome contract fixture still names academy install/Rocdown/News targets.
[^planner]: Discovery artifacts are built from every non-draft page, with no unlisted filter.
[^site-shell]: Experimental banner is rendered only for `/` and `/docs/`.
[^layouts]: Shared navigated frame for FAQ, section, docs, product, plain, and 404.
[^root-readme]: First-run copy still names `@on` as the standalone dispatcher form.
[^roadmap]: Root roadmap still describes standalone HTTP apps as `@on`.
[^preview-plan]: Phase 0 publication gate: license done; conduct, contribution, security, support, governance, and a clean public install remain.
[^site-plan]: Approved News 308 targets after the stack-first cut: `/docs/the-stack/`, `/docs/rocdown/sites/`, `/docs/applications/package/`.
[^publish-plan]: Staging Access-gated; production hostnames unrouted until a launch decision; `security@rocci.dev` later.
[^ux-audit]: Prior P0/P1 chrome findings closed locally on 2026-08-22; remote CI remains a post-push gate.
[^github-repo]: Authenticated 2026-08-22 metadata: `private: true`, Apache-2.0, issues on, discussions off, GitHub language C.
[^github-health]: GitHub's default community-health file set.
[^gitattributes]: Image LFS only; grammar C sources are not marked vendored.
[^vscode-readme]: Completion list still includes `@on` as a current directive.
[^example-caddy]: Host routing for live-counter and datastar example origins.
[^apps-catalog]: Those two apps are the catalog `hosting = "live"` rows.
[^known-limits]: Finder is not full-text search; `@island` and production packaging remain absent.
