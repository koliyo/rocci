---
type: Research Report
title: Concurrent knowledge log entries and Git union merge
description: Independent bullets under a shared date heading in knowledge/log.md are not real content conflicts; Git's built-in union merge driver combines unique lines so concurrent research can land without a conflict marker, while in-place rewrites of the same line still duplicate.
tags: [domain/okf, domain/rocci-okf, concern/authoring, concern/tooling, concern/git]
status: draft
generated: { by: process:cursor, at: 2026-08-24T21:55:00Z }
stale_after: 2026-11-24
authority: exploratory
owners: [human:nils]
sources:
  - id: git-union
    resource: https://git-scm.com/docs/gitattributes
    title: gitattributes built-in merge drivers (union)
    author: organization:git
  - id: okf-spec
    resource: https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md
    title: Open Knowledge Format v0.2 specification
    author: organization:google-cloud
  - id: parse-log
    resource: ../../../crates/okf/src/lib.rs
    title: okf parse_log reserved-file handling
    author: process:git
    last_modified: 2026-08-24
  - id: nested
    resource: ../../decisions/nested-okf-collections.md
    title: Nest large OKF collections under a closed product-area vocabulary
    author: process:cursor
    last_modified: 2026-08-24
  - id: worktree-audit
    resource: ../../audits/rocci/worktree-main-push-conflicts.md
    title: Worktree landings and origin/main push conflicts
    author: process:cursor
    last_modified: 2026-08-20
  - id: gitattributes
    resource: ../../../.gitattributes
    title: Repository gitattributes including knowledge/log.md merge=union
    author: process:cursor
    last_modified: 2026-08-24
---

# Concurrent knowledge log entries and Git union merge

## Claim

Most `knowledge/log.md` merge failures from concurrent research are **two independent bullets under the same `## YYYY-MM-DD` heading**, not competing edits of one sentence. Git's built-in `union` merge driver takes the union of unique lines from both sides, which is the right resolution for that case. The repository now attributes the file that way. This is not an OKF format change and does not replace later work on per-entry files if rewrite conflicts become the dominant pain.[^git-union][^gitattributes][^worktree-audit]

## Current log contract

OKF v0.2 allows an optional `log.md` at any directory with ISO date headings and prose entries. Rocci keeps a **single bundle-root** log; per-collection `log.md` is out of bound for nested collections.[^okf-spec][^nested]

`okf` treats any file named `log.md` as reserved: no frontmatter (`OKF1021`), `## YYYY-MM-DD` headings (`OKF1022`). The parser does not build an entry AST or `article_html`. Discovery skips hidden paths. The review site does not publish `/log/`; `rocci-okf view` refuses the file as a concept. The log is for Git, humans, and agents in the working tree.[^parse-log]

Authoring rules still require a log line for material bundle changes, so concurrent agents append to the **same first heading** on a busy day.

## Why the default merge conflicts

Git's default text merge is hunk-based. Two branches that each insert a new `- …` line immediately after `## 2026-08-24` produce overlapping hunks. The texts commute, but the tool cannot see that.

The worktree audit already listed `knowledge/log.md` as a frequent conflict file after rebase onto a moved `main`. History divergence is a separate problem; union merge only helps **content** merges of this file.[^worktree-audit]

## What `merge=union` does

`.gitattributes` contains:

```
knowledge/log.md merge=union
```

`union` is a **built-in** driver. No `git config` merge.driver, no `rocci-okf` helper, no hook. `ort` still runs the merge; this attribute only changes how this path is combined: keep every line that appears in ours, theirs, or both; drop exact duplicate lines.[^git-union][^gitattributes]

Verified in a throwaway repository (2026-08-24):

| Situation | Result |
| --- | --- |
| Both sides add a different bullet under the same date heading | One heading, both bullets, no conflict |
| Both sides rewrite the **same** bullet to different text | **Both** wordings kept (duplicate entries), no conflict marker |

Shared identical lines (`# Knowledge log`, `## 2026-08-24`, `- alpha`) appear once. Order of new bullets follows Git's combination of the two sides, not alphabetical or timestamp order.

## How to write log entries

1. Put new work as a **new** `- ` bullet under today's `## YYYY-MM-DD` (create that heading at the top of the date list if it is missing).
2. Do not reflow, wrap, or rephrase another session's bullet in the same change. Union cannot tell a rewrite from a second entry.
3. Completing a "do not log complete until CI" line **is** an in-place edit. If another branch also edits that line, union will keep both versions; delete the stale wording after merge.
4. Do not add YAML frontmatter. Do not use a non-ISO `##` heading.
5. Collection `index.md` files are **not** union-merged. Adding two records in the same area can still conflict there.

## Alternatives considered (not adopted here)

| Approach | Why not now |
| --- | --- |
| Custom `rocci-okf` merge driver | Needs `git config` in every clone and CI image; union is already built in |
| One file per entry (`knowledge/log/…` or `.log_entries/`) | Removes false conflicts and isolates rewrites; needs discovery/CLI rules so fragments are not concepts; hidden dirs are skipped by `okf` today |
| Generate `log.md`, gitignore it | Clone has no log unless a tool runs; spec allows a missing log, but agents and grep currently expect the file |
| Changelog in concept frontmatter | Bundle-level events have no home; same-record edits still conflict; not an OKF field |
| One file per day | Busy days still conflict |
| Per-collection `log.md` | Spec-legal; rejected for layout; only splits conflicts by area |
| Log less / derive from `git log` | Loses curated summaries |

Per-entry fragments plus synthesize remain the stronger design if CI-complete rewrites and duplicate union results become common. Union is the small Git-native fit for **append-only bullets** under a shared heading.

## Limits

- **Not append-only in practice.** CI-complete edits rewrite bullets. Union will duplicate, not three-way-merge, those lines.
- **Near-duplicate bullets** (typo vs intended text) both survive.
- **Blank lines and heading order** can get messier than a hand merge. Newest date should stay first; if union ever inverts sections, restore order in a follow-up commit.
- **Attribute must be in the merge base** for Git to use it. Clones that never fetched this `.gitattributes` still conflict until they merge a revision that contains it.
- **`index.md` and other shared knowledge files** are unchanged.
- This is not an approved Decision and does not change the nested-collections "single root `log.md`" choice.[^nested]

[^git-union]: Built-in `union` driver: line union, no conflict markers; Git documents it as appropriate only for simple cases.
[^okf-spec]: Reserved `log.md`, date-grouped entries, optional at any directory.
[^parse-log]: `parse_log` in `crates/okf/src/lib.rs`; discovery skips names starting with `.`.
[^nested]: Single bundle-root `log.md`; per-directory logs out of bound.
[^worktree-audit]: Shared append-only knowledge files collide when unrebased worktrees land.
[^gitattributes]: `knowledge/log.md merge=union` in the repository `.gitattributes`.
