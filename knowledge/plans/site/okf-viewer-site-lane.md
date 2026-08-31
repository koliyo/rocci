---
type: Implementation Plan
title: Mount the OKF knowledge viewer on rocci.dev
description: Package the existing rocci-okf static review site under /knowledge/ and expose it as a rocci.dev lane so visitors can browse the committed knowledge bundle without turning knowledge Markdown into Rocdown.
tags: [domain/site, domain/okf, domain/rocci-okf, concern/publication, concern/navigation, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-08-26T08:05:00Z }
stale_after: 2026-11-24
authority: exploratory
owners: [human:nils]
sources:
  - id: publication
    resource: ../../decisions/local-knowledge-publication.md
    title: Keep generated knowledge publication local
    author: process:okf-phase-5
    last_modified: 2026-08-16
  - id: static-okf
    resource: ../../decisions/static-okf-boundary.md
    title: Strict OKF Markdown and static rendering boundary
    author: process:okf-migration
    last_modified: 2026-08-17
  - id: product-boundary
    resource: ../../decisions/consolidate-rocdown-product-boundary.md
    title: Approved Rocdown product-boundary decision
    author: process:cursor
    last_modified: 2026-08-17
  - id: system-overview
    resource: ../../architecture/system-overview.md
    title: Current Rocci system overview
    author: process:cursor
    last_modified: 2026-08-18
  - id: site-plan
    resource: rocci-dev-site.md
    title: rocci.dev UX and authoring plan
    author: process:cursor
    last_modified: 2026-08-22
  - id: publish-plan
    resource: rocci-dev-publish.md
    title: Deploy rocci.dev with Cloudflare, a small VPS, and CI
    author: process:cursor
    last_modified: 2026-08-21
  - id: okf-app-plan
    resource: ../okf/rocci-okf-app.md
    title: Standalone Rocci OKF review and query application
    author: process:cursor
    last_modified: 2026-08-17
  - id: cli-plan
    resource: ../shared/cli-entry-points.md
    title: CLI entry points for Rocci, Rocdown, and OKF preview
    author: process:cursor
    last_modified: 2026-08-19
  - id: site-config
    resource: ../../../site/rocdown.toml
    title: Current rocci.dev mounts and global lanes
    author: process:git
    last_modified: 2026-08-24
  - id: site-shell
    resource: ../../../site/theme/SiteShell.rocci
    title: rocci.dev header lanes and page finder
    author: process:git
    last_modified: 2026-08-24
  - id: nav-config
    resource: ../../../crates/rocci-rocdown/src/config.rs
    title: Rocdown NavConfig requires catalog page items
    author: process:git
    last_modified: 2026-08-24
  - id: okf-readme
    resource: ../../../crates/rocci-okf/README.md
    title: rocci-okf review viewer and build contract
    author: process:git
    last_modified: 2026-08-24
  - id: okf-cli
    resource: ../../../crates/rocci-okf/src/main.rs
    title: rocci-okf view, build, and asset routes
    author: process:git
    last_modified: 2026-08-24
  - id: okf-presentation
    resource: ../../../crates/rocci-okf/src/presentation.rs
    title: Root-absolute review HTML, pages.json, and session script
    author: process:git
    last_modified: 2026-08-24
  - id: okf-theme
    resource: ../../../crates/rocci-okf/templates/OkfTheme.rocci
    title: OKF KnowledgeShell chrome
    author: process:git
    last_modified: 2026-08-24
  - id: okf-build-roc
    resource: ../../../crates/rocci-okf/runtime/OkfBuild.roc
    title: Roc knowledge-page HTML wrapper with root-absolute assets
    author: process:git
    last_modified: 2026-08-24
  - id: published-href
    resource: ../../../crates/okf/src/graph.rs
    title: Bundle Markdown to root-absolute review routes
    author: process:git
    last_modified: 2026-08-24
  - id: goto-js
    resource: ../../../crates/rocci-ui/assets/goto.js
    title: Shared Cmd-K palette fetching /pages.json
    author: process:git
    last_modified: 2026-08-24
  - id: ops-package
    resource: ../../../rocci-ops/src/rocci_ops/site.py
    title: rocci-ops package site pipeline
    author: process:git
    last_modified: 2026-08-24
  - id: cdn-caddy
    resource: ../../../docker/cdn/Caddyfile
    title: Origin static file_server and try_files
    author: process:git
    last_modified: 2026-08-24
  - id: knowledge-ci
    resource: ../../../.github/workflows/knowledge.yml
    title: Knowledge validation workflow without artifact publish
    author: process:git
    last_modified: 2026-08-24
  - id: ux-contract
    resource: ../../../site/tests/rocci-dev-site-ux-contract.toml
    title: Approved rocci.dev lane and chrome contract
    author: process:git
    last_modified: 2026-08-22
  - id: okmate
    resource: ../okf/okmate.md
    title: Okmate — extractable Rust OKF mate
    author: process:cursor
    last_modified: 2026-08-26
  - id: rust-datastar
    resource: ../okf/okf-viewer-rust-datastar.md
    title: In-place rocci-okf Askama rewrite (superseded as vehicle)
    author: process:cursor
    last_modified: 2026-08-26
  - id: rust-vs-rocci
    resource: ../../research/okf/okf-viewer-rust-vs-rocci.md
    title: OKF viewer Rust HTML versus Rocci shell
    author: process:cursor
    last_modified: 2026-08-26
