#!/usr/bin/env python3
"""Check workspace package edges against the frozen Rocdown product boundary.

Package classes and allowlisted reverse edges come from
knowledge/decisions/consolidate-rocdown-product-boundary.md. Unclassified
workspace members fail. Today's Rocci-to-Rocdown edges stay allowlisted until
Phase 3 removes them.
"""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

BASE_ROCCI = {
    "rocci-core",
    "rocci-template",
    "rocci-desktop",
    "rocci-cli",
    "rocci-lsp",
    "rocci-highlight",
    "rocci-ui",
}

ROCDOWN = {
    "rocci-rocdown",
    "rocci-theme",
    "rocci-rocdown-cli",
}

OKF_ENGINE = {"okf"}
OKF_APP = {"rocci-okf"}

CLASSES = {
    "base-rocci": BASE_ROCCI,
    "rocdown": ROCDOWN,
    "okf-engine": OKF_ENGINE,
    "okf-app": OKF_APP,
}

ALLOWED_REVERSE: set[tuple[str, str]] = set()

# Populate with (rocci-okf, rocdown-package) when the presentation adapter is
# introduced together with a tracking issue. Empty until then.
TEMPORARY_OKF_ROCDOWN_PRESENTATION: set[tuple[str, str]] = {
    ("rocci-okf", "rocci-rocdown"),
}


def classify(name: str) -> str | None:
    for label, members in CLASSES.items():
        if name in members:
            return label
    return None


def cargo_metadata() -> dict:
    result = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        sys.stderr.write(result.stderr)
        raise SystemExit(result.returncode or 1)
    return json.loads(result.stdout)


def workspace_packages(metadata: dict) -> dict[str, dict]:
    members = set(metadata["workspace_members"])
    return {
        package["name"]: package
        for package in metadata["packages"]
        if package["id"] in members
    }


def direct_workspace_edges(packages: dict[str, dict]) -> list[tuple[str, str]]:
    names = set(packages)
    edges = []
    for name, package in packages.items():
        seen = set()
        for dep in package.get("dependencies", []):
            dest = dep["name"]
            if dest in names and dest not in seen:
                seen.add(dest)
                edges.append((name, dest))
    return edges


def forbidden(src: str, dest: str) -> str | None:
    src_class = classify(src)
    dest_class = classify(dest)
    if src_class is None or dest_class is None:
        return None

    if src_class == "base-rocci" and dest_class in {"rocdown", "okf-engine", "okf-app"}:
        if (src, dest) in ALLOWED_REVERSE:
            return None
        return f"base Rocci package {src} must not depend on {dest_class} package {dest}"

    if src_class == "rocdown" and dest_class in {"okf-engine", "okf-app"}:
        return f"Rocdown package {src} must not depend on {dest_class} package {dest}"

    if src_class == "okf-engine" and dest_class != "okf-engine":
        return f"okf engine must not depend on {dest_class} package {dest}"

    if src_class == "okf-app" and dest_class == "rocdown":
        if (src, dest) in TEMPORARY_OKF_ROCDOWN_PRESENTATION:
            return None
        return (
            f"rocci-okf may depend on Rocdown package {dest} only while that "
            "presentation edge is allowlisted with a tracking issue"
        )

    return None


def main() -> int:
    packages = workspace_packages(cargo_metadata())
    errors: list[str] = []
    notes: list[str] = []

    for name in sorted(packages):
        if classify(name) is None:
            errors.append(f"unclassified workspace package {name}")

    edges = direct_workspace_edges(packages)
    edge_set = set(edges)
    for src, dest in edges:
        if (src, dest) in ALLOWED_REVERSE:
            notes.append(f"allowlisted reverse edge {src} -> {dest}")
        reason = forbidden(src, dest)
        if reason:
            errors.append(f"{src} -> {dest}: {reason}")

    stale = [
        f"{src} -> {dest}"
        for src, dest in sorted(ALLOWED_REVERSE)
        if src in packages and dest in packages and (src, dest) not in edge_set
    ]
    for edge in stale:
        errors.append(f"allowlisted reverse edge no longer present: {edge}")

    for note in notes:
        print(note)

    if errors:
        print("workspace dependency check failed:", file=sys.stderr)
        for error in errors:
            print(f"  {error}", file=sys.stderr)
        return 1

    print(f"ok: {len(packages)} workspace packages, {len(notes)} allowlisted reverse edges")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
