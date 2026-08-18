---
type: Implementation Plan
title: Full Rocci and Rocdown language tooling
description: Build region-aware Rocci and Rocdown language tooling for VS Code and Zed and reuse language-neutral token spans for static Rocdown code highlighting.
tags: [domain/rocci, domain/rocdown, domain/rocs, integration/roc, concern/tooling, concern/syntax]
status: draft
generated: { by: process:codex, at: 2026-08-17T19:29:13Z }
stale_after: 2026-10-01
authority: exploratory
owners: [human:nils]
sources:
  - id: detailed-plan
    resource: ../../ROCCI_LANGUAGE_SERVER_IMPLEMENTATION_PLAN.md
    title: Rocci language-server report and detailed implementation plan
    author: process:codex
    last_modified: 2026-08-17
  - id: tooling-architecture
    resource: ../architecture/language-tooling.md
    title: Current Rocci language-tooling boundary
    author: process:codex
    last_modified: 2026-08-17
  - id: rocdown-boundary
    resource: ../architecture/rocdown-format.md
    title: Rocdown format boundary
    author: process:codex
    last_modified: 2026-08-16
  - id: rocdown-compiler
    resource: ../architecture/rocdown-documentation-compiler.md
    title: Current Rocdown documentation generator boundary
    author: process:codex
    last_modified: 2026-08-17
  - id: rocs-article
    resource: ../../crates/rocs/src/article.rs
    title: Current Rocs static article renderer
    author: process:git
    last_modified: 2026-08-16
  - id: rocs-docs
    resource: ../../crates/rocs/src/docs.rs
    title: Current Rocs include and example pipeline
    author: process:git
    last_modified: 2026-08-16
  - id: source-map
    resource: ../../crates/rocci-template/src/source_map.rs
    title: Current generated-Roc source-map segment model
    author: process:git
    last_modified: 2026-08-15
  - id: zed-roc
    resource: https://github.com/h2000/zed-roc/tree/f6a07bfb336549724f9c5694084bfb1869614b5d
    title: Zed Roc extension at inspected revision
    author: human:alf-richter
    last_modified: 2026-06-26
  - id: tree-sitter-roc
    resource: https://github.com/faldor20/tree-sitter-roc/tree/edc18052a9d7382ac9f9f5bf413db3a78d5ea12c
    title: Roc Tree-sitter grammar pinned by zed-roc
    author: human:eli-dowling
    last_modified: 2026-01-27
  - id: zed-languages
    resource: https://zed.dev/docs/extensions/languages
    title: Zed language extension documentation
    author: organization:zed-industries
    last_modified: 2026-08-17
  - id: vscode-embedded
    resource: https://code.visualstudio.com/api/language-extensions/embedded-languages
    title: VS Code embedded-language guidance
    author: organization:microsoft
    last_modified: 2026-08-17
  - id: vscode-semantic
    resource: https://code.visualstudio.com/api/language-extensions/semantic-highlight-guide
    title: VS Code semantic highlighting guide
    author: organization:microsoft
    last_modified: 2026-08-17
  - id: tree-sitter-highlighting
    resource: https://tree-sitter.github.io/tree-sitter/3-syntax-highlighting.html
    title: Tree-sitter syntax-highlighting documentation
    author: organization:tree-sitter
    last_modified: 2026-08-17
  - id: tree-sitter-highlight-rust
    resource: https://docs.rs/tree-sitter-highlight/latest/tree_sitter_highlight/
    title: tree-sitter-highlight Rust API
    author: organization:tree-sitter
    last_modified: 2026-08-17
  - id: syntect-html
    resource: https://docs.rs/syntect/latest/syntect/html/struct.ClassedHTMLGenerator.html
    title: Syntect classed HTML generator
    author: organization:trishume
    last_modified: 2026-08-17
  - id: shiki
    resource: https://shiki.style/guide/install
    title: Shiki installation and HTML generation
    author: organization:shikijs
    last_modified: 2026-08-17
  - id: product-boundary
    resource: ../decisions/consolidate-rocdown-product-boundary.md
    title: Approved consolidated Rocdown product direction
    author: process:codex
    last_modified: 2026-08-17
---

# Full Rocci and Rocdown language tooling

## Goal

