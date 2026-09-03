---
type: Research Report
title: A Roc port of rocci-ops is a parallel exercise
description: "Exploratory: rewrite rocci-ops in ordinary Roc on a parallel branch to test parity and basic-cli viability. Python plus uv stays the operator CLI until a later human cutover. Not shipped."
tags: [domain/ops, domain/rocci, integration/roc, concern/ci, concern/tooling, concern/publication]
status: draft
generated: { by: process:cursor, at: 2026-09-03T11:40:00Z }
stale_after: 2026-12-02
authority: exploratory
owners: [human:nils]
sources:
  - id: plan
    resource: ../../plans/ops/rocci-ops-roc.md
    title: Implementation plan for a Roc rocci-ops on a parallel branch
    author: process:cursor
    last_modified: 2026-09-03
  - id: python-research
    resource: python-uv-ops-pipeline.md
    title: Findings after migrating operator scripts to Python and uv
    author: process:cursor
    last_modified: 2026-08-31
  - id: python-plan
    resource: ../../plans/ops/python-uv-ops-pipeline.md
    title: Python and uv operator pipeline; Roc port was a later branch
    author: process:cursor
    last_modified: 2026-08-31
  - id: dx-plan
    resource: ../../plans/ops/rocci-ops.md
    title: Current rocci-ops command tree
    author: process:cursor
    last_modified: 2026-08-31
  - id: template-plan
    resource: ../../plans/rocci/roc-native-template-compiler.md
    title: Parallel-branch proof of concept pattern; Rust stays product
    author: process:cursor
    last_modified: 2026-08-31
  - id: cli
    resource: ../../../rocci-ops/src/rocci_ops/cli.py
    title: Console script dispatch and usage text
    author: process:git
    last_modified: 2026-08-31
  - id: ci
    resource: ../../../rocci-ops/src/rocci_ops/ci.py
    title: Job names, Step argv, stdout_path, cwd, extra_env
    author: process:git
    last_modified: 2026-08-31
  - id: deps
    resource: ../../../rocci-ops/src/rocci_ops/workspace_deps.py
    title: cargo metadata JSON plus product-boundary classes
    author: process:git
    last_modified: 2026-08-31
  - id: docs
    resource: ../../../rocci-ops/src/rocci_ops/docs_coverage.py
    title: tomllib plus heading slug regex
    author: process:git
    last_modified: 2026-08-31
  - id: version
    resource: ../../../rocci-ops/src/rocci_ops/version.py
    title: Semver bump and Cargo.toml / Cargo.lock string rewrite
    author: process:git
    last_modified: 2026-08-31
  - id: origin
    resource: ../../../rocci-ops/src/rocci_ops/origin.py
    title: Compose up, tar extract, urllib health probe
    author: process:git
    last_modified: 2026-08-31
  - id: deploy
    resource: ../../../rocci-ops/src/rocci_ops/deploy.py
    title: SSH kit copy including Python rocci-ops sources
    author: process:git
    last_modified: 2026-08-31
  - id: archive
    resource: ../../../rocci-ops/src/rocci_ops/archive.py
    title: tar.gz plus SHA-256 of release archives
    author: process:git
    last_modified: 2026-08-31
  - id: release
    resource: ../../../rocci-ops/src/rocci_ops/release.py
    title: git worktree version commit and gh wait-ci
    author: process:git
    last_modified: 2026-08-31
  - id: util
    resource: ../../../rocci-ops/src/rocci_ops/util.py
    title: subprocess.run with check=True and Darwin gate
    author: process:git
    last_modified: 2026-08-31
  - id: pyproject
    resource: ../../../rocci-ops/pyproject.toml
    title: Stdlib-only runtime; console script rocci-ops
    author: process:git
    last_modified: 2026-08-31
  - id: install-roc
    resource: ../../../docker/install-roc.sh
    title: Pinned nightly-2026-08-23-fb208ba
    author: process:git
    last_modified: 2026-08-25
  - id: basic-cli-pin
    resource: ../../../crates/rocci-rocdown/src/lib.rs
    title: BASIC_CLI_PLATFORM 0.22.0
    author: process:git
    last_modified: 2026-08-23
  - id: basic-cli-cmd
    resource: https://roc-lang.github.io/basic-cli/0.22.0/Cmd/
    title: Cmd exec, capture, env; no cwd or stdout file redirect
    author: organization:roc-lang
    last_modified: 2026-08-23
  - id: basic-cli-docs
    resource: https://roc-lang.github.io/basic-cli/0.22.0/
    title: Path, Env, Http, Sleep, Utc on basic-cli 0.22
    author: organization:roc-lang
    last_modified: 2026-08-23
  - id: encoding-json
    resource: https://www.roc-lang.org/docs/main/Encoding/
    title: Encoding.Json.parse on the pinned nightly
    author: organization:roc-lang
    last_modified: 2026-08-23
  - id: postmortem
    resource: ../rocci/roc-native-template-compiler-postmortem.md
    title: Pin dialect; UnixBytes argv; Path; Exit(1)
    author: process:cursor
    last_modified: 2026-09-03
  - id: pin-app
    resource: ../../../roc/rocci-ops/app.roc
    title: Phase 0 basic-cli pin spike; Phase 6 x64musl driver
    author: process:cursor
    last_modified: 2026-09-03
  - id: origin-roc
    resource: ../../../roc/rocci-ops/Origin.roc
    title: Lane, health URL, compose argv, origin_publish_cmd
    author: process:cursor
    last_modified: 2026-09-03
  - id: parity-sh
    resource: ../../../roc/rocci-ops/parity.sh
    title: Phase 7 Python vs Roc stdout/stderr/exit diffs
    author: process:cursor
    last_modified: 2026-09-03
  - id: pin-fixture
    resource: ../../../roc/rocci-ops/fixtures/cargo-metadata-subset.json
    title: cargo-metadata subset for Encoding.Json.parse
    author: process:cursor
    last_modified: 2026-09-03
  - id: author
    resource: ../../../.agents/skills/rocci-author/SKILL.md
    title: Ordinary Roc for helpers; .rocci is UI and HTTP apps
    author: process:git
    last_modified: 2026-08-31
  - id: impl-postmortem
    resource: ./rocci-ops-roc-postmortem.md
    title: Implementation findings after Phases 0–7
    author: process:cursor
    last_modified: 2026-09-03
