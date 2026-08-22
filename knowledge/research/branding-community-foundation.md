---
type: Research Report
title: Rocci branding and community foundation
description: Exploratory naming, brand architecture, searchability, visual identity, and public-preview community research for Rocci.
tags: [domain/rocci, domain/rocdown, domain/rocs, domain/design-system, concern/branding, concern/community, concern/publication]
status: draft
generated: { by: process:codex, at: 2026-08-18T12:04:47Z }
stale_after: 2026-10-01
authority: exploratory
owners: [human:nils]
sources:
  - id: branding-report
    resource: ../../archive/reports/branding/BRANDING_AND_COMMUNITY_REPORT.rocdown
    title: Rocci branding and community foundation report
    author: process:codex
    last_modified: 2026-08-18
  - id: root-readme
    resource: ../../README.md
    title: Rocci workspace overview
    author: process:git
    last_modified: 2026-08-17
  - id: site-config
    resource: ../../docs/rocdown.toml
    title: Rocdown documentation site configuration
    author: process:git
    last_modified: 2026-08-18
  - id: site-theme
    resource: ../../crates/rocci-rocdown/templates/RocdownTheme.rocci
    title: Rocdown documentation shell
    author: process:git
    last_modified: 2026-08-17
  - id: paper-theme
    resource: ../../crates/rocci-theme/src/themes/paper.css
    title: Default Rocdown Paper theme
    author: process:git
    last_modified: 2026-08-16
  - id: rocci-theme
    resource: ../../crates/rocci-theme/src/themes/rocci.css
    title: Branded Rocdown Rocci theme
    author: process:git
    last_modified: 2026-08-16
  - id: roc-faq
    resource: https://www.roc-lang.org/faq
    title: Roc FAQ and logo origin
    author: organization:roc-programming-language-foundation
  - id: roc-community
    resource: https://roc-lang.org/community
    title: Roc community
    author: organization:roc-programming-language-foundation
  - id: datastar-community
    resource: https://data-star.dev/star_federation
    title: Star Federation community and purpose
    author: organization:star-federation
  - id: google-site-names
    resource: https://developers.google.com/search/docs/appearance/site-names
    title: Google Search guidance for site names
    author: organization:google
    last_modified: 2025-12-10
  - id: github-health
    resource: https://docs.github.com/en/communities/setting-up-your-project-for-healthy-contributions/creating-a-default-community-health-file
    title: GitHub community health files
    author: organization:github
  - id: pypi-rocci
    resource: https://pypi.org/project/rocci/
    title: Existing rocci Python package
    author: human:ndelaneybusch
    last_modified: 2026-07-07
  - id: rocci-cloud
    resource: https://indigo-dc.gitbook.io/rocci
    title: rOCCI cloud interoperability project
    author: organization:indigo-data-cloud
  - id: community-launch-kit
    resource: ../../archive/reports/branding/COMMUNITY_LAUNCH_KIT.md
    title: Rocci public-preview community launch kit
    author: process:codex
    last_modified: 2026-08-18
  - id: landing-prototype
    resource: ../../archive/reports/branding/LANDING_PAGE_PROTOTYPE.md
    title: Rocci landing-page prototype brief
    author: process:codex
    last_modified: 2026-08-17
---

# Rocci branding and community foundation

## Scope and authority

This record is exploratory research for a public preview. It does not approve a
permanent project name, create a legal foundation, authorize Roc-logo use, or
provide trademark clearance. The detailed evidence, alternatives, generated
visual probes, and implementation backlog live in the branding report.[^branding-report]

## Existing product hierarchy

The implementation already makes Rocci the broadest identity: it names the
workspace, command, configuration, template format, runtime, repository, and
documentation site. Rocdown is a Markdown-first sibling source format. Rocs is
the static documentation compiler, whose visible shell is authored in Rocci and
whose catalog and article work remain in Rust.[^root-readme][^site-config][^site-theme]

The recommended public-preview hierarchy is therefore a masterbrand:

- **Rocci** is the project and toolchain.
- **Rocci templates** describes `.rocci` source.
- **Rocdown** is “Rocci's Markdown-first document format.”
- **Rocci Docs** is the public product name for the existing Rocs engine and
  `rocs` command.
- Roc and Datastar are ecosystem relationships, not Rocci subbrands.[^branding-report]

This structure avoids a costly implementation rename while reducing the public
impression that Rocci, Rocdown, and Rocs are three peer projects.

## Naming findings

Rocci is short, has an acquired `.dev` domain, and unifies the command, config,
and `.rocci` syntax. Its weaknesses are unclear pronunciation, dependence on a
descriptor for product meaning, and incomplete search ownership. An active
Python package already uses `rocci`, and older rOCCI cloud tooling is indexed
under a visually similar name.[^pypi-rocci][^rocci-cloud]

The exploratory recommendation is to retain Rocci through a reversible preview,
pair first mentions with **“Composable authoring for applications and
content,”** and ask the community explicitly about pronunciation, recall, and
perceived official status. The report's basic search screen found no fresh name
with enough benefit to offset the existing domain, syntax, and migration
cost.[^branding-report]

A 17 August 2026 exact-name snapshot found no crates.io or npm package record
for Rocci but confirmed that PyPI is occupied and that GitHub's
case-insensitive name search is crowded by the existing rOCCI family. Rocdown
returned no exact crates.io, npm, PyPI, or GitHub repository-name result.
Rocweave returned no exact result on those package and repository screens, and
RDAP returned no record for its `.dev`, `.org`, or `.com` domains. These are
volatile discovery signals, not reservations or legal clearance.[^branding-report]

