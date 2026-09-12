# Template preparation experiment

An isolated experiment in typed compile-time template preparation. The
canonical findings and limits are in
[`knowledge/research/rocci/compile-time-template-preparation.md`](../../knowledge/research/rocci/compile-time-template-preparation.md).
This directory is not a workspace member, a product library, or a new Rocci
grammar. It uses the existing Mustache experiment as its data-template syntax.

`TypedTemplate.roc` returns a renderer whose input type shares the sample's
type variable. `Card.rocci` is the ordinary Rocci comparison fixture;
`card.mustache.html` expresses the same small HTML shape with data paths,
a boolean section, and a list section.

## Run

Use the repository's pinned `roc`, Python 3.11+, Cargo, and, for optional
allocation measurements, the macOS C compiler. Run from the repository root:

```sh
python3 roc/template-preparation-experiment/run.py \
  --self-test \
  --output /tmp/template-preparation-harness-faults.json

python3 roc/template-preparation-experiment/run.py \
  --output /tmp/template-preparation-results.json

python3 roc/template-preparation-experiment/run.py \
  --bench --allocations \
  --output /tmp/template-preparation-benchmark.json
```

The runner downloads four upstream source files into a fresh temporary
directory at commit `e13a34b63ee466a03588e1e3cba85fcb4d045f71` and verifies
their SHA-256 hashes. Upstream sources are not vendored here. With those four
files already available, avoid downloading with `--engine-dir DIRECTORY`.
Basic-cli 0.22.0 is pinned in the runner; Roc may download its platform archive
on the first benchmark build. Roc execution needs shared-memory access, which
some sandboxes deny.

`--work-dir DIRECTORY` preserves generated sources at a chosen **new**
directory. Otherwise the runner allocates and prints a temporary directory.
It retains the directory for inspection. `--repetitions N` changes the default
5,000 runtime renders per timing sample. Each subprocess has a timeout; timed-out
children are killed with their process group.

`--self-test` runs only the harness fault probes: absent backend, same-length
wrong output, wrong expected error, corrupted source hash, a controlled
timed-out child, and git porcelain parsing of dirty tracked paths. A full
experiment run also executes those probes first.

The runner writes a complete receipt even on exception or failed builds. Exit 0
means `harness_ok` and `type_contract_ok` are true, and, with `--bench`, that
named HTML findings match expectations. Inspect `html_compatible` separately:
exit 0 still includes the known failing carriage-return attribute case.
`html_compatible` is not true merely because that mismatch is expected.

Render cases are named. Summaries are `harness_ok`, `type_contract_ok`, and
`html_compatible`. Compiler work records `--no-cache` and warm incremental
checks/builds, and upstream tests record a default run plus `roc test --no-cache`.
Invalid UTF-8 from these valid-input HTML programs is an error. Timed renderer
loops still use a length checksum; an untimed `digest` mode hashes the same
dynamic inputs outside that loop.

## Measurements

`--bench` builds `rocci-template`, inspects the fixture's AST and maps, and
generates three basic-cli apps: prepared Mustache, generated Rocci with the
string Html runtime, and generated Rocci with the product node Html runtime.
The last uses unchanged platform Html/Attribute code and changes only the
wrapper's import paths to local copies. No HTTP server or product host is used.

The Mustache file's terminal newline is removed when staged because Rocci
drops formatting whitespace outside its root element. No rendered-output
normalization is used for byte comparisons. A small HTML event parser checks
this well-formed fixture against an independent escaped reference. It applies
HTML CR/CRLF preprocessing before entity decoding; it is not a general browser
conformance test.

Compiler checks use `--no-cache` three times, then warm `roc check` three times.
Optimized builds use `--no-cache --opt=speed` once per backend, then a warm
`--opt=speed` build. Runtime timings are medians of three uninstrumented
processes, each consuming actual argv input and rendering a varying
title/boolean 5,000 times. Every repetition's checksum is checked, not only
the first sample. Times include process startup and common context
construction. They are not isolated renderer latency, especially for the
smallest case.

`--allocations` optionally builds `allocations.c` as a temporary dylib and
loads it only into the measured child processes. It counts intercepted libc
`malloc`, `calloc`, `realloc`, and `free` calls. Per-render counts subtract a
zero-render run; double-length runs check allocation-call scaling. Every
instrumented output is checked. These are not exhaustive Roc allocations or
peak live memory. Requested bytes include reallocation requests. Instrumented
processes are excluded from timing data.

Receipts hash the runner, adapter, fixtures, staged Html/Attribute modules,
generated app sources, and the local basic-cli platform archive when present.
They also record OS, architecture, CPU, compiler flags, dirty tracked files,
and untracked experimental inputs.

Preserve the September 12 receipt. Write a new receipt for each later revision
under `knowledge/research/rocci/`.

For local source checks:

```sh
roc fmt --check roc/template-preparation-experiment/TypedTemplate.roc
python3 -c 'import ast, pathlib; ast.parse(pathlib.Path("roc/template-preparation-experiment/run.py").read_text())'
python3 roc/template-preparation-experiment/run.py \
  --self-test --output /tmp/template-preparation-harness-faults.json
```