---

# A Roc port of rocci-ops is a parallel exercise

## Status

Exploratory **exercise**. Nothing here is shipped. `rocci-ops` in Python
remains the operator CLI for CI, origin, and localhost.[^python-research][^cli][^pyproject]
This is not a cutover, not a replacement for `uv run --no-dev rocci-ops`,
and not a reason to put `roc` on the origin.

The same dual-implementation pattern as the Roc-native template compiler
applies: a second implementation lives on a named branch; the production
surface stays where it is until a human accepts parity and
viability.[^template-plan][^plan][^postmortem]

Phase 0 of the plan is recorded below on branch `rocci-ops-roc`. Phase 7
records a go / no-go table: the four-command harness matches Python;
origin x64musl is viable; CI uv replacement and cutover are no-go.
Python still wins. Do not treat the spike as a cutover. After Phases
0–7: [implementation post-mortem](/research/ops/rocci-ops-roc-postmortem.md).[^pin-app][^plan][^parity-sh][^impl-postmortem]

## Recommendation

Port `rocci-ops` to ordinary Roc under `roc/rocci-ops/` on git branch
`rocci-ops-roc`. Keep command names. Run it as `roc roc/rocci-ops/app.roc -- …`
or a distinctly named binary. Do not install a colliding `rocci-ops` on
`PATH`.

Python wins whenever the two CLIs disagree. POSIX stays POSIX for
container PID 1, `install-roc.sh`, and OpenSSH `ProxyCommand`. Phase 0
is recorded; later phases follow the plan.[^python-plan][^python-research][^pin-app]

## Current Python surface

One stdlib package, ~3.2k lines of implementation and ~1.9k lines of
pytest, dispatched from a single usage block:[^cli][^dx-plan][^pyproject]

- `ci` — lint, test, fixtures-and-docs, editors, knowledge, roc
- `check` — deps, docs, zed
- `build`, `install`, `package`, `site`, `serve`
- `deploy`, `origin`, `push-worktrees`, `pr-checkout`
- `promote staging|production`
- `release patch|minor|major|v*|dev`
- `archive version|package|params|wait-ci|publish`

Most commands are subprocess sequencers (`cargo`, `git`, `gh`, `docker`,
`uv`, `ssh`, `tar`, `npm`). A smaller core is pure: workspace class
sets, semver bump, coverage TOML, archive naming.[^ci][^deps][^version][^docs][^util]

## Primitive map (Python vs basic-cli 0.22)

The pin is `nightly-2026-08-23-fb208ba` plus basic-cli
0.22.0.[^install-roc][^basic-cli-pin] Host effects that the Python package
uses, and what 0.22 actually offers:

