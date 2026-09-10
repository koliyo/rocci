---
type: Research Report
title: Rocdown LSP does not complete wiki or internal page links
description: "Rocdown LSP already indexes workspace pages for diagnostics and go-to-definition, but completion returns an empty list in Markdown prose, trigger characters omit `[`, and incomplete `[[` never becomes an AST link."
tags: [domain/rocdown, concern/tooling, concern/authoring, concern/syntax]
status: draft
generated: { by: process:cursor, at: 2026-09-10T08:10:00Z }
stale_after: 2026-12-10
authority: exploratory
owners: [human:nils]
sources:
  - id: rocdown-lsp
    resource: ../../../crates/rocci-rocdown/src/lsp.rs
    title: Rocdown analyzer compile_text, completion, goto_definition
    author: process:git
    last_modified: 2026-09-01
  - id: lsp-tests
    resource: ../../../crates/rocci-rocdown/tests/lsp.rs
    title: Rocdown LSP tests for kinds, fields, interpolation
    author: process:git
    last_modified: 2026-09-01
  - id: lsp-core
    resource: ../../../crates/rocci-lsp/src/lib.rs
    title: Composed server capabilities and completion triggers
    author: process:git
    last_modified: 2026-09-04
  - id: links-rs
    resource: ../../../crates/rocci-rocdown/src/links.rs
    title: Compile-time page index and URL resolution
    author: process:git
    last_modified: 2026-09-01
  - id: wiki-catalog
    resource: ../../../crates/rocci-rocdown/src/catalog/graph.rs
    title: Catalog wiki_target, RD2101, RD2105
    author: process:git
    last_modified: 2026-09-01
  - id: site-index
    resource: ../../../crates/rocci-rocdown/src/site.rs
    title: workspace_pages cache from rocdown.toml
    author: process:git
    last_modified: 2026-09-01
  - id: markdown-rs
    resource: ../../../crates/rocci-rocdown/src/markdown.rs
    title: Comrak WikiLink lowered to MdNode::Link
    author: process:git
    last_modified: 2026-08-23
  - id: readme
    resource: ../../../crates/rocci-rocdown/README.md
    title: Wiki and page-link contract
    author: process:git
    last_modified: 2026-09-10
  - id: lang-ref
    resource: ../../../docs/rocdown/language.rocdown
    title: Public Rocdown links reference
    author: process:git
    last_modified: 2026-09-10
  - id: pages-ref
    resource: ../../../docs/rocdown/pages.rocdown
    title: Published routes versus wiki keys
    author: process:git
    last_modified: 2026-09-10
  - id: tooling-arch
    resource: ../../architecture/language-tooling.md
    title: Rocci language-tooling boundary
    author: process:cursor
    last_modified: 2026-08-25
  - id: language-plan
    resource: ../../plans/rocci/language-server.md
    title: Umbrella language-tooling plan
    author: process:cursor
    last_modified: 2026-08-31
  - id: impl-plan
    resource: ../../plans/rocdown/lsp-wiki-link-completion.md
    title: Implementation plan for Rocdown link completion
    author: process:cursor
    last_modified: 2026-09-10
---

# Rocdown LSP does not complete wiki or internal page links

## Claim

Authors can type `[[` in a `.rocdown` file and get no page completions. The
composed language server already knows enough pages to diagnose and jump to
targets, but `completion` never consults that index in Markdown, and clients
are not told to trigger on `[`. Incomplete wiki markup is not an AST link, so
a parse-only approach cannot work. Internal Markdown destinations
(`[label](…)` and `#heading`) are the same gap with a second lexical
context.[^rocdown-lsp][^lsp-core][^readme]

This is evidence for [Rocdown LSP wiki and internal link completion](/plans/rocdown/lsp-wiki-link-completion.md),
not a description of shipped editor behavior. It is a slice of the umbrella
[language-server](/plans/rocci/language-server.md) “richer completion” and
workspace-index work, not a replacement for that plan.[^language-plan][^tooling-arch][^impl-plan]

## What is already implemented

**Page index at compile.** `compile_text` loads `workspace_pages(path)` when
the file sits under a `rocdown.toml` root (mounts and peers included).
Otherwise it indexes sibling `.rocdown` files in the parent directory. The
open buffer replaces the on-disk `PageRef` for that path. That same index
feeds `resolve_links`, so unknown wiki or `.rocdown` targets already become
diagnostics in the editor when the file is on disk.[^rocdown-lsp][^site-index][^links-rs]

