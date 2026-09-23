---
type: Audit
title: Template preparation research delivered evidence but accumulated low-value follow-ups
description: A September 12 process audit separates useful compiler research and escaping improvements from phase-driven continuation, weak value checks, and documentation overhead.
tags: [domain/rocci, concern/workflow, concern/evidence, concern/performance]
status: draft
generated: { by: process:cursor, at: 2026-09-12T16:14:33Z }
stale_after: 2026-12-12
authority: descriptive
owners: [human:nils]
sources:
  - id: research
    resource: ../../research/rocci/compile-time-template-preparation.md
    title: Compiler fit, typed adapter, compatibility findings, and measured costs
  - id: parent
    resource: ../../plans/rocci/compile-time-template-preparation.md
    title: Investigation phases, stop condition, and follow-up selection
  - id: costs
    resource: ../../../roc/template-preparation-experiment/costs.py
    title: Runtime variants and Node-baseline candidate selection
  - id: host
    resource: ../../plans/rocci/html-scan-copy-host-coverage.md
    title: HTTP byte equality, no visible renderer gain, and absent Linux evidence
  - id: string
    resource: ../../plans/rocci/html-string-theme-escape.md
    title: String consumer exploration and experimental painter limitation
  - id: string-port
    resource: ../../plans/rocci/html-string-scan-copy-escape.md
    title: Local string-runtime implementation and validation record
  - id: html
    resource: ../../../crates/rocci-platform/platform/Html.roc
    title: Actual escape implementation and unchanged boolean helper
  - id: theme
    resource: ../../../crates/rocci-rocdown/src/plan/theme.rs
    title: Production painters use Str and exclude embedded CSS
  - id: history
    resource: ../../log.md
    title: Dated investigation and follow-up sequence
---

# Template preparation process audit

## Judgment and evidence boundary

The initial question was legitimate: could compile-time template preparation
improve Rocci or remove an external generator? The investigation produced useful
negative architectural evidence and small runtime improvements. The process
then continued through individually reasonable follow-ups without making their
combined value sufficiently visible. This is a judgment about prioritization
and communication, not a finding that the experiments were fabricated.[^research][^history]

This audit inspected local history from `0225aa9c` through `402398e4`, then
reconciled the documentation completion commit `080a353b` observed during the
audit. It read the reports, experiment code, and product diffs; it did not rerun
performance experiments or verify hosted CI. The owner's September 12 request
reports dissatisfaction with continuation through recommended next steps.
Full earlier agent conversations, token charges, working time, and approval
history were not examined. Therefore no numerical productivity loss, model
ranking, or unauthorized-execution claim follows from this audit.

## What was worth keeping

| Delivered evidence or behavior | Practical value and limit |
| --- | --- |
| Prepared data cannot bind arbitrary embedded Roc expressions | Supports retaining source lowering; no compiler replacement or product library was delivered. |
| Typed closure ties checking samples to render inputs | A concrete experimental fix for context-shape drift, with 16 targeted probes; restricted encoder and sample requirements remain. |
| Controlled renderer variants | Builder control beat prepared rendering on the measured large fixture, undermining preparation as the explanation for speed. |
| Node and string escaping changes | Small product optimizations with compatibility expects; no parser, lowering, or compiler-pipeline change in this series. |
| Compatibility discrepancies | Identified CR attribute differences and a real false-boolean helper defect; discovery is not repair. |

These outcomes justify an initial investigation. Negative findings can prevent
expensive architectural mistakes; code volume alone is not their value measure.
The local string port is recorded through `080a353b`; the platform helper still
constructs the same attribute in both boolean branches.[^research][^costs][^html][^string-port]

Recorded Node render totals fell from 401 to 265 ms for 3,000 synthetic renders.
The string painter experiment fell from approximately 215 to 170 ms for 800
renders. These are workload measurements, not user-perceived gains. The local
HTTP comparison found byte equality and no visible renderer gain; Linux was
not measured. The painter setup includes embedded CSS while production theme
compilation sets `embed_css: false`. A site-build speedup remains unproven.
[^research][^host][^string][^theme]

## How continuation lost proportion

1. **Local exits displaced the original decision.** The parent plan explicitly
   allowed stopping and deferred the library after cost isolation. That was a
   sound boundary. Later host and string explorations nevertheless extended
   the work. Each had a technical question, but their completion criteria did
   not require demonstrating that the answer was worth more effort than
   closing the investigation or shipping a bounded repair. This is an
   inference from the recorded sequence, not reconstructed agent intent.
   [^parent][^host][^string][^history]
