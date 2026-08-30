"""Enforce docs/coverage.toml and docs/search-queries.toml."""

import argparse
import re
import sys
import tomllib
from pathlib import Path

from rocci_ops.paths import repo_root

ALLOWED_STATUS = {"current", "experimental", "planned", "removed"}
OWNED_STATUS = {"current", "experimental"}


def slugify(text: str) -> str:
    out: list[str] = []
    hyphen = False
    for ch in text:
        if ch.isascii() and ch.isalnum():
            out.append(ch.lower())
            hyphen = False
        elif out and not hyphen:
            out.append("-")
            hyphen = True
    while out and out[-1] == "-":
        out.pop()
    return "".join(out)


def split_route(route: str) -> tuple[str, str]:
    path, _, fragment = route.partition("#")
    if not path.endswith("/"):
        path = path + "/"
    return path, fragment


def docs_file_for(root: Path, path: str) -> Path | None:
    if path.startswith("/examples/"):
        return root / "examples/rocci/apps.toml"
    trimmed = path.strip("/")
    if trimmed.startswith("docs/"):
        trimmed = trimmed[len("docs/") :]
    rel = Path(trimmed)
    for candidate in (
        root / "docs" / rel.with_suffix(".rocdown"),
        root / "docs" / rel / "index.rocdown",
        root / "site" / rel.with_suffix(".rocdown"),
        root / "site" / rel / "index.rocdown",
    ):
        if candidate.is_file():
            return candidate
    return None


def heading_ids(text: str) -> set[str]:
    ids: set[str] = set()
    for match in re.finditer(r"^#{1,6}\s+(.+)$", text, re.MULTILINE):
        ids.add(slugify(match.group(1)))
    return ids


def check_coverage(root: Path) -> list[str]:
    path = root / "docs" / "coverage.toml"
    data = tomllib.loads(path.read_text(encoding="utf-8"))
    errors: list[str] = []
    seen: set[str] = set()
    for feature in data.get("feature", []):
        fid = feature.get("id", "")
        status = feature.get("status", "")
        canonical = feature.get("canonical", "")
        if fid in seen:
            errors.append(f"duplicate feature id `{fid}`")
        seen.add(fid)
        if status not in ALLOWED_STATUS:
            errors.append(f"{fid}: unknown status `{status}`")
            continue
        if status == "current" and "removed" in fid:
            errors.append(f"{fid}: removed feature labeled current")
        if status not in OWNED_STATUS:
            continue
        if not canonical:
            errors.append(f"{fid}: current/experimental feature has no canonical page")
            continue
        path_part, fragment = split_route(canonical)
        page = docs_file_for(root, path_part)
        if page is None:
            errors.append(f"{fid}: missing canonical page {canonical}")
            continue
        if fragment:
            ids = heading_ids(page.read_text(encoding="utf-8"))
            if fragment not in ids:
                errors.append(f"{fid}: canonical fragment `#{fragment}` missing on {page.relative_to(root)}")
    return errors


ALLOWED_ENTRY = {"roc-first", "web-first"}
ALLOWED_DISPOSITION = {"page-fix", "product-issue", "non-goal"}


def check_first_use_sessions(root: Path) -> list[str]:
    path = root / "docs" / "first-use-sessions.toml"
    if not path.is_file():
        return ["missing docs/first-use-sessions.toml"]
    data = tomllib.loads(path.read_text(encoding="utf-8"))
    errors: list[str] = []
    if data.get("schema_version") != 1:
        errors.append("first-use-sessions: schema_version must be 1")
    if data.get("product") != "rocci":
        errors.append("first-use-sessions: product must be rocci")
    for session in data.get("session", []):
        sid = session.get("id", "")
        prefix = f"session `{sid}`" if sid else "session"
        entry = session.get("entry", "")
        if entry not in ALLOWED_ENTRY:
            errors.append(f"{prefix}: entry must be roc-first or web-first")
        if not session.get("date"):
            errors.append(f"{prefix}: missing date")
        if "success" not in session:
            errors.append(f"{prefix}: missing success")
            continue
        if session["success"] is True:
            if session.get("minutes_to_visible") in (None, ""):
                errors.append(f"{prefix}: successful session needs minutes_to_visible")
        else:
            if not session.get("failed_step"):
                errors.append(f"{prefix}: failed session needs failed_step")
            disposition = session.get("disposition", "")
            if disposition not in ALLOWED_DISPOSITION:
                errors.append(f"{prefix}: disposition must be page-fix, product-issue, or non-goal")
    return errors


def check_search_queries(root: Path) -> list[str]:
    path = root / "docs" / "search-queries.toml"
    data = tomllib.loads(path.read_text(encoding="utf-8"))
    errors: list[str] = []
    for query in data.get("query", []):
        expect = query.get("expect", "")
        q = query.get("q", "")
        path_part, fragment = split_route(expect)
        page = docs_file_for(root, path_part)
        if page is None:
            errors.append(f"search `{q}`: missing page {expect}")
            continue
        if fragment:
            ids = heading_ids(page.read_text(encoding="utf-8"))
            if fragment not in ids:
                errors.append(f"search `{q}`: missing fragment `#{fragment}` on {expect}")
    return errors


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="rocci-ops check docs")
    parser.parse_args([] if argv is None else argv)
    root = repo_root()
    errors = check_coverage(root) + check_search_queries(root) + check_first_use_sessions(root)
    if errors:
        sys.stderr.write("docs coverage failed:\n")
        for err in errors:
            sys.stderr.write(f"  {err}\n")
        return 1
    print("docs coverage ok")
    return 0
