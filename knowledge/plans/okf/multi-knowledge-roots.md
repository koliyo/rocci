---
type: Implementation Plan
title: Multiple knowledge roots for rocci-okf
description: User-level TOML registry of local and git OKF roots, with cached git checkouts, directed edge policy, a settings UI, and a CLI that prints resolved paths for agents.
tags: [domain/okf, domain/rocci-okf, concern/architecture, concern/tooling, concern/security, concern/retrieval]
status: draft
generated: { by: process:cursor, at: 2026-08-25T12:17:00Z }
stale_after: 2026-11-25
authority: exploratory
owners: [human:nils]
sources:
  - id: okf-app-plan
    resource: rocci-okf-app.md
    title: Standalone Rocci OKF application plan
    author: process:cursor
    last_modified: 2026-08-17
  - id: okf-cli
    resource: ../../../crates/rocci-okf/src/main.rs
    title: rocci-okf commands and single-root dispatch
    author: process:git
    last_modified: 2026-08-25
  - id: okf-session
    resource: ../../../crates/rocci-okf/src/session.rs
    title: Last-opened bundle session in ~/.rocci/state/okf.json
    author: process:git
    last_modified: 2026-08-25
  - id: okf-readme
    resource: ../../../crates/rocci-okf/README.md
    title: rocci-okf usage and review-server contract
    author: process:git
    last_modified: 2026-08-25
  - id: okf-engine
    resource: ../../../crates/okf/src/lib.rs
    title: Single-bundle load, parse, and graph resolve
    author: process:git
    last_modified: 2026-08-25
  - id: okf-graph
    resource: ../../../crates/okf/src/graph.rs
    title: Intra-bundle link resolution and OKF3001 escape
    author: process:git
    last_modified: 2026-08-25
  - id: okf-ast
    resource: ../../../crates/okf/src/ast.rs
    title: Bundle, Concept, and Edge types
    author: process:git
    last_modified: 2026-08-25
  - id: okf-engine-readme
    resource: ../../../crates/okf/README.md
    title: Portable OKF engine boundary
    author: process:git
    last_modified: 2026-08-25
  - id: okf-validate
    resource: ../../../crates/okf/src/validate.rs
    title: Git CLI provenance and external_url classification
    author: process:git
    last_modified: 2026-08-25
  - id: okf-dev
    resource: ../../../crates/rocci-okf/src/dev.rs
    title: Preview rebuild, parse cache, and extra HTTP
    author: process:git
    last_modified: 2026-08-25
  - id: okf-presentation
    resource: ../../../crates/rocci-okf/src/presentation.rs
    title: Static review HTML, dashboard, and sidebar
    author: process:git
    last_modified: 2026-08-25
  - id: desktop-state
    resource: ../../../crates/rocci-desktop/src/state.rs
    title: ~/.rocci/state directory resolution
    author: process:git
    last_modified: 2026-08-25
  - id: roc-host-cache
    resource: ../../../crates/rocci-roc-host/src/cache.rs
    title: ROCCI_CACHE and ~/.rocci/cache layout
    author: process:git
    last_modified: 2026-08-25
  - id: static-okf
    resource: ../../decisions/static-okf-boundary.md
    title: Strict OKF Markdown and static rendering boundary
    author: process:okf-migration
    last_modified: 2026-08-17
  - id: product-boundary
    resource: ../../decisions/consolidate-rocdown-product-boundary.md
    title: Approved Rocdown product-boundary decision
    author: process:cursor
    last_modified: 2026-08-18
  - id: server-state
    resource: ../../decisions/server-owned-state.md
    title: Server-owned durable state
    author: process:cursor
    last_modified: 2026-08-16
  - id: nested
    resource: nested-collections.md
    title: Nested OKF collections plan
    author: process:cursor
    last_modified: 2026-08-24
  - id: manage-skill
    resource: ../../../.agents/skills/manage-rocci-knowledge/SKILL.md
    title: Agent knowledge retrieval via rocci-okf
    author: process:git
    last_modified: 2026-08-25