2. **Evidence quality and product priority were treated too similarly.**
   Checksums, hashes, negative probes, and repeatable timings improve confidence
   in an answer. They do not establish that the answer matters to an actual
   consumer. Microbenchmark thresholds were useful filters, but the HTTP and
   painter qualifications prevented translating them into demonstrated
   application value.[^research][^host][^string]
3. **A comparison obscured the string result.** `select_candidate` compares
   every candidate against Node, including string scan/copy. Its 3.7% figure
   is valid for that selection question. It is not the string helper's
   improvement over its own baseline: the recorded 565 to 386 ms is about
   32%. Calling this a failed string optimization screen without foregrounding
   the baseline makes the subsequent exploration harder to assess.[^costs][^research]
4. **Phase titles made dispositions resemble delivery.** `7e3384ba` calls
   Phase 3 complete although the library boundary work was deferred;
   `3e533243` completes the Linux-or-narrow phase by recording absent Linux
   evidence. Those satisfy the written exits, but `feat` and phase counts
   overstate product progress unless the actual outcome is the headline.
   [^parent][^host][^history]
5. **The evidence acquired a maintenance cost.** A scoped diff through
   `402398e4` contains 3,251 added Python lines in the experiment and 31,187
   JSON receipt lines. Six new investigation/implementation plans accompany
   the research. These counts describe footprint, not time or waste: raw
   receipts are legitimate evidence. Repeated summaries and phase transitions
   nevertheless create reading and synchronization work disproportionate to
   the small implementation. The recorded follow-up chain is the relevant
   process signal.[^history][^parent][^string]

## Better steering at the actual decision points

The following are audit recommendations, not newly approved repository rules.

| Point in the work | Proportionate intervention |
| --- | --- |
| Initial architecture question | Name the decision: retain lowering, replace it, or pursue a restricted library for a named consumer. Allow a no-change conclusion. |
| Prepared rendering appears faster | Run the controlled escaping/builder comparison because it can reverse the architectural recommendation. This continuation was justified. |
| Builder control wins and library is deferred | Close the compiler question. Describe escaping as a separate small opportunity; do not inherit the research program's momentum. |
| Candidate runtime optimization | Validate semantics and compare the actual owning consumer. State whether acceptance needs a local kernel gain or application-level improvement before measuring. |
| HTTP gain is not visible or Linux unavailable | Narrow the claim and finish unless deployment-specific evidence is required for the accepted change. Missing coverage alone is not a new priority. |
| Clear boolean defect | Prioritize a focused repair when authorized, or leave one explicit deferred item. Another plan is not a repaired defect. |

This counterfactual retains useful investigation and necessary validation while
removing automatic expansion. It does not demonstrate how much time it would
have saved.[^research][^parent][^host][^html]

## Recommended continuation rule

Before recommending another experiment, state the remaining user decision,
the result that could change it, the cheapest adequate check, and why this
is preferable to stopping. If no plausible result changes the decision, close
the question and retain the limitation. Complete already-authorized coherent
work autonomously; request direction only for a material new objective, not
for every reversible step.

Report delivery, knowledge gained, and deferred work separately. Prefer commit
titles such as “retain source lowering after renderer comparison” or “speed up
HTML escaping” over “complete Phase N.” Keep one current conclusion with links
to historical evidence. This audit creates no follow-up implementation plan,
changes no product code, and does not prescribe more agents or a model switch.

[^research]: Local experiment findings and limitations; reported timings were inspected, not reproduced by this audit.
[^parent]: Conditional library phase, cost-isolation stop, and Phase 5 follow-up table.
[^costs]: `select_candidate` uses the Node baseline for all variants; controlled runtime algorithms are implemented here.
[^host]: Low-load HTTP equality and unavailable Linux recorded as explicit limits.
[^string]: Painter screen and the embedded-CSS departure from production.
[^string-port]: Current branch records the two string runtime copies and local validation, without hosted-CI completion.
[^html]: Escape kernel changed; `boolean_attribute` still has identical branches at the audited revision.
[^theme]: `compile_single_module` selects `Str` and disables embedded CSS.
[^history]: Follow-up sequence; commit identifiers and scoped diff counts were inspected directly in local Git history.
