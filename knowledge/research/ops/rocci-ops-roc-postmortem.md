---
type: Research Report
title: Roc rocci-ops post-mortem
description: "After Phases 0–7 on nightly-2026-08-23-fb208ba, the CLI pin and x64musl origin binary worked as hoped. Unexpected cost was reserved words, Ok/Err union merge, and one-union-per-file isolation. Python uv rocci-ops stays the operator CLI."
tags: [domain/ops, domain/rocci, integration/roc, concern/ci, concern/tooling, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-09-03T11:40:00Z }
stale_after: 2026-12-02
authority: exploratory
owners: [human:nils]
sources:
  - id: plan
    resource: ../../plans/ops/rocci-ops-roc.md
    title: Implementation plan and Phase 0–7 pin notes
    author: process:cursor
    last_modified: 2026-09-03
  - id: research
    resource: ./rocci-ops-roc.md
    title: Pre-implementation research; Phase 0/6/7 pins
    author: process:cursor
    last_modified: 2026-09-03
  - id: template-postmortem
    resource: ../rocci/roc-native-template-compiler-postmortem.md
    title: Same pin; type-module recursion; open-union merge; UnixBytes argv
    author: process:cursor
    last_modified: 2026-09-03
  - id: python-research
    resource: python-uv-ops-pipeline.md
    title: uv is the operator runtime; POSIX leftovers
    author: process:cursor
    last_modified: 2026-08-31
  - id: cli
    resource: ../../../rocci-ops/src/rocci_ops/cli.py
    title: Python USAGE; exit 2 for unknown
    author: process:git
    last_modified: 2026-08-31
  - id: app
    resource: ../../../roc/rocci-ops/app.roc
    title: basic-cli 0.22.0 driver; does not import Version
    author: process:cursor
    last_modified: 2026-09-03
  - id: cli-roc
    resource: ../../../roc/rocci-ops/Cli.roc
    title: parse = do_parse; usage byte-identical to Python
    author: process:cursor
    last_modified: 2026-09-03
  - id: ci-roc
    resource: ../../../roc/rocci-ops/Ci.roc
    title: JOB_NAMES; steps_for as data
    author: process:cursor
    last_modified: 2026-09-03
  - id: deps-roc
    resource: ../../../roc/rocci-ops/WorkspaceDeps.roc
    title: packages JSON key rewritten to pkgs
    author: process:cursor
    last_modified: 2026-09-03
  - id: version-roc
    resource: ../../../roc/rocci-ops/Version.roc
    title: Got/Nope/CoreSemver; not imported by app.roc
    author: process:cursor
    last_modified: 2026-09-03
  - id: docs-roc
    resource: ../../../roc/rocci-ops/DocsCoverage.roc
    title: expect identifier avoided; query field via table_get
    author: process:cursor
    last_modified: 2026-09-03
  - id: git-roc
    resource: ../../../roc/rocci-ops/Git.roc
    title: Archive/release/pr parsers; no Str.split_first
    author: process:cursor
    last_modified: 2026-09-03
  - id: local-roc
    resource: ../../../roc/rocci-ops/Local.roc
    title: One LocalReq union; darwin_ok as macos string
    author: process:cursor
    last_modified: 2026-09-03
  - id: origin-roc
    resource: ../../../roc/rocci-ops/Origin.roc
    title: Lane, health URL, compose argv, origin_publish_cmd
    author: process:cursor
    last_modified: 2026-09-03
  - id: parity-sh
    resource: ../../../roc/rocci-ops/parity.sh
    title: Phase 7 Python vs Roc diffs
    author: process:cursor
    last_modified: 2026-09-03
  - id: pin-fixture
    resource: ../../../roc/rocci-ops/fixtures/cargo-metadata-subset.json
    title: cargo-metadata subset for Encoding.Json.parse
    author: process:cursor
    last_modified: 2026-09-03
  - id: install-roc
    resource: ../../../docker/install-roc.sh
    title: nightly-2026-08-23-fb208ba
    author: process:git
    last_modified: 2026-08-25
  - id: basic-cli-cmd
    resource: https://roc-lang.github.io/basic-cli/0.22.0/Cmd/
    title: Cmd without cwd or stdout-file redirect
    author: organization:roc-lang
    last_modified: 2026-08-23
  - id: roc-tutorial
    resource: https://github.com/roc-lang/roc/blob/main/docs/mini-tutorial-new-compiler.md
    title: New-compiler var, for, expect, packages
    author: organization:roc-lang
    last_modified: 2026-08-31
---

# Roc rocci-ops post-mortem

