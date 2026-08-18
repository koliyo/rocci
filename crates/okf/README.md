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
- **Graph Resolution**: Strict and fuzzy concept ID matching, fragment checking, and directed edge construction. Authored `/path.md` links are bundle-root; `article_html` rewrites in-bundle Markdown hrefs to published `/{id}/` routes while `concept.links` keep the source URLs.
- **Search & Chunking**: Semantic search indexing by metadata and headings with BM25/lexical matching.
- **Retrieval Benchmarking**: Automated evaluation against test questions with hit rate and MRR metrics.
- **Deterministic Build**: Emits `catalog.json`, `search.json`, `validation.json`, and `llms.txt`.
- **Preview path resolution**: Resolves a bundle directory, root `index.md`, or concept `.md` file to a bundle root and open URL without depending on Rocci.
