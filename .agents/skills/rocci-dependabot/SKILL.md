---
name: rocci-dependabot
description: >-
  List, triage, and patch GitHub Dependabot security alerts on koliyo/rocci
  (Cargo.lock, npm lockfiles, uv.lock, Actions, Docker). Use when a push reports
  vulnerabilities on the default branch, when the user mentions Dependabot,
  GHSA, or security/dependabot, or when closing open dependency alerts. Do not
  use for GitHub Actions workflow failures (rocci-devops) or ordinary non-security
  version bumps unless an alert names the package.
---

# Rocci Dependabot alerts

Close GitHub security alerts with the smallest in-range upgrade. Keep durable
facts in [Dependabot security updates](/knowledge/research/ops/dependabot-security-updates.md);
this skill is the procedure.

## 1. List alerts

```sh
gh api 'repos/koliyo/rocci/dependabot/alerts?state=open&per_page=50' \
  --jq '.[] | {number, severity: .security_advisory.severity, package: .security_vulnerability.package.name, ecosystem: .security_vulnerability.package.ecosystem, ghsa: .security_advisory.ghsa_id, patched: .security_vulnerability.first_patched_version.identifier, manifest: .dependency.manifest_path}'
```

Do not guess from lockfile folklore (Wasmtime, brace-expansion, esbuild) until
this list is in hand. Open Dependabot PRs are optional; alerts often exist with
no PR.

## 2. Match ecosystem to owner

| Manifest | Action |
| --- | --- |
| `Cargo.lock` | `cargo update -p <crate> --precise <patched>` from the workspace root. If Cargo refuses, the parent crate pins an older series. |
| `editors/vscode/package-lock.json` | Prefer a direct bump. If the parent (for example mocha) caps an old major, add an npm `overrides` entry and `npm install` in `editors/vscode`. |
| `editors/zed/Cargo.lock` | Same as Cargo, in that directory. |
| `uv.lock` | `uv lock` after bumping `pyproject.toml` constraints. |
| GitHub Actions / Docker | Pin the patched action digest or image tag in the owning workflow or Dockerfile. |

Do not `cargo update` the whole workspace to clear one alert.

## 3. Stay inside the current generation

- Prefer the advisory's `first_patched_version` on the **same major/minor line** already locked (`ring` 0.17.9 → 0.17.14, not a 0.18 rewrite).
- **Do not** bump `glib` 0.18.x to 0.20+ to close GHSA-wrw7-89jp-8q8g. That version is a gtk-rs generation cut. Linux desktop still uses `gtk` 0.18 / `webkit2gtk` via `tao`, `wry`, and `muda`. Closing that alert needs an upstream wry/tao GTK4 + WebKitGTK 6 migration, not a lockfile pin.
- **Do not** take a Wasmtime 48.x major to fix a 47.0.z patch advisory. Use `--precise 47.0.z`.
- npm `overrides` are allowed for **dev-only** transitives when upstream has not published a patched range. Pin an exact patched version. Note that mocha uses `serialize-javascript` for worker payloads, not the shipped VSIX runtime.

## 4. Validate

Run the narrowest owning tests:

- Cargo: crates that link the crate (`rocci-cli` for `ring`/`rustls`; `rocci-desktop` / Linux for gtk).
- VS Code: `uv run --no-dev rocci-ops ci editors` or the `editors` job commands.
- Knowledge-only edits: `okmate check knowledge --profile base --format terminal`.

GitHub marks alerts fixed only after the **default branch** contains the patched lockfile. Local `gh` still shows `open` until that lands.

## 5. If the patched version is unreachable

Record the blocker in the knowledge record (or a short log bullet). Do not
dismiss an alert from the API unless the user asks. Dismissal reasons:
`not_used` only when the vulnerable API is not in the product path;
`tolerable_risk` when the generation bump is the real fix.
