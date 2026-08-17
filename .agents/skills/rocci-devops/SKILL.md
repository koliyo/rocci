---
name: rocci-devops
description: Inspect, monitor, trigger, diagnose, and fix GitHub Actions CI/CD workflows, release pipelines, and repository automation using the `gh` CLI and local validation tools. Use for checking CI status, triaging failing workflow runs, inspecting job logs and annotations, dispatching workflows, downloading CI artifacts, and reproducing and fixing CI/CD failures.
---

# Rocci DevOps

Inspect, triage, reproduce, fix, and verify GitHub Actions CI/CD workflows across
the Rocci repository using GitHub CLI (`gh`) and local verification tools.

## Establish context

1. Work from the repository root and inspect `git status --short` before
   drawing CI provenance conclusions or editing code. Preserve unrelated
   work.
2. Understand the repository's GitHub Actions workflows in `.github/workflows/`:
   - `ci.yml`: Main validation pipeline running on every push/PR:
     - `lint`: Rust formatting (`cargo fmt`) and clippy checks (`cargo clippy -D warnings`).
     - `test`: Cross-platform matrix unit/integration/doc tests on `macos-latest` and `ubuntu-latest`.
     - `fixtures-and-docs`: AST inspection fixtures (`inspect --ast`) and Rocdown documentation check (`check docs`).
     - `editors`: VS Code extension lint/compilation/packaging and Zed WebAssembly WASI check.
   - `knowledge.yml`: Open Knowledge Format (OKF) validation, graph integrity, retrieval benchmarks, and deterministic build diffs.
   - `release.yml`: Multi-platform binary builds, CI check gating (`ci-gate`), artifact packaging, and GitHub release creation.
3. Note that `gh` commands communicate with `https://api.github.com`. When running
   in sandboxed environments, run `gh` with unsandboxed execution permissions
   (e.g., `BypassSandbox: true`).

## Inspect and monitor CI runs

Use `gh` to query workflows, runs, jobs, logs, and artifacts:

### List runs and check statuses

```sh
# List recent runs across all workflows
gh run list --limit 5

# List recent runs for the main CI workflow
gh run list --workflow ci.yml --limit 5

# List runs for a specific branch or commit
gh run list --branch main --limit 5
gh run list --commit $(git rev-parse HEAD)
```

### Inspect a run and its failures

```sh
# View high-level run summary, duration, and job status matrix
gh run view RUN_ID

# View failed step logs across all failing jobs
gh run view RUN_ID --log-failed

# View full log for a specific job
gh run view RUN_ID --job JOB_ID --log

# View failed logs for a specific job
gh run view RUN_ID --job JOB_ID --log-failed
```

### Watch, rerun, or dispatch workflows

```sh
# Watch an active workflow run until it finishes
gh run watch RUN_ID

# Check status of pull request checks
gh pr checks

# Rerun only failed jobs from a run
gh run rerun RUN_ID --failed

# Rerun all jobs in a run
gh run rerun RUN_ID

# Trigger a workflow dispatch
gh workflow run ci.yml --ref main

# Download artifacts produced by a run
gh run download RUN_ID --dir target/ci-artifacts
```

## Triage workflow failures

Categorize the root cause by examining the failing job and log output:

| Job | Common failure modes | Reproduction & fix strategy |
|---|---|---|
| `lint` | Unformatted code, clippy warnings | Run `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings`. Fix warnings or format with `cargo fmt --all`. |
| `test` (macOS / Ubuntu) | Logic regressions, platform differences, socket permissions, timing/budget assertions | Run `cargo test -p CRATE` for the failing test. Ensure timing assertions account for unoptimized debug mode on shared CI VMs (`cfg!(debug_assertions)`). Ensure stress/fuzz iterations scale appropriately in debug mode. |
| `fixtures-and-docs` | AST snapshot drift, broken markdown links, missing frontmatter | Inspect syntax with `cargo run -q -p rocci-cli -- inspect --ast test/AllSyntax.rocci` (and `.rocdown`, `EmbeddedLanguages.*`). Check documentation with `cargo run -q -p rocci-rocdown-cli -- check docs`. |
| `editors` | TypeScript/ESLint errors, VS Code packaging issues, Zed Wasm build errors | Run `npm --prefix editors/vscode ci && npm --prefix editors/vscode run lint && npm --prefix editors/vscode run compile` and `cargo check --manifest-path editors/zed/Cargo.toml --target wasm32-wasip1`. |
| `knowledge` | OKF schema errors, broken cross-references, graph cycles, benchmark regressions | Run `cargo run -q -p rocci-rocdown-cli -- knowledge check knowledge --profile rocci` and `cargo test -p rocci-rocdown okf::`. |

### Timing and benchmark budgets in CI

- CI runners (GitHub Actions `ubuntu-latest` and `macos-latest`) are shared virtual
  machines with variable CPU allocation and lower single-thread throughput than
  local development machines.
- Unoptimized debug-mode builds (`cfg!(debug_assertions)`) have substantial function
  call overhead, no vectorization, no inlining, and bounds checking on every slice.
- When asserting performance budgets in tests:
  - Provide generous thresholds for debug builds (e.g. 150–250ms for small fixtures,
    1500ms for 10k-line documents) to prevent flaky CI failures while still catching
    accidental quadratic complexity or infinite loops.
  - Reserve tight budgets (e.g. <30ms, <150ms) for release builds (`--release`).
  - Scale deterministic fuzz iterations in debug mode (e.g., 1,000 in debug vs
    5,000 in release) so the suite completes in seconds rather than minutes.

## Reproduce and fix locally

1. Run the narrowest relevant test first before running workspace tests:
   ```sh
   cargo test -p CRATE --test TEST_FILE TEST_NAME -- --exact
   ```
2. Apply the focused fix in the owning crate or test file.
3. Validate formatting and workspace lints:
   ```sh
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   ```
4. Run workspace tests locally:
   ```sh
   cargo test --workspace
   cargo test --workspace --doc
   ```
5. If public syntax or CLI contracts changed, inspect AST fixtures and docs:
   ```sh
   cargo run -q -p rocci-cli -- inspect --ast test/AllSyntax.rocci
   cargo run -q -p rocci-rocdown-cli -- check docs
   ```

## Trigger and verify CI

1. Review staged changes with `git diff` and commit with a concise conventional commit.
2. Push to GitHub or rerun failed jobs:
   ```sh
   git push origin BRANCH
   # Or rerun failed jobs on an existing run:
   gh run rerun RUN_ID --failed
   ```
3. Monitor the new run to completion:
   ```sh
   gh run watch RUN_ID
   ```
4. Verify all matrix jobs and required checks complete with green `✓` status.

## Report results

- Identify the failed run ID, job name, and specific error or panic message.
- Explain the root cause (e.g., timing threshold under debug mode, lint violation, platform dependency).
- Detail the fix applied and local validation results (`cargo fmt`, `cargo clippy`, `cargo test`).
- Report the resulting CI workflow run ID, duration, and status.