---

# Mount the OKF knowledge viewer on rocci.dev

## Goal

Let visitors of rocci.dev browse the committed `knowledge/` bundle through the
existing OKF review viewer, reached from a new global site lane, without
re-authoring records as Rocdown or giving Rocdown a catalog of those
pages.[^okf-readme][^static-okf][^site-config]

## Recommended shape

Treat the viewer as a **foreign static app** copied into the site package,
not as a Rocdown `[[mount]]`.

| Piece | Owner | Public location |
| --- | --- | --- |
| Canonical records | `knowledge/**/*.md` | GitHub / the repo, unchanged |
| Generated HTML, `pages.json`, `catalog.json` | `rocci-okf build` | `/knowledge/` on rocci.dev |
| Global lane | `site/rocdown.toml` + `SiteShell` | Header link to `/knowledge/` |
| Knowledge chrome | `okmate build` once [okmate](/plans/okf/okmate.md) lands; today `OkfTheme` / `presentation.rs` | Dashboard, collections, review queue |
| Site chrome on knowledge pages | Thin lane strip in the OKF shell | Docs, Examples, Playground, FAQ, Project, Knowledge |

The published tree is the same static review site `rocci-okf build` already
writes to `dist/knowledge`, rebuilt with a URL prefix and without preview-only
scripts.[^okf-cli][^okf-presentation]

Default names, pending the Phase 0 gate:

- **Lane label:** Knowledge. "Development" is accepted if the maintainer wants
  contributor framing; do not use it as the URL prefix.
- **Canonical entry:** `/knowledge/` (the OKF dashboard).
- **Collections:** `/knowledge/architecture/`, `/knowledge/plans/okf/…`, and
  the rest of today's review routes under that prefix.

Project stays the product/status/community lane. This lane is the working
memory (architecture, decisions, plans, research, audits). Do not move
`docs/reference/contributor` or `/project/contributing/` into it.[^site-plan]

## Why not a Rocdown mount

`site/rocdown.toml` already mounts `docs/` and generated example docs as
Rocdown catalogs with `[[nav]]` items that must resolve to pages.[^site-config][^nav-config]
Knowledge records are inert OKF Markdown. Rocdown must not depend on OKF, and
canonical records must not grow Rocdown declarations or generated Roc
modules.[^product-boundary][^static-okf][^system-overview]

A `[[mount]]` of `knowledge/` would either fail catalog checks or force a
second Markdown pipeline through Rocdown. Copying the already-generated OKF
HTML tree avoids both.

The copy does not depend on Rocci apply. A Rust-only
`html_page_for` tree is the same artifact: prefix `/knowledge/`, strip
preview scripts, add a lane strip in the OKF document. Interactive
settings or later Datastar operations do not ride this `file_server` tree;
they stay on local `rocci-okf view` or a separately reviewed live
origin. Reasoning:
[viewer rust vs rocci](/research/okf/okf-viewer-rust-vs-rocci.md);
product path [okmate](/plans/okf/okmate.md), not the in-place
[rust+datastar](/plans/okf/okf-viewer-rust-datastar.md)
rewrite.[^rust-vs-rocci][^okmate][^rust-datastar]

## Why a prefix, not the site root or a subdomain

