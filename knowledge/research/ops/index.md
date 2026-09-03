# Ops

CI, hosting, python-uv, Tangled, and Cursor My Machines.

* [Cursor My Machines worker on a personal Mac mini](cursor-my-machines-mac-mini.md) - Personal `agent worker start` so Cloud Agents from the iPhone app execute tool calls on a Mac mini; local `target/` and Cargo caches persist; not Enterprise Self-Hosted Pool. Exploratory.
* [Dependabot security updates for Rocci lockfiles](dependabot-security-updates.md) - Close GitHub alerts with in-range pins; `glib` 0.18 stays until wry/tao GTK4 and is dismissed `tolerable_risk`. Agent procedure: `.agents/skills/rocci-dependabot`. Exploratory.

* [A Roc port of rocci-ops is a parallel exercise](rocci-ops-roc.md) - Rewrite `rocci-ops` in ordinary Roc on branch `rocci-ops-roc` to test command parity and basic-cli viability. Python plus uv stays the operator CLI. Phase 0 pin recorded; Phase 6 x64musl binary prints `origin --help` in Debian with no Roc. Implementation plan: [Roc rocci-ops](/plans/ops/rocci-ops-roc.md). Exploratory.
* [Findings after migrating operator scripts to Python and uv](python-uv-ops-pipeline.md) - POSIX remains PID 1, image Roc install, and ProxyCommand. Later Roc port should reuse `rocci-ops` names; uv stays the operator runtime until `basic-cli` covers CI, laptops, and origin. Implementation plan: [Python uv pipeline](/plans/ops/python-uv-ops-pipeline.md). Exploratory.
* [Tangled as canonical host with a GitHub macOS CI mirror](tangled-hosting.md) - Inverse topology for a near-term public open-source clone: Tangled owns git, issues, PRs, and Linux CI; GitHub remains a SHA-faithful mirror for `macos-latest` Actions. No shipped orgs; grouping is sibling repos under one handle. Implementation plan: [Tangled hosting](/plans/ops/tangled-hosting.md). Exploratory; not approved.
* [Repository hosting for Rocci's distributed governance](repository-hosting-and-distributed-governance.md) - GitHub and Tangled comparison for public launch, contributor workflow, infrastructure ownership, and future shared authority.
