# Audits

* [Agent-model comparison for Rocci component-generation research](agent-model-component-generation-comparison.md) - Evidence-based comparison of Gemini 3.7 Flash and Grok 4.6 results for the same architecture research and planning task.
* [hybrid-rocdown-islands preview performance audit](hybrid-rocdown-islands-preview-performance.md) - Profiled `rocci-okf run knowledge/plans/hybrid-rocdown-islands.md`; single-concept preview still loads the full bundle and spends ~98% of cached rebuild time in `okf::load`.
* [rocci-okf headless load-performance audit](rocci-okf-headless-load-performance.md) - Headless `rocci-okf run --no-window` rebuild timings, new CLI profile reporting, and evidence that cached latency is dominated by `okf::load`.
* [Rocdown product-boundary refactor completion review](rocdown-boundary-refactor-review.md) - Exit-gate coverage, residual coupling, stale automation and documentation, and prioritized closure work.

The historical syntax audit remains a retained Phase 3 review input.