| Python | Roc 0.22 host | Gap |
| --- | --- | --- |
| `sys.argv` / argparse | `main!` argv list | argv includes the executable; macOS tags are `UnixBytes`, decode with `Str.from_utf8_lossy`[^pin-app][^postmortem] |
| `subprocess.run` inherit stdio | `Cmd.exec!` / `exec_cmd!` | Non-zero is `Err`, not `Ok(code)` unless `exec_exit_code!` (`true` → `Ok(0)`, `false` → `Ok(1)`)[^basic-cli-cmd][^pin-app] |
| `capture_output` | `Cmd.exec_output!` | Fine for `gh` / `cargo metadata` |
| `cwd=` | **No `Cmd` cwd** | `Env.set_cwd!` around a sequential step, or absolute argv[^basic-cli-docs] |
| `stdout=` file | **No redirect** | Capture then `Path.write_utf8!` (knowledge JSON is small)[^ci] |
| `os.environ` | `Env.var_str!`, `Cmd.env_str` | Available |
| `json.loads` | `Encoding.Json.parse` | Subset decode works after renaming reserved JSON key `packages` to `pkgs`; extra fields including `null` are ignored[^encoding-json][^deps][^pin-app][^pin-fixture] |
| `tomllib` | **No TOML** | Hand-roll the coverage / search-queries / first-use schemas[^docs] |
| `re` | **No regex** | Explicit scanners for slugs, semver, SHA, lock rewrite[^docs][^version] |
| `hashlib.sha256` | **No SHA-256** | Shell `shasum` / `sha256sum` for archives[^archive] |
| `tarfile` / `shutil.copytree` | Path list/create/delete; no tar | Shell `tar` and `cp` via `Cmd`[^deploy][^origin] |
| `urllib.request` | `Http.get_utf8!` | Health probe; confirm timeout and no-proxy behavior[^origin][^basic-cli-docs] |
| `time.sleep` | `Sleep.seconds!` | `wait_for_check`[^release] |
| `platform.system` | `Env.platform!` | Darwin gate[^util][^basic-cli-docs] |

`Cmd` does not expand globs or `$FOO` in arguments. That matches how
Python already passes argv lists, not a shell string.[^basic-cli-cmd][^util]

## Phase 0 pin (`nightly-2026-08-23-fb208ba` + basic-cli 0.22.0)

Recorded from `roc test roc/rocci-ops/app.roc` and
`roc roc/rocci-ops/app.roc -- pin-arg` on macOS. Dialect matches the
template-compiler post-mortem: `var` unused here; `match`; annotations
on the line above; `UnixBytes` argv; `Path.read_utf8!(Path.utf8(path))`;
no type-module `name = name`. The spike is a single `app.roc` so
cross-module import poisoning does not apply.[^postmortem][^pin-app]

| Check | Result |
| --- | --- |
| `main!` argv | Received. Count includes the executable (`argv_count=2` for one user arg). |
| argv tag | `UnixBytes` on macOS. Decode with `OsStr.to_raw` then `Str.from_utf8_lossy`. |
| `Cmd.exec_exit_code!` | `true` → `Ok(0)`; `false` → `Ok(1)`. Non-zero is **not** `Err` on this API. |
| `Path` UTF-8 | `write_utf8!` / `read_utf8!` / `delete!` roundtrip `pin-ok` under `Env.temp_dir!()`. |
| `Env.var_str!` | `OsStr.utf8("HOME")` succeeds. |
| JSON subset | `Encoding.Json.parse` into `{ workspace_members, pkgs, name, id, dependencies }`. Error union includes `InvalidJson(Str)` and `MissingRequiredField(Str)`. |
| Reserved `packages` | Roc cannot use `packages` as a record field. Rewrite the first `"packages":` JSON key to `"pkgs":` before parse. Not a Rust helper. |
| Extra JSON fields | Ignored, including `"version": 1` and nested `"source": null`. Live `cargo metadata` can feed the same subset type. |
| `Env.set_cwd!` | Sequential set / `Cmd.exec_output!` of `pwd` / restore works. On macOS `Env.temp_dir!()` may be `/var/folders/.../T/` while `pwd` prints `/private/var/folders/.../T`. |

JSON parse of the checked-in subset fixture succeeded. Do not add a Rust
JSON helper.[^pin-fixture][^encoding-json]

## Phase 6 origin binary spike

Health probes use **curl** argv (`--max-time 5`, `--noproxy *`, optional
`Host:` header), not `Http.get_utf8!`. basic-cli Http is not wired for
the Python 5s no-proxy opener plus per-check `Host` header in this
port.[^origin][^origin-roc][^basic-cli-docs]

