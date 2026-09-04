from rocci_ops.ci import JOB_NAMES, parse_ci_args, steps_for
from rocci_ops.paths import repo_root


def test_list_jobs_are_stable() -> None:
    assert JOB_NAMES == (
        "lint",
        "test",
        "fixtures-and-docs",
        "editors",
        "knowledge",
        "roc",
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
        argv[argv.index("--profile") + 1] == "base"
        for argv in argv_lists
        if "--profile" in argv
    )
    assert any(argv[:4] == ("diff", "-qr", "-x", "*.html") for argv in argv_lists)


def test_fixtures_and_docs_stages_example_docs() -> None:
    argv_lists = [s.argv for s in steps_for("fixtures-and-docs", repo_root())]
    assert any("rocci-docs" in argv for argv in argv_lists)
    assert any(argv[-2:] == ("check", "site") for argv in argv_lists)
    assert any(argv[-2:] == ("check", "docs") for argv in argv_lists)
    assert all(argv[-3:] != ("test", "-p", "rocci-docs") for argv in argv_lists)
    assert all("inspect" not in argv for argv in argv_lists)


def test_editors_job_uses_check_zed() -> None:
    argv_lists = [s.argv for s in steps_for("editors", repo_root())]
    assert any(argv[-2:] == ("check", "zed") for argv in argv_lists)
    lint = steps_for("lint", repo_root())
    argv_lists = [s.argv for s in lint]
    assert any(argv[-3:] == ("rocci-ops", "check", "deps") for argv in argv_lists)
    pytest_steps = [s for s in lint if s.argv[-1:] == ("pytest",)]
    assert pytest_steps
    assert pytest_steps[0].argv[:3] == ("uv", "run", "--group")
    assert pytest_steps[0].cwd == "rocci-ops"


def test_roc_job_installs_nightly_and_requires_roc() -> None:
    steps = steps_for("roc", repo_root())
    argv_lists = [s.argv for s in steps]
    assert any("install-roc.sh" in argv[-1] for argv in argv_lists)
    assert any(s.extra_env == (("ROCCI_REQUIRE_ROC", "1"),) for s in steps)
    assert any(
        argv[:3] == ("cargo", "test", "-p") and "rocci-cli" in argv and "rocci-rocdown" in argv
        for argv in argv_lists
    )
    assert all("--workspace" not in argv for argv in argv_lists)


def test_roc_job_runs_build_sh_before_gated_cargo_test() -> None:
    argv_lists = [s.argv for s in steps_for("roc", repo_root())]
    joined = [" ".join(argv) for argv in argv_lists]
    build_idx = next(i for i, line in enumerate(joined) if "rocci-platform/build.sh" in line)
    test_idx = next(
        i
        for i, argv in enumerate(argv_lists)
        if argv[:3] == ("cargo", "test", "-p") and "rocci-cli" in argv
    )
    install_idx = next(i for i, line in enumerate(joined) if "install-roc.sh" in line)
    assert install_idx < build_idx < test_idx
