#!/usr/bin/env bash
# Run GitHub Actions CI jobs on this machine.
# Skips the ubuntu/macos test matrix and release cross-platform builds.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

ALL_JOBS="lint test fixtures-and-docs editors knowledge"
keep_going=0
list_only=0
requested_jobs=""

if [ -t 1 ]; then
  GREEN="$(printf '\033[32m')"
  RED="$(printf '\033[31m')"
  YELLOW="$(printf '\033[33m')"
  BOLD="$(printf '\033[1m')"
  RESET="$(printf '\033[0m')"
  export CARGO_TERM_COLOR=always
else
  GREEN=""
  RED=""
  YELLOW=""
  BOLD=""
  RESET=""
fi

usage() {
  cat <<'EOF'
Usage: scripts/ci-local.sh [options] [job ...]

Run the GitHub Actions validation jobs on this OS. Does not run the
ubuntu/macos test matrix or release.yml cross-platform binary builds.

Jobs (default: all):
  lint                 workspace deps, rustfmt, clippy
  test                 cargo test --workspace and --doc
  fixtures-and-docs    AST fixture inspect and docs check
  editors              VS Code lint/compile/tests and Zed WASM check
  knowledge            OKF tests, bundle check, deterministic build

Options:
  -h, --help           show this help
  -l, --list           list jobs
  -k, --keep-going     continue after a job fails
EOF
}

is_known_job() {
  case " $ALL_JOBS " in
    *" $1 "*) return 0 ;;
    *) return 1 ;;
  esac
}

while [ $# -gt 0 ]; do
  case "$1" in
    -h|--help)
      usage
      exit 0
      ;;
    -l|--list)
      list_only=1
      shift
      ;;
    -k|--keep-going)
      keep_going=1
      shift
      ;;
    --)
      shift
      break
      ;;
    -*)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
    *)
      if ! is_known_job "$1"; then
        echo "Unknown job: $1" >&2
        echo "Known jobs: $ALL_JOBS" >&2
        exit 2
      fi
      requested_jobs="${requested_jobs:+$requested_jobs }$1"
      shift
      ;;
  esac
done

if [ "$list_only" -eq 1 ]; then
  for job in $ALL_JOBS; do
    echo "$job"
  done
  exit 0
fi

jobs_to_run=${requested_jobs:-$ALL_JOBS}

fmt_duration() {
  local s=$1
  if [ "$s" -ge 60 ]; then
    printf '%dm%02ds' $((s / 60)) $((s % 60))
  else
    printf '%ds' "$s"
  fi
}

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command: $1" >&2
    exit 1
  fi
}

ensure_rust_components() {
  if command -v rustup >/dev/null 2>&1; then
    rustup component add rustfmt clippy
  fi
}

ensure_wasm_targets() {
  if command -v rustup >/dev/null 2>&1; then
    rustup target add wasm32-wasip1 wasm32-wasip2
  fi
}

job_lint() {
  require_cmd python3
  require_cmd cargo
  ensure_rust_components
  python3 scripts/check-workspace-deps.py
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets -- -D warnings
}

job_test() {
  require_cmd cargo
  cargo test --workspace
  cargo test --workspace --doc
}

job_fixtures_and_docs() {
  require_cmd cargo
  cargo run -q -p rocci-cli -- inspect --ast test/AllSyntax.rocci
  cargo run -q -p rocci-rocdown-cli -- inspect ast test/AllSyntax.rocdown
  cargo run -q -p rocci-cli -- inspect --ast test/EmbeddedLanguages.rocci
  cargo run -q -p rocci-rocdown-cli -- inspect ast test/EmbeddedLanguages.rocdown
  cargo run -q -p rocci-rocdown-cli -- check docs
}

job_editors() {
  require_cmd cargo
  require_cmd npm
  ensure_wasm_targets
  cargo build -p rocci-rocdown-lsp
  npm --prefix editors/vscode ci
  npm --prefix editors/vscode run lint
  npm --prefix editors/vscode run compile
  npm --prefix editors/vscode run vscode:prepublish
  npm --prefix editors/vscode test
  cargo check --manifest-path editors/zed/Cargo.toml --target wasm32-wasip1
  cargo check --manifest-path editors/zed/Cargo.toml
  ./scripts/test-zed-extension.sh
}

job_knowledge() {
  require_cmd cargo
  cargo test -p okf
  cargo test -p rocci-okf
  mkdir -p target/knowledge-ci
  cargo run -q -p rocci-okf -- check knowledge --profile rocci --format json > target/knowledge-ci/validation.json
  cargo run -q -p rocci-okf -- inspect --profile rocci graph knowledge > target/knowledge-ci/graph.json
  cargo run -q -p rocci-okf -- benchmark knowledge/retrieval-benchmark.toml knowledge --profile rocci > target/knowledge-ci/retrieval.json
  cargo run -q -p rocci-okf -- build knowledge --output target/knowledge-ci/build-a --profile rocci
  cargo run -q -p rocci-okf -- build knowledge --output target/knowledge-ci/build-b --profile rocci
  diff -qr target/knowledge-ci/build-a target/knowledge-ci/build-b
}

run_named_job() {
  case "$1" in
    lint) job_lint ;;
    test) job_test ;;
    fixtures-and-docs) job_fixtures_and_docs ;;
    editors) job_editors ;;
    knowledge) job_knowledge ;;
    *)
      echo "internal error: unknown job $1" >&2
      return 1
      ;;
  esac
}

passed=""
failed=""
SECONDS=0

echo "${BOLD}CI local${RESET} on $(uname -s) $(uname -m)"
echo "Jobs: $jobs_to_run"
echo "Skipping: ubuntu/macos test matrix, release cross-platform builds"
echo

for job in $jobs_to_run; do
  echo "${BOLD}==> ${job}${RESET}"
  job_start=$SECONDS
  set +e
  run_named_job "$job"
  status=$?
  set -e
  elapsed=$((SECONDS - job_start))
  duration="$(fmt_duration "$elapsed")"
  if [ "$status" -eq 0 ]; then
    echo "${GREEN}✓ ${job}${RESET} (${duration})"
    echo
    passed="${passed:+$passed }$job"
  else
    echo "${RED}✗ ${job}${RESET} (${duration})"
    echo
    failed="${failed:+$failed }$job"
    if [ "$keep_going" -eq 0 ]; then
      break
    fi
  fi
done

total="$(fmt_duration "$SECONDS")"
pass_count=0
fail_count=0
for _ in $passed; do
  pass_count=$((pass_count + 1))
done
for _ in $failed; do
  fail_count=$((fail_count + 1))
done

if [ -n "$failed" ]; then
  echo "${RED}${BOLD}CI local: ${pass_count} passed, ${fail_count} failed in ${total}${RESET}"
  echo "${YELLOW}Failed:${RESET} $failed"
  exit 1
fi

echo "${GREEN}${BOLD}CI local: ${pass_count} passed in ${total}${RESET}"