Lane, health URL list, compose argv (`--remove-orphans`, optional
`compose.origin.yml`), SHA hex check, and `origin_publish_cmd` live in
`Origin.roc` and match the Python tests' expectations. Mutating
publish/up/backup/SSH is still `not implemented` in the app so this
branch cannot live-deploy.[^origin][^deploy][^origin-roc]

`roc build` on this pin requires equals-form flags
(`--target=x64musl`, `--output=path`). Space-separated `--target x64musl`
errors with `no value was supplied`.[^pin-app]

From macOS arm64 on `nightly-2026-08-23-fb208ba`:

```sh
roc build --target=x64musl --output=/tmp/rocci-ops-linux-spike/rocci-ops \
  roc/rocci-ops/app.roc
```

The artifact is a **statically linked** ELF x86-64 (~1.7M). In Docker
`debian:bookworm-slim` `--platform linux/amd64` with **no Roc** in the
image:

```sh
docker run --rm --platform linux/amd64 \
  -v /tmp/rocci-ops-linux-spike/rocci-ops:/rocci-ops:ro \
  debian:bookworm-slim /rocci-ops origin --help
```

prints `usage: rocci-ops origin [-h] {publish,up,backup} ...` (exit 0).
`command -v roc` is empty in that container. Origin viability for a
**prebuilt binary** is a go on this pin; origin on the VPS must still
stay Python until a later cutover copies a binary instead of
`uv run`.[^origin][^deploy][^pin-app]

## Phase 7 go / no-go

`./roc/rocci-ops/parity.sh` diffs Python `uv run --no-dev rocci-ops`
against `roc roc/rocci-ops/app.roc --` for `-h`, `check -h`, `ci --list`,
and `check deps` (stdout, stderr, exit). On this revision the harness
exits 0.[^parity-sh][^cli][^ci][^deps]

| Question | Verdict | Evidence |
| --- | --- | --- |
| Cmd mapping | **Go** | `exec_exit_code!` returns `Ok(0)`/`Ok(1)`; sequential `Env.set_cwd!`; capture-then-write for `stdout_path`[^pin-app][^basic-cli-cmd] |
| JSON (`cargo metadata`) | **Go** | `Encoding.Json.parse` after rewriting reserved `packages` to `pkgs`[^pin-fixture][^encoding-json] |
| TOML subset | **Go** | Hand-rolled `[[feature]]` / query / session tables; live `check docs` matches Python[^docs] |
| Origin Linux binary | **Go** | `roc build --target=x64musl`; Debian amd64, no Roc, prints `origin --help`[^pin-app][^origin-roc] |
| CI toolchain cost | **No-go** for replacing uv | Lint/test jobs would need the nightly or a checked-in binary on every runner that today only has uv. Extra `roc test` on this branch is acceptable; swapping `uv run --no-dev rocci-ops ci` is a later plan.[^ci][^python-research] |
| Product cutover | **No-go** | Workflows, origin `uv run`, and README stay Python. Many commands remain `not implemented` (mutating origin/deploy, `check zed`, archive package/publish, non-dry-run release). Python still wins. A cutover is a **new** plan.[^cli][^deploy][^pyproject] |

Parity on the four harness surfaces plus origin-binary viability is not
a cutover. Dual `CLASSES` copies must not drift while both CLIs
exist.[^deps]

## Origin delivery is the hard viability question

Today bootstrap copies Compose, Caddy, `uv.lock`, and the **Python
sources** to `/srv/rocci/{prod,staging}`. Remote publish is `uv run
--no-dev rocci-ops origin publish SHA`. The VPS must not gain `rocci`,
`rocdown`, `roc`, rustc, or WebKit.[^origin][^deploy][^python-research]

A Roc `origin` command therefore cannot be `roc run` on the VPS. The
exercise must prove a **prebuilt native binary** (likely Linux musl from
`roc build --target`) that origin can exec with no Roc toolchain. Until
that binary exists and matches Python publish/up/backup, origin stays
Python. Do not copy `roc/rocci-ops/` onto the VPS as source.

## CI toolchain is the other viability question

Lint and test jobs run `uv run --no-dev rocci-ops ci …` after Rust/Node
setup. They do not install Roc. The hosted `roc` job already installs
the pinned nightly; site packaging also has Roc.[^ci][^python-research]

