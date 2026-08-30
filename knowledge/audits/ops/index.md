# Ops

CI, test suite, hosting, python-uv, and Tangled.

* [Workspace test-suite review](workspace-test-suite.md) - Default Rust suite is broader than the documented sub-second budget; Roc-gated builds run whenever `roc` is on `PATH`; kitchen-sink and CI checks overlap; generated-app HTTP, operator pytest, and the SSE target rule are the main coverage holes. Plan: [fast default suite](/plans/ops/workspace-test-suite.md).
* [Public-repo GitHub Actions security review](public-ci-security.md) - Comment-gated self-hosted CI is private-repo shaped: `/ci` is too narrow, local runners share the deploy host, protected branches have no hosted CI, and Dependabot is absent. Plan: [public-repo CI security](/plans/ops/public-ci-security.md).