This is exploratory evidence from executing [Roc rewrite of rocci-ops on
a parallel branch](/plans/ops/rocci-ops-roc.md) on
`nightly-2026-08-23-fb208ba`. It is **not** shipped operator behavior.
Python `uv run --no-dev rocci-ops` remains the CI, origin, and localhost
CLI.[^plan][^research][^cli][^python-research][^install-roc]

Work lives on branch `rocci-ops-roc`. Phases 0–7 are in that tree. Do
not log them complete until hosted CI and Knowledge succeed on the
revision that contains the Roc sources.[^plan]

Learnings from the [template-compiler post-mortem](/research/rocci/roc-native-template-compiler-postmortem.md)
were applied from Phase 0 (`parse = do_parse`, no `Ok`/`Err` reuse next
to `Str.split_first`, UnixBytes argv). This record is the CLI-shaped
cost of the same pin, not a repeat of the parser isolation
story.[^template-postmortem][^app]

## Outcome

The pin's `var` / `while` / `match` / `expect` / `Cmd` stack was enough
to dispatch the Python command tree, port the pure rules (workspace
deps, version bump, coverage TOML, lanes, archive names), wrap
maintainer `Cmd` sequences, and emit a Linux musl binary that prints
origin usage without Roc.[^app][^roc-tutorial][^origin-roc][^parity-sh]

On that branch:

- `roc test roc/rocci-ops/app.roc` is the app suite (it also runs
  basic-cli platform expects; hundreds of extra tests are expected).
- `roc roc/rocci-ops/app.roc -- -h` is byte-identical to Python `USAGE`.
- `./roc/rocci-ops/parity.sh` matches Python on `-h`, `check -h`,
  `ci --list`, and `check deps`.
- `Ci.roc` keeps `JOB_NAMES` in Python order and `steps_for` as data
  (`argv`, `cwd`, `stdout_path`, `extra_env`).[^ci-roc][^parity-sh]
- `roc build --target=x64musl` from macOS arm64 emits a static ELF
  x86-64; `debian:bookworm-slim` `linux/amd64` with no Roc prints
  `origin --help`.[^research][^plan][^parity-sh][^app]

The architecture that actually shipped is **not** one file of Python
translated 1:1. Isolation rules forced by this nightly dominate the rest
of this record: `Version.roc` stays out of `app.roc`; `Local.roc` uses
one request union; Darwin is a `"macos"` string; JSON `packages` is
rewritten before parse.[^version-roc][^local-roc][^deps-roc]

## Expected on this pin (Phase 0 plus template post-mortem)

These were the point of the pin spike and the template exercise. They
surprised relative to Python, not relative to the plan after Phase
0.[^research][^template-postmortem][^roc-tutorial]

- macOS `main!` argv is `UnixBytes`. Decode with `OsStr.to_raw` and
  `Str.from_utf8_lossy`.
- `Cmd.exec_exit_code!`: `true` → `Ok(0)`, `false` → `Ok(1)`. Non-zero
  is **not** `Err` on this API.
- File IO is `Path.read_utf8!(Path.utf8(path))` / `write_utf8!`.
  Failure is `Err(Exit(1))`, not a two-field `Exit`.
- Type-module export `name = name` is infinite recursion. Use
  `parse = do_parse`. Expects in the defining file must call `do_*`.
- `var` / `while` / `break` / `return` exist. There is **no `continue`**.
- No `List.walk` / `List.contains`; use `List.fold`.
- Integer literals are `0.U64` / `0.I32`. `n.to_str()` exists.
- `//` is a comment, not integer division.
- `Cmd` has no cwd and no stdout-file redirect. Sequential
  `Env.set_cwd!` and capture-then-write are the workarounds.[^basic-cli-cmd][^app]

On macOS, `Env.temp_dir!()` may be `/var/folders/.../T/` while `pwd`
prints `/private/var/folders/.../T`. That is a path alias, not a cwd
bug.[^research]

## Unexpected compiler and runtime issues

These were **not** all in the Phase 0 pin. Several fail at **runtime**
(a crashing `expect`, or a sibling `expect` going red) with **no type
error**, same class as the template port.[^template-postmortem]

### Reserved `packages` is not a JSON field name

Roc cannot use `packages` as a record field. `Encoding.Json.parse` of
live `cargo metadata` requires rewriting the first `"packages":` key to
`"pkgs":` before parse. Extra JSON fields, including `null`, are
ignored. That is not a Rust helper.[^deps-roc][^pin-fixture][^research]

### Reserved `expect` is not a record field name

The search-queries TOML key `expect` cannot be a Roc identifier in this
dialect. `DocsCoverage` stores the value as `target` via
`table_get(..., "expect")`. Top-level `expect` tests still work; the
clash is the field name.[^docs-roc]

### Open tag unions merge; do not reuse `Ok` / `Err`

