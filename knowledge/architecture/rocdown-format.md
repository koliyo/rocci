---
type: Architecture
title: Rocdown format boundary
description: Rocdown is Markdown-first content with explicit document-root Roc and Rocci regions, static defaults, and a separate static knowledge-body profile.
tags: [domain/rocdown, concern/syntax, concern/rendering, concern/security]
status: draft
generated: { by: process:cursor, at: 2026-08-19T19:10:00Z }
verified:
  - { by: human:nils, at: 2026-08-16T18:14:13Z }
stale_after: 2027-02-12
authority: descriptive
owners: [human:nils]
sources:
  - id: rocdown-readme
    resource: ../../crates/rocci-rocdown/README.md
    title: Implemented Rocdown language reference
    author: process:git
    last_modified: 2026-08-17
  - id: parser
    resource: ../../crates/rocci-rocdown/src/parse.rs
    title: Rocdown parser
    author: process:git
    last_modified: 2026-08-17
  - id: scanner
    resource: ../../crates/rocci-rocdown/src/scan.rs
    title: Rocdown document-root scanner
    author: process:git
    last_modified: 2026-08-17
  - id: lowerer
    resource: ../../crates/rocci-rocdown/src/lower.rs
    title: Rocdown to Roc lowerer
    author: process:git
    last_modified: 2026-08-17
  - id: img
    resource: ../../crates/rocci-rocdown/src/img.rs
    title: Rocdown image field extraction
    author: process:git
    last_modified: 2026-08-17
  - id: compiler-tests
    resource: ../../crates/rocci-rocdown/tests/compile.rs
    title: Rocdown compiler contract tests
    author: process:git
    last_modified: 2026-08-17
  - id: cli-options
    resource: ../../crates/rocci-rocdown-cli/src/main.rs
    title: Rocdown CLI compile-option construction
    author: process:git
    last_modified: 2026-08-17
  - id: format-report
    resource: ../../archive/reports/ROCDOWN_FORMAT_REPORT.md
    title: Original Rocdown format investigation
    author: human:nils
    last_modified: 2026-08-15
---

# Rocdown format boundary

## Current contract

A `.rocdown` document interleaves ordinary Markdown with reserved declarations recognized only at a document-root line boundary. Reserved declarations include page metadata, Roc blocks, rendered Roc expressions, Rocci components and fixtures, scoped CSS, server lifecycle forms, structural template forms, line-start `:kind` article blocks (`:note`, `:img`, `:tabs`, …), and document-root HTML islands.[^rocdown-readme][^parser]

Article blocks use `:kind[params]`. Content is line-scope, a `{{ }}` section, or `:kind.begin` ... `:kind.end`. A call uses one delimiter, not both. Line-start `:end.kind` is a removal diagnostic naming `:kind.end`.[^rocdown-readme][^parser]

Article widgets have no family prefix. A short-lived experimental `@docs` / `@img` family is **removed** and is not a live syntax, alias, compatibility window, or design analogy. Line-start `@docs` and `@img` remain only as removal diagnostics that name `:note` and `:img[...]`.[^rocdown-readme][^parser]

Markdown supports CommonMark plus tables, strikethrough, task lists, extended autolinks, heading IDs, footnotes, and Rocdown page-link forms. Raw inline HTML is disabled by default. Ordinary compilation and the OKF adapter share footnote parsing; OKF still validates keyed `sources[].id` separately.[^rocdown-readme][^parser][^compiler-tests]

`:img` requires `alt` unless `decorative: Bool.true`. Nested `:img` inside `:figure` owns accessibility text; figure-level `alt` is not a figure field. Caption and credit remain figure metadata and do not substitute for image alt. Local image paths resolve against the source file directory.[^img][^compiler-tests]

The standalone compiler lowers documents to ordinary Roc exports for metadata, content, and the page shell. It does not type-check Roc or run the server itself.[^rocdown-readme]

## HTML boundary

A line-start `<Tag>` at document root is scanned before CommonMark and parsed as a Rocci `Element`, `ComponentCall`, or `Fragment`. It therefore uses Rocci template syntax and lowers through structured `Html.element`, `Html.void_element`, `Html.text`, attributes, and component calls; it is not preserved as an authored HTML string and does not itself invoke `Html.dangerously_include_unescaped_html`. Autolinks, comments, doctypes, processing instructions, namespaced-looking tags, and tags inside a list, quote, or fence do not enter this document-root island path.[^scanner][^parser][^lowerer][^compiler-tests]

CommonMark raw HTML is a separate AST case. The default parser reports it as an error, while the library-only `CompileOptions.raw_html` opt-in preserves the literal with `Html.dangerously_include_unescaped_html`. That opt-in bypasses normal text and attribute escaping and must therefore be limited to trusted authored input; it does not reinterpret the tag as a Rocci component. The normal CLI compile options leave it disabled.[^rocdown-readme][^lowerer][^compiler-tests][^cli-options]

This split has a syntax consequence: moving an otherwise HTML-looking block to document root can change it from rejected Markdown raw HTML into executable Rocci template syntax, including interpolation and component calls. Authors who intend to display HTML should use a fenced code block.[^scanner][^compiler-tests]

## Language boundary

Markdown owns prose. Roc owns data and computation inside explicit regions. Rocci owns server-rendered templates, and server handlers remain visible declarations. Fenced code is always displayed rather than executed.[^format-report]

This is the implemented form of the [Markdown-first explicit-islands decision](/decisions/markdown-first-explicit-islands.md). The stricter [OKF boundary](/decisions/static-okf-boundary.md) reuses only the Markdown AST and renderer, not Rocdown declarations.

## Not yet implemented

`@island`, a formatter, content collections, near-miss directive warnings, and several proposed Markdown extensions (admonitions, definition lists, math, automatic TOC) are not part of ordinary Rocdown today. `:api-operation` is a reserved article kind that Rocdown rejects until generated API reference ships. Multi-page static generation and hashed assets are owned by Rocdown (`rocci-rocdown`); standalone preview copies local images without hashing.[^rocdown-readme]

## Evidence policy

The crate README and parser describe shipped behavior. The 2026-08-15 report supplies design rationale and future proposals only where explicitly labeled; it is not allowed to override the current implementation.[^format-report]

[^rocdown-readme]: Current file shape, declarations, Markdown profile, lowering, and implemented/deferred list.
[^parser]: Executable recognition and parsing behavior in code, including ordinary footnotes.
[^scanner]: Document-root recognition, exclusions, and handoff to the Rocci template parser.
[^lowerer]: Structured Markdown lowering and the explicit raw-HTML escape hatch.
[^img]: `@img` field extraction, alt/decorative validation, and source-relative asset resolution.
[^compiler-tests]: Regression coverage for root HTML islands, autolinks, list and fence boundaries, raw-HTML defaults, footnotes, and the image/figure contract.
[^cli-options]: CLI construction of Rocdown compile options without a raw-HTML override.
[^format-report]: Original design rationale, with its own warning that current crate documentation has precedence.