Rocdown remains a bounded endorsed name. Rocs is not recommended as a public
peer brand because its plural/acronym form produces crowded and unrelated
search results. Implementation names can remain while public prose moves to
Rocci Docs.[^branding-report]

## Search and message direction

Google recommends one concise, unique, consistently used site name and supports
an alternate name through `WebSite` structured data. Rocci should be the sole
site name; product descriptors belong in titles, headings, descriptions, and
visible copy rather than being substituted for the brand.[^google-site-names]

The current generated site already has canonical URLs, descriptive metadata,
Open Graph fields, a social image, a sitemap, robots policy, and static semantic
content. Priority gaps are a shorter descriptive home title and H1, structured
site identity, favicon assets, consistent typography between site and social
card, and intent-focused landing pages for Rocdown, Rocci Docs, and Datastar
integration.[^site-config][^site-theme][^branding-report]

The selected preview descriptor is **“Rocci — Composable authoring for
applications and content.”** Supporting copy should still explain concretely
that Rocci builds web and desktop interfaces, documents, and sites in Roc. The
current “without the framework tax” line is better treated as optional supporting
copy than the primary explanation.[^branding-report]

## Visual direction

The Rocs shell uses warm ivory, charcoal, and coral; the default Rocdown Paper
theme uses stone and blue; the branded Rocdown theme uses green. The surfaces
are individually coherent but do not yet form one recognizable family.[^site-theme][^paper-theme][^rocci-theme]

Keep the warm neutral/coral foundation, reserve a restrained violet for rare
Roc-ecosystem references, and make Paper a neutral, useful default without
product chrome. The branded Rocdown theme can converge on coral and violet only
through a separately reviewed compatibility-preserving change.[^branding-report]

Spot checks found two contrast risks: small white text on the current coral
button fill and the light-scheme green link in the branded Rocdown theme. These
findings require a complete accessibility audit rather than an isolated color
replacement.[^site-theme][^rocci-theme][^branding-report]

The existing single-letter `r` is a placeholder and must not constrain the
identity search. An `r` is not disqualified, but it must compete with non-letter
and wordmark-only directions. The maintainer likes the first orange folded-R
generation; that makes it a valid candidate, not an inherited default. Because
that generation saw the current social card, any production version must be
redrawn from first principles rather than traced.[^branding-report]

Roc describes its logo as an origami bird constructed from triangles, with an
Elm homage and computer-graphics rationale. Rocci may explore folds,
composition, and bounded geometry, but these are optional ideas rather than a
creative constraint, and the Roc bird must not be copied or modified.[^roc-faq]

The reset adds open-aperture, modular-commons, signal-bridge, and wordmark-only
probes. The aperture is too camera-like and the bridge too literal. The next
comparison should use three black-and-white vector routes: a fresh folded-letter
mark, a simplified non-letter modular mark, and a wordmark with no emblem. The
orange folded R is the current subjective favorite; no final route is approved.[^branding-report]

## Community foundation

Roc identifies Zulip as its primary community gathering place. Star Federation
identifies Discord as Datastar's community support venue. Rocci should seek Roc
feedback first on need, hierarchy, naming, and official-status perception, then
seek Datastar feedback on the accuracy and value of the integration.[^roc-community][^datastar-community]

The repository declares `MIT OR Apache-2.0` in Cargo metadata but currently has
no root license texts or standard root community health files. License text,
conduct, contribution, security, support, governance, issue forms, compatibility
policy, and a reproducible first-run path are public-preview blockers. GitHub
documents these files as infrastructure for transparent and healthy
contribution.[^root-readme][^github-health][^branding-report]

Begin with maintainer-led open development and call the effort the Rocci Project,
not a foundation. Formal nonprofit or fiscal-host work belongs after multiple
maintainers, funding, or durable shared assets create a real governance need.[^branding-report]

## Execution progress

The launch recommendation now has two working execution drafts. The community
kit contains Roc-first and Datastar-focused announcement copy, structured
feedback questions, consent and moderation rules, and a two-week synthesis
template.[^community-launch-kit] The landing brief defines the first-screen
information hierarchy, neutral wordmark-only identity posture, responsive
behavior, content rules, and acceptance checks for a dedicated home-page
surface.[^landing-prototype]

These drafts do not clear the publication gate. Placeholders, venue choice,
support ownership, compatibility claims, and public links require verification
immediately before use.

[^branding-report]: Full exploratory synthesis, naming matrix, SEO and visual audit, concept images, launch gates, and deferred investigations.
[^root-readme]: Current workspace, source formats, CLI, runtime, and documentation-generator description.
[^site-config]: Current public site name, descriptor, URL, repository, social image, output, and navigation.
[^site-theme]: Current Rocs shell structure, palette, responsive behavior, metadata, and accessibility CSS.
[^paper-theme]: Current default neutral Rocdown values.
[^rocci-theme]: Current green branded Rocdown values.
[^roc-faq]: Roc's explanation of its name and origami-bird logo construction.
[^roc-community]: Roc's current primary community venue and participation guidance.
[^datastar-community]: Datastar community venue, open-source purpose, and support framing.
[^google-site-names]: Search guidance for unique, concise, consistent site names and structured alternate names.
[^github-health]: Supported contribution, conduct, governance, security, support, and issue-template files.
[^pypi-rocci]: Current independent Python package occupying the `rocci` name on PyPI.
[^rocci-cloud]: Existing rOCCI suite using the similar name for cloud interoperability tooling.
[^community-launch-kit]: Draft external messages, feedback form, response protocol, synthesis format, and publishing order.
[^landing-prototype]: Proposed landing-page structure, visual direction, responsive behavior, and launch acceptance checks.
