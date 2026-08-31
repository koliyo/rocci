---
type: Audit
title: rocci.dev public-launch checklist
description: After the 2026-08-23 Should pass, live example hostnames are not advertised as serving, a public support matrix and one GitHub-issues feedback URL are published; remaining gates are a signed-out staging smoke, a tagged clean install, the known repository-visibility flip, and production DNS.
tags: [domain/rocci, domain/rocdown, concern/publication, concern/community, concern/ux, concern/navigation]
status: draft
generated: { by: process:cursor, at: 2026-08-31T08:15:00Z }
stale_after: 2026-11-22
authority: descriptive
owners: [human:nils]
sources:
  - id: site-config
    resource: ../../../site/rocdown.toml
    title: Current rocci.dev catalog, mounts, and navigation
    author: human:nils
    last_modified: 2026-08-22
  - id: landing
    resource: ../../../site/index.rocdown
    title: Current rocci.dev landing page
    author: human:nils
    last_modified: 2026-08-22
  - id: faq
    resource: ../../../site/faq/index.rocdown
    title: Current rocci.dev FAQ
    author: process:cursor
    last_modified: 2026-08-23
  - id: project-status
    resource: ../../../site/project/status.rocdown
    title: Current public project status page
    author: process:cursor
    last_modified: 2026-08-23
  - id: contributing-page
    resource: ../../../site/project/contributing.rocdown
    title: Current public contributing page
    author: process:cursor
    last_modified: 2026-08-31
  - id: contributing-md
    resource: ../../../CONTRIBUTING.md
    title: Root contributor contract
    author: process:cursor
    last_modified: 2026-08-23
  - id: playground
    resource: ../../../site/playground/index.rocdown
    title: Current in-browser lower playground
    author: process:git
    last_modified: 2026-08-22
  - id: install
    resource: ../../../docs/install.rocdown
    title: Public install and clone instructions
    author: process:cursor
    last_modified: 2026-08-23
  - id: inventory
    resource: ../../../docs/inventory.toml
    title: Stack-first retired-route list and page dispositions
    author: process:git
    last_modified: 2026-08-22
  - id: caddy
    resource: ../../../docker/cdn/Caddyfile
    title: Production hybrid Caddy without News 308s
    author: process:git
    last_modified: 2026-08-22
  - id: caddy-test
    resource: ../../../rocci-ops/tests/test_example_origins.py
    title: Origin tests for News 410 and no News redirects
    author: process:git
    last_modified: 2026-08-22
  - id: ux-contract
    resource: ../../../site/tests/rocci-dev-site-ux-contract.toml
    title: Chrome contract News dispositions
    author: process:codex
    last_modified: 2026-08-22
  - id: site-shell
    resource: ../../../site/theme/SiteShell.rocci
    title: Current rocci.dev document shell
    author: process:git
    last_modified: 2026-08-22
  - id: layouts
    resource: ../../../site/theme/Layouts.rocci
    title: Current navigated frame used by non-home layouts
    author: process:git
    last_modified: 2026-08-22
  - id: root-readme
    resource: ../../../README.md
    title: Current workspace overview and first-run copy
    author: human:nils
    last_modified: 2026-08-22
  - id: vscode-readme
    resource: ../../../editors/vscode/README.md
    title: VS Code extension README
    author: process:git
    last_modified: 2026-08-22
  - id: preview-plan
    resource: ../../plans/site/public-preview-community.md
    title: Public-preview branding and community plan
    author: process:cursor
    last_modified: 2026-08-21
  - id: publish-plan
    resource: ../../plans/site/rocci-dev-publish.md
    title: rocci.dev Cloudflare and VPS deploy plan
    author: process:cursor
    last_modified: 2026-08-21
  - id: ux-audit
    resource: ../rocci-dev-site-ux-dx.md
    title: Prior rocci.dev UX and authoring DX review
    author: process:cursor
    last_modified: 2026-08-22
  - id: github-health
    resource: https://docs.github.com/en/communities/setting-up-your-project-for-healthy-contributions/creating-a-default-community-health-file
    title: GitHub community health files
    author: organization:github
  - id: gitattributes
    resource: ../../../.gitattributes
    title: Current Git attributes
    author: process:git
    last_modified: 2026-08-23
  - id: example-caddy
    resource: ../../../docker/examples/Caddyfile
    title: Planned live example hostnames
    author: process:git
    last_modified: 2026-08-21
  - id: apps-catalog
    resource: ../../../examples/rocci/apps.toml
    title: Cataloged example apps and live hosting flags
    author: process:git
    last_modified: 2026-08-22
  - id: known-limits
    resource: ../../status/known-limitations.md
    title: Canonical known-limitations snapshot
    author: process:cursor
    last_modified: 2026-08-22
  - id: compatibility
    resource: ../../../docs/reference/compatibility.rocdown
    title: Public support matrix
    author: process:cursor
    last_modified: 2026-08-23
  - id: support-md
    resource: ../../../SUPPORT.md
    title: Root support and public feedback URL
    author: process:cursor
    last_modified: 2026-08-23
  - id: operator-plan
    resource: ../../plans/site/public-launch-operator.md
    title: Maintainer flip, promote, and DNS sequence
    author: process:cursor
    last_modified: 2026-08-23