On this nightly, extra tags in the **same file** (and sometimes tags
imported into the same file) collapse into one open union. A file that
also uses `Str.split_first`'s `Ok({ before, after })` cannot safely
return `Ok("string")` for application results.[^template-postmortem][^version-roc]

`Version.roc` uses `Got` / `Nope` / `CoreNum` / `CoreSemver`. Always
destructure **both** `before` and `after`. `app.roc` does **not** import
`Version`; bump tests live in `Version.roc` so `Got`/`Nope` never meet
`decode_argv`'s `Ok`/`Err`.[^version-roc][^app]

`Git.roc` avoids `Str.split_first` and uses byte scanners plus
`CheckEmpty` / `CheckStat` instead of `Ok`/`Err` for check-run
lines.[^git-roc]

### Several request unions in one file can segfault the compiler

Splitting `BuildReq` + `InstallReq` + … in `Local.roc` segfaulted this
nightly. The working form is **one** `LocalReq` covering build, install,
package, and serve. Darwin is a **string** (`"macos"`) compared in
`darwin_ok`; `MACOS` tags from `Env.platform!` stay in `app.roc` and are
mapped before the call.[^local-roc][^app]

`Git.roc` and `Origin.roc` each keep several unions and compiled. The
Local crash is a density/import interaction, not a proof that a file may
have only one union. Prefer one union per command-family module until a
later nightly is known-good.[^local-roc][^git-roc][^origin-roc]

### `Cmd.new` versus `Cmd.new_str`

Mixed `OsStr` inference from `Cmd.new` / `args_str` failed in this app.
The working form is `Cmd.new_str` plus `arg_str` on each
argument.[^app][^basic-cli-cmd]

### Nested functions cannot reassign outer `var`

A `var $flush = |_| { $entries = List.concat(...) }` that mutated
enclosing vars was a compile error. Inline the append in the `while`.
The same pin allows `var $cur` record updates with `{ ..$cur, field: n
}` but not cross-function reassignment.[^git-roc][^template-postmortem]

### Record patterns must name or `..` every field

`match cfg { { err } => ... }` does not match a wider record. Use
`{ err, .. }`. Partial `Ok({ host: "…" })` without `url` similarly
fails when the payload has both fields.[^origin-roc]

### Interpolating a `U64` can poison a parameter as `Str`

`Stdout.line!("… ${pushed} …")` made `push_entries!`'s counter a `Str`,
so `0.U64` at the call site and `skipped + 1` no longer typechecked.
Format with `pushed.to_str()` (or a local `pstr`) and keep the counter
`U64`.[^app]

### `roc build` flags are equals-form

`--target x64musl` and `--output path` error with `no value was
supplied`. The working form is `--target=x64musl --output=path`.[^app][^research]

### Health is curl, not `Http.get_utf8!`

Python's probe is a 5s no-proxy opener plus an optional `Host` header.
This port builds `curl` argv (`--max-time 5`, `--noproxy *`, optional
`-H Host:`) rather than basic-cli `Http`. Mutating origin
publish/up/backup and live SSH stay `not implemented` so the branch
cannot deploy.[^origin-roc][^research][^plan]

## Architecture that resulted

| Module | Imports | Role |
| --- | --- | --- |
| `Cli.roc` | none | `USAGE` / `CHECK_USAGE`; `parse = do_parse` |
| `Ci.roc` | none | `JOB_NAMES`; `steps_for` as data |
| `WorkspaceDeps.roc` | none | `CLASSES`; JSON via `pkgs` |
| `Version.roc` | none | Semver and lock rewrite; **not** imported by `app.roc` |
| `DocsCoverage.roc` | none | Hand-rolled coverage / query / session TOML |
| `Local.roc` | none | One `LocalReq`; Darwin as `"macos"` |
| `Git.roc` | none | Archive keys, release parse, PR/worktree scanners |
| `Origin.roc` | none | Lane, health URLs, compose argv, publish cmd string |
| `app.roc` | Cli, Ci, DocsCoverage, Git, Local, Origin, WorkspaceDeps | `Cmd` / `Path` / `Env` edge |
| `parity.sh` | — | Python vs Roc stdout/stderr/exit |

Unlike the template package, `app.roc` **can** import several type
modules at once when tag names stay distinct (`ArchiveArgs` versus
`OriginHelp` versus `BuildUsage`). The template rule "do not import
Parse and Template into the same file" is about overlapping payload
tags and Cursor method poisoning, not a ban on a CLI driver importing
siblings.[^app][^template-postmortem][^cli-roc]

Rules that now have to stay true on this pin:[^version-roc][^local-roc][^deps-roc][^app]

