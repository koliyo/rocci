#!/usr/bin/env bash
# Compare Roc roc/rocci-ops/app.roc to Python uv rocci-ops on the four
# surfaces Phase 7 requires. Python is the oracle. Do not install a
# colliding rocci-ops binary. From the repo root:
#   ROC=roc ./roc/rocci-ops/parity.sh
set -euo pipefail

root="$(git rev-parse --show-toplevel)"
cd "$root"
ROC="${ROC:-roc}"
APP="$root/roc/rocci-ops/app.roc"
PY=(uv run --no-dev rocci-ops)

fail=0
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

run_pair() {
  local name="$1"
  shift
  local py_out="$tmp/${name}.py.out"
  local roc_out="$tmp/${name}.roc.out"
  local py_err="$tmp/${name}.py.err"
  local roc_err="$tmp/${name}.roc.err"
  set +e
  "${PY[@]}" "$@" >"$py_out" 2>"$py_err"
  local py_code=$?
  "$ROC" "$APP" -- "$@" >"$roc_out" 2>"$roc_err"
  local roc_code=$?
  set -e
  if ! diff -u "$py_out" "$roc_out"; then
    echo "parity: stdout mismatch for: $*" >&2
    fail=1
  fi
  if ! diff -u "$py_err" "$roc_err"; then
    echo "parity: stderr mismatch for: $*" >&2
    fail=1
  fi
  if [ "$py_code" -ne "$roc_code" ]; then
    echo "parity: exit $roc_code != python $py_code for: $*" >&2
    fail=1
  fi
}

run_pair help -h
run_pair check_help check -h
run_pair ci_list ci --list
run_pair check_deps check deps

if [ "$fail" -ne 0 ]; then
  echo "parity: FAIL" >&2
  exit 1
fi
echo "parity: OK (-h, check -h, ci --list, check deps)"
