---
name: manage-rocci-knowledge
description: Query, inspect, validate, author, or review Rocci's `knowledge/` Open Knowledge Format bundle with `rocci-okf`. Use for architecture and decision retrieval, knowledge-record edits, lifecycle or provenance review, graph inspection, and OKF diagnostics. Do not use for ordinary source-code changes unless they also require consulting or updating canonical knowledge.
---

# Manage Rocci Knowledge

Use the checked-in `knowledge/` bundle as the canonical database and the
repository's `rocci-okf` CLI as its deterministic interface. Keep domain
facts in the bundle rather than copying them into this skill.

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

When the concept ID is unknown, search authored records first:

```sh
rg -n "SEARCH_TERMS" knowledge --glob '*.md'
```

When the concept ID is known, inspect its normalized representation:

```sh
cargo run -q -p rocci-okf -- inspect \
  --profile rocci concept CONCEPT_ID knowledge
```

Inspect the catalog for metadata-wide questions and the graph for relationships:

```sh
cargo run -q -p rocci-okf -- inspect \
  --profile rocci catalog knowledge
cargo run -q -p rocci-okf -- inspect \
  --profile rocci graph knowledge
```

Place `--profile` before the `catalog`, `concept`, or `graph` target. Prefer
targeted record and source reads over loading the entire JSON catalog.

## Author or revise records

1. Read `knowledge/reference/priority-1-review.md` before changing lifecycle,
   verification, or provenance metadata. Read `OKF_PLAN.md` when the task
   changes the knowledge-system contract rather than an individual record.
2. Choose a collection and record type based on the claim's purpose and
   authority, not merely the file being discussed.
3. Keep record bodies inert Markdown. Do not add Rocdown declarations, raw
   HTML, wikilinks, or executable content.
4. Preserve unknown OKF metadata unless the task explicitly removes it.
5. Give each source a unique `id`, use paths relative to the record, and attach
   a matching keyed footnote to every sourced claim. Keep source IDs and
   footnote IDs synchronized.
6. On a substantive generated revision, update `generated.at`, set the record
   to `draft`, and retain historical verification events. Never invent or
   advance a human verification event.
7. Update the collection index when adding, moving, or removing a record.
   Update `knowledge/log.md` for a meaningful bundle-level change. Do not log a
   phase as complete until the required GitHub workflows (CI and Knowledge)
   have succeeded on that revision; cite the run IDs in the log entry.
8. Update public Rocdown documentation separately when the changed fact is also
   part of the public product contract.

## Validate

Run the Rocci profile after every knowledge edit:

```sh
cargo run -q -p rocci-okf -- check knowledge \
  --profile rocci --format terminal
```

Use the base profile only when explicitly testing portable OKF behavior:

```sh
cargo run -q -p rocci-okf -- check knowledge \
  --profile base --format terminal
```

If parser, validation, inspection, or CLI behavior changed, also run focused
Rust tests before broader workspace tests:

```sh
cargo test -p okf
cargo test -p rocci-okf
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