---

# Multiple knowledge roots for rocci-okf

## Goal

Give `rocci-okf` a user-level registry of many OKF bundles — directories on
disk and git repositories (private token, branch, in-repo bundle path) — with
cached checkouts, configurable fetch polling, directed allow/deny edges
between roots, a Rocci settings surface, and a CLI that prints resolved local
paths so agents can read every configured knowledge tree.[^okf-app-plan][^okf-cli][^manage-skill]

## Out of bound

- Merging N bundles into one review site, catalog, search index, or concept-ID
  namespace.
- Changing `check`, `inspect`, `search`, `benchmark`, or `build` so they
  implicitly load the whole registry. Those commands stay single-root; agents
  pass a path from `rocci-okf roots`.[^okf-cli][^okf-engine]
- Teaching the portable `okf` crate to clone remotes, store tokens, or choose
  `~/.rocci`. Git hosting stays an application adapter.[^okf-engine-readme][^okf-app-plan][^product-boundary]
- Writable git roots, committing, or pushing from the cache.
- An always-on fetch daemon when `view` is not running.
- Project-committed `.rocci/okf.toml`, config merge across files, or a
  knowledge-bundle copy of this registry.
- Sparse-checkout optimization, libgit2, SSH-key UI, token rotation jobs,
  rate limits, or public filtered export (still deferred on the application
  plan).[^okf-app-plan]
- Authoring UI for Markdown records. Settings configure roots and edges only.
- Minting an approved Decision. Cross-root Markdown remains an application
  convention until a later format decision.

## Constraints that do not move

- Canonical records stay inert OKF Markdown. Settings UI is application
  chrome, not a knowledge record.[^static-okf]
- `okf::load` remains one filesystem bundle. Intra-bundle `/path.md` links,
  OKF3001 escapes, and concept IDs (path without `.md`) do not change.[^okf-graph][^nested]
- Durable settings live in the TOML file (and env-backed secrets), not in
  Datastar signals or `okf.json` session recents.[^server-state][^okf-session]
- Tokens never appear in logs, `roots` JSON, git `origin` URLs, or re-rendered
  settings HTML. Prefer `token_env` over inline `token`.
- Git cache under `ROCCI_CACHE` (default `~/.rocci/cache`) is derived. A
  failed fetch keeps the last successful checkout.[^roc-host-cache]
- Directory roots are the writable authoring path. Git roots are read-only
  snapshots for agents and review.
- `~/.rocci/state/okf.json` stays last-opened bundle and recents for `view`;
  it is not the registry.[^okf-session][^okf-readme]

## Current behavior

`rocci-okf` takes one bundle directory (default `knowledge`). `view` with no
path restores that last directory from `~/.rocci/state/okf.json`. Git is
invoked only for provenance (`git -C`, last-modified, dirty), not to fetch a
remote knowledge tree.[^okf-cli][^okf-session][^okf-validate]

Graph resolution is intra-bundle. A link that leaves the bundle is OKF3001;
`okf:` is not classified as external (`external_url` requires `://` or
`mailto:` / `tel:` / `data:`), so a cross-root href would be treated as a
broken relative path today.[^okf-graph][^okf-validate]

The standalone application plan already names a multi-bundle registry and
cross-bundle references as operational-maturity work. This plan specifies that
slice without authenticated query, MCP, or semantic search.[^okf-app-plan]

## Target contract

### Configuration file

Path, first existing wins:

1. `ROCCI_OKF_CONFIG` if set and non-empty.
2. `~/.rocci/okf.toml` (same home resolution as `state_dir`: `ROCCI_HOME`,
   then `HOME` / `USERPROFILE`).[^desktop-state]

Create `~/.rocci` with mode `0700` when writing; write the TOML via a temp
file then rename. If the file contains an inline `token`, keep mode `0600`.