The viewer is a complete site at `/`. Dashboard, review, and every concept
route are root-absolute (`/review/`, `/architecture/`, `/__rocci_okf/app.css`).
Cmd-K loads `/pages.json` and `/catalog.json`.[^okf-presentation][^goto-js][^published-href]
rocci.dev already owns `/`, `/pages.json`, `/llms.txt`, `/sitemap.xml`, and
the product lanes.[^site-config]

| Option | Verdict |
| --- | --- |
| Serve OKF at `/` | Destroys Home and collides with site `pages.json`. |
| Subdomain `knowledge.rocci.dev` | Clean isolation, but not a site lane; extra DNS/TLS. Out of bound here. Follow-on if the prefix proves painful. |
| iframe inside a Rocdown page | Breaks deep links, Cmd-K, and history. Rejected. |
| Prefix `/knowledge/` on the same origin | One package, one Caddy `file_server`, lane can point at it. **Recommended.** |

Caddy already does `try_files {path} {path}/index.html` on the site dist
root, so a copied `knowledge/` directory is enough at the origin once URLs
inside that tree are prefixed.[^cdn-caddy]

## Chrome join

Do not wrap every knowledge page in `SiteShell`. That would require OKF pages
to be Rocdown `PageView`s and would run two sidebars plus two Cmd-K catalogs
in one document.[^site-shell][^okf-theme]

Do:

1. Keep OKF collection nav, dashboard, and review queue as the in-app chrome.
2. Add a **site lane strip** to the OKF document (same labels and hrefs as
   `SiteShell`) so Knowledge is `aria-current` while the visitor is under
   `/knowledge/`.
3. Keep Cmd-K scoped: site pages search site `pages.json`; knowledge pages
   search `/knowledge/pages.json`. Merging catalogs is a follow-on.

Local `rocci-okf view` stays unprefixed at `/` and does not need the site
lane strip.

## Public viewer profile

The review viewer is the right public surface for this lane. Lifecycle,
sources, trust, and the review queue are the point of a knowledge base, not
a defect to hide for marketing.[^okf-readme][^okf-app-plan]

Strip preview-only behavior from the packaged tree:

- Omit `reload.js` (live preview).
- Omit `session.js` POSTs to `/__rocci_okf/session` (desktop last-route).
- Build with `--profile base`, matching Knowledge CI `check`.
  Untracked local research never appears; the GitHub checkout is the snapshot.

Keep `catalog.json` (Cmd-K). Ship `llms.txt` at `/knowledge/llms.txt`. Do not
advertise a downloadable verbatim bundle archive.[^publication]

Dark One Dark Pro versus the site's light/dark chrome is accepted for v1.
Theming alignment is out of bound.

## Publication gate

The local-first decision already says a future public site needs a separately
reviewed change: audience, access, source/license inventory, copy-versus-link,
and an explicit deploy path.[^publication] Site publish currently lists a
public knowledge deploy as something the agent must not invent.[^publish-plan]
Knowledge CI validates and discards artifacts; it does not upload them.[^knowledge-ci]

This plan is that reviewed change for **generated HTML of the committed
bundle only**. It does not approve a tarball of `knowledge/` plus every
linked repository file.

Phase 0 must amend [local knowledge publication](/decisions/local-knowledge-publication.md)
to: public HTML under `/knowledge/` on rocci.dev is allowed; a verbatim
bundle archive is still forbidden. Until that amendment, this record stays
exploratory and no later phase starts.

## Out of bound

- Re-authoring `knowledge/**/*.md` as `.rocdown` or mounting it in the
  Rocdown catalog.
- A `knowledge.rocci.dev` (or similar) hostname.
- Authenticated query, MCP, or the hosted review-decision service from the
  OKF application plan.[^okf-app-plan]
- Merging site and knowledge Cmd-K indexes.
- Full-text search beyond the existing OKF palette.
- Publishing `archive/`, untracked research, or a zip of the bundle.
- Moving contributor docs or Project pages into this lane.
- Visual-identity work or forcing OKF onto the site light/dark tokens.
- Changing local `rocci-okf view` into a site-prefixed server by default.
- Executing any phase as part of writing this plan.

## Constraints that do not move

1. Canonical records stay inert OKF Markdown. Presentation stays in
   `rocci-okf`.[^static-okf][^cli-plan]
