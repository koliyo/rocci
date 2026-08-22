---
type: Audit
title: rocci.dev public-launch checklist
description: After the 2026-08-22 PR landings, rocci.dev is an experimental preview with stack-first docs, a playground lane, and no News 308s; remaining gates are GitHub community-health files, a thin Contributing page, and a signed-out staging smoke. The repository staying private until flip is known.
tags: [domain/rocci, domain/rocdown, concern/publication, concern/community, concern/ux, concern/navigation]
status: draft
generated: { by: process:cursor, at: 2026-08-22T21:40:00Z }
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
  - id: playground
    resource: ../../site/playground/index.rocdown
    title: Current in-browser lower playground
    author: process:git
    last_modified: 2026-08-22
  - id: install
    resource: ../../docs/install.rocdown
    title: Public install and clone instructions
    author: process:git
    last_modified: 2026-08-22
  - id: inventory
    resource: ../../docs/inventory.toml
    title: Stack-first retired-route list and page dispositions
    author: process:git
    last_modified: 2026-08-22
  - id: caddy
    resource: ../../docker/cdn/Caddyfile
    title: Production hybrid Caddy without News 308s
    author: process:git
    last_modified: 2026-08-22
  - id: caddy-test
    resource: ../../tools/rocci-ops/tests/test_example_origins.py
    title: Origin tests for News 410 and no News redirects
    author: process:git
    last_modified: 2026-08-22
  - id: ux-contract
    resource: ../../site/tests/rocci-dev-site-ux-contract.toml
    title: Chrome contract News dispositions
    author: process:codex
    last_modified: 2026-08-22
  - id: site-shell
    resource: ../../site/theme/SiteShell.rocci
    title: Current rocci.dev document shell
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
    title: Root implementation roadmap
    author: human:nils
    last_modified: 2026-08-22
  - id: vscode-readme
    resource: ../../editors/vscode/README.md
    title: VS Code extension README
    author: process:git
    last_modified: 2026-08-22
  - id: preview-plan
    resource: ../plans/public-preview-community.md
    title: Public-preview branding and community plan
    author: process:cursor
    last_modified: 2026-08-21
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
  - id: github-health
    resource: https://docs.github.com/en/communities/setting-up-your-project-for-healthy-contributions/creating-a-default-community-health-file
    title: GitHub community health files
    author: organization:github
  - id: gitattributes
    resource: ../../.gitattributes
    title: Current Git attributes
    author: process:git
    last_modified: 2026-08-17
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

The site is ready to show as a labeled experimental preview once GitHub
community-health files exist and staging has been smoked signed-out. Home,
stack-first Docs, FAQ, Project status, Examples, and a Playground lane are in
place. News 308s are gone because the hostname was never public. First-contact
copy uses verb-first handlers. The repository remaining private until the
maintainer flips it is known and is not an open defect.[^landing][^site-config][^project-status][^caddy][^root-readme][^preview-plan]

This audit does not authorize publication.

## Scope and method

Re-reviewed on 2026-08-22 against `staging` (`9bfe631` plus this revision).
Sources were the current `site/`, mounted `docs/`, origin Caddy, root README,
and ROADMAP after the day's merged PRs. The first-use protocol page is
deleted from `docs/reference/contributor/`. This pass did not rebuild
`dist/rocci.dev` and did not hit production `rocci.dev`.[^publish-plan][^inventory]

## Closed since the first pass

- `docs/reference/contributor/first-use.rocdown` is deleted. Inventory marks
  that path retired. Session notes stay in `docs/first-use-sessions.toml`.[^inventory]
- Origin Caddy has no News `redir` / 308 lines. `/news/` and `/news/feed.xml`
  still 410. Former article URLs fall through to the themed 404. Origin tests
  and the chrome-contract fixture match that. Trailing-slash 308s on authored
  routes are unchanged product behavior, not News compatibility.[^caddy][^caddy-test][^ux-contract]
- Root README, ROADMAP, the VS Code extension README, and the Rocdown CLI
  README no longer teach `@on` as current. Public language and diagnostics
  pages still name `@on` only as a removed form.[^root-readme][^roadmap][^vscode-readme][^project-status]

## What is already launch-shaped

- Global lanes are Docs, Examples, Playground, FAQ, and Project. There is no
  News lane and no top-level `/rocdown/` product lane.[^site-config]
- Home is a short brand line, an experimental caution, a hybrid-island proof,
  then six path cards including the playground.[^landing]
- Docs nav is stack-first: Start, Templates, Applications, Rocdown,
  Reference, Troubleshooting. Install is a source clone and build; GitHub
  archives are experimental.[^site-config][^install]
- FAQ answers stay short and point at canonical owners. Project status lists
  shipped work and deliberate limits, including verb-first `@method:role`.[^faq][^project-status]
- Playground states that the browser lowers to Roc and AST and that HTML
  preview is unavailable without a Roc WASM compiler.[^playground]
- License text is Apache-2.0. Footer copy repeats the experimental
  boundary.[^root-readme][^preview-plan]
- Staging deploy exists behind Cloudflare Access; production hostname
  routing remains a launch decision.[^publish-plan]

## Known, not a finding

`https://github.com/koliyo/rocci` stays private until the maintainer flips
it. Install and the header already name that URL. Do not treat privacy as
unresolved site work.[^site-config][^install][^preview-plan]

## Must before public

### Add the GitHub community-health files

**Severity:** P1. Public-preview Phase 0 still requires conduct, contribution,
security, support, and governance documents plus focused issue or discussion
forms. The root has `LICENSE` and `THIRD_PARTY_LICENSES.md` only. There is no
`CODE_OF_CONDUCT.md`, `CONTRIBUTING.md`, `SECURITY.md`, `SUPPORT.md`,
`GOVERNANCE.md`, or `.github/ISSUE_TEMPLATE`.[^preview-plan][^github-health]

