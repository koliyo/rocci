"""Check workspace package edges against the frozen Rocdown product boundary.

Package classes and dependency rules come from
knowledge/decisions/consolidate-rocdown-product-boundary.md. Unclassified
workspace members fail.
"""

from __future__ import annotations

import json
import subprocess
import sys

from rocci_ops.paths import repo_root

BASE_ROCCI = {
    "rocci-core",
    "rocci-template",
    "rocci-ungram",
    "rocci-desktop",
    "rocci-cli",
    "rocci-lsp",
    "rocci-highlight",
    "rocci-ui",
    "rocci-roc-host",
    "rocci-datastar",
    "rocci-browser",
}

ROCDOWN = {
    "rocci-rocdown",
    "rocci-theme",
    "rocci-rocdown-cli",
    "rocci-rocdown-lsp",
    "rocci-playground-spike",
    "rocci-playground",
    "rocci-playground-wasm",
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


def classify(name: str) -> str | None:
    for label, members in CLASSES.items():
        if name in members:
            return label
    return None


def cargo_metadata() -> dict:
    result = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"],
        cwd=repo_root(),
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
        return f"rocci-okf must not depend on Rocdown package {dest}"

    return None


def main() -> int:
    packages = workspace_packages(cargo_metadata())
    errors: list[str] = []
    notes: list[str] = []

    classified = {name for members in CLASSES.values() for name in members}
    package_names = set(packages)
    for name in sorted(package_names - classified):
        errors.append(f"unclassified workspace package {name}")
    for name in sorted(classified - package_names):
        errors.append(f"classified name is not a workspace package: {name}")

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
