# Ops

CI, hosting, python-uv, and Tangled.

* [Public-repo CI security and Dependabot](public-ci-security.md) - Hosted `/ci`/`/CI` on review comments, `koliyo`-only `/ci-local` self-hosted (`/cl-local` alias), automatic hosted CI on `main`/`staging`/`production`, environment-secret isolation, Dependabot. Audit: [public CI security](/audits/ops/public-ci-security.md). Exploratory; YAML phases are in tree; UI residuals remain.
* [Python and uv operator pipeline](python-uv-ops-pipeline.md) - Replace CI, deploy, origin, and local maintainer shell with `tools/rocci-ops`. POSIX remains for container PID 1, `install-roc.sh`, and OpenSSH ProxyCommand. Exploratory; Phases 1–6 implemented in this revision; not CI-complete. Research: [Python uv findings](/research/ops/python-uv-ops-pipeline.md).
* [Separate staging and production origins on one VPS](origin-lane-separation.md) - Two Compose projects: `/srv/rocci/prod` on `:8080` (hybrid only) and `/srv/rocci/staging` on `:8081` (live examples). Code in this revision; Tunnel cutover is operator work. Exploratory.
* [Tangled hosting and devops with a GitHub macOS mirror](tangled-hosting.md) - Tangled as canonical git, review, and Linux CI before the near-term public open-source clone; GitHub as a fast-forward mirror that supplies `macos-latest` runners. Exploratory; no phase started.
