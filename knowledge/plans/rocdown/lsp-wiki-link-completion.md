---
type: Implementation Plan
title: Rocdown LSP wiki and internal link completion
description: "Complete wiki `[[` targets and Markdown internal destinations from the existing Rocdown workspace page index, using lexical context because incomplete links are not in the AST."
tags: [domain/rocdown, concern/tooling, concern/authoring, concern/syntax]
status: draft
generated: { by: process:cursor, at: 2026-09-10T08:10:00Z }
stale_after: 2026-12-10
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../../research/rocdown/lsp-wiki-link-completion.md
    title: Rocdown LSP does not complete wiki or internal page links
    author: process:cursor
    last_modified: 2026-09-10
  - id: rocdown-lsp
    resource: ../../../crates/rocci-rocdown/src/lsp.rs
    title: Rocdown analyzer completion and compile_text
    author: process:git
    last_modified: 2026-09-01
  - id: lsp-core
    resource: ../../../crates/rocci-lsp/src/lib.rs
    title: Completion trigger characters
    author: process:git
    last_modified: 2026-09-04
  - id: links-rs
    resource: ../../../crates/rocci-rocdown/src/links.rs
    title: PageRef and compile-time resolve_url
    author: process:git
    last_modified: 2026-09-01
  - id: wiki-catalog
    resource: ../../../crates/rocci-rocdown/src/catalog/graph.rs
    title: Catalog wiki_target
    author: process:git
    last_modified: 2026-09-01
  - id: site-index
    resource: ../../../crates/rocci-rocdown/src/site.rs
    title: workspace_pages
    author: process:git
    last_modified: 2026-09-01
  - id: lsp-tests
    resource: ../../../crates/rocci-rocdown/tests/lsp.rs
    title: Rocdown LSP integration tests
    author: process:git
    last_modified: 2026-09-01
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
  - id: tooling-arch
    resource: ../../architecture/language-tooling.md
    title: Analyzer composition boundary
    author: process:cursor
    last_modified: 2026-08-25
  - id: language-plan
    resource: ../rocci/language-server.md
    title: Umbrella language-tooling plan
    author: process:cursor
    last_modified: 2026-08-31
  - id: language-dev
    resource: ../../../.agents/skills/rocci-language-dev/SKILL.md
    title: Language-development skill
    author: process:git
    last_modified: 2026-08-22
---

# Rocdown LSP wiki and internal link completion

## Purpose and authority

This is the implementation plan for editor completion of Rocdown wiki links
and, in the same slice, other internal page destinations. Evidence of the
gap is [Rocdown LSP does not complete wiki or internal page links](/research/rocdown/lsp-wiki-link-completion.md).
It does not describe shipped behavior. Crate READMEs remain the link
contract until a phase lands.[^research][^readme]

Do not start a phase until the user asks. Branch name:
`lsp-wiki-link-completion`. Rocdown owns analysis; `rocci-lsp` only gains
trigger characters. Umbrella leftover (document-link protocol, workspace
symbols, rename) stays in [language tooling](/plans/rocci/language-server.md).[^tooling-arch][^language-plan][^language-dev]

## Goal

In a `.rocdown` buffer under a site root or sibling index, typing `[[` (or
invoking completion inside an unclosed wiki target) offers pages that the
same index would resolve. Typing inside `[label](…)` offers published
routes and relative document paths. `#` inside those destinations offers
heading ids from the current or target page.

After the last in-scope phase:

- Wiki items reuse the workspace/sibling page index already used by
  `compile_text`; they do not spawn a full catalog HTML resolve.[^rocdown-lsp][^site-index]
- Incomplete `[[prefix` completes even though Comrak has no `LinkInfo`.[^research]
- `:kind[` field completion still wins over a single `[`.[^rocdown-lsp]
- Clients trigger on `[`, and for internal links also `(` and `#`.[^lsp-core]
- Tests in `crates/rocci-rocdown/tests/lsp.rs` cover wiki prefix, Markdown
  href prefix, heading fragment, code-fence suppression, and no steal from
  `:note[`.[^lsp-tests]

## Out of bound

- Changing wiki or route **resolution** semantics, `RD2101` / `RD2105`
  wording, or published-route rewrite in HTML.
- OKF knowledge wikilinks (canonical `knowledge/` stays inert Markdown).
- `textDocument/documentLink`, workspace symbols, rename, or a second
  workspace indexer in `rocci-lsp`.
- Completing `https:`, `mailto:`, or image/asset hrefs.
- Fuzzy ranking beyond prefix (and uniqueness) filter.
- Per-keystroke recrawl of every page body for live titles across files.
- Editor-extension UI beyond standard LSP `CompletionItem`.

## Constraints that do not move

- Completions must be keys the open file’s resolver will accept. Do not
  offer a title-only wiki key if compile-time `resolve_url` still matches
  stem/filename only, unless a phase extracts a shared matcher and compile
  uses it too.[^links-rs][^wiki-catalog][^readme]
- Prefer catalog-relative id and unique file stem for `[[…]]`. Prefer
  published route (`/docs/…/`) and relative `.rocdown` / `.md` path for
  `[text](…)`. A target containing `/` remains a relative path, not a page
  id.[^lang-ref][^readme]
- Skip fenced code, inline code, `@page`, `@roc`, and template/component
  regions (existing completion order stays first).