```toml
# ~/.rocci/okf.toml
poll = "5m"   # default for git roots; false or "off" disables

[[roots]]
id = "rocci"
kind = "directory"
path = "~/Projects/rocci/knowledge"
incoming = "allow"

[[roots]]
id = "notes"
kind = "git"
url = "https://github.com/example/private-notes.git"
branch = "main"
bundle = "knowledge"
token_env = "GITHUB_TOKEN"
incoming = "deny"
poll = "15m"
allow_from = ["rocci"]
deny_from = []
```

| Field | Rule |
| --- | --- |
| `id` | Required, unique, `[a-z][a-z0-9-]{0,63}`. |
| `kind` | `directory` or `git`. |
| `path` | Directory root: filesystem path; `~` expanded. Must be an OKF bundle root (directory containing `index.md` with `okf_version`, or a later load error). |
| `url` | Git root: clone URL (`https://`, `ssh://`, or `git@`). |
| `branch` | Git root: default `main` if omitted. |
| `bundle` | Git root: subdirectory of the checkout that is the OKF root. Empty or `.` means repo root. |
| `token` | Optional HTTPS secret. Discouraged. Settings UI is write-only. |
| `token_env` | Optional env var name. Wins over `token` when the var is non-empty. |
| `incoming` | `allow` or `deny`. Omitted: `allow` for `directory`, `deny` for `git`. |
| `allow_from` | Root ids that may cite this root even when `incoming = "deny"`. |
| `deny_from` | Root ids that may not cite this root even when `incoming = "allow"`. |
| `poll` | Per-root override of the file-level default. Duration string (`"5m"`, `"1h"`) or `false`. |

Unknown keys are preserved on round-trip (toml table, not a closed struct
dump that drops comments if a lossless path exists; if comments cannot be
kept, document that the settings UI rewrites a canonical file).

Duplicate `id`, unknown `kind`, git root missing `url`, directory root
missing `path`, `allow_from` / `deny_from` names that are not configured
ids, or `id` in its own allow/deny lists are load errors.

### Edge policy

Edges are directed *from citing root* → *cited root*. Intra-root links are
always allowed and stay ordinary bundle-root Markdown.[^okf-graph]

For `from != to`:

1. If `to.deny_from` contains `from` → **deny**.
2. Else if `to.allow_from` contains `from` → **allow**.
3. Else use `to.incoming`.

A pair listed in both `allow_from` and `deny_from` is a config error.
One-way citation is the usual case: git `incoming = "deny"` plus
`allow_from = ["rocci"]` lets `rocci` cite `notes` while `notes` cannot cite
`rocci` unless `rocci.incoming` allows it and `rocci.deny_from` does not
name `notes`.

Settings UI edits the same fields as a matrix (row = from, column = to).
There is no second `[[edges]]` table in v1; the matrix is a view of
`incoming` / `allow_from` / `deny_from`.

### Cross-root link spelling

Authored href: `okf:<id>/<bundle-relative-path>`

Examples: `okf:notes/plans/okf/nested-collections.md`,
`okf:rocci/decisions/static-okf-boundary.md#consequences`.

Portable engine change (small): classify `okf:` the same way as `mailto:` —
not an intra-bundle path, not OKF3001. Do not require the engine to know
configured ids. Record the raw href on `concept.links` as today.[^okf-ast]

`rocci-okf` workspace check (new, see CLI) resolves `okf:<id>/...` against
the registry and emits:

- `OKF3010` error: unknown root id or path that does not exist in that root.
- `OKF3011` error: link exists but policy denies `from` → `to`.

`article_html` may leave `okf:` hrefs unchanged in v1 (no unified site).
Agents follow filesystem paths from `roots`, not published `/@id/` routes.

### Git handler

Own in `rocci-okf`, invoke `git` as a subprocess (same family as provenance,
not libgit2).[^okf-validate]

Cache layout (`ROCCI_CACHE` or `~/.rocci/cache`):[^roc-host-cache]

```text
okf-roots/<id>/
  repo/          # clone working tree
  meta.toml      # url, branch, bundle, last_commit, last_fetch_unix, last_error
```

Resolved bundle path: `okf-roots/<id>/repo/<bundle>`.

Operations:

- Clone if `repo/` is missing (`git clone --branch <branch> --single-branch`).
- Else `git fetch origin <branch>` and `git checkout --force FETCH_HEAD` (or
  `origin/<branch>`). Never fast-forward a dirty authoring clone; this tree is
  disposable.
- Pass HTTPS credentials through the environment for that process only
  (`GIT_ASKPASS` helper or `http.extraHeader` `Authorization: Bearer`), never
  rewrite `remote.origin.url` to embed the token.
- SSH URLs use the user's existing agent; `token` / `token_env` are ignored
  with a warning.
- After success, update `meta.toml` with `rev-parse HEAD`.
- After failure, leave `repo/` as-is, record `last_error`, and still return
  the last resolved path when it exists.

Polling: while `view` (or `run`) is up, a background tick per git root uses
that root's `poll` (else file-level `poll`). `false` / `"off"` skips the
timer. Tick + `sync` share one fetch function. No daemon outside the preview
process.

### Agent CLI

```text
rocci-okf roots [--format json|paths] [--sync|--no-sync]
rocci-okf sync [id]
```

`roots` prints every configured root's **resolved local bundle directory**.

- `--format paths` (default): one absolute path per line, stable `id` sort.
  Missing or failed git roots with no cache: skip the line and print a
  diagnostic on stderr; exit `1` if any configured root is unresolved.
- `--format json`: array of objects `{ id, kind, path, revision, incoming,
  enabled, error }`. Never include `token` or resolved secret values.
  `revision` is HEAD for git when known, `null` for directories.
- `--sync` (default for `roots` when any git root is stale relative to
  `poll`, or when `meta.toml` is missing): fetch then print. `--no-sync`:
  print cache / directory paths only.
- If the config file is missing or `roots` is empty: if `./knowledge` is a
  directory, emit that one path so a repo checkout still works for agents;
  otherwise print nothing and exit `0`.

`sync [id]` fetches one git root or all git roots. Directory roots are
no-ops.

Existing `inspect` / `check` / `search` keep their `root` argument. Agents
loop:

```sh
rocci-okf roots --format paths | while IFS= read -r root; do
  rocci-okf inspect catalog "$root"
done
```

Document this in `crates/rocci-okf/README.md` and point
`manage-rocci-knowledge` at `roots` for multi-tree orientation.[^okf-readme][^manage-skill]

### Settings UI

A Rocci surface in `rocci-okf`, not a knowledge page: `/settings/` on the
preview server, linked from the existing sidebar next to Dashboard and Review
queue.[^okf-presentation]

Durable state is `okf.toml`. Mutations are one-shot commands (POST), not a
live SSE stream.[^server-state]

Minimum screen:

- List roots (id, kind, resolved path or last error, incoming default).
- Add directory (folder path) and add git (url, branch, bundle, poll,
  `token_env` name, optional token field that clears after save).
- Edit / remove a root.
- Incoming allow/deny control per root.
- Edge matrix: from × to checkboxes that compile into `allow_from` /
  `deny_from` without creating illegal both-listed pairs.
- Last sync time and a Sync now control for git roots.

Because preview is a static review tree plus `extra_http` today, v1 may
render Rocci templates through the existing apply host and handle POSTs in
Rust next to the session handler, as long as markup lives in `.rocci` and
secrets are not echoed.[^okf-dev][^okf-session] Do not put the registry in
client signals.

## Phases

### Phase 1 — Config schema

Bound: `crates/rocci-okf/src/config.rs` — `OkfUserConfig`, `RootConfig`
(`Directory` | `Git`), `Incoming::Allow|Deny`, load/save, `~` expansion,
validation errors above, redacted `Display`. Wire `ROCCI_OKF_CONFIG`. No git
network, no CLI subcommand required beyond unit tests.

Exit: `cargo test -p rocci-okf config` (module tests) and
`cargo fmt --all -- --check`.

### Phase 2 — Git checkout cache