---

# rocci.dev public-launch checklist

## Executive verdict

The site is ready to show as a labeled experimental preview once staging has
been smoked signed-out. Home, stack-first Docs, FAQ, Project status, Examples,
and a Playground lane are in place. News 308s are gone because the hostname
was never public. First-contact copy uses verb-first handlers. GitHub
community-health files, a matching Contributing page, linguist-vendored
Tree-sitter C, and an archived leftover sweep are in the tree. The 2026-08-23
Should pass published a support matrix, one GitHub-issues feedback URL, and
stopped advertising reserved example hostnames as live. The repository
remaining private until the maintainer flips it is known and is not an open
defect.[^landing][^site-config][^project-status][^caddy][^root-readme][^preview-plan][^contributing-md][^compatibility][^support-md]

This audit does not authorize publication.

## Scope and method

Re-reviewed on 2026-08-22 against `staging` (`9bfe631` plus that revision),
then updated on 2026-08-23 for the public-repo surface sweep and again for
the Should pass (example-host probes, Access-gated staging, published
support matrix and feedback URL). Sources were the current `site/`, mounted
`docs/`, origin Caddy, root README, and community-health files. The
first-use protocol page is deleted from `docs/reference/contributor/`. This
pass did not rebuild `dist/rocci.dev` and did not complete a signed-out
staging browser walk.[^publish-plan][^inventory][^contributing-md]

## Closed since the first pass

- `docs/reference/contributor/first-use.rocdown` is deleted. Inventory marks
  that path retired. Session notes stay in `docs/first-use-sessions.toml`.[^inventory]
- Origin Caddy has no News `redir` / 308 lines. `/news/` and `/news/feed.xml`
  still 410. Former article URLs fall through to the themed 404. Origin tests
  and the chrome-contract fixture match that. Trailing-slash 308s on authored
  routes are unchanged product behavior, not News compatibility.[^caddy][^caddy-test][^ux-contract]
- Root README, the VS Code extension README, and the Rocdown CLI
  README no longer teach `@on` as current. Public language and diagnostics
  pages still name `@on` only as a removed form.[^root-readme][^vscode-readme][^project-status]
- Root community-health files exist: `CONTRIBUTING.md`,
  `SECURITY.md`, `SUPPORT.md`, `GOVERNANCE.md`, and focused
  `.github/ISSUE_TEMPLATE` forms. Vulnerability mail is
  `oss@rocci.dev`; `security@rocci.dev` is still a later mailbox.[^contributing-md][^github-health][^publish-plan]
- The in-site Contributing page matches AGENTS ownership layers and points at
  root `CONTRIBUTING.md`.[^contributing-page][^contributing-md]
- Tree-sitter grammars under `crates/rocci-highlight/grammars/` are
  `linguist-vendored`.[^gitattributes]
- Root leftover plans and the later `archive/reports/` tree are gone.
  `knowledge/plans/site/rocci-playground.md` is the knowledge-facing playground
  plan. `README.md`, `LICENSE`, `AGENTS.md`, and `DESIGN.md`
  stay at the root.
- Dedicated example hostnames
  (`live-counter.examples.rocci.dev`, `datastar.examples.rocci.dev`) fail
  TLS handshake; staging example hosts do not resolve. Public example pages
  and READMEs say those names are reserved and not serving. The generated
  `/examples/` table labels catalog `hosting = "live"` as `planned live` and
  does not emit those URLs.[^apps-catalog][^example-caddy]
- The single public feedback URL is
  `https://github.com/koliyo/rocci/issues`. FAQ, SUPPORT, Contributing, and
  Project status point there. Discussions stay a later announcement
  gate.[^support-md][^faq][^preview-plan]
- The public Compatibility page is the support matrix (Roc nightlies,
  Datastar `1.0.2`, OS, editors, packaging, limits). Install links
  it.[^compatibility][^install]

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

Community-health files from public-preview Phase 0 are in the tree. Remaining
musts are operational: signed-out staging smoke, then the known repository
visibility flip, then production DNS.[^contributing-md][^github-health][^publish-plan]

`security@rocci.dev` is still a later mailbox; `oss@rocci.dev` already
forwards and is the listed contact.[^publish-plan]

## Should before public

