from pathlib import Path

from rocci_ops.docs_coverage import check_coverage, check_search_queries, docs_file_for, slugify
from rocci_ops.paths import repo_root


def test_slugify_matches_rocdown() -> None:
    assert slugify("Build release") == "build-release"
    assert slugify("Serve options") == "serve-options"
    assert slugify("App") == "app"


def test_repo_coverage_and_queries_pass() -> None:
    root = repo_root()
    assert check_coverage(root) == []
    assert check_search_queries(root) == []


def test_examples_lane_resolves_to_catalog() -> None:
    root = repo_root()
    found = docs_file_for(root, "/examples/")
    assert found == root / "examples/rocci/apps.toml"


def test_missing_canonical_is_reported(tmp_path: Path, monkeypatch) -> None:
    docs = tmp_path / "docs"
    docs.mkdir()
    (docs / "coverage.toml").write_text(
        """
schema_version = 1
[[feature]]
id = "syntax.ghost"
name = "Ghost"
area = "syntax"
owner = "rocci-template"
source = "x"
contract = "x"
canonical = "/docs/reference/missing/"
example = ""
status = "current"
""",
        encoding="utf-8",
    )
    monkeypatch.setattr("rocci_ops.docs_coverage.repo_root", lambda: tmp_path)
    from rocci_ops.docs_coverage import check_coverage as check

    errors = check(tmp_path)
    assert any("missing canonical page" in err for err in errors)
