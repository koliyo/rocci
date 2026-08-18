## Recommendation

Use a hybrid setup:

1. A short root `AGENTS.md` for rules that apply to every Rocci task.
2. Focused, task-oriented skills under `.agents/skills/`.
3. The `knowledge/` bundle as the source of domain knowledge—not a giant skill that duplicates it.

Do not create one all-encompassing `rocci` skill, and do not create one skill per crate. The useful boundary is a repeatable workflow.

OpenAI’s current guidance says skills should be focused on one job, use concise trigger descriptions, and load detailed instructions progressively. Repository skills belong in `.agents/skills`. [`AGENTS.md` is always loaded](https://developers.openai.com/codex/guides/agents-md/), whereas a [skill’s full instructions load only when selected](https://developers.openai.com/codex/skills/).

## Why this fits Rocci

Rocci has several distinct work surfaces:

| Surface | Workflow |
|---|---|
| Knowledge system | Query, inspect, author, review, and validate OKF records with `rocci-okf` |
| Language/compiler | Change `.rocci` or `.rocdown` parsing, lowering, diagnostics, and source maps |
| Rocdown sites | Change documentation cataloging, routes, assets, static rendering, or the theme |
| Runtime/desktop | Change configuration, serving, bundling, desktop host, or application behavior |

Those boundaries already exist in the workspace map in [README.md](/Users/nils/Projects/rocci/README.md:10) and the ownership table in [contributing.rocdown](/Users/nils/Projects/rocci/docs/project/contributing.rocdown:26).

A large skill would trigger for almost any Rocci prompt and load irrelevant knowledge. Per-crate skills would go too far in the other direction because real changes often cross crate boundaries.

## Initial setup

Start with only one specialized skill:

```text
AGENTS.md
.agents/
  skills/
    manage-rocci-knowledge/
      SKILL.md
      agents/
        openai.yaml
```

Add further skills only after observing recurring workflows.

### Root `AGENTS.md`

Keep it around 40–80 lines and include only universal policy:

- Brief product and crate ownership map.
- Preserve the compiler/runtime/Rocdown boundaries documented in the repository.
- Inspect the working tree before editing and preserve unrelated changes.
- Use focused crate tests first; use `cargo test --workspace` for cross-cutting work.
- Update the owning README or Rocdown reference when behavior changes.
- Treat proposed reports differently from implemented behavior.
- Point agents to `knowledge/` for architecture, decisions, status, and limitations.
- Tell agents that specialized workflows live in `.agents/skills`.

Do not embed CLI tutorials, the OKF schema, or a full architecture guide. Those would consume context on every task.

### `manage-rocci-knowledge` skill

Suggested trigger description:

> Query, inspect, validate, author, or review Rocci’s `knowledge/` OKF bundle using `rocci-okf`. Use for architecture and decision retrieval, knowledge-record edits, lifecycle or provenance review, graph inspection, and OKF diagnostics. Do not use for ordinary source-code changes unless the task also requires consulting or updating canonical knowledge.

The body should define four workflows:

1. **Retrieve**

   - Search `knowledge/` with `rg` when the concept ID is unknown.
   - Inspect the normalized record when the ID is known.
   - Inspect the graph when relationships matter.
   - Read cited sources before relying on mutable implementation claims.

2. **Author or revise**

   - Distinguish normative, descriptive, exploratory, and historical authority.
   - Keep knowledge Markdown inert; never add Rocdown declarations.
   - Match source IDs to footnote IDs.
   - On substantive generated changes, update `generated.at`, return the record to `draft`, and retain historical verification rather than manufacturing a new human verification.
   - Update the appropriate collection index and `knowledge/log.md` when warranted.

3. **Validate**

   Use the exact current command forms:

   ```sh
   cargo run -q -p rocci-okf -- check knowledge \
     --profile rocci --format terminal

   cargo run -q -p rocci-okf -- inspect \
     --profile rocci concept CONCEPT_ID knowledge

   cargo run -q -p rocci-okf -- inspect \
     --profile rocci catalog knowledge

   cargo run -q -p rocci-okf -- inspect \
     --profile rocci graph knowledge
   ```

   For `inspect`, `--profile` must precede `catalog`, `concept`, or `graph`.

4. **Report**

   - Separate errors from lifecycle/provenance warnings.
   - Do not “fix” source-drift warnings by changing verification metadata.
   - Identify which conclusions came from canonical records and which were verified directly against code.

The skill should point to the existing review policy in [priority-1-review.md](/Users/nils/Projects/rocci/knowledge/reference/priority-1-review.md:21) and CLI reference in [cli.rocdown](/Users/nils/Projects/rocci/docs/reference/cli.rocdown:134), rather than copying them into bundled references.

No script is needed initially: `rocci-okf` is already the deterministic implementation. The current knowledge check completes without errors, while correctly reporting warnings caused by the dirty/untracked source state and outdated verification.

## Implementation plan

### Phase 1 — Establish the repository baseline

- Add the root `AGENTS.md`.
- Keep it limited to invariants, ownership, validation expectations, and pointers.
- Verify it does not merely restate README or documentation content.
- Acceptance: a fresh agent can identify the owning layer and appropriate test surface without loading a skill.

### Phase 2 — Implement the knowledge skill

- Run the skill-creator initializer with `.agents/skills` as its destination.
- Write the focused `SKILL.md`.
- Generate `agents/openai.yaml` with:
  - `display_name: "Manage Rocci Knowledge"`
  - a concise short description;
  - a default prompt mentioning query, author, review, and validate.
- Keep implicit invocation enabled.
- Do not add scripts, assets, README files, or duplicated schema documentation.
- Run the skill validator.

### Phase 3 — Trigger and workflow evaluation

Test the skill in fresh agent contexts.

Positive prompts:

- “What has Rocci decided about client-side islands?”
- “Find the current Rocdown architecture and verify it against code.”
- “Add a knowledge record for this accepted decision.”
- “Audit stale and source-drift warnings in the knowledge bundle.”
- “Show the knowledge graph around the static OKF boundary.”

Negative prompts:

- “Fix this Rust borrow-checker error.”
- “Change the Rocdown parser.”
- “Run the workspace tests.”
- “Restyle the documentation navigation.”

Acceptance criteria:

- All positive prompts activate the skill.
- Ordinary Rust work does not activate it.
- Commands are issued with valid argument ordering.
- The agent does not claim warnings are validation failures.
- The agent never fabricates human verification.
- Mutable claims are checked against cited implementation sources.

### Phase 4 — Add skills only where repetition justifies them

Likely next candidates:

- `change-rocci-language`: `.rocci`/`.rocdown` grammar, AST, lowering, diagnostics, source maps, fixtures, and reference updates.
- `build-rocdown-site`: catalog, routes, navigation, assets, static HTML, theme shell, preview, and visual QA.
- `develop-rocci-runtime`: only if runtime, desktop host, serving, and packaging tasks recur enough to need a dedicated workflow.

Each should represent one end-to-end job, not one crate. Defer creating them until several real prompts demonstrate repeated mistakes or rediscovery cost.

### Phase 5 — Maintenance gate

Whenever CLI syntax, ownership boundaries, or knowledge lifecycle rules change:

- Update the owning product documentation first.
- Update the skill only when its procedure changes.
- Re-run skill validation and trigger tests.
- Forward-test the modified skill using fresh agents and realistic prompts.

This gives Rocci a lightweight always-on operating contract plus precise procedural expertise where it is already most valuable: the knowledge database workflow. No repository files were changed during this investigation.