- [ ] Walk `/`, `/docs/`, `/docs/install/`, `/docs/five-minutes/`,
  `/docs/the-stack/`, `/docs/applications/standalone/`, `/docs/rocdown/`,
  `/examples/`, `/playground/`, `/faq/`, and `/project/status/` on
  **staging**, signed out. Confirm hybrid Home `GET /sse` still increments.
  Signed-out `https://staging.rocci.dev/` still 302s to Cloudflare Access;
  this walk needs a maintainer session or a temporary Access bypass.
- [ ] Confirm `/docs/start/install/`, `/rocdown/`, and `/docs/tutorials/ship/`
  404. Confirm `/news/` and `/news/feed.xml` 410. Confirm former News article
  paths 404, not 308. Origin Caddy already encodes the 410 pair and has no
  News `redir` lines; the signed-out staging walk is still required.[^caddy][^caddy-test]
- [ ] Prove one clean clone-and-install from a tagged revision using only
  public Install copy. The only git tag is `dev` (`6d00e60`), which is not
  current. The support matrix is published; a launch tag plus a clean clone
  remains.[^compatibility][^install][^preview-plan]
- [x] Rewrite the public Contributing page to match AGENTS ownership layers
  and point at root `CONTRIBUTING.md`.[^contributing-page][^contributing-md]
- [x] Mark Tree-sitter grammar C as linguist-vendored so GitHub does not
  classify the public repo as C.[^gitattributes]
- [x] Sweep root leftovers that would dominate the GitHub file list. Dated
  plans and reports are gone from the tree.
- [x] Confirm live example hostnames
  (`live-counter.examples.rocci.dev`, `datastar.examples.rocci.dev`) are
  either serving or not linked as if they were. They are not serving; public
  copy no longer treats them as live demos.[^apps-catalog][^example-caddy]
- [x] Enable GitHub Discussions or publish a single public feedback URL
  before any Roc/Datastar announcement. Discussions stay later; the URL is
  `https://github.com/koliyo/rocci/issues`.[^preview-plan][^support-md][^faq]
- [ ] Route production `rocci.dev` / `www.rocci.dev` only after that staging
  smoke. Apex and `www` currently 502 through Cloudflare; do not treat that
  as a live site.[^publish-plan]

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

Operator sequence: [public-launch operator plan](/plans/site/public-launch-operator.md).[^operator-plan]

1. Promote current `main` onto `staging` (`uv run rocci-ops promote staging`).
   `origin/staging` was four commits behind `main` on 2026-08-23.
2. Smoke staging signed-out, including playground honesty and News 410/404.
3. Flip the GitHub repository to public. Enable Dependabot alerts and a
   `main` / `staging` / `production` ruleset after the flip.
4. After that smoke, `uv run rocci-ops promote production` creates
   `origin/production` from smoked `staging` and runs hosted CI plus site
   deploy. Then route production DNS.

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
[^contributing-page]: In-site Contributing page lists AGENTS layers and links to root CONTRIBUTING.md.
[^contributing-md]: Root CONTRIBUTING.md, SECURITY.md, SUPPORT.md, GOVERNANCE.md, and focused issue forms.
[^playground]: Playground copy states lower-only; no HTML preview without Roc WASM.
[^install]: Public clone URL, source-build path, and experimental GitHub archives.
[^inventory]: First-use path is retired; academy and `/rocdown/` routes stay 404.
[^caddy]: No News `redir` lines; `/news/` and `/news/feed.xml` 410; themed 404 for other misses.
[^caddy-test]: Origin tests forbid `redir ` and keep the 410 pair.
[^ux-contract]: Former News articles are retire/404; index and feed stay 410.
[^site-shell]: Current header, lanes, and skip link.
[^layouts]: Shared navigated frame for non-home layouts.
[^root-readme]: First-run copy names `@method:role` and a Datastar fragment.
[^vscode-readme]: Completion list names current handler forms, not `@on`.
[^preview-plan]: Phase 0 license and community-health files are in tree; the support matrix is published; a clean tagged install and Discussions remain later.
[^publish-plan]: Staging Access-gated; production hostnames unrouted until a launch decision. Apex and www 502 on 2026-08-23.
[^ux-audit]: Prior P0/P1 chrome findings closed locally on 2026-08-22.
[^github-health]: GitHub's default community-health file set.
[^gitattributes]: LFS for images and WASM; `crates/rocci-highlight/grammars/**` is linguist-vendored.
[^example-caddy]: Host routing for live-counter and datastar example origins; those names are not serving.
[^apps-catalog]: Those two apps remain catalog `hosting = "live"` rows for a future origin; public copy says planned.
[^known-limits]: Finder is not full-text search; `@island` and production packaging remain absent.
[^compatibility]: Support matrix lists documented Roc 2026-08-10, maintainer macOS 2026-08-18, Datastar 1.0.2, OS, editors, packaging.
[^support-md]: SUPPORT.md names https://github.com/koliyo/rocci/issues as the single public feedback URL.
[^operator-plan]: Maintainer sequence: promote staging, smoke, flip, ruleset, promote production, then DNS.