1. Export `name = do_name`, never `name = name`.
2. Do not import `Version` into `app.roc` (or any file that already
   matches `Ok`/`Err` from argv, `List.get`, or `Str.split_first`).
3. Prefer one command-family tag union per module (`LocalReq`).
4. Rewrite cargo-metadata `"packages":` to `"pkgs":` before
   `Encoding.Json.parse`.
5. Do not name a record field `expect` or `packages`.
6. Use `Cmd.new_str` / `arg_str`. Map `Env.platform!` to a string
   before `Local`.

## Remaining gaps (not compiler crashes)

Recorded in the Phase 7 table; still true after the last
phase:[^plan][^research][^parity-sh]

- Mutating `origin publish|up|backup` and `deploy probe|bootstrap|push`
  SSH are `not implemented` in the app. Logic (lane, URLs, compose
  argv, `origin_publish_cmd`) is in `Origin.roc`.
- `check zed`, archive `package|wait-ci|publish`, and non-dry-run
  `release` (tag push / force-move `dev`) are out of bound or
  unfinished.
- Dual `CLASSES` copies: Python is updated first; Roc follows.
- `roc test` on the app also runs platform expects. That is noise, not
  a parity failure.
- When behavior disagrees, **Python wins**. Do not change pytest
  goldens to accommodate the port.[^cli]

## What this means for a later Roc pin

The features the research counted on (`var`, `while`, `match`, `expect`,
`Cmd`, `Encoding.Json.parse`, `roc build --target=x64musl`) are present
and usable.[^research][^roc-tutorial][^app]

The unexpected tax is the same **open-union merging** as the template
port, plus **reserved words** (`packages`, `expect`) that a mechanical
Python port hits immediately, plus **CLI flag parsing** that requires
`--flag=value` on this `roc build`.

A later nightly that (a) rejects `parse = parse` at compile time, (b)
does not merge unrelated open unions in one file, (c) allows `packages`
as a JSON field name or a distinct decode API, and (d) accepts
space-separated `--target` would let `Version` live next to `app.roc`
and would shrink `LocalReq`.

Until then, treat `roc/rocci-ops/` as a pin-specific parallel exercise,
not as evidence that replacing uv on CI or the origin is a small
port.[^plan][^python-research]

**Cutover stays a new plan.** Cmd mapping, JSON subset, TOML subset, and
the origin musl binary are **go**. Replacing `uv run --no-dev rocci-ops`
on hosted lint/test jobs, and copying this binary onto the VPS instead
of Python sources, are **no-go** until that plan exists.[^research][^plan]

## Related

- Plan (pins and remaining mismatches): [Roc rewrite of rocci-ops](/plans/ops/rocci-ops-roc.md)
- Pre-implementation research: [A Roc port of rocci-ops is a parallel exercise](/research/ops/rocci-ops-roc.md)
- Same pin, parser isolation: [Roc-native template compiler post-mortem](/research/rocci/roc-native-template-compiler-postmortem.md)

[^plan]: Phases 0–7 on `rocci-ops-roc`; Python stays operator CLI; cutover is a new plan.
[^research]: Dual implementation; Phase 0/6/7 pins; go/no-go table.
[^template-postmortem]: `parse = do_parse`; open-union merge; UnixBytes; no `continue`.
[^python-research]: uv is the operator runtime; POSIX PID 1 / install-roc / ProxyCommand stay.
[^cli]: `USAGE`; unknown command exits 2; Python wins.
[^app]: `UnixBytes` argv; `Cmd.new_str`; does not import `Version`; x64musl driver.
[^cli-roc]: `parse = do_parse`; usage matches Python `USAGE`.
[^ci-roc]: `JOB_NAMES` order; `stdout_path` as capture-then-write.
[^deps-roc]: `"packages":` → `"pkgs":`; same `CLASSES` as Python.
[^version-roc]: `Got`/`Nope`/`CoreSemver`; isolated from `app.roc`.
[^docs-roc]: TOML `expect` field read as `target`; slugify without regex.
[^git-roc]: Byte scanners; `CheckEmpty`/`CheckStat`; no `Str.split_first`.
[^local-roc]: One `LocalReq`; `darwin_ok("macos")`.
[^origin-roc]: Lane env records; curl argv; compose `--remove-orphans`.
[^parity-sh]: `parity.sh` vs `uv run --no-dev rocci-ops`.
[^pin-fixture]: Subset JSON with `workspace_members` and `packages`.
[^install-roc]: Product nightly `nightly-2026-08-23-fb208ba`.
[^basic-cli-cmd]: `exec_exit_code!`; no `Cmd` cwd or stdout-file redirect.
[^roc-tutorial]: New-compiler `var`, `for`, `expect`, packages.
