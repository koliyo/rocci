---
type: Research Report
title: Rocdown long-form report authoring friction
description: Evidence and recommendations from using Rocdown and Rocs to author and publish the Rocci branding and community report.
tags: [domain/rocdown, domain/rocs, concern/authoring, concern/rendering, concern/publication, concern/accessibility]
status: draft
generated: { by: process:cursor, at: 2026-08-19T18:55:00Z }
stale_after: 2026-11-15
authority: exploratory
owners: [human:nils]
sources:
  - id: evaluation-report
    resource: ../../reports/branding/ROCDOWN_REPORT_AUTHORING_FRICTION.rocdown
    title: Rocdown report authoring friction and missing features
    author: process:codex
    last_modified: 2026-08-17
  - id: branding-report
    resource: ../../reports/branding/BRANDING_AND_COMMUNITY_REPORT.rocdown
    title: Rocci branding and community foundation report
    author: process:codex
    last_modified: 2026-08-17
  - id: rocdown-readme
    resource: ../../crates/rocci-rocdown/README.md
    title: Implemented Rocdown language reference
    author: process:git
    last_modified: 2026-08-17
  - id: img-parser
    resource: ../../crates/rocci-rocdown/src/img.rs
    title: Native image field extraction
    author: process:git
    last_modified: 2026-08-17
  - id: rocdown-lowerer
    resource: ../../crates/rocci-rocdown/src/lower.rs
    title: Standalone Rocdown lowerer
    author: process:git
    last_modified: 2026-08-17
  - id: rocs-docs
    resource: ../../crates/rocs/src/docs.rs
    title: Rocs static documentation component projection
    author: process:git
    last_modified: 2026-08-17
  - id: rocs-article
    resource: ../../crates/rocs/src/article.rs
    title: Rocs static article renderer
    author: process:git
    last_modified: 2026-08-17
  - id: rocs-theme
    resource: ../../crates/rocs/templates/RocsTheme.rocci
    title: Rocs documentation shell and print styling
    author: process:git
    last_modified: 2026-08-17
---

# Rocdown long-form report authoring friction

## Scope and authority

This is exploratory use-case research, not an approved language change. The
detailed evidence, friction matrix, capability architecture, priorities,
acceptance scenarios, and non-goals live in the accompanying report.[^evaluation-report]

The exercise used the 673-line branding and community report with page metadata,
tables, code, four heading levels, and seven native image declarations. The
source parsed successfully, and Rocs produced semantic static HTML plus hashed
copies of all seven assets.[^branding-report][^evaluation-report]

## Findings

Rocdown's Markdown-first prose layer is a strong fit for long-form reports.
Ordinary content required almost no conversion, while explicit page and image
declarations remained readable and statically inspectable.[^branding-report][^rocdown-readme]

The main gap is consumer parity. The standalone image lowerer preserves the
native image source, alt, title, class, width, height, loading, and decoding
fields.[^img-parser][^rocdown-lowerer] Rocs currently projects a native image
into the smaller Markdown image model, so its rendered HTML retains source,
alt, and title but loses the other authored fields.[^rocs-docs][^rocs-article]
The exercised static build confirmed that the branding report's lazy-loading
and async-decoding attributes disappeared.[^evaluation-report]

The same build exposed a page-link parity failure: Rocs accepted a relative
Rocdown source target during catalog validation but emitted the `.rocdown` href
unchanged instead of the generated page route.[^evaluation-report]

Image semantics are also split among Markdown image shorthand, the native
`:img` declaration, and `:figure`. The figure component requires a
figure-level alt field while rendering the nested image's own alt, which can
allow validation and emitted accessibility semantics to disagree.[^rocs-docs]

Ordinary Rocdown keeps footnotes disabled and lists standalone assets,
formatting, and automatic contents support among deferred features. Those gaps
are minor for application pages but material for evidence-based reports and
long-document maintenance.[^rocdown-readme]

Rocs already supplies valuable report-adjacent infrastructure: asset hashing,
an extracted heading outline, strict static output, and basic print rules. Its
default visible shell is nevertheless a documentation-site presentation rather
than a standalone report presentation.[^evaluation-report][^rocs-theme]

## Recommended direction

Preserve Rocdown's Markdown-first boundary and establish a shared, lossless
static-document contract. Rocdown should own semantic image, figure, footnote,
table, heading, and metadata nodes. Rocs should own asset publication, report
chrome, screen navigation, print presentation, and derived exports.[^evaluation-report]

The first implementation slice should:

1. preserve every native image field through Rocs;
2. add cross-consumer semantic parity tests;
3. rewrite validated internal page links to emitted routes;
4. make nested image alt authoritative for figures and provide explicit
   decorative-image intent;
5. enable accessible ordinary footnotes;
6. resolve source-relative assets in standalone preview;
7. provide an obvious one-file static report render path.[^evaluation-report]

A second slice should add optional report metadata, a logo-free Rocs report
presentation, a generated contents insertion point, responsive table
containers, report-grade print CSS, and deterministic PDF output. Figure
numbering, responsive image variants, shared static includes, formatting, and
DOCX interoperability can follow later.[^evaluation-report]

## Boundary

This recommendation does not propose raw HTML by default, WYSIWYG behavior in
the parser, site navigation in Rocdown core, or PDF and DOCX as canonical
formats. The OKF knowledge profile remains inert Markdown and separate from
executable Rocdown.[^evaluation-report][^rocdown-readme]

[^evaluation-report]: Detailed use-case procedure, observed output, friction matrix, recommended architecture, priorities, acceptance scenarios, and non-goals.
[^branding-report]: Concrete long-form report source and its seven native image declarations.
[^rocdown-readme]: Shipped Markdown-first contract, declarations, CLI boundary, and deferred feature list.
[^img-parser]: Accepted native image fields and current string-literal validation.
[^rocdown-lowerer]: Standalone image-to-HTML lowering and preservation of optional attributes.
[^rocs-docs]: Rocs image projection, docs figure validation, component fields, and includes.
[^rocs-article]: Static Markdown image rendering and available image attributes.
[^rocs-theme]: Current documentation-site shell, extracted outline presentation, and basic print rules.
