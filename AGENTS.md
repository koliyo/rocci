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
| Markdown, Rocdown declarations, site catalog, and generator | `crates/rocci-rocdown` |
| Shared runtime configuration | `crates/rocci-core` |
| Application documentation staging | `crates/rocci-docs` |
| CLI template build, run, preview, or bundle behavior | `crates/rocci-cli` and `crates/rocci-desktop` |
| CLI/desktop host behavior for the project browser | `crates/rocci-browser` |
| CLI document and site build, run, check, or test behavior | `crates/rocci-rocdown-cli` |
| Portable OKF parsing, validation, search, and artifacts | `crates/okf` |
| Knowledge review CLI, desktop preview, and bundle tools | `crates/rocci-okf` |
| Shared UI primitives, view records, and component templates | `crates/rocci-ui` |
| Documentation site chrome | `crates/rocci-rocdown/templates/RocdownTheme.rocci` |
| Editor behavior | `crates/rocci-lsp`, `crates/rocci-rocdown-lsp`, `editors/vscode`, or `editors/zed` |

- Preserve the boundary where Rocdown owns static catalog and article work in
  Rust while the Rocdown theme owns the visible documentation shell.
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
- Ensure all cursor and token scanner loops enforce monotonic forward progress
  (`cur.pos > before` or `cur.bump()`) on every path to guarantee termination
  on malformed, unclosed, or multiline inputs.
- When behavior changes, update the relevant public Rocdown reference and the
  owning crate README. Mark planned behavior as planned.
- Treat `dist/` and other generated output as derived artifacts, not sources of
  truth.
- When adding a workspace member in the root `Cargo.toml`, classify it in the
  same change in `tools/rocci-ops/src/rocci_ops/workspace_deps.py` under the matching
  `CLASSES` set. CI runs that checker in the lint job.

## Validate proportionally

- Run the narrowest relevant crate tests while iterating.
- When running commands asynchronously, inspect running background tasks with
  `manage_task` and terminate stuck or superseded tasks (`manage_task` with
  `Action: 'kill'`) before launching subsequent runs.
- Run `cargo fmt --all -- --check` for Rust changes.
- Run `cargo test --workspace` for cross-cutting changes or before handing off a
  change that affects multiple crates. Some end-to-end tests use Roc only when
  it is available; use `ROCCI_REQUIRE_ROC=1` only where Roc is required.
- Default test suites (`cargo test`, `cargo test -p <pkg>`, `cargo test --workspace`)
  are structured for sub-second execution (<2s).
- Intensive stress tests, exhaustive property fuzzing, and latency benchmarks
  are gated behind `#[ignore]`. Run them on demand:
  - Deep invariant fuzzing: `cargo test -p rocci-lsp --test fuzz_invariants -- --nocapture --ignored`
  - Release latency benchmarks: `cargo test -p rocci-lsp --test perf --release -- --nocapture --ignored`
- For syntax changes, inspect the corresponding `test/AllSyntax.rocci`
  fixture with `cargo run -q -p rocci-cli -- inspect --ast test/AllSyntax.rocci`
  or `test/AllSyntax.rocdown` with
  `cargo run -q -p rocci-rocdown-cli -- inspect ast test/AllSyntax.rocdown`.
- For documentation-site changes, run `cargo run -q -p rocci-rocdown-cli -- build docs`
  and inspect the generated result when layout or navigation changed.
- For knowledge changes, run
  `cargo run -q -p rocci-okf -- check knowledge --profile rocci` and
  report lifecycle or provenance warnings separately from errors.
- Test runtime changes through the same HTTP origin used by the webview. Failed
  static builds must preserve the previous output tree.

## Use specialized workflows when available

- Repository-scoped skills live under `.agents/skills`. Use a matching skill
  when the task invokes one or clearly matches its description.
- Author `.rocci`, `.rocdown`, and Roc used from those files with
  `rocci-author`. Change the languages themselves with `rocci-language-dev`.
  Fit Roc, Datastar, Rocci, Rocdown, Markdown, HTML, and CSS together with
  `rocci-stack` (Datastar is the browser transport; do not put that policy in
  the parser).
- Keep workflow detail in focused skills and canonical domain facts in the
  repository documentation or knowledge bundle; do not duplicate them here.