2. Rocdown does not depend on OKF. `okf` does not depend on Rocdown or
   Rocci. Prefixing happens in `rocci-okf` when emitting HTML and indexes;
   `okf::published_href` may stay bundle-root.[^product-boundary][^published-href]
3. Failed knowledge builds fail `package site`. Do not copy a stale
   `dist/knowledge` over a successful docs tree.
4. Navigation works without JavaScript. Cmd-K remains an enhancement.[^site-plan][^goto-js]
5. `rocci-ops package site` remains the only public deploy artifact path;
   the Knowledge workflow stays validation-only.[^ops-package][^knowledge-ci]
6. Public behavior claims stay verified against code, tests, or current
   package docs.

## Delivery phases

Each phase is one mergeable change. Start only when asked.

### Phase 0 — Publication and naming gate

**Bound:** answers recorded on this plan and an amendment to the local
publication decision. No viewer or site packaging code.

**Does:**

- Confirm audience: signed-out rocci.dev visitors, same access as the rest
  of the site (no extra auth).
- Confirm published set: generated HTML plus `pages.json`, `catalog.json`,
  `llms.txt`, and `validation.json` of the committed bundle. Not a source
  archive.
- Confirm lane label (Knowledge vs Development) and URL prefix
  (`/knowledge/`).
- Confirm the review queue stays public.
- Confirm knowledge URLs are listed in a sitemap (site sitemap append or
  `/knowledge/sitemap.xml` linked from robots).
- Amend [local knowledge publication](/decisions/local-knowledge-publication.md)
  so generated HTML on rocci.dev is allowed and a verbatim archive is still
  not.

**Exit:** the decision record states the public-HTML exception; this plan
records the four naming/visibility answers; `cargo run -q -p rocci-okf --
check knowledge --profile base --format terminal` is clean for those
knowledge edits.

### Phase 1 — Prefix-aware static export

**Bound:** `rocci-okf` build/presentation, `OkfBuild.roc`, and focused
tests. No site catalog or Caddy changes.

**Does:**