The in-site Contributing page is three short lists and predates
`rocci-docs` ownership. It is not a substitute for GitHub's default community
files.[^contributing-page]

`security@rocci.dev` is still recorded as a later mailbox; `oss@rocci.dev`
already forwards.[^publish-plan]

## Should before public

- [ ] Walk `/`, `/docs/`, `/docs/install/`, `/docs/five-minutes/`,
  `/docs/the-stack/`, `/docs/applications/standalone/`, `/docs/rocdown/`,
  `/examples/`, `/playground/`, `/faq/`, and `/project/status/` on
  **staging**, signed out. Confirm hybrid Home `GET /sse` still increments.
- [ ] Confirm `/docs/start/install/`, `/rocdown/`, and `/docs/tutorials/ship/`
  404. Confirm `/news/` and `/news/feed.xml` 410. Confirm former News article
  paths 404, not 308.
- [ ] Prove one clean clone-and-install from a tagged revision using only
  public Install copy. Record the Roc nightly, Datastar pin, and OS in one
  support matrix.
- [ ] Rewrite the public Contributing page to match AGENTS ownership layers
  and point at the new GitHub `CONTRIBUTING.md`.
- [ ] Mark Tree-sitter grammar C as linguist-vendored so GitHub does not
  classify the public repo as C. `.gitattributes` currently lists image and
  WASM LFS rules only.[^gitattributes]
- [ ] Sweep root leftovers that will become the GitHub file list:
  `AGENT_SKILLS_PLAN.md`, `ROCCI_LANGUAGE_SERVER_IMPLEMENTATION_PLAN.md`,
  `ROCCI_PLAYGROUND_IMPLEMENTATION_PLAN.md`, `ROC_TEMPLATE.md`,
  `archive/reports/`, and `reports/branding/`.
- [ ] Confirm live example hostnames
  (`live-counter.examples.rocci.dev`, `datastar.examples.rocci.dev`) are
  either serving or not linked as if they were.[^apps-catalog][^example-caddy]
- [ ] Enable GitHub Discussions or publish a single public feedback URL
  before any Roc/Datastar announcement.[^preview-plan]
- [ ] Route production `rocci.dev` / `www.rocci.dev` only after that staging
  smoke.[^publish-plan]

## Explicitly not a launch gate

- Visual identity, logo comparison, and trademark clearance.[^preview-plan]
- Full-text search. The finder is titles and paths only.[^known-limits]
- Windows/Linux installers, notarization, crates.io, Homebrew.[^project-status][^install]
- Reserved `@island` syntax and client-owned islands.[^project-status][^known-limits]
- Human first-use timing sessions.
- Tangled as canonical git.[^publish-plan]
- Publishing the OKF knowledge bundle.[^publish-plan]
- A broader accessibility audit beyond the earlier chrome matrix.[^ux-audit][^layouts][^site-shell]

## Suggested order

1. Add community-health files and refresh the Contributing page.
2. Smoke staging signed-out, including playground honesty and News 410/404.
3. Flip the GitHub repository to public.
4. Route production DNS.

## Launch-day smoke (after DNS)

From a signed-out browser and `curl -I`:

```text
200  / /docs/ /docs/install/ /docs/five-minutes/ /faq/ /project/status/ /examples/ /playground/
200  /robots.txt /sitemap.xml /llms.txt
410  /news/ /news/feed.xml
404  /news/introducing-rocci/ /docs/start/install/ /rocdown/ /docs/tutorials/ship/
```

Home must still show the path cards and a working increment on the live
island. GitHub must open without authentication after the known flip.

[^site-config]: Current lanes include Playground; clone URL and live-counter island service are unchanged.
[^landing]: Current home: caution, hybrid island, then six path cards.
[^faq]: Current FAQ questions and canonical follow-up links.
[^project-status]: Current shipped inventory uses verb-first handlers; `@island` is reserved.
[^contributing-page]: Short in-site contributing notes last revised 2026-08-18.
[^playground]: Playground copy states lower-only; no HTML preview without Roc WASM.
[^install]: Public clone URL, source-build path, and experimental GitHub archives.
[^inventory]: First-use path is retired; academy and `/rocdown/` routes stay 404.
[^caddy]: No News `redir` lines; `/news/` and `/news/feed.xml` 410; themed 404 for other misses.
[^caddy-test]: Origin tests forbid `redir ` and keep the 410 pair.
[^ux-contract]: Former News articles are retire/404; index and feed stay 410.
[^site-shell]: Current header, lanes, and skip link.
[^layouts]: Shared navigated frame for non-home layouts.
[^root-readme]: First-run copy names `@method:role` and a Datastar fragment.
[^roadmap]: Root roadmap names `@method:role` for standalone HTTP apps.
[^vscode-readme]: Completion list names current handler forms, not `@on`.
[^preview-plan]: Phase 0: license done; community-health files remain.
[^publish-plan]: Staging Access-gated; production hostnames unrouted until a launch decision.
[^ux-audit]: Prior P0/P1 chrome findings closed locally on 2026-08-22.
[^github-health]: GitHub's default community-health file set.
[^gitattributes]: Image and WASM LFS only; grammar C sources are not marked vendored.
[^example-caddy]: Host routing for live-counter and datastar example origins.
[^apps-catalog]: Those two apps are the catalog `hosting = "live"` rows.
[^known-limits]: Finder is not full-text search; `@island` and production packaging remain absent.
