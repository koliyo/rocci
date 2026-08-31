---
name: manage-rocci-knowledge
description: Query, inspect, validate, author, or review Rocci's `knowledge/` Open Knowledge Format bundle with `okmate`. Use for architecture and decision retrieval, knowledge-record edits, lifecycle or provenance review, graph inspection, and OKF diagnostics. Do not use for ordinary source-code changes unless they also require consulting or updating canonical knowledge.
---

# Manage Rocci Knowledge

Use the checked-in `knowledge/` bundle as the canonical database and
[okmate](https://github.com/koliyo/okmate) as its deterministic interface
(`okmate` on `PATH`, or `cargo run -q --no-default-features --manifest-path
../okmate/Cargo.toml -p okmate --` from a sibling checkout). Keep domain
facts in the bundle rather than copying them into this skill. Engine tests
live in the okmate repo (`cargo test -p okf` / `cargo test -p okmate`).

Cursor may inject `.cursor/rules/write-knowledge.mdc` for destination and
collection routing. This skill is the retrieve, author, and validate
procedure.

## Establish context

1. Work from the repository root and inspect `git status --short` before
   drawing provenance conclusions or editing records.
2. Read `knowledge/index.md` and the relevant collection index.
3. Treat `dist/knowledge` and other generated files as derived output.
4. For claims about current behavior, read the record's cited code, tests, or
   published documentation. Treat root reports and research as evidence, not
   automatically as descriptions of shipped behavior.
5. Keep normative, descriptive, exploratory, and historical claims distinct.

## Retrieve knowledge

When more than one OKF tree is configured, list resolved local directories
first, then run inspect/check/search against each path:

```sh
okmate roots --format paths
```

```sh
okmate roots --format paths | while IFS= read -r root; do
  okmate inspect --profile base catalog "$root"
done
```

`check`, `inspect`, and `search` stay single-root. Author only in this
repository's `knowledge/` unless the user names another root.

When the concept ID is unknown, search authored records first:

```sh
rg -n "SEARCH_TERMS" knowledge --glob '*.md'
```

When the concept ID is known, inspect its normalized representation:

```sh
okmate inspect \
  --profile base concept CONCEPT_ID knowledge
```

Inspect the catalog for metadata-wide questions and the graph for relationships:

```sh
okmate inspect \
  --profile base catalog knowledge
okmate inspect \
  --profile base graph knowledge
```

Place `--profile` before the `catalog`, `concept`, or `graph` target. Prefer
targeted record and source reads over loading the entire JSON catalog.

## Author or revise records

1. Read `knowledge/reference/priority-1-review.md` before changing lifecycle,
   verification, or provenance metadata. Read
   `knowledge/reference/consolidation.md` when the task changes the
   knowledge-system contract rather than an individual record.
2. Choose a type collection and area based on the claim's purpose and
   authority, not merely the file being discussed. Prefer bundle-root
   `/path.md` links. Inspect accepts a unique filename stem as well as
   the full concept ID.

   | User says | Collection | `type` | Default `authority` |
   | --- | --- | --- | --- |
   | write a plan / implementation plan | `knowledge/plans/<area>/` | Implementation Plan | exploratory |
   | write a report / research | `knowledge/research/<area>/` | Research Report | exploratory |
   | audit / findings vs current behavior | `knowledge/audits/<area>/` | Audit | descriptive |
   | status snapshot / results | `knowledge/status/` | Status | descriptive |

   `<area>` is one of `rocci`, `rocdown`, `okf`, `site`, `ops`, or
   `shared`. Pick the primary owner of the work, not the union of tags.
   Architecture, decisions, status, reference, design, and case-studies
   stay flat. Deepen in place later; do not flatten or nest by lifecycle.

   Architecture and decisions are existing canonical records. Revise them
   only when the claim belongs there. Do not mint a new Decision as
   approved.

   A report plus a plan is two records that cite each other, not one
   file. Keep paired stems on parallel area paths. Writing a plan is not
   executing it. Do not start phases unless the user asks.

   Record shape: kebab-case filename; stem unique under
   `knowledge/plans/`; YAML frontmatter; inert Markdown body;
   `status: draft`; `generated.by: process:cursor`;
   `owners: [human:nils]`. Plans include Goal, Out of bound, Constraints
   that do not move, and phased Bound/Exit.

   ```yaml
   type: Research Report
   title: Concise title
   description: One-sentence claim
   tags: [domain/rocci, concern/architecture]
   status: draft
   generated: { by: process:cursor, at: 2026-08-20T00:00:00Z }
   stale_after: 2026-11-20
   authority: exploratory
   owners: [human:nils]
   sources: []
   ```

3. Keep record bodies inert Markdown. Do not add Rocdown declarations, raw
   HTML, wikilinks, or executable content.
4. Preserve unknown OKF metadata unless the task explicitly removes it.
5. Give each source a unique `id`, use paths relative to the record, and attach
   a matching keyed footnote to every sourced claim. Keep source IDs and
   footnote IDs synchronized.
6. On a substantive generated revision, update `generated.at`, set the record
   to `draft`, and retain historical verification events. Never invent or
   advance a human verification event.
7. Update the nearest directory `index.md` when adding, moving, or removing a
   record (and the parent type index when adding an area). Update
   `knowledge/log.md` for a meaningful bundle-level change. Append a new
   bullet under today's `## YYYY-MM-DD` heading (create it at the top if
   needed). Do not reword another session's bullet in the same change; Git
   `merge=union` combines unique lines and would keep both wordings.
   Details: `/research/okf/knowledge-log-concurrency.md`. Do not log a
   phase as complete until the required GitHub workflows (CI and Knowledge)
   have succeeded on that revision; cite the run IDs in the log entry.
8. Update public Rocdown documentation separately when the changed fact is also
   part of the public product contract.

## Validate

Run the base profile after every knowledge edit (same as Knowledge CI):

```sh
okmate check knowledge \
  --profile base --format terminal
```

Use `--profile strict` only when checking owners-and-evidence rules that
Rocci CI does not require.

If parser, validation, inspection, or CLI behavior changed, run focused
tests in the [okmate](https://github.com/koliyo/okmate) repository:

```sh
cargo test -p okf
cargo test -p okmate --no-default-features
```

Treat validation errors as failures. Report lifecycle and repository-provenance
warnings separately; do not silence source-drift warnings by falsifying
timestamps, status, ownership, or verification metadata.

Build or preview the knowledge site only when output or presentation matters.
Avoid writing `dist/knowledge` during read-only retrieval tasks.

## Report results

- Name the canonical records consulted and any cited implementation sources
  checked directly.
- Separate established project decisions from inference or exploratory work.
- Summarize authored record changes, including lifecycle changes.
- Report validation errors and warnings separately, including warnings caused
  by pre-existing working-tree state.