Provide region-aware Rocci and Rocdown analysis that composes embedded-language
results into ordinary LSP responses for VS Code and Zed. The first milestone is
syntax highlighting for embedded Roc and CSS plus HTML-shaped Rocci templates
and Rocdown Markdown; the full target adds workspace navigation, safe rename
and formatting, and compiler-backed Roc semantics. The same language-neutral
byte-span tokens should also drive static syntax-highlighting HTML in the
documentation generator.[^detailed-plan]

This record is a proposed implementation sequence, not an approved or shipped
contract.

The detailed root plan originally assumed one `rocci-language-server` owned
both file types. Under the approved product direction, base Rocci
tooling must not import Rocdown AST types. Reusable protocol, Roc/Rocci
analysis, region, and token primitives stay below the boundary; a
Rocdown-owned server or adapter composes them with Rocdown analysis and editor
registration.[^product-boundary]

## Direction

Use the Rocci and Rocdown parsers as the only authority for language
boundaries. Model each region with a language, syntactic context, purpose, byte
span, parent, and precedence. Purpose must distinguish executable regions from
display-only fences so syntax highlighting never changes execution
semantics.[^rocdown-boundary][^detailed-plan]

Use source-preserving projections for fast lexical features and the compiler's
generated Roc plus an expanded bidirectional source map for later type-aware
Roc features. Compose backend results once in source byte spans before
converting to the client's position encoding.[^source-map][^detailed-plan]

Reuse the Roc Tree-sitter grammar and adapted highlight queries from the
`zed-roc` dependency chain, with pinned revisions and license attribution. Do
not merge the Zed manifest or its direct Roc-server launcher into the common
server. Zed-specific queries and VS Code-specific forwarding are evidence, not
portable LSP APIs.[^zed-roc][^tree-sitter-roc][^zed-languages][^vscode-embedded]

Keep grammar/query configuration, normalized token kinds, span validation, and
overlap composition in a small shared Rust crate. Protocol position encoding
and Rocci analysis remain reusable without a Rocdown dependency; escaped HTML
and CSS theming stay in the current Rocs generator and move with it into
Rocdown. The generator links the token library in-process and never launches an
LSP, editor, Node process, or authored code to highlight a site.[^rocdown-compiler][^detailed-plan][^product-boundary]

## Prerequisites

- Restore the `rocci-lsp` build after Rocdown `Item::Docs` and define its
  symbols/tokens.
- Add comprehensive embedded-language fixtures and region/token goldens.
- Smoke-test the checked-in VS Code and Zed development extensions on declared
  editor versions.
- Select one compatible Tree-sitter runtime and pinned Roc/CSS/HTML grammar set.
- Record third-party license notices for copied queries or vendored parsers.

## Phases

### 0. Compatibility baseline

Repair the `@docs` LSP regression, freeze current behavior with fixtures, and
verify both clients attach to the current server. Exit when `cargo test -p
rocci-lsp` passes and editor prerequisites are reproducible.[^tooling-architecture]

### 1. Embedded-highlighting demonstrator

Extract a typed region graph, add pinned in-process Tree-sitter backends for
Roc, CSS, and ordinary/display-only HTML, retain Rocci-AST highlighting for
executable HTML-shaped templates, add Markdown and display-fence tokens, and
merge all streams into standard non-overlapping semantic tokens. VS Code and
Zed must render the same fixture through the applicable product server or
adapter.[^tree-sitter-highlighting][^vscode-semantic][^product-boundary]

### 1b. Static documentation demonstrator

Inject the shared token service at the current Rocs `MdNode::CodeBlock`
renderer, which moves into Rocdown under the approved boundary. Preserve
the existing escaped `<pre><code>` fallback and emit only allowlisted semantic
classes around escaped source slices. Cover ordinary fences, non-Rocdown
`@docs include`, fences nested in `@docs example`, unknown languages,
malformed snippets, and hostile HTML text.[^rocs-article][^rocs-docs]

Start with Roc, HTML, CSS, and composite Rocci/Rocdown snippets. Add shell,
TOML, and Markdown from measured documentation demand. Theme the stable token
classes in Rocci-owned CSS with light, dark, print, and forced-colors behavior;
do not emit inline colors or require client-side JavaScript.[^detailed-plan]

Tree-sitter's Rust highlighter already exposes reusable configurations,
highlight events, injections, and HTML rendering support. Normalize its events
before rendering so language tooling and the generator share classification
and precedence.
Syntect or Shiki can provide broader TextMate language coverage, but either
would create a second grammar/theme pipeline and is not the initial product
language solution.[^tree-sitter-highlight-rust][^syntect-html][^shiki]

