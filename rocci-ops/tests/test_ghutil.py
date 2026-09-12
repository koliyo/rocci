import json

from rocci_ops.ghutil import (
    CI_WORKFLOW,
    parse_check_line,
    wait_for_existing_ci,
    wait_for_workflow_run,
)

SUCCESS = json.dumps(
    [
        {
            "databaseId": 1,
            "status": "completed",
            "conclusion": "success",
            "createdAt": "2026-01-01T00:00:00Z",
        }
    ]
)
IN_PROGRESS = json.dumps(
    [
        {
            "databaseId": 2,
            "status": "in_progress",
            "conclusion": "",
            "createdAt": "2026-01-01T00:00:00Z",
        }
    ]
)
FAILED = json.dumps(
    [
        {
            "databaseId": 3,
            "status": "completed",
            "conclusion": "failure",
            "createdAt": "2026-01-01T00:00:00Z",
        }
    ]
)


def test_ci_workflow_is_ci_yml() -> None:
    assert CI_WORKFLOW == "ci.yml"


def test_parse_check_line() -> None:
    assert parse_check_line("completed success") == ("completed", "success")
    assert parse_check_line("") is None


def test_wait_for_workflow_run_success() -> None:
    seen: list[list[str]] = []

    def gh(args: list[str]) -> str:
        seen.append(args)
        return SUCCESS

    wait_for_workflow_run(sha="abc", gh=gh, sleep=lambda s: None)
    assert seen[0][0:2] == ["run", "list"]
    assert "--workflow" in seen[0]
    assert seen[0][seen[0].index("--workflow") + 1] == "ci.yml"
    assert "knowledge.yml" not in seen[0]


def test_wait_for_workflow_run_watches_in_progress() -> None:
    replies = [IN_PROGRESS, SUCCESS]
    watched: list[int] = []

    def gh(args: list[str]) -> str:
        if args[:2] == ["run", "list"]:
            return replies.pop(0)
        raise AssertionError(args)

    wait_for_workflow_run(
        sha="abc",
        gh=gh,
        sleep=lambda s: None,
        watch=watched.append,
    )
    assert watched == [2]


def test_wait_for_workflow_run_fails_on_failure() -> None:
    try:
        wait_for_workflow_run(
            sha="abc",
            gh=lambda args: FAILED,
            sleep=lambda s: None,
        )
    except SystemExit as exc:
        assert "failure" in str(exc)
    else:
        raise AssertionError("expected SystemExit")


def test_wait_for_workflow_run_fails_when_missing() -> None:
    try:
        wait_for_workflow_run(
            sha="abc",
            gh=lambda args: "[]",
            sleep=lambda s: None,
        )
    except SystemExit as exc:
        assert "has not run on abc" in str(exc)
        assert "ci.yml" in str(exc)
    else:
        raise AssertionError("expected SystemExit")


def test_wait_for_existing_ci_falls_back_to_parent() -> None:
    shas: list[str] = []

    def gh(args: list[str]) -> str:
        sha = args[args.index("--commit") + 1]
        shas.append(sha)
        if sha == "child":
            return "[]"
        return SUCCESS

    wait_for_existing_ci(
        "child",
        parent_sha="parent",
        gh=gh,
        sleep=lambda s: None,
    )
    assert shas == ["child", "parent"]
