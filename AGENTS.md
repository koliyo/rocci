# Rocci agent instructions

## Start with repository evidence

- Inspect `git status --short` before editing. Preserve unrelated tracked and
  untracked work; do not clean or rewrite it to simplify a task.
- Read the root `README.md` and the README for the owning crate before changing
  a public contract.
- For architecture, decisions, implementation status, and known limitations,
  start at `knowledge/index.md` and read the relevant knowledge record.
- Verify mutable implementation claims against the code, tests, or published
  documentation cited by the record. Root reports and research are evidence,
  not automatically descriptions of shipped behavior.
- Keep implemented, approved, exploratory, and historical claims distinct.

## Work in the owning layer

| Change | Primary owner |
| --- | --- |
| `.rocci` grammar, lowering, or source maps | `crates/rocci-template` |
| Markdown or Rocdown declarations | `crates/rocci-rocdown` |
| Shared runtime configuration | `crates/rocci-core` |
| CLI build, run, preview, or bundle behavior | `crates/rocci-cli` and `crates/rocci-wry` |
| Static documentation catalog, routes, and outputs | `crates/rocs` and `crates/rocs-cli` |
| Documentation site chrome | `crates/rocs/templates/RocsTheme.rocci` |
| Editor behavior | `crates/rocci-lsp`, `editors/vscode`, or `editors/zed` |

- Preserve the boundary where Rocs owns static catalog and article work in
  Rust while the Rocci theme owns the visible documentation shell.
- Do not interpret Rocci templates in Rust merely to avoid compiling a theme.
- Do not encode static documentation prose as Roc constructors merely to keep
  catalog logic in Roc.
- Keep `knowledge/**/*.md` inert Markdown with OKF YAML. Do not add Rocdown or
  executable declarations to canonical knowledge records.

## Make focused changes

- Keep a change traceable from source syntax or configuration to its generated
  Roc, HTML, runtime behavior, or diagnostic.
- Add tests at the lowest owning boundary. Parser and lowering tests should not
  require a server; catalog and route tests should not require Roc.
- When behavior changes, update the relevant public Rocdown reference and the
  owning crate README. Mark planned behavior as planned.
- Treat `dist/` and other generated output as derived artifacts, not sources of
  truth.

## Validate proportionally

- Run the narrowest relevant crate tests while iterating.
- Run `cargo fmt --all -- --check` for Rust changes.
- Run `cargo test --workspace` for cross-cutting changes or before handing off a
  change that affects multiple crates. Some end-to-end tests use Roc only when
  it is available; use `ROCCI_REQUIRE_ROC=1` only where Roc is required.
- For syntax changes, inspect the corresponding `test/AllSyntax.rocci` or
  `test/AllSyntax.rocdown` fixture with
  `cargo run -q -p rocci-cli -- inspect --ast PATH`.
- For documentation-site changes, run `cargo run -q -p rocs-cli -- build docs`
  and inspect the generated result when layout or navigation changed.
- For knowledge changes, run
  `cargo run -q -p rocs-cli -- knowledge check knowledge --profile rocci` and
  report lifecycle or provenance warnings separately from errors.
- Test runtime changes through the same HTTP origin used by the webview. Failed
  static builds must preserve the previous output tree.

## Use specialized workflows when available

- Repository-scoped skills live under `.agents/skills`. Use a matching skill
  when the task invokes one or clearly matches its description.
- Keep workflow detail in focused skills and canonical domain facts in the
  repository documentation or knowledge bundle; do not duplicate them here.