**Go to definition.** If the cursor sits on a *parsed* `LinkInfo` span and
the resolved URL matches a workspace or sibling route, the server returns
that file. Closed `[[Page]]` and `[text](Page.rocdown)` can jump. Open
`[[Pag` cannot.[^rocdown-lsp][^markdown-rs]

**Completion today.** Rocdown completion covers `@` declarations, `:kind`
names, block fields, `@page` fields/themes, Rocci template tags, and
forwarded Roc in executable regions. After those contexts miss, the
response is an empty array. There is no wiki, route, relative-path, or
heading-fragment item. Tests lock kinds, fields, and interpolation, not
links.[^rocdown-lsp][^lsp-tests]

**Triggers.** The generic server advertises `completion_provider.trigger_characters`
as `<` and `@` only. Typing `[` does not request completion unless the
editor uses a manual invoke. There is one capability set for `.rocci` and
`.rocdown`.[^lsp-core][^tooling-arch]

## Two resolvers, two key spaces

The public contract says wiki `[[Foo]]` matches a unique title, file stem, or
page id; a target containing `/` is a relative path, not a page id; composed
sites should prefer published routes such as `[label](/docs/applications/)`.[^readme][^lang-ref][^pages-ref]

Catalog check implements that with `wiki_target` on `ResolvedPage` (`by_id`,
id last segment, file stem, title) and `RD2105` on ambiguity.[^wiki-catalog]

Compile-time `resolve_url` does **not** use `wiki_target`. A slashless wiki
or file href is a relative join, then a stem/filename lookup on `PageRef`.
`PageRef` stores stem, file name, path, route, and heading ids. It does not
store catalog id or title. Title-only wiki keys therefore resolve in
`rocdown check` / site graph and can fail in single-file compile and LSP
diagnostics.[^links-rs][^site-index][^wiki-catalog]

Completions that offer a key the open file’s resolver will reject would
teach the wrong spelling. Honest items are keys that `workspace_pages` plus
the compile resolver (and, for site files, catalog `wiki_target` if the
plan unifies matching) actually accept.

`workspace_pages` already derives catalog-like routes from the relative path
under the site root (including mount prefixes). That relative stem is the
practical wiki id for docs authors (`rocdown/pages`, not `docs/rocdown/pages`
inside the `docs/` catalog).[^site-index][^pages-ref]

## Why incomplete wiki cannot use the AST

Comrak wiki links become `MdNode::Link` only when the `]]` (and optional
`|label`) is present. While the author types `[[rocd`, there is no
`LinkInfo`. Completion must scan the line for an unclosed `[[` (and
optionally `#` / `|` inside it), the same way `:note[ti` uses
`incomplete_bracket_before` instead of a finished `BlockCall`.[^markdown-rs][^rocdown-lsp]

That scan must not steal `:kind[` field completion: colon fields are a
single `[` after `:name`. Wiki is `[[`. It must not run in fenced or inline
code, `@page` / `@roc` / template bodies, or HTML-shaped component tags.

Markdown internal links need a second scan: after `](` on the same
construct, prefix-filter routes, relative `.rocdown` / `.md` paths, and
same-page or target-page `#` heading ids. Image destinations and external
schemes are a different product.

## `.md` and cache limits

`filesystem_path` used by `compile_text` returns a path only for `.rocdown`.
A `.md` / `.markdown` buffer that the analyzer accepts therefore compiles
without a page index, so even closed wiki links get no sibling/workspace
resolution in LSP. Wiki completion for those files needs the same path
gate widened, or it stays Rocdown-only.[^rocdown-lsp]

The workspace index cache keys on `rocdown.toml` mtime plus discovered file
count, not per-file content. Stems and routes from disk can lag title
edits in unsaved buffers. Overlaying the open document (already done for
the current file’s `PageRef`) is enough for v1; live cross-file title
sync is workspace-intelligence leftover.[^site-index][^language-plan]

## Implications for a small plan

1. Detect wiki and Markdown-link completion **lexically**, not from
   `compiled.links`.
2. Reuse `workspace_pages` / sibling `index_pages_in_dir`; do not run a
   full catalog HTML resolve on every keystroke.
3. Add `[` (and for internal links `(` and `#`) to composed-server trigger
   characters; Rocci completion already no-ops unknown contexts.
4. Offer keys the resolver will accept; prefer catalog id and unique stem
   for wiki, published route and relative document path for `[text](`.
5. Keep this out of `rocci-lsp` analysis types. Rocdown owns the items;
   the generic crate only advertises triggers.[^tooling-arch]
