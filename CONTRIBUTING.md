# Contributing to Rocci

Rocci is experimental preview software. **We do not accept pull requests at
this point.** That may change in the near future; until then, do not open a PR
expecting review or a merge. Issues for bugs and docs remain useful. Security
reports go to [SECURITY.md](SECURITY.md).

Read [AGENTS.md](AGENTS.md) if you are running the tree locally or writing as
the maintainer.

## Getting started

1. Clone `https://github.com/koliyo/rocci` and work on a named branch (not `main`).
2. Install a current Rust toolchain (1.85+) plus the platform prerequisites for
   Wry, and put `roc` and `cargo` on `PATH`. See [docs/install.rocdown](docs/install.rocdown)
   and the root [README.md](README.md).
3. From the repository root:

```sh
cargo test --workspace
cargo fmt --all -- --check
```

`cargo test --workspace` is the offline crate suite: Roc on `PATH` does not
enable generated-app builds or HTTP smokes. Set `ROCCI_REQUIRE_ROC=1` for
that lane (hosted Linux `roc` job, or local on-demand). `uv run rocci-ops ci`
runs the local GitHub Actions job bodies for this OS.

## Ownership

Change the owning layer, not a convenient neighbor. Full table:
[AGENTS.md](AGENTS.md).

| Change | Primary owner |
| --- | --- |
| `.rocci` grammar, lowering, or source maps | `crates/rocci-template` |
| Markdown, Rocdown declarations, site catalog, and generator | `crates/rocci-rocdown` |
| Shared runtime configuration | `crates/rocci-core` |
| Application documentation staging | `crates/rocci-docs` |
| CLI template build, run, preview, or bundle | `crates/rocci-cli` and `crates/rocci-desktop` |
| CLI document and site build, run, check, or test | `crates/rocci-rocdown-cli` |
| Portable OKF parsing, validation, search, artifacts, and knowledge CLI | [okmate](https://github.com/koliyo/okmate) |
| Canonical knowledge bundle | `knowledge/` |
| Shared UI primitives, view records, and component templates | `crates/rocci-ui` |
| Documentation site chrome (Rocdown theme) | `crates/rocci-rocdown/templates/RocdownTheme.rocci` |
| rocci.dev document shell | `site/theme/` |
| Editor behavior | `crates/rocci-lsp`, `crates/rocci-rocdown-lsp`, `editors/vscode`, or `editors/zed` |
| Syntax highlighting grammars | `crates/rocci-highlight` |

Keep `knowledge/**/*.md` inert Markdown with OKF YAML. Do not add Rocdown or
executable declarations to canonical knowledge records.

## Code conventions

- Keep parser and token-scanner loops monotonic (`cur.pos > before` or
  `cur.bump()`) on every path.
- Add tests at the lowest owning boundary. Parser tests should not require a
  server; catalog tests should not require Roc.
- Run `cargo fmt --all -- --check` before committing on a maintainer branch.
- Prefer crate READMEs and `docs/` for public contracts. `archive/reports/` is
  historical evidence, not shipped behavior.

## Pull requests and CI

External pull requests are not accepted in this preview. If that policy
changes, it will be stated here.

Maintainer and Dependabot PRs do not start GitHub Actions automatically.
After review, a maintainer comments `/ci` or `/CI` to run hosted CI and
Knowledge on the PR head. Dependabot PRs need the same `/ci` comment.

`/ci-local` and `/cl-local` currently queue the same hosted jobs. Self-hosted
CI runners are disabled.

## Conduct, security, and support

- [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)
- [SECURITY.md](SECURITY.md) for vulnerability reports
- [SUPPORT.md](SUPPORT.md) for usage questions
- Public feedback: [https://github.com/koliyo/rocci/issues](https://github.com/koliyo/rocci/issues)
- [GOVERNANCE.md](GOVERNANCE.md)

The public site page is [Contributing](https://rocci.dev/project/contributing/).