### 2. Structural editing

Add versioned snapshots, incremental text changes, cancellation, folding,
selection ranges, document links, linked tags, richer completion/hover,
syntax code actions, and semantic-token deltas. These features must work
without Roc installed.

### 3. Workspace intelligence

Index modules, components, pages, routes, headings, links, styles, selectors,
and literal class/id uses. Add cross-file definition, references, workspace
symbols, and conservative rename with dependency-based invalidation. Reuse
the documentation catalog logic for page and link semantics without moving
that logic into base Rocci tooling.

### 4. Roc semantics

Prototype one optional Roc child server per workspace against generated Roc
modules. Expand source maps for exact bidirectional mapping and add mapped type
diagnostics, hover, completion, signatures, definitions, and references.
Reject ambiguous edits and preserve a useful degraded mode when the Roc
backend is absent or incompatible.[^source-map]

### 5. Formatting and refactoring

Define a lossless host formatter, compose region-owned Roc/CSS/Markdown edits,
and add refactors only where mappings are reversible. Formatting must be
idempotent and preserve the executable/display-only boundary.

### 6. Productization

Ship versioned platform binaries, adapter compatibility policy, grammar and
protocol revision reporting, performance/cancellation budgets, fuzz and crash
hardening, license inventory, and release smoke tests for supported VS Code
and Zed versions.

## Validation and exit criteria

- Every language region and malformed boundary has a byte-span golden test.
- Semantic tokens never overlap, escape a region, or expose synthetic wrapper
  text; UTF-8 and UTF-16 results are equivalent.
- Range and delta tokens reproduce the corresponding full-token state.
- Fenced examples can be highlighted but are never classified as executable.
- Child-backend locations, diagnostics, and edits map only to authored spans;
  stale or ambiguous edits are refused.
- Both editors pass client smoke tests with the same fixtures and the
  product-owned server or adapter selected by the boundary refactor.
- Static documentation HTML and LSP semantic tokens are derived from the same
  byte-span golden for representative Roc, HTML, CSS, Rocci, and Rocdown
  snippets.
- Static output escapes every source segment exactly once, uses only
  allowlisted classes, has a deterministic plain fallback, and is identical
  across repeated builds.
- Host functionality remains available when optional embedded backends fail.
- Performance, compatibility, and third-party licenses are recorded before a
  release.

## Open gates

The project must choose the Roc grammar revision/runtime combination, verify
current Zed's grammar requirement against the existing adapter, prove the Roc
child server's generated-workspace behavior, and decide whether display fences
and static sites support only bundled lexical backends or a broader optional
documentation pack. It must also decide when token CSS class names become a
public theme API. These gates do not block the common semantic-token and static
HTML demonstrators.[^detailed-plan]

[^detailed-plan]: Detailed baseline, alternatives, region/projection architecture, demonstrator tasks, feature roadmap, risks, and evidence.
[^tooling-architecture]: Current shipped surface, client boundary, embedded-range gap, and `@docs` build regression.
[^rocdown-boundary]: Implemented Markdown-first and explicit executable-region contract.
[^source-map]: Current generated/source span and origin representation that requires richer bidirectional policies.
[^zed-roc]: Editor-specific bundle whose grammar/query assets are reusable but whose manifest and launcher are not.
[^tree-sitter-roc]: Exact grammar revision pinned by the inspected Zed extension.
[^zed-languages]: Current Zed grammar, injection, and semantic-token behavior.
[^vscode-embedded]: Embedded-service alternatives and portability tradeoffs.
[^vscode-semantic]: Standard semantic-token classification and enablement behavior.
[^tree-sitter-highlighting]: Query capture and in-process highlighting model.
[^tree-sitter-highlight-rust]: Reusable Rust highlighter configuration, event stream, injection callback, and HTML-renderer API.
[^rocdown-compiler]: Current Rust article-rendering and Rocci-shell ownership boundary.
[^rocs-article]: Current escaped code-block HTML shape and central rendering path.
[^rocs-docs]: Current include-language precedence, example metadata, and normalization to article code blocks.
[^syntect-html]: Alternative classed HTML renderer over TextMate-style syntax sets.
[^shiki]: Alternative ESM/WASM TextMate highlighter with HTML, token, and HAST output.
[^product-boundary]: Proposed removal of Rocdown from base Rocci tooling, Rocdown-owned document composition, and one-way dependency rules.
