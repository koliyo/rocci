from rocci_ops.ci import JOB_NAMES, parse_ci_args, steps_for
from rocci_ops.paths import ensure_h35_desktop, repo_root


def test_h35_desktop_sibling_is_present_or_cloneable() -> None:
    dest = ensure_h35_desktop(repo_root())
    assert (dest / "Cargo.toml").is_file()


def test_list_jobs_are_stable() -> None:
    assert JOB_NAMES == (
        "lint",
        "test",
        "fixtures-and-docs",
        "editors",
        "knowledge",
    )


def test_parse_list_flag() -> None:
    args = parse_ci_args(["--list"])
    assert args.list is True
    assert args.jobs == []


def test_parse_subset() -> None:
    args = parse_ci_args(["lint", "test"])
    assert args.jobs == ["lint", "test"]
    assert args.keep_going is False


def test_knowledge_redirects_validation_json() -> None:
    steps = steps_for("knowledge", repo_root())
    redirected = [s for s in steps if s.stdout_path]
    paths = {s.stdout_path for s in redirected}
    assert "target/knowledge-ci/validation.json" in paths
    assert "target/knowledge-ci/graph.json" in paths
    assert "target/knowledge-ci/retrieval.json" in paths
    argv_lists = [s.argv for s in steps]
    assert any("okmate" in argv for argv in argv_lists)
    assert all("rocci-okf" not in argv for argv in argv_lists)
    assert all(
        argv[argv.index("--profile") + 1] == "strict"
        for argv in argv_lists
        if "--profile" in argv
    )


def test_fixtures_and_docs_stages_example_docs() -> None:
    argv_lists = [s.argv for s in steps_for("fixtures-and-docs", repo_root())]
    assert any("rocci-docs" in argv for argv in argv_lists)
    assert any(argv[-2:] == ("check", "site") for argv in argv_lists)
    assert any(argv[-2:] == ("check", "docs") for argv in argv_lists)
    assert any(argv[-2:] == ("-p", "rocci-docs") or argv[-3:] == ("test", "-p", "rocci-docs") for argv in argv_lists)


def test_editors_job_uses_check_zed() -> None:
    argv_lists = [s.argv for s in steps_for("editors", repo_root())]
    assert any(argv[-2:] == ("check", "zed") for argv in argv_lists)
    argv_lists = [s.argv for s in steps_for("lint", repo_root())]
    assert any(argv[-3:] == ("rocci-ops", "check", "deps") for argv in argv_lists)