Bound: `crates/rocci-okf/src/git_root.rs` — `sync_git_root(root, cache_parent,
secrets) -> ResolvedRoot`. Clone/fetch/checkout, `bundle` subpath, token via
env helper, `meta.toml`, ignore SSH tokens with warning. Tests use temporary
`git init` remotes over `file://` (no GitHub).

Exit: `cargo test -p rocci-okf git_root` and `cargo fmt --all -- --check`.

### Phase 3 — Resolve registry and poll

Bound: `resolve_all(config) -> Vec<ResolvedRoot>` combining directory
canonicalize and git cache. Poll interval parser. Preview (`dev.rs`) starts a
timer that calls `sync_git_root` when due; `--no-sync` is CLI-only until
Phase 4. Last-good cache on fetch failure.

Exit: unit tests for stale vs fresh poll skip; `cargo test -p rocci-okf` and
`cargo fmt --all -- --check`.

### Phase 4 — Agent CLI

Bound: `rocci-okf roots` and `rocci-okf sync` as specified. Empty-config
`./knowledge` fallback. JSON redaction tests. README agent recipe.
`cli_e2e.rs` coverage with `ROCCI_OKF_CONFIG` and `ROCCI_CACHE` pointed at
temp dirs.

Exit: `cargo test -p rocci-okf`, `cargo fmt --all -- --check`.

### Phase 5 — Cross-root links and edge policy

Bound: `okf` treats `okf:` hrefs as non-bundle (no OKF3001/3002). `rocci-okf`
`edge_allowed(from, to, config)` plus `rocci-okf check-edges` (or
`check --workspace` reading the config and every resolved root) emitting
OKF3010/OKF3011. Register new codes in `okf` diagnostics allow-list only if
the engine emits them; otherwise keep them application-side and document in
the README. Intra-bundle tests unchanged.

Exit: `cargo test -p okf`, `cargo test -p rocci-okf`,
`cargo fmt --all -- --check`.

### Phase 6 — Settings UI

Bound: `/settings/` Rocci UI, sidebar link, one-shot save/add/remove/sync
commands writing `okf.toml`, write-only token field, incoming + matrix
editors. Update README. No live stream. No change to `check` defaults.

Exit: `cargo test -p rocci-okf`, `cargo fmt --all -- --check`, and
`crates/rocci-okf/README.md` describes settings and the agent `roots`
workflow.

## Status

Exploratory; no phase started.

[^okf-app-plan]: Application plan Phase 6 already lists a multi-bundle registry and cross-bundle references as later operational work.
[^okf-cli]: Current subcommands take a single `root` `PathBuf`; default `knowledge`.
[^okf-session]: Session file stores one `bundle` path and recents, not a root list.
[^okf-readme]: Documented restore from `~/.rocci/state/okf.json` and single-bundle `view`.
[^okf-engine]: `okf::load` / `check` take one filesystem root.
[^okf-graph]: `resolve_graph` resolves `/` and relative hrefs inside one bundle; escape is OKF3001.
[^okf-ast]: `Edge` is `from` / `to` / `raw` / `broken` with path-derived concept ids.
[^okf-engine-readme]: Portable engine must not choose `~/.rocci` or depend on Rocci.
[^okf-validate]: Provenance uses `Command::new("git")`; `external_url` does not treat `okf:` as external.
[^okf-dev]: Preview rebuilds one root; `extra_http` already serves the session endpoint.
[^okf-presentation]: Sidebar is Dashboard and Review queue plus collection indexes.
[^desktop-state]: `ROCCI_STATE_DIR` / `ROCCI_HOME` / `HOME` for `~/.rocci/state`.
[^roc-host-cache]: `ROCCI_CACHE` else `~/.rocci/cache`.
[^static-okf]: Canonical knowledge remains inert Markdown.
[^product-boundary]: Rocdown must not depend on OKF; `okf` must not depend on Rocci.
[^server-state]: Durable state is server-owned, not a browser store.
[^nested]: Concept ID remains bundle path without `.md`.
[^manage-skill]: Agents orient with `rocci-okf inspect` / `check` against a bundle path.