- Add a build flag such as `--base-path /knowledge` (empty default preserves
  today's local viewer).
- Prefix every emitted route: concept links, nav, breadcrumbs, dashboard,
  review, `pages.json` `route` values, and static assets
  (`/knowledge/__rocci_okf/…`).
- Make `goto.js` read a document base (for example `data-rocci-goto-base` on
  `<html>`) instead of hard-coded `/pages.json`.
- Add a public/static mode that omits `reload.js` and `session.js`.
- Keep `rocci-okf view` on the empty prefix.

**Exit:** `cargo test -p rocci-okf` and `cargo test -p rocci-ui`; a fixture
build with `--base-path /knowledge` contains no `href="/architecture/"` or
`src="/__rocci_okf/` at site root; `cargo fmt --all -- --check`.

### Phase 2 — Foreign site lane

**Bound:** Rocdown nav config, rocci.dev theme, and the UX contract fixture.
No knowledge HTML copy yet.

**Does:**

- Extend `NavConfig` with an optional foreign `href` (or equivalent) so a
  lane can exist without catalog `items`.
- Add the Knowledge (or Development) lane in `site/rocdown.toml` pointing at
  `/knowledge/`.
- Keep `lane.current` false on ordinary site pages; the OKF strip in Phase 3
  marks it current under the prefix.
- Update `site/tests/rocci-dev-site-ux-contract.toml` and any README that
  lists global lanes.
- Optional one-page Rocdown intro is **not** required if the lane target is
  the dashboard. If copy is needed, a short `/project/` or home sentence is
  enough; do not create a second knowledge IA.

**Exit:** `cargo test -p rocci-rocdown`; `cargo run -q -p rocci-rocdown-cli
-- build docs` still succeeds; built Home/header HTML includes the new lane
href; `cargo fmt --all -- --check`.

### Phase 3 — Package, chrome strip, and serve

**Bound:** `rocci-ops package site` / `build_site`, OKF site-lane strip,
origin tarball contents. No Cloudflare or DNS work.

**Does:**

- After the Rocdown site package writes `dist/rocci.dev`, run
  `rocci-okf build knowledge -o dist/knowledge --profile base --base-path
  /knowledge` with public/static mode, then copy into
  `dist/rocci.dev/knowledge/`.
- Fail the package if that build fails.
- Inject the site lane list into the OKF shell so visitors can leave the
  knowledge app. Pass the list from packaging (do not hard-code it in two
  crates).
- Include the knowledge files in `site.tgz` / `publish.json` file lists.
- Append knowledge URLs to the site sitemap or emit `/knowledge/sitemap.xml`
  and mention it from `robots.txt`, matching the Phase 0 sitemap choice.
- Document that `rocdown run site` is catalog-only; the joined preview is
  `rocci-ops` build/package (or a documented two-step). Do not add an OKF
  dependency to `rocci-rocdown-cli`.

**Exit:** `uv run rocci-ops package site --target x64musl` (or the existing
package test substitute) produces `dist/rocci.dev/knowledge/index.html` whose
dashboard links stay under `/knowledge/`; `site.tgz` lists those paths;
`cargo test -p rocci-okf`; `cargo fmt --all -- --check`.

### Phase 4 — Honesty and smoke

**Bound:** public copy, home/project mention, staging smoke. No IA rewrite.

**Does:**

- One sentence on Home and/or Project: this lane is working memory, not the
  product manual; docs stay at `/docs/`.
- Keep the site experimental footer; do not present draft plans as shipped
  product behavior.
- Staging smoke: open `/knowledge/`, follow a collection, open a concept,
  use the review queue, return via a site lane, confirm Cmd-K on a docs page
  does not list only knowledge routes and Cmd-K on a knowledge page does not
  list only docs routes.

**Exit:** the copy is in the built site; a staging (or local packaged)
walkthrough of the smoke list is recorded on this plan or in the knowledge
log. Do not log the plan complete until CI and Knowledge workflows succeed
on that revision.

## Follow-ons (not this plan)

- Subdomain if the prefix fights caching or Cmd-K.
- Unified site+knowledge page finder.
- Shared light/dark tokens with `SiteShell`.
- `llms.txt` at the site root that points at `/knowledge/llms.txt`.
- Hosted review decisions from the OKF application plan.

## Phase 0 answers (blank until the gate)

| Question | Answer |
| --- | --- |
| Lane label | |
| URL prefix | `/knowledge/` recommended |
| Review queue public | yes, recommended |
| Sitemap | include knowledge URLs, recommended |

[^publication]: Local HTML and CI verification only; a public site needs an explicit later change. Verbatim archives stay out of scope.
[^static-okf]: Canonical records are OKF Markdown; `rocci-okf` presents them.
[^product-boundary]: Rocdown must not depend on OKF; knowledge stays a separate product.
[^system-overview]: Knowledge is inert Markdown managed by `okf` and `rocci-okf`.
[^site-plan]: Global lanes today are Docs, Examples, Playground, FAQ, and Project.
[^publish-plan]: Public knowledge deploy is listed as something not to invent during site hosting work.
[^okf-app-plan]: Explorer, review, and authenticated query are application concerns, not Rocdown catalog pages.
[^cli-plan]: `rocci-okf` remains the OKF viewer CLI.
[^site-config]: Mounts and `[[nav]]` lanes are Rocdown catalog entries.
[^site-shell]: Header lanes come from `view.lanes`.
[^nav-config]: `NavConfig` is label plus page `items` or nested groups; no foreign href today.
[^okf-readme]: Review site routes, Cmd-K, and `build -o dist/knowledge`.
[^okf-cli]: `build` writes the static review site; `__rocci_okf` is the asset prefix.
[^okf-presentation]: Root-absolute hrefs, `/pages.json`, session POST, Home at `/`.
[^okf-theme]: KnowledgeShell owns OKF nav, main, and outline slots.
[^okf-build-roc]: Roc wrapper hard-codes `/__rocci_okf/` asset URLs.
[^published-href]: Bundle Markdown becomes `/decisions/foo/`-style routes.
[^goto-js]: Shared palette fetches `/pages.json` then `/catalog.json`.
[^ops-package]: `package site` builds playground, live apps, then `rocdown package site`.
[^cdn-caddy]: Origin `file_server` plus `try_files` on `/src/site/dist`.
[^knowledge-ci]: Knowledge job validates; it does not retain or publish HTML.
[^ux-contract]: Phase 0 site UX evidence lists current lanes and chrome.
[^rust-vs-rocci]: Rust-only HTML is the same static artifact class; Datastar ops are a live host, not this file_server tree.
[^okmate]: `okmate build` is the same artifact class this lane copies; live settings stay off `file_server`.
[^rust-datastar]: Superseded in-place vehicle.