- `PageRef` may grow id/title fields if required; do not thread
  `ResolvedPage` or article HTML into the LSP analyzer.
- One composed server capability list; extra triggers must be harmless in
  `.rocci` (empty completion is fine).[^lsp-core][^tooling-arch]
- Monotonic scanners on every incomplete-delimiter path.

## Phases

### 0. Lexical wiki context and fixtures

**Bound:** Helper that, given source text and byte offset, returns wiki
completion context (`prefix`, optional heading prefix, optional after
`|label`) or none. Closed `[[Page]]` with the cursor in the target counts.
Suppress in code spans/fences. Do not treat `:name[` as wiki. Add failing
or table-driven unit tests next to `crates/rocci-rocdown/src/lsp.rs` (or a
focused `links` helper module) before wiring items.

**Out of bound:** Trigger characters, Markdown `](`, real completion items.

**Exit:** `cargo test -p rocci-rocdown --lib` (or the new test target)
covers `[[`, `[[pre`, `[[Page#`, not `:note[`, not `` `[[x` ``.

### 1. Wiki items from the existing page index

**Bound:** `completion` consults the same pages `compile_text` built (store
`Vec<PageRef>` on `RocdownAnalysis` or re-query `workspace_pages` /
`index_pages_in_dir`). Filter by prefix. Label is a unique wiki key:
catalog-relative id when present, else stem; `detail` is the route. Dedup
ambiguous stems (catalog `RD2105` keys): omit or mark, do not insert a
key that would be ambiguous. Advertise `[` as a trigger character on the
composed server. Overlay the open buffer’s current `PageRef` as today.

If compile-time matching still lacks catalog id/title, either (a) put the
relative id onto `PageRef` in `page_ref_from_relative` / sibling index, or
(b) extract `wiki_target` against a thin key struct shared with
`catalog/graph.rs`. Prefer (b) only if both compile resolve and completion
call it in this phase so offered keys resolve.

**Out of bound:** Markdown `](` hrefs, heading `#` lists, `.md` path-gate
unless required to index the current file.

**Exit:** `cargo test -p rocci-rocdown --test lsp` — temp site or sibling
dir, buffer `See [[pag` completes `pages` (or the fixture stem); empty
inside a fence. `cargo test -p rocci-rocdown-lsp`. `cargo fmt --all -- --check`.

### 2. Heading fragments

**Bound:** Inside wiki or after `[[Page#` / `[text](Page#` / same-page `#`
in a link destination, complete `heading_ids` from `PageRef` (and the open
document’s `compiled.headings` for the current file). Prefix filter.
`#` is a trigger character.

**Out of bound:** Completing headings as wiki page keys. Source-line `L123`
anchors.

**Exit:** LSP test: `[[Target#he` offers `heading` from the target file’s
ATX ids.

### 3. Internal Markdown destinations

**Bound:** Lexical context after `](` (not images `![`) until `)` or
whitespace that ends the destination. Prefix-complete (1) canonical
routes from the page index, (2) relative document paths
(`Foo.rocdown`, `./nested/Foo.rocdown`). `(` is a trigger character.
Schemes with `:` are ignored. Widen `filesystem_path` to `.md` /
`.markdown` if those buffers should share the index; otherwise document
Rocdown-only in the crate README.

**Out of bound:** Asset files, query strings, reference-link definitions
`[id]: dest` unless they fall out of the same helper cheaply.

**Exit:** LSP test: `[x](/do` offers a `/docs/…` route from the fixture
site; `[x](Foo.` offers `Foo.rocdown`. `cargo test -p rocci-rocdown --test lsp`.
`cargo test -p rocci-lsp` (trigger list must not break Rocci tests).

### 4. Public contract note

**Bound:** One short paragraph in `crates/rocci-rocdown/README.md` and
`docs/rocdown/language.rocdown` (or `pages.rocdown`) that the language
server completes wiki targets and internal destinations from the site or
sibling index. No new language syntax.

**Out of bound:** Editor marketplace copy, VS Code snippet files.

**Exit:** Docs mention completion. `cargo test -p rocci-rocdown --test lsp`.
`cargo fmt --all -- --check`.

## Validation

- `cargo test -p rocci-rocdown --test lsp`
- `cargo test -p rocci-rocdown-lsp`
- `cargo test -p rocci-lsp` after trigger-character changes
- `cargo fmt --all -- --check`

Do not log complete until CI and Knowledge succeed on the revision.

[^research]: Incomplete wiki is not AST; empty Markdown completion; trigger gap.
[^rocdown-lsp]: Completion order and `compile_text` page index.
[^lsp-core]: Triggers currently `<` and `@`.
[^links-rs]: Stem/filename compile resolve versus catalog wiki keys.
[^wiki-catalog]: Title/id/stem `wiki_target` and ambiguity.
[^site-index]: `rocdown.toml` workspace cache including mounts and peers.
[^lsp-tests]: Existing kind/field/interpolation coverage only.
[^readme]: Wiki and page-link product contract.
[^lang-ref]: Published routes versus wiki ids.
[^tooling-arch]: Rocdown analyzer vs generic `rocci-lsp`.
[^language-plan]: Document links and workspace index remain umbrella leftovers.
[^language-dev]: Parser/LSP test expectations for language work.
