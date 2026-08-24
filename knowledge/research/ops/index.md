# Ops

CI, hosting, python-uv, and Tangled.

* [Findings after migrating operator scripts to Python and uv](python-uv-ops-pipeline.md) - POSIX remains PID 1, image Roc install, and ProxyCommand. Later Roc port should reuse `rocci-ops` names; uv stays the operator runtime until `basic-cli` covers CI, laptops, and origin. Implementation plan: [Python uv pipeline](/plans/ops/python-uv-ops-pipeline.md). Exploratory.
* [Tangled as canonical host with a GitHub macOS CI mirror](tangled-hosting.md) - Inverse topology for a near-term public open-source clone: Tangled owns git, issues, PRs, and Linux CI; GitHub remains a SHA-faithful mirror for `macos-latest` Actions. No shipped orgs; grouping is sibling repos under one handle. Implementation plan: [Tangled hosting](/plans/ops/tangled-hosting.md). Exploratory; not approved.
* [Repository hosting for Rocci's distributed governance](repository-hosting-and-distributed-governance.md) - GitHub and Tangled comparison for public launch, contributor workflow, infrastructure ownership, and future shared authority.