A Roc ops CLI used **as** CI would require Roc (or a checked-in binary)
on every job that today only needs uv. That is a cutover cost, not a
Phase 0–6 requirement. Until cutover, `roc test` on `roc/rocci-ops/` is
an extra check on the exercise branch, not a replacement for pytest.

Dual maintenance is real: `CLASSES` in `workspace_deps.py` and any Roc
copy must not drift while both exist.[^deps]

## What parity and viability mean

**Parity** (Python is the oracle):

- Same command names and usage text
- Same exit codes: `0` ok, `2` usage/unknown, subprocess failure forwarded
- `ci --list` prints the same job names in the same order
- `check deps` accepts or rejects the same workspace graph
- Version bump math and `vX.Y.Z` parsing match
- Help and unknown-command tests can be compared without network

**Viability** (Roc is a credible operator host):

- `Cmd` maps non-zero exits without swallowing them
- JSON subset decode of `cargo metadata --format-version 1 --no-deps`
- Sequential cwd and stdout-capture workarounds do not lose output
- A musl (or glibc) Linux binary runs `origin --help` without `roc`
- Startup and compile time are acceptable for laptop replay of `ci --list` and `check deps`
- No requirement to put `roc` on the origin or to rewrite ProxyCommand / PID 1

Parity without viability is not a cutover. Viability without a
side-by-side help/`ci --list`/`check deps` match is not a cutover.

## Authoring shape

This is a **CLI**, not a page or component. Write ordinary `.roc` with
`basic-cli`, not `.rocci`. Follow the pinned-nightly dialect this repo
already compiles: `snake_case`, parenthesized `List(Str)`, `match` on
tag unions, `=> Try({}, [..])` on `main!`. Put pure rules in modules;
keep `Cmd` / `Path` / `Http` at the app edge.[^author]

Suggested layout on the exercise branch:

```text
roc/rocci-ops/
  app.roc           # basic-cli driver
  Cli.roc
  Ci.roc
  WorkspaceDeps.roc
  Version.roc
  fixtures/
```

Not a Cargo workspace member. Not a Rocci app (`rocci.toml`, `@context`).

## What stays POSIX even after a later cutover

Unchanged from the Python migration:[^python-research]

- `docker/app/entrypoint.sh` and `docker/cdn/entrypoint.sh` as PID 1
- `docker/install-roc.sh` (Roc is not installed yet)
- `docker/prod/access-ssh-proxy.sh` as OpenSSH `ProxyCommand`

Those are out of bound for the Roc rewrite.

[^plan]: Phased Bound/Exit; Python stays operator CLI.
[^python-research]: uv is the operator runtime; Roc was deferred.
[^python-plan]: Rewriting operator scripts in Roc was out of bound.
[^dx-plan]: Current command names to match.
[^template-plan]: Parallel branch; production compiler unchanged.
[^cli]: `rocci-ops <command>`; unknown command exits 2.
[^ci]: `JOB_NAMES`; knowledge captures JSON via `stdout_path`.
[^deps]: `cargo metadata` JSON; unclassified members fail.
[^docs]: `docs/coverage.toml` via `tomllib`.
[^version]: `patch|minor|major` on `X.Y.Z`.
[^origin]: Health, compose, rollback; no product toolchain.
[^deploy]: Copies `rocci-ops/src/rocci_ops` onto the lane root.
[^archive]: `sha256_file` plus `tarfile`.
[^release]: `gh` wait loop with `time.sleep(30)`.
[^util]: `subprocess.run(..., check=True)`.
[^pyproject]: No third-party runtime deps.
[^install-roc]: Nightly installer used by the hosted `roc` job.
[^basic-cli-pin]: Same 0.22 URL as island snapshot eval.
[^basic-cli-cmd]: `exec_exit_code!` when the numeric code matters.
[^basic-cli-docs]: `Path`, `Env.set_cwd!`, `Http.get_utf8!`.
[^encoding-json]: Typed `Encoding.Json.parse`.
[^postmortem]: UnixBytes argv; Path; no `parse = parse`; import isolation.
[^pin-app]: Phase 0 `roc/rocci-ops/app.roc`; Phase 6 x64musl `roc build`.
[^pin-fixture]: Subset JSON with `workspace_members` and `packages`.
[^origin-roc]: `Origin.roc` lane/health/compose/`origin_publish_cmd`.
[^parity-sh]: `roc/rocci-ops/parity.sh` vs `uv run --no-dev rocci-ops`.
[^impl-postmortem]: Isolation, reserved words, x64musl; Python stays operator CLI.
[^author]: Helpers are `.roc`; widgets are `.rocci`.
