# okf

Portable, UI-neutral engine for Open Knowledge Format (OKF) bundles.

`okf` provides deterministic parsing, schema validation, graph resolution, lexical search, retrieval benchmarks, and artifact generation for OKF knowledge repositories.

## Scope & Dependencies

`okf` is completely UI-neutral and has **zero dependencies on any other Rocci crate**. It can be consumed by third-party Rust applications or external tools without pulling in template lowering, desktop runtime, or Roc compiler dependencies.

Dependencies:
- `comrak`: Inert CommonMark markdown body parsing.
- `yaml-rust`: YAML frontmatter extraction with lossless preservation of custom keys.
- `serde`, `serde_json`, `toml`: Data serialization and benchmark parsing.
- `sha2`: Cryptographic digest calculations.
- `thiserror`, `anyhow`: Standard error handling.

## Core Features

- **Multi-Profile Validation**: `Profile::Base` (portable OKF specification) and `Profile::Rocci` (strict evidence, verification, and owners).
- **Load timings**: `load_timed` returns ordinary `Duration` breakdowns (`discover`, `parse`, `graph`, and `provenance` when git provenance runs) beside the `Bundle`. `LoadOptions` selects the profile and whether provenance runs. `ParseCache` reuses unchanged documents across loads, including from a caller-provided directory via `load_dir` / `save_dir`. `okf` does not depend on CLI snapshot types and does not choose `~/.rocci`.
- **Graph Resolution**: Strict and fuzzy concept ID matching, fragment checking, and directed edge construction. Authored `/path.md` links are bundle-root; `article_html` rewrites in-bundle Markdown hrefs to published `/{id}/` routes while `concept.links` keep the source URLs. `okf:` hrefs are classified like `mailto:` (not intra-bundle paths, not OKF3001).
- **Search & Chunking**: Semantic search indexing by metadata and headings with BM25/lexical matching.
- **Retrieval Benchmarking**: Automated evaluation against test questions with hit rate and MRR metrics.
- **Deterministic Build**: Emits `catalog.json`, `search.json`, `validation.json`, and `llms.txt`.
- **Preview path resolution**: Resolves a bundle directory, root `index.md`, or concept `.md` file to a bundle root and open URL without depending on Rocci.
