# Rocci

Templates, handlers, runtime, desktop inspector, component generation, and falling-block.

* [Rocci-as-platform post-mortem](rocci-as-roc-platform-postmortem.md) - First-cut platform cutover bought `pf` ownership of Datastar/Html, not a new Snake authoring API. Authored apps changed pin plus imports; `respond!` stayed. A Roc-native compiler remains an unstarted parity POC; Html on `pf` is option value if a later vision check typechecks without CLI staging. Pair: [package Rocci as a Roc platform](/plans/rocci/rocci-as-roc-platform.md). Descriptive; do not log complete until CI and Knowledge succeed.
* [Rocci implementation structure review](implementation-structure.md) - Crate-level product boundaries are sound; `plan.rs` and other mixed-concern files are the debt. Pair: [split oversized modules](/plans/rocci/implementation-structure.md). Descriptive; no phase started.
* [Standalone falling-block post-mortem](standalone-falling-block-postmortem.md) - Custom arena versus shipped nested standalone Blocks: play-feel, handler-only modules, gravity-in-live, quoted keydown, origin removal, and what must stay custom.
* [Worktree landings and origin/main push conflicts](worktree-main-push-conflicts.md) - Detached HEAD worktrees, rebasing `main` onto features, and pull-rebase after rejected pushes; named plan branches plus merge-worktree-to-main as the landing fix.
* [Agent-model comparison for Rocci component-generation research](agent-model-component-generation-comparison.md) - Evidence-based comparison of Gemini 3.7 Flash and Grok 4.6 results for the same architecture research and planning task.
